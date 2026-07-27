use std::{fmt, io, sync::Mutex};

use sha2::{Digest, Sha256};

use crate::{
    ByteSpan, DxfAcadVersionReport, DxfAsciiGroup, DxfAsciiGroupCursor, DxfAsciiLineEnding,
    DxfAsciiStructureIndex, DxfByteSource, DxfCancellationToken, DxfDiagnostic, DxfDiagnosticCode,
    DxfError, DxfGroupCode, DxfHandseedReport, DxfIoOperation, DxfReadControl, DxfReadMode,
    DxfReadObserver, DxfReadOptions, DxfReadProgress, DxfSourceId, DxfTextEncodingReport,
    ascii_group::trim_horizontal_ascii, ascii_index::DxfAsciiStructureTracker,
    dialect::DxfAcadVersionTracker, encoding::DxfTextEncodingTracker, handseed::DxfHandseedTracker,
};

const HASH_CHUNK_BYTES: usize = 64 * 1024;
const PROGRESS_INTERVAL_BYTES: u64 = HASH_CHUNK_BYTES as u64;

/// Whether the raw document required a documented framing recovery.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAsciiDocumentConformance {
    Strict,
    Recovered,
}

/// Compact, owned location metadata for one source-backed ASCII group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfAsciiRawGroup {
    occurrence: u32,
    group_code: DxfGroupCode,
    group_code_content_span: ByteSpan,
    value_content_span: ByteSpan,
    full_span: ByteSpan,
    group_code_ending: DxfAsciiLineEnding,
    value_ending: DxfAsciiLineEnding,
}

impl DxfAsciiRawGroup {
    fn from_borrowed(group: DxfAsciiGroup<'_>) -> Result<Self, DxfError> {
        let occurrence = u32::try_from(group.occurrence()).map_err(|_| invalid_source_data())?;
        Ok(Self {
            occurrence,
            group_code: group.group_code(),
            group_code_content_span: group.group_code_line().content_span(),
            value_content_span: group.value_line().content_span(),
            full_span: group.full_span(),
            group_code_ending: group.group_code_line().ending(),
            value_ending: group.value_line().ending(),
        })
    }

    #[must_use]
    pub const fn occurrence(self) -> u64 {
        self.occurrence as u64
    }

    #[must_use]
    pub const fn group_code(self) -> DxfGroupCode {
        self.group_code
    }

    #[must_use]
    pub const fn group_code_content_span(self) -> ByteSpan {
        self.group_code_content_span
    }

    #[must_use]
    pub const fn value_content_span(self) -> ByteSpan {
        self.value_content_span
    }

    #[must_use]
    pub const fn full_span(self) -> ByteSpan {
        self.full_span
    }

    #[must_use]
    pub const fn group_code_ending(self) -> DxfAsciiLineEnding {
        self.group_code_ending
    }

    #[must_use]
    pub const fn value_ending(self) -> DxfAsciiLineEnding {
        self.value_ending
    }
}

/// Immutable, source-backed framing snapshot of one ASCII DXF byte stream.
pub struct DxfAsciiRawDocument<'a> {
    source: &'a dyn DxfByteSource,
    source_id: DxfSourceId,
    source_len: u64,
    groups: Box<[DxfAsciiRawGroup]>,
    diagnostics: Box<[DxfDiagnostic]>,
    diagnostics_truncated: bool,
    conformance: DxfAsciiDocumentConformance,
    acad_version: DxfAcadVersionReport,
    text_encoding: DxfTextEncodingReport,
    handseed: DxfHandseedReport,
    structure_index: DxfAsciiStructureIndex,
    eof_occurrence: Option<u64>,
    trailing_span: Option<ByteSpan>,
}

impl fmt::Debug for DxfAsciiRawDocument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfAsciiRawDocument")
            .field("source_id", &self.source_id)
            .field("source_len", &self.source_len)
            .field("groups", &self.groups.len())
            .field("diagnostics", &self.diagnostics.len())
            .field("diagnostics_truncated", &self.diagnostics_truncated)
            .field("conformance", &self.conformance)
            .field("acad_version", &self.acad_version.state())
            .field("text_encoding", &self.text_encoding.policy())
            .field("handseed", &self.handseed.state())
            .field("sections", &self.structure_index.sections().len())
            .field("eof_occurrence", &self.eof_occurrence)
            .field("trailing_span", &self.trailing_span)
            .finish()
    }
}

impl<'a> DxfAsciiRawDocument<'a> {
    pub fn open(
        source: &'a dyn DxfByteSource,
        options: DxfReadOptions,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let hashing_source = SequentialHashingSource::new(source);
        let mut cursor = DxfAsciiGroupCursor::new(&hashing_source, options)?;
        let source_len = cursor.source_len();
        notify_progress(observer, cancellation, 0, source_len)?;

        let limits = options.resource_profile().limits();
        let mut groups = Vec::new();
        let mut acad_version_tracker = DxfAcadVersionTracker::default();
        let mut text_encoding_tracker = DxfTextEncodingTracker::default();
        let mut handseed_tracker = DxfHandseedTracker::default();
        let mut structure_tracker = DxfAsciiStructureTracker::new(limits.max_diagnostics());
        let mut eof_occurrence = None;
        let mut eof_recovered = false;
        let mut last_reported = 0_u64;

        while let Some(group) = cursor.next_group(cancellation)? {
            let eof_match = classify_eof(group, options.mode());
            acad_version_tracker.observe(group)?;
            text_encoding_tracker.observe(group)?;
            handseed_tracker.observe(group)?;
            structure_tracker.observe(group, eof_match.is_some())?;
            let compact = DxfAsciiRawGroup::from_borrowed(group)?;
            groups.try_reserve(1).map_err(|_| out_of_memory())?;
            groups.push(compact);

            report_parse_progress(
                observer,
                cancellation,
                compact.full_span().end(),
                source_len,
                &mut last_reported,
            )?;
            if let Some(recovered) = eof_match {
                eof_occurrence = Some(compact.occurrence());
                eof_recovered = recovered;
                break;
            }
        }

        let consumed_bytes = cursor.consumed_bytes();
        let mut diagnostics = Vec::new();
        let mut diagnostics_truncated = cursor.diagnostics_were_truncated();
        for diagnostic in cursor.diagnostics() {
            retain_diagnostic(
                &mut diagnostics,
                limits.max_diagnostics(),
                &mut diagnostics_truncated,
                *diagnostic,
            )?;
        }
        drop(cursor);

        let mut trailing_span = None;
        if eof_occurrence.is_some() && consumed_bytes < source_len {
            let span = ByteSpan::new(consumed_bytes, source_len).ok_or_else(invalid_source_data)?;
            if options.mode() == DxfReadMode::Strict {
                return Err(DxfError::TrailingAsciiData { span });
            }
            trailing_span = Some(span);
            retain_diagnostic(
                &mut diagnostics,
                limits.max_diagnostics(),
                &mut diagnostics_truncated,
                DxfDiagnostic::new(DxfDiagnosticCode::ASCII_TRAILING_DATA_IGNORED, Some(span)),
            )?;
        }

        if eof_occurrence.is_none() {
            if options.mode() == DxfReadMode::Strict {
                return Err(DxfError::MissingAsciiEof {
                    at_offset: source_len,
                });
            }
            let span = ByteSpan::new(source_len, source_len).ok_or_else(invalid_source_data)?;
            retain_diagnostic(
                &mut diagnostics,
                limits.max_diagnostics(),
                &mut diagnostics_truncated,
                DxfDiagnostic::new(DxfDiagnosticCode::ASCII_EOF_MISSING_RECOVERED, Some(span)),
            )?;
        } else if eof_recovered {
            let eof_index = eof_occurrence.ok_or_else(invalid_source_data)?;
            let index = usize::try_from(eof_index).map_err(|_| invalid_source_data())?;
            let span = groups
                .get(index)
                .map(|group| group.value_content_span())
                .ok_or_else(invalid_source_data)?;
            retain_diagnostic(
                &mut diagnostics,
                limits.max_diagnostics(),
                &mut diagnostics_truncated,
                DxfDiagnostic::new(DxfDiagnosticCode::ASCII_EOF_WHITESPACE_IGNORED, Some(span)),
            )?;
        }

        let source_id =
            hashing_source.finalize(cancellation, observer, source_len, &mut last_reported)?;
        let acad_version = acad_version_tracker.finish(source_id)?;
        let text_encoding = text_encoding_tracker.finish(source_id, acad_version.state())?;
        let handseed = handseed_tracker.finish(source_id)?;
        let group_count = u64::try_from(groups.len()).map_err(|_| invalid_source_data())?;
        let structure_index = structure_tracker.finish(source_id, group_count)?;
        let conformance = if diagnostics.is_empty() {
            DxfAsciiDocumentConformance::Strict
        } else {
            DxfAsciiDocumentConformance::Recovered
        };

        Ok(Self {
            source,
            source_id,
            source_len,
            groups: groups.into_boxed_slice(),
            diagnostics: diagnostics.into_boxed_slice(),
            diagnostics_truncated,
            conformance,
            acad_version,
            text_encoding,
            handseed,
            structure_index,
            eof_occurrence,
            trailing_span,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_len(&self) -> u64 {
        self.source_len
    }

    #[must_use]
    pub fn groups(&self) -> &[DxfAsciiRawGroup] {
        &self.groups
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[DxfDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub const fn diagnostics_were_truncated(&self) -> bool {
        self.diagnostics_truncated
    }

    #[must_use]
    pub const fn conformance(&self) -> DxfAsciiDocumentConformance {
        self.conformance
    }

    #[must_use]
    pub const fn acad_version_report(&self) -> &DxfAcadVersionReport {
        &self.acad_version
    }

    #[must_use]
    pub const fn text_encoding_report(&self) -> &DxfTextEncodingReport {
        &self.text_encoding
    }

    #[must_use]
    pub const fn handseed_report(&self) -> &DxfHandseedReport {
        &self.handseed
    }

    #[must_use]
    pub const fn structure_index(&self) -> &DxfAsciiStructureIndex {
        &self.structure_index
    }

    #[must_use]
    pub const fn eof_occurrence(&self) -> Option<u64> {
        self.eof_occurrence
    }

    #[must_use]
    pub const fn trailing_span(&self) -> Option<ByteSpan> {
        self.trailing_span
    }

    pub fn read_span(&self, span: ByteSpan, destination: &mut [u8]) -> Result<(), DxfError> {
        let destination_len =
            u64::try_from(destination.len()).map_err(|_| invalid_source_data())?;
        if span.end() > self.source_len || span.len() != destination_len {
            return Err(invalid_source_data());
        }
        self.source.read_exact_at(span.start(), destination)
    }
}

fn classify_eof(group: DxfAsciiGroup<'_>, mode: DxfReadMode) -> Option<bool> {
    if group.group_code().value() != 0 {
        return None;
    }
    if group.raw_value() == b"EOF" {
        return Some(false);
    }
    if mode == DxfReadMode::Compatible && trim_horizontal_ascii(group.raw_value()) == Some(b"EOF") {
        return Some(true);
    }
    None
}

fn retain_diagnostic(
    diagnostics: &mut Vec<DxfDiagnostic>,
    max_diagnostics: u64,
    truncated: &mut bool,
    diagnostic: DxfDiagnostic,
) -> Result<(), DxfError> {
    let retained = u64::try_from(diagnostics.len()).map_err(|_| invalid_source_data())?;
    if retained < max_diagnostics {
        diagnostics.try_reserve(1).map_err(|_| out_of_memory())?;
        diagnostics.push(diagnostic);
    } else if !*truncated {
        if let Some(last) = diagnostics.last_mut() {
            *last = DxfDiagnostic::new(DxfDiagnosticCode::DIAGNOSTICS_TRUNCATED, diagnostic.span());
        }
        *truncated = true;
    }
    Ok(())
}

pub(crate) struct SequentialHashingSource<'a> {
    source: &'a dyn DxfByteSource,
    state: Mutex<HashState>,
}

struct HashState {
    next_offset: u64,
    hasher: Sha256,
}

impl<'a> SequentialHashingSource<'a> {
    pub(crate) fn new(source: &'a dyn DxfByteSource) -> Self {
        Self {
            source,
            state: Mutex::new(HashState {
                next_offset: 0,
                hasher: Sha256::new(),
            }),
        }
    }

    pub(crate) fn finalize(
        self,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
        source_len: u64,
        last_reported: &mut u64,
    ) -> Result<DxfSourceId, DxfError> {
        let mut state = match self.state.into_inner() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut buffer = [0_u8; HASH_CHUNK_BYTES];
        while state.next_offset < source_len {
            ensure_not_cancelled(cancellation)?;
            let remaining = source_len - state.next_offset;
            let requested_u64 = remaining.min(PROGRESS_INTERVAL_BYTES);
            let requested = usize::try_from(requested_u64).map_err(|_| invalid_source_data())?;
            let read = self
                .source
                .read_at(state.next_offset, &mut buffer[..requested])?;
            if read == 0 {
                return Err(source_io_error(io::ErrorKind::UnexpectedEof));
            }
            update_hash_state(&mut state, &buffer, requested, read, source_len)?;
            notify_progress(observer, cancellation, state.next_offset, source_len)?;
            *last_reported = state.next_offset;
        }
        if *last_reported != source_len {
            notify_progress(observer, cancellation, source_len, source_len)?;
            *last_reported = source_len;
        }
        let digest = state.hasher.finalize();
        let mut hash = [0_u8; DxfSourceId::BYTE_LEN];
        hash.copy_from_slice(&digest);
        Ok(DxfSourceId::from_sha256(hash))
    }
}

impl DxfByteSource for SequentialHashingSource<'_> {
    fn len(&self) -> u64 {
        self.source.len()
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        if offset != state.next_offset {
            return Err(invalid_source_data());
        }
        let read = self.source.read_at(offset, destination)?;
        update_hash_state(&mut state, destination, destination.len(), read, self.len())?;
        Ok(read)
    }
}

fn update_hash_state(
    state: &mut HashState,
    bytes: &[u8],
    requested: usize,
    read: usize,
    source_len: u64,
) -> Result<(), DxfError> {
    if read > requested {
        return Err(invalid_source_data());
    }
    let payload = bytes.get(..read).ok_or_else(invalid_source_data)?;
    let read_u64 = u64::try_from(read).map_err(|_| invalid_source_data())?;
    let next = state
        .next_offset
        .checked_add(read_u64)
        .ok_or(DxfError::OffsetOverflow {
            offset: state.next_offset,
            requested: read_u64,
        })?;
    if next > source_len {
        return Err(invalid_source_data());
    }
    state.hasher.update(payload);
    state.next_offset = next;
    Ok(())
}

pub(crate) fn report_parse_progress(
    observer: &mut dyn DxfReadObserver,
    cancellation: &DxfCancellationToken,
    processed: u64,
    total: u64,
    last_reported: &mut u64,
) -> Result<(), DxfError> {
    if processed.saturating_sub(*last_reported) >= PROGRESS_INTERVAL_BYTES {
        notify_progress(observer, cancellation, processed, total)?;
        *last_reported = processed;
    }
    Ok(())
}

pub(crate) fn notify_progress(
    observer: &mut dyn DxfReadObserver,
    cancellation: &DxfCancellationToken,
    processed: u64,
    total: u64,
) -> Result<(), DxfError> {
    ensure_not_cancelled(cancellation)?;
    let progress = DxfReadProgress::new(processed, total).ok_or_else(invalid_source_data)?;
    if observer.on_progress(progress) == DxfReadControl::Cancel {
        return Err(DxfError::Cancelled);
    }
    ensure_not_cancelled(cancellation)
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_source_data() -> DxfError {
    source_io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    source_io_error(io::ErrorKind::OutOfMemory)
}

fn source_io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs, io,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use sha2::{Digest, Sha256};

    use super::{DxfAsciiDocumentConformance, DxfAsciiRawDocument, HASH_CHUNK_BYTES};
    use crate::{
        ByteSpan, DxfByteSource, DxfCancellationToken, DxfDiagnosticCode, DxfError, DxfFileSource,
        DxfMemorySource, DxfReadControl, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
    };

    const BASE: &[u8] = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";
    static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn strict_document_is_source_backed_immutable_and_hashed_once() -> Result<(), Box<dyn Error>> {
        let source = CountingSource::new(BASE);
        let token = DxfCancellationToken::default();
        let mut progress = Vec::new();
        let mut observer = |value: crate::DxfReadProgress| {
            progress.push(value.processed_bytes());
            DxfReadControl::Continue
        };
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut observer)?;

        assert_eq!(document.conformance(), DxfAsciiDocumentConformance::Strict);
        assert_eq!(document.groups().len(), 4);
        assert_eq!(document.eof_occurrence(), Some(3));
        assert_eq!(document.trailing_span(), None);
        assert!(document.diagnostics().is_empty());
        assert_eq!(source.bytes_read(), BASE.len() as u64);
        assert_eq!(progress, [0, BASE.len() as u64]);
        assert_eq!(document.structure_index().total_group_count(), 4);
        assert_eq!(document.structure_index().inside_section_group_count(), 3);
        assert_eq!(document.structure_index().outside_section_group_count(), 1);
        assert_eq!(document.structure_index().zero_group_count(), 3);
        assert_eq!(
            document.text_encoding_report().policy(),
            crate::DxfTextEncodingPolicy::Indeterminate
        );
        assert_eq!(
            document.text_encoding_report().source_id(),
            document.source_id()
        );

        let expected = Sha256::digest(BASE);
        assert_eq!(
            document.source_id().as_bytes().as_slice(),
            expected.as_slice()
        );
        let eof = document
            .groups()
            .get(3)
            .ok_or(io::Error::other("missing EOF"))?;
        let mut raw = [0_u8; 3];
        document.read_span(eof.value_content_span(), &mut raw)?;
        assert_eq!(&raw, b"EOF");
        Ok(())
    }

    #[test]
    fn large_tail_is_streamed_and_hashed_exactly_once() -> Result<(), Box<dyn Error>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"0\nEOF\n");
        bytes.resize(HASH_CHUNK_BYTES * 3 + 17, b'x');
        let source = CountingSource::new(&bytes);
        let token = DxfCancellationToken::default();
        let mut progress = Vec::new();
        let mut observer = |value: crate::DxfReadProgress| {
            progress.push(value.processed_bytes());
            DxfReadControl::Continue
        };

        let document = DxfAsciiRawDocument::open(
            &source,
            DxfReadOptions::compatible(),
            &token,
            &mut observer,
        )?;
        assert_eq!(source.bytes_read(), bytes.len() as u64);
        assert_eq!(
            document.trailing_span(),
            ByteSpan::new(6, bytes.len() as u64)
        );
        assert_eq!(progress.first().copied(), Some(0));
        assert_eq!(progress.last().copied(), Some(bytes.len() as u64));
        assert!(progress.windows(2).all(|pair| pair[0] < pair[1]));

        let expected = Sha256::digest(&bytes);
        assert_eq!(
            document.source_id().as_bytes().as_slice(),
            expected.as_slice()
        );
        Ok(())
    }

    #[test]
    fn file_and_memory_documents_have_the_same_identity() -> Result<(), Box<dyn Error>> {
        let fixture = TestFile::new(BASE)?;
        let file_source = DxfFileSource::open(fixture.path(), DxfResourceProfile::Safe)?;
        let memory_source = DxfMemorySource::new(BASE, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let file_document =
            DxfAsciiRawDocument::open(&file_source, DxfReadOptions::strict(), &token, &mut noop)?;
        let memory_document =
            DxfAsciiRawDocument::open(&memory_source, DxfReadOptions::strict(), &token, &mut noop)?;

        assert_eq!(file_document.source_id(), memory_document.source_id());
        assert_eq!(file_document.groups(), memory_document.groups());
        Ok(())
    }

    #[test]
    fn strict_enforces_exact_terminal_eof() -> Result<(), DxfError> {
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        for bytes in [
            b"0\nSECTION\n0\nENDSEC\n".as_slice(),
            b"0\nSECTION\n0\neof\n".as_slice(),
            b"0\nSECTION\n0\n EOF\n".as_slice(),
        ] {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            assert!(matches!(
                DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop),
                Err(DxfError::MissingAsciiEof { .. })
            ));
        }

        let bytes = b"0\nEOF\n999\nafter\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        assert!(matches!(
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop),
            Err(DxfError::TrailingAsciiData { .. })
        ));
        Ok(())
    }

    #[test]
    fn strict_accepts_eof_without_final_newline_and_integer_code_spelling() -> Result<(), DxfError>
    {
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        for bytes in [b"0\nEOF".as_slice(), b"+0\r\nEOF\r\n".as_slice()] {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            let document =
                DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop)?;
            assert_eq!(document.eof_occurrence(), Some(0));
            assert_eq!(document.conformance(), DxfAsciiDocumentConformance::Strict);
        }
        Ok(())
    }

    #[test]
    fn raw_group_metadata_stays_compact() {
        assert!(std::mem::size_of::<crate::DxfAsciiRawGroup>() <= 56);
    }

    #[test]
    fn compatible_recoveries_are_visible_and_byte_preserving() -> Result<(), Box<dyn Error>> {
        let cases: [(&[u8], &[DxfDiagnosticCode], Option<ByteSpan>); 3] = [
            (
                b"0\nSECTION\n0\nENDSEC\n",
                &[DxfDiagnosticCode::ASCII_EOF_MISSING_RECOVERED],
                None,
            ),
            (
                b"0\n EOF \n",
                &[DxfDiagnosticCode::ASCII_EOF_WHITESPACE_IGNORED],
                None,
            ),
            (
                b"0\nEOF\n999\nafter\n",
                &[DxfDiagnosticCode::ASCII_TRAILING_DATA_IGNORED],
                ByteSpan::new(6, 16),
            ),
        ];
        let token = DxfCancellationToken::default();

        for (bytes, expected_codes, expected_tail) in cases {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            let mut noop = NoopDxfReadObserver;
            let document = DxfAsciiRawDocument::open(
                &source,
                DxfReadOptions::compatible(),
                &token,
                &mut noop,
            )?;
            let codes: Vec<_> = document
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code())
                .collect();
            assert_eq!(codes, expected_codes);
            assert_eq!(
                document.conformance(),
                DxfAsciiDocumentConformance::Recovered
            );
            assert_eq!(document.trailing_span(), expected_tail);
            assert_eq!(document.source_len(), bytes.len() as u64);
        }
        Ok(())
    }

    #[test]
    fn bom_and_eof_recoveries_accumulate_in_source_order() -> Result<(), Box<dyn Error>> {
        let bytes = b"\xef\xbb\xbf0\n EOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::compatible(), &token, &mut noop)?;
        let codes: Vec<_> = document
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code())
            .collect();
        assert_eq!(
            codes,
            [
                DxfDiagnosticCode::UTF8_BOM_IGNORED,
                DxfDiagnosticCode::ASCII_EOF_WHITESPACE_IGNORED,
            ]
        );
        Ok(())
    }

    #[test]
    fn cancellation_and_hostile_source_fail_closed() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(BASE, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut observer = |_progress: crate::DxfReadProgress| DxfReadControl::Cancel;
        assert!(matches!(
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut observer),
            Err(DxfError::Cancelled)
        ));

        let mut noop = NoopDxfReadObserver;
        assert!(
            DxfAsciiRawDocument::open(
                &OverReportingSource,
                DxfReadOptions::strict(),
                &DxfCancellationToken::default(),
                &mut noop,
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn raw_document_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DxfAsciiRawDocument<'static>>();
    }

    struct TestFile {
        path: PathBuf,
    }

    impl TestFile {
        fn new(bytes: &[u8]) -> io::Result<Self> {
            let id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "seacad-ascii-document-{}-{id}.dxf",
                std::process::id()
            ));
            fs::write(&path, bytes)?;
            Ok(Self { path })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    struct CountingSource<'a> {
        bytes: &'a [u8],
        bytes_read: AtomicU64,
    }

    impl<'a> CountingSource<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self {
                bytes,
                bytes_read: AtomicU64::new(0),
            }
        }

        fn bytes_read(&self) -> u64 {
            self.bytes_read.load(Ordering::Relaxed)
        }
    }

    impl DxfByteSource for CountingSource<'_> {
        fn len(&self) -> u64 {
            self.bytes.len() as u64
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            if start >= self.bytes.len() || destination.is_empty() {
                return Ok(0);
            }
            let count = (self.bytes.len() - start).min(destination.len());
            destination[..count].copy_from_slice(&self.bytes[start..start + count]);
            self.bytes_read.fetch_add(count as u64, Ordering::Relaxed);
            Ok(count)
        }
    }

    struct OverReportingSource;

    impl DxfByteSource for OverReportingSource {
        fn len(&self) -> u64 {
            8
        }

        fn read_at(&self, _offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            Ok(destination.len().saturating_add(1))
        }
    }
}

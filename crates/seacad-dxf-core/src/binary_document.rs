use std::{fmt, io};

use crate::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAcadVersionReport, DxfAcadVersionState,
    DxfBinaryGroup, DxfBinaryGroupCodeEncoding, DxfBinaryGroupCursor, DxfBinaryValueFamily,
    DxfByteSource, DxfCancellationToken, DxfError, DxfGroupCode, DxfIoOperation, DxfPhysicalFormat,
    DxfReadObserver, DxfReadOptions, DxfSourceId,
    ascii_document::{SequentialHashingSource, notify_progress, report_parse_progress},
    dialect::DxfAcadVersionTracker,
    probe_dxf_physical_format,
};

const ONE_BYTE_OPENING: [u8; 9] = *b"\0SECTION\0";
const TWO_BYTE_OPENING: [u8; 10] = *b"\0\0SECTION\0";
const GROUP_CHUNK_RECORDS: usize = 4 * 1024;

/// Compact, owned location metadata for one source-backed Binary DXF group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBinaryRawGroup {
    occurrence: u32,
    group_code: DxfGroupCode,
    value_family: DxfBinaryValueFamily,
    group_code_span: ByteSpan,
    full_span: ByteSpan,
}

impl DxfBinaryRawGroup {
    fn from_borrowed(group: DxfBinaryGroup<'_>) -> Result<Self, DxfError> {
        Ok(Self {
            occurrence: u32::try_from(group.occurrence).map_err(|_| invalid_source_data())?,
            group_code: group.group_code,
            value_family: group.value_family,
            group_code_span: group.group_code_span,
            full_span: group.full_span,
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
    pub const fn value_family(self) -> DxfBinaryValueFamily {
        self.value_family
    }

    #[must_use]
    pub const fn group_code_span(self) -> ByteSpan {
        self.group_code_span
    }

    #[must_use]
    pub const fn value_span(self) -> ByteSpan {
        ByteSpan::from_validated_bounds(self.group_code_span.end(), self.full_span.end())
    }

    #[must_use]
    pub const fn payload_span(self) -> ByteSpan {
        let value = self.value_span();
        match self.value_family {
            DxfBinaryValueFamily::NullTerminatedString => {
                ByteSpan::from_validated_bounds(value.start(), value.end() - 1)
            }
            DxfBinaryValueFamily::BinaryChunk => {
                ByteSpan::from_validated_bounds(value.start() + 1, value.end())
            }
            DxfBinaryValueFamily::F64LittleEndian
            | DxfBinaryValueFamily::I16LittleEndian
            | DxfBinaryValueFamily::I32LittleEndian
            | DxfBinaryValueFamily::I64LittleEndian
            | DxfBinaryValueFamily::BooleanByte => value,
        }
    }

    #[must_use]
    pub const fn full_span(self) -> ByteSpan {
        self.full_span
    }
}

/// Immutable source-backed framing snapshot of one encoding-verified Binary DXF stream.
pub struct DxfBinaryRawDocument<'a> {
    source: &'a dyn DxfByteSource,
    source_id: DxfSourceId,
    source_len: u64,
    group_code_encoding: DxfBinaryGroupCodeEncoding,
    groups: DxfBinaryRawGroupTable,
    acad_version: DxfAcadVersionReport,
}

impl fmt::Debug for DxfBinaryRawDocument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfBinaryRawDocument")
            .field("source_id", &self.source_id)
            .field("source_len", &self.source_len)
            .field("group_code_encoding", &self.group_code_encoding)
            .field("groups", &self.groups.len)
            .field("acad_version", &self.acad_version.state())
            .finish()
    }
}

impl<'a> DxfBinaryRawDocument<'a> {
    pub fn open(
        source: &'a dyn DxfByteSource,
        options: DxfReadOptions,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<Self, DxfError> {
        if cancellation.is_cancelled() {
            return Err(DxfError::Cancelled);
        }
        let group_code_encoding = detect_group_code_encoding(source, options)?;
        let hashing_source = SequentialHashingSource::new(source);
        let mut cursor = DxfBinaryGroupCursor::new(&hashing_source, group_code_encoding, options)?;
        let source_len = cursor.source_len();
        notify_progress(observer, cancellation, 0, source_len)?;

        let mut groups = DxfBinaryRawGroupTableBuilder::new();
        let mut tracker = DxfAcadVersionTracker::default();
        let mut last_reported = 0_u64;
        while let Some(group) = cursor.next_group(cancellation)? {
            tracker.observe_raw(
                group.occurrence,
                group.group_code,
                payload_bytes(group)?,
                group.payload_span,
            )?;
            let compact = DxfBinaryRawGroup::from_borrowed(group)?;
            groups.push(compact)?;
            report_parse_progress(
                observer,
                cancellation,
                compact.full_span().end(),
                source_len,
                &mut last_reported,
            )?;
        }
        drop(cursor);

        let source_id =
            hashing_source.finalize(cancellation, observer, source_len, &mut last_reported)?;
        let acad_version = tracker.finish(source_id)?;
        let (declared_version, version_span) = require_supported_version(&acad_version)?;
        if declared_version.binary_group_code_encoding() != group_code_encoding {
            return Err(DxfError::BinaryEncodingDialectMismatch {
                encoding: group_code_encoding,
                declared_version,
                span: version_span,
            });
        }

        Ok(Self {
            source,
            source_id,
            source_len,
            group_code_encoding,
            groups: groups.finish()?,
            acad_version,
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
    pub const fn group_code_encoding(&self) -> DxfBinaryGroupCodeEncoding {
        self.group_code_encoding
    }

    #[must_use]
    pub const fn group_count(&self) -> u64 {
        self.groups.len as u64
    }

    #[must_use]
    pub fn group(&self, occurrence: u64) -> Option<DxfBinaryRawGroup> {
        self.groups.get(occurrence).copied()
    }

    pub fn groups(&self) -> impl Iterator<Item = &DxfBinaryRawGroup> {
        self.groups.chunks.iter().flat_map(|chunk| chunk.iter())
    }

    #[must_use]
    pub const fn acad_version_report(&self) -> &DxfAcadVersionReport {
        &self.acad_version
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

struct DxfBinaryRawGroupTable {
    chunks: Box<[Box<[DxfBinaryRawGroup]>]>,
    len: u32,
}

impl DxfBinaryRawGroupTable {
    fn get(&self, occurrence: u64) -> Option<&DxfBinaryRawGroup> {
        let index = usize::try_from(occurrence).ok()?;
        if occurrence >= u64::from(self.len) {
            return None;
        }
        self.chunks
            .get(index / GROUP_CHUNK_RECORDS)?
            .get(index % GROUP_CHUNK_RECORDS)
    }
}

struct DxfBinaryRawGroupTableBuilder {
    chunks: Vec<Box<[DxfBinaryRawGroup]>>,
    current: Vec<DxfBinaryRawGroup>,
    len: u32,
}

impl DxfBinaryRawGroupTableBuilder {
    const fn new() -> Self {
        Self {
            chunks: Vec::new(),
            current: Vec::new(),
            len: 0,
        }
    }

    fn push(&mut self, group: DxfBinaryRawGroup) -> Result<(), DxfError> {
        if self.current.len() == GROUP_CHUNK_RECORDS {
            self.flush()?;
        }
        self.current.try_reserve(1).map_err(|_| out_of_memory())?;
        self.current.push(group);
        self.len = self.len.checked_add(1).ok_or_else(invalid_source_data)?;
        Ok(())
    }

    fn finish(mut self) -> Result<DxfBinaryRawGroupTable, DxfError> {
        self.flush()?;
        Ok(DxfBinaryRawGroupTable {
            chunks: self.chunks.into_boxed_slice(),
            len: self.len,
        })
    }

    fn flush(&mut self) -> Result<(), DxfError> {
        if self.current.is_empty() {
            return Ok(());
        }
        self.chunks.try_reserve(1).map_err(|_| out_of_memory())?;
        self.chunks
            .push(std::mem::take(&mut self.current).into_boxed_slice());
        Ok(())
    }
}

fn detect_group_code_encoding(
    source: &dyn DxfByteSource,
    options: DxfReadOptions,
) -> Result<DxfBinaryGroupCodeEncoding, DxfError> {
    if probe_dxf_physical_format(source, options.resource_profile())? != DxfPhysicalFormat::Binary {
        return Err(DxfError::InvalidBinarySentinel {
            span: span(0, source.len().min(DXF_BINARY_SENTINEL.len() as u64))?,
        });
    }
    let start = DXF_BINARY_SENTINEL.len() as u64;
    let observed_len = source
        .len()
        .saturating_sub(start)
        .min(TWO_BYTE_OPENING.len() as u64);
    let observed_len_usize = usize::try_from(observed_len).map_err(|_| invalid_source_data())?;
    let mut observed = [0_u8; TWO_BYTE_OPENING.len()];
    if observed_len_usize > 0 {
        source.read_exact_at(start, &mut observed[..observed_len_usize])?;
    }
    if observed_len_usize >= ONE_BYTE_OPENING.len()
        && observed[..ONE_BYTE_OPENING.len()] == ONE_BYTE_OPENING
    {
        return Ok(DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape);
    }
    if observed_len_usize == TWO_BYTE_OPENING.len() && observed == TWO_BYTE_OPENING {
        return Ok(DxfBinaryGroupCodeEncoding::TwoByteLittleEndian);
    }
    Err(DxfError::InvalidBinaryOpening {
        span: span(
            start,
            start
                .checked_add(observed_len)
                .ok_or(DxfError::OffsetOverflow {
                    offset: start,
                    requested: observed_len,
                })?,
        )?,
    })
}

fn payload_bytes(group: DxfBinaryGroup<'_>) -> Result<&[u8], DxfError> {
    let prefix = group
        .payload_span
        .start()
        .checked_sub(group.value_span.start())
        .ok_or_else(invalid_source_data)?;
    let start = usize::try_from(prefix).map_err(|_| invalid_source_data())?;
    let payload_len =
        usize::try_from(group.payload_span.len()).map_err(|_| invalid_source_data())?;
    let end = start
        .checked_add(payload_len)
        .ok_or_else(invalid_source_data)?;
    group
        .raw_value
        .get(start..end)
        .ok_or_else(invalid_source_data)
}

fn require_supported_version(
    report: &DxfAcadVersionReport,
) -> Result<(DxfAcadVersion, ByteSpan), DxfError> {
    let primary = report.primary_occurrence();
    let span = primary
        .and_then(|occurrence| occurrence.value_span())
        .or_else(|| primary.map(|occurrence| occurrence.variable_span()));
    match report.state() {
        DxfAcadVersionState::Supported(version) => {
            Ok((version, span.ok_or_else(invalid_source_data)?))
        }
        state => Err(DxfError::BinaryAcadVersionUnavailable { state, span }),
    }
}

fn span(start: u64, end: u64) -> Result<ByteSpan, DxfError> {
    ByteSpan::new(start, end).ok_or_else(invalid_source_data)
}

fn invalid_source_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs, io, path::PathBuf};

    use sha2::{Digest, Sha256};

    use super::{DxfBinaryRawDocument, DxfBinaryRawGroup};
    use crate::{
        DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAcadVersionState, DxfBinaryGroupCodeEncoding,
        DxfCancellationToken, DxfError, DxfErrorCode, DxfFileSource, DxfMemorySource,
        DxfReadControl, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
    };

    #[test]
    fn every_supported_version_selects_and_verifies_its_encoding() -> Result<(), Box<dyn Error>> {
        for version in DxfAcadVersion::SUPPORTED {
            let encoding = version.binary_group_code_encoding();
            let bytes = fixture(version.code().as_bytes(), encoding, 1, false);
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let mut progress = Vec::new();
            let mut observer = |value: crate::DxfReadProgress| {
                progress.push(value.processed_bytes());
                DxfReadControl::Continue
            };
            let document = DxfBinaryRawDocument::open(
                &source,
                DxfReadOptions::strict(),
                &DxfCancellationToken::default(),
                &mut observer,
            )?;

            assert_eq!(document.group_code_encoding(), encoding);
            assert_eq!(document.group_count(), 6);
            assert_eq!(
                document.acad_version_report().state(),
                DxfAcadVersionState::Supported(version)
            );
            assert_eq!(document.source_len(), bytes.len() as u64);
            assert_eq!(progress.first().copied(), Some(0));
            assert_eq!(progress.last().copied(), Some(bytes.len() as u64));
            assert!(progress.windows(2).all(|pair| pair[0] < pair[1]));
            assert_eq!(
                document.source_id().as_bytes().as_slice(),
                Sha256::digest(&bytes).as_slice()
            );
            assert_contiguous(&document, &bytes)?;
        }
        Ok(())
    }

    #[test]
    fn canonical_opening_is_required_without_compatible_guessing() -> Result<(), DxfError> {
        for opening in [b"section\0".as_slice(), b"HEADER\0".as_slice()] {
            let mut bytes = DXF_BINARY_SENTINEL.to_vec();
            bytes.push(0);
            bytes.extend_from_slice(opening);
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            for options in [DxfReadOptions::strict(), DxfReadOptions::compatible()] {
                let result = open(&source, options);
                assert!(matches!(result, Err(DxfError::InvalidBinaryOpening { .. })));
            }
        }
        Ok(())
    }

    #[test]
    fn physical_encoding_must_agree_with_declared_dialect() -> Result<(), Box<dyn Error>> {
        let cases = [
            (
                b"AC1015".as_slice(),
                DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape,
            ),
            (
                b"AC1009".as_slice(),
                DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            ),
        ];
        for (version, encoding) in cases {
            let bytes = fixture(version, encoding, 1, false);
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let error = open(&source, DxfReadOptions::strict())
                .err()
                .ok_or_else(|| io::Error::other("encoding mismatch was accepted"))?;
            assert_eq!(error.code(), DxfErrorCode::BINARY_ENCODING_DIALECT_MISMATCH);
        }
        Ok(())
    }

    #[test]
    fn missing_invalid_unsupported_and_duplicate_acadver_fail_typed() -> Result<(), Box<dyn Error>>
    {
        let encoding = DxfBinaryGroupCodeEncoding::TwoByteLittleEndian;
        let cases = [
            (b"AC1032".as_slice(), 0, false, DxfAcadVersionState::Absent),
            (
                b"AC1033".as_slice(),
                1,
                false,
                DxfAcadVersionState::Unsupported,
            ),
            (
                b"AC1032".as_slice(),
                70,
                false,
                DxfAcadVersionState::Invalid,
            ),
            (
                b"AC1032".as_slice(),
                1,
                true,
                DxfAcadVersionState::Ambiguous,
            ),
        ];
        for (version, value_code, duplicate, expected_state) in cases {
            let bytes = fixture(version, encoding, value_code, duplicate);
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let error = open(&source, DxfReadOptions::strict())
                .err()
                .ok_or_else(|| io::Error::other("unusable ACADVER was accepted"))?;
            assert!(matches!(
                error,
                DxfError::BinaryAcadVersionUnavailable { state, .. } if state == expected_state
            ));
        }
        Ok(())
    }

    #[test]
    fn file_and_memory_snapshots_match_and_document_is_send_sync() -> Result<(), Box<dyn Error>> {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DxfBinaryRawDocument<'static>>();
        assert!(std::mem::size_of::<DxfBinaryRawGroup>() <= 40);

        let bytes = fixture(
            b"AC1032",
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            1,
            false,
        );
        let path = temporary_path();
        fs::write(&path, &bytes)?;
        let file_source = DxfFileSource::open(&path, DxfResourceProfile::Safe)?;
        let memory_source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let file_document = open(&file_source, DxfReadOptions::strict())?;
        let memory_document = open(&memory_source, DxfReadOptions::strict())?;
        let _ = fs::remove_file(&path);

        assert_eq!(file_document.source_id(), memory_document.source_id());
        assert!(file_document.groups().eq(memory_document.groups()));
        Ok(())
    }

    #[test]
    fn compact_metadata_derives_every_value_shape_exactly() -> Result<(), Box<dyn Error>> {
        let encoding = DxfBinaryGroupCodeEncoding::TwoByteLittleEndian;
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        pair(&mut bytes, encoding, 0, b"SECTION\0");
        pair(&mut bytes, encoding, 2, b"HEADER\0");
        pair(&mut bytes, encoding, 9, b"$ACADVER\0");
        pair(&mut bytes, encoding, 1, b"AC1032\0");
        pair(&mut bytes, encoding, 10, &1_f64.to_le_bytes());
        pair(&mut bytes, encoding, 70, &2_i16.to_le_bytes());
        pair(&mut bytes, encoding, 90, &3_i32.to_le_bytes());
        pair(&mut bytes, encoding, 160, &4_i64.to_le_bytes());
        pair(&mut bytes, encoding, 290, b"\xff");
        pair(&mut bytes, encoding, 310, b"\x03abc");
        pair(&mut bytes, encoding, 0, b"ENDSEC\0");
        pair(&mut bytes, encoding, 0, b"EOF\0");
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;

        let expected = [
            (10, 8, 8),
            (70, 2, 2),
            (90, 4, 4),
            (160, 8, 8),
            (290, 1, 1),
            (310, 4, 3),
        ];
        for (code, value_len, payload_len) in expected {
            let group = document
                .groups()
                .find(|group| group.group_code().value() == code)
                .ok_or_else(|| io::Error::other("missing value-family group"))?;
            assert_eq!(group.group_code_span().len(), 2);
            assert_eq!(group.value_span().len(), value_len);
            assert_eq!(group.payload_span().len(), payload_len);
            assert_eq!(group.full_span().start(), group.group_code_span().start());
            assert_eq!(group.full_span().end(), group.value_span().end());
        }
        Ok(())
    }

    #[test]
    fn group_access_and_iteration_cross_chunk_boundaries() -> Result<(), Box<dyn Error>> {
        let encoding = DxfBinaryGroupCodeEncoding::TwoByteLittleEndian;
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        pair(&mut bytes, encoding, 0, b"SECTION\0");
        pair(&mut bytes, encoding, 2, b"HEADER\0");
        pair(&mut bytes, encoding, 9, b"$ACADVER\0");
        pair(&mut bytes, encoding, 1, b"AC1032\0");
        for _ in 0..4_100 {
            pair(&mut bytes, encoding, 290, b"\0");
        }
        pair(&mut bytes, encoding, 0, b"ENDSEC\0");
        pair(&mut bytes, encoding, 0, b"EOF\0");
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;

        assert_eq!(document.group_count(), 4_106);
        for occurrence in [4_095, 4_096, 4_103] {
            assert_eq!(
                document
                    .group(occurrence)
                    .map(|group| group.group_code().value()),
                Some(290)
            );
        }
        assert_eq!(document.groups().count(), 4_106);
        assert!(document.group(4_106).is_none());
        Ok(())
    }

    #[test]
    fn cancellation_and_truncated_payload_fail_closed() -> Result<(), DxfError> {
        let bytes = fixture(
            b"AC1032",
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            1,
            false,
        );
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let cancelled = DxfCancellationToken::default();
        cancelled.cancel();
        let mut noop = NoopDxfReadObserver;
        assert!(matches!(
            DxfBinaryRawDocument::open(&source, DxfReadOptions::strict(), &cancelled, &mut noop),
            Err(DxfError::Cancelled)
        ));

        let truncated = &bytes[..bytes.len() - 1];
        let source = DxfMemorySource::new(truncated, DxfResourceProfile::Safe)?;
        assert!(
            DxfBinaryRawDocument::open(
                &source,
                DxfReadOptions::strict(),
                &DxfCancellationToken::default(),
                &mut noop
            )
            .is_err()
        );
        Ok(())
    }

    fn fixture(
        version: &[u8],
        encoding: DxfBinaryGroupCodeEncoding,
        value_code: i16,
        duplicate: bool,
    ) -> Vec<u8> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        pair(&mut bytes, encoding, 0, b"SECTION\0");
        pair(&mut bytes, encoding, 2, b"HEADER\0");
        if value_code != 0 {
            pair(&mut bytes, encoding, 9, b"$ACADVER\0");
            if value_code == 1 {
                let mut value = version.to_vec();
                value.push(0);
                pair(&mut bytes, encoding, 1, &value);
            } else {
                pair(&mut bytes, encoding, value_code, &1032_i16.to_le_bytes());
            }
        }
        if duplicate {
            pair(&mut bytes, encoding, 9, b"$ACADVER\0");
            pair(&mut bytes, encoding, 1, b"AC1032\0");
        }
        pair(&mut bytes, encoding, 0, b"ENDSEC\0");
        pair(&mut bytes, encoding, 0, b"EOF\0");
        bytes
    }

    fn pair(bytes: &mut Vec<u8>, encoding: DxfBinaryGroupCodeEncoding, code: i16, value: &[u8]) {
        match encoding {
            DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape => bytes.push(code as u8),
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian => {
                bytes.extend_from_slice(&code.to_le_bytes());
            }
        }
        bytes.extend_from_slice(value);
    }

    fn open<'a>(
        source: &'a dyn crate::DxfByteSource,
        options: DxfReadOptions,
    ) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
        let mut noop = NoopDxfReadObserver;
        DxfBinaryRawDocument::open(source, options, &DxfCancellationToken::default(), &mut noop)
    }

    fn assert_contiguous(
        document: &DxfBinaryRawDocument<'_>,
        bytes: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let mut offset = DXF_BINARY_SENTINEL.len() as u64;
        for (occurrence, group) in document.groups().enumerate() {
            assert_eq!(group.occurrence(), occurrence as u64);
            assert_eq!(group.full_span().start(), offset);
            offset = group.full_span().end();
            let mut raw = vec![0_u8; group.full_span().len() as usize];
            document.read_span(group.full_span(), &mut raw)?;
            assert_eq!(
                raw,
                bytes[group.full_span().start() as usize..group.full_span().end() as usize]
            );
        }
        assert_eq!(offset, bytes.len() as u64);
        Ok(())
    }

    fn temporary_path() -> PathBuf {
        std::env::temp_dir().join(format!("seacad-binary-document-{}.dxf", std::process::id()))
    }
}

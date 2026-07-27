//! Borrowed format-neutral access to validated ASCII and Binary raw documents.

use std::fmt;

use crate::{
    ByteSpan, DxfAcadVersionReport, DxfAsciiRawDocument, DxfAsciiStructureIndex,
    DxfBinaryRawDocument, DxfError, DxfGroupCode, DxfHandseedReport, DxfHeaderVariable,
    DxfHeaderVariableIndex, DxfSourceId, DxfTextEncodingReport,
};

const EXACT_NAME_COMPARE_CHUNK: usize = 256;

/// Validated physical representation of an already opened raw document.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfRawDocumentFormat {
    Ascii,
    Binary,
}

/// Whether opening the document required a documented framing recovery.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfRawDocumentConformance {
    Strict,
    Recovered,
}

/// Exact lookup outcome for a raw HEADER variable name.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHeaderVariableLookupState {
    Absent,
    Unique,
    Ambiguous,
}

/// Ordered evidence retained for an exact raw HEADER variable name lookup.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderVariableLookup {
    source_id: DxfSourceId,
    state: DxfHeaderVariableLookupState,
    occurrence_count: u64,
    primary: Option<DxfHeaderVariable>,
    conflicting: Option<DxfHeaderVariable>,
}

impl DxfHeaderVariableLookup {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn state(self) -> DxfHeaderVariableLookupState {
        self.state
    }

    #[must_use]
    pub const fn occurrence_count(self) -> u64 {
        self.occurrence_count
    }

    #[must_use]
    pub const fn primary(self) -> Option<DxfHeaderVariable> {
        self.primary
    }

    #[must_use]
    pub const fn conflicting(self) -> Option<DxfHeaderVariable> {
        self.conflicting
    }
}

/// Format-neutral location metadata for one raw DXF group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfRawGroup {
    occurrence: u32,
    group_code: DxfGroupCode,
    value_payload_span: ByteSpan,
    full_span: ByteSpan,
}

impl DxfRawGroup {
    #[must_use]
    pub const fn occurrence(self) -> u64 {
        self.occurrence as u64
    }

    #[must_use]
    pub const fn group_code(self) -> DxfGroupCode {
        self.group_code
    }

    /// Exact ASCII line content or Binary payload, excluding wire delimiters.
    #[must_use]
    pub const fn value_payload_span(self) -> ByteSpan {
        self.value_payload_span
    }

    #[must_use]
    pub const fn full_span(self) -> ByteSpan {
        self.full_span
    }
}

trait DxfRawDocumentAdapter: Send + Sync {
    fn format(&self) -> DxfRawDocumentFormat;
    fn conformance(&self) -> DxfRawDocumentConformance;
    fn source_id(&self) -> DxfSourceId;
    fn source_len(&self) -> u64;
    fn group_count(&self) -> u64;
    fn group(&self, occurrence: u64) -> Option<DxfRawGroup>;
    fn read_span(&self, span: ByteSpan, destination: &mut [u8]) -> Result<(), DxfError>;
    fn acad_version_report(&self) -> &DxfAcadVersionReport;
    fn text_encoding_report(&self) -> &DxfTextEncodingReport;
    fn handseed_report(&self) -> &DxfHandseedReport;
    fn structure_index(&self) -> &DxfAsciiStructureIndex;
    fn header_variable_index(&self) -> &DxfHeaderVariableIndex;
}

/// Copyable borrowed adapter over either validated raw document representation.
#[derive(Clone, Copy)]
pub struct DxfRawDocumentView<'a> {
    document: &'a dyn DxfRawDocumentAdapter,
}

impl<'a> DxfRawDocumentView<'a> {
    #[must_use]
    pub fn from_ascii(document: &'a DxfAsciiRawDocument<'_>) -> Self {
        Self { document }
    }

    #[must_use]
    pub fn from_binary(document: &'a DxfBinaryRawDocument<'_>) -> Self {
        Self { document }
    }

    #[must_use]
    pub fn format(self) -> DxfRawDocumentFormat {
        self.document.format()
    }

    #[must_use]
    pub fn conformance(self) -> DxfRawDocumentConformance {
        self.document.conformance()
    }

    #[must_use]
    pub fn source_id(self) -> DxfSourceId {
        self.document.source_id()
    }

    #[must_use]
    pub fn source_len(self) -> u64 {
        self.document.source_len()
    }

    #[must_use]
    pub fn group_count(self) -> u64 {
        self.document.group_count()
    }

    #[must_use]
    pub fn group(self, occurrence: u64) -> Option<DxfRawGroup> {
        self.document.group(occurrence)
    }

    pub fn read_span(self, span: ByteSpan, destination: &mut [u8]) -> Result<(), DxfError> {
        self.document.read_span(span, destination)
    }

    #[must_use]
    pub fn acad_version_report(self) -> &'a DxfAcadVersionReport {
        self.document.acad_version_report()
    }

    #[must_use]
    pub fn text_encoding_report(self) -> &'a DxfTextEncodingReport {
        self.document.text_encoding_report()
    }

    #[must_use]
    pub fn handseed_report(self) -> &'a DxfHandseedReport {
        self.document.handseed_report()
    }

    #[must_use]
    pub fn structure_index(self) -> &'a DxfAsciiStructureIndex {
        self.document.structure_index()
    }

    #[must_use]
    pub fn header_variable_index(self) -> &'a DxfHeaderVariableIndex {
        self.document.header_variable_index()
    }

    /// Looks up a HEADER variable by exact raw bytes without decoding or case folding.
    pub fn lookup_header_variable(
        self,
        exact_name: &[u8],
    ) -> Result<DxfHeaderVariableLookup, DxfError> {
        lookup_header_variable_in_index(self, self.header_variable_index(), exact_name)
    }
}

fn lookup_header_variable_in_index(
    view: DxfRawDocumentView<'_>,
    index: &DxfHeaderVariableIndex,
    exact_name: &[u8],
) -> Result<DxfHeaderVariableLookup, DxfError> {
    if index.source_id() != view.source_id() {
        return Err(DxfError::SourceIdentityMismatch {
            expected: view.source_id(),
            observed: index.source_id(),
        });
    }

    let expected_hash = index.hash_name(exact_name);
    let mut occurrence_count = 0_u64;
    let mut primary = None;
    let mut conflicting = None;
    for (ordinal, variable) in index.variables().iter().copied().enumerate() {
        if index.variable_name_hash(ordinal) != Some(expected_hash)
            || !span_equals(view, variable.name_span(), exact_name)?
        {
            continue;
        }
        occurrence_count += 1;
        if primary.is_none() {
            primary = Some(variable);
        } else if conflicting.is_none() {
            conflicting = Some(variable);
        }
    }

    let state = match occurrence_count {
        0 => DxfHeaderVariableLookupState::Absent,
        1 => DxfHeaderVariableLookupState::Unique,
        _ => DxfHeaderVariableLookupState::Ambiguous,
    };
    Ok(DxfHeaderVariableLookup {
        source_id: view.source_id(),
        state,
        occurrence_count,
        primary,
        conflicting,
    })
}

fn span_equals(
    view: DxfRawDocumentView<'_>,
    span: ByteSpan,
    expected: &[u8],
) -> Result<bool, DxfError> {
    if span.len() != expected.len() as u64 {
        return Ok(false);
    }

    let mut buffer = [0_u8; EXACT_NAME_COMPARE_CHUNK];
    let mut consumed = 0_usize;
    while consumed < expected.len() {
        let chunk_len = (expected.len() - consumed).min(buffer.len());
        let source_start =
            span.start()
                .checked_add(consumed as u64)
                .ok_or(DxfError::OffsetOverflow {
                    offset: span.start(),
                    requested: consumed as u64,
                })?;
        let chunk_span = ByteSpan::from_start_and_len(source_start, chunk_len as u64).ok_or(
            DxfError::OffsetOverflow {
                offset: source_start,
                requested: chunk_len as u64,
            },
        )?;
        let expected_end = consumed
            .checked_add(chunk_len)
            .ok_or(DxfError::OffsetOverflow {
                offset: consumed as u64,
                requested: chunk_len as u64,
            })?;
        view.read_span(chunk_span, &mut buffer[..chunk_len])?;
        if buffer[..chunk_len] != expected[consumed..expected_end] {
            return Ok(false);
        }
        consumed = expected_end;
    }
    Ok(true)
}

impl fmt::Debug for DxfRawDocumentView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfRawDocumentView")
            .field("format", &self.format())
            .field("conformance", &self.conformance())
            .field("source_id", &self.source_id())
            .field("source_len", &self.source_len())
            .field("group_count", &self.group_count())
            .finish()
    }
}

impl<'a, 'source> From<&'a DxfAsciiRawDocument<'source>> for DxfRawDocumentView<'a> {
    fn from(document: &'a DxfAsciiRawDocument<'source>) -> Self {
        Self::from_ascii(document)
    }
}

impl<'a, 'source> From<&'a DxfBinaryRawDocument<'source>> for DxfRawDocumentView<'a> {
    fn from(document: &'a DxfBinaryRawDocument<'source>) -> Self {
        Self::from_binary(document)
    }
}

impl DxfRawDocumentAdapter for DxfAsciiRawDocument<'_> {
    fn format(&self) -> DxfRawDocumentFormat {
        DxfRawDocumentFormat::Ascii
    }

    fn conformance(&self) -> DxfRawDocumentConformance {
        match self.conformance() {
            crate::DxfAsciiDocumentConformance::Strict => DxfRawDocumentConformance::Strict,
            crate::DxfAsciiDocumentConformance::Recovered => DxfRawDocumentConformance::Recovered,
        }
    }

    fn source_id(&self) -> DxfSourceId {
        self.source_id()
    }

    fn source_len(&self) -> u64 {
        self.source_len()
    }

    fn group_count(&self) -> u64 {
        self.groups().len() as u64
    }

    fn group(&self, occurrence: u64) -> Option<DxfRawGroup> {
        let index = usize::try_from(occurrence).ok()?;
        let group = self.groups().get(index)?;
        Some(DxfRawGroup {
            occurrence: u32::try_from(group.occurrence()).ok()?,
            group_code: group.group_code(),
            value_payload_span: group.value_content_span(),
            full_span: group.full_span(),
        })
    }

    fn read_span(&self, span: ByteSpan, destination: &mut [u8]) -> Result<(), DxfError> {
        self.read_span(span, destination)
    }

    fn acad_version_report(&self) -> &DxfAcadVersionReport {
        self.acad_version_report()
    }

    fn text_encoding_report(&self) -> &DxfTextEncodingReport {
        self.text_encoding_report()
    }

    fn handseed_report(&self) -> &DxfHandseedReport {
        self.handseed_report()
    }

    fn structure_index(&self) -> &DxfAsciiStructureIndex {
        self.structure_index()
    }

    fn header_variable_index(&self) -> &DxfHeaderVariableIndex {
        self.header_variable_index()
    }
}

impl DxfRawDocumentAdapter for DxfBinaryRawDocument<'_> {
    fn format(&self) -> DxfRawDocumentFormat {
        DxfRawDocumentFormat::Binary
    }

    fn conformance(&self) -> DxfRawDocumentConformance {
        match self.conformance() {
            crate::DxfBinaryDocumentConformance::Strict => DxfRawDocumentConformance::Strict,
            crate::DxfBinaryDocumentConformance::Recovered => DxfRawDocumentConformance::Recovered,
        }
    }

    fn source_id(&self) -> DxfSourceId {
        self.source_id()
    }

    fn source_len(&self) -> u64 {
        self.source_len()
    }

    fn group_count(&self) -> u64 {
        self.group_count()
    }

    fn group(&self, occurrence: u64) -> Option<DxfRawGroup> {
        let group = self.group(occurrence)?;
        Some(DxfRawGroup {
            occurrence: u32::try_from(group.occurrence()).ok()?,
            group_code: group.group_code(),
            value_payload_span: group.payload_span(),
            full_span: group.full_span(),
        })
    }

    fn read_span(&self, span: ByteSpan, destination: &mut [u8]) -> Result<(), DxfError> {
        self.read_span(span, destination)
    }

    fn acad_version_report(&self) -> &DxfAcadVersionReport {
        self.acad_version_report()
    }

    fn text_encoding_report(&self) -> &DxfTextEncodingReport {
        self.text_encoding_report()
    }

    fn handseed_report(&self) -> &DxfHandseedReport {
        self.handseed_report()
    }

    fn structure_index(&self) -> &DxfAsciiStructureIndex {
        self.structure_index()
    }

    fn header_variable_index(&self) -> &DxfHeaderVariableIndex {
        self.header_variable_index()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        io,
        mem::size_of,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{
        DxfHeaderVariableLookup, DxfHeaderVariableLookupState, DxfRawDocumentConformance,
        DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup, lookup_header_variable_in_index,
    };
    use crate::{
        DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
        DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions,
        DxfResourceProfile, NoopDxfReadObserver,
    };

    const GROUP_CODES: [i16; 8] = [0, 2, 9, 1, 9, 1, 0, 0];

    #[test]
    fn every_supported_version_has_one_ascii_binary_contract() -> Result<(), Box<dyn Error>> {
        for version in DxfAcadVersion::SUPPORTED {
            let expected_values = [
                "SECTION",
                "HEADER",
                "$ACADVER",
                version.code(),
                "$CUSTOM",
                "PAYLOAD",
                "ENDSEC",
                "EOF",
            ];

            let ascii_bytes = ascii_fixture(version.code(), true);
            let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
            let ascii = open_ascii(&ascii_source, DxfReadOptions::strict())?;
            assert_view(
                DxfRawDocumentView::from(&ascii),
                DxfRawDocumentFormat::Ascii,
                &expected_values,
            )?;

            let binary_bytes = binary_fixture(version, true)?;
            let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
            let binary = open_binary(&binary_source, DxfReadOptions::strict())?;
            assert_view(
                DxfRawDocumentView::from(&binary),
                DxfRawDocumentFormat::Binary,
                &expected_values,
            )?;
        }
        Ok(())
    }

    #[test]
    fn compatible_recovery_and_span_errors_remain_format_neutral() -> Result<(), Box<dyn Error>> {
        let ascii_bytes = ascii_fixture("AC1032", false);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source, DxfReadOptions::compatible())?;

        let binary_bytes = binary_fixture(DxfAcadVersion::Ac1032, false)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source, DxfReadOptions::compatible())?;

        for view in [
            DxfRawDocumentView::from_ascii(&ascii),
            DxfRawDocumentView::from_binary(&binary),
        ] {
            assert_eq!(view.conformance(), DxfRawDocumentConformance::Recovered);
            assert_eq!(view.group_count(), 7);
            assert_eq!(view.header_variable_index().variables().len(), 2);
            assert_eq!(view.group(view.group_count()), None);
            assert_eq!(view.group(u64::MAX), None);

            let first = view.group(0).ok_or(io::Error::other("missing group"))?;
            let short_len = usize::try_from(first.value_payload_span().len() - 1)?;
            let mut short = vec![0_u8; short_len];
            assert!(
                view.read_span(first.value_payload_span(), &mut short)
                    .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn metadata_access_is_borrowed_and_performs_no_source_io() -> Result<(), Box<dyn Error>> {
        let bytes = ascii_fixture("AC1032", true);
        let source = CountingSource::new(&bytes);
        let document = open_ascii(&source, DxfReadOptions::strict())?;
        let reads_after_open = source.reads();
        let view = DxfRawDocumentView::from(&document);

        let _ = (
            view.format(),
            view.conformance(),
            view.source_id(),
            view.source_len(),
            view.group_count(),
            view.group(3),
            view.acad_version_report().state(),
            view.text_encoding_report().policy(),
            view.handseed_report().state(),
            view.structure_index().sections().len(),
            view.header_variable_index().variables().len(),
        );
        assert_eq!(source.reads(), reads_after_open);

        let missing = view.lookup_header_variable(b"$A_NAME_THAT_HAS_A_DIFFERENT_LENGTH")?;
        assert_eq!(missing.state(), DxfHeaderVariableLookupState::Absent);
        assert_eq!(source.reads(), reads_after_open);

        let custom = view.lookup_header_variable(b"$CUSTOM")?;
        assert_eq!(custom.state(), DxfHeaderVariableLookupState::Unique);
        assert!(source.reads() > reads_after_open);
        let reads_after_lookup = source.reads();

        let group = view.group(3).ok_or(io::Error::other("missing group"))?;
        let mut value = vec![0_u8; usize::try_from(group.value_payload_span().len())?];
        view.read_span(group.value_payload_span(), &mut value)?;
        assert_eq!(value, b"AC1032");
        assert!(source.reads() > reads_after_lookup);

        let debug = format!("{view:?}");
        assert!(debug.contains("DxfRawDocumentView"));
        assert!(!debug.contains("AC1032"));
        assert!(!debug.contains("PAYLOAD"));
        Ok(())
    }

    #[test]
    fn duplicate_case_and_empty_names_are_exact_for_ascii_and_binary() -> Result<(), Box<dyn Error>>
    {
        let ascii_bytes = exact_lookup_ascii_fixture();
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source, DxfReadOptions::strict())?;

        let binary_bytes = exact_lookup_binary_fixture(DxfAcadVersion::Ac1032)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source, DxfReadOptions::strict())?;

        for view in [
            DxfRawDocumentView::from(&ascii),
            DxfRawDocumentView::from(&binary),
        ] {
            let target = view.lookup_header_variable(b"$TARGET")?;
            assert_lookup(
                target,
                DxfHeaderVariableLookupState::Ambiguous,
                2,
                4,
                Some(8),
            );
            assert_lookup(
                view.lookup_header_variable(b"$target")?,
                DxfHeaderVariableLookupState::Unique,
                1,
                6,
                None,
            );
            assert_lookup(
                view.lookup_header_variable(b"")?,
                DxfHeaderVariableLookupState::Unique,
                1,
                10,
                None,
            );
            assert_lookup(
                view.lookup_header_variable(b"$MISSING")?,
                DxfHeaderVariableLookupState::Absent,
                0,
                0,
                None,
            );
        }
        Ok(())
    }

    #[test]
    fn long_names_are_compared_in_bounded_chunks() -> Result<(), Box<dyn Error>> {
        let mut exact_name = vec![b'$'];
        exact_name.extend(std::iter::repeat_n(b'A', 1_024));
        let mut bytes = b"0\nSECTION\n2\nHEADER\n9\n".to_vec();
        bytes.extend_from_slice(&exact_name);
        bytes.extend_from_slice(b"\n1\nVALUE\n0\nENDSEC\n0\nEOF\n");
        let source = CountingSource::new(&bytes);
        let document = open_ascii(&source, DxfReadOptions::strict())?;
        let view = DxfRawDocumentView::from(&document);
        let reads_after_open = source.reads();

        let found = view.lookup_header_variable(&exact_name)?;
        assert_eq!(found.state(), DxfHeaderVariableLookupState::Unique);
        assert!(source.reads() >= reads_after_open + 5);

        let mut near_name = exact_name;
        let Some(last) = near_name.last_mut() else {
            return Err(io::Error::other("missing name byte").into());
        };
        *last = b'B';
        assert_eq!(
            view.lookup_header_variable(&near_name)?.state(),
            DxfHeaderVariableLookupState::Absent
        );
        Ok(())
    }

    #[test]
    fn fingerprint_collision_still_requires_exact_source_bytes() -> Result<(), Box<dyn Error>> {
        let bytes = ascii_fixture("AC1032", true);
        let source = CountingSource::new(&bytes);
        let document = open_ascii(&source, DxfReadOptions::strict())?;
        let view = DxfRawDocumentView::from(&document);
        let index = view.header_variable_index();
        let forced = index
            .with_name_hash_for_test(1, index.hash_name(b"$CUSTOX"))
            .ok_or(io::Error::other("missing fingerprint"))?;
        let reads_after_open = source.reads();

        let lookup = lookup_header_variable_in_index(view, &forced, b"$CUSTOX")?;
        assert_eq!(lookup.state(), DxfHeaderVariableLookupState::Absent);
        assert!(source.reads() > reads_after_open);
        Ok(())
    }

    #[test]
    fn public_adapter_metadata_is_compact_copy_send_and_sync() {
        assert_copy::<DxfRawDocumentView<'static>>();
        assert_copy::<DxfRawGroup>();
        assert_copy::<DxfRawDocumentFormat>();
        assert_copy::<DxfRawDocumentConformance>();
        assert_copy::<DxfHeaderVariableLookup>();
        assert_copy::<DxfHeaderVariableLookupState>();
        assert_send_sync::<DxfRawDocumentView<'static>>();
        assert_send_sync::<DxfRawGroup>();
        assert_send_sync::<DxfHeaderVariableLookup>();
        assert!(size_of::<DxfRawDocumentView<'static>>() <= 2 * size_of::<usize>());
        assert!(size_of::<DxfRawGroup>() <= 48);
        assert!(size_of::<DxfHeaderVariableLookup>() <= 128);
    }

    fn assert_view(
        view: DxfRawDocumentView<'_>,
        expected_format: DxfRawDocumentFormat,
        expected_values: &[&str; 8],
    ) -> Result<(), Box<dyn Error>> {
        assert_eq!(view.format(), expected_format);
        assert_eq!(view.conformance(), DxfRawDocumentConformance::Strict);
        assert_eq!(view.group_count(), GROUP_CODES.len() as u64);
        assert_eq!(view.acad_version_report().source_id(), view.source_id());
        assert_eq!(view.text_encoding_report().source_id(), view.source_id());
        assert_eq!(view.handseed_report().source_id(), view.source_id());
        assert_eq!(view.structure_index().source_id(), view.source_id());
        assert_eq!(view.header_variable_index().source_id(), view.source_id());
        assert_eq!(view.header_variable_index().variables().len(), 2);
        assert_lookup(
            view.lookup_header_variable(b"$CUSTOM")?,
            DxfHeaderVariableLookupState::Unique,
            1,
            4,
            None,
        );

        let mut previous_end = 0_u64;
        for (occurrence, (expected_code, expected_value)) in
            GROUP_CODES.iter().zip(expected_values).enumerate()
        {
            let occurrence = u64::try_from(occurrence)?;
            let group = view
                .group(occurrence)
                .ok_or(io::Error::other("missing group"))?;
            assert_eq!(group.occurrence(), occurrence);
            assert_eq!(group.group_code().value(), *expected_code);
            assert!(group.full_span().start() >= previous_end);
            assert!(
                group.value_payload_span().start() >= group.full_span().start()
                    && group.value_payload_span().end() <= group.full_span().end()
            );

            let mut value = vec![0_u8; usize::try_from(group.value_payload_span().len())?];
            view.read_span(group.value_payload_span(), &mut value)?;
            assert_eq!(value, expected_value.as_bytes());
            previous_end = group.full_span().end();
        }
        assert_eq!(previous_end, view.source_len());
        assert_eq!(view.group(view.group_count()), None);
        Ok(())
    }

    fn assert_lookup(
        lookup: DxfHeaderVariableLookup,
        state: DxfHeaderVariableLookupState,
        count: u64,
        primary_occurrence: u64,
        conflicting_occurrence: Option<u64>,
    ) {
        assert_eq!(lookup.state(), state);
        assert_eq!(lookup.occurrence_count(), count);
        assert_eq!(
            lookup
                .primary()
                .map(|variable| variable.marker_occurrence()),
            (count != 0).then_some(primary_occurrence)
        );
        assert_eq!(
            lookup
                .conflicting()
                .map(|variable| variable.marker_occurrence()),
            conflicting_occurrence
        );
    }

    fn ascii_fixture(version: &str, include_eof: bool) -> Vec<u8> {
        let eof = if include_eof { "0\nEOF\n" } else { "" };
        format!(
            "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n9\n$CUSTOM\n1\nPAYLOAD\n0\nENDSEC\n{eof}"
        )
        .into_bytes()
    }

    fn binary_fixture(version: DxfAcadVersion, include_eof: bool) -> Result<Vec<u8>, io::Error> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        for (code, value) in [
            (0_i16, "SECTION"),
            (2, "HEADER"),
            (9, "$ACADVER"),
            (1, version.code()),
            (9, "$CUSTOM"),
            (1, "PAYLOAD"),
            (0, "ENDSEC"),
        ] {
            push_binary_string(&mut bytes, version, code, value.as_bytes())?;
        }
        if include_eof {
            push_binary_string(&mut bytes, version, 0, b"EOF")?;
        }
        Ok(bytes)
    }

    fn exact_lookup_ascii_fixture() -> Vec<u8> {
        b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$TARGET\n1\nA\n9\n$target\n1\nB\n9\n$TARGET\n1\nC\n9\n\n1\nD\n0\nENDSEC\n0\nEOF\n"
            .to_vec()
    }

    fn exact_lookup_binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        for (code, value) in [
            (0_i16, b"SECTION".as_slice()),
            (2, b"HEADER"),
            (9, b"$ACADVER"),
            (1, version.code().as_bytes()),
            (9, b"$TARGET"),
            (1, b"A"),
            (9, b"$target"),
            (1, b"B"),
            (9, b"$TARGET"),
            (1, b"C"),
            (9, b""),
            (1, b"D"),
            (0, b"ENDSEC"),
            (0, b"EOF"),
        ] {
            push_binary_string(&mut bytes, version, code, value)?;
        }
        Ok(bytes)
    }

    fn push_binary_string(
        bytes: &mut Vec<u8>,
        version: DxfAcadVersion,
        group_code: i16,
        value: &[u8],
    ) -> Result<(), io::Error> {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
        } else {
            bytes.extend_from_slice(&group_code.to_le_bytes());
        }
        bytes.extend_from_slice(value);
        bytes.push(0);
        Ok(())
    }

    fn open_ascii<'a>(
        source: &'a dyn DxfByteSource,
        options: DxfReadOptions,
    ) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
        let mut observer = NoopDxfReadObserver;
        DxfAsciiRawDocument::open(
            source,
            options,
            &DxfCancellationToken::default(),
            &mut observer,
        )
    }

    fn open_binary<'a>(
        source: &'a dyn DxfByteSource,
        options: DxfReadOptions,
    ) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
        let mut observer = NoopDxfReadObserver;
        DxfBinaryRawDocument::open(
            source,
            options,
            &DxfCancellationToken::default(),
            &mut observer,
        )
    }

    struct CountingSource<'a> {
        bytes: &'a [u8],
        reads: AtomicU64,
    }

    impl<'a> CountingSource<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self {
                bytes,
                reads: AtomicU64::new(0),
            }
        }

        fn reads(&self) -> u64 {
            self.reads.load(Ordering::Relaxed)
        }
    }

    impl DxfByteSource for CountingSource<'_> {
        fn len(&self) -> u64 {
            self.bytes.len() as u64
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: destination.len() as u64,
            })?;
            let Some(available) = self.bytes.get(start..) else {
                return Ok(0);
            };
            let count = available.len().min(destination.len());
            destination[..count].copy_from_slice(&available[..count]);
            Ok(count)
        }
    }

    fn assert_copy<T: Copy>() {}
    fn assert_send_sync<T: Send + Sync>() {}
}

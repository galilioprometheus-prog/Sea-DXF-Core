//! Borrowed format-neutral access to validated ASCII and Binary raw documents.

use std::fmt;

use crate::{
    ByteSpan, DxfAcadVersionReport, DxfAsciiRawDocument, DxfAsciiStructureIndex,
    DxfBinaryRawDocument, DxfError, DxfGroupCode, DxfHandseedReport, DxfHeaderVariableIndex,
    DxfSourceId, DxfTextEncodingReport,
};

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

    use super::{DxfRawDocumentConformance, DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup};
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

        let group = view.group(3).ok_or(io::Error::other("missing group"))?;
        let mut value = vec![0_u8; usize::try_from(group.value_payload_span().len())?];
        view.read_span(group.value_payload_span(), &mut value)?;
        assert_eq!(value, b"AC1032");
        assert!(source.reads() > reads_after_open);

        let debug = format!("{view:?}");
        assert!(debug.contains("DxfRawDocumentView"));
        assert!(!debug.contains("AC1032"));
        assert!(!debug.contains("PAYLOAD"));
        Ok(())
    }

    #[test]
    fn public_adapter_metadata_is_compact_copy_send_and_sync() {
        assert_copy::<DxfRawDocumentView<'static>>();
        assert_copy::<DxfRawGroup>();
        assert_copy::<DxfRawDocumentFormat>();
        assert_copy::<DxfRawDocumentConformance>();
        assert_send_sync::<DxfRawDocumentView<'static>>();
        assert_send_sync::<DxfRawGroup>();
        assert!(size_of::<DxfRawDocumentView<'static>>() <= 2 * size_of::<usize>());
        assert!(size_of::<DxfRawGroup>() <= 48);
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

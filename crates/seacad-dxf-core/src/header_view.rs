use std::{io, num::NonZeroU64};

use crate::{
    DxfAcadVersion, DxfAcadVersionOccurrence, DxfAcadVersionReport, DxfAcadVersionState,
    DxfAcadVersionValue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfError, DxfGroupCode,
    DxfIoOperation, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfSourceId,
    generated::header_schema::{DxfSchemaStorageKind, HEADER_FIELDS},
};

const HEADER_NAMESPACE: &str = "header";

/// Exact reason why `$ACADVER` has no usable typed value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAcadVersionIssue {
    UnsupportedValue,
    InvalidGroupCode(DxfGroupCode),
    MissingValue,
    MultipleValues { occurrence_count: NonZeroU64 },
}

/// Fixed-size read-only projection of reviewed HEADER semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderView {
    acad_version: DxfSemanticValue<DxfAcadVersion, DxfAcadVersionIssue>,
}

impl DxfHeaderView {
    fn from_report(report: &DxfAcadVersionReport) -> Result<Self, DxfError> {
        let field = acad_version_field_provenance(report.source_id())?;
        let acad_version = match report.state() {
            DxfAcadVersionState::Supported(version) => {
                let occurrence = report
                    .primary_occurrence()
                    .ok_or_else(invalid_internal_data)?;
                DxfSemanticValue::explicit(version, field, required_value_provenance(occurrence)?)
            }
            DxfAcadVersionState::Absent => DxfSemanticValue::absent(field),
            DxfAcadVersionState::Unsupported => DxfSemanticValue::invalid(
                DxfAcadVersionIssue::UnsupportedValue,
                field,
                optional_evidence(report.primary_occurrence())?,
            ),
            DxfAcadVersionState::Invalid => {
                let occurrence = report.primary_occurrence();
                DxfSemanticValue::invalid(
                    invalid_issue(occurrence)?,
                    field,
                    optional_evidence(occurrence)?,
                )
            }
            DxfAcadVersionState::Ambiguous => {
                let occurrence_count =
                    NonZeroU64::new(report.occurrence_count()).ok_or_else(invalid_internal_data)?;
                let evidence = report
                    .conflicting_occurrence()
                    .or_else(|| report.primary_occurrence());
                DxfSemanticValue::invalid(
                    DxfAcadVersionIssue::MultipleValues { occurrence_count },
                    field,
                    optional_evidence(evidence)?,
                )
            }
        };
        Ok(Self { acad_version })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.acad_version.field_provenance().document_source_id()
    }

    #[must_use]
    pub const fn acad_version(&self) -> &DxfSemanticValue<DxfAcadVersion, DxfAcadVersionIssue> {
        &self.acad_version
    }
}

impl DxfAsciiRawDocument<'_> {
    /// Projects already-indexed `$ACADVER` evidence without reading the source again.
    pub fn header_view(&self) -> Result<DxfHeaderView, DxfError> {
        DxfHeaderView::from_report(self.acad_version_report())
    }
}

impl DxfBinaryRawDocument<'_> {
    /// Projects already-indexed `$ACADVER` evidence without reading the source again.
    pub fn header_view(&self) -> Result<DxfHeaderView, DxfError> {
        DxfHeaderView::from_report(self.acad_version_report())
    }
}

fn acad_version_field_provenance(
    source_id: DxfSourceId,
) -> Result<DxfSemanticFieldProvenance, DxfError> {
    let field = HEADER_FIELDS
        .iter()
        .find(|field| {
            field.id == "acadver"
                && field.dxf_name == "$ACADVER"
                && field.group_codes == [1]
                && field.storage == DxfSchemaStorageKind::ExactText
        })
        .ok_or_else(invalid_internal_data)?;
    Ok(DxfSemanticFieldProvenance::new(
        source_id,
        HEADER_NAMESPACE,
        field.id,
    ))
}

fn invalid_issue(
    occurrence: Option<DxfAcadVersionOccurrence>,
) -> Result<DxfAcadVersionIssue, DxfError> {
    match occurrence.map(DxfAcadVersionOccurrence::value) {
        Some(DxfAcadVersionValue::InvalidGroupCode(group_code)) => {
            Ok(DxfAcadVersionIssue::InvalidGroupCode(group_code))
        }
        Some(DxfAcadVersionValue::MissingValue) | None => Ok(DxfAcadVersionIssue::MissingValue),
        Some(DxfAcadVersionValue::Supported(_) | DxfAcadVersionValue::Unsupported) => {
            Err(invalid_internal_data())
        }
    }
}

fn optional_evidence(
    occurrence: Option<DxfAcadVersionOccurrence>,
) -> Result<Option<DxfRawValueProvenance>, DxfError> {
    occurrence.map(raw_evidence).transpose()
}

fn required_value_provenance(
    occurrence: DxfAcadVersionOccurrence,
) -> Result<DxfRawValueProvenance, DxfError> {
    let group_occurrence = occurrence
        .value_occurrence()
        .ok_or_else(invalid_internal_data)?;
    let value_span = occurrence.value_span().ok_or_else(invalid_internal_data)?;
    DxfRawValueProvenance::new(group_occurrence, value_span).ok_or_else(invalid_internal_data)
}

fn raw_evidence(occurrence: DxfAcadVersionOccurrence) -> Result<DxfRawValueProvenance, DxfError> {
    match (occurrence.value_occurrence(), occurrence.value_span()) {
        (Some(group_occurrence), Some(value_span)) => {
            DxfRawValueProvenance::new(group_occurrence, value_span)
                .ok_or_else(invalid_internal_data)
        }
        (None, None) => {
            DxfRawValueProvenance::new(occurrence.variable_occurrence(), occurrence.variable_span())
                .ok_or_else(invalid_internal_data)
        }
        _ => Err(invalid_internal_data()),
    }
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::DxfAcadVersionIssue;
    use crate::{
        DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
        DxfCancellationToken, DxfGroupCode, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
        DxfSemanticValueState, NoopDxfReadObserver,
    };

    #[test]
    fn every_supported_version_has_ascii_binary_semantic_parity() -> Result<(), Box<dyn Error>> {
        for version in DxfAcadVersion::SUPPORTED {
            let ascii_bytes = ascii_fixture(version.code());
            let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
            let ascii = open_ascii(&ascii_source, DxfReadOptions::strict())?;
            let ascii_view = ascii.header_view()?;

            let binary_bytes = binary_fixture(version)?;
            let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
            let binary = open_binary(&binary_source, DxfReadOptions::strict())?;
            let binary_view = binary.header_view()?;

            for (view, expected_source) in [
                (&ascii_view, ascii.source_id()),
                (&binary_view, binary.source_id()),
            ] {
                assert_eq!(view.source_id(), expected_source);
                assert_eq!(view.acad_version().state(), DxfSemanticValueState::Explicit);
                assert_eq!(view.acad_version().value(), Some(&version));
                let field = view.acad_version().field_provenance();
                assert_eq!(field.schema_namespace(), "header");
                assert_eq!(field.schema_field_id(), "acadver");
            }

            assert_raw_value(&ascii, &ascii_view, version.code().as_bytes())?;
            assert_raw_value(&binary, &binary_view, version.code().as_bytes())?;
        }
        Ok(())
    }

    #[test]
    fn absent_unsupported_and_malformed_are_distinct() -> Result<(), Box<dyn Error>> {
        let absent_bytes = b"0\nSECTION\n2\nHEADER\n0\nENDSEC\n0\nEOF\n";
        let absent_source = DxfMemorySource::new(absent_bytes, DxfResourceProfile::Safe)?;
        let absent = open_ascii(&absent_source, DxfReadOptions::strict())?;
        let absent_view = absent.header_view()?;
        assert_eq!(
            absent_view.acad_version().state(),
            DxfSemanticValueState::Absent
        );
        assert_eq!(absent_view.acad_version().raw_provenance(), None);

        let unsupported_bytes = ascii_fixture("AC1006");
        let unsupported_source =
            DxfMemorySource::new(&unsupported_bytes, DxfResourceProfile::Safe)?;
        let unsupported = open_ascii(&unsupported_source, DxfReadOptions::strict())?;
        let unsupported_view = unsupported.header_view()?;
        assert_eq!(
            unsupported_view.acad_version().invalid_issue(),
            Some(&DxfAcadVersionIssue::UnsupportedValue)
        );
        assert_raw_value(&unsupported, &unsupported_view, b"AC1006")?;

        let wrong_group_bytes =
            b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n3\nAC1018\n0\nENDSEC\n0\nEOF\n";
        let wrong_group_source = DxfMemorySource::new(wrong_group_bytes, DxfResourceProfile::Safe)?;
        let wrong_group = open_ascii(&wrong_group_source, DxfReadOptions::strict())?;
        let wrong_group_view = wrong_group.header_view()?;
        let code_3 = DxfGroupCode::new(3).ok_or(io::Error::other("group code"))?;
        assert_eq!(
            wrong_group_view.acad_version().invalid_issue(),
            Some(&DxfAcadVersionIssue::InvalidGroupCode(code_3))
        );
        assert_raw_value(&wrong_group, &wrong_group_view, b"AC1018")?;
        Ok(())
    }

    #[test]
    fn missing_and_duplicates_keep_the_best_exact_evidence() -> Result<(), Box<dyn Error>> {
        let missing_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n";
        let missing_source = DxfMemorySource::new(missing_bytes, DxfResourceProfile::Safe)?;
        let missing = open_ascii(&missing_source, DxfReadOptions::compatible())?;
        let missing_view = missing.header_view()?;
        assert_eq!(
            missing_view.acad_version().invalid_issue(),
            Some(&DxfAcadVersionIssue::MissingValue)
        );
        assert_raw_value(&missing, &missing_view, b"$ACADVER")?;

        let duplicate_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1018\n9\n$ACADVER\n1\nAC1021\n0\nENDSEC\n0\nEOF\n";
        let duplicate_source = DxfMemorySource::new(duplicate_bytes, DxfResourceProfile::Safe)?;
        let duplicate = open_ascii(&duplicate_source, DxfReadOptions::strict())?;
        let duplicate_view = duplicate.header_view()?;
        let issue = duplicate_view
            .acad_version()
            .invalid_issue()
            .ok_or(io::Error::other("missing issue"))?;
        assert!(matches!(
            issue,
            DxfAcadVersionIssue::MultipleValues { occurrence_count }
                if occurrence_count.get() == 2
        ));
        assert_raw_value(&duplicate, &duplicate_view, b"AC1021")?;
        Ok(())
    }

    fn ascii_fixture(version: &str) -> Vec<u8> {
        format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nEOF\n")
            .into_bytes()
    }

    fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        for (group_code, value) in [
            (0_i16, "SECTION"),
            (2, "HEADER"),
            (9, "$ACADVER"),
            (1, version.code()),
            (0, "ENDSEC"),
            (0, "EOF"),
        ] {
            push_binary_group(&mut bytes, version, group_code, value.as_bytes())?;
        }
        Ok(bytes)
    }

    fn push_binary_group(
        bytes: &mut Vec<u8>,
        version: DxfAcadVersion,
        group_code: i16,
        value: &[u8],
    ) -> Result<(), io::Error> {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code width"))?);
        } else {
            bytes.extend_from_slice(&group_code.to_le_bytes());
        }
        bytes.extend_from_slice(value);
        bytes.push(0);
        Ok(())
    }

    fn open_ascii<'a>(
        source: &'a DxfMemorySource<'_>,
        options: DxfReadOptions,
    ) -> Result<DxfAsciiRawDocument<'a>, crate::DxfError> {
        let cancellation = DxfCancellationToken::default();
        let mut observer = NoopDxfReadObserver;
        DxfAsciiRawDocument::open(source, options, &cancellation, &mut observer)
    }

    fn open_binary<'a>(
        source: &'a DxfMemorySource<'_>,
        options: DxfReadOptions,
    ) -> Result<DxfBinaryRawDocument<'a>, crate::DxfError> {
        let cancellation = DxfCancellationToken::default();
        let mut observer = NoopDxfReadObserver;
        DxfBinaryRawDocument::open(source, options, &cancellation, &mut observer)
    }

    fn assert_raw_value(
        document: &impl RawSpanReader,
        view: &super::DxfHeaderView,
        expected: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let raw = view
            .acad_version()
            .raw_provenance()
            .ok_or(io::Error::other("missing raw provenance"))?;
        let mut bytes = vec![0_u8; expected.len()];
        document.read(raw.value_span(), &mut bytes)?;
        assert_eq!(bytes, expected);
        Ok(())
    }

    trait RawSpanReader {
        fn read(
            &self,
            span: crate::ByteSpan,
            destination: &mut [u8],
        ) -> Result<(), crate::DxfError>;
    }

    impl RawSpanReader for DxfAsciiRawDocument<'_> {
        fn read(
            &self,
            span: crate::ByteSpan,
            destination: &mut [u8],
        ) -> Result<(), crate::DxfError> {
            self.read_span(span, destination)
        }
    }

    impl RawSpanReader for DxfBinaryRawDocument<'_> {
        fn read(
            &self,
            span: crate::ByteSpan,
            destination: &mut [u8],
        ) -> Result<(), crate::DxfError> {
            self.read_span(span, destination)
        }
    }
}

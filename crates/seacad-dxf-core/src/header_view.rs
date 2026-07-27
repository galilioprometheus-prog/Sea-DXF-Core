use std::{io, num::NonZeroU64};

use crate::{
    DxfAcadVersion, DxfAcadVersionOccurrence, DxfAcadVersionReport, DxfAcadVersionState,
    DxfAcadVersionValue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCodePageOccurrence,
    DxfCodePageState, DxfCodePageValue, DxfError, DxfGroupCode, DxfIoOperation, DxfLegacyCodePage,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    DxfTextEncodingReport,
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

/// Typed interpretation of one structurally valid `$DWGCODEPAGE` declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCodePageDeclaration {
    Reviewed(DxfLegacyCodePage),
    Unrecognized,
}

/// Exact structural reason why `$DWGCODEPAGE` has no usable declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCodePageIssue {
    EmptyValue,
    InvalidGroupCode(DxfGroupCode),
    MissingValue,
    MultipleValues { occurrence_count: NonZeroU64 },
}

/// Fixed-size read-only projection of reviewed HEADER semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderView {
    acad_version: DxfSemanticValue<DxfAcadVersion, DxfAcadVersionIssue>,
    dwg_code_page: DxfSemanticValue<DxfCodePageDeclaration, DxfCodePageIssue>,
}

impl DxfHeaderView {
    fn from_reports(
        acad_version_report: &DxfAcadVersionReport,
        text_encoding_report: &DxfTextEncodingReport,
    ) -> Result<Self, DxfError> {
        if acad_version_report.source_id() != text_encoding_report.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: acad_version_report.source_id(),
                observed: text_encoding_report.source_id(),
            });
        }
        let field = acad_version_field_provenance(acad_version_report.source_id())?;
        let acad_version = match acad_version_report.state() {
            DxfAcadVersionState::Supported(version) => {
                let occurrence = acad_version_report
                    .primary_occurrence()
                    .ok_or_else(invalid_internal_data)?;
                DxfSemanticValue::explicit(version, field, required_value_provenance(occurrence)?)
            }
            DxfAcadVersionState::Absent => DxfSemanticValue::absent(field),
            DxfAcadVersionState::Unsupported => DxfSemanticValue::invalid(
                DxfAcadVersionIssue::UnsupportedValue,
                field,
                optional_evidence(acad_version_report.primary_occurrence())?,
            ),
            DxfAcadVersionState::Invalid => {
                let occurrence = acad_version_report.primary_occurrence();
                DxfSemanticValue::invalid(
                    invalid_issue(occurrence)?,
                    field,
                    optional_evidence(occurrence)?,
                )
            }
            DxfAcadVersionState::Ambiguous => {
                let occurrence_count = NonZeroU64::new(acad_version_report.occurrence_count())
                    .ok_or_else(invalid_internal_data)?;
                let evidence = acad_version_report
                    .conflicting_occurrence()
                    .or_else(|| acad_version_report.primary_occurrence());
                DxfSemanticValue::invalid(
                    DxfAcadVersionIssue::MultipleValues { occurrence_count },
                    field,
                    optional_evidence(evidence)?,
                )
            }
        };
        let dwg_code_page = code_page_semantic(text_encoding_report)?;
        Ok(Self {
            acad_version,
            dwg_code_page,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.acad_version.field_provenance().document_source_id()
    }

    #[must_use]
    pub const fn acad_version(&self) -> &DxfSemanticValue<DxfAcadVersion, DxfAcadVersionIssue> {
        &self.acad_version
    }

    #[must_use]
    pub const fn dwg_code_page(
        &self,
    ) -> &DxfSemanticValue<DxfCodePageDeclaration, DxfCodePageIssue> {
        &self.dwg_code_page
    }
}

impl DxfAsciiRawDocument<'_> {
    /// Projects indexed HEADER evidence without reading the source again.
    pub fn header_view(&self) -> Result<DxfHeaderView, DxfError> {
        DxfHeaderView::from_reports(self.acad_version_report(), self.text_encoding_report())
    }
}

impl DxfBinaryRawDocument<'_> {
    /// Projects indexed HEADER evidence without reading the source again.
    pub fn header_view(&self) -> Result<DxfHeaderView, DxfError> {
        DxfHeaderView::from_reports(self.acad_version_report(), self.text_encoding_report())
    }
}

fn acad_version_field_provenance(
    source_id: DxfSourceId,
) -> Result<DxfSemanticFieldProvenance, DxfError> {
    field_provenance(source_id, "acadver", "$ACADVER", 1)
}

fn code_page_field_provenance(
    source_id: DxfSourceId,
) -> Result<DxfSemanticFieldProvenance, DxfError> {
    field_provenance(source_id, "dwgcodepage", "$DWGCODEPAGE", 3)
}

fn field_provenance(
    source_id: DxfSourceId,
    id: &str,
    dxf_name: &str,
    group_code: i16,
) -> Result<DxfSemanticFieldProvenance, DxfError> {
    let field = HEADER_FIELDS
        .iter()
        .find(|field| {
            field.id == id
                && field.dxf_name == dxf_name
                && field.group_codes == [group_code]
                && field.storage == DxfSchemaStorageKind::ExactText
        })
        .ok_or_else(invalid_internal_data)?;
    Ok(DxfSemanticFieldProvenance::new(
        source_id,
        HEADER_NAMESPACE,
        field.id,
    ))
}

fn code_page_semantic(
    report: &DxfTextEncodingReport,
) -> Result<DxfSemanticValue<DxfCodePageDeclaration, DxfCodePageIssue>, DxfError> {
    let field = code_page_field_provenance(report.source_id())?;
    match report.code_page_state() {
        DxfCodePageState::Declared => {
            let occurrence = report
                .primary_occurrence()
                .ok_or_else(invalid_internal_data)?;
            if occurrence.value() != DxfCodePageValue::Declared {
                return Err(invalid_internal_data());
            }
            let declaration = match report.primary_legacy_code_page() {
                Some(code_page) => DxfCodePageDeclaration::Reviewed(code_page),
                None => DxfCodePageDeclaration::Unrecognized,
            };
            Ok(DxfSemanticValue::explicit(
                declaration,
                field,
                required_code_page_value_provenance(occurrence)?,
            ))
        }
        DxfCodePageState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfCodePageState::Invalid => {
            let occurrence = report.primary_occurrence();
            Ok(DxfSemanticValue::invalid(
                code_page_invalid_issue(occurrence)?,
                field,
                optional_code_page_evidence(occurrence)?,
            ))
        }
        DxfCodePageState::Ambiguous => {
            let occurrence_count =
                NonZeroU64::new(report.occurrence_count()).ok_or_else(invalid_internal_data)?;
            let evidence = report
                .conflicting_occurrence()
                .or_else(|| report.primary_occurrence());
            Ok(DxfSemanticValue::invalid(
                DxfCodePageIssue::MultipleValues { occurrence_count },
                field,
                optional_code_page_evidence(evidence)?,
            ))
        }
    }
}

fn code_page_invalid_issue(
    occurrence: Option<DxfCodePageOccurrence>,
) -> Result<DxfCodePageIssue, DxfError> {
    match occurrence.map(DxfCodePageOccurrence::value) {
        Some(DxfCodePageValue::Empty) => Ok(DxfCodePageIssue::EmptyValue),
        Some(DxfCodePageValue::InvalidGroupCode(group_code)) => {
            Ok(DxfCodePageIssue::InvalidGroupCode(group_code))
        }
        Some(DxfCodePageValue::MissingValue) | None => Ok(DxfCodePageIssue::MissingValue),
        Some(DxfCodePageValue::Declared) => Err(invalid_internal_data()),
    }
}

fn optional_code_page_evidence(
    occurrence: Option<DxfCodePageOccurrence>,
) -> Result<Option<DxfRawValueProvenance>, DxfError> {
    occurrence.map(code_page_raw_evidence).transpose()
}

fn required_code_page_value_provenance(
    occurrence: DxfCodePageOccurrence,
) -> Result<DxfRawValueProvenance, DxfError> {
    let group_occurrence = occurrence
        .value_occurrence()
        .ok_or_else(invalid_internal_data)?;
    let value_span = occurrence.value_span().ok_or_else(invalid_internal_data)?;
    DxfRawValueProvenance::new(group_occurrence, value_span).ok_or_else(invalid_internal_data)
}

fn code_page_raw_evidence(
    occurrence: DxfCodePageOccurrence,
) -> Result<DxfRawValueProvenance, DxfError> {
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

    use super::{DxfAcadVersionIssue, DxfCodePageDeclaration, DxfCodePageIssue};
    use crate::{
        DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
        DxfCancellationToken, DxfDiagnosticCode, DxfGroupCode, DxfLegacyCodePage, DxfMemorySource,
        DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, DxfTextEncodingPolicy,
        DxfTextEncodingResolution, NoopDxfReadObserver,
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
                assert_eq!(view.dwg_code_page().state(), DxfSemanticValueState::Absent);
                assert_eq!(
                    view.dwg_code_page().field_provenance().schema_field_id(),
                    "dwgcodepage"
                );
            }

            assert_raw_value(&ascii, &ascii_view, version.code().as_bytes())?;
            assert_raw_value(&binary, &binary_view, version.code().as_bytes())?;
            for report in [ascii.text_encoding_report(), binary.text_encoding_report()] {
                if version.uses_utf8_string_storage() {
                    assert_eq!(report.policy(), DxfTextEncodingPolicy::Utf8(version));
                    assert!(report.diagnostics().is_empty());
                } else {
                    assert_eq!(report.policy(), DxfTextEncodingPolicy::Indeterminate);
                    assert!(report.diagnostics().iter().any(|diagnostic| {
                        diagnostic.code() == DxfDiagnosticCode::CODEPAGE_REQUIRED
                    }));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn every_supported_version_has_code_page_ascii_binary_parity() -> Result<(), Box<dyn Error>> {
        for version in DxfAcadVersion::SUPPORTED {
            let declarations = [(3_i16, "ANSI_1252")];
            let ascii_bytes = ascii_code_page_fixture(version.code(), &declarations);
            let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
            let ascii = open_ascii(&ascii_source, DxfReadOptions::strict())?;
            let ascii_view = ascii.header_view()?;

            let binary_bytes = binary_code_page_fixture(version, &declarations)?;
            let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
            let binary = open_binary(&binary_source, DxfReadOptions::strict())?;
            let binary_view = binary.header_view()?;

            let expected_policy = if version.uses_utf8_string_storage() {
                DxfTextEncodingPolicy::Utf8(version)
            } else {
                DxfTextEncodingPolicy::LegacyDeclared(version)
            };
            for (view, report) in [
                (&ascii_view, ascii.text_encoding_report()),
                (&binary_view, binary.text_encoding_report()),
            ] {
                assert_eq!(
                    view.dwg_code_page().state(),
                    DxfSemanticValueState::Explicit
                );
                assert_eq!(
                    view.dwg_code_page().value(),
                    Some(&DxfCodePageDeclaration::Reviewed(
                        DxfLegacyCodePage::Windows1252
                    ))
                );
                assert_eq!(report.policy(), expected_policy);
                assert_eq!(
                    report.primary_legacy_code_page(),
                    Some(DxfLegacyCodePage::Windows1252)
                );
                assert!(report.diagnostics().is_empty());
            }
            assert_code_page_raw_value(&ascii, &ascii_view, b"ANSI_1252")?;
            assert_code_page_raw_value(&binary, &binary_view, b"ANSI_1252")?;
        }
        Ok(())
    }

    #[test]
    fn unrecognized_declaration_is_explicit_but_not_a_legacy_decoder() -> Result<(), Box<dyn Error>>
    {
        let declarations = [(3_i16, "ANSI_1362")];
        let ascii_bytes = ascii_code_page_fixture("AC1018", &declarations);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source, DxfReadOptions::strict())?;
        let ascii_view = ascii.header_view()?;

        let binary_bytes = binary_code_page_fixture(DxfAcadVersion::Ac1018, &declarations)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source, DxfReadOptions::strict())?;
        let binary_view = binary.header_view()?;

        for (view, report) in [
            (&ascii_view, ascii.text_encoding_report()),
            (&binary_view, binary.text_encoding_report()),
        ] {
            assert_eq!(
                view.dwg_code_page().value(),
                Some(&DxfCodePageDeclaration::Unrecognized)
            );
            assert_eq!(report.primary_legacy_code_page(), None);
            assert!(matches!(
                report.resolution(),
                DxfTextEncodingResolution::UnsupportedLegacy {
                    version: DxfAcadVersion::Ac1018,
                    ..
                }
            ));
            assert_eq!(report.diagnostics().len(), 1);
            assert_eq!(
                report.diagnostics()[0].code(),
                DxfDiagnosticCode::CODEPAGE_UNSUPPORTED
            );
        }
        assert_code_page_raw_value(&ascii, &ascii_view, b"ANSI_1362")?;
        assert_code_page_raw_value(&binary, &binary_view, b"ANSI_1362")?;

        let modern_bytes = binary_code_page_fixture(DxfAcadVersion::Ac1021, &declarations)?;
        let modern_source = DxfMemorySource::new(&modern_bytes, DxfResourceProfile::Safe)?;
        let modern = open_binary(&modern_source, DxfReadOptions::strict())?;
        let modern_view = modern.header_view()?;
        assert_eq!(
            modern_view.dwg_code_page().value(),
            Some(&DxfCodePageDeclaration::Unrecognized)
        );
        assert_eq!(
            modern.text_encoding_report().resolution(),
            DxfTextEncodingResolution::Utf8(DxfAcadVersion::Ac1021)
        );
        assert!(modern.text_encoding_report().diagnostics().is_empty());
        Ok(())
    }

    #[test]
    fn binary_code_page_boundaries_remain_typed_and_source_anchored() -> Result<(), Box<dyn Error>>
    {
        let empty_bytes = binary_code_page_fixture(DxfAcadVersion::Ac1018, &[(3, "")])?;
        let empty_source = DxfMemorySource::new(&empty_bytes, DxfResourceProfile::Safe)?;
        let empty = open_binary(&empty_source, DxfReadOptions::strict())?;
        let empty_view = empty.header_view()?;
        assert_eq!(
            empty_view.dwg_code_page().invalid_issue(),
            Some(&DxfCodePageIssue::EmptyValue)
        );
        assert_eq!(
            empty.text_encoding_report().diagnostics()[0].code(),
            DxfDiagnosticCode::CODEPAGE_VALUE_INVALID
        );
        assert_code_page_raw_value(&empty, &empty_view, b"")?;

        let wrong_bytes = binary_code_page_fixture(DxfAcadVersion::Ac1018, &[(1, "ANSI_1252")])?;
        let wrong_source = DxfMemorySource::new(&wrong_bytes, DxfResourceProfile::Safe)?;
        let wrong = open_binary(&wrong_source, DxfReadOptions::strict())?;
        let wrong_view = wrong.header_view()?;
        let code_1 = DxfGroupCode::new(1).ok_or(io::Error::other("group code"))?;
        assert_eq!(
            wrong_view.dwg_code_page().invalid_issue(),
            Some(&DxfCodePageIssue::InvalidGroupCode(code_1))
        );
        assert_eq!(
            wrong.text_encoding_report().diagnostics()[0].code(),
            DxfDiagnosticCode::CODEPAGE_VALUE_INVALID
        );
        assert_code_page_raw_value(&wrong, &wrong_view, b"ANSI_1252")?;

        let duplicate_bytes =
            binary_code_page_fixture(DxfAcadVersion::Ac1018, &[(3, "ANSI_1252"), (3, "ANSI_932")])?;
        let duplicate_source = DxfMemorySource::new(&duplicate_bytes, DxfResourceProfile::Safe)?;
        let duplicate = open_binary(&duplicate_source, DxfReadOptions::strict())?;
        let duplicate_view = duplicate.header_view()?;
        assert!(matches!(
            duplicate_view.dwg_code_page().invalid_issue(),
            Some(DxfCodePageIssue::MultipleValues { occurrence_count })
                if occurrence_count.get() == 2
        ));
        assert!(
            duplicate
                .text_encoding_report()
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code() == DxfDiagnosticCode::CODEPAGE_DUPLICATE)
        );
        assert_code_page_raw_value(&duplicate, &duplicate_view, b"ANSI_932")?;

        let missing_bytes = binary_missing_code_page_fixture(DxfAcadVersion::Ac1018)?;
        let missing_source = DxfMemorySource::new(&missing_bytes, DxfResourceProfile::Safe)?;
        let missing = open_binary(&missing_source, DxfReadOptions::compatible())?;
        let missing_view = missing.header_view()?;
        assert_eq!(
            missing_view.dwg_code_page().invalid_issue(),
            Some(&DxfCodePageIssue::MissingValue)
        );
        assert!(
            missing
                .text_encoding_report()
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code() == DxfDiagnosticCode::CODEPAGE_VALUE_INVALID)
        );
        assert_code_page_raw_value(&missing, &missing_view, b"$DWGCODEPAGE")?;
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
        ascii_code_page_fixture(version, &[])
    }

    fn ascii_code_page_fixture(version: &str, declarations: &[(i16, &str)]) -> Vec<u8> {
        let mut text = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n");
        for (group_code, value) in declarations {
            text.push_str(&format!("9\n$DWGCODEPAGE\n{group_code}\n{value}\n"));
        }
        text.push_str("0\nENDSEC\n0\nEOF\n");
        text.into_bytes()
    }

    fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
        binary_code_page_fixture(version, &[])
    }

    fn binary_code_page_fixture(
        version: DxfAcadVersion,
        declarations: &[(i16, &str)],
    ) -> Result<Vec<u8>, io::Error> {
        let mut bytes = binary_header_prefix(version)?;
        for (group_code, value) in declarations {
            push_binary_group(&mut bytes, version, 9, b"$DWGCODEPAGE")?;
            push_binary_group(&mut bytes, version, *group_code, value.as_bytes())?;
        }
        for (group_code, value) in [(0_i16, "ENDSEC"), (0, "EOF")] {
            push_binary_group(&mut bytes, version, group_code, value.as_bytes())?;
        }
        Ok(bytes)
    }

    fn binary_missing_code_page_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
        let mut bytes = binary_header_prefix(version)?;
        push_binary_group(&mut bytes, version, 9, b"$DWGCODEPAGE")?;
        Ok(bytes)
    }

    fn binary_header_prefix(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        for (group_code, value) in [
            (0_i16, "SECTION"),
            (2, "HEADER"),
            (9, "$ACADVER"),
            (1, version.code()),
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
        assert_semantic_raw_value(document, view.acad_version().raw_provenance(), expected)
    }

    fn assert_code_page_raw_value(
        document: &impl RawSpanReader,
        view: &super::DxfHeaderView,
        expected: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        assert_semantic_raw_value(document, view.dwg_code_page().raw_provenance(), expected)
    }

    fn assert_semantic_raw_value(
        document: &impl RawSpanReader,
        raw: Option<crate::DxfRawValueProvenance>,
        expected: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let raw = raw.ok_or(io::Error::other("missing raw provenance"))?;
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

//! Source-anchored `$DWGCODEPAGE` discovery and text-encoding policy.

use std::io;

use crate::{
    ByteSpan, DxfAcadVersion, DxfAcadVersionState, DxfAsciiGroup, DxfDiagnostic, DxfDiagnosticCode,
    DxfError, DxfGroupCode, DxfIoOperation, DxfSourceId,
};

/// Classification of the group immediately following one `$DWGCODEPAGE`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCodePageValue {
    Declared,
    Empty,
    InvalidGroupCode(DxfGroupCode),
    MissingValue,
}

/// Fixed-size provenance for one exact `$DWGCODEPAGE` occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCodePageOccurrence {
    variable_occurrence: u32,
    variable_span: ByteSpan,
    value_occurrence: Option<u32>,
    value_span: Option<ByteSpan>,
    value: DxfCodePageValue,
}

impl DxfCodePageOccurrence {
    #[must_use]
    pub const fn variable_occurrence(self) -> u64 {
        self.variable_occurrence as u64
    }

    #[must_use]
    pub const fn variable_span(self) -> ByteSpan {
        self.variable_span
    }

    #[must_use]
    pub const fn value_occurrence(self) -> Option<u64> {
        match self.value_occurrence {
            Some(value) => Some(value as u64),
            None => None,
        }
    }

    #[must_use]
    pub const fn value_span(self) -> Option<ByteSpan> {
        self.value_span
    }

    #[must_use]
    pub const fn value(self) -> DxfCodePageValue {
        self.value
    }
}

/// Structural state of the exact HEADER codepage declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCodePageState {
    Absent,
    Declared,
    Invalid,
    Ambiguous,
}

/// Evidence-backed string storage policy; this is not a decoded text view.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextEncodingPolicy {
    Utf8(DxfAcadVersion),
    LegacyDeclared(DxfAcadVersion),
    Indeterminate,
}

/// Immutable encoding decision tied to raw source identity and byte spans.
#[derive(Debug)]
pub struct DxfTextEncodingReport {
    source_id: DxfSourceId,
    policy: DxfTextEncodingPolicy,
    code_page_state: DxfCodePageState,
    occurrence_count: u64,
    primary: Option<DxfCodePageOccurrence>,
    conflicting: Option<DxfCodePageOccurrence>,
    diagnostics: Box<[DxfDiagnostic]>,
}

impl DxfTextEncodingReport {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn policy(&self) -> DxfTextEncodingPolicy {
        self.policy
    }

    #[must_use]
    pub const fn code_page_state(&self) -> DxfCodePageState {
        self.code_page_state
    }

    #[must_use]
    pub const fn occurrence_count(&self) -> u64 {
        self.occurrence_count
    }

    #[must_use]
    pub const fn primary_occurrence(&self) -> Option<DxfCodePageOccurrence> {
        self.primary
    }

    #[must_use]
    pub const fn conflicting_occurrence(&self) -> Option<DxfCodePageOccurrence> {
        self.conflicting
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[DxfDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Copy)]
struct VariableMarker {
    occurrence: u32,
    span: ByteSpan,
}

#[derive(Default)]
pub(crate) struct DxfTextEncodingTracker {
    awaiting_section_name: bool,
    inside_header: bool,
    first_header_span: Option<ByteSpan>,
    pending: Option<VariableMarker>,
    occurrence_count: u64,
    primary: Option<DxfCodePageOccurrence>,
    conflicting: Option<DxfCodePageOccurrence>,
}

impl DxfTextEncodingTracker {
    pub(crate) fn observe(&mut self, group: DxfAsciiGroup<'_>) -> Result<(), DxfError> {
        if let Some(variable) = self.pending.take() {
            self.record_candidate(variable, group)?;
        }

        if self.awaiting_section_name {
            self.awaiting_section_name = false;
            self.inside_header = group.group_code().value() == 2 && group.raw_value() == b"HEADER";
            if self.inside_header && self.first_header_span.is_none() {
                self.first_header_span = Some(group.value_line().content_span());
            }
        }

        if group.group_code().value() == 0 {
            match group.raw_value() {
                b"SECTION" => {
                    self.inside_header = false;
                    self.awaiting_section_name = true;
                }
                b"ENDSEC" => {
                    self.inside_header = false;
                    self.awaiting_section_name = false;
                }
                _ => {}
            }
        } else if self.inside_header
            && group.group_code().value() == 9
            && group.raw_value() == b"$DWGCODEPAGE"
        {
            self.pending = Some(VariableMarker {
                occurrence: compact_occurrence(group.occurrence())?,
                span: group.value_line().content_span(),
            });
        }
        Ok(())
    }

    pub(crate) fn finish(
        mut self,
        source_id: DxfSourceId,
        acad_version_state: DxfAcadVersionState,
    ) -> Result<DxfTextEncodingReport, DxfError> {
        if let Some(variable) = self.pending.take() {
            self.record_occurrence(DxfCodePageOccurrence {
                variable_occurrence: variable.occurrence,
                variable_span: variable.span,
                value_occurrence: None,
                value_span: None,
                value: DxfCodePageValue::MissingValue,
            })?;
        }

        let code_page_state = if self.occurrence_count == 0 {
            DxfCodePageState::Absent
        } else if self.occurrence_count > 1 {
            DxfCodePageState::Ambiguous
        } else {
            match self.primary.map(DxfCodePageOccurrence::value) {
                Some(DxfCodePageValue::Declared) => DxfCodePageState::Declared,
                Some(
                    DxfCodePageValue::Empty
                    | DxfCodePageValue::InvalidGroupCode(_)
                    | DxfCodePageValue::MissingValue,
                ) => DxfCodePageState::Invalid,
                None => DxfCodePageState::Absent,
            }
        };

        let policy = match acad_version_state {
            DxfAcadVersionState::Supported(version) if version.uses_utf8_string_storage() => {
                DxfTextEncodingPolicy::Utf8(version)
            }
            DxfAcadVersionState::Supported(version)
                if code_page_state == DxfCodePageState::Declared =>
            {
                DxfTextEncodingPolicy::LegacyDeclared(version)
            }
            _ => DxfTextEncodingPolicy::Indeterminate,
        };

        let mut diagnostics = Vec::new();
        if matches!(
            acad_version_state,
            DxfAcadVersionState::Supported(version) if !version.uses_utf8_string_storage()
        ) && code_page_state == DxfCodePageState::Absent
        {
            push_diagnostic(
                &mut diagnostics,
                DxfDiagnostic::new(DxfDiagnosticCode::CODEPAGE_REQUIRED, self.first_header_span),
            )?;
        }
        for occurrence in [self.primary, self.conflicting].into_iter().flatten() {
            if occurrence.value() != DxfCodePageValue::Declared {
                push_diagnostic(
                    &mut diagnostics,
                    DxfDiagnostic::new(
                        DxfDiagnosticCode::CODEPAGE_VALUE_INVALID,
                        occurrence.value_span().or(Some(occurrence.variable_span())),
                    ),
                )?;
            }
        }
        if let Some(conflicting) = self.conflicting {
            push_diagnostic(
                &mut diagnostics,
                DxfDiagnostic::new(
                    DxfDiagnosticCode::CODEPAGE_DUPLICATE,
                    Some(conflicting.variable_span()),
                ),
            )?;
        }

        Ok(DxfTextEncodingReport {
            source_id,
            policy,
            code_page_state,
            occurrence_count: self.occurrence_count,
            primary: self.primary,
            conflicting: self.conflicting,
            diagnostics: diagnostics.into_boxed_slice(),
        })
    }

    fn record_candidate(
        &mut self,
        variable: VariableMarker,
        candidate: DxfAsciiGroup<'_>,
    ) -> Result<(), DxfError> {
        let value = if candidate.group_code().value() != 3 {
            DxfCodePageValue::InvalidGroupCode(candidate.group_code())
        } else if candidate.raw_value().is_empty() {
            DxfCodePageValue::Empty
        } else {
            DxfCodePageValue::Declared
        };
        self.record_occurrence(DxfCodePageOccurrence {
            variable_occurrence: variable.occurrence,
            variable_span: variable.span,
            value_occurrence: Some(compact_occurrence(candidate.occurrence())?),
            value_span: Some(candidate.value_line().content_span()),
            value,
        })
    }

    fn record_occurrence(&mut self, occurrence: DxfCodePageOccurrence) -> Result<(), DxfError> {
        self.occurrence_count = self
            .occurrence_count
            .checked_add(1)
            .ok_or_else(invalid_source_data)?;
        if self.primary.is_none() {
            self.primary = Some(occurrence);
        } else if self.conflicting.is_none() {
            self.conflicting = Some(occurrence);
        }
        Ok(())
    }
}

fn compact_occurrence(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_source_data())
}

fn push_diagnostic(
    diagnostics: &mut Vec<DxfDiagnostic>,
    diagnostic: DxfDiagnostic,
) -> Result<(), DxfError> {
    diagnostics.try_reserve(1).map_err(|_| out_of_memory())?;
    diagnostics.push(diagnostic);
    Ok(())
}

fn invalid_source_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::{
        DxfCodePageOccurrence, DxfCodePageState, DxfCodePageValue, DxfTextEncodingPolicy,
        DxfTextEncodingReport,
    };
    use crate::{
        DxfAcadVersion, DxfAsciiRawDocument, DxfCancellationToken, DxfDiagnosticCode, DxfGroupCode,
        DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
    };

    #[test]
    fn every_supported_version_selects_the_documented_storage_policy() -> Result<(), Box<dyn Error>>
    {
        assert_send_sync::<DxfTextEncodingReport>();
        for version in DxfAcadVersion::SUPPORTED {
            let bytes = fixture(version.code(), Some((3, "ANSI_1252")));
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open(&source, DxfReadOptions::strict())?;
            let report = document.text_encoding_report();
            let expected = if version.uses_utf8_string_storage() {
                DxfTextEncodingPolicy::Utf8(version)
            } else {
                DxfTextEncodingPolicy::LegacyDeclared(version)
            };
            assert_eq!(report.source_id(), document.source_id());
            assert_eq!(report.policy(), expected);
            assert_eq!(report.code_page_state(), DxfCodePageState::Declared);
            assert_eq!(report.occurrence_count(), 1);
            assert!(report.diagnostics().is_empty());

            let occurrence = required_primary(report)?;
            assert_eq!(occurrence.variable_occurrence(), 4);
            assert_eq!(occurrence.value_occurrence(), Some(5));
            assert_eq!(occurrence.value(), DxfCodePageValue::Declared);
            let mut variable = [0_u8; 12];
            document.read_span(occurrence.variable_span(), &mut variable)?;
            assert_eq!(&variable, b"$DWGCODEPAGE");
            let mut value = [0_u8; 9];
            document.read_span(
                occurrence
                    .value_span()
                    .ok_or(io::Error::other("missing codepage span"))?,
                &mut value,
            )?;
            assert_eq!(&value, b"ANSI_1252");
        }
        Ok(())
    }

    #[test]
    fn modern_version_does_not_require_a_codepage_but_legacy_does() -> Result<(), Box<dyn Error>> {
        let modern_bytes = fixture("AC1021", None);
        let modern_source = DxfMemorySource::new(&modern_bytes, DxfResourceProfile::Safe)?;
        let modern = open(&modern_source, DxfReadOptions::strict())?;
        assert_eq!(
            modern.text_encoding_report().policy(),
            DxfTextEncodingPolicy::Utf8(DxfAcadVersion::Ac1021)
        );
        assert_eq!(
            modern.text_encoding_report().code_page_state(),
            DxfCodePageState::Absent
        );
        assert!(modern.text_encoding_report().diagnostics().is_empty());

        let legacy_bytes = fixture("AC1018", None);
        let legacy_source = DxfMemorySource::new(&legacy_bytes, DxfResourceProfile::Safe)?;
        let legacy = open(&legacy_source, DxfReadOptions::strict())?;
        assert_eq!(
            legacy.text_encoding_report().policy(),
            DxfTextEncodingPolicy::Indeterminate
        );
        assert_eq!(
            legacy.text_encoding_report().diagnostics()[0].code(),
            DxfDiagnosticCode::CODEPAGE_REQUIRED
        );
        assert!(
            legacy.text_encoding_report().diagnostics()[0]
                .span()
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn modern_version_wins_over_a_malformed_stale_declaration() -> Result<(), Box<dyn Error>> {
        let bytes = fixture("AC1021", Some((1, "ANSI_1252")));
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let report = document.text_encoding_report();
        let code_1 = DxfGroupCode::new(1).ok_or(io::Error::other("group code"))?;
        assert_eq!(
            report.policy(),
            DxfTextEncodingPolicy::Utf8(DxfAcadVersion::Ac1021)
        );
        assert_eq!(report.code_page_state(), DxfCodePageState::Invalid);
        assert_eq!(
            required_primary(report)?.value(),
            DxfCodePageValue::InvalidGroupCode(code_1)
        );
        assert_eq!(
            report.diagnostics()[0].code(),
            DxfDiagnosticCode::CODEPAGE_VALUE_INVALID
        );
        Ok(())
    }

    #[test]
    fn empty_wrong_group_and_missing_values_are_invalid_without_fallback()
    -> Result<(), Box<dyn Error>> {
        let cases = [
            (fixture("AC1018", Some((3, ""))), DxfCodePageValue::Empty),
            (
                fixture("AC1018", Some((1, "ANSI_1252"))),
                DxfCodePageValue::InvalidGroupCode(
                    DxfGroupCode::new(1).ok_or(io::Error::other("group code"))?,
                ),
            ),
        ];
        for (bytes, expected) in cases {
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open(&source, DxfReadOptions::strict())?;
            let report = document.text_encoding_report();
            assert_eq!(report.policy(), DxfTextEncodingPolicy::Indeterminate);
            assert_eq!(report.code_page_state(), DxfCodePageState::Invalid);
            assert_eq!(required_primary(report)?.value(), expected);
            assert_eq!(
                report.diagnostics()[0].code(),
                DxfDiagnosticCode::CODEPAGE_VALUE_INVALID
            );
        }

        let missing_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1018\n9\n$DWGCODEPAGE\n";
        let missing_source = DxfMemorySource::new(missing_bytes, DxfResourceProfile::Safe)?;
        let missing = open(&missing_source, DxfReadOptions::compatible())?;
        let missing_report = missing.text_encoding_report();
        assert_eq!(missing_report.code_page_state(), DxfCodePageState::Invalid);
        assert_eq!(
            required_primary(missing_report)?.value(),
            DxfCodePageValue::MissingValue
        );
        assert_eq!(required_primary(missing_report)?.value_occurrence(), None);
        Ok(())
    }

    #[test]
    fn duplicates_are_ambiguous_with_fixed_evidence() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1018\n9\n$DWGCODEPAGE\n3\nANSI_1252\n9\n$DWGCODEPAGE\n3\nANSI_932\n9\n$DWGCODEPAGE\n3\nANSI_950\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let report = document.text_encoding_report();
        assert_eq!(report.policy(), DxfTextEncodingPolicy::Indeterminate);
        assert_eq!(report.code_page_state(), DxfCodePageState::Ambiguous);
        assert_eq!(report.occurrence_count(), 3);
        assert_eq!(
            required_primary(report)?.value(),
            DxfCodePageValue::Declared
        );
        assert_eq!(
            report
                .conflicting_occurrence()
                .ok_or(io::Error::other("missing conflict"))?
                .value(),
            DxfCodePageValue::Declared
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code() == DxfDiagnosticCode::CODEPAGE_DUPLICATE)
        );
        Ok(())
    }

    #[test]
    fn discovery_is_exact_and_ignores_other_sections() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nENTITIES\n9\n$DWGCODEPAGE\n3\nANSI_1252\n0\nENDSEC\n0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1018\n9\n $DWGCODEPAGE\n3\nANSI_932\n9\n$dwgcodepage\n3\nANSI_950\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let report = document.text_encoding_report();
        assert_eq!(report.code_page_state(), DxfCodePageState::Absent);
        assert_eq!(report.occurrence_count(), 0);
        assert_eq!(report.policy(), DxfTextEncodingPolicy::Indeterminate);
        assert_eq!(
            report.diagnostics()[0].code(),
            DxfDiagnosticCode::CODEPAGE_REQUIRED
        );
        Ok(())
    }

    #[test]
    fn ambiguous_acad_version_prevents_an_encoding_guess() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1018\n9\n$ACADVER\n1\nAC1021\n9\n$DWGCODEPAGE\n3\nANSI_1252\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        assert_eq!(
            document.text_encoding_report().code_page_state(),
            DxfCodePageState::Declared
        );
        assert_eq!(
            document.text_encoding_report().policy(),
            DxfTextEncodingPolicy::Indeterminate
        );
        assert!(document.text_encoding_report().diagnostics().is_empty());
        Ok(())
    }

    fn fixture(version: &str, codepage: Option<(i16, &str)>) -> Vec<u8> {
        let mut text = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n");
        if let Some((group_code, value)) = codepage {
            text.push_str(&format!("9\n$DWGCODEPAGE\n{group_code}\n{value}\n"));
        }
        text.push_str("0\nENDSEC\n0\nEOF\n");
        text.into_bytes()
    }

    fn open<'a>(
        source: &'a DxfMemorySource<'_>,
        options: DxfReadOptions,
    ) -> Result<DxfAsciiRawDocument<'a>, crate::DxfError> {
        let cancellation = DxfCancellationToken::default();
        let mut observer = NoopDxfReadObserver;
        DxfAsciiRawDocument::open(source, options, &cancellation, &mut observer)
    }

    fn required_primary(
        report: &DxfTextEncodingReport,
    ) -> Result<DxfCodePageOccurrence, io::Error> {
        report
            .primary_occurrence()
            .ok_or(io::Error::other("missing primary occurrence"))
    }

    fn assert_send_sync<T: Send + Sync>() {}
}

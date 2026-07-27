//! Typed `$ACADVER` registry and source-anchored ASCII discovery.

use std::io;

use crate::{
    ByteSpan, DxfAsciiGroup, DxfDiagnostic, DxfDiagnosticCode, DxfError, DxfGroupCode,
    DxfIoOperation, DxfSourceId,
};

/// Supported AutoCAD drawing database versions from the public DXF reference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DxfAcadVersion {
    Ac1009,
    Ac1012,
    Ac1014,
    Ac1015,
    Ac1018,
    Ac1021,
    Ac1024,
    Ac1027,
    Ac1032,
}

impl DxfAcadVersion {
    pub const SUPPORTED: [Self; 9] = [
        Self::Ac1009,
        Self::Ac1012,
        Self::Ac1014,
        Self::Ac1015,
        Self::Ac1018,
        Self::Ac1021,
        Self::Ac1024,
        Self::Ac1027,
        Self::Ac1032,
    ];

    pub const MIN_SUPPORTED: Self = Self::Ac1009;
    pub const MAX_SUPPORTED: Self = Self::Ac1032;

    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Ac1009 => "AC1009",
            Self::Ac1012 => "AC1012",
            Self::Ac1014 => "AC1014",
            Self::Ac1015 => "AC1015",
            Self::Ac1018 => "AC1018",
            Self::Ac1021 => "AC1021",
            Self::Ac1024 => "AC1024",
            Self::Ac1027 => "AC1027",
            Self::Ac1032 => "AC1032",
        }
    }

    #[must_use]
    pub const fn autodesk_release_name(self) -> &'static str {
        match self {
            Self::Ac1009 => "AutoCAD R11/R12",
            Self::Ac1012 => "AutoCAD R13",
            Self::Ac1014 => "AutoCAD R14",
            Self::Ac1015 => "AutoCAD 2000",
            Self::Ac1018 => "AutoCAD 2004",
            Self::Ac1021 => "AutoCAD 2007",
            Self::Ac1024 => "AutoCAD 2010",
            Self::Ac1027 => "AutoCAD 2013",
            Self::Ac1032 => "AutoCAD 2018",
        }
    }

    #[must_use]
    pub fn from_code_bytes(code: &[u8]) -> Option<Self> {
        match code {
            b"AC1009" => Some(Self::Ac1009),
            b"AC1012" => Some(Self::Ac1012),
            b"AC1014" => Some(Self::Ac1014),
            b"AC1015" => Some(Self::Ac1015),
            b"AC1018" => Some(Self::Ac1018),
            b"AC1021" => Some(Self::Ac1021),
            b"AC1024" => Some(Self::Ac1024),
            b"AC1027" => Some(Self::Ac1027),
            b"AC1032" => Some(Self::Ac1032),
            _ => None,
        }
    }
}

/// Classification of the group immediately following one `$ACADVER` name.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAcadVersionValue {
    Supported(DxfAcadVersion),
    Unsupported,
    InvalidGroupCode(DxfGroupCode),
    MissingValue,
}

/// Fixed-size provenance for one exact `$ACADVER` variable occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfAcadVersionOccurrence {
    variable_occurrence: u32,
    variable_span: ByteSpan,
    value_occurrence: Option<u32>,
    value_span: Option<ByteSpan>,
    value: DxfAcadVersionValue,
}

impl DxfAcadVersionOccurrence {
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
    pub const fn value(self) -> DxfAcadVersionValue {
        self.value
    }
}

/// Whether `$ACADVER` is usable as one unambiguous supported dialect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAcadVersionState {
    Absent,
    Supported(DxfAcadVersion),
    Unsupported,
    Invalid,
    Ambiguous,
}

/// Immutable discovery result tied to the raw document source identity.
#[derive(Debug)]
pub struct DxfAcadVersionReport {
    source_id: DxfSourceId,
    state: DxfAcadVersionState,
    occurrence_count: u64,
    primary: Option<DxfAcadVersionOccurrence>,
    conflicting: Option<DxfAcadVersionOccurrence>,
    diagnostics: Box<[DxfDiagnostic]>,
}

impl DxfAcadVersionReport {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn state(&self) -> DxfAcadVersionState {
        self.state
    }

    #[must_use]
    pub const fn occurrence_count(&self) -> u64 {
        self.occurrence_count
    }

    #[must_use]
    pub const fn primary_occurrence(&self) -> Option<DxfAcadVersionOccurrence> {
        self.primary
    }

    #[must_use]
    pub const fn conflicting_occurrence(&self) -> Option<DxfAcadVersionOccurrence> {
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
pub(crate) struct DxfAcadVersionTracker {
    awaiting_section_name: bool,
    inside_header: bool,
    first_header_span: Option<ByteSpan>,
    pending: Option<VariableMarker>,
    occurrence_count: u64,
    primary: Option<DxfAcadVersionOccurrence>,
    conflicting: Option<DxfAcadVersionOccurrence>,
}

impl DxfAcadVersionTracker {
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
            && group.raw_value() == b"$ACADVER"
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
    ) -> Result<DxfAcadVersionReport, DxfError> {
        if let Some(variable) = self.pending.take() {
            self.record_occurrence(DxfAcadVersionOccurrence {
                variable_occurrence: variable.occurrence,
                variable_span: variable.span,
                value_occurrence: None,
                value_span: None,
                value: DxfAcadVersionValue::MissingValue,
            })?;
        }

        let state = if self.occurrence_count == 0 {
            DxfAcadVersionState::Absent
        } else if self.occurrence_count > 1 {
            DxfAcadVersionState::Ambiguous
        } else {
            match self.primary.map(DxfAcadVersionOccurrence::value) {
                Some(DxfAcadVersionValue::Supported(version)) => {
                    DxfAcadVersionState::Supported(version)
                }
                Some(DxfAcadVersionValue::Unsupported) => DxfAcadVersionState::Unsupported,
                Some(
                    DxfAcadVersionValue::InvalidGroupCode(_) | DxfAcadVersionValue::MissingValue,
                ) => DxfAcadVersionState::Invalid,
                None => DxfAcadVersionState::Absent,
            }
        };

        let mut diagnostics = Vec::new();
        if self.occurrence_count == 0 {
            push_diagnostic(
                &mut diagnostics,
                DxfDiagnostic::new(DxfDiagnosticCode::ACADVER_MISSING, self.first_header_span),
            )?;
        }
        if let Some(primary) = self.primary {
            match primary.value() {
                DxfAcadVersionValue::Unsupported => push_diagnostic(
                    &mut diagnostics,
                    DxfDiagnostic::new(
                        DxfDiagnosticCode::ACADVER_UNSUPPORTED,
                        primary.value_span(),
                    ),
                )?,
                DxfAcadVersionValue::InvalidGroupCode(_) | DxfAcadVersionValue::MissingValue => {
                    push_diagnostic(
                        &mut diagnostics,
                        DxfDiagnostic::new(
                            DxfDiagnosticCode::ACADVER_VALUE_INVALID,
                            primary.value_span().or(Some(primary.variable_span())),
                        ),
                    )?;
                }
                DxfAcadVersionValue::Supported(_) => {}
            }
        }
        if let Some(conflicting) = self.conflicting {
            push_diagnostic(
                &mut diagnostics,
                DxfDiagnostic::new(
                    DxfDiagnosticCode::ACADVER_DUPLICATE,
                    Some(conflicting.variable_span()),
                ),
            )?;
        }

        Ok(DxfAcadVersionReport {
            source_id,
            state,
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
        let value = if candidate.group_code().value() == 1 {
            match DxfAcadVersion::from_code_bytes(candidate.raw_value()) {
                Some(version) => DxfAcadVersionValue::Supported(version),
                None => DxfAcadVersionValue::Unsupported,
            }
        } else {
            DxfAcadVersionValue::InvalidGroupCode(candidate.group_code())
        };
        self.record_occurrence(DxfAcadVersionOccurrence {
            variable_occurrence: variable.occurrence,
            variable_span: variable.span,
            value_occurrence: Some(compact_occurrence(candidate.occurrence())?),
            value_span: Some(candidate.value_line().content_span()),
            value,
        })
    }

    fn record_occurrence(&mut self, occurrence: DxfAcadVersionOccurrence) -> Result<(), DxfError> {
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
    use std::{error::Error, io};

    use super::{
        DxfAcadVersion, DxfAcadVersionOccurrence, DxfAcadVersionReport, DxfAcadVersionState,
        DxfAcadVersionValue,
    };
    use crate::{
        DxfAsciiDocumentConformance, DxfAsciiRawDocument, DxfCancellationToken, DxfDiagnosticCode,
        DxfGroupCode, DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
    };

    #[test]
    fn supported_registry_matches_the_autodesk_reference() {
        let expected = [
            (DxfAcadVersion::Ac1009, "AC1009", "AutoCAD R11/R12"),
            (DxfAcadVersion::Ac1012, "AC1012", "AutoCAD R13"),
            (DxfAcadVersion::Ac1014, "AC1014", "AutoCAD R14"),
            (DxfAcadVersion::Ac1015, "AC1015", "AutoCAD 2000"),
            (DxfAcadVersion::Ac1018, "AC1018", "AutoCAD 2004"),
            (DxfAcadVersion::Ac1021, "AC1021", "AutoCAD 2007"),
            (DxfAcadVersion::Ac1024, "AC1024", "AutoCAD 2010"),
            (DxfAcadVersion::Ac1027, "AC1027", "AutoCAD 2013"),
            (DxfAcadVersion::Ac1032, "AC1032", "AutoCAD 2018"),
        ];
        assert_eq!(DxfAcadVersion::SUPPORTED.len(), expected.len());
        assert_eq!(DxfAcadVersion::MIN_SUPPORTED, DxfAcadVersion::Ac1009);
        assert_eq!(DxfAcadVersion::MAX_SUPPORTED, DxfAcadVersion::Ac1032);
        for (index, (version, code, release)) in expected.iter().enumerate() {
            assert_eq!(DxfAcadVersion::SUPPORTED.get(index), Some(version));
            assert_eq!(version.code(), *code);
            assert_eq!(version.autodesk_release_name(), *release);
            assert_eq!(
                DxfAcadVersion::from_code_bytes(code.as_bytes()),
                Some(*version)
            );
        }
        assert_eq!(DxfAcadVersion::from_code_bytes(b"AC1006"), None);
        assert_eq!(DxfAcadVersion::from_code_bytes(b"AC1033"), None);
        assert_eq!(DxfAcadVersion::from_code_bytes(b" ac1032 "), None);
    }

    #[test]
    fn every_supported_version_is_discovered_with_exact_provenance() -> Result<(), Box<dyn Error>> {
        assert_send_sync::<DxfAcadVersionReport>();
        for version in DxfAcadVersion::SUPPORTED {
            let bytes = format!(
                "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nEOF\n",
                version.code()
            )
            .into_bytes();
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open(&source, DxfReadOptions::strict())?;
            let report = document.acad_version_report();
            assert_eq!(report.source_id(), document.source_id());
            assert_eq!(report.state(), DxfAcadVersionState::Supported(version));
            assert_eq!(report.occurrence_count(), 1);
            assert!(report.diagnostics().is_empty());

            let occurrence = required_primary(report)?;
            assert_eq!(occurrence.variable_occurrence(), 2);
            assert_eq!(occurrence.value_occurrence(), Some(3));
            assert_eq!(occurrence.value(), DxfAcadVersionValue::Supported(version));
            let mut variable = [0_u8; 8];
            document.read_span(occurrence.variable_span(), &mut variable)?;
            assert_eq!(&variable, b"$ACADVER");
            let mut value = [0_u8; 6];
            document.read_span(
                occurrence
                    .value_span()
                    .ok_or(io::Error::other("missing value span"))?,
                &mut value,
            )?;
            assert_eq!(&value, version.code().as_bytes());
        }
        Ok(())
    }

    #[test]
    fn absent_and_unsupported_versions_remain_distinct() -> Result<(), Box<dyn Error>> {
        let absent_bytes = b"0\nSECTION\n2\nHEADER\n9\n$HANDSEED\n5\nFF\n0\nENDSEC\n0\nEOF\n";
        let absent_source = DxfMemorySource::new(absent_bytes, DxfResourceProfile::Safe)?;
        let absent = open(&absent_source, DxfReadOptions::strict())?;
        let absent_report = absent.acad_version_report();
        assert_eq!(absent_report.state(), DxfAcadVersionState::Absent);
        assert_eq!(absent_report.occurrence_count(), 0);
        assert_eq!(absent_report.primary_occurrence(), None);
        assert_eq!(absent_report.diagnostics().len(), 1);
        assert_eq!(
            absent_report.diagnostics()[0].code(),
            DxfDiagnosticCode::ACADVER_MISSING
        );
        assert!(absent_report.diagnostics()[0].span().is_some());
        assert_eq!(absent.conformance(), DxfAsciiDocumentConformance::Strict);
        assert!(absent.diagnostics().is_empty());

        let unsupported_bytes =
            b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1006\n0\nENDSEC\n0\nEOF\n";
        let unsupported_source = DxfMemorySource::new(unsupported_bytes, DxfResourceProfile::Safe)?;
        let unsupported = open(&unsupported_source, DxfReadOptions::strict())?;
        let report = unsupported.acad_version_report();
        assert_eq!(report.state(), DxfAcadVersionState::Unsupported);
        assert_eq!(
            required_primary(report)?.value(),
            DxfAcadVersionValue::Unsupported
        );
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].code(),
            DxfDiagnosticCode::ACADVER_UNSUPPORTED
        );
        Ok(())
    }

    #[test]
    fn wrong_group_and_missing_value_are_invalid_without_guessing() -> Result<(), Box<dyn Error>> {
        let wrong_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n70\n1032\n0\nENDSEC\n0\nEOF\n";
        let wrong_source = DxfMemorySource::new(wrong_bytes, DxfResourceProfile::Safe)?;
        let wrong = open(&wrong_source, DxfReadOptions::strict())?;
        let wrong_report = wrong.acad_version_report();
        let code_70 = DxfGroupCode::new(70).ok_or(io::Error::other("group code"))?;
        assert_eq!(wrong_report.state(), DxfAcadVersionState::Invalid);
        assert_eq!(
            required_primary(wrong_report)?.value(),
            DxfAcadVersionValue::InvalidGroupCode(code_70)
        );
        assert_eq!(
            wrong_report.diagnostics()[0].code(),
            DxfDiagnosticCode::ACADVER_VALUE_INVALID
        );

        let missing_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n";
        let missing_source = DxfMemorySource::new(missing_bytes, DxfResourceProfile::Safe)?;
        let missing = open(&missing_source, DxfReadOptions::compatible())?;
        let missing_report = missing.acad_version_report();
        assert_eq!(missing_report.state(), DxfAcadVersionState::Invalid);
        assert_eq!(
            required_primary(missing_report)?.value(),
            DxfAcadVersionValue::MissingValue
        );
        assert_eq!(required_primary(missing_report)?.value_occurrence(), None);
        Ok(())
    }

    #[test]
    fn duplicates_are_ambiguous_with_fixed_evidence() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1009\n9\n$ACADVER\n1\nAC1032\n9\n$ACADVER\n1\nAC1027\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let report = document.acad_version_report();
        assert_eq!(report.state(), DxfAcadVersionState::Ambiguous);
        assert_eq!(report.occurrence_count(), 3);
        assert_eq!(
            required_primary(report)?.value(),
            DxfAcadVersionValue::Supported(DxfAcadVersion::Ac1009)
        );
        assert_eq!(
            report
                .conflicting_occurrence()
                .ok_or(io::Error::other("missing conflict"))?
                .value(),
            DxfAcadVersionValue::Supported(DxfAcadVersion::Ac1032)
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code() == DxfDiagnosticCode::ACADVER_DUPLICATE)
        );
        Ok(())
    }

    #[test]
    fn compatible_framing_recovery_keeps_exact_dialect_discovery() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::compatible())?;
        assert_eq!(
            document.conformance(),
            DxfAsciiDocumentConformance::Recovered
        );
        assert_eq!(document.diagnostics().len(), 1);
        assert_eq!(
            document.acad_version_report().state(),
            DxfAcadVersionState::Supported(DxfAcadVersion::Ac1032)
        );
        assert!(document.acad_version_report().diagnostics().is_empty());
        Ok(())
    }

    #[test]
    fn discovery_requires_exact_header_structure_and_ignores_other_sections()
    -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nENTITIES\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nHEADER\n9\n $ACADVER\n1\nAC1032\n9\n$acadver\n1\nAC1032\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::compatible())?;
        let report = document.acad_version_report();
        assert_eq!(report.state(), DxfAcadVersionState::Absent);
        assert_eq!(report.occurrence_count(), 0);
        assert_eq!(
            report.diagnostics()[0].code(),
            DxfDiagnosticCode::ACADVER_MISSING
        );
        Ok(())
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
        report: &DxfAcadVersionReport,
    ) -> Result<DxfAcadVersionOccurrence, io::Error> {
        report
            .primary_occurrence()
            .ok_or(io::Error::other("missing primary occurrence"))
    }

    fn assert_send_sync<T: Send + Sync>() {}
}

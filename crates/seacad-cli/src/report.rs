use seacad_dxf_core::{
    DxfAsciiDocumentConformance, DxfBinaryDocumentConformance, DxfDiagnosticSeverity,
    DxfPhysicalFormat, DxfReadMode, DxfResourceProfile,
};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct CliReport {
    pub(super) schema_version: &'static str,
    pub(super) command: &'static str,
    pub(super) status: &'static str,
    pub(super) options: OptionsReport,
    pub(super) source: SourceReport,
    pub(super) format: FormatReport,
    pub(super) document: Option<DocumentReport>,
    pub(super) diagnostics: Vec<DiagnosticReport>,
    pub(super) error: Option<ErrorReport>,
}

#[derive(Serialize)]
pub(super) struct OptionsReport {
    pub(super) read_mode: &'static str,
    pub(super) resource_profile: &'static str,
    pub(super) language: &'static str,
}

#[derive(Serialize)]
pub(super) struct SourceReport {
    pub(super) id: Option<String>,
    pub(super) bytes: Option<u64>,
    pub(super) path: Option<String>,
}

#[derive(Serialize)]
pub(super) struct FormatReport {
    pub(super) physical: &'static str,
}

#[derive(Serialize)]
pub(super) struct DocumentReport {
    pub(super) conformance: &'static str,
    pub(super) groups: u64,
    pub(super) eof_occurrence: Option<u64>,
    pub(super) trailing_bytes: u64,
    pub(super) diagnostics_truncated: bool,
}

#[derive(Serialize)]
pub(super) struct DiagnosticReport {
    pub(super) code: &'static str,
    pub(super) severity: &'static str,
    pub(super) span: Option<SpanReport>,
}

#[derive(Serialize)]
pub(super) struct SpanReport {
    pub(super) start: u64,
    pub(super) end: u64,
}

#[derive(Serialize)]
pub(super) struct ErrorReport {
    pub(super) code: String,
    pub(super) message: String,
}

pub(super) const fn read_mode_name(mode: DxfReadMode) -> &'static str {
    match mode {
        DxfReadMode::Strict => "strict",
        DxfReadMode::Compatible => "compatible",
        _ => "unknown",
    }
}

pub(super) const fn profile_name(profile: DxfResourceProfile) -> &'static str {
    match profile {
        DxfResourceProfile::Safe => "safe",
        DxfResourceProfile::Large => "large",
        _ => "unknown",
    }
}

pub(super) const fn physical_name(format: DxfPhysicalFormat) -> &'static str {
    match format {
        DxfPhysicalFormat::AsciiCandidate => "ascii_candidate",
        DxfPhysicalFormat::Binary => "binary",
        DxfPhysicalFormat::Unknown => "unknown",
        _ => "unknown",
    }
}

pub(super) const fn ascii_conformance_name(
    conformance: DxfAsciiDocumentConformance,
) -> &'static str {
    match conformance {
        DxfAsciiDocumentConformance::Strict => "strict",
        DxfAsciiDocumentConformance::Recovered => "recovered",
        _ => "unknown",
    }
}

pub(super) const fn binary_conformance_name(
    conformance: DxfBinaryDocumentConformance,
) -> &'static str {
    match conformance {
        DxfBinaryDocumentConformance::Strict => "strict",
        DxfBinaryDocumentConformance::Recovered => "recovered",
        _ => "unknown",
    }
}

pub(super) const fn severity_name(severity: DxfDiagnosticSeverity) -> &'static str {
    match severity {
        DxfDiagnosticSeverity::Info => "info",
        DxfDiagnosticSeverity::Warning => "warning",
        DxfDiagnosticSeverity::Error => "error",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ascii_conformance_name, binary_conformance_name, physical_name, profile_name,
        read_mode_name, severity_name,
    };
    use seacad_dxf_core::{
        DxfAsciiDocumentConformance, DxfBinaryDocumentConformance, DxfDiagnosticSeverity,
        DxfPhysicalFormat, DxfReadMode, DxfResourceProfile,
    };

    #[test]
    fn names_cover_every_published_core_variant() {
        assert_eq!(read_mode_name(DxfReadMode::Strict), "strict");
        assert_eq!(read_mode_name(DxfReadMode::Compatible), "compatible");
        assert_eq!(profile_name(DxfResourceProfile::Safe), "safe");
        assert_eq!(profile_name(DxfResourceProfile::Large), "large");
        assert_eq!(
            physical_name(DxfPhysicalFormat::AsciiCandidate),
            "ascii_candidate"
        );
        assert_eq!(physical_name(DxfPhysicalFormat::Binary), "binary");
        assert_eq!(physical_name(DxfPhysicalFormat::Unknown), "unknown");
        assert_eq!(
            ascii_conformance_name(DxfAsciiDocumentConformance::Strict),
            "strict"
        );
        assert_eq!(
            ascii_conformance_name(DxfAsciiDocumentConformance::Recovered),
            "recovered"
        );
        assert_eq!(
            binary_conformance_name(DxfBinaryDocumentConformance::Strict),
            "strict"
        );
        assert_eq!(
            binary_conformance_name(DxfBinaryDocumentConformance::Recovered),
            "recovered"
        );
        assert_eq!(severity_name(DxfDiagnosticSeverity::Info), "info");
        assert_eq!(severity_name(DxfDiagnosticSeverity::Warning), "warning");
        assert_eq!(severity_name(DxfDiagnosticSeverity::Error), "error");
    }
}

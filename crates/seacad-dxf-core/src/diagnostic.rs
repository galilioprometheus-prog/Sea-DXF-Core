use std::fmt;

/// Half-open byte range in the original source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByteSpan {
    start: u64,
    end: u64,
}

impl ByteSpan {
    /// Creates [start, end), rejecting reversed bounds.
    #[must_use]
    pub const fn new(start: u64, end: u64) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Creates a span from a start and length, rejecting u64 overflow.
    #[must_use]
    pub const fn from_start_and_len(start: u64, len: u64) -> Option<Self> {
        match start.checked_add(len) {
            Some(end) => Some(Self { start, end }),
            None => None,
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        self.end - self.start
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Stable machine-readable code for a non-fatal DXF diagnostic.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDiagnosticCode {
    value: &'static str,
    severity: DxfDiagnosticSeverity,
}

impl DxfDiagnosticCode {
    /// Additional diagnostics were suppressed by the selected resource limit.
    pub const DIAGNOSTICS_TRUNCATED: Self = Self {
        value: "DXF-W0001",
        severity: DxfDiagnosticSeverity::Warning,
    };

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.value
    }

    #[must_use]
    pub const fn severity(self) -> DxfDiagnosticSeverity {
        self.severity
    }
}

impl fmt::Display for DxfDiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

/// Source-anchored, language-neutral diagnostic metadata.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDiagnostic {
    code: DxfDiagnosticCode,
    span: Option<ByteSpan>,
}

impl DxfDiagnostic {
    #[must_use]
    pub const fn new(code: DxfDiagnosticCode, span: Option<ByteSpan>) -> Self {
        Self { code, span }
    }

    #[must_use]
    pub const fn code(self) -> DxfDiagnosticCode {
        self.code
    }

    #[must_use]
    pub const fn severity(self) -> DxfDiagnosticSeverity {
        self.code.severity()
    }

    #[must_use]
    pub const fn span(self) -> Option<ByteSpan> {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteSpan, DxfDiagnostic, DxfDiagnosticCode, DxfDiagnosticSeverity};

    #[test]
    fn byte_spans_are_half_open_and_overflow_safe() {
        let span = ByteSpan::from_start_and_len(7, 5);
        assert_eq!(span, ByteSpan::new(7, 12));
        assert_eq!(span.map(ByteSpan::len), Some(5));
        assert_eq!(ByteSpan::new(7, 7).map(ByteSpan::is_empty), Some(true));
        assert_eq!(ByteSpan::new(12, 7), None);
        assert_eq!(ByteSpan::from_start_and_len(u64::MAX, 1), None);
    }

    #[test]
    fn diagnostic_code_is_stable_and_source_anchored() {
        let span = ByteSpan::new(2, 9);
        let diagnostic = DxfDiagnostic::new(DxfDiagnosticCode::DIAGNOSTICS_TRUNCATED, span);
        assert_eq!(diagnostic.code().as_str(), "DXF-W0001");
        assert_eq!(diagnostic.code().to_string(), "DXF-W0001");
        assert_eq!(diagnostic.severity(), DxfDiagnosticSeverity::Warning);
        assert_eq!(diagnostic.span(), span);
    }
}

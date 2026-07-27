use std::fmt;

/// Half-open byte range in the original source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByteSpan {
    start: u64,
    end: u64,
}

impl ByteSpan {
    /// Internal constructor for bounds already proven ordered by a framing invariant.
    pub(crate) const fn from_validated_bounds(start: u64, end: u64) -> Self {
        Self { start, end }
    }

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

    /// Compatible mode ignored a UTF-8 BOM before the first ASCII group code.
    pub const UTF8_BOM_IGNORED: Self = Self {
        value: "DXF-W0201",
        severity: DxfDiagnosticSeverity::Warning,
    };

    /// Compatible mode recognized an EOF value with horizontal padding.
    pub const ASCII_EOF_WHITESPACE_IGNORED: Self = Self {
        value: "DXF-W0202",
        severity: DxfDiagnosticSeverity::Warning,
    };

    /// Compatible mode opened a completely paired ASCII stream without EOF.
    pub const ASCII_EOF_MISSING_RECOVERED: Self = Self {
        value: "DXF-W0203",
        severity: DxfDiagnosticSeverity::Warning,
    };

    /// Compatible mode preserved but did not parse source bytes after EOF.
    pub const ASCII_TRAILING_DATA_IGNORED: Self = Self {
        value: "DXF-W0204",
        severity: DxfDiagnosticSeverity::Warning,
    };

    /// Compatible mode opened a completely framed Binary stream without EOF.
    pub const BINARY_EOF_MISSING_RECOVERED: Self = Self {
        value: "DXF-W0210",
        severity: DxfDiagnosticSeverity::Warning,
    };

    /// Compatible mode preserved but did not parse Binary source bytes after EOF.
    pub const BINARY_TRAILING_DATA_IGNORED: Self = Self {
        value: "DXF-W0211",
        severity: DxfDiagnosticSeverity::Warning,
    };

    /// No exact `$ACADVER` variable was found in an exact HEADER section.
    pub const ACADVER_MISSING: Self = Self {
        value: "DXF-E0401",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// `$ACADVER` was not followed by its required group-code 1 value.
    pub const ACADVER_VALUE_INVALID: Self = Self {
        value: "DXF-E0402",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// More than one exact `$ACADVER` variable makes the dialect ambiguous.
    pub const ACADVER_DUPLICATE: Self = Self {
        value: "DXF-E0403",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// The exact group-code 1 value is outside SeaCad's supported registry.
    pub const ACADVER_UNSUPPORTED: Self = Self {
        value: "DXF-W0401",
        severity: DxfDiagnosticSeverity::Warning,
    };

    /// A SECTION marker has no immediate group-code 2 name.
    pub const SECTION_NAME_INVALID: Self = Self {
        value: "DXF-E0410",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// A new SECTION marker interrupted an open section.
    pub const SECTION_INTERRUPTED: Self = Self {
        value: "DXF-E0411",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// An ENDSEC marker appeared without an open section.
    pub const SECTION_END_ORPHAN: Self = Self {
        value: "DXF-E0412",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// An open section reached framed EOF or input end without ENDSEC.
    pub const SECTION_UNCLOSED: Self = Self {
        value: "DXF-E0413",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// A pre-2007 supported dialect has no exact `$DWGCODEPAGE`.
    pub const CODEPAGE_REQUIRED: Self = Self {
        value: "DXF-E0420",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// `$DWGCODEPAGE` has a missing, empty, or non-group-code 3 value.
    pub const CODEPAGE_VALUE_INVALID: Self = Self {
        value: "DXF-E0421",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// More than one exact `$DWGCODEPAGE` makes the declaration ambiguous.
    pub const CODEPAGE_DUPLICATE: Self = Self {
        value: "DXF-E0422",
        severity: DxfDiagnosticSeverity::Error,
    };

    /// A legacy `$DWGCODEPAGE` token has no reviewed SeaCad decoder.
    pub const CODEPAGE_UNSUPPORTED: Self = Self {
        value: "DXF-E0423",
        severity: DxfDiagnosticSeverity::Error,
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

        let bom = DxfDiagnostic::new(DxfDiagnosticCode::UTF8_BOM_IGNORED, span);
        assert_eq!(bom.code().as_str(), "DXF-W0201");
        assert_eq!(bom.severity(), DxfDiagnosticSeverity::Warning);

        let envelope_codes = [
            (DxfDiagnosticCode::ASCII_EOF_WHITESPACE_IGNORED, "DXF-W0202"),
            (DxfDiagnosticCode::ASCII_EOF_MISSING_RECOVERED, "DXF-W0203"),
            (DxfDiagnosticCode::ASCII_TRAILING_DATA_IGNORED, "DXF-W0204"),
            (DxfDiagnosticCode::BINARY_EOF_MISSING_RECOVERED, "DXF-W0210"),
            (DxfDiagnosticCode::BINARY_TRAILING_DATA_IGNORED, "DXF-W0211"),
        ];
        for (code, expected) in envelope_codes {
            let diagnostic = DxfDiagnostic::new(code, span);
            assert_eq!(diagnostic.code().as_str(), expected);
            assert_eq!(diagnostic.severity(), DxfDiagnosticSeverity::Warning);
            assert_eq!(diagnostic.span(), span);
        }

        let dialect_codes = [
            (
                DxfDiagnosticCode::ACADVER_MISSING,
                "DXF-E0401",
                DxfDiagnosticSeverity::Error,
            ),
            (
                DxfDiagnosticCode::ACADVER_VALUE_INVALID,
                "DXF-E0402",
                DxfDiagnosticSeverity::Error,
            ),
            (
                DxfDiagnosticCode::ACADVER_DUPLICATE,
                "DXF-E0403",
                DxfDiagnosticSeverity::Error,
            ),
            (
                DxfDiagnosticCode::ACADVER_UNSUPPORTED,
                "DXF-W0401",
                DxfDiagnosticSeverity::Warning,
            ),
        ];
        for (code, expected, severity) in dialect_codes {
            let diagnostic = DxfDiagnostic::new(code, span);
            assert_eq!(diagnostic.code().as_str(), expected);
            assert_eq!(diagnostic.severity(), severity);
            assert_eq!(diagnostic.span(), span);
        }

        let structure_codes = [
            (DxfDiagnosticCode::SECTION_NAME_INVALID, "DXF-E0410"),
            (DxfDiagnosticCode::SECTION_INTERRUPTED, "DXF-E0411"),
            (DxfDiagnosticCode::SECTION_END_ORPHAN, "DXF-E0412"),
            (DxfDiagnosticCode::SECTION_UNCLOSED, "DXF-E0413"),
        ];
        for (code, expected) in structure_codes {
            let diagnostic = DxfDiagnostic::new(code, span);
            assert_eq!(diagnostic.code().as_str(), expected);
            assert_eq!(diagnostic.severity(), DxfDiagnosticSeverity::Error);
            assert_eq!(diagnostic.span(), span);
        }

        let encoding_codes = [
            (DxfDiagnosticCode::CODEPAGE_REQUIRED, "DXF-E0420"),
            (DxfDiagnosticCode::CODEPAGE_VALUE_INVALID, "DXF-E0421"),
            (DxfDiagnosticCode::CODEPAGE_DUPLICATE, "DXF-E0422"),
            (DxfDiagnosticCode::CODEPAGE_UNSUPPORTED, "DXF-E0423"),
        ];
        for (code, expected) in encoding_codes {
            let diagnostic = DxfDiagnostic::new(code, span);
            assert_eq!(diagnostic.code().as_str(), expected);
            assert_eq!(diagnostic.severity(), DxfDiagnosticSeverity::Error);
            assert_eq!(diagnostic.span(), span);
        }
    }
}

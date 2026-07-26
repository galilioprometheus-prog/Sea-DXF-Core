use std::{fmt, io};

use crate::{
    DxfCancellationToken,
    ascii_line::{DxfAsciiLineCursor, DxfAsciiLineMetadata},
    diagnostic::{ByteSpan, DxfDiagnostic, DxfDiagnosticCode},
    error::{DxfError, DxfIoOperation, DxfResource},
    limits::DxfResourceProfile,
    read_options::{DxfReadMode, DxfReadOptions},
    source::DxfByteSource,
};

const UTF8_BOM: &[u8; 3] = b"\xef\xbb\xbf";

/// Valid numeric domain for a DXF group code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DxfGroupCode(i16);

impl DxfGroupCode {
    pub const MIN: i16 = -5;
    pub const MAX: i16 = 1_071;

    #[must_use]
    pub const fn new(value: i16) -> Option<Self> {
        if value >= Self::MIN && value <= Self::MAX {
            Some(Self(value))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> i16 {
        self.0
    }
}

/// Borrowed raw group-code/value pair with exact source locations.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct DxfAsciiGroup<'a> {
    occurrence: u64,
    group_code: DxfGroupCode,
    raw_group_code: &'a [u8],
    raw_value: &'a [u8],
    group_code_line: DxfAsciiLineMetadata,
    value_line: DxfAsciiLineMetadata,
    full_span: ByteSpan,
}

impl fmt::Debug for DxfAsciiGroup<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfAsciiGroup")
            .field("occurrence", &self.occurrence)
            .field("group_code", &self.group_code)
            .field("raw_group_code_bytes", &self.raw_group_code.len())
            .field("raw_value_bytes", &self.raw_value.len())
            .field("group_code_line", &self.group_code_line)
            .field("value_line", &self.value_line)
            .field("full_span", &self.full_span)
            .finish()
    }
}

impl<'a> DxfAsciiGroup<'a> {
    #[must_use]
    pub const fn occurrence(self) -> u64 {
        self.occurrence
    }

    #[must_use]
    pub const fn group_code(self) -> DxfGroupCode {
        self.group_code
    }

    #[must_use]
    pub const fn raw_group_code(self) -> &'a [u8] {
        self.raw_group_code
    }

    #[must_use]
    pub const fn raw_value(self) -> &'a [u8] {
        self.raw_value
    }

    #[must_use]
    pub const fn group_code_line(self) -> DxfAsciiLineMetadata {
        self.group_code_line
    }

    #[must_use]
    pub const fn value_line(self) -> DxfAsciiLineMetadata {
        self.value_line
    }

    #[must_use]
    pub const fn full_span(self) -> ByteSpan {
        self.full_span
    }
}

/// Bounded cursor over lossless ASCII DXF group-code/value pairs.
pub struct DxfAsciiGroupCursor<'a> {
    lines: DxfAsciiLineCursor<'a>,
    mode: DxfReadMode,
    max_records: u64,
    max_diagnostics: u64,
    next_occurrence: u64,
    raw_group_code: Vec<u8>,
    diagnostics: Vec<DxfDiagnostic>,
    diagnostics_truncated: bool,
}

impl<'a> DxfAsciiGroupCursor<'a> {
    pub fn new(source: &'a dyn DxfByteSource, options: DxfReadOptions) -> Result<Self, DxfError> {
        let limits = options.resource_profile().limits();
        Self::with_limits(
            source,
            options.mode(),
            options.resource_profile(),
            limits.max_records(),
            limits.max_diagnostics(),
        )
    }

    fn with_limits(
        source: &'a dyn DxfByteSource,
        mode: DxfReadMode,
        profile: DxfResourceProfile,
        max_records: u64,
        max_diagnostics: u64,
    ) -> Result<Self, DxfError> {
        Ok(Self {
            lines: DxfAsciiLineCursor::new(source, profile)?,
            mode,
            max_records,
            max_diagnostics,
            next_occurrence: 0,
            raw_group_code: Vec::new(),
            diagnostics: Vec::new(),
            diagnostics_truncated: false,
        })
    }

    #[must_use]
    pub const fn records_framed(&self) -> u64 {
        self.next_occurrence
    }

    #[must_use]
    pub const fn source_len(&self) -> u64 {
        self.lines.source_len()
    }

    #[must_use]
    pub const fn consumed_bytes(&self) -> u64 {
        self.lines.consumed_bytes()
    }

    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.lines.is_complete()
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[DxfDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub const fn diagnostics_were_truncated(&self) -> bool {
        self.diagnostics_truncated
    }

    /// Returns the next pair. Discard the cursor after any error.
    pub fn next_group(
        &mut self,
        cancellation: &DxfCancellationToken,
    ) -> Result<Option<DxfAsciiGroup<'_>>, DxfError> {
        if self.lines.is_complete() {
            return Ok(None);
        }
        let observed = self
            .next_occurrence
            .checked_add(1)
            .ok_or(DxfError::OffsetOverflow {
                offset: self.next_occurrence,
                requested: 1,
            })?;
        if observed > self.max_records {
            return Err(DxfError::resource_limit(
                DxfResource::Records,
                self.max_records,
                observed,
            ));
        }

        let Some(group_code_line) = self.lines.next_line(cancellation)? else {
            return Ok(None);
        };
        let group_code_metadata = group_code_line.metadata();
        let (group_code, bom_span) =
            parse_record_group_code(group_code_line.bytes(), group_code_metadata, self.mode)?;

        self.raw_group_code.clear();
        self.raw_group_code
            .try_reserve(group_code_line.bytes().len())
            .map_err(|_| out_of_memory())?;
        self.raw_group_code
            .extend_from_slice(group_code_line.bytes());
        if let Some(span) = bom_span {
            self.retain_diagnostic(DxfDiagnostic::new(
                DxfDiagnosticCode::UTF8_BOM_IGNORED,
                Some(span),
            ))?;
        }

        let Some(value_line) = self.lines.next_line(cancellation)? else {
            return Err(DxfError::MissingAsciiGroupValue {
                group_code_span: group_code_metadata.content_span(),
            });
        };
        let value_metadata = value_line.metadata();
        let full_span = ByteSpan::new(
            group_code_metadata.full_span().start(),
            value_metadata.full_span().end(),
        )
        .ok_or_else(invalid_source_data)?;
        self.next_occurrence = observed;

        Ok(Some(DxfAsciiGroup {
            occurrence: observed - 1,
            group_code,
            raw_group_code: &self.raw_group_code,
            raw_value: value_line.bytes(),
            group_code_line: group_code_metadata,
            value_line: value_metadata,
            full_span,
        }))
    }

    fn retain_diagnostic(&mut self, diagnostic: DxfDiagnostic) -> Result<(), DxfError> {
        let retained = usize_to_u64(self.diagnostics.len())?;
        if retained < self.max_diagnostics {
            self.diagnostics
                .try_reserve(1)
                .map_err(|_| out_of_memory())?;
            self.diagnostics.push(diagnostic);
        } else if !self.diagnostics_truncated {
            if let Some(last) = self.diagnostics.last_mut() {
                *last =
                    DxfDiagnostic::new(DxfDiagnosticCode::DIAGNOSTICS_TRUNCATED, diagnostic.span());
            }
            self.diagnostics_truncated = true;
        }
        Ok(())
    }
}

fn parse_record_group_code(
    raw: &[u8],
    metadata: DxfAsciiLineMetadata,
    mode: DxfReadMode,
) -> Result<(DxfGroupCode, Option<ByteSpan>), DxfError> {
    let can_recover_bom = mode == DxfReadMode::Compatible
        && metadata.line_index() == 0
        && metadata.content_span().start() == 0
        && raw.starts_with(UTF8_BOM);
    let (candidate, bom_span) = if can_recover_bom {
        let candidate = raw.get(UTF8_BOM.len()..).ok_or_else(invalid_source_data)?;
        let span = ByteSpan::from_start_and_len(0, UTF8_BOM.len() as u64).ok_or(
            DxfError::OffsetOverflow {
                offset: 0,
                requested: UTF8_BOM.len() as u64,
            },
        )?;
        (candidate, Some(span))
    } else {
        (raw, None)
    };

    parse_ascii_group_code(candidate)
        .map(|code| (code, bom_span))
        .ok_or(DxfError::InvalidAsciiGroupCode {
            span: metadata.content_span(),
        })
}

fn parse_ascii_group_code(raw: &[u8]) -> Option<DxfGroupCode> {
    let candidate = trim_horizontal_ascii(raw)?;
    let (negative, digits) = match candidate.first() {
        Some(b'+') => (false, candidate.get(1..)?),
        Some(b'-') => (true, candidate.get(1..)?),
        Some(_) => (false, candidate),
        None => return None,
    };
    if digits.is_empty() {
        return None;
    }

    let mut magnitude = 0_i32;
    for byte in digits {
        if !byte.is_ascii_digit() {
            return None;
        }
        magnitude = magnitude
            .checked_mul(10)?
            .checked_add(i32::from(*byte - b'0'))?;
    }
    let signed = if negative {
        magnitude.checked_neg()?
    } else {
        magnitude
    };
    DxfGroupCode::new(i16::try_from(signed).ok()?)
}

fn trim_horizontal_ascii(bytes: &[u8]) -> Option<&[u8]> {
    let mut start = 0_usize;
    while matches!(bytes.get(start), Some(b' ' | b'\t')) {
        start = start.checked_add(1)?;
    }

    let mut end = bytes.len();
    while end > start {
        let previous = end.checked_sub(1)?;
        if !matches!(bytes.get(previous), Some(b' ' | b'\t')) {
            break;
        }
        end = previous;
    }
    bytes.get(start..end)
}

fn usize_to_u64(value: usize) -> Result<u64, DxfError> {
    u64::try_from(value).map_err(|_| DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    })
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

    use super::{DxfAsciiGroupCursor, DxfGroupCode, parse_ascii_group_code};
    use crate::{
        ByteSpan, DxfAsciiLineEnding, DxfCancellationToken, DxfDiagnostic, DxfDiagnosticCode,
        DxfError, DxfMemorySource, DxfReadMode, DxfReadOptions, DxfResource, DxfResourceProfile,
    };

    #[test]
    fn groups_preserve_raw_bytes_spans_endings_and_occurrence() -> Result<(), Box<dyn Error>> {
        let bytes = b"  0\r\nSECTION\n+2\rHEADER";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiGroupCursor::new(&source, DxfReadOptions::strict())?;
        let token = DxfCancellationToken::default();

        {
            let first = cursor
                .next_group(&token)?
                .ok_or(io::Error::other("missing first group"))?;
            assert_eq!(first.occurrence(), 0);
            assert_eq!(first.group_code().value(), 0);
            assert_eq!(first.raw_group_code(), b"  0");
            assert_eq!(first.raw_value(), b"SECTION");
            assert_eq!(
                first.group_code_line().content_span(),
                ByteSpan::new(0, 3).ok_or(io::Error::other("span"))?
            );
            assert_eq!(first.group_code_line().ending(), DxfAsciiLineEnding::CrLf);
            assert_eq!(first.value_line().ending(), DxfAsciiLineEnding::Lf);
            assert_eq!(
                first.full_span(),
                ByteSpan::new(0, 13).ok_or(io::Error::other("span"))?
            );
            let debug = format!("{first:?}");
            assert!(debug.contains("raw_value_bytes"));
            assert!(!debug.contains("SECTION"));
        }

        {
            let second = cursor
                .next_group(&token)?
                .ok_or(io::Error::other("missing second group"))?;
            assert_eq!(second.occurrence(), 1);
            assert_eq!(second.group_code().value(), 2);
            assert_eq!(second.raw_group_code(), b"+2");
            assert_eq!(second.raw_value(), b"HEADER");
            assert_eq!(second.group_code_line().ending(), DxfAsciiLineEnding::Cr);
            assert_eq!(second.value_line().ending(), DxfAsciiLineEnding::None);
            assert_eq!(
                second.full_span(),
                ByteSpan::new(13, 22).ok_or(io::Error::other("span"))?
            );
        }

        assert!(cursor.next_group(&token)?.is_none());
        assert_eq!(cursor.records_framed(), 2);
        assert_eq!(cursor.source_len(), bytes.len() as u64);
        assert_eq!(cursor.consumed_bytes(), bytes.len() as u64);
        assert!(cursor.is_complete());
        assert!(cursor.diagnostics().is_empty());
        Ok(())
    }

    #[test]
    fn strict_rejects_bom_and_compatible_recovers_once() -> Result<(), Box<dyn Error>> {
        let bytes = b"\xef\xbb\xbf  0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut strict = DxfAsciiGroupCursor::new(&source, DxfReadOptions::strict())?;
        assert!(matches!(
            strict.next_group(&token),
            Err(DxfError::InvalidAsciiGroupCode { .. })
        ));

        let mut compatible = DxfAsciiGroupCursor::new(&source, DxfReadOptions::compatible())?;
        {
            let group = compatible
                .next_group(&token)?
                .ok_or(io::Error::other("missing recovered group"))?;
            assert_eq!(group.group_code().value(), 0);
            assert_eq!(group.raw_group_code(), b"\xef\xbb\xbf  0");
            assert_eq!(group.raw_value(), b"EOF");
        }
        assert_eq!(compatible.diagnostics().len(), 1);
        assert_eq!(
            compatible.diagnostics()[0].code(),
            DxfDiagnosticCode::UTF8_BOM_IGNORED
        );
        assert_eq!(compatible.diagnostics()[0].span(), ByteSpan::new(0, 3));
        assert!(!compatible.diagnostics_were_truncated());
        Ok(())
    }

    #[test]
    fn signed_padded_codes_are_accepted_within_the_normative_range() -> Result<(), io::Error> {
        let cases: [(&[u8], i16); 4] = [
            (b"  0", 0),
            (b"\t+0009\t", 9),
            (b"-5", -5),
            (b"1071", 1_071),
        ];
        for (raw, expected) in cases {
            let code = parse_ascii_group_code(raw)
                .ok_or(io::Error::other("valid group code was rejected"))?;
            assert_eq!(code.value(), expected);
        }
        assert_eq!(DxfGroupCode::new(-6), None);
        assert_eq!(DxfGroupCode::new(1_072), None);
        Ok(())
    }

    #[test]
    fn invalid_group_codes_fail_without_semantic_guessing() -> Result<(), DxfError> {
        let token = DxfCancellationToken::default();
        for bytes in [
            b"\nvalue\n".as_slice(),
            b"X\nvalue\n".as_slice(),
            b"0 0\nvalue\n".as_slice(),
            b"1072\nvalue\n".as_slice(),
            b"-6\nvalue\n".as_slice(),
            b"999999999999999999\nvalue\n".as_slice(),
            b"\x0b0\nvalue\n".as_slice(),
        ] {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            let mut cursor = DxfAsciiGroupCursor::new(&source, DxfReadOptions::strict())?;
            assert!(matches!(
                cursor.next_group(&token),
                Err(DxfError::InvalidAsciiGroupCode { .. })
            ));
        }
        Ok(())
    }

    #[test]
    fn compatible_only_recovers_bom_at_absolute_start() -> Result<(), DxfError> {
        let bytes = b"0\nSECTION\n\xef\xbb\xbf0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut cursor = DxfAsciiGroupCursor::new(&source, DxfReadOptions::compatible())?;
        assert!(cursor.next_group(&token)?.is_some());
        assert!(matches!(
            cursor.next_group(&token),
            Err(DxfError::InvalidAsciiGroupCode { .. })
        ));
        assert!(cursor.diagnostics().is_empty());
        Ok(())
    }

    #[test]
    fn compatible_does_not_skip_blank_group_code_lines() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(b"\nvalue\n0\nEOF\n", DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiGroupCursor::new(&source, DxfReadOptions::compatible())?;
        let token = DxfCancellationToken::default();

        assert!(matches!(
            cursor.next_group(&token),
            Err(DxfError::InvalidAsciiGroupCode { .. })
        ));
        assert!(cursor.diagnostics().is_empty());
        Ok(())
    }

    #[test]
    fn missing_value_is_fatal_in_both_modes() -> Result<(), Box<dyn Error>> {
        let expected_span = ByteSpan::new(0, 1).ok_or(io::Error::other("span"))?;

        for mode in [DxfReadMode::Strict, DxfReadMode::Compatible] {
            let source = DxfMemorySource::new(b"0\n", DxfResourceProfile::Safe)?;
            let options = DxfReadOptions::new(mode, DxfResourceProfile::Safe);
            let mut cursor = DxfAsciiGroupCursor::new(&source, options)?;
            let token = DxfCancellationToken::default();
            let error = match cursor.next_group(&token) {
                Err(error) => error,
                Ok(_) => return Err(io::Error::other("missing value was accepted").into()),
            };
            assert!(matches!(
                error,
                DxfError::MissingAsciiGroupValue { group_code_span }
                    if group_code_span == expected_span
            ));
        }
        Ok(())
    }

    #[test]
    fn record_limit_fails_before_consuming_the_next_code_line() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(b"0\nA\n1\nB\n", DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiGroupCursor::with_limits(
            &source,
            DxfReadMode::Strict,
            DxfResourceProfile::Safe,
            1,
            1,
        )?;
        let token = DxfCancellationToken::default();
        assert!(cursor.next_group(&token)?.is_some());
        let consumed = cursor.consumed_bytes();
        assert!(matches!(
            cursor.next_group(&token),
            Err(DxfError::ResourceLimitExceeded {
                resource: DxfResource::Records,
                limit: 1,
                observed: 2,
            })
        ));
        assert_eq!(cursor.consumed_bytes(), consumed);
        Ok(())
    }

    #[test]
    fn diagnostic_cap_replaces_the_last_item_with_truncation() -> Result<(), Box<dyn Error>> {
        let source = DxfMemorySource::new(b"", DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiGroupCursor::with_limits(
            &source,
            DxfReadMode::Compatible,
            DxfResourceProfile::Safe,
            1,
            1,
        )?;
        let first = ByteSpan::new(0, 1).ok_or(io::Error::other("span"))?;
        let second = ByteSpan::new(2, 3).ok_or(io::Error::other("span"))?;
        cursor.retain_diagnostic(DxfDiagnostic::new(
            DxfDiagnosticCode::UTF8_BOM_IGNORED,
            Some(first),
        ))?;
        cursor.retain_diagnostic(DxfDiagnostic::new(
            DxfDiagnosticCode::UTF8_BOM_IGNORED,
            Some(second),
        ))?;
        assert_eq!(cursor.diagnostics().len(), 1);
        assert_eq!(
            cursor.diagnostics()[0].code(),
            DxfDiagnosticCode::DIAGNOSTICS_TRUNCATED
        );
        assert_eq!(cursor.diagnostics()[0].span(), Some(second));
        assert!(cursor.diagnostics_were_truncated());
        Ok(())
    }

    #[test]
    fn empty_source_contains_no_groups() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(b"", DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiGroupCursor::new(&source, DxfReadOptions::strict())?;
        let token = DxfCancellationToken::default();
        assert!(cursor.next_group(&token)?.is_none());
        assert!(cursor.is_complete());
        assert_eq!(cursor.records_framed(), 0);
        Ok(())
    }
}

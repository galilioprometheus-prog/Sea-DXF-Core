//! Lossless tokenization of documented AutoCAD text controls.

use std::fmt;

const MAX_MTEXT_BLOCK_DEPTH: u8 = 8;

/// Entity context that determines which documented control family applies.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextControlContext {
    MText,
    SingleLineText,
}

/// One documented lexical control, without semantic value interpretation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextControlTokenKind {
    Text,
    MTextBlockStart,
    MTextBlockEnd,
    MTextOverlineOn,
    MTextOverlineOff,
    MTextUnderlineOn,
    MTextUnderlineOff,
    MTextStrikethroughOn,
    MTextStrikethroughOff,
    MTextNonbreakingSpace,
    MTextEscapedBackslash,
    MTextEscapedOpenBrace,
    MTextEscapedCloseBrace,
    MTextParagraphBreak,
    MTextAlignment,
    MTextColor,
    MTextFont,
    MTextHeight,
    MTextStack,
    MTextTracking,
    MTextOblique,
    MTextWidth,
    PercentDiameter,
    PercentDegree,
    PercentPlusMinus,
    PercentStrikethroughToggle,
    PercentOverlineToggle,
    PercentUnderlineToggle,
    PercentLiteral,
    PercentUnicodeDecimal,
}

/// Half-open byte span inside already-decoded UTF-8 text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDecodedTextSpan {
    start: usize,
    end: usize,
}

impl DxfDecodedTextSpan {
    const fn from_parts(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[must_use]
    pub fn slice(self, source: &str) -> Option<&str> {
        source.get(self.start..self.end)
    }
}

/// Exact raw and optional payload spans for one token.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextControlToken {
    kind: DxfTextControlTokenKind,
    source_span: DxfDecodedTextSpan,
    payload_span: Option<DxfDecodedTextSpan>,
}

impl DxfTextControlToken {
    #[must_use]
    pub const fn kind(self) -> DxfTextControlTokenKind {
        self.kind
    }

    #[must_use]
    pub const fn source_span(self) -> DxfDecodedTextSpan {
        self.source_span
    }

    #[must_use]
    pub const fn payload_span(self) -> Option<DxfDecodedTextSpan> {
        self.payload_span
    }
}

/// Structural reason that a documented MTEXT sequence cannot be tokenized.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextControlIssue {
    MTextMissingTerminator,
    MTextNestingLimitExceeded,
    MTextUnmatchedBlockEnd,
    MTextUnclosedBlock,
}

/// Terminal typed failure at an offset in decoded UTF-8 input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextControlError {
    source_offset: usize,
    issue: DxfTextControlIssue,
}

impl DxfTextControlError {
    #[must_use]
    pub const fn source_offset(self) -> usize {
        self.source_offset
    }

    #[must_use]
    pub const fn issue(self) -> DxfTextControlIssue {
        self.issue
    }
}

impl fmt::Display for DxfTextControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let issue = match self.issue {
            DxfTextControlIssue::MTextMissingTerminator => "missing MTEXT terminator",
            DxfTextControlIssue::MTextNestingLimitExceeded => "MTEXT nesting limit exceeded",
            DxfTextControlIssue::MTextUnmatchedBlockEnd => "unmatched MTEXT block end",
            DxfTextControlIssue::MTextUnclosedBlock => "unclosed MTEXT block",
        };
        write!(
            formatter,
            "{issue} at decoded UTF-8 byte {}",
            self.source_offset
        )
    }
}

impl std::error::Error for DxfTextControlError {}

/// Allocation-free cursor over documented text controls and exact text spans.
pub struct DxfTextControlCursor<'a> {
    source: &'a str,
    context: DxfTextControlContext,
    offset: usize,
    block_depth: u8,
    block_offsets: [usize; MAX_MTEXT_BLOCK_DEPTH as usize],
    terminal: Option<Result<(), DxfTextControlError>>,
}

impl<'a> DxfTextControlCursor<'a> {
    #[must_use]
    pub const fn new(source: &'a str, context: DxfTextControlContext) -> Self {
        Self {
            source,
            context,
            offset: 0,
            block_depth: 0,
            block_offsets: [0; MAX_MTEXT_BLOCK_DEPTH as usize],
            terminal: None,
        }
    }

    #[must_use]
    pub const fn context(&self) -> DxfTextControlContext {
        self.context
    }

    #[must_use]
    pub const fn consumed_bytes(&self) -> usize {
        self.offset
    }

    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.terminal, Some(Ok(())))
    }

    /// Returns the next exact token. Discard the cursor after an error.
    pub fn next_token(&mut self) -> Result<Option<DxfTextControlToken>, DxfTextControlError> {
        if let Some(terminal) = self.terminal {
            return terminal.map(|()| None);
        }
        if self.offset == self.source.len() {
            if self.block_depth == 0 {
                self.terminal = Some(Ok(()));
                return Ok(None);
            }
            let error = DxfTextControlError {
                source_offset: self.block_offsets[0],
                issue: DxfTextControlIssue::MTextUnclosedBlock,
            };
            self.terminal = Some(Err(error));
            return Err(error);
        }

        match control_at(self.source, self.offset, self.context) {
            ControlMatch::Token { kind, end, payload } => self.emit_control(kind, end, payload),
            ControlMatch::MissingTerminator => {
                let error = DxfTextControlError {
                    source_offset: self.offset,
                    issue: DxfTextControlIssue::MTextMissingTerminator,
                };
                self.terminal = Some(Err(error));
                Err(error)
            }
            ControlMatch::None => self.emit_text(),
        }
    }

    fn emit_control(
        &mut self,
        kind: DxfTextControlTokenKind,
        end: usize,
        payload: Option<DxfDecodedTextSpan>,
    ) -> Result<Option<DxfTextControlToken>, DxfTextControlError> {
        if kind == DxfTextControlTokenKind::MTextBlockStart {
            if self.block_depth >= MAX_MTEXT_BLOCK_DEPTH {
                let error = DxfTextControlError {
                    source_offset: self.offset,
                    issue: DxfTextControlIssue::MTextNestingLimitExceeded,
                };
                self.terminal = Some(Err(error));
                return Err(error);
            }
            let index = usize::from(self.block_depth);
            let Some(slot) = self.block_offsets.get_mut(index) else {
                let error = DxfTextControlError {
                    source_offset: self.offset,
                    issue: DxfTextControlIssue::MTextNestingLimitExceeded,
                };
                self.terminal = Some(Err(error));
                return Err(error);
            };
            *slot = self.offset;
            self.block_depth = match self.block_depth.checked_add(1) {
                Some(depth) => depth,
                None => {
                    let error = DxfTextControlError {
                        source_offset: self.offset,
                        issue: DxfTextControlIssue::MTextNestingLimitExceeded,
                    };
                    self.terminal = Some(Err(error));
                    return Err(error);
                }
            };
        } else if kind == DxfTextControlTokenKind::MTextBlockEnd {
            self.block_depth = match self.block_depth.checked_sub(1) {
                Some(depth) => depth,
                None => {
                    let error = DxfTextControlError {
                        source_offset: self.offset,
                        issue: DxfTextControlIssue::MTextUnmatchedBlockEnd,
                    };
                    self.terminal = Some(Err(error));
                    return Err(error);
                }
            };
        }

        let start = self.offset;
        self.offset = end;
        Ok(Some(DxfTextControlToken {
            kind,
            source_span: DxfDecodedTextSpan::from_parts(start, end),
            payload_span: payload,
        }))
    }

    fn emit_text(&mut self) -> Result<Option<DxfTextControlToken>, DxfTextControlError> {
        let start = self.offset;
        while self.offset < self.source.len() {
            if control_at(self.source, self.offset, self.context) != ControlMatch::None {
                break;
            }
            let Some(character) = self
                .source
                .get(self.offset..)
                .and_then(|text| text.chars().next())
            else {
                break;
            };
            self.offset = match self.offset.checked_add(character.len_utf8()) {
                Some(offset) => offset,
                None => break,
            };
        }
        Ok(Some(DxfTextControlToken {
            kind: DxfTextControlTokenKind::Text,
            source_span: DxfDecodedTextSpan::from_parts(start, self.offset),
            payload_span: None,
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControlMatch {
    None,
    MissingTerminator,
    Token {
        kind: DxfTextControlTokenKind,
        end: usize,
        payload: Option<DxfDecodedTextSpan>,
    },
}

fn control_at(source: &str, offset: usize, context: DxfTextControlContext) -> ControlMatch {
    let Some(remaining) = source.as_bytes().get(offset..) else {
        return ControlMatch::None;
    };
    match context {
        DxfTextControlContext::MText => {
            mtext_control_at(remaining, offset).or_percent_mtext(remaining, offset)
        }
        DxfTextControlContext::SingleLineText => percent_text_at(remaining, offset),
    }
}

impl ControlMatch {
    fn or_percent_mtext(self, source: &[u8], offset: usize) -> Self {
        if self == Self::None {
            percent_symbol_at(source, offset)
        } else {
            self
        }
    }
}

fn mtext_control_at(source: &[u8], offset: usize) -> ControlMatch {
    match source.first().copied() {
        Some(b'{') => return fixed(DxfTextControlTokenKind::MTextBlockStart, offset, 1),
        Some(b'}') => return fixed(DxfTextControlTokenKind::MTextBlockEnd, offset, 1),
        Some(b'\\') => {}
        _ => return ControlMatch::None,
    }
    let Some(code) = source.get(1).copied() else {
        return ControlMatch::None;
    };
    let kind = match code {
        b'O' => DxfTextControlTokenKind::MTextOverlineOn,
        b'o' => DxfTextControlTokenKind::MTextOverlineOff,
        b'L' => DxfTextControlTokenKind::MTextUnderlineOn,
        b'l' => DxfTextControlTokenKind::MTextUnderlineOff,
        b'K' => DxfTextControlTokenKind::MTextStrikethroughOn,
        b'k' => DxfTextControlTokenKind::MTextStrikethroughOff,
        b'~' => DxfTextControlTokenKind::MTextNonbreakingSpace,
        b'\\' => DxfTextControlTokenKind::MTextEscapedBackslash,
        b'{' => DxfTextControlTokenKind::MTextEscapedOpenBrace,
        b'}' => DxfTextControlTokenKind::MTextEscapedCloseBrace,
        b'P' => DxfTextControlTokenKind::MTextParagraphBreak,
        b'A' => return parameter(source, offset, DxfTextControlTokenKind::MTextAlignment),
        b'C' => return parameter(source, offset, DxfTextControlTokenKind::MTextColor),
        b'F' => return parameter(source, offset, DxfTextControlTokenKind::MTextFont),
        b'H' => return parameter(source, offset, DxfTextControlTokenKind::MTextHeight),
        b'S' => return parameter(source, offset, DxfTextControlTokenKind::MTextStack),
        b'T' => return parameter(source, offset, DxfTextControlTokenKind::MTextTracking),
        b'Q' => return parameter(source, offset, DxfTextControlTokenKind::MTextOblique),
        b'W' => return parameter(source, offset, DxfTextControlTokenKind::MTextWidth),
        _ => return ControlMatch::None,
    };
    fixed(kind, offset, 2)
}

fn parameter(source: &[u8], offset: usize, kind: DxfTextControlTokenKind) -> ControlMatch {
    let payload_start = match offset.checked_add(2) {
        Some(start) => start,
        None => return ControlMatch::MissingTerminator,
    };
    let Some(relative_end) = source
        .get(2..)
        .and_then(|bytes| bytes.iter().position(|b| *b == b';'))
    else {
        return ControlMatch::MissingTerminator;
    };
    let payload_end = match payload_start.checked_add(relative_end) {
        Some(end) => end,
        None => return ControlMatch::MissingTerminator,
    };
    let end = match payload_end.checked_add(1) {
        Some(end) => end,
        None => return ControlMatch::MissingTerminator,
    };
    ControlMatch::Token {
        kind,
        end,
        payload: Some(DxfDecodedTextSpan::from_parts(payload_start, payload_end)),
    }
}

fn percent_symbol_at(source: &[u8], offset: usize) -> ControlMatch {
    if !source.starts_with(b"%%") {
        return ControlMatch::None;
    }
    let kind = match source.get(2).copied() {
        Some(b'c' | b'C') => DxfTextControlTokenKind::PercentDiameter,
        Some(b'd' | b'D') => DxfTextControlTokenKind::PercentDegree,
        Some(b'p' | b'P') => DxfTextControlTokenKind::PercentPlusMinus,
        _ => return ControlMatch::None,
    };
    fixed(kind, offset, 3)
}

fn percent_text_at(source: &[u8], offset: usize) -> ControlMatch {
    let symbol = percent_symbol_at(source, offset);
    if symbol != ControlMatch::None {
        return symbol;
    }
    if !source.starts_with(b"%%") {
        return ControlMatch::None;
    }
    let kind = match source.get(2).copied() {
        Some(b'k' | b'K') => DxfTextControlTokenKind::PercentStrikethroughToggle,
        Some(b'o' | b'O') => DxfTextControlTokenKind::PercentOverlineToggle,
        Some(b'u' | b'U') => DxfTextControlTokenKind::PercentUnderlineToggle,
        Some(b'%') => DxfTextControlTokenKind::PercentLiteral,
        Some(_) | None => return percent_decimal_at(source, offset),
    };
    fixed(kind, offset, 3)
}

fn percent_decimal_at(source: &[u8], offset: usize) -> ControlMatch {
    let Some(digits) = source.get(2..5) else {
        return ControlMatch::None;
    };
    if !digits.iter().all(u8::is_ascii_digit) {
        return ControlMatch::None;
    }
    let Some(payload_start) = offset.checked_add(2) else {
        return ControlMatch::None;
    };
    let Some(end) = offset.checked_add(5) else {
        return ControlMatch::None;
    };
    ControlMatch::Token {
        kind: DxfTextControlTokenKind::PercentUnicodeDecimal,
        end,
        payload: Some(DxfDecodedTextSpan::from_parts(payload_start, end)),
    }
}

fn fixed(kind: DxfTextControlTokenKind, offset: usize, length: usize) -> ControlMatch {
    let Some(end) = offset.checked_add(length) else {
        return ControlMatch::None;
    };
    ControlMatch::Token {
        kind,
        end,
        payload: None,
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::{
        DxfDecodedTextSpan, DxfTextControlContext, DxfTextControlCursor, DxfTextControlError,
        DxfTextControlIssue, DxfTextControlToken, DxfTextControlTokenKind,
    };
    use crate::{DxfTextEscapeDecodeStatus, decode_dxf_text_escapes_to_utf8_without_replacement};

    #[test]
    fn tokenizes_every_documented_mtext_family_with_exact_accounting() -> Result<(), Box<dyn Error>>
    {
        let source = r"A{\Oover\o|\Lunder\l|\Kstrike\k|\~|\\|\{literal\}|\C2;color|\FArial;font|\H2;height|\H3x;relative|\S1/2;|\T2;track|\Q20;oblique|\W2;width|\A1;align|\Pparagraph}Z";
        let tokens = collect(source, DxfTextControlContext::MText)?;
        assert_exact_accounting(source, &tokens)?;
        let kinds: Vec<_> = tokens
            .iter()
            .copied()
            .map(DxfTextControlToken::kind)
            .collect();
        for expected in [
            DxfTextControlTokenKind::MTextBlockStart,
            DxfTextControlTokenKind::MTextOverlineOn,
            DxfTextControlTokenKind::MTextOverlineOff,
            DxfTextControlTokenKind::MTextUnderlineOn,
            DxfTextControlTokenKind::MTextUnderlineOff,
            DxfTextControlTokenKind::MTextStrikethroughOn,
            DxfTextControlTokenKind::MTextStrikethroughOff,
            DxfTextControlTokenKind::MTextNonbreakingSpace,
            DxfTextControlTokenKind::MTextEscapedBackslash,
            DxfTextControlTokenKind::MTextEscapedOpenBrace,
            DxfTextControlTokenKind::MTextEscapedCloseBrace,
            DxfTextControlTokenKind::MTextColor,
            DxfTextControlTokenKind::MTextFont,
            DxfTextControlTokenKind::MTextHeight,
            DxfTextControlTokenKind::MTextStack,
            DxfTextControlTokenKind::MTextTracking,
            DxfTextControlTokenKind::MTextOblique,
            DxfTextControlTokenKind::MTextWidth,
            DxfTextControlTokenKind::MTextAlignment,
            DxfTextControlTokenKind::MTextParagraphBreak,
            DxfTextControlTokenKind::MTextBlockEnd,
        ] {
            assert!(kinds.contains(&expected), "missing {expected:?}");
        }
        Ok(())
    }

    #[test]
    fn parameter_payloads_remain_exact_unparsed_spans() -> Result<(), Box<dyn Error>> {
        let source = r"\A1;\C2;\FArial;\H2;\H3x;\S1/2;\T.75;\Q-20;\W2;";
        let tokens = collect(source, DxfTextControlContext::MText)?;
        let payloads: Vec<_> = tokens
            .iter()
            .filter_map(|token| token.payload_span())
            .map(|span| span.slice(source).ok_or(io::Error::other("payload span")))
            .collect::<Result<_, _>>()?;
        assert_eq!(
            payloads,
            ["1", "2", "Arial", "2", "3x", "1/2", ".75", "-20", "2"]
        );
        assert_exact_accounting(source, &tokens)?;
        Ok(())
    }

    #[test]
    fn composes_after_cif_mif_decode_without_consuming_mtext_controls() -> Result<(), Box<dyn Error>>
    {
        let source = r"{\C2;A\U+4F60\P%%d}";
        let mut decoded = [0_u8; 64];
        let result = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut decoded);
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        let text = std::str::from_utf8(&decoded[..result.written()])?;
        let tokens = collect(text, DxfTextControlContext::MText)?;
        let kinds: Vec<_> = tokens
            .iter()
            .copied()
            .map(DxfTextControlToken::kind)
            .collect();
        assert_eq!(
            kinds,
            [
                DxfTextControlTokenKind::MTextBlockStart,
                DxfTextControlTokenKind::MTextColor,
                DxfTextControlTokenKind::Text,
                DxfTextControlTokenKind::MTextParagraphBreak,
                DxfTextControlTokenKind::PercentDegree,
                DxfTextControlTokenKind::MTextBlockEnd,
            ]
        );
        assert_exact_accounting(text, &tokens)?;
        Ok(())
    }

    #[test]
    fn mtext_percent_symbols_are_contextual_and_case_insensitive() -> Result<(), Box<dyn Error>> {
        for (source, expected) in [
            ("%%c", DxfTextControlTokenKind::PercentDiameter),
            ("%%C", DxfTextControlTokenKind::PercentDiameter),
            ("%%d", DxfTextControlTokenKind::PercentDegree),
            ("%%D", DxfTextControlTokenKind::PercentDegree),
            ("%%p", DxfTextControlTokenKind::PercentPlusMinus),
            ("%%P", DxfTextControlTokenKind::PercentPlusMinus),
        ] {
            assert_eq!(
                collect(source, DxfTextControlContext::MText)?[0].kind(),
                expected
            );
        }
        for source in ["%%u", "%%O", "%%k", "%%%", "%%065", "%%x"] {
            let tokens = collect(source, DxfTextControlContext::MText)?;
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind(), DxfTextControlTokenKind::Text);
        }
        Ok(())
    }

    #[test]
    fn single_line_percent_controls_follow_the_documented_registry() -> Result<(), Box<dyn Error>> {
        for (source, expected, payload) in [
            ("%%c", DxfTextControlTokenKind::PercentDiameter, None),
            ("%%D", DxfTextControlTokenKind::PercentDegree, None),
            ("%%P", DxfTextControlTokenKind::PercentPlusMinus, None),
            (
                "%%K",
                DxfTextControlTokenKind::PercentStrikethroughToggle,
                None,
            ),
            ("%%o", DxfTextControlTokenKind::PercentOverlineToggle, None),
            ("%%U", DxfTextControlTokenKind::PercentUnderlineToggle, None),
            ("%%%", DxfTextControlTokenKind::PercentLiteral, None),
            (
                "%%065",
                DxfTextControlTokenKind::PercentUnicodeDecimal,
                Some("065"),
            ),
        ] {
            let tokens = collect(source, DxfTextControlContext::SingleLineText)?;
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind(), expected);
            assert_eq!(
                tokens[0].payload_span().and_then(|span| span.slice(source)),
                payload
            );
        }
        for source in ["%%65", "%%x", r"\P", r"\C2;"] {
            let tokens = collect(source, DxfTextControlContext::SingleLineText)?;
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind(), DxfTextControlTokenKind::Text);
        }

        let split = collect("%%1234", DxfTextControlContext::SingleLineText)?;
        assert_eq!(split.len(), 2);
        assert_eq!(
            split[0].kind(),
            DxfTextControlTokenKind::PercentUnicodeDecimal
        );
        assert_eq!(
            split[0]
                .payload_span()
                .and_then(|span| span.slice("%%1234")),
            Some("123")
        );
        assert_eq!(split[1].source_span().slice("%%1234"), Some("4"));
        Ok(())
    }

    #[test]
    fn unknown_controls_and_utf8_are_one_exact_text_run() -> Result<(), Box<dyn Error>> {
        let source = "Việt\\Xabc;\\pbad;\\u+0041%%x";
        let tokens = collect(source, DxfTextControlContext::MText)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind(), DxfTextControlTokenKind::Text);
        assert_eq!(tokens[0].source_span().slice(source), Some(source));
        Ok(())
    }

    #[test]
    fn every_parameter_family_requires_a_semicolon_after_its_raw_payload() {
        for code in ['A', 'C', 'F', 'H', 'S', 'T', 'Q', 'W'] {
            let source = format!("x\\{code}value");
            let mut cursor = DxfTextControlCursor::new(&source, DxfTextControlContext::MText);
            let prefix = cursor.next_token();
            assert!(matches!(
                prefix,
                Ok(Some(token)) if token.kind() == DxfTextControlTokenKind::Text
            ));
            let error = cursor.next_token();
            assert_eq!(
                error,
                Err(DxfTextControlError {
                    source_offset: 1,
                    issue: DxfTextControlIssue::MTextMissingTerminator,
                })
            );
            assert_eq!(cursor.consumed_bytes(), 1);
            assert!(!cursor.is_complete());
        }
    }

    #[test]
    fn block_structure_accepts_eight_levels_and_rejects_every_bad_boundary()
    -> Result<(), Box<dyn Error>> {
        let valid = "{{{{{{{{x}}}}}}}}";
        let tokens = collect(valid, DxfTextControlContext::MText)?;
        assert_exact_accounting(valid, &tokens)?;

        let mut ninth =
            DxfTextControlCursor::new("{{{{{{{{{x}}}}}}}}}", DxfTextControlContext::MText);
        for _ in 0..8 {
            assert!(matches!(ninth.next_token(), Ok(Some(_))));
        }
        assert_eq!(
            ninth.next_token(),
            Err(DxfTextControlError {
                source_offset: 8,
                issue: DxfTextControlIssue::MTextNestingLimitExceeded,
            })
        );

        let mut unmatched = DxfTextControlCursor::new("}x", DxfTextControlContext::MText);
        assert_eq!(
            unmatched.next_token(),
            Err(DxfTextControlError {
                source_offset: 0,
                issue: DxfTextControlIssue::MTextUnmatchedBlockEnd,
            })
        );

        let mut unclosed = DxfTextControlCursor::new("{x", DxfTextControlContext::MText);
        assert!(matches!(unclosed.next_token(), Ok(Some(_))));
        assert!(matches!(unclosed.next_token(), Ok(Some(_))));
        assert_eq!(
            unclosed.next_token(),
            Err(DxfTextControlError {
                source_offset: 0,
                issue: DxfTextControlIssue::MTextUnclosedBlock,
            })
        );
        Ok(())
    }

    #[test]
    fn empty_and_plain_inputs_complete_without_phantom_tokens() -> Result<(), Box<dyn Error>> {
        let mut empty = DxfTextControlCursor::new("", DxfTextControlContext::MText);
        assert_eq!(empty.next_token()?, None);
        assert!(empty.is_complete());
        assert_eq!(empty.next_token()?, None);

        let source = "plain";
        let mut plain = DxfTextControlCursor::new(source, DxfTextControlContext::SingleLineText);
        let token = plain
            .next_token()?
            .ok_or(io::Error::other("missing plain token"))?;
        assert_eq!(token.source_span().slice(source), Some(source));
        assert_eq!(plain.next_token()?, None);
        assert!(plain.is_complete());
        Ok(())
    }

    #[test]
    fn every_ascii_introducer_path_terminates_or_advances() {
        for context in [
            DxfTextControlContext::MText,
            DxfTextControlContext::SingleLineText,
        ] {
            for byte in 0x20_u8..=0x7E {
                for prefix in ["\\", "%%"] {
                    let source = format!("a{prefix}{}z", char::from(byte));
                    let mut cursor = DxfTextControlCursor::new(&source, context);
                    let mut previous = 0_usize;
                    for _ in 0..=source.len() {
                        match cursor.next_token() {
                            Ok(Some(token)) => {
                                assert!(token.source_span().end() > previous);
                                previous = token.source_span().end();
                            }
                            Ok(None) | Err(_) => break,
                        }
                    }
                    assert!(previous <= source.len());
                }
            }
        }
    }

    #[test]
    fn public_values_are_send_sync_and_copy() {
        assert_traits::<DxfTextControlContext>();
        assert_traits::<DxfTextControlTokenKind>();
        assert_traits::<DxfDecodedTextSpan>();
        assert_traits::<DxfTextControlToken>();
        assert_traits::<DxfTextControlIssue>();
        assert_traits::<DxfTextControlError>();
        assert_send_sync::<DxfTextControlCursor<'static>>();
    }

    fn collect(
        source: &str,
        context: DxfTextControlContext,
    ) -> Result<Vec<DxfTextControlToken>, DxfTextControlError> {
        let mut cursor = DxfTextControlCursor::new(source, context);
        let mut tokens = Vec::new();
        while let Some(token) = cursor.next_token()? {
            tokens.push(token);
        }
        assert!(cursor.is_complete());
        Ok(tokens)
    }

    fn assert_exact_accounting(
        source: &str,
        tokens: &[DxfTextControlToken],
    ) -> Result<(), io::Error> {
        let mut offset = 0_usize;
        for token in tokens {
            assert_eq!(token.source_span().start(), offset);
            let raw = token
                .source_span()
                .slice(source)
                .ok_or(io::Error::other("token span"))?;
            assert!(!raw.is_empty());
            offset = token.source_span().end();
        }
        assert_eq!(offset, source.len());
        Ok(())
    }

    fn assert_traits<T: Send + Sync + Copy>() {}
    fn assert_send_sync<T: Send + Sync>() {}
}

//! Bounded interpretation of documented DXF text control sequences.

use crate::{DxfLegacyCodePage, DxfTextDecodeStatus, DxfTextDecoder};

const CIF_PREFIX: &[u8] = b"\\U+";
const MIF_UPPER_PREFIX: &[u8] = b"\\M+";
const MIF_LOWER_PREFIX: &[u8] = b"\\m+";
const CIF_TOKEN_BYTES: usize = 7;
const MIF_TOKEN_BYTES: usize = 8;

/// Windows codepage selected by the MIF digit in `\M+nxxxx`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMifCodePage {
    Windows932,
    Windows950,
    Windows949,
    Windows1361,
    Windows936,
}

impl DxfMifCodePage {
    #[must_use]
    pub const fn from_selector(selector: u8) -> Option<Self> {
        match selector {
            1 => Some(Self::Windows932),
            2 => Some(Self::Windows950),
            3 => Some(Self::Windows949),
            4 => Some(Self::Windows1361),
            5 => Some(Self::Windows936),
            _ => None,
        }
    }

    #[must_use]
    pub const fn selector(self) -> u8 {
        match self {
            Self::Windows932 => 1,
            Self::Windows950 => 2,
            Self::Windows949 => 3,
            Self::Windows1361 => 4,
            Self::Windows936 => 5,
        }
    }

    #[must_use]
    pub const fn windows_code_page(self) -> u16 {
        match self {
            Self::Windows932 => 932,
            Self::Windows950 => 950,
            Self::Windows949 => 949,
            Self::Windows1361 => 1361,
            Self::Windows936 => 936,
        }
    }

    const fn legacy_decoder_page(self) -> Option<DxfLegacyCodePage> {
        match self {
            Self::Windows932 => Some(DxfLegacyCodePage::Windows932),
            Self::Windows950 => Some(DxfLegacyCodePage::Windows950),
            Self::Windows949 => Some(DxfLegacyCodePage::Windows949),
            Self::Windows1361 => None,
            Self::Windows936 => Some(DxfLegacyCodePage::Windows936),
        }
    }
}

/// Why a recognized DXF text control sequence is malformed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextEscapeIssue {
    CifTruncated,
    CifInvalidHex,
    CifUnpairedHighSurrogate,
    CifUnpairedLowSurrogate,
    MifTruncated,
    MifInvalidSelector,
    MifInvalidHex,
    MifInvalidCode,
}

/// Terminal state of one bounded DXF text escape decode.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextEscapeDecodeStatus {
    Complete,
    OutputFull,
    Malformed {
        source_offset: usize,
        issue: DxfTextEscapeIssue,
    },
    UnsupportedMifCodePage {
        source_offset: usize,
        code_page: DxfMifCodePage,
        code: u16,
    },
}

/// Counts and terminal state for one replacement-free escape decode.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextEscapeDecodeResult {
    status: DxfTextEscapeDecodeStatus,
    read: usize,
    written: usize,
}

impl DxfTextEscapeDecodeResult {
    #[must_use]
    pub const fn status(self) -> DxfTextEscapeDecodeStatus {
        self.status
    }

    /// Returns source UTF-8 bytes consumed before the terminal condition.
    #[must_use]
    pub const fn read(self) -> usize {
        self.read
    }

    #[must_use]
    pub const fn written(self) -> usize {
        self.written
    }
}

/// Interprets documented DXF CIF controls in already-decoded UTF-8 text.
///
/// Exact `\U+hhhh` controls are converted without allocation or replacement.
/// Adjacent high/low surrogate controls form one Unicode scalar. Exact
/// Evidence-backed `\M+nxxxx` controls decode through their selected Windows
/// codepage. CP1361/Johab remains typed and fail-closed until its full mapping
/// is independently verified. Only
/// `destination[..result.written()]` is defined output. Retry the complete
/// source after `OutputFull`.
#[must_use]
pub fn decode_dxf_text_escapes_to_utf8_without_replacement(
    source: &str,
    destination: &mut [u8],
) -> DxfTextEscapeDecodeResult {
    let bytes = source.as_bytes();
    let mut read = 0_usize;
    let mut written = 0_usize;

    while read < bytes.len() {
        let remaining = match bytes.get(read..) {
            Some(value) => value,
            None => return malformed(read, written, read, DxfTextEscapeIssue::CifTruncated),
        };
        if remaining.starts_with(CIF_PREFIX) {
            let (scalar, consumed) = match parse_cif_scalar(bytes, read) {
                Ok(value) => value,
                Err((source_offset, issue)) => {
                    return malformed(read, written, source_offset, issue);
                }
            };
            let mut encoded_storage = [0_u8; 4];
            let encoded = scalar.encode_utf8(&mut encoded_storage).as_bytes();
            if !copy_complete(encoded, destination, &mut written) {
                return result(DxfTextEscapeDecodeStatus::OutputFull, read, written);
            }
            read += consumed;
        } else if starts_supported_mif_shape(remaining) {
            let (code_page, code) = match parse_mif(bytes, read) {
                Ok(value) => value,
                Err(issue) => return malformed(read, written, read, issue),
            };
            let Some(legacy_page) = code_page.legacy_decoder_page() else {
                return result(
                    DxfTextEscapeDecodeStatus::UnsupportedMifCodePage {
                        source_offset: read,
                        code_page,
                        code,
                    },
                    read,
                    written,
                );
            };
            let code_bytes = code.to_be_bytes();
            let encoded = if code_bytes[0] == 0 {
                &code_bytes[1..]
            } else {
                &code_bytes[..]
            };
            let mut scalar_storage = [0_u8; 4];
            let decoded = DxfTextDecoder::Legacy(legacy_page)
                .decode_complete_to_utf8_without_replacement(encoded, &mut scalar_storage);
            let decoded_output = &scalar_storage[..decoded.written()];
            if decoded.status() != DxfTextDecodeStatus::Complete
                || decoded.read() != encoded.len()
                || !contains_exactly_one_scalar(decoded_output)
            {
                return malformed(read, written, read, DxfTextEscapeIssue::MifInvalidCode);
            }
            if !copy_complete(decoded_output, destination, &mut written) {
                return result(DxfTextEscapeDecodeStatus::OutputFull, read, written);
            }
            read += MIF_TOKEN_BYTES;
        } else {
            let plain = match source.get(read..).and_then(|value| value.chars().next()) {
                Some(value) => value,
                None => return malformed(read, written, read, DxfTextEscapeIssue::CifTruncated),
            };
            let mut encoded_storage = [0_u8; 4];
            let encoded = plain.encode_utf8(&mut encoded_storage).as_bytes();
            if !copy_complete(encoded, destination, &mut written) {
                return result(DxfTextEscapeDecodeStatus::OutputFull, read, written);
            }
            read += encoded.len();
        }
    }

    result(DxfTextEscapeDecodeStatus::Complete, read, written)
}

fn parse_cif_scalar(
    source: &[u8],
    offset: usize,
) -> Result<(char, usize), (usize, DxfTextEscapeIssue)> {
    let first = parse_cif_unit(source, offset)?;
    if (0xD800..=0xDBFF).contains(&first) {
        let second_offset = offset + CIF_TOKEN_BYTES;
        let remaining = source
            .get(second_offset..)
            .ok_or((offset, DxfTextEscapeIssue::CifUnpairedHighSurrogate))?;
        if !remaining.starts_with(CIF_PREFIX) {
            return Err((offset, DxfTextEscapeIssue::CifUnpairedHighSurrogate));
        }
        let second = parse_cif_unit(source, second_offset)?;
        if !(0xDC00..=0xDFFF).contains(&second) {
            return Err((offset, DxfTextEscapeIssue::CifUnpairedHighSurrogate));
        }
        let scalar = 0x1_0000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00);
        let character =
            char::from_u32(scalar).ok_or((offset, DxfTextEscapeIssue::CifUnpairedHighSurrogate))?;
        Ok((character, CIF_TOKEN_BYTES * 2))
    } else if (0xDC00..=0xDFFF).contains(&first) {
        Err((offset, DxfTextEscapeIssue::CifUnpairedLowSurrogate))
    } else {
        let character =
            char::from_u32(u32::from(first)).ok_or((offset, DxfTextEscapeIssue::CifInvalidHex))?;
        Ok((character, CIF_TOKEN_BYTES))
    }
}

fn parse_cif_unit(source: &[u8], offset: usize) -> Result<u16, (usize, DxfTextEscapeIssue)> {
    let end = offset
        .checked_add(CIF_TOKEN_BYTES)
        .ok_or((offset, DxfTextEscapeIssue::CifTruncated))?;
    let token = source
        .get(offset..end)
        .ok_or((offset, DxfTextEscapeIssue::CifTruncated))?;
    let digits = token
        .get(CIF_PREFIX.len()..)
        .ok_or((offset, DxfTextEscapeIssue::CifTruncated))?;
    parse_hex_u16(digits).ok_or((offset, DxfTextEscapeIssue::CifInvalidHex))
}

fn starts_supported_mif_shape(source: &[u8]) -> bool {
    let recognized_prefix =
        source.starts_with(MIF_UPPER_PREFIX) || source.starts_with(MIF_LOWER_PREFIX);
    recognized_prefix && matches!(source.get(MIF_UPPER_PREFIX.len()), Some(b'1'..=b'5'))
}

fn parse_mif(source: &[u8], offset: usize) -> Result<(DxfMifCodePage, u16), DxfTextEscapeIssue> {
    let end = offset
        .checked_add(MIF_TOKEN_BYTES)
        .ok_or(DxfTextEscapeIssue::MifTruncated)?;
    let token = source
        .get(offset..end)
        .ok_or(DxfTextEscapeIssue::MifTruncated)?;
    let selector_byte = *token
        .get(MIF_UPPER_PREFIX.len())
        .ok_or(DxfTextEscapeIssue::MifTruncated)?;
    let selector = selector_byte
        .checked_sub(b'0')
        .and_then(DxfMifCodePage::from_selector)
        .ok_or(DxfTextEscapeIssue::MifInvalidSelector)?;
    let digits = token
        .get(MIF_UPPER_PREFIX.len() + 1..)
        .ok_or(DxfTextEscapeIssue::MifTruncated)?;
    let code = parse_hex_u16(digits).ok_or(DxfTextEscapeIssue::MifInvalidHex)?;
    Ok((selector, code))
}

fn parse_hex_u16(digits: &[u8]) -> Option<u16> {
    if digits.len() != 4 {
        return None;
    }
    let mut value = 0_u16;
    for digit in digits {
        value = value
            .checked_mul(16)?
            .checked_add(u16::from(hex(*digit)?))?;
    }
    Some(value)
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn contains_exactly_one_scalar(source: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(source) else {
        return false;
    };
    let mut characters = text.chars();
    characters.next().is_some() && characters.next().is_none()
}

fn copy_complete(source: &[u8], destination: &mut [u8], written: &mut usize) -> bool {
    let end = match written.checked_add(source.len()) {
        Some(value) => value,
        None => return false,
    };
    let Some(target) = destination.get_mut(*written..end) else {
        return false;
    };
    target.copy_from_slice(source);
    *written = end;
    true
}

const fn malformed(
    read: usize,
    written: usize,
    source_offset: usize,
    issue: DxfTextEscapeIssue,
) -> DxfTextEscapeDecodeResult {
    result(
        DxfTextEscapeDecodeStatus::Malformed {
            source_offset,
            issue,
        },
        read,
        written,
    )
}

const fn result(
    status: DxfTextEscapeDecodeStatus,
    read: usize,
    written: usize,
) -> DxfTextEscapeDecodeResult {
    DxfTextEscapeDecodeResult {
        status,
        read,
        written,
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::{
        DxfMifCodePage, DxfTextEscapeDecodeResult, DxfTextEscapeDecodeStatus, DxfTextEscapeIssue,
        decode_dxf_text_escapes_to_utf8_without_replacement,
    };
    use crate::{
        DxfAsciiRawDocument, DxfCancellationToken, DxfMemorySource, DxfReadOptions,
        DxfResourceProfile, DxfTextDecodeStatus, NoopDxfReadObserver,
    };

    #[test]
    fn decodes_bmp_cif_with_upper_and_lower_hex() {
        let mut destination = [0_u8; 32];
        let result = decode_dxf_text_escapes_to_utf8_without_replacement(
            r"A\U+00E9\U+4f60Z",
            &mut destination,
        );
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(result.read(), 16);
        assert_eq!(&destination[..result.written()], "Aé你Z".as_bytes());
    }

    #[test]
    fn combines_adjacent_utf16_surrogates() {
        let mut destination = [0_u8; 4];
        let result = decode_dxf_text_escapes_to_utf8_without_replacement(
            r"\U+D83D\U+DE00",
            &mut destination,
        );
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(result.read(), 14);
        assert_eq!(result.written(), 4);
        assert_eq!(&destination, "😀".as_bytes());
    }

    #[test]
    fn reports_every_cif_failure_without_replacement() {
        let cases = [
            (r"x\U+123", 1, DxfTextEscapeIssue::CifTruncated, 1),
            (r"x\U+12G4", 1, DxfTextEscapeIssue::CifInvalidHex, 1),
            (
                r"x\U+D83D",
                1,
                DxfTextEscapeIssue::CifUnpairedHighSurrogate,
                1,
            ),
            (
                r"x\U+D83D\U+0041",
                1,
                DxfTextEscapeIssue::CifUnpairedHighSurrogate,
                1,
            ),
            (r"x\U+D83D\U+12G4", 8, DxfTextEscapeIssue::CifInvalidHex, 1),
            (
                r"x\U+DE00",
                1,
                DxfTextEscapeIssue::CifUnpairedLowSurrogate,
                1,
            ),
        ];
        for (source, source_offset, issue, expected_prefix) in cases {
            let mut destination = [0xCC_u8; 32];
            let result =
                decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut destination);
            assert_eq!(
                result.status(),
                DxfTextEscapeDecodeStatus::Malformed {
                    source_offset,
                    issue,
                }
            );
            assert_eq!(result.read(), expected_prefix);
            assert_eq!(result.written(), expected_prefix);
            assert_eq!(&destination[..expected_prefix], b"x");
            assert!(
                !destination[..result.written()]
                    .windows(3)
                    .any(|bytes| bytes == b"\xEF\xBF\xBD")
            );
        }
    }

    #[test]
    fn selector_registry_is_exact_and_stable() {
        let cases = [
            (1, DxfMifCodePage::Windows932, 932),
            (2, DxfMifCodePage::Windows950, 950),
            (3, DxfMifCodePage::Windows949, 949),
            (4, DxfMifCodePage::Windows1361, 1361),
            (5, DxfMifCodePage::Windows936, 936),
        ];
        for (selector, code_page, identifier) in cases {
            assert_eq!(DxfMifCodePage::from_selector(selector), Some(code_page));
            assert_eq!(code_page.selector(), selector);
            assert_eq!(code_page.windows_code_page(), identifier);
        }
        assert_eq!(DxfMifCodePage::from_selector(0), None);
        assert_eq!(DxfMifCodePage::from_selector(6), None);
    }

    #[test]
    fn decodes_evidence_backed_mif_pages_and_single_byte_form() {
        let source = r"A\M+182A0\M+2A440\M+3B0A1\M+5C4E3\M+10041Z";
        let mut destination = [0_u8; 32];
        let result = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut destination);
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(result.read(), source.len());
        assert_eq!(&destination[..result.written()], "Aあ一가你AZ".as_bytes());
    }

    #[test]
    fn lowercase_m_prefix_matches_autocad_oracle() {
        let mut destination = [0_u8; 8];
        let result =
            decode_dxf_text_escapes_to_utf8_without_replacement(r"A\m+182A0B", &mut destination);
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(&destination[..result.written()], "AあB".as_bytes());
    }

    #[test]
    fn johab_selector_is_mapped_but_remains_fail_closed() {
        let mut destination = [0xCC_u8; 8];
        let result =
            decode_dxf_text_escapes_to_utf8_without_replacement(r"\M+48861", &mut destination);
        assert_eq!(
            result,
            DxfTextEscapeDecodeResult {
                status: DxfTextEscapeDecodeStatus::UnsupportedMifCodePage {
                    source_offset: 0,
                    code_page: DxfMifCodePage::Windows1361,
                    code: 0x8861,
                },
                read: 0,
                written: 0,
            }
        );
        assert_eq!(destination, [0xCC; 8]);
    }

    #[test]
    fn selectors_outside_one_through_five_remain_literal() {
        let source = r"\M+00041|\M+60041|\M+A0041";
        let mut destination = [0_u8; 32];
        let result = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut destination);
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(&destination[..result.written()], source.as_bytes());
    }

    #[test]
    fn malformed_supported_mif_is_typed_and_stops_at_the_token() {
        let cases = [
            (r"x\M+182A", DxfTextEscapeIssue::MifTruncated),
            (r"x\M+182G0", DxfTextEscapeIssue::MifInvalidHex),
            (r"x\M+18130", DxfTextEscapeIssue::MifInvalidCode),
            (r"x\M+14142", DxfTextEscapeIssue::MifInvalidCode),
        ];
        for (source, issue) in cases {
            let mut destination = [0_u8; 16];
            let result =
                decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut destination);
            assert_eq!(
                result.status(),
                DxfTextEscapeDecodeStatus::Malformed {
                    source_offset: 1,
                    issue,
                }
            );
            assert_eq!(result.read(), 1);
            assert_eq!(result.written(), 1);
            assert_eq!(&destination[..1], b"x");
        }
    }

    #[test]
    fn unrelated_and_case_different_sequences_remain_literal() {
        let source = r"a\u+00E9\b\\U+0041";
        let mut destination = [0_u8; 32];
        let result = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut destination);
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(&destination[..result.written()], r"a\u+00E9\b\A".as_bytes());
    }

    #[test]
    fn output_full_stops_before_a_complete_scalar_and_retries_from_start() {
        let source = r"x\U+4F60";
        let mut small = [0xCC_u8; 3];
        let blocked = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut small);
        assert_eq!(blocked.status(), DxfTextEscapeDecodeStatus::OutputFull);
        assert_eq!(blocked.read(), 1);
        assert_eq!(blocked.written(), 1);
        assert_eq!(small, [b'x', 0xCC, 0xCC]);

        let mut exact = [0_u8; 4];
        let complete = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut exact);
        assert_eq!(complete.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(complete.read(), source.len());
        assert_eq!(complete.written(), exact.len());
        assert_eq!(&exact, "x你".as_bytes());
    }

    #[test]
    fn output_full_stops_before_a_complete_mif_scalar() {
        let source = r"x\M+182A0";
        let mut small = [0xCC_u8; 3];
        let blocked = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut small);
        assert_eq!(blocked.status(), DxfTextEscapeDecodeStatus::OutputFull);
        assert_eq!(blocked.read(), 1);
        assert_eq!(blocked.written(), 1);
        assert_eq!(small, [b'x', 0xCC, 0xCC]);

        let mut exact = [0_u8; 4];
        let complete = decode_dxf_text_escapes_to_utf8_without_replacement(source, &mut exact);
        assert_eq!(complete.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(complete.read(), source.len());
        assert_eq!(&exact, "xあ".as_bytes());
    }

    #[test]
    fn plain_utf8_is_copied_only_on_scalar_boundaries() {
        let mut too_small = [0xCC_u8; 2];
        let blocked = decode_dxf_text_escapes_to_utf8_without_replacement("你", &mut too_small);
        assert_eq!(blocked.status(), DxfTextEscapeDecodeStatus::OutputFull);
        assert_eq!(blocked.read(), 0);
        assert_eq!(blocked.written(), 0);
        assert_eq!(too_small, [0xCC; 2]);

        let mut exact = [0_u8; 3];
        let complete = decode_dxf_text_escapes_to_utf8_without_replacement("你", &mut exact);
        assert_eq!(complete.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(&exact, "你".as_bytes());
    }

    #[test]
    fn composes_with_the_source_anchored_group_value_view() -> Result<(), Box<dyn Error>> {
        let bytes = br"0
SECTION
2
HEADER
9
$ACADVER
1
AC1021
0
ENDSEC
0
SECTION
2
ENTITIES
0
TEXT
1
Bien\U+0111
0
ENDSEC
0
EOF
";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let cancellation = DxfCancellationToken::default();
        let mut observer = NoopDxfReadObserver;
        let document = DxfAsciiRawDocument::open(
            &source,
            DxfReadOptions::strict(),
            &cancellation,
            &mut observer,
        )?;
        let group = document
            .groups()
            .iter()
            .copied()
            .rfind(|group| group.group_code().value() == 1)
            .ok_or(io::Error::other("missing text group"))?;

        let mut decoded_storage = [0_u8; 32];
        let receipt = document.decode_group_value_to_utf8_without_replacement(
            group.occurrence(),
            &mut decoded_storage,
        )?;
        let decoded = receipt
            .decode_result()
            .ok_or(io::Error::other("missing storage decode result"))?;
        assert_eq!(decoded.status(), DxfTextDecodeStatus::Complete);
        let text = std::str::from_utf8(&decoded_storage[..decoded.written()])?;

        let mut semantic_storage = [0_u8; 16];
        let escaped =
            decode_dxf_text_escapes_to_utf8_without_replacement(text, &mut semantic_storage);
        assert_eq!(escaped.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(&semantic_storage[..escaped.written()], "Bienđ".as_bytes());
        assert_eq!(receipt.source_id(), document.source_id());
        assert_eq!(receipt.value_span(), group.value_content_span());
        Ok(())
    }

    #[test]
    fn empty_input_completes_with_empty_output() {
        let mut destination = [];
        let result = decode_dxf_text_escapes_to_utf8_without_replacement("", &mut destination);
        assert_eq!(result.status(), DxfTextEscapeDecodeStatus::Complete);
        assert_eq!(result.read(), 0);
        assert_eq!(result.written(), 0);
    }

    #[test]
    fn public_result_types_are_send_sync_and_copy() {
        assert_traits::<DxfTextEscapeIssue>();
        assert_traits::<DxfTextEscapeDecodeStatus>();
        assert_traits::<DxfTextEscapeDecodeResult>();
        assert_traits::<DxfMifCodePage>();
    }

    fn assert_traits<T: Send + Sync + Copy>() {}
}

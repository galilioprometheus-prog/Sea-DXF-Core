//! Exact legacy codepage registry and bounded, replacement-free decoding.

use encoding_rs::{
    BIG5, DecoderResult, EUC_KR, Encoding, GBK, SHIFT_JIS, UTF_8, WINDOWS_874, WINDOWS_1250,
    WINDOWS_1251, WINDOWS_1252, WINDOWS_1253, WINDOWS_1254, WINDOWS_1255, WINDOWS_1256,
    WINDOWS_1257, WINDOWS_1258,
};

/// Legacy Windows codepages whose exact Autodesk token has a reviewed decoder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLegacyCodePage {
    Windows874,
    Windows932,
    Windows936,
    Windows949,
    Windows950,
    Windows1250,
    Windows1251,
    Windows1252,
    Windows1253,
    Windows1254,
    Windows1255,
    Windows1256,
    Windows1257,
    Windows1258,
}

impl DxfLegacyCodePage {
    /// Looks up an exact, case-sensitive `$DWGCODEPAGE` value.
    ///
    /// No whitespace, aliases, DOS/OEM pages, or fallback are accepted.
    #[must_use]
    pub fn from_dwg_codepage_token(token: &[u8]) -> Option<Self> {
        match token {
            b"ANSI_874" => Some(Self::Windows874),
            b"ANSI_932" => Some(Self::Windows932),
            b"ANSI_936" => Some(Self::Windows936),
            b"ANSI_949" => Some(Self::Windows949),
            b"ANSI_950" => Some(Self::Windows950),
            b"ANSI_1250" => Some(Self::Windows1250),
            b"ANSI_1251" => Some(Self::Windows1251),
            b"ANSI_1252" => Some(Self::Windows1252),
            b"ANSI_1253" => Some(Self::Windows1253),
            b"ANSI_1254" => Some(Self::Windows1254),
            b"ANSI_1255" => Some(Self::Windows1255),
            b"ANSI_1256" => Some(Self::Windows1256),
            b"ANSI_1257" => Some(Self::Windows1257),
            b"ANSI_1258" => Some(Self::Windows1258),
            _ => None,
        }
    }

    /// Returns the one canonical Autodesk declaration recognized by SeaCad.
    #[must_use]
    pub const fn dwg_codepage_token(self) -> &'static [u8] {
        match self {
            Self::Windows874 => b"ANSI_874",
            Self::Windows932 => b"ANSI_932",
            Self::Windows936 => b"ANSI_936",
            Self::Windows949 => b"ANSI_949",
            Self::Windows950 => b"ANSI_950",
            Self::Windows1250 => b"ANSI_1250",
            Self::Windows1251 => b"ANSI_1251",
            Self::Windows1252 => b"ANSI_1252",
            Self::Windows1253 => b"ANSI_1253",
            Self::Windows1254 => b"ANSI_1254",
            Self::Windows1255 => b"ANSI_1255",
            Self::Windows1256 => b"ANSI_1256",
            Self::Windows1257 => b"ANSI_1257",
            Self::Windows1258 => b"ANSI_1258",
        }
    }

    /// Returns the Windows numeric codepage identifier.
    #[must_use]
    pub const fn windows_code_page(self) -> u16 {
        match self {
            Self::Windows874 => 874,
            Self::Windows932 => 932,
            Self::Windows936 => 936,
            Self::Windows949 => 949,
            Self::Windows950 => 950,
            Self::Windows1250 => 1250,
            Self::Windows1251 => 1251,
            Self::Windows1252 => 1252,
            Self::Windows1253 => 1253,
            Self::Windows1254 => 1254,
            Self::Windows1255 => 1255,
            Self::Windows1256 => 1256,
            Self::Windows1257 => 1257,
            Self::Windows1258 => 1258,
        }
    }

    fn encoding(self) -> &'static Encoding {
        match self {
            Self::Windows874 => WINDOWS_874,
            Self::Windows932 => SHIFT_JIS,
            Self::Windows936 => GBK,
            Self::Windows949 => EUC_KR,
            Self::Windows950 => BIG5,
            Self::Windows1250 => WINDOWS_1250,
            Self::Windows1251 => WINDOWS_1251,
            Self::Windows1252 => WINDOWS_1252,
            Self::Windows1253 => WINDOWS_1253,
            Self::Windows1254 => WINDOWS_1254,
            Self::Windows1255 => WINDOWS_1255,
            Self::Windows1256 => WINDOWS_1256,
            Self::Windows1257 => WINDOWS_1257,
            Self::Windows1258 => WINDOWS_1258,
        }
    }
}

/// One explicit DXF byte-to-Unicode decoder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextDecoder {
    Utf8,
    Legacy(DxfLegacyCodePage),
}

impl DxfTextDecoder {
    /// Decodes one complete DXF value into caller-owned UTF-8 storage.
    ///
    /// The method never allocates and never inserts U+FFFD. A non-empty input
    /// with fewer than four destination bytes returns `OutputFull` unchanged.
    /// Only `destination[..result.written()]` is defined output. Because this
    /// one-shot API does not return decoder state, `OutputFull` must be retried
    /// from the start of `source` with a larger destination.
    #[must_use]
    pub fn decode_complete_to_utf8_without_replacement(
        self,
        source: &[u8],
        destination: &mut [u8],
    ) -> DxfTextDecodeResult {
        if source.is_empty() {
            return DxfTextDecodeResult::new(DxfTextDecodeStatus::Complete, 0, 0);
        }
        if destination.len() < 4 {
            return DxfTextDecodeResult::new(DxfTextDecodeStatus::OutputFull, 0, 0);
        }

        let encoding = match self {
            Self::Utf8 => UTF_8,
            Self::Legacy(code_page) => code_page.encoding(),
        };
        let mut decoder = encoding.new_decoder_without_bom_handling();
        let (status, read, written) =
            decoder.decode_to_utf8_without_replacement(source, destination, true);
        let status = match status {
            DecoderResult::InputEmpty => DxfTextDecodeStatus::Complete,
            DecoderResult::OutputFull => DxfTextDecodeStatus::OutputFull,
            DecoderResult::Malformed(malformed_len, bytes_after_malformed) => {
                DxfTextDecodeStatus::Malformed {
                    malformed_len,
                    bytes_after_malformed,
                }
            }
        };
        DxfTextDecodeResult::new(status, read, written)
    }
}

/// Terminal reason for one bounded decode call.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextDecodeStatus {
    Complete,
    OutputFull,
    Malformed {
        malformed_len: u8,
        bytes_after_malformed: u8,
    },
}

/// Counts and status from one replacement-free decode call.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextDecodeResult {
    status: DxfTextDecodeStatus,
    read: usize,
    written: usize,
}

impl DxfTextDecodeResult {
    const fn new(status: DxfTextDecodeStatus, read: usize, written: usize) -> Self {
        Self {
            status,
            read,
            written,
        }
    }

    #[must_use]
    pub const fn status(self) -> DxfTextDecodeStatus {
        self.status
    }

    /// Returns bytes consumed for evidence, not an `OutputFull` resume cursor.
    #[must_use]
    pub const fn read(self) -> usize {
        self.read
    }

    #[must_use]
    pub const fn written(self) -> usize {
        self.written
    }
}

#[cfg(test)]
mod tests {
    use super::{DxfLegacyCodePage, DxfTextDecodeResult, DxfTextDecodeStatus, DxfTextDecoder};

    #[test]
    fn exact_registry_round_trips_all_reviewed_windows_pages() {
        let cases = [
            (b"ANSI_874".as_slice(), DxfLegacyCodePage::Windows874, 874),
            (b"ANSI_932".as_slice(), DxfLegacyCodePage::Windows932, 932),
            (b"ANSI_936".as_slice(), DxfLegacyCodePage::Windows936, 936),
            (b"ANSI_949".as_slice(), DxfLegacyCodePage::Windows949, 949),
            (b"ANSI_950".as_slice(), DxfLegacyCodePage::Windows950, 950),
            (
                b"ANSI_1250".as_slice(),
                DxfLegacyCodePage::Windows1250,
                1250,
            ),
            (
                b"ANSI_1251".as_slice(),
                DxfLegacyCodePage::Windows1251,
                1251,
            ),
            (
                b"ANSI_1252".as_slice(),
                DxfLegacyCodePage::Windows1252,
                1252,
            ),
            (
                b"ANSI_1253".as_slice(),
                DxfLegacyCodePage::Windows1253,
                1253,
            ),
            (
                b"ANSI_1254".as_slice(),
                DxfLegacyCodePage::Windows1254,
                1254,
            ),
            (
                b"ANSI_1255".as_slice(),
                DxfLegacyCodePage::Windows1255,
                1255,
            ),
            (
                b"ANSI_1256".as_slice(),
                DxfLegacyCodePage::Windows1256,
                1256,
            ),
            (
                b"ANSI_1257".as_slice(),
                DxfLegacyCodePage::Windows1257,
                1257,
            ),
            (
                b"ANSI_1258".as_slice(),
                DxfLegacyCodePage::Windows1258,
                1258,
            ),
        ];
        for (token, expected, identifier) in cases {
            let actual = DxfLegacyCodePage::from_dwg_codepage_token(token);
            assert_eq!(actual, Some(expected));
            assert_eq!(expected.dwg_codepage_token(), token);
            assert_eq!(expected.windows_code_page(), identifier);
        }
    }

    #[test]
    fn registry_refuses_aliases_oem_pages_and_unimplemented_johab() {
        for token in [
            b"ansi_1252".as_slice(),
            b" ANSI_1252".as_slice(),
            b"ANSI_1252 ".as_slice(),
            b"windows-1252".as_slice(),
            b"DOS437".as_slice(),
            b"ANSI_1361".as_slice(),
            b"UTF-8".as_slice(),
            b"".as_slice(),
        ] {
            assert_eq!(DxfLegacyCodePage::from_dwg_codepage_token(token), None);
        }
    }

    #[test]
    fn decodes_reviewed_windows_vectors_without_replacement() {
        let cases = [
            (DxfLegacyCodePage::Windows874, b"\xA1".as_slice(), "ก"),
            (DxfLegacyCodePage::Windows932, b"\x82\xA0".as_slice(), "あ"),
            (DxfLegacyCodePage::Windows936, b"\xC4\xE3".as_slice(), "你"),
            (DxfLegacyCodePage::Windows949, b"\xB0\xA1".as_slice(), "가"),
            (DxfLegacyCodePage::Windows950, b"\xA4\x40".as_slice(), "一"),
            (DxfLegacyCodePage::Windows1250, b"\x8C".as_slice(), "Ś"),
            (DxfLegacyCodePage::Windows1251, b"\xC0".as_slice(), "А"),
            (DxfLegacyCodePage::Windows1252, b"\xE9".as_slice(), "é"),
            (DxfLegacyCodePage::Windows1253, b"\xC1".as_slice(), "Α"),
            (DxfLegacyCodePage::Windows1254, b"\xD0".as_slice(), "Ğ"),
            (DxfLegacyCodePage::Windows1255, b"\xE0".as_slice(), "א"),
            (DxfLegacyCodePage::Windows1256, b"\xC7".as_slice(), "ا"),
            (DxfLegacyCodePage::Windows1257, b"\xC0".as_slice(), "Ą"),
            (DxfLegacyCodePage::Windows1258, b"\xD0".as_slice(), "Đ"),
        ];
        for (code_page, source, expected) in cases {
            let mut destination = [0_u8; 8];
            let result = DxfTextDecoder::Legacy(code_page)
                .decode_complete_to_utf8_without_replacement(source, &mut destination);
            assert_eq!(result.status(), DxfTextDecodeStatus::Complete);
            assert_eq!(result.read(), source.len());
            assert_eq!(&destination[..result.written()], expected.as_bytes());
        }
    }

    #[test]
    fn utf8_is_validated_and_malformed_input_is_fatal() {
        let mut destination = [0_u8; 16];
        let valid = DxfTextDecoder::Utf8
            .decode_complete_to_utf8_without_replacement("Biển".as_bytes(), &mut destination);
        assert_eq!(valid.status(), DxfTextDecodeStatus::Complete);
        assert_eq!(&destination[..valid.written()], "Biển".as_bytes());

        destination.fill(0);
        let malformed = DxfTextDecoder::Utf8
            .decode_complete_to_utf8_without_replacement(b"\xC3(", &mut destination);
        assert!(matches!(
            malformed.status(),
            DxfTextDecodeStatus::Malformed { .. }
        ));
        assert!(
            !destination[..malformed.written()]
                .windows(3)
                .any(|bytes| bytes == b"\xEF\xBF\xBD")
        );

        destination.fill(0);
        let legacy_malformed = DxfTextDecoder::Legacy(DxfLegacyCodePage::Windows932)
            .decode_complete_to_utf8_without_replacement(b"\x82", &mut destination);
        assert!(matches!(
            legacy_malformed.status(),
            DxfTextDecodeStatus::Malformed { .. }
        ));
        assert!(
            !destination[..legacy_malformed.written()]
                .windows(3)
                .any(|bytes| bytes == b"\xEF\xBF\xBD")
        );
    }

    #[test]
    fn destination_capacity_is_explicit_and_never_allocates() {
        let mut tiny = [0_u8; 3];
        let blocked = DxfTextDecoder::Legacy(DxfLegacyCodePage::Windows932)
            .decode_complete_to_utf8_without_replacement(b"\x82\xA0", &mut tiny);
        assert_eq!(
            blocked,
            DxfTextDecodeResult::new(DxfTextDecodeStatus::OutputFull, 0, 0)
        );
        assert_eq!(tiny, [0; 3]);

        let mut bounded = [0_u8; 4];
        let partial = DxfTextDecoder::Legacy(DxfLegacyCodePage::Windows932)
            .decode_complete_to_utf8_without_replacement(b"\x82\xA0\x82\xA2", &mut bounded);
        assert_eq!(partial.status(), DxfTextDecodeStatus::OutputFull);
        assert!(partial.read() < 4);
        assert_eq!(&bounded[..partial.written()], "あ".as_bytes());

        let mut retry_destination = [0_u8; 8];
        let retry = DxfTextDecoder::Legacy(DxfLegacyCodePage::Windows932)
            .decode_complete_to_utf8_without_replacement(
                b"\x82\xA0\x82\xA2",
                &mut retry_destination,
            );
        assert_eq!(retry.status(), DxfTextDecodeStatus::Complete);
        assert_eq!(&retry_destination[..retry.written()], "あい".as_bytes());

        let mut empty = [];
        let complete =
            DxfTextDecoder::Utf8.decode_complete_to_utf8_without_replacement(b"", &mut empty);
        assert_eq!(complete.status(), DxfTextDecodeStatus::Complete);
    }

    #[test]
    fn public_types_are_send_sync_and_copy() {
        assert_traits::<DxfLegacyCodePage>();
        assert_traits::<DxfTextDecoder>();
        assert_traits::<DxfTextDecodeStatus>();
        assert_traits::<DxfTextDecodeResult>();
    }

    fn assert_traits<T: Send + Sync + Copy>() {}
}

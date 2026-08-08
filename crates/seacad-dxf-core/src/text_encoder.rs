//! Bounded replacement-free DXF text encoding.

use encoding_rs::{EncoderResult, UTF_8};

use crate::DxfLegacyCodePage;

/// One explicit Unicode-to-DXF byte encoder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextEncoder {
    Utf8,
    Legacy(DxfLegacyCodePage),
}

impl DxfTextEncoder {
    /// Encodes one complete UTF-8 string without replacement or allocation.
    #[must_use]
    pub fn encode_complete_from_utf8_without_replacement(
        self,
        source: &str,
        destination: &mut [u8],
    ) -> DxfTextEncodeResult {
        let Some(encoding) = self.encoding() else {
            return DxfTextEncodeResult::from_parts(DxfTextEncodeStatus::Unavailable, 0, 0);
        };
        let mut encoder = encoding.new_encoder();
        let (status, read, written) =
            encoder.encode_from_utf8_without_replacement(source, destination, true);
        let status = match status {
            EncoderResult::InputEmpty => DxfTextEncodeStatus::Complete,
            EncoderResult::OutputFull => DxfTextEncodeStatus::OutputFull,
            EncoderResult::Unmappable(character) => DxfTextEncodeStatus::Unmappable {
                utf8_offset: read,
                utf8_len: character.len_utf8() as u8,
            },
        };
        DxfTextEncodeResult::from_parts(status, read, written)
    }

    fn encoding(self) -> Option<&'static encoding_rs::Encoding> {
        match self {
            Self::Utf8 => Some(UTF_8),
            Self::Legacy(code_page) => code_page.encoding(),
        }
    }
}

/// Terminal reason for one bounded encoding call.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextEncodeStatus {
    Complete,
    OutputFull,
    Unmappable { utf8_offset: usize, utf8_len: u8 },
    Unavailable,
}

/// Counts and status from one replacement-free encoding call.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextEncodeResult {
    status: DxfTextEncodeStatus,
    read: usize,
    written: usize,
}

impl DxfTextEncodeResult {
    const fn from_parts(status: DxfTextEncodeStatus, read: usize, written: usize) -> Self {
        Self {
            status,
            read,
            written,
        }
    }

    #[must_use]
    pub const fn status(self) -> DxfTextEncodeStatus {
        self.status
    }

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
    use super::{DxfTextEncodeStatus, DxfTextEncoder};
    use crate::DxfLegacyCodePage;

    #[test]
    fn reviewed_encoders_emit_exact_bytes_without_replacement() {
        for (encoder, source, expected) in [
            (DxfTextEncoder::Utf8, "Biển", "Biển".as_bytes()),
            (
                DxfTextEncoder::Legacy(DxfLegacyCodePage::Windows1252),
                "é",
                b"\xE9".as_slice(),
            ),
            (
                DxfTextEncoder::Legacy(DxfLegacyCodePage::Windows932),
                "あ",
                b"\x82\xA0".as_slice(),
            ),
            (
                DxfTextEncoder::Legacy(DxfLegacyCodePage::Windows936),
                "你",
                b"\xC4\xE3".as_slice(),
            ),
        ] {
            let mut destination = [0_u8; 16];
            let result =
                encoder.encode_complete_from_utf8_without_replacement(source, &mut destination);
            assert_eq!(result.status(), DxfTextEncodeStatus::Complete);
            assert_eq!(result.read(), source.len());
            assert_eq!(&destination[..result.written()], expected);
        }
    }

    #[test]
    fn unmappable_output_full_and_unavailable_are_distinct() {
        let mut destination = [0_u8; 8];
        let unmappable = DxfTextEncoder::Legacy(DxfLegacyCodePage::Windows1252)
            .encode_complete_from_utf8_without_replacement("你", &mut destination);
        assert!(matches!(
            unmappable.status(),
            DxfTextEncodeStatus::Unmappable { utf8_len: 3, .. }
        ));
        assert!(!destination[..unmappable.written()].contains(&b'?'));

        let mut tiny = [0_u8; 1];
        let output_full =
            DxfTextEncoder::Utf8.encode_complete_from_utf8_without_replacement("é", &mut tiny);
        assert_eq!(output_full.status(), DxfTextEncodeStatus::OutputFull);

        let unavailable = DxfTextEncoder::Legacy(DxfLegacyCodePage::Windows1361)
            .encode_complete_from_utf8_without_replacement("가", &mut destination);
        assert_eq!(unavailable.status(), DxfTextEncodeStatus::Unavailable);
        assert_eq!(unavailable.read(), 0);
        assert_eq!(unavailable.written(), 0);
    }

    #[test]
    fn public_encoder_types_are_copy_send_and_sync() {
        assert_traits::<DxfTextEncoder>();
        assert_traits::<DxfTextEncodeStatus>();
        assert_traits::<super::DxfTextEncodeResult>();
    }

    fn assert_traits<T: Copy + Send + Sync>() {}
}

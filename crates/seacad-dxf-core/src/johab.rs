//! Fixed, replacement-free Windows CP1361/Johab decoding.

use crate::{DxfTextDecodeResult, DxfTextDecodeStatus};

const CODE_SPACE: usize = 1 << 16;
const TABLE_BYTES: usize = CODE_SPACE * 2;
const JOHAB_DECODE_LE: &[u8; TABLE_BYTES] = include_bytes!("johab_decode_le.bin");

pub(crate) fn decode_johab_code(code: u16) -> Option<char> {
    let index = usize::from(code).checked_mul(2)?;
    let low = *JOHAB_DECODE_LE.get(index)?;
    let high = *JOHAB_DECODE_LE.get(index.checked_add(1)?)?;
    let stored = u16::from_le_bytes([low, high]);
    let scalar = stored.checked_sub(1)?;
    char::from_u32(u32::from(scalar))
}

#[derive(Default)]
pub(crate) struct DxfJohabDecoderSession {
    pending_lead: Option<u8>,
}

impl DxfJohabDecoderSession {
    pub(crate) fn decode(
        &mut self,
        source: &[u8],
        destination: &mut [u8],
        last: bool,
    ) -> DxfTextDecodeResult {
        let mut read = 0_usize;
        let mut written = 0_usize;

        if let Some(lead) = self.pending_lead {
            let Some(&trail) = source.first() else {
                if last {
                    self.pending_lead = None;
                    return decode_result(malformed(1), read, written);
                }
                return decode_result(DxfTextDecodeStatus::Complete, read, written);
            };
            let code = u16::from_be_bytes([lead, trail]);
            if let Some(character) = decode_johab_code(code) {
                if !write_scalar(character, destination, written) {
                    return decode_result(DxfTextDecodeStatus::OutputFull, read, written);
                }
                written += character.len_utf8();
                read += 1;
                self.pending_lead = None;
            } else {
                self.pending_lead = None;
                return if decode_johab_code(u16::from(trail)).is_some() {
                    decode_result(malformed(1), read, written)
                } else {
                    read += 1;
                    decode_result(malformed(2), read, written)
                };
            }
        }

        while let Some(&byte) = source.get(read) {
            if let Some(character) = decode_johab_code(u16::from(byte)) {
                if !write_scalar(character, destination, written) {
                    return decode_result(DxfTextDecodeStatus::OutputFull, read, written);
                }
                written += character.len_utf8();
                read += 1;
                continue;
            }
            if !is_johab_lead(byte) {
                read += 1;
                return decode_result(malformed(1), read, written);
            }

            let Some(&trail) = source.get(read + 1) else {
                read += 1;
                if last {
                    return decode_result(malformed(1), read, written);
                }
                self.pending_lead = Some(byte);
                return decode_result(DxfTextDecodeStatus::Complete, read, written);
            };
            let code = u16::from_be_bytes([byte, trail]);
            let Some(character) = decode_johab_code(code) else {
                if decode_johab_code(u16::from(trail)).is_some() {
                    read += 1;
                    return decode_result(malformed(1), read, written);
                }
                read += 2;
                return decode_result(malformed(2), read, written);
            };
            if !write_scalar(character, destination, written) {
                return decode_result(DxfTextDecodeStatus::OutputFull, read, written);
            }
            written += character.len_utf8();
            read += 2;
        }

        decode_result(DxfTextDecodeStatus::Complete, read, written)
    }
}

fn is_johab_lead(byte: u8) -> bool {
    matches!(byte, 0x84..=0xD3 | 0xD8..=0xDE | 0xE0..=0xF9)
}

fn write_scalar(character: char, destination: &mut [u8], written: usize) -> bool {
    let mut storage = [0_u8; 4];
    let encoded = character.encode_utf8(&mut storage).as_bytes();
    let Some(end) = written.checked_add(encoded.len()) else {
        return false;
    };
    let Some(target) = destination.get_mut(written..end) else {
        return false;
    };
    target.copy_from_slice(encoded);
    true
}

const fn malformed(length: u8) -> DxfTextDecodeStatus {
    DxfTextDecodeStatus::Malformed {
        malformed_len: length,
        bytes_after_malformed: 0,
    }
}

const fn decode_result(
    status: DxfTextDecodeStatus,
    read: usize,
    written: usize,
) -> DxfTextDecodeResult {
    DxfTextDecodeResult::from_parts(status, read, written)
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::{DxfJohabDecoderSession, JOHAB_DECODE_LE, decode_johab_code, malformed};
    use crate::DxfTextDecodeStatus;

    #[test]
    fn frozen_table_has_exhaustive_receipts_and_valid_scalars() {
        assert_eq!(
            Sha256::digest(JOHAB_DECODE_LE).as_slice(),
            [
                0xd0, 0x4a, 0x1a, 0x13, 0xd5, 0xf4, 0x70, 0x6d, 0xf6, 0xfa, 0x46, 0x39, 0x4c, 0xda,
                0x98, 0xa8, 0x17, 0x57, 0x0e, 0x77, 0x4d, 0x60, 0x1a, 0xcd, 0x04, 0x2e, 0x0f, 0xa5,
                0x72, 0x49, 0xf7, 0xea,
            ]
        );

        let mut canonical = Sha256::new();
        let mut count = 0_usize;
        let mut singles = 0_usize;
        for code in 0_u16..=u16::MAX {
            if let Some(character) = decode_johab_code(code) {
                count += 1;
                if code <= 0x00FF {
                    singles += 1;
                }
                canonical.update(code.to_le_bytes());
                canonical.update((character as u16).to_le_bytes());
            }
        }
        assert_eq!(count, 17_384);
        assert_eq!(singles, 132);
        assert_eq!(count - singles, 17_252);
        assert_eq!(
            canonical.finalize().as_slice(),
            [
                0x5f, 0x03, 0x8a, 0xb2, 0x83, 0x2f, 0xc3, 0xb5, 0x97, 0x11, 0x5b, 0x31, 0x38, 0x48,
                0x01, 0x42, 0xd5, 0xdc, 0xcc, 0x97, 0xa3, 0x9e, 0xdc, 0xf6, 0x15, 0x67, 0xb0, 0xdf,
                0xf8, 0x38, 0x5a, 0x84,
            ]
        );
    }

    #[test]
    fn representative_families_match_windows_and_autocad() {
        let cases = [
            (0x0000, '\0'),
            (0x0041, 'A'),
            (0x8442, 'ᆨ'),
            (0x8861, '가'),
            (0xD065, '한'),
            (0xD831, '\u{E000}'),
            (0xDE32, 'ア'),
            (0xE031, '伽'),
            (0xF931, '禍'),
        ];
        for (code, expected) in cases {
            assert_eq!(decode_johab_code(code), Some(expected));
        }
        for code in [
            0x00D4, 0x00D5, 0x00D6, 0x00D7, 0x00DF, 0x00FA, 0x00FB, 0x00FC, 0x00FD, 0x00FE, 0x00FF,
            0x8441, 0xFFFF,
        ] {
            assert_eq!(decode_johab_code(code), None);
        }
    }

    #[test]
    fn session_streams_across_a_lead_boundary_without_replacement() {
        let mut session = DxfJohabDecoderSession::default();
        let mut destination = [0_u8; 3];
        let lead = session.decode(b"\x88", &mut destination, false);
        assert_eq!(lead.status(), DxfTextDecodeStatus::Complete);
        assert_eq!(lead.read(), 1);
        assert_eq!(lead.written(), 0);

        let trail = session.decode(b"\x61", &mut destination, true);
        assert_eq!(trail.status(), DxfTextDecodeStatus::Complete);
        assert_eq!(trail.read(), 1);
        assert_eq!(trail.written(), 3);
        assert_eq!(&destination, "가".as_bytes());
    }

    #[test]
    fn output_full_never_consumes_a_partial_scalar() {
        let mut session = DxfJohabDecoderSession::default();
        let mut small = [0xCC_u8; 2];
        let blocked = session.decode(b"\x88\x61", &mut small, true);
        assert_eq!(blocked.status(), DxfTextDecodeStatus::OutputFull);
        assert_eq!(blocked.read(), 0);
        assert_eq!(blocked.written(), 0);
        assert_eq!(small, [0xCC; 2]);

        let mut exact = [0_u8; 3];
        let complete = session.decode(b"\x88\x61", &mut exact, true);
        assert_eq!(complete.status(), DxfTextDecodeStatus::Complete);
        assert_eq!(complete.read(), 2);
        assert_eq!(&exact, "가".as_bytes());
    }

    #[test]
    fn malformed_input_is_bounded_and_typed() {
        let cases = [
            (b"\xD4".as_slice(), malformed(1), 1),
            (b"\x88".as_slice(), malformed(1), 1),
            (b"\x88@".as_slice(), malformed(1), 1),
            (b"\x88\xFF".as_slice(), malformed(2), 2),
        ];
        for (source, expected, expected_read) in cases {
            let mut session = DxfJohabDecoderSession::default();
            let mut destination = [0xCC_u8; 8];
            let result = session.decode(source, &mut destination, true);
            assert_eq!(result.status(), expected);
            assert_eq!(result.read(), expected_read);
            assert_eq!(result.written(), 0);
            assert_eq!(destination, [0xCC; 8]);
        }
    }
}

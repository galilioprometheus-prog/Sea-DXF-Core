use crate::{ByteSpan, DxfError, DxfGroupCode};

/// Reviewed physical group-code encodings used by Binary DXF.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBinaryGroupCodeEncoding {
    /// Release 10 through Release 12: one byte, with byte 255 escaping XDATA codes.
    OneByteWithExtendedDataEscape,
    /// Release 13 and later: signed 16-bit little-endian group codes.
    TwoByteLittleEndian,
}

impl DxfBinaryGroupCodeEncoding {
    /// Minimum bytes required before the value can begin.
    #[must_use]
    pub const fn minimum_group_code_bytes(self) -> u8 {
        match self {
            Self::OneByteWithExtendedDataEscape => 1,
            Self::TwoByteLittleEndian => 2,
        }
    }
}

/// Physical Binary DXF value representation selected only by group code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBinaryValueFamily {
    NullTerminatedString,
    F64LittleEndian,
    I16LittleEndian,
    I32LittleEndian,
    I64LittleEndian,
    BooleanByte,
    BinaryChunk,
}

impl DxfBinaryValueFamily {
    /// Returns None for reserved codes whose wire length cannot be inferred.
    #[must_use]
    pub const fn from_group_code(group_code: DxfGroupCode) -> Option<Self> {
        match group_code.value() {
            0..=9
            | 100..=102
            | 105
            | 300..=309
            | 320..=369
            | 390..=399
            | 410..=419
            | 430..=439
            | 470..=481
            | 999
            | 1000..=1003
            | 1005..=1009 => Some(Self::NullTerminatedString),
            10..=59 | 110..=149 | 210..=239 | 460..=469 | 1010..=1059 => {
                Some(Self::F64LittleEndian)
            }
            60..=79 | 170..=179 | 270..=289 | 370..=389 | 400..=409 | 1060..=1070 => {
                Some(Self::I16LittleEndian)
            }
            90..=99 | 420..=429 | 440..=459 | 1071 => Some(Self::I32LittleEndian),
            160..=169 => Some(Self::I64LittleEndian),
            290..=299 => Some(Self::BooleanByte),
            310..=319 | 1004 => Some(Self::BinaryChunk),
            _ => None,
        }
    }

    /// Fixed payload width, or None for NUL strings and length-prefixed chunks.
    #[must_use]
    pub const fn fixed_payload_bytes(self) -> Option<u8> {
        match self {
            Self::F64LittleEndian | Self::I64LittleEndian => Some(8),
            Self::I16LittleEndian => Some(2),
            Self::I32LittleEndian => Some(4),
            Self::BooleanByte => Some(1),
            Self::NullTerminatedString | Self::BinaryChunk => None,
        }
    }
}

/// Decoded group-code header; value bytes begin after `wire_bytes`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct DxfBinaryGroupCodeHeader {
    pub group_code: DxfGroupCode,
    pub wire_bytes: u8,
}

/// Decodes one complete Binary DXF group-code header without reading its value.
///
/// `source_offset` anchors typed error spans. Extra bytes are ignored and remain
/// available to the caller as value bytes.
pub fn decode_binary_group_code(
    bytes: &[u8],
    source_offset: u64,
    encoding: DxfBinaryGroupCodeEncoding,
) -> Result<DxfBinaryGroupCodeHeader, DxfError> {
    let expected = encoding.minimum_group_code_bytes();
    let first = required_byte(bytes, 0, source_offset, expected)?;
    let (raw, wire_bytes) = match encoding {
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape if first == u8::MAX => {
            let low = required_byte(bytes, 1, source_offset, 3)?;
            let high = required_byte(bytes, 2, source_offset, 3)?;
            let code = i16::from_le_bytes([low, high]);
            if !(1000..=1071).contains(&code) {
                return Err(DxfError::InvalidBinaryGroupCode {
                    span: span_from_len(source_offset, 3)?,
                });
            }
            (code, 3)
        }
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape => (i16::from(first), 1),
        DxfBinaryGroupCodeEncoding::TwoByteLittleEndian => {
            let high = required_byte(bytes, 1, source_offset, 2)?;
            (i16::from_le_bytes([first, high]), 2)
        }
    };
    let Some(group_code) = DxfGroupCode::new(raw) else {
        return Err(DxfError::InvalidBinaryGroupCode {
            span: span_from_len(source_offset, u64::from(wire_bytes))?,
        });
    };
    Ok(DxfBinaryGroupCodeHeader {
        group_code,
        wire_bytes,
    })
}

fn required_byte(
    bytes: &[u8],
    index: usize,
    source_offset: u64,
    expected_bytes: u8,
) -> Result<u8, DxfError> {
    bytes
        .get(index)
        .copied()
        .ok_or(DxfError::TruncatedBinaryGroupCode {
            span: span_from_len(source_offset, bytes.len() as u64)?,
            expected_bytes,
        })
}

fn span_from_len(start: u64, len: u64) -> Result<ByteSpan, DxfError> {
    ByteSpan::from_start_and_len(start, len).ok_or(DxfError::OffsetOverflow {
        offset: start,
        requested: len,
    })
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::{DxfBinaryGroupCodeEncoding, DxfBinaryValueFamily, decode_binary_group_code};
    use crate::{ByteSpan, DxfError, DxfErrorCode, DxfGroupCode};

    fn code(value: i16) -> Result<DxfGroupCode, io::Error> {
        DxfGroupCode::new(value).ok_or(io::Error::other("invalid test group code"))
    }

    #[test]
    fn autocad_boundary_bytes_decode_with_the_documented_encodings() -> Result<(), Box<dyn Error>> {
        let r12 = decode_binary_group_code(
            b"\x00SECTION\0",
            22,
            DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape,
        )?;
        let post_r13 = decode_binary_group_code(
            b"\x00\x00SECTION\0",
            22,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
        )?;
        assert_eq!((r12.group_code.value(), r12.wire_bytes), (0, 1));
        assert_eq!((post_r13.group_code.value(), post_r13.wire_bytes), (0, 2));
        Ok(())
    }

    #[test]
    fn pre_r13_escape_accepts_only_extended_data_codes() -> Result<(), Box<dyn Error>> {
        let header = decode_binary_group_code(
            &[0xff, 0x2f, 0x04, 0x3f],
            100,
            DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape,
        )?;
        assert_eq!(header.group_code.value(), 1071);
        assert_eq!(header.wire_bytes, 3);

        for bytes in [[0xff, 0xe7, 0x03], [0xff, 0x30, 0x04]] {
            assert!(matches!(
                decode_binary_group_code(
                    &bytes,
                    100,
                    DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape
                ),
                Err(DxfError::InvalidBinaryGroupCode { span })
                    if span == ByteSpan::new(100, 103).ok_or(io::Error::other("span"))?
            ));
        }
        Ok(())
    }

    #[test]
    fn signed_two_byte_domain_is_validated() -> Result<(), Box<dyn Error>> {
        let valid = decode_binary_group_code(
            &(-5_i16).to_le_bytes(),
            7,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
        )?;
        assert_eq!(valid.group_code.value(), -5);
        for raw in [-6_i16, 1072_i16] {
            assert!(matches!(
                decode_binary_group_code(
                    &raw.to_le_bytes(),
                    7,
                    DxfBinaryGroupCodeEncoding::TwoByteLittleEndian
                ),
                Err(DxfError::InvalidBinaryGroupCode { .. })
            ));
        }
        Ok(())
    }

    #[test]
    fn truncation_reports_exact_span_and_required_width() -> Result<(), Box<dyn Error>> {
        let cases: [(&[u8], DxfBinaryGroupCodeEncoding, u8, u64); 3] = [
            (b"", DxfBinaryGroupCodeEncoding::TwoByteLittleEndian, 2, 0),
            (b"\0", DxfBinaryGroupCodeEncoding::TwoByteLittleEndian, 2, 1),
            (
                b"\xff\xe8",
                DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape,
                3,
                2,
            ),
        ];
        for (bytes, encoding, expected_bytes, observed_bytes) in cases {
            let error = decode_binary_group_code(bytes, 50, encoding);
            assert!(matches!(
                error,
                Err(DxfError::TruncatedBinaryGroupCode { span, expected_bytes: observed })
                    if observed == expected_bytes
                        && span == ByteSpan::new(50, 50 + observed_bytes)
                            .ok_or(io::Error::other("span"))?
            ));
        }
        Ok(())
    }

    #[test]
    fn every_documented_value_family_and_special_case_is_frozen() -> Result<(), Box<dyn Error>> {
        let cases = [
            (0, DxfBinaryValueFamily::NullTerminatedString, None),
            (10, DxfBinaryValueFamily::F64LittleEndian, Some(8)),
            (60, DxfBinaryValueFamily::I16LittleEndian, Some(2)),
            (90, DxfBinaryValueFamily::I32LittleEndian, Some(4)),
            (160, DxfBinaryValueFamily::I64LittleEndian, Some(8)),
            (280, DxfBinaryValueFamily::I16LittleEndian, Some(2)),
            (290, DxfBinaryValueFamily::BooleanByte, Some(1)),
            (310, DxfBinaryValueFamily::BinaryChunk, None),
            (450, DxfBinaryValueFamily::I32LittleEndian, Some(4)),
            (999, DxfBinaryValueFamily::NullTerminatedString, None),
            (1004, DxfBinaryValueFamily::BinaryChunk, None),
            (1005, DxfBinaryValueFamily::NullTerminatedString, None),
            (1071, DxfBinaryValueFamily::I32LittleEndian, Some(4)),
        ];
        for (raw, family, width) in cases {
            let observed = DxfBinaryValueFamily::from_group_code(code(raw)?)
                .ok_or(io::Error::other("missing value family"))?;
            assert_eq!(observed, family);
            assert_eq!(observed.fixed_payload_bytes(), width);
        }
        Ok(())
    }

    #[test]
    fn every_range_boundary_and_reserved_gap_is_deterministic() -> Result<(), Box<dyn Error>> {
        let supported_boundaries = [
            0, 9, 10, 59, 60, 79, 90, 99, 100, 102, 105, 110, 149, 160, 169, 170, 179, 210, 239,
            270, 299, 300, 319, 320, 369, 370, 409, 410, 481, 999, 1000, 1071,
        ];
        for raw in supported_boundaries {
            assert!(DxfBinaryValueFamily::from_group_code(code(raw)?).is_some());
        }
        for raw in [-5, 80, 89, 103, 104, 106, 109, 180, 209, 240, 269, 482, 998] {
            assert_eq!(DxfBinaryValueFamily::from_group_code(code(raw)?), None);
        }
        Ok(())
    }

    #[test]
    fn encoding_widths_and_error_codes_are_stable() {
        assert_eq!(
            DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape.minimum_group_code_bytes(),
            1
        );
        assert_eq!(
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian.minimum_group_code_bytes(),
            2
        );
        assert_eq!(
            DxfErrorCode::TRUNCATED_BINARY_GROUP_CODE.as_str(),
            "DXF-E0211"
        );
        assert_eq!(
            DxfErrorCode::INVALID_BINARY_GROUP_CODE.as_str(),
            "DXF-E0212"
        );
    }
}

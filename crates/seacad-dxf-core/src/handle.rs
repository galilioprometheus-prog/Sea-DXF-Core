use crate::ascii_group::DxfGroupCode;

/// Exact 64-bit DXF handle value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DxfHandle(u64);

impl DxfHandle {
    pub const MAX_HEX_DIGITS: usize = 16;

    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// Exact syntax failure for one DXF hexadecimal handle string.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleParseIssue {
    Empty,
    TooLong,
    InvalidDigit { offset: u8 },
}

/// Context-neutral role assigned by the numeric DXF group-code registry.
///
/// This classification does not prove that a referenced object exists, assign
/// ownership, or resolve any handle into document topology.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleGroupClass {
    ObjectIdentity,
    Arbitrary,
    SoftPointer,
    HardPointer,
    SoftOwner,
    HardOwner,
}

/// Classifies one numeric DXF group code whose value is a handle string.
///
/// Plot-style codes `390..=399` and codes `480..=481` are hard pointers.
/// Extended-data code `1005` has soft-pointer behavior. The caller retains the
/// original group code when those contexts need to remain distinguishable.
#[must_use]
pub const fn classify_dxf_handle_group_code(
    group_code: DxfGroupCode,
) -> Option<DxfHandleGroupClass> {
    match group_code.value() {
        5 | 105 => Some(DxfHandleGroupClass::ObjectIdentity),
        320..=329 => Some(DxfHandleGroupClass::Arbitrary),
        330..=339 | 1005 => Some(DxfHandleGroupClass::SoftPointer),
        340..=349 | 390..=399 | 480..=481 => Some(DxfHandleGroupClass::HardPointer),
        350..=359 => Some(DxfHandleGroupClass::SoftOwner),
        360..=369 => Some(DxfHandleGroupClass::HardOwner),
        _ => None,
    }
}

/// Parses one unprefixed 1-16 digit ASCII hexadecimal DXF handle.
///
/// Raw spelling remains authoritative and is not normalized by this function.
pub fn parse_dxf_handle_hex(raw: &[u8]) -> Result<DxfHandle, DxfHandleParseIssue> {
    if raw.is_empty() {
        return Err(DxfHandleParseIssue::Empty);
    }
    if raw.len() > DxfHandle::MAX_HEX_DIGITS {
        return Err(DxfHandleParseIssue::TooLong);
    }

    let mut value = 0_u64;
    for (index, byte) in raw.iter().copied().enumerate() {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'A'..=b'F' => byte - b'A' + 10,
            b'a'..=b'f' => byte - b'a' + 10,
            _ => {
                let offset = u8::try_from(index).map_err(|_| DxfHandleParseIssue::TooLong)?;
                return Err(DxfHandleParseIssue::InvalidDigit { offset });
            }
        };
        value = value
            .checked_mul(16)
            .and_then(|current| current.checked_add(u64::from(digit)))
            .ok_or(DxfHandleParseIssue::TooLong)?;
    }
    Ok(DxfHandle::from_u64(value))
}

#[cfg(test)]
mod tests {
    use crate::DxfGroupCode;

    use super::{
        DxfHandle, DxfHandleGroupClass, DxfHandleParseIssue, classify_dxf_handle_group_code,
        parse_dxf_handle_hex,
    };

    #[test]
    fn classifies_every_documented_handle_group_code_range() {
        let cases = [
            (5, DxfHandleGroupClass::ObjectIdentity),
            (105, DxfHandleGroupClass::ObjectIdentity),
            (320, DxfHandleGroupClass::Arbitrary),
            (329, DxfHandleGroupClass::Arbitrary),
            (330, DxfHandleGroupClass::SoftPointer),
            (339, DxfHandleGroupClass::SoftPointer),
            (340, DxfHandleGroupClass::HardPointer),
            (349, DxfHandleGroupClass::HardPointer),
            (350, DxfHandleGroupClass::SoftOwner),
            (359, DxfHandleGroupClass::SoftOwner),
            (360, DxfHandleGroupClass::HardOwner),
            (369, DxfHandleGroupClass::HardOwner),
            (390, DxfHandleGroupClass::HardPointer),
            (399, DxfHandleGroupClass::HardPointer),
            (480, DxfHandleGroupClass::HardPointer),
            (481, DxfHandleGroupClass::HardPointer),
            (1005, DxfHandleGroupClass::SoftPointer),
        ];

        for (raw_code, expected) in cases {
            let code = DxfGroupCode::new(raw_code);
            assert_eq!(
                code.map(classify_dxf_handle_group_code),
                Some(Some(expected))
            );
        }
    }

    #[test]
    fn rejects_adjacent_and_unrelated_group_codes() {
        for raw_code in [
            -5, 4, 6, 104, 106, 319, 370, 389, 400, 479, 482, 1004, 1006, 1071,
        ] {
            let code = DxfGroupCode::new(raw_code);
            assert_eq!(code.and_then(classify_dxf_handle_group_code), None);
        }
    }

    #[test]
    fn accepts_exact_hex_domain_without_normalizing_spelling() {
        let cases: [(&[u8], u64); 6] = [
            (b"0", 0),
            (b"1", 1),
            (b"00000a", 10),
            (b"ABCDEF", 0x00ab_cdef),
            (b"ffffffffffffffff", u64::MAX),
            (b"FFFFFFFFFFFFFFFF", u64::MAX),
        ];
        for (raw, expected) in cases {
            let handle = parse_dxf_handle_hex(raw);
            assert_eq!(handle, Ok(DxfHandle::from_u64(expected)));
            assert_eq!(handle.map(DxfHandle::value), Ok(expected));
        }
        assert!(parse_dxf_handle_hex(b"0").is_ok_and(DxfHandle::is_null));
    }

    #[test]
    fn rejects_every_non_exact_boundary_with_typed_offset() {
        assert_eq!(parse_dxf_handle_hex(b""), Err(DxfHandleParseIssue::Empty));
        assert_eq!(
            parse_dxf_handle_hex(b"00000000000000000"),
            Err(DxfHandleParseIssue::TooLong)
        );
        for (raw, offset) in [
            (b" 1".as_slice(), 0),
            (b"1 ".as_slice(), 1),
            (b"0x1".as_slice(), 1),
            (b"12G4".as_slice(), 2),
            (&[b'1', 0, b'2'], 1),
        ] {
            assert_eq!(
                parse_dxf_handle_hex(raw),
                Err(DxfHandleParseIssue::InvalidDigit { offset })
            );
        }
    }

    #[test]
    fn public_handle_types_are_send_sync_and_copy() {
        assert_send_sync::<DxfHandle>();
        assert_send_sync::<DxfHandleGroupClass>();
        assert_send_sync::<DxfHandleParseIssue>();
        assert_copy::<DxfHandle>();
        assert_copy::<DxfHandleGroupClass>();
        assert_copy::<DxfHandleParseIssue>();
    }

    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_copy<T: Copy>() {}
}

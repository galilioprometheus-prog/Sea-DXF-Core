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
    use super::{DxfHandle, DxfHandleParseIssue, parse_dxf_handle_hex};

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
        assert_send_sync::<DxfHandleParseIssue>();
        assert_copy::<DxfHandle>();
        assert_copy::<DxfHandleParseIssue>();
    }

    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_copy<T: Copy>() {}
}

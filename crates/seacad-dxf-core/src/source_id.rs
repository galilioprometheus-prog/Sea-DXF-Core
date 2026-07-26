use std::fmt;

/// Stable SHA-256 identity of the exact byte stream observed during a scan.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DxfSourceId([u8; Self::BYTE_LEN]);

impl DxfSourceId {
    pub const BYTE_LEN: usize = 32;
    pub const HEX_LEN: usize = Self::BYTE_LEN * 2;

    #[must_use]
    pub const fn from_sha256(bytes: [u8; Self::BYTE_LEN]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; Self::BYTE_LEN] {
        &self.0
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; Self::BYTE_LEN] {
        self.0
    }
}

impl From<[u8; DxfSourceId::BYTE_LEN]> for DxfSourceId {
    fn from(bytes: [u8; DxfSourceId::BYTE_LEN]) -> Self {
        Self::from_sha256(bytes)
    }
}

impl fmt::Display for DxfSourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for DxfSourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "DxfSourceId({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::DxfSourceId;

    #[test]
    fn source_id_has_stable_lowercase_hex_format() {
        let mut bytes = [0_u8; DxfSourceId::BYTE_LEN];
        bytes[0] = 0xab;
        bytes[DxfSourceId::BYTE_LEN - 1] = 0xcd;
        let source_id = DxfSourceId::from(bytes);
        let expected = format!("ab{}cd", "00".repeat(DxfSourceId::BYTE_LEN - 2));

        assert_eq!(source_id.to_string(), expected);
        assert_eq!(format!("{source_id:?}"), format!("DxfSourceId({expected})"));
        assert_eq!(source_id.as_bytes(), &bytes);
        assert_eq!(source_id.into_bytes(), bytes);
        assert_eq!(source_id.to_string().len(), DxfSourceId::HEX_LEN);
    }
}

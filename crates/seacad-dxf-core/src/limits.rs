const MIB: u64 = 1024 * 1024;

/// A reviewed collection of hard limits for untrusted DXF input.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfResourceProfile {
    /// Default limits suitable for interactive and server use.
    #[default]
    Safe,
    /// Explicit opt-in limits for unusually large drawings.
    Large,
}

/// Immutable limits selected by a resource profile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfResourceLimits {
    max_source_bytes: u64,
    max_records: u64,
    max_value_bytes: u64,
    max_diagnostics: u64,
}

impl DxfResourceProfile {
    /// Returns the exact limits bound to this profile.
    #[must_use]
    pub const fn limits(self) -> DxfResourceLimits {
        match self {
            Self::Safe => DxfResourceLimits {
                max_source_bytes: 512 * MIB,
                max_records: 5_000_000,
                max_value_bytes: MIB,
                max_diagnostics: 10_000,
            },
            Self::Large => DxfResourceLimits {
                max_source_bytes: 2 * 1024 * MIB,
                max_records: 25_000_000,
                max_value_bytes: 16 * MIB,
                max_diagnostics: 100_000,
            },
        }
    }
}

impl DxfResourceLimits {
    #[must_use]
    pub const fn max_source_bytes(self) -> u64 {
        self.max_source_bytes
    }

    #[must_use]
    pub const fn max_records(self) -> u64 {
        self.max_records
    }

    #[must_use]
    pub const fn max_value_bytes(self) -> u64 {
        self.max_value_bytes
    }

    #[must_use]
    pub const fn max_diagnostics(self) -> u64 {
        self.max_diagnostics
    }

    #[cfg(test)]
    pub(crate) const fn test_with_max_source_bytes(max_source_bytes: u64) -> Self {
        Self {
            max_source_bytes,
            max_records: u64::MAX,
            max_value_bytes: u64::MAX,
            max_diagnostics: u64::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DxfResourceProfile, MIB};

    #[test]
    fn safe_profile_matches_the_public_contract() {
        let limits = DxfResourceProfile::Safe.limits();
        assert_eq!(limits.max_source_bytes(), 512 * MIB);
        assert_eq!(limits.max_records(), 5_000_000);
        assert_eq!(limits.max_value_bytes(), MIB);
        assert_eq!(limits.max_diagnostics(), 10_000);
    }

    #[test]
    fn large_profile_matches_the_public_contract() {
        let limits = DxfResourceProfile::Large.limits();
        assert_eq!(limits.max_source_bytes(), 2 * 1024 * MIB);
        assert_eq!(limits.max_records(), 25_000_000);
        assert_eq!(limits.max_value_bytes(), 16 * MIB);
        assert_eq!(limits.max_diagnostics(), 100_000);
    }

    #[test]
    fn safe_is_the_default_profile() {
        assert_eq!(DxfResourceProfile::default(), DxfResourceProfile::Safe);
    }
}

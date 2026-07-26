use crate::DxfResourceProfile;

/// Controls whether framing errors may use the documented compatibility list.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfReadMode {
    /// Accept only standard-conforming framing.
    #[default]
    Strict,
    /// Permit only explicitly documented framing recoveries.
    Compatible,
}

/// Immutable options for opening a DXF source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfReadOptions {
    mode: DxfReadMode,
    resource_profile: DxfResourceProfile,
}

impl DxfReadOptions {
    #[must_use]
    pub const fn new(mode: DxfReadMode, resource_profile: DxfResourceProfile) -> Self {
        Self {
            mode,
            resource_profile,
        }
    }

    #[must_use]
    pub const fn strict() -> Self {
        Self::new(DxfReadMode::Strict, DxfResourceProfile::Safe)
    }

    #[must_use]
    pub const fn compatible() -> Self {
        Self::new(DxfReadMode::Compatible, DxfResourceProfile::Safe)
    }

    #[must_use]
    pub const fn mode(self) -> DxfReadMode {
        self.mode
    }

    #[must_use]
    pub const fn resource_profile(self) -> DxfResourceProfile {
        self.resource_profile
    }
}

impl Default for DxfReadOptions {
    fn default() -> Self {
        Self::strict()
    }
}

#[cfg(test)]
mod tests {
    use super::{DxfReadMode, DxfReadOptions};
    use crate::DxfResourceProfile;

    #[test]
    fn default_is_strict_and_safe() {
        let options = DxfReadOptions::default();
        assert_eq!(options.mode(), DxfReadMode::Strict);
        assert_eq!(options.resource_profile(), DxfResourceProfile::Safe);
    }

    #[test]
    fn explicit_constructor_combines_mode_and_profile() {
        let options = DxfReadOptions::new(DxfReadMode::Compatible, DxfResourceProfile::Large);
        assert_eq!(options.mode(), DxfReadMode::Compatible);
        assert_eq!(options.resource_profile(), DxfResourceProfile::Large);
    }
}

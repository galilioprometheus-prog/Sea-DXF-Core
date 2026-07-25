//! Lossless DXF core for SeaCad.
//!
//! Milestone M0 establishes crate boundaries only. DXF parsing starts in a
//! later, separately approved milestone.

#![forbid(unsafe_code)]

/// Returns the SeaCad DXF core package version.
#[must_use]
pub const fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn package_version_is_exposed() {
        assert_eq!(super::core_version(), env!("CARGO_PKG_VERSION"));
    }
}

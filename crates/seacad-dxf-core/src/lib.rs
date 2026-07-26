//! Lossless DXF core for SeaCad.
//!
//! Raw source bytes remain authoritative. Public contracts are designed for
//! bounded, synchronous operation on untrusted input.

#![forbid(unsafe_code)]

mod diagnostic;
mod error;
mod limits;
mod progress;
mod read_options;

pub use diagnostic::{ByteSpan, DxfDiagnostic, DxfDiagnosticCode, DxfDiagnosticSeverity};
pub use error::{DxfError, DxfErrorCode, DxfIoOperation, DxfResource};
pub use limits::{DxfResourceLimits, DxfResourceProfile};
pub use progress::{
    DxfCancellationToken, DxfReadControl, DxfReadObserver, DxfReadProgress, NoopDxfReadObserver,
};
pub use read_options::{DxfReadMode, DxfReadOptions};

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

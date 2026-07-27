//! Lossless DXF core for SeaCad.
//!
//! Raw source bytes remain authoritative. Public contracts are designed for
//! bounded, synchronous operation on untrusted input.

#![forbid(unsafe_code)]

mod ascii_document;
mod ascii_group;
mod ascii_index;
mod ascii_line;
mod diagnostic;
mod dialect;
mod error;
mod format_probe;
mod limits;
mod progress;
mod read_options;
mod source;
mod source_id;
mod source_scan;
mod verbatim;

pub use ascii_document::{DxfAsciiDocumentConformance, DxfAsciiRawDocument, DxfAsciiRawGroup};
pub use ascii_group::{DxfAsciiGroup, DxfAsciiGroupCursor, DxfGroupCode};
pub use ascii_index::{
    DxfAsciiGroupRange, DxfAsciiSection, DxfAsciiSectionClosure, DxfAsciiSectionKind,
    DxfAsciiSectionName, DxfAsciiStructureIndex,
};
pub use ascii_line::{
    DxfAsciiLineCursor, DxfAsciiLineEnding, DxfAsciiLineMetadata, DxfAsciiPhysicalLine,
};
pub use diagnostic::{ByteSpan, DxfDiagnostic, DxfDiagnosticCode, DxfDiagnosticSeverity};
pub use dialect::{
    DxfAcadVersion, DxfAcadVersionOccurrence, DxfAcadVersionReport, DxfAcadVersionState,
    DxfAcadVersionValue,
};
pub use error::{DxfError, DxfErrorCode, DxfIoOperation, DxfResource};
pub use format_probe::{DXF_BINARY_SENTINEL, DxfPhysicalFormat, probe_dxf_physical_format};
pub use limits::{DxfResourceLimits, DxfResourceProfile};
pub use progress::{
    DxfCancellationToken, DxfReadControl, DxfReadObserver, DxfReadProgress, NoopDxfReadObserver,
};
pub use read_options::{DxfReadMode, DxfReadOptions};
pub use source::{DxfByteSource, DxfFileSource, DxfMemorySource};
pub use source_id::DxfSourceId;
pub use source_scan::{DxfSourceScanReceipt, scan_dxf_source};
pub use verbatim::DxfVerbatimWriteReceipt;

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

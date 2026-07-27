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
mod encoding;
mod error;
mod format_probe;
mod limits;
mod progress;
mod read_options;
mod source;
mod source_id;
mod source_scan;
mod text_decoder;
mod text_escape;
mod text_view;
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
pub use encoding::{
    DxfCodePageOccurrence, DxfCodePageState, DxfCodePageValue, DxfTextEncodingPolicy,
    DxfTextEncodingReport, DxfTextEncodingResolution,
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
pub use text_decoder::{
    DxfLegacyCodePage, DxfTextDecodeResult, DxfTextDecodeStatus, DxfTextDecoder,
};
pub use text_escape::{
    DxfMifCodePage, DxfTextEscapeDecodeResult, DxfTextEscapeDecodeStatus, DxfTextEscapeIssue,
    decode_dxf_text_escapes_to_utf8_without_replacement,
};
pub use text_view::DxfTextValueDecodeReceipt;
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

//! Lossless DXF core for SeaCad.
//!
//! Raw source bytes remain authoritative. Public contracts are designed for
//! bounded, synchronous operation on untrusted input.

#![forbid(unsafe_code)]

mod ascii_document;
mod ascii_group;
mod ascii_index;
mod ascii_line;
mod binary_document;
mod binary_group;
mod binary_wire;
mod diagnostic;
mod dialect;
mod encoding;
mod error;
mod format_probe;
#[allow(dead_code)]
mod generated;
mod handle;
mod handseed;
mod header_index;
mod header_view;
mod johab;
mod limits;
mod progress;
mod raw_document;
mod read_options;
mod semantic_value;
mod source;
mod source_id;
mod source_scan;
mod text_control;
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
/// Binary view of the shared raw group-occurrence range.
pub type DxfBinaryGroupRange = DxfAsciiGroupRange;
/// Binary view of the shared section metadata.
pub type DxfBinarySection = DxfAsciiSection;
/// Binary view of the shared section-closure classification.
pub type DxfBinarySectionClosure = DxfAsciiSectionClosure;
/// Binary view of the shared documented section-kind registry.
pub type DxfBinarySectionKind = DxfAsciiSectionKind;
/// Binary view of the shared exact section-name classification.
pub type DxfBinarySectionName = DxfAsciiSectionName;
/// Binary view of the shared section and group-zero index.
pub type DxfBinaryStructureIndex = DxfAsciiStructureIndex;
pub use ascii_line::{
    DxfAsciiLineCursor, DxfAsciiLineEnding, DxfAsciiLineMetadata, DxfAsciiPhysicalLine,
};
pub use binary_document::{DxfBinaryDocumentConformance, DxfBinaryRawDocument, DxfBinaryRawGroup};
pub use binary_group::{DxfBinaryGroup, DxfBinaryGroupCursor};
pub use binary_wire::{
    DxfBinaryGroupCodeEncoding, DxfBinaryGroupCodeHeader, DxfBinaryValueFamily,
    decode_binary_group_code,
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
pub use handle::{DxfHandle, DxfHandleParseIssue, parse_dxf_handle_hex};
pub use handseed::{DxfHandseedOccurrence, DxfHandseedReport, DxfHandseedState, DxfHandseedValue};
pub use header_index::{DxfHeaderGroupRange, DxfHeaderVariable, DxfHeaderVariableIndex};
pub use header_view::{
    DxfAcadVersionIssue, DxfCodePageDeclaration, DxfCodePageIssue, DxfHandseedIssue, DxfHeaderView,
};
pub use limits::{DxfResourceLimits, DxfResourceProfile};
pub use progress::{
    DxfCancellationToken, DxfReadControl, DxfReadObserver, DxfReadProgress, NoopDxfReadObserver,
};
pub use raw_document::{
    DxfRawDocumentConformance, DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup,
};
pub use read_options::{DxfReadMode, DxfReadOptions};
pub use semantic_value::{
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSemanticValueState,
};
pub use source::{DxfByteSource, DxfFileSource, DxfMemorySource};
pub use source_id::DxfSourceId;
pub use source_scan::{DxfSourceScanReceipt, scan_dxf_source};
pub use text_control::{
    DxfDecodedTextSpan, DxfTextControlContext, DxfTextControlCursor, DxfTextControlError,
    DxfTextControlIssue, DxfTextControlToken, DxfTextControlTokenKind,
};
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

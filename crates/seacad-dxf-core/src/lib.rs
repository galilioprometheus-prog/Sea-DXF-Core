//! Lossless DXF core for SeaCad.
//!
//! Raw source bytes remain authoritative. Public contracts are designed for
//! bounded, synchronous operation on untrusted input.

#![forbid(unsafe_code)]

mod application_group;
mod ascii_document;
mod ascii_group;
mod ascii_index;
mod ascii_line;
mod ascii_numeric;
mod basic_geometry;
mod basic_geometry_card;
mod basic_geometry_semantic;
mod binary_document;
mod binary_group;
mod binary_wire;
mod block_definition;
mod block_name_consistency;
mod block_name_index;
mod block_record_card;
mod block_record_semantic;
mod block_record_value;
mod circular_geometry;
mod circular_geometry_card;
mod circular_geometry_semantic;
mod common_owner_candidate;
mod diagnostic;
mod dialect;
mod ellipse_geometry;
mod ellipse_geometry_card;
mod ellipse_geometry_semantic;
mod encoding;
mod error;
mod format_probe;
#[allow(dead_code)]
mod generated;
mod handle;
mod handle_context;
mod handle_identity;
mod handle_reference;
mod handle_resolution;
mod handle_role;
mod handseed;
mod header_handle;
mod header_index;
mod header_numeric;
mod header_numeric_value;
mod header_scalar;
mod header_schema_directory;
mod header_text;
mod header_view;
mod infinite_line_geometry;
mod infinite_line_geometry_card;
mod infinite_line_geometry_semantic;
mod insert_array;
mod insert_attribute_card;
mod insert_attribute_double_semantic;
mod insert_attribute_sequence;
mod insert_attribute_value;
mod insert_block_resolution;
mod insert_record_card;
mod insert_record_semantic;
mod insert_record_value;
mod insert_target_eligibility;
mod insert_transform;
mod johab;
mod lightweight_polyline;
mod lightweight_polyline_integer;
mod lightweight_polyline_record_card;
mod lightweight_polyline_record_semantic;
mod lightweight_polyline_segment;
mod lightweight_polyline_segment_geometry;
mod lightweight_polyline_vertex;
mod lightweight_polyline_vertex_semantic;
mod limits;
mod owner_evidence_comparison;
mod ownership_evidence;
mod polyline_family_semantic;
mod polyline_polyface_face;
mod polyline_polyface_geometry;
mod polyline_polyface_topology;
mod polyline_polygon_mesh;
mod polyline_polygon_mesh_geometry;
mod polyline_polygon_mesh_smoothing;
mod polyline_record_card;
mod polyline_record_semantic;
mod polyline_record_value;
mod polyline_segment;
mod polyline_segment_geometry;
mod polyline_segment_semantic;
mod polyline_segment_wcs_geometry;
mod polyline_segment_width;
mod polyline_sequence;
mod polyline_vertex_card;
mod polyline_vertex_integer_semantic;
mod polyline_vertex_semantic;
mod polyline_vertex_value;
mod progress;
mod raw_document;
mod raw_double;
mod raw_handle;
mod raw_integer;
mod raw_record;
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
pub use application_group::{
    DxfApplicationControlEntry, DxfApplicationControlKind, DxfApplicationGroupDirectory,
    DxfApplicationGroupEntry, DxfApplicationGroupKind, DxfApplicationGroupState,
};
pub use ascii_line::{
    DxfAsciiLineCursor, DxfAsciiLineEnding, DxfAsciiLineMetadata, DxfAsciiPhysicalLine,
};
pub use ascii_numeric::DxfAsciiNumericIssue;
pub use basic_geometry::{
    DxfBasicGeometryComponent, DxfBasicGeometryComponentRange, DxfBasicGeometryComponentRole,
    DxfBasicGeometryDirectory, DxfBasicGeometryKind, DxfBasicGeometryNumericIssue,
    DxfBasicGeometryRecordEntry,
};
pub use basic_geometry_card::{
    DxfBasicGeometryCardDirectory, DxfBasicGeometryCardMember, DxfBasicGeometryCardMemberRange,
    DxfBasicGeometryComponentCard, DxfBasicGeometryComponentCardState,
};
pub use basic_geometry_semantic::{
    DxfBasicGeometrySemanticDirectory, DxfBasicGeometrySemanticEntry,
    DxfBasicGeometrySemanticIssue, DxfBasicGeometrySemanticKind, DxfBasicGeometrySemanticValue,
    DxfLineGeometrySemantics, DxfPointGeometrySemantics,
};
pub use binary_document::{DxfBinaryDocumentConformance, DxfBinaryRawDocument, DxfBinaryRawGroup};
pub use binary_group::{DxfBinaryGroup, DxfBinaryGroupCursor};
pub use binary_wire::{
    DxfBinaryGroupCodeEncoding, DxfBinaryGroupCodeHeader, DxfBinaryValueFamily,
    decode_binary_group_code,
};
pub use block_definition::{
    DxfBlockDefinitionDirectory, DxfBlockDefinitionEntry, DxfBlockDefinitionState,
    DxfBlockMemberRecordRange,
};
pub use block_name_consistency::{
    DxfBlockNameConsistencyDirectory, DxfBlockNameConsistencyEntry, DxfBlockNameConsistencyState,
};
pub use block_name_index::{
    DxfBlockNameIndexDirectory, DxfBlockNameIndexLookup, DxfBlockNameIndexMatch,
};
pub use block_record_card::{
    DxfBlockRecordCardDirectory, DxfBlockRecordCardMember, DxfBlockRecordCardMemberRange,
    DxfBlockRecordValueCard, DxfBlockRecordValueCardState,
};
pub use block_record_semantic::{
    DxfBlockRecordSemanticDirectory, DxfBlockRecordSemanticDouble, DxfBlockRecordSemanticInteger,
    DxfBlockRecordSemanticIssue, DxfBlockRecordSemanticText, DxfBlockRecordSemantics,
};
pub use block_record_value::{
    DxfBlockRecordTextValue, DxfBlockRecordValue, DxfBlockRecordValueData,
    DxfBlockRecordValueDirectory, DxfBlockRecordValueEntry, DxfBlockRecordValueIssue,
    DxfBlockRecordValueRange, DxfBlockRecordValueRole,
};
pub use circular_geometry::{
    DxfCircularGeometryDirectory, DxfCircularGeometryKind, DxfCircularGeometryNumericIssue,
    DxfCircularGeometryRecordEntry, DxfCircularGeometryValue, DxfCircularGeometryValueRange,
    DxfCircularGeometryValueRole,
};
pub use circular_geometry_card::{
    DxfCircularGeometryCardDirectory, DxfCircularGeometryCardMember,
    DxfCircularGeometryCardMemberRange, DxfCircularGeometryValueCard,
    DxfCircularGeometryValueCardState,
};
pub use circular_geometry_semantic::{
    DxfCircularGeometrySemanticDirectory, DxfCircularGeometrySemanticIssue,
    DxfCircularGeometrySemanticValue, DxfCircularGeometrySemantics,
};
pub use common_owner_candidate::{
    DxfCommonOwnerCandidateDirectory, DxfCommonOwnerCandidateEntry, DxfCommonOwnerCandidateRange,
    DxfCommonOwnerCandidateState, DxfCommonOwnerRecordEntry,
};
pub use diagnostic::{ByteSpan, DxfDiagnostic, DxfDiagnosticCode, DxfDiagnosticSeverity};
pub use dialect::{
    DxfAcadVersion, DxfAcadVersionOccurrence, DxfAcadVersionReport, DxfAcadVersionState,
    DxfAcadVersionValue,
};
pub use ellipse_geometry::{
    DxfEllipseGeometryDirectory, DxfEllipseGeometryNumericIssue, DxfEllipseGeometryRecordEntry,
    DxfEllipseGeometryValue, DxfEllipseGeometryValueRange, DxfEllipseGeometryValueRole,
};
pub use ellipse_geometry_card::{
    DxfEllipseGeometryCardDirectory, DxfEllipseGeometryCardMember,
    DxfEllipseGeometryCardMemberRange, DxfEllipseGeometryValueCard,
    DxfEllipseGeometryValueCardState,
};
pub use ellipse_geometry_semantic::{
    DxfEllipseGeometrySemanticDirectory, DxfEllipseGeometrySemanticIssue,
    DxfEllipseGeometrySemanticValue, DxfEllipseGeometrySemantics,
};
pub use encoding::{
    DxfCodePageOccurrence, DxfCodePageState, DxfCodePageValue, DxfTextEncodingPolicy,
    DxfTextEncodingReport, DxfTextEncodingResolution,
};
pub use error::{DxfError, DxfErrorCode, DxfIoOperation, DxfResource};
pub use format_probe::{DXF_BINARY_SENTINEL, DxfPhysicalFormat, probe_dxf_physical_format};
pub use handle::{
    DxfHandle, DxfHandleGroupClass, DxfHandleParseIssue, classify_dxf_handle_group_code,
    parse_dxf_handle_hex,
};
pub use handle_context::{
    DxfContextualHandleReferenceDirectory, DxfContextualHandleReferenceEntry,
    DxfHandleReferenceContext,
};
pub use handle_identity::{
    DxfHandleIdentityCandidateRange, DxfHandleIdentityDirectory, DxfHandleIdentityEntry,
    DxfHandleIdentityLookup, DxfHandleIdentityMatch, DxfHandleIdentityState,
};
pub use handle_reference::{DxfHandleReferenceDirectory, DxfHandleReferenceEntry};
pub use handle_resolution::{
    DxfHandleResolutionDirectory, DxfHandleResolutionEntry, DxfHandleResolutionState,
};
pub use handle_role::{DxfHandleRoleDirectory, DxfHandleRoleEntry, DxfHandleRoleEvidence};
pub use handseed::{DxfHandseedOccurrence, DxfHandseedReport, DxfHandseedState, DxfHandseedValue};
pub use header_handle::{
    DxfHeaderHandleDirectory, DxfHeaderHandleEntry, DxfHeaderHandleIssue, DxfHeaderHandleValue,
};
pub use header_index::{DxfHeaderGroupRange, DxfHeaderVariable, DxfHeaderVariableIndex};
pub use header_numeric::{DxfHeaderNumericDirectory, DxfHeaderNumericEntry, DxfHeaderNumericView};
pub use header_numeric_value::{DxfHeaderNumericIssue, DxfHeaderNumericValue};
pub use header_scalar::{DxfDayParts, DxfDouble, DxfElapsedDays, DxfJulianDate};
pub use header_schema_directory::{DxfHeaderSchemaDirectory, DxfHeaderSchemaMatch};
pub use header_text::{
    DxfHeaderTextDirectory, DxfHeaderTextEntry, DxfHeaderTextIssue, DxfHeaderTextValue,
};
pub use header_view::{
    DxfAcadVersionIssue, DxfCodePageDeclaration, DxfCodePageIssue, DxfHandseedIssue, DxfHeaderView,
};
pub use infinite_line_geometry::{
    DxfInfiniteLineGeometryDirectory, DxfInfiniteLineGeometryKind,
    DxfInfiniteLineGeometryNumericIssue, DxfInfiniteLineGeometryRecordEntry,
    DxfInfiniteLineGeometryValue, DxfInfiniteLineGeometryValueRange,
    DxfInfiniteLineGeometryValueRole,
};
pub use infinite_line_geometry_card::{
    DxfInfiniteLineGeometryCardDirectory, DxfInfiniteLineGeometryCardMember,
    DxfInfiniteLineGeometryCardMemberRange, DxfInfiniteLineGeometryValueCard,
    DxfInfiniteLineGeometryValueCardState,
};
pub use infinite_line_geometry_semantic::{
    DxfInfiniteLineGeometrySemanticDirectory, DxfInfiniteLineGeometrySemanticIssue,
    DxfInfiniteLineGeometrySemanticValue, DxfInfiniteLineGeometrySemantics,
};
pub use insert_array::{
    DxfInsertArrayApplicationIssue, DxfInsertArrayDirectory, DxfInsertArrayEntry,
    DxfInsertArrayInstance, DxfInsertArrayIssue, DxfInsertArrayLayout,
};
pub use insert_attribute_card::{
    DxfInsertAttributeCardDirectory, DxfInsertAttributeCardMember,
    DxfInsertAttributeCardMemberRange, DxfInsertAttributeValueCard,
    DxfInsertAttributeValueCardState,
};
pub use insert_attribute_double_semantic::{
    DxfInsertAttributeDoubleSemanticDirectory, DxfInsertAttributeDoubleSemanticIssue,
    DxfInsertAttributeDoubleSemantics, DxfInsertAttributeSemanticDouble,
};
pub use insert_attribute_sequence::{
    DxfInsertAttributeRecordRange, DxfInsertAttributeSequenceDirectory,
    DxfInsertAttributeSequenceEntry, DxfInsertAttributeSequenceState,
};
pub use insert_attribute_value::{
    DxfInsertAttributeTextValue, DxfInsertAttributeValue, DxfInsertAttributeValueData,
    DxfInsertAttributeValueDirectory, DxfInsertAttributeValueEntry, DxfInsertAttributeValueIssue,
    DxfInsertAttributeValueRange, DxfInsertAttributeValueRole,
};
pub use insert_block_resolution::{
    DxfInsertBlockResolutionDirectory, DxfInsertBlockResolutionEntry,
    DxfInsertBlockResolutionState, DxfInsertBlockTargetRange,
};
pub use insert_record_card::{
    DxfInsertRecordCardDirectory, DxfInsertRecordCardMember, DxfInsertRecordCardMemberRange,
    DxfInsertRecordValueCard, DxfInsertRecordValueCardState,
};
pub use insert_record_semantic::{
    DxfInsertRecordSemanticDirectory, DxfInsertRecordSemanticDouble,
    DxfInsertRecordSemanticInteger, DxfInsertRecordSemanticIssue, DxfInsertRecordSemanticText,
    DxfInsertRecordSemantics,
};
pub use insert_record_value::{
    DxfInsertRecordTextValue, DxfInsertRecordValue, DxfInsertRecordValueData,
    DxfInsertRecordValueDirectory, DxfInsertRecordValueEntry, DxfInsertRecordValueIssue,
    DxfInsertRecordValueRange, DxfInsertRecordValueRole,
};
pub use insert_target_eligibility::{
    DxfBlockExpansionEdge, DxfInsertTargetEligibilityDirectory, DxfInsertTargetEligibilityEntry,
    DxfInsertTargetEligibilityState,
};
pub use insert_transform::{
    DxfInsertAffineTransform, DxfInsertTransformApplicationIssue, DxfInsertTransformDirectory,
    DxfInsertTransformEntry, DxfInsertTransformInput, DxfInsertTransformIssue,
};
pub use lightweight_polyline::{
    DxfLightweightPolylineDirectory, DxfLightweightPolylineNumericIssue,
    DxfLightweightPolylineRecordEntry, DxfLightweightPolylineValue,
    DxfLightweightPolylineValueRange, DxfLightweightPolylineValueRole,
};
pub use lightweight_polyline_integer::{
    DxfLightweightPolylineInteger, DxfLightweightPolylineIntegerDirectory,
    DxfLightweightPolylineIntegerIssue, DxfLightweightPolylineIntegerRange,
    DxfLightweightPolylineIntegerRecordEntry, DxfLightweightPolylineIntegerRole,
    DxfLightweightPolylineIntegerValue,
};
pub use lightweight_polyline_record_card::{
    DxfLightweightPolylineRecordCard, DxfLightweightPolylineRecordCardDirectory,
    DxfLightweightPolylineRecordCardEntry, DxfLightweightPolylineRecordCardMember,
    DxfLightweightPolylineRecordCardMemberRange, DxfLightweightPolylineRecordCardState,
    DxfLightweightPolylineRecordRole,
};
pub use lightweight_polyline_record_semantic::{
    DxfLightweightPolylineRecordSemanticDirectory, DxfLightweightPolylineRecordSemanticDouble,
    DxfLightweightPolylineRecordSemanticInteger, DxfLightweightPolylineRecordSemanticIssue,
    DxfLightweightPolylineRecordSemantics, DxfLightweightPolylineVertexCountComparison,
    DxfLightweightPolylineWidthEvidenceState,
};
pub use lightweight_polyline_segment::{
    DxfLightweightPolylineClosureState, DxfLightweightPolylineSegmentDirectory,
    DxfLightweightPolylineSegmentEntry, DxfLightweightPolylineSegmentRange,
    DxfLightweightPolylineSegmentRecordEntry, DxfLightweightPolylineSegmentSemantics,
    DxfLightweightPolylineSegmentShape, DxfLightweightPolylineSegmentTopology,
};
pub use lightweight_polyline_segment_geometry::{
    DxfLightweightPolylineOcsArcSegment, DxfLightweightPolylineOcsLineSegment,
    DxfLightweightPolylineOcsSegmentGeometry, DxfLightweightPolylineSegmentGeometryDirectory,
    DxfLightweightPolylineSegmentGeometryIssue, DxfLightweightPolylineSegmentGeometrySemantics,
};
pub use lightweight_polyline_vertex::{
    DxfLightweightPolylineGroupedRecordEntry, DxfLightweightPolylineVertexCard,
    DxfLightweightPolylineVertexCardState, DxfLightweightPolylineVertexDirectory,
    DxfLightweightPolylineVertexEntry, DxfLightweightPolylineVertexMember,
    DxfLightweightPolylineVertexRole,
};
pub use lightweight_polyline_vertex_semantic::{
    DxfLightweightPolylineVertexSemanticDirectory, DxfLightweightPolylineVertexSemanticDouble,
    DxfLightweightPolylineVertexSemanticIdentifier, DxfLightweightPolylineVertexSemanticIssue,
    DxfLightweightPolylineVertexSemantics,
};
pub use limits::{DxfResourceLimits, DxfResourceProfile};
pub use owner_evidence_comparison::{
    DxfOwnerEvidenceComparisonDirectory, DxfOwnerEvidenceComparisonEntry,
    DxfOwnerEvidenceComparisonState,
};
pub use ownership_evidence::{
    DxfIncomingOwnershipState, DxfOwnershipEvidenceDirectory, DxfOwnershipEvidenceEntry,
    DxfOwnershipLinkRange, DxfOwnershipTargetEntry, DxfResolvedOwnershipLink,
};
pub use polyline_family_semantic::{
    DxfPolylineFamily, DxfPolylineFamilySemanticDirectory, DxfPolylineFamilySemantics,
    DxfPolylineFamilyState, DxfPolylineVertexFamily, DxfPolylineVertexFamilyComparison,
    DxfPolylineVertexFamilySemantics,
};
pub use polyline_polyface_face::{
    DxfPolylinePolyfaceCornerRange, DxfPolylinePolyfaceFaceResolutionDirectory,
    DxfPolylinePolyfaceFaceResolutionState, DxfPolylinePolyfaceResolvedCornerEntry,
    DxfPolylinePolyfaceResolvedFaceEntry,
};
pub use polyline_polyface_geometry::{
    DxfPolylinePolyfaceFaceGeometryDirectory, DxfPolylinePolyfaceFaceGeometryEntry,
    DxfPolylinePolyfaceFaceGeometryState, DxfPolylinePolyfacePointEntry,
    DxfPolylinePolyfacePointRange,
};
pub use polyline_polyface_topology::{
    DxfPolylinePolyfaceCoordinateEntry, DxfPolylinePolyfaceFaceEntry,
    DxfPolylinePolyfaceMemberRange, DxfPolylinePolyfaceOrdering, DxfPolylinePolyfaceRecordEntry,
    DxfPolylinePolyfaceRecordState, DxfPolylinePolyfaceTopologyDirectory,
};
pub use polyline_polygon_mesh::{
    DxfPolylinePolygonMeshCellEntry, DxfPolylinePolygonMeshCellRange,
    DxfPolylinePolygonMeshDirectory, DxfPolylinePolygonMeshRecordEntry,
    DxfPolylinePolygonMeshRecordState,
};
pub use polyline_polygon_mesh_geometry::{
    DxfPolylinePolygonMeshCellCorner, DxfPolylinePolygonMeshCellGeometryDirectory,
    DxfPolylinePolygonMeshCellGeometryEntry, DxfPolylinePolygonMeshCellGeometryState,
};
pub use polyline_polygon_mesh_smoothing::{
    DxfPolylinePolygonMeshSmoothSurfaceType, DxfPolylinePolygonMeshSmoothingDirectory,
    DxfPolylinePolygonMeshSmoothingEntry, DxfPolylinePolygonMeshSmoothingState,
};
pub use polyline_record_card::{
    DxfPolylineRecordCardDirectory, DxfPolylineRecordCardMember, DxfPolylineRecordCardMemberRange,
    DxfPolylineRecordValueCard, DxfPolylineRecordValueCardState,
};
pub use polyline_record_semantic::{
    DxfPolylineRecordSemanticDirectory, DxfPolylineRecordSemanticDouble,
    DxfPolylineRecordSemanticInteger, DxfPolylineRecordSemanticIssue, DxfPolylineRecordSemantics,
};
pub use polyline_record_value::{
    DxfPolylineRecordNumber, DxfPolylineRecordNumericIssue, DxfPolylineRecordValue,
    DxfPolylineRecordValueDirectory, DxfPolylineRecordValueEntry, DxfPolylineRecordValueRange,
    DxfPolylineRecordValueRole,
};
pub use polyline_segment::{
    DxfPolylineSegmentDirectory, DxfPolylineSegmentEntry, DxfPolylineSegmentRange,
    DxfPolylineSegmentRecordEntry, DxfPolylineSegmentRecordState, DxfPolylineSegmentTopology,
};
pub use polyline_segment_geometry::{
    DxfPolylineOcsArcSegment, DxfPolylineOcsLineSegment, DxfPolylineSegmentGeometry,
    DxfPolylineSegmentGeometryDirectory, DxfPolylineSegmentGeometryIssue,
    DxfPolylineSegmentGeometrySemantics, DxfPolylineWcsLineSegment,
};
pub use polyline_segment_semantic::{
    DxfPolylineSegmentCoordinateSystem, DxfPolylineSegmentSemanticDirectory,
    DxfPolylineSegmentSemantics,
};
pub use polyline_segment_wcs_geometry::{
    DxfPolylineTransformedWcsArcSegment, DxfPolylineTransformedWcsLineSegment,
    DxfPolylineWcsSegmentGeometry, DxfPolylineWcsSegmentGeometryDirectory,
    DxfPolylineWcsSegmentGeometryIssue, DxfPolylineWcsSegmentGeometrySemantics,
};
pub use polyline_segment_width::{
    DxfPolylineEffectiveWidth, DxfPolylineEffectiveWidthOrigin, DxfPolylineSegmentEffectiveWidths,
    DxfPolylineSegmentWidthDirectory, DxfPolylineSegmentWidthIssue,
};
pub use polyline_sequence::{
    DxfPolylineSequenceDirectory, DxfPolylineSequenceEntry, DxfPolylineSequenceState,
    DxfPolylineVertexRecordRange,
};
pub use polyline_vertex_card::{
    DxfPolylineVertexCardDirectory, DxfPolylineVertexCardMember, DxfPolylineVertexCardMemberRange,
    DxfPolylineVertexValueCard, DxfPolylineVertexValueCardState,
};
pub use polyline_vertex_integer_semantic::{
    DxfPolylineVertexIntegerSemanticDirectory, DxfPolylineVertexIntegerSemanticIssue,
    DxfPolylineVertexIntegerSemantics, DxfPolylineVertexSemanticI16, DxfPolylineVertexSemanticI32,
};
pub use polyline_vertex_semantic::{
    DxfPolylineVertexSemanticDirectory, DxfPolylineVertexSemanticDouble,
    DxfPolylineVertexSemanticIssue, DxfPolylineVertexSemantics,
};
pub use polyline_vertex_value::{
    DxfPolylineVertexNumber, DxfPolylineVertexNumericIssue, DxfPolylineVertexValue,
    DxfPolylineVertexValueDirectory, DxfPolylineVertexValueEntry, DxfPolylineVertexValueRange,
    DxfPolylineVertexValueRole,
};
pub use progress::{
    DxfCancellationToken, DxfReadControl, DxfReadObserver, DxfReadProgress, NoopDxfReadObserver,
};
pub use raw_document::{
    DxfHeaderVariableLookup, DxfHeaderVariableLookupState, DxfRawDocumentConformance,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup,
};
pub use raw_handle::{DxfRawHandleLookup, DxfRawHandleValue};
pub use raw_record::{
    DxfRawRecord, DxfRawRecordDirectory, DxfRawRecordRange, DxfRawRecordSection,
    DxfRawRecordSectionKind, DxfRawRecordSectionState,
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

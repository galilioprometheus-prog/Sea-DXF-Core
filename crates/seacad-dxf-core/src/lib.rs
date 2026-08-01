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
mod block_attribute_definition;
mod block_attribute_definition_anchor;
mod block_attribute_definition_card;
mod block_attribute_definition_double_semantic;
mod block_attribute_definition_integer_semantic;
mod block_attribute_definition_justification;
mod block_attribute_definition_tag_index;
mod block_attribute_definition_text_semantic;
mod block_attribute_definition_value;
mod block_attribute_definition_wcs_anchor;
mod block_definition;
mod block_name_consistency;
mod block_name_index;
mod block_record_card;
mod block_record_semantic;
mod block_record_value;
mod canonical_ascii_write;
mod canonical_binary_write;
mod circular_geometry;
mod circular_geometry_card;
mod circular_geometry_semantic;
mod common_owner_candidate;
mod diagnostic;
mod dialect;
mod dimstyle_field;
mod dimstyle_field_card;
mod dimstyle_field_evidence;
mod dimstyle_field_semantic;
mod dimstyle_handle_resolution;
mod dimstyle_handle_target_validation;
mod dimstyle_table;
mod ellipse_geometry;
mod ellipse_geometry_card;
mod ellipse_geometry_semantic;
mod encoding;
mod entity_directory;
mod entity_field_evidence;
mod error;
mod format_probe;
#[allow(dead_code)]
mod generated;
mod handle;
mod handle_allocation_policy;
mod handle_assignment_plan;
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
mod helix_evidence;
mod infinite_line_geometry;
mod infinite_line_geometry_card;
mod infinite_line_geometry_semantic;
mod insert_array;
mod insert_attribute_anchor;
mod insert_attribute_card;
mod insert_attribute_definition_resolution;
mod insert_attribute_double_semantic;
mod insert_attribute_integer_semantic;
mod insert_attribute_justification;
mod insert_attribute_sequence;
mod insert_attribute_text_semantic;
mod insert_attribute_value;
mod insert_attribute_wcs_anchor;
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
mod mtext_column_relation;
mod mtext_column_semantic;
mod mtext_column_semantic_project;
mod mtext_embedded_column_evidence;
mod mtext_flat_column_evidence;
mod mtext_layout;
mod mtext_numeric_domain;
mod mtext_orientation;
mod mtext_tolerance_scalar;
mod mtext_x_axis_direction;
mod mtext_xdata_column_evidence;
mod mtext_xdata_defined_height;
mod mtext_xdata_linked_column;
mod mtext_xdata_linked_column_resolution;
mod named_symbol_table;
mod owner_evidence_comparison;
mod ownership_evidence;
mod planar_face_geometry;
mod planar_face_geometry_card;
mod planar_face_geometry_semantic;
mod planar_face_geometry_semantic_value;
mod planar_face_wcs_geometry;
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
mod shape_wcs_insertion;
mod shape_wcs_orientation;
mod source;
mod source_id;
mod source_scan;
mod source_span;
mod spline_analytic;
mod spline_analytic_value;
mod spline_auxiliary;
mod spline_auxiliary_semantic;
mod spline_card;
mod spline_count_relation;
mod spline_evidence;
mod spline_point_tuple;
mod spline_relation;
mod spline_scalar_semantic;
mod spline_topology;
mod text_control;
mod text_decoder;
mod text_escape;
mod text_layout;
mod text_placement_anchor;
mod text_shape_scalar;
mod text_symbol_card;
mod text_symbol_evidence;
mod text_symbol_ocs_projection;
mod text_symbol_role;
mod text_symbol_scalar_value;
mod text_symbol_text;
mod text_view;
mod text_wcs_anchor;
mod text_wcs_orientation;
mod tolerance_dimstyle_resolution;
mod tolerance_wcs_placement;
mod transaction_inverse;
mod transaction_plan;
mod transaction_write;
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
pub use block_attribute_definition::{
    DxfBlockAttributeDefinitionDirectory, DxfBlockAttributeDefinitionEntry,
};
pub use block_attribute_definition_anchor::{
    DxfBlockAttributeDefinitionPlacementAnchor,
    DxfBlockAttributeDefinitionPlacementAnchorDirectory,
    DxfBlockAttributeDefinitionPlacementAnchorState,
};
pub use block_attribute_definition_card::{
    DxfBlockAttributeDefinitionCardDirectory, DxfBlockAttributeDefinitionCardMember,
    DxfBlockAttributeDefinitionCardMemberRange, DxfBlockAttributeDefinitionValueCard,
    DxfBlockAttributeDefinitionValueCardState,
};
pub use block_attribute_definition_double_semantic::{
    DxfBlockAttributeDefinitionDoubleSemanticDirectory,
    DxfBlockAttributeDefinitionDoubleSemanticIssue, DxfBlockAttributeDefinitionDoubleSemantics,
    DxfBlockAttributeDefinitionSemanticDouble,
};
pub use block_attribute_definition_integer_semantic::{
    DxfBlockAttributeDefinitionIntegerSemanticDirectory,
    DxfBlockAttributeDefinitionIntegerSemanticIssue, DxfBlockAttributeDefinitionIntegerSemantics,
    DxfBlockAttributeDefinitionSemanticInteger,
};
pub use block_attribute_definition_justification::{
    DxfBlockAttributeDefinitionHorizontalJustification,
    DxfBlockAttributeDefinitionHorizontalJustificationSemantic,
    DxfBlockAttributeDefinitionJustificationDirectory,
    DxfBlockAttributeDefinitionJustificationIssue,
    DxfBlockAttributeDefinitionJustificationSemantics,
    DxfBlockAttributeDefinitionVerticalJustification,
    DxfBlockAttributeDefinitionVerticalJustificationSemantic,
};
pub use block_attribute_definition_tag_index::{
    DxfBlockAttributeDefinitionTagIndexBlock, DxfBlockAttributeDefinitionTagIndexDirectory,
    DxfBlockAttributeDefinitionTagIndexLookup, DxfBlockAttributeDefinitionTagIndexMatch,
};
pub use block_attribute_definition_text_semantic::{
    DxfBlockAttributeDefinitionSemanticText, DxfBlockAttributeDefinitionSemanticTextStyle,
    DxfBlockAttributeDefinitionTextSemanticDirectory, DxfBlockAttributeDefinitionTextSemanticIssue,
    DxfBlockAttributeDefinitionTextSemantics, DxfBlockAttributeDefinitionTextStyleName,
};
pub use block_attribute_definition_value::{
    DxfBlockAttributeDefinitionTextValue, DxfBlockAttributeDefinitionValue,
    DxfBlockAttributeDefinitionValueData, DxfBlockAttributeDefinitionValueDirectory,
    DxfBlockAttributeDefinitionValueEntry, DxfBlockAttributeDefinitionValueIssue,
    DxfBlockAttributeDefinitionValueRange, DxfBlockAttributeDefinitionValueRole,
};
pub use block_attribute_definition_wcs_anchor::{
    DxfBlockAttributeDefinitionWcsAnchor, DxfBlockAttributeDefinitionWcsAnchorDirectory,
    DxfBlockAttributeDefinitionWcsAnchorEntry, DxfBlockAttributeDefinitionWcsAnchorIssue,
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
pub use canonical_ascii_write::{DxfCanonicalAsciiEnvelopeAction, DxfCanonicalAsciiWriteReceipt};
pub use canonical_binary_write::{
    DxfCanonicalBinaryEnvelopeAction, DxfCanonicalBinaryWriteReceipt,
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
pub use dimstyle_field::{DxfDimStyleField, DxfDimStyleWireKind, dxf_dimstyle_fields};
pub use dimstyle_field_card::{
    DxfDimStyleCardMember, DxfDimStyleCardMemberRange, DxfDimStyleFieldCard,
    DxfDimStyleFieldCardDirectory, DxfDimStyleFieldCardState,
};
pub use dimstyle_field_evidence::{
    DxfDimStyleValue, DxfDimStyleValueData, DxfDimStyleValueDirectory, DxfDimStyleValueEntry,
    DxfDimStyleValueIssue, DxfDimStyleValueRange,
};
pub use dimstyle_field_semantic::{
    DxfDimStyleSemanticDirectory, DxfDimStyleSemanticIssue, DxfDimStyleSemanticValue,
    DxfDimStyleStandardFlags, DxfDimStyleStandardFlagsSemantic,
};
pub use dimstyle_handle_resolution::{
    DxfDimStyleHandleResolutionDirectory, DxfDimStyleHandleResolutionEntry, DxfDimStyleHandleRole,
    DxfDimStyleHandleTargetState,
};
pub use dimstyle_handle_target_validation::{
    DxfDimStyleHandleTargetValidationDirectory, DxfDimStyleHandleTargetValidationEntry,
    DxfDimStyleHandleTargetValidationState,
};
pub use dimstyle_table::{DxfDimStyleTableDirectory, DxfDimStyleTableEntry};
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
pub use entity_directory::{
    DxfEntityClassification, DxfEntityDirectory, DxfEntityKnownClassification, DxfEntityRef,
    DxfEntitySubclassMarker, DxfEntitySubclassRange,
};
pub use entity_field_evidence::{
    DxfEntityFieldCard, DxfEntityFieldCardMember, DxfEntityFieldCardMemberRange,
    DxfEntityFieldCardState, DxfEntityFieldEvidenceDirectory, DxfEntityFieldOccurrence,
};
pub use error::{DxfError, DxfErrorCode, DxfIoOperation, DxfResource};
pub use format_probe::{DXF_BINARY_SENTINEL, DxfPhysicalFormat, probe_dxf_physical_format};
pub use generated::entity_schema::{
    DXF_ENTITY_ALIAS_SCHEMA_SHA256, DXF_ENTITY_ALIASES, DXF_ENTITY_APPLICABILITY,
    DXF_ENTITY_APPLICABILITY_SCHEMA_SHA256, DXF_ENTITY_COMMON_FIELD_SCHEMA_SHA256,
    DXF_ENTITY_COMMON_FIELDS, DXF_ENTITY_TOPIC_SCHEMA_SHA256, DXF_ENTITY_TOPICS, DxfEntityAlias,
    DxfEntityAliasDescriptor, DxfEntityAliasEvidence, DxfEntityApplicability,
    DxfEntityApplicabilityDescriptor, DxfEntityApplicabilityEvidence, DxfEntityCoordinateSpace,
    DxfEntityField, DxfEntityFieldApplicability, DxfEntityFieldCardinality, DxfEntityFieldDefault,
    DxfEntityFieldDescriptor, DxfEntityFieldScope, DxfEntityFieldWireType,
    DxfEntityNameClassification, DxfEntityTopic, DxfEntityTopicDescriptor,
    classify_exact_dxf_entity_name, dxf_entity_aliases, dxf_entity_applicability,
    dxf_entity_common_fields, dxf_entity_topics,
};
pub use handle::{
    DxfHandle, DxfHandleGroupClass, DxfHandleParseIssue, classify_dxf_handle_group_code,
    parse_dxf_handle_hex,
};
pub use handle_allocation_policy::{
    DxfHandleAllocationOutcome, DxfHandleAllocationPolicyDirectory, DxfHandleAllocationPolicyState,
    DxfHandleAllocationProposal,
};
pub use handle_assignment_plan::{
    DxfHandleAssignmentPlan, DxfHandleAssignmentPlanOutcome, DxfHandleAssignmentTargetState,
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
pub use helix_evidence::{
    DXF_HELIX_ROLES, DxfHelixDirectory, DxfHelixNumber, DxfHelixNumericIssue, DxfHelixRecordEntry,
    DxfHelixValue, DxfHelixValueRange, DxfHelixValueRole,
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
pub use insert_attribute_anchor::{
    DxfInsertAttributePlacementAnchor, DxfInsertAttributePlacementAnchorDirectory,
    DxfInsertAttributePlacementAnchorState,
};
pub use insert_attribute_card::{
    DxfInsertAttributeCardDirectory, DxfInsertAttributeCardMember,
    DxfInsertAttributeCardMemberRange, DxfInsertAttributeValueCard,
    DxfInsertAttributeValueCardState,
};
pub use insert_attribute_definition_resolution::{
    DxfInsertAttributeDefinitionRange, DxfInsertAttributeDefinitionResolutionDirectory,
    DxfInsertAttributeDefinitionResolutionEntry, DxfInsertAttributeDefinitionResolutionState,
};
pub use insert_attribute_double_semantic::{
    DxfInsertAttributeDoubleSemanticDirectory, DxfInsertAttributeDoubleSemanticIssue,
    DxfInsertAttributeDoubleSemantics, DxfInsertAttributeSemanticDouble,
};
pub use insert_attribute_integer_semantic::{
    DxfInsertAttributeIntegerSemanticDirectory, DxfInsertAttributeIntegerSemanticIssue,
    DxfInsertAttributeIntegerSemantics, DxfInsertAttributeSemanticInteger,
};
pub use insert_attribute_justification::{
    DxfInsertAttributeHorizontalJustification, DxfInsertAttributeHorizontalJustificationSemantic,
    DxfInsertAttributeJustificationDirectory, DxfInsertAttributeJustificationIssue,
    DxfInsertAttributeJustificationSemantics, DxfInsertAttributeVerticalJustification,
    DxfInsertAttributeVerticalJustificationSemantic,
};
pub use insert_attribute_sequence::{
    DxfInsertAttributeRecordRange, DxfInsertAttributeSequenceDirectory,
    DxfInsertAttributeSequenceEntry, DxfInsertAttributeSequenceState,
};
pub use insert_attribute_text_semantic::{
    DxfInsertAttributeSemanticText, DxfInsertAttributeSemanticTextStyle,
    DxfInsertAttributeTextSemanticDirectory, DxfInsertAttributeTextSemanticIssue,
    DxfInsertAttributeTextSemantics, DxfInsertAttributeTextStyleName,
};
pub use insert_attribute_value::{
    DxfInsertAttributeTextValue, DxfInsertAttributeValue, DxfInsertAttributeValueData,
    DxfInsertAttributeValueDirectory, DxfInsertAttributeValueEntry, DxfInsertAttributeValueIssue,
    DxfInsertAttributeValueRange, DxfInsertAttributeValueRole,
};
pub use insert_attribute_wcs_anchor::{
    DxfInsertAttributeWcsAnchor, DxfInsertAttributeWcsAnchorDirectory,
    DxfInsertAttributeWcsAnchorEntry, DxfInsertAttributeWcsAnchorIssue,
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
pub use mtext_column_relation::{
    DxfMTextColumnMode, DxfMTextColumnModeSemantic, DxfMTextColumnRelationDirectory,
    DxfMTextColumnRelationIssue, DxfMTextColumnRelationSemantics,
};
pub use mtext_column_semantic::{
    DxfMTextColumnBooleanSemantic, DxfMTextColumnCountSemantic, DxfMTextColumnDoubleSemantic,
    DxfMTextColumnHeightDisposition, DxfMTextColumnIssue, DxfMTextColumnSemanticDirectory,
    DxfMTextColumnSemantics, DxfMTextColumnSourceEntry, DxfMTextColumnType,
    DxfMTextColumnTypeSemantic,
};
pub use mtext_embedded_column_evidence::{
    DxfMTextEmbeddedColumnDirectory, DxfMTextEmbeddedColumnEntry, DxfMTextEmbeddedColumnRole,
    DxfMTextEmbeddedColumnValue,
};
pub use mtext_flat_column_evidence::{
    DxfMTextFlatColumnDirectory, DxfMTextFlatColumnEntry, DxfMTextFlatColumnRole,
    DxfMTextFlatColumnValue,
};
pub use mtext_layout::{
    DxfMTextAttachment, DxfMTextAttachmentSemantic, DxfMTextDrawingDirection,
    DxfMTextDrawingDirectionSemantic, DxfMTextLayoutDirectory, DxfMTextLayoutIssue,
    DxfMTextLayoutSemantics, DxfMTextLineSpacingStyle, DxfMTextLineSpacingStyleSemantic,
};
pub use mtext_numeric_domain::{
    DxfMTextActualWidthRelation, DxfMTextActualWidthRelationSemantic,
    DxfMTextBackgroundFillSetting, DxfMTextBackgroundFillSettingSemantic,
    DxfMTextLineSpacingFactor, DxfMTextLineSpacingFactorSemantic, DxfMTextNumericDomainDirectory,
    DxfMTextNumericDomainIssue, DxfMTextNumericDomainSemantics,
};
pub use mtext_orientation::{
    DxfMTextOrientationDirectory, DxfMTextOrientationInput, DxfMTextOrientationInputSemantic,
    DxfMTextOrientationIssue, DxfMTextOrientationSemantics, DxfMTextRotationSemantic,
};
pub use mtext_tolerance_scalar::{
    DxfMTextNumericSemantics, DxfMTextToleranceScalarDirectory, DxfToleranceNumericSemantics,
};
pub use mtext_x_axis_direction::{
    DxfMTextXAxisComponent, DxfMTextXAxisDirection, DxfMTextXAxisDirectionDirectory,
    DxfMTextXAxisDirectionIssue, DxfMTextXAxisDirectionSemantic, DxfMTextXAxisDirectionSemantics,
};
pub use mtext_xdata_column_evidence::{
    DxfMTextXDataColumnDirectory, DxfMTextXDataColumnEntry, DxfMTextXDataColumnRole,
    DxfMTextXDataColumnValue,
};
pub use mtext_xdata_linked_column::{
    DxfMTextXDataLinkedColumnDirectory, DxfMTextXDataLinkedColumnEntry,
};
pub use mtext_xdata_linked_column_resolution::{
    DxfMTextXDataLinkedColumnResolutionDirectory, DxfMTextXDataLinkedColumnResolutionEntry,
    DxfMTextXDataLinkedColumnTarget, DxfMTextXDataLinkedColumnTargetState,
};
pub use named_symbol_table::{
    DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind,
};
pub use owner_evidence_comparison::{
    DxfOwnerEvidenceComparisonDirectory, DxfOwnerEvidenceComparisonEntry,
    DxfOwnerEvidenceComparisonState,
};
pub use ownership_evidence::{
    DxfIncomingOwnershipState, DxfOwnershipEvidenceDirectory, DxfOwnershipEvidenceEntry,
    DxfOwnershipLinkRange, DxfOwnershipTargetEntry, DxfResolvedOwnershipLink,
};
pub use planar_face_geometry::{
    DxfPlanarFaceDirectory, DxfPlanarFaceKind, DxfPlanarFaceNumber, DxfPlanarFaceNumericIssue,
    DxfPlanarFaceRecordEntry, DxfPlanarFaceValue, DxfPlanarFaceValueRange, DxfPlanarFaceValueRole,
};
pub use planar_face_geometry_card::{
    DxfPlanarFaceCardDirectory, DxfPlanarFaceCardMember, DxfPlanarFaceCardMemberRange,
    DxfPlanarFaceValueCard, DxfPlanarFaceValueCardState,
};
pub use planar_face_geometry_semantic::{
    DxfPlanarFaceDoubleSemanticValue, DxfPlanarFaceInt16SemanticValue,
    DxfPlanarFaceSemanticDirectory, DxfPlanarFaceSemanticIssue, DxfPlanarFaceSemantics,
};
pub use planar_face_wcs_geometry::{
    DxfPlanarFaceWcsGeometry, DxfPlanarFaceWcsGeometryDirectory, DxfPlanarFaceWcsGeometryEntry,
    DxfPlanarFaceWcsGeometryIssue,
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
pub use shape_wcs_insertion::{
    DxfShapeInsertionComponent, DxfShapeWcsInsertion, DxfShapeWcsInsertionDirectory,
    DxfShapeWcsInsertionIssue, DxfShapeWcsInsertionSemantic, DxfShapeWcsInsertionSemantics,
};
pub use shape_wcs_orientation::{
    DxfShapeWcsOrientation, DxfShapeWcsOrientationDirectory, DxfShapeWcsOrientationIssue,
    DxfShapeWcsOrientationSemantic, DxfShapeWcsOrientationSemantics,
};
pub use source::{DxfByteSource, DxfFileSource, DxfMemorySource};
pub use source_id::DxfSourceId;
pub use source_scan::{DxfSourceScanReceipt, scan_dxf_source};
pub use spline_analytic::{
    DxfSplineAnalyticData, DxfSplineAnalyticDirectory, DxfSplineAnalyticEntry,
    DxfSplineAnalyticIssueKind, DxfSplineAnalyticIssues, DxfSplineAnalyticState,
};
pub use spline_analytic_value::{
    DxfSplineAnalyticControlPoint, DxfSplineAnalyticKnot, DxfSplineAnalyticPoint,
    DxfSplineAnalyticValueDirectory, DxfSplineAnalyticValueEntry, DxfSplineAnalyticValueIssueKind,
    DxfSplineAnalyticValueIssues, DxfSplineAnalyticValueRange, DxfSplineAnalyticValueState,
};
pub use spline_auxiliary::{
    DxfSplineAuxiliaryDirectory, DxfSplineVectorComponentState, DxfSplineVectorComponents,
    DxfSplineVectorEntry, DxfSplineVectorKind, DxfSplineVectorState,
    DxfSplineWeightCountDisposition, DxfSplineWeightEntry, DxfSplineWeightState,
};
pub use spline_auxiliary_semantic::{
    DxfSplineAuxiliarySemanticDirectory, DxfSplineEffectiveVector, DxfSplineVectorSemanticEntry,
    DxfSplineVectorSemanticIssue, DxfSplineVectorSemanticState, DxfSplineWeightSemanticEntry,
    DxfSplineWeightSemanticState, DxfSplineWeightValueRange, DxfSplineWeightValueState,
};
pub use spline_card::{
    DXF_SPLINE_ROLES, DxfSplineCardDirectory, DxfSplineCardMember, DxfSplineCardMemberRange,
    DxfSplineCardState, DxfSplineFlags, DxfSplineFlagsSemantic, DxfSplineValueCard,
};
pub use spline_count_relation::{
    DxfSplineCountDirectory, DxfSplineCountDisposition, DxfSplineCountEntry, DxfSplineCountKind,
    DxfSplineCountState,
};
pub use spline_evidence::{
    DxfSplineDirectory, DxfSplineNumber, DxfSplineNumericIssue, DxfSplineRecordEntry,
    DxfSplineValue, DxfSplineValueRange, DxfSplineValueRole,
};
pub use spline_point_tuple::{
    DxfSplinePointComponentCounts, DxfSplinePointComponents, DxfSplinePointKind,
    DxfSplinePointTuple, DxfSplinePointTupleDirectory, DxfSplinePointTupleEntry,
    DxfSplinePointTupleRange, DxfSplinePointTupleState,
};
pub use spline_relation::{
    DxfSplineLinearPlanarRelation, DxfSplinePlanarNormalRelation, DxfSplineRationalWeightRelation,
    DxfSplineRelationDirectory, DxfSplineRelationEntry,
};
pub use spline_scalar_semantic::{
    DXF_SPLINE_SCALAR_ROLES, DxfSplineScalarDirectory, DxfSplineScalarEntry, DxfSplineScalarState,
};
pub use spline_topology::{
    DxfSplineDegreeControlRelation, DxfSplineDegreeState, DxfSplineInvariantDisposition,
    DxfSplineKnotOrderState, DxfSplineNurbsCountRelation, DxfSplinePeriodicClosedRelation,
    DxfSplineTopologyDirectory, DxfSplineTopologyEntry,
};
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
pub use text_layout::{
    DxfTextGenerationFlags, DxfTextGenerationFlagsSemantic, DxfTextHorizontalJustification,
    DxfTextHorizontalJustificationSemantic, DxfTextLayoutDirectory, DxfTextLayoutIssue,
    DxfTextLayoutSemantics, DxfTextVerticalJustification, DxfTextVerticalJustificationSemantic,
};
pub use text_placement_anchor::{
    DxfTextJustificationAxis, DxfTextOcsPlacementAnchor, DxfTextOcsPlacementAnchorDirectory,
    DxfTextOcsPlacementAnchorIssue, DxfTextOcsPlacementAnchorKind,
    DxfTextOcsPlacementAnchorSemantic, DxfTextOcsPlacementAnchorSemantics,
    DxfTextPlacementComponent,
};
pub use text_shape_scalar::{
    DxfShapeNumericSemantics, DxfTextNumericSemantics, DxfTextShapeDoubleValue,
    DxfTextShapeInt16Value, DxfTextShapeScalarDirectory, DxfTextShapeScalarIssue,
    DxfTextSymbolDoubleValue, DxfTextSymbolInt16Value, DxfTextSymbolInt32Value,
    DxfTextSymbolScalarIssue,
};
pub use text_symbol_card::{
    DxfTextSymbolCardDirectory, DxfTextSymbolCardMember, DxfTextSymbolCardMemberRange,
    DxfTextSymbolValueCard, DxfTextSymbolValueCardState,
};
pub use text_symbol_evidence::{
    DxfTextSymbolDirectory, DxfTextSymbolKind, DxfTextSymbolNumericIssue, DxfTextSymbolRecordEntry,
    DxfTextSymbolValue, DxfTextSymbolValueData, DxfTextSymbolValueRange, DxfTextSymbolValueRole,
};
pub use text_symbol_text::{
    DxfMTextChunkEntry, DxfMTextChunkKind, DxfMTextChunkRange, DxfMTextChunkSequence,
    DxfMTextFieldSemantics, DxfShapeFieldSemantics, DxfTextFieldSemantics,
    DxfTextSymbolSemanticStyle, DxfTextSymbolSemanticText, DxfTextSymbolStyleName,
    DxfTextSymbolTextDirectory, DxfTextSymbolTextIssue, DxfToleranceFieldSemantics,
};
pub use text_view::DxfTextValueDecodeReceipt;
pub use text_wcs_anchor::{
    DxfTextExtrusionComponent, DxfTextWcsPlacementAnchor, DxfTextWcsPlacementAnchorDirectory,
    DxfTextWcsPlacementAnchorIssue, DxfTextWcsPlacementAnchorSemantic,
    DxfTextWcsPlacementAnchorSemantics,
};
pub use text_wcs_orientation::{
    DxfTextWcsOrientation, DxfTextWcsOrientationDirectory, DxfTextWcsOrientationIssue,
    DxfTextWcsOrientationSemantic, DxfTextWcsOrientationSemantics,
};
pub use tolerance_dimstyle_resolution::{
    DxfToleranceDimStyleResolutionDirectory, DxfToleranceDimStyleResolutionEntry,
    DxfToleranceDimStyleResolutionState, DxfToleranceDimStyleTargetRange,
};
pub use tolerance_wcs_placement::{
    DxfToleranceWcsComponent, DxfToleranceWcsPlacement, DxfToleranceWcsPlacementDirectory,
    DxfToleranceWcsPlacementIssue, DxfToleranceWcsPlacementSemantic,
    DxfToleranceWcsPlacementSemantics, DxfToleranceWcsVector,
};
pub use transaction_plan::{
    DxfTransactionByteRange, DxfTransactionPatch, DxfTransactionPlan, DxfTransactionPlanBuilder,
};
pub use transaction_write::{DxfTransactionWriteJournal, DxfTransactionWriteReceipt};
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

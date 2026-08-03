//! Unified batching for common-entity singleton updates.

use std::{fmt, io};

use crate::entity_common_color_book_edit::classify_with_color_domains;
use crate::entity_common_layout_edit::classify_with_layouts;
use crate::entity_common_reference_edit::classify_with_identities;
use crate::entity_common_reference_target::reviewed_common_reference_target_kind;
use crate::entity_common_symbol_edit::classify_with_symbols;
use crate::entity_common_text_semantic::reviewed_common_symbol_kind;
use crate::entity_draft_applicability::admit_entity_draft_name;
use crate::entity_draft_record::{
    DxfEntityDraftEncodingContext, DxfPointDraftRecordExpectation, encode_entity_draft_record_parts,
};
use crate::entity_edit_verification::{DxfEntityEditExpectation, DxfEntityExpectedField};
use crate::point_edit::{
    DxfPointExtrusionResetPlan, DxfPointExtrusionSetDisposition, DxfPointThicknessResetPlan,
    DxfPointThicknessSetDisposition, DxfPointUcsXAxisAngleResetPlan,
    DxfPointUcsXAxisAngleSetDisposition, plan_point_extrusion_edit, plan_point_extrusion_reset,
    plan_point_location_edit, plan_point_thickness_edit, plan_point_thickness_reset,
    plan_point_ucs_x_axis_angle_edit, plan_point_ucs_x_axis_angle_reset,
};
use crate::{
    ByteSpan, DxfAcadVersion, DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfCommonOwnerCandidateState, DxfEntityClassification,
    DxfEntityCommonColorBookEditIssue, DxfEntityCommonColorBookEditOutcome,
    DxfEntityCommonFieldDomainDirectory, DxfEntityCommonFieldDomainIssue,
    DxfEntityCommonFieldDomainOutcome, DxfEntityCommonLayoutEditIssue,
    DxfEntityCommonLayoutEditOutcome, DxfEntityCommonReferenceEditIssue,
    DxfEntityCommonReferenceEditOutcome, DxfEntityCommonSymbolEditIssue,
    DxfEntityCommonSymbolEditOutcome, DxfEntityDraft, DxfEntityDraftApplicabilityIssue,
    DxfEntityDraftName, DxfEntityDraftRecordIssue, DxfEntityEditPlan, DxfEntityEditValue,
    DxfEntityField, DxfEntityFieldCardState, DxfEntityFieldEvidenceDirectory,
    DxfEntityFieldInsertionIssue, DxfEntityFieldInsertionOutcome, DxfEntityFieldReplacementIssue,
    DxfEntityFieldReplacementOutcome, DxfEntityFieldResetIssue, DxfEntityFieldResetOutcome,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityIndexedColor, DxfEntityKey,
    DxfEntityLineweight, DxfEntityPlacement, DxfEntityPlacementOwnerIssue,
    DxfEntityPlacementOwnerOutcome, DxfEntityPlacementTarget, DxfEntityProxyGraphicsState,
    DxfEntityShadowMode, DxfEntitySpace, DxfEntityTransparency, DxfEntityTrueColor,
    DxfEntityVisibility, DxfError, DxfHandle, DxfHandleAllocationOutcome,
    DxfHandleAllocationPolicyState, DxfHandleGroupClass, DxfHandleIdentityDirectory,
    DxfHandleIdentityLookup, DxfHandleIdentityState, DxfHandleReservationPlanOutcome,
    DxfHandleResolutionState, DxfIoOperation, DxfLayoutObjectDirectory,
    DxfNamedSymbolTableDirectory, DxfPointDraft, DxfPointEditIssue, DxfPointPatch,
    DxfPointPatchKind, DxfRawDocumentView, DxfResource, DxfResourceProfile, DxfSemanticValueState,
    DxfSourceId, DxfTransactionPlan,
};

/// One atomic replacement of the common indexed/true/color-book tuple.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonColorBookPatch<'a> {
    name: &'a [u8],
    indexed_color: crate::DxfEntityIndexedColor,
    true_color: crate::DxfEntityTrueColor,
}

impl fmt::Debug for DxfEntityCommonColorBookPatch<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityCommonColorBookPatch")
            .field("name_byte_count", &self.name.len())
            .field("indexed_color", &self.indexed_color)
            .field("true_color", &self.true_color)
            .finish()
    }
}

impl<'a> DxfEntityCommonColorBookPatch<'a> {
    #[must_use]
    pub const fn new(
        name: &'a [u8],
        indexed_color: crate::DxfEntityIndexedColor,
        true_color: crate::DxfEntityTrueColor,
    ) -> Self {
        Self {
            name,
            indexed_color,
            true_color,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'a [u8] {
        self.name
    }

    #[must_use]
    pub const fn indexed_color(self) -> crate::DxfEntityIndexedColor {
        self.indexed_color
    }

    #[must_use]
    pub const fn true_color(self) -> crate::DxfEntityTrueColor {
        self.true_color
    }
}

/// One typed common-field operation accepted by an entity edit session.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonFieldPatch<'a> {
    SetExplicit {
        field: DxfEntityField,
        value: DxfEntityEditValue<'a>,
    },
    ResetToDefault {
        field: DxfEntityField,
    },
}

impl DxfEntityCommonFieldPatch<'_> {
    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        match self {
            Self::SetExplicit { field, .. } | Self::ResetToDefault { field } => field,
        }
    }
}

/// Topic-discriminated entity update. More topic variants are added by family.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityPatch<'a> {
    CommonField(DxfEntityCommonFieldPatch<'a>),
    CommonColorBook(DxfEntityCommonColorBookPatch<'a>),
    /// Removes every explicit member of the common 62/420/430 color-book tuple.
    ResetCommonColorBook,
    Point(DxfPointPatch),
}

impl<'a> DxfEntityPatch<'a> {
    #[must_use]
    pub const fn common_field(self) -> Option<DxfEntityCommonFieldPatch<'a>> {
        match self {
            Self::CommonField(patch) => Some(patch),
            Self::CommonColorBook(_) | Self::ResetCommonColorBook | Self::Point(_) => None,
        }
    }
}

/// Typed reason why an update was not admitted to the session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityEditIssue {
    DuplicateFieldEdit {
        key: DxfEntityKey,
        field: DxfEntityField,
    },
    Domain(DxfEntityCommonFieldDomainIssue),
    ColorBook(DxfEntityCommonColorBookEditIssue),
    Reference(DxfEntityCommonReferenceEditIssue),
    Layout(DxfEntityCommonLayoutEditIssue),
    Symbol(DxfEntityCommonSymbolEditIssue),
    Insertion(DxfEntityFieldInsertionIssue),
    Replacement(DxfEntityFieldReplacementIssue),
    Reset(DxfEntityFieldResetIssue),
    Point(DxfPointEditIssue),
    InsertPending,
    DeletePending,
}

/// Effect of one accepted update request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityEditDisposition {
    AlreadyImplicit,
    Inserted,
    Replaced,
    Reset,
    Composite,
}

/// Non-payload receipt for one accepted update request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityEditReceipt {
    key: DxfEntityKey,
    field: DxfEntityField,
    disposition: DxfEntityEditDisposition,
    queued_edit_count: u32,
}

impl DxfEntityEditReceipt {
    #[must_use]
    pub const fn key(self) -> DxfEntityKey {
        self.key
    }

    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn disposition(self) -> DxfEntityEditDisposition {
        self.disposition
    }

    #[must_use]
    pub const fn queued_edit_count(self) -> u64 {
        self.queued_edit_count as u64
    }
}

/// Result of applying one update request to an edit session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityEditOutcome {
    Applied(DxfEntityEditReceipt),
    PointApplied(DxfPointEditReceipt),
    Unavailable(DxfEntityEditIssue),
}

/// Payload-free receipt for one accepted POINT-family update.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPointEditReceipt {
    key: DxfEntityKey,
    kind: DxfPointPatchKind,
    disposition: DxfEntityEditDisposition,
    queued_edit_count: u32,
}

impl DxfPointEditReceipt {
    #[must_use]
    pub const fn key(self) -> DxfEntityKey {
        self.key
    }

    #[must_use]
    pub const fn kind(self) -> DxfPointPatchKind {
        self.kind
    }

    #[must_use]
    pub const fn disposition(self) -> DxfEntityEditDisposition {
        self.disposition
    }

    #[must_use]
    pub const fn queued_edit_count(self) -> u64 {
        self.queued_edit_count as u64
    }
}

/// Compact placement-owner failure retained by a whole-entity insert request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityInsertOwnerIssue {
    PlacementUnavailable {
        target: DxfEntityPlacementTarget,
    },
    NullOwner,
    OwnerMissing {
        handle: DxfHandle,
    },
    OwnerAmbiguous {
        handle: DxfHandle,
        target_count: u32,
    },
    OwnerNotBlockRecord {
        handle: DxfHandle,
        raw_record_ordinal: u64,
    },
    BlockOwnerCardinality {
        state: DxfCommonOwnerCandidateState,
    },
    BlockOwnerResolution {
        state: DxfHandleResolutionState,
    },
    BlockOwnerMismatch {
        declared: DxfHandle,
        requested: DxfHandle,
    },
}

/// Typed reason why a whole-entity insert was not admitted to the session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityInsertIssue {
    OwnerRequired,
    UpdatePending {
        queued_update_count: u32,
    },
    Owner(DxfEntityInsertOwnerIssue),
    HandlePolicy(DxfHandleAllocationPolicyState),
    HandleExhausted {
        handseed: DxfHandle,
        requested_count: u64,
    },
    Applicability(DxfEntityDraftApplicabilityIssue),
    Record(DxfEntityDraftRecordIssue),
    DeletePending,
}

/// Non-payload receipt for one admitted whole-entity insertion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityInsertReceipt {
    handle: DxfHandle,
    name: DxfEntityDraftName,
    placement: DxfEntityPlacementTarget,
}

impl DxfEntityInsertReceipt {
    #[must_use]
    pub const fn handle(self) -> DxfHandle {
        self.handle
    }

    #[must_use]
    pub const fn name(self) -> DxfEntityDraftName {
        self.name
    }

    #[must_use]
    pub const fn placement(self) -> DxfEntityPlacementTarget {
        self.placement
    }
}

/// Result of adding one whole-entity insertion to an edit session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityInsertOutcome {
    Applied(DxfEntityInsertReceipt),
    Unavailable(DxfEntityInsertIssue),
}

/// Typed reason why a whole-entity delete was not admitted to the session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDeleteIssue {
    PendingOperations {
        queued_operation_count: u32,
    },
    EntityMissing {
        key: DxfEntityKey,
    },
    WrongClassification {
        key: DxfEntityKey,
        observed: DxfEntityClassification,
    },
    DuplicateDelete {
        key: DxfEntityKey,
    },
    SourceUpdatePending {
        key: DxfEntityKey,
    },
    IdentityUnavailable {
        key: DxfEntityKey,
        state: DxfHandleIdentityState,
    },
    AmbiguousIdentity {
        key: DxfEntityKey,
        handle: DxfHandle,
        target_count: u32,
    },
    IncomingReference {
        key: DxfEntityKey,
        handle: DxfHandle,
        source_record_ordinal: u64,
        group_occurrence: u64,
        class: DxfHandleGroupClass,
    },
    AttachedGraphScope {
        key: DxfEntityKey,
        group_occurrence: u64,
        group_code: i16,
    },
}

/// Non-payload receipt for one admitted whole-entity deletion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityDeleteReceipt {
    key: DxfEntityKey,
    handle: DxfHandle,
}

impl DxfEntityDeleteReceipt {
    #[must_use]
    pub const fn key(self) -> DxfEntityKey {
        self.key
    }

    #[must_use]
    pub const fn handle(self) -> DxfHandle {
        self.handle
    }
}

/// Non-payload receipt for one admitted handleless whole-entity deletion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityHandlelessDeleteReceipt {
    key: DxfEntityKey,
}

impl DxfEntityHandlelessDeleteReceipt {
    #[must_use]
    pub const fn key(self) -> DxfEntityKey {
        self.key
    }
}

/// Result of adding one whole-entity deletion to an edit session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDeleteOutcome {
    Applied(DxfEntityDeleteReceipt),
    HandlelessApplied(DxfEntityHandlelessDeleteReceipt),
    Unavailable(DxfEntityDeleteIssue),
}

/// Typed reason why a semantic entity clone was not admitted to the session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCloneIssue {
    SourceUpdatePending {
        key: DxfEntityKey,
    },
    SourceDeletePending {
        key: DxfEntityKey,
    },
    EntityMissing {
        key: DxfEntityKey,
    },
    WrongClassification {
        key: DxfEntityKey,
        observed: DxfEntityClassification,
    },
    VersionUnavailable {
        state: DxfAcadVersionState,
    },
    SourcePlacementUnavailable {
        key: DxfEntityKey,
    },
    PlacementMismatch {
        key: DxfEntityKey,
        expected: DxfEntityPlacementTarget,
        requested: DxfEntityPlacementTarget,
    },
    OwnerMismatch {
        key: DxfEntityKey,
        expected: DxfHandle,
        requested: DxfHandle,
    },
    UnsupportedSourceGroup {
        key: DxfEntityKey,
        group_occurrence: u64,
        group_code: i16,
    },
    CommonFieldUnavailable {
        key: DxfEntityKey,
        field: DxfEntityField,
        state: DxfEntityFieldCardState,
    },
    PointSemanticsUnavailable {
        key: DxfEntityKey,
    },
    PointFieldUnavailable {
        key: DxfEntityKey,
        kind: DxfPointPatchKind,
    },
    ProxyGraphicsUnavailable {
        key: DxfEntityKey,
        state: DxfEntityProxyGraphicsState,
    },
    Insert(DxfEntityInsertIssue),
}

/// Result of cloning one supported entity into a fresh identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCloneOutcome {
    Applied(DxfEntityInsertReceipt),
    Unavailable(DxfEntityCloneIssue),
}

struct PendingEdit {
    key: DxfEntityKey,
    field: DxfEntityField,
    write_order: u8,
    source_span: ByteSpan,
    replacement: Box<[u8]>,
    expected: DxfEntityExpectedField,
}

struct PendingInsert {
    handle: DxfHandle,
    owner: DxfHandle,
    version: DxfAcadVersion,
    placement: DxfEntityPlacement,
    bytes: Box<[u8]>,
    expectation: DxfPointDraftRecordExpectation,
}

struct PendingDelete {
    key: DxfEntityKey,
    expectation: PendingDeleteExpectation,
    transaction: DxfTransactionPlan,
}

#[derive(Clone, Copy)]
enum PendingDeleteExpectation {
    Handle(DxfHandle),
    Handleless,
}

struct OwnedPointCloneDraft {
    layer: Box<[u8]>,
    layout: Option<Box<[u8]>>,
    linetype: Option<Box<[u8]>>,
    material: Option<DxfHandle>,
    plot_style: Option<DxfHandle>,
    space: Option<DxfEntitySpace>,
    indexed_color: Option<DxfEntityIndexedColor>,
    lineweight: Option<DxfEntityLineweight>,
    linetype_scale: Option<crate::DxfDouble>,
    visibility: Option<DxfEntityVisibility>,
    proxy_graphics: Option<Box<[u8]>>,
    true_color: Option<DxfEntityTrueColor>,
    color_name: Box<[u8]>,
    transparency: Option<DxfEntityTransparency>,
    shadow_mode: Option<DxfEntityShadowMode>,
    location: [crate::DxfDouble; 3],
    thickness: Option<crate::DxfDouble>,
    extrusion: Option<[crate::DxfDouble; 3]>,
    ucs_x_axis_angle: Option<crate::DxfDouble>,
}

type PointCloneTextResult = Result<Option<Box<[u8]>>, DxfEntityCloneIssue>;

impl OwnedPointCloneDraft {
    fn borrowed(&self, owner: DxfHandle) -> DxfEntityDraft<'_> {
        let mut point = DxfPointDraft::new(&self.layer, self.location);
        if let Some(layout) = self.layout.as_deref() {
            point = point.with_layout(layout);
        }
        if let Some(space) = self.space {
            point = point.with_space(space);
        }
        if let Some(linetype) = self.linetype.as_deref() {
            point = point.with_linetype(linetype);
        }
        if let Some(material) = self.material {
            point = point.with_material(material);
        }
        if let Some(plot_style) = self.plot_style {
            point = point.with_plot_style(plot_style);
        }
        if let Some(color) = self.indexed_color {
            point = point.with_indexed_color(color);
        }
        if let Some(lineweight) = self.lineweight {
            point = point.with_lineweight(lineweight);
        }
        if let Some(scale) = self.linetype_scale {
            point = point.with_linetype_scale(scale);
        }
        if let Some(visibility) = self.visibility {
            point = point.with_visibility(visibility);
        }
        if let Some(payload) = self.proxy_graphics.as_deref() {
            point = point.with_proxy_graphics(payload);
        }
        if let Some(color) = self.true_color {
            point = point.with_true_color(color);
        }
        if !self.color_name.is_empty() {
            point = point.with_color_name(&self.color_name);
        }
        if let Some(transparency) = self.transparency {
            point = point.with_transparency(transparency);
        }
        if let Some(shadow_mode) = self.shadow_mode {
            point = point.with_shadow_mode(shadow_mode);
        }
        if let Some(thickness) = self.thickness {
            point = point.with_thickness(thickness);
        }
        if let Some(extrusion) = self.extrusion {
            point = point.with_extrusion(extrusion);
        }
        if let Some(angle) = self.ucs_x_axis_angle {
            point = point.with_ucs_x_axis_angle(angle);
        }
        DxfEntityDraft::point(point).with_owner(owner)
    }
}

enum PendingPointExpectation {
    Location([crate::DxfDouble; 3]),
    Thickness(crate::DxfDouble),
    ThicknessReset,
    Extrusion([crate::DxfDouble; 3]),
    ExtrusionReset,
    UcsXAxisAngle(crate::DxfDouble),
    UcsXAxisAngleReset,
}

struct PendingPointEdit {
    key: DxfEntityKey,
    kind: DxfPointPatchKind,
    transaction: DxfTransactionPlan,
    expectation: PendingPointExpectation,
}

/// Source-bound batch of entity edits that finishes as one immutable transaction.
pub struct DxfEntityEditSession<'document, 'evidence, 'cancellation> {
    document: DxfRawDocumentView<'document>,
    evidence: &'evidence DxfEntityFieldEvidenceDirectory,
    profile: DxfResourceProfile,
    cancellation: &'cancellation DxfCancellationToken,
    common_field_domains: Option<DxfEntityCommonFieldDomainDirectory>,
    handle_identities: Option<DxfHandleIdentityDirectory>,
    layout_objects: Option<DxfLayoutObjectDirectory>,
    named_symbols: Option<DxfNamedSymbolTableDirectory>,
    pending: Vec<PendingEdit>,
    pending_point_edits: Vec<PendingPointEdit>,
    pending_inserts: Vec<PendingInsert>,
    pending_deletes: Vec<PendingDelete>,
}

impl fmt::Debug for DxfEntityEditSession<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityEditSession")
            .field("source_id", &self.document.source_id())
            .field("format", &self.document.format())
            .field("queued_edit_count", &self.queued_edit_count())
            .finish()
    }
}

impl<'document, 'evidence, 'cancellation>
    DxfEntityEditSession<'document, 'evidence, 'cancellation>
{
    fn new(
        document: DxfRawDocumentView<'document>,
        evidence: &'evidence DxfEntityFieldEvidenceDirectory,
        profile: DxfResourceProfile,
        cancellation: &'cancellation DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(document.source_id(), evidence.source_id())?;
        Ok(Self {
            document,
            evidence,
            profile,
            cancellation,
            common_field_domains: None,
            handle_identities: None,
            layout_objects: None,
            named_symbols: None,
            pending: Vec::new(),
            pending_point_edits: Vec::new(),
            pending_inserts: Vec::new(),
            pending_deletes: Vec::new(),
        })
    }

    #[must_use]
    pub fn source_id(&self) -> DxfSourceId {
        self.document.source_id()
    }

    #[must_use]
    pub fn queued_edit_count(&self) -> u64 {
        self.pending.len() as u64
            + self.pending_point_edits.len() as u64
            + self.pending_inserts.len() as u64
            + self.pending_deletes.len() as u64
    }

    /// Adds one complete typed entity insertion without changing the source.
    ///
    /// Every admitted draft is copied into session-owned encoded evidence.
    pub fn insert(
        &mut self,
        placement: DxfEntityPlacement,
        draft: DxfEntityDraft<'_>,
    ) -> Result<DxfEntityInsertOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        let Some(owner) = draft.owner() else {
            return Ok(DxfEntityInsertOutcome::Unavailable(
                DxfEntityInsertIssue::OwnerRequired,
            ));
        };
        let owners = self
            .document
            .entity_placement_owner_directory(self.cancellation)?;
        let binding = match owners.bind(placement, owner, self.cancellation)? {
            DxfEntityPlacementOwnerOutcome::Bound(binding) => binding,
            DxfEntityPlacementOwnerOutcome::Rejected(issue) => {
                return Ok(DxfEntityInsertOutcome::Unavailable(
                    DxfEntityInsertIssue::Owner(compact_owner_issue(issue)),
                ));
            }
        };
        let policy = self
            .document
            .handle_allocation_policy_directory(self.cancellation)?;
        let next_count = self
            .pending_inserts
            .len()
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        let requested_count = u64::try_from(next_count).map_err(|_| invalid_internal_data())?;
        let allocation = match policy.propose_allocation(requested_count, self.profile)? {
            DxfHandleAllocationOutcome::Proposed(allocation) => allocation,
            DxfHandleAllocationOutcome::Unavailable { state } => {
                return Ok(DxfEntityInsertOutcome::Unavailable(
                    DxfEntityInsertIssue::HandlePolicy(state),
                ));
            }
            DxfHandleAllocationOutcome::Exhausted {
                handseed,
                requested_count,
            } => {
                return Ok(DxfEntityInsertOutcome::Unavailable(
                    DxfEntityInsertIssue::HandleExhausted {
                        handseed,
                        requested_count,
                    },
                ));
            }
        };
        let handle = allocation
            .handle_at(requested_count - 1)
            .ok_or_else(invalid_internal_data)?;
        let name = draft.name();
        let (version, _) =
            match admit_entity_draft_name(name, self.document.acad_version_report().state())? {
                Ok(admitted) => admitted,
                Err(issue) => {
                    return Ok(DxfEntityInsertOutcome::Unavailable(
                        DxfEntityInsertIssue::Applicability(issue),
                    ));
                }
            };
        let context = DxfEntityDraftEncodingContext::new(
            name,
            version,
            handle,
            binding.owner_handle(),
            placement.target(),
        );
        let encoded = match encode_entity_draft_record_parts(
            self.document,
            context,
            draft,
            self.profile,
            self.cancellation,
        )? {
            Ok(plan) => plan,
            Err(issue) => {
                return Ok(DxfEntityInsertOutcome::Unavailable(
                    DxfEntityInsertIssue::Record(issue),
                ));
            }
        };
        let (bytes, expectation) = encoded.into_parts();
        let combined_count = self
            .queued_len()?
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        enforce_edit_limit(self.profile, combined_count)?;
        self.pending_inserts
            .try_reserve(1)
            .map_err(|_| out_of_memory())?;
        ensure_not_cancelled(self.cancellation)?;
        self.pending_inserts.push(PendingInsert {
            handle,
            owner: binding.owner_handle(),
            version,
            placement,
            bytes,
            expectation,
        });
        Ok(DxfEntityInsertOutcome::Applied(DxfEntityInsertReceipt {
            handle,
            name,
            placement: placement.target(),
        }))
    }

    /// Clones the currently supported canonical semantics into a fresh identity.
    ///
    /// Unsupported source groups fail closed so cloning never silently drops
    /// opaque or not-yet-modeled payload.
    pub fn clone_entity(
        &mut self,
        key: DxfEntityKey,
        placement: DxfEntityPlacement,
        owner: DxfHandle,
    ) -> Result<DxfEntityCloneOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        if self.pending_deletes.iter().any(|delete| delete.key == key) {
            return Ok(DxfEntityCloneOutcome::Unavailable(
                DxfEntityCloneIssue::SourceDeletePending { key },
            ));
        }
        if self.pending.iter().any(|edit| edit.key == key)
            || self.pending_point_edits.iter().any(|edit| edit.key == key)
        {
            return Ok(DxfEntityCloneOutcome::Unavailable(
                DxfEntityCloneIssue::SourceUpdatePending { key },
            ));
        }
        let draft = match prepare_point_clone_draft(
            self.document,
            self.evidence,
            key,
            placement.target(),
            owner,
            self.profile,
            self.cancellation,
        )? {
            Ok(draft) => draft,
            Err(issue) => return Ok(DxfEntityCloneOutcome::Unavailable(issue)),
        };
        match self.insert(placement, draft.borrowed(owner))? {
            DxfEntityInsertOutcome::Applied(receipt) => Ok(DxfEntityCloneOutcome::Applied(receipt)),
            DxfEntityInsertOutcome::Unavailable(issue) => Ok(DxfEntityCloneOutcome::Unavailable(
                DxfEntityCloneIssue::Insert(issue),
            )),
        }
    }

    /// Adds one typed update without changing the source document.
    pub fn update(
        &mut self,
        key: DxfEntityKey,
        patch: DxfEntityPatch<'_>,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        if self.pending_deletes.iter().any(|delete| delete.key == key) {
            return Ok(DxfEntityEditOutcome::Unavailable(
                DxfEntityEditIssue::DeletePending,
            ));
        }
        match patch {
            DxfEntityPatch::CommonField(patch) => self.update_common_field(key, patch),
            DxfEntityPatch::CommonColorBook(patch) => self.update_color_book(key, patch),
            DxfEntityPatch::ResetCommonColorBook => self.reset_color_book(key),
            DxfEntityPatch::Point(patch) => self.update_point(key, patch),
        }
    }

    /// Deletes one standalone canonical POINT record when reference safety is proven.
    ///
    /// Sessions may queue multiple distinct records and unrelated edits. Any
    /// uniquely resolved incoming pointer or owner from another record blocks
    /// deletion.
    pub fn delete(&mut self, key: DxfEntityKey) -> Result<DxfEntityDeleteOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        if self.pending_deletes.iter().any(|delete| delete.key == key) {
            return Ok(DxfEntityDeleteOutcome::Unavailable(
                DxfEntityDeleteIssue::DuplicateDelete { key },
            ));
        }
        if self.pending.iter().any(|edit| edit.key == key)
            || self.pending_point_edits.iter().any(|edit| edit.key == key)
        {
            return Ok(DxfEntityDeleteOutcome::Unavailable(
                DxfEntityDeleteIssue::SourceUpdatePending { key },
            ));
        }
        let Some(entity) = self.evidence.entity_directory().entity_for_key(key)? else {
            return Ok(DxfEntityDeleteOutcome::Unavailable(
                DxfEntityDeleteIssue::EntityMissing { key },
            ));
        };
        if entity.classification()
            != DxfEntityClassification::Canonical(crate::DxfEntityTopic::POINT)
        {
            return Ok(DxfEntityDeleteOutcome::Unavailable(
                DxfEntityDeleteIssue::WrongClassification {
                    key,
                    observed: entity.classification(),
                },
            ));
        }
        let range = entity.record().group_range();
        for occurrence in range.start()..range.end() {
            ensure_not_cancelled(self.cancellation)?;
            let group = self
                .document
                .group(occurrence)
                .ok_or_else(invalid_internal_data)?;
            let group_code = group.group_code().value();
            if matches!(group_code, 102 | 360) {
                return Ok(DxfEntityDeleteOutcome::Unavailable(
                    DxfEntityDeleteIssue::AttachedGraphScope {
                        key,
                        group_occurrence: occurrence,
                        group_code,
                    },
                ));
            }
        }
        let resolutions = self
            .document
            .handle_resolution_directory(self.cancellation)?;
        let identity = resolutions
            .identity_directory()
            .entry(key.raw_record_ordinal())
            .ok_or_else(invalid_internal_data)?;
        let expectation = match identity.state() {
            DxfHandleIdentityState::Absent => PendingDeleteExpectation::Handleless,
            DxfHandleIdentityState::UniqueParsed(handle) if !handle.is_null() => {
                PendingDeleteExpectation::Handle(handle)
            }
            state => {
                return Ok(DxfEntityDeleteOutcome::Unavailable(
                    DxfEntityDeleteIssue::IdentityUnavailable { key, state },
                ));
            }
        };
        if let PendingDeleteExpectation::Handle(handle) = expectation {
            match resolutions.identity_directory().lookup(handle) {
                DxfHandleIdentityLookup::Unique(target)
                    if target.record().ordinal() == key.raw_record_ordinal() => {}
                DxfHandleIdentityLookup::Ambiguous(targets) => {
                    return Ok(DxfEntityDeleteOutcome::Unavailable(
                        DxfEntityDeleteIssue::AmbiguousIdentity {
                            key,
                            handle,
                            target_count: u32::try_from(targets.len())
                                .map_err(|_| invalid_internal_data())?,
                        },
                    ));
                }
                DxfHandleIdentityLookup::Missing | DxfHandleIdentityLookup::Unique(_) => {
                    return Err(invalid_internal_data());
                }
            }
            for (ordinal, resolution) in resolutions.entries().iter().copied().enumerate() {
                ensure_not_cancelled(self.cancellation)?;
                if resolution.reference().record().ordinal() == key.raw_record_ordinal()
                    || resolution.state() != DxfHandleResolutionState::Unique
                {
                    continue;
                }
                let ordinal = u64::try_from(ordinal).map_err(|_| invalid_internal_data())?;
                let [target] = resolutions
                    .targets_for_reference(ordinal)
                    .ok_or_else(invalid_internal_data)?
                else {
                    return Err(invalid_internal_data());
                };
                if target.record().ordinal() == key.raw_record_ordinal() {
                    let reference = resolution.reference();
                    return Ok(DxfEntityDeleteOutcome::Unavailable(
                        DxfEntityDeleteIssue::IncomingReference {
                            key,
                            handle,
                            source_record_ordinal: reference.record().ordinal(),
                            group_occurrence: reference.value().group().occurrence(),
                            class: reference.class(),
                        },
                    ));
                }
            }
        }
        let first = self
            .document
            .group(range.start())
            .ok_or_else(invalid_internal_data)?;
        let last_ordinal = range
            .end()
            .checked_sub(1)
            .ok_or_else(invalid_internal_data)?;
        let last = self
            .document
            .group(last_ordinal)
            .ok_or_else(invalid_internal_data)?;
        let span = ByteSpan::new(first.full_span().start(), last.full_span().end())
            .ok_or_else(invalid_internal_data)?;
        let mut builder = self.document.transaction_plan_builder(self.profile)?;
        builder.replace_raw_span(span, &[], self.cancellation)?;
        let transaction = builder.finish(self.cancellation)?;
        let next_count = self
            .queued_len()?
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        enforce_edit_limit(self.profile, next_count)?;
        self.pending_deletes
            .try_reserve(1)
            .map_err(|_| out_of_memory())?;
        ensure_not_cancelled(self.cancellation)?;
        self.pending_deletes.push(PendingDelete {
            key,
            expectation,
            transaction,
        });
        Ok(match expectation {
            PendingDeleteExpectation::Handle(handle) => {
                DxfEntityDeleteOutcome::Applied(DxfEntityDeleteReceipt { key, handle })
            }
            PendingDeleteExpectation::Handleless => {
                DxfEntityDeleteOutcome::HandlelessApplied(DxfEntityHandlelessDeleteReceipt { key })
            }
        })
    }

    fn update_point(
        &mut self,
        key: DxfEntityKey,
        patch: DxfPointPatch,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        let kind = patch.kind();
        if self
            .pending_point_edits
            .iter()
            .any(|edit| edit.key == key && edit.kind == kind)
        {
            return Ok(DxfEntityEditOutcome::Unavailable(
                DxfEntityEditIssue::Point(DxfPointEditIssue::DuplicatePatch { key, kind }),
            ));
        }
        let (transaction, expectation, expected_patch_count, disposition) = match patch {
            DxfPointPatch::SetLocation { location } => {
                let plan = match plan_point_location_edit(
                    self.document,
                    self.evidence,
                    key,
                    location,
                    self.profile,
                    self.cancellation,
                )? {
                    Ok(plan) => plan,
                    Err(issue) => {
                        return Ok(DxfEntityEditOutcome::Unavailable(
                            DxfEntityEditIssue::Point(issue),
                        ));
                    }
                };
                let (transaction, location) = plan.into_parts();
                (
                    transaction,
                    PendingPointExpectation::Location(location),
                    3,
                    DxfEntityEditDisposition::Replaced,
                )
            }
            DxfPointPatch::SetThickness { thickness } => {
                let plan = match plan_point_thickness_edit(
                    self.document,
                    self.evidence,
                    key,
                    thickness,
                    self.profile,
                    self.cancellation,
                )? {
                    Ok(plan) => plan,
                    Err(issue) => {
                        return Ok(DxfEntityEditOutcome::Unavailable(
                            DxfEntityEditIssue::Point(issue),
                        ));
                    }
                };
                let (transaction, thickness, set_disposition) = plan.into_parts();
                let disposition = match set_disposition {
                    DxfPointThicknessSetDisposition::Inserted => DxfEntityEditDisposition::Inserted,
                    DxfPointThicknessSetDisposition::Replaced => DxfEntityEditDisposition::Replaced,
                };
                (
                    transaction,
                    PendingPointExpectation::Thickness(thickness),
                    1,
                    disposition,
                )
            }
            DxfPointPatch::ResetThickness => {
                let plan = match plan_point_thickness_reset(
                    self.document,
                    self.evidence,
                    key,
                    self.profile,
                    self.cancellation,
                )? {
                    Ok(plan) => plan,
                    Err(issue) => {
                        return Ok(DxfEntityEditOutcome::Unavailable(
                            DxfEntityEditIssue::Point(issue),
                        ));
                    }
                };
                match plan {
                    DxfPointThicknessResetPlan::AlreadyImplicit => {
                        return Ok(DxfEntityEditOutcome::PointApplied(DxfPointEditReceipt {
                            key,
                            kind,
                            disposition: DxfEntityEditDisposition::AlreadyImplicit,
                            queued_edit_count: u32::try_from(self.queued_len()?)
                                .map_err(|_| invalid_internal_data())?,
                        }));
                    }
                    DxfPointThicknessResetPlan::Planned(transaction) => (
                        transaction,
                        PendingPointExpectation::ThicknessReset,
                        1,
                        DxfEntityEditDisposition::Reset,
                    ),
                }
            }
            DxfPointPatch::SetExtrusion { extrusion } => {
                let plan = match plan_point_extrusion_edit(
                    self.document,
                    self.evidence,
                    key,
                    extrusion,
                    self.profile,
                    self.cancellation,
                )? {
                    Ok(plan) => plan,
                    Err(issue) => {
                        return Ok(DxfEntityEditOutcome::Unavailable(
                            DxfEntityEditIssue::Point(issue),
                        ));
                    }
                };
                let (transaction, extrusion, set_disposition, expected_patch_count) =
                    plan.into_parts();
                let disposition = match set_disposition {
                    DxfPointExtrusionSetDisposition::Inserted => DxfEntityEditDisposition::Inserted,
                    DxfPointExtrusionSetDisposition::Replaced => DxfEntityEditDisposition::Replaced,
                    DxfPointExtrusionSetDisposition::Composite => {
                        DxfEntityEditDisposition::Composite
                    }
                };
                (
                    transaction,
                    PendingPointExpectation::Extrusion(extrusion),
                    expected_patch_count,
                    disposition,
                )
            }
            DxfPointPatch::ResetExtrusion => {
                let plan = match plan_point_extrusion_reset(
                    self.document,
                    self.evidence,
                    key,
                    self.profile,
                    self.cancellation,
                )? {
                    Ok(plan) => plan,
                    Err(issue) => {
                        return Ok(DxfEntityEditOutcome::Unavailable(
                            DxfEntityEditIssue::Point(issue),
                        ));
                    }
                };
                match plan {
                    DxfPointExtrusionResetPlan::AlreadyImplicit => {
                        return Ok(DxfEntityEditOutcome::PointApplied(DxfPointEditReceipt {
                            key,
                            kind,
                            disposition: DxfEntityEditDisposition::AlreadyImplicit,
                            queued_edit_count: u32::try_from(self.queued_len()?)
                                .map_err(|_| invalid_internal_data())?,
                        }));
                    }
                    DxfPointExtrusionResetPlan::Planned {
                        transaction,
                        patch_count,
                    } => (
                        transaction,
                        PendingPointExpectation::ExtrusionReset,
                        patch_count,
                        DxfEntityEditDisposition::Reset,
                    ),
                }
            }
            DxfPointPatch::SetUcsXAxisAngle { angle } => {
                let plan = match plan_point_ucs_x_axis_angle_edit(
                    self.document,
                    self.evidence,
                    key,
                    angle,
                    self.profile,
                    self.cancellation,
                )? {
                    Ok(plan) => plan,
                    Err(issue) => {
                        return Ok(DxfEntityEditOutcome::Unavailable(
                            DxfEntityEditIssue::Point(issue),
                        ));
                    }
                };
                let (transaction, angle, set_disposition) = plan.into_parts();
                let disposition = match set_disposition {
                    DxfPointUcsXAxisAngleSetDisposition::Inserted => {
                        DxfEntityEditDisposition::Inserted
                    }
                    DxfPointUcsXAxisAngleSetDisposition::Replaced => {
                        DxfEntityEditDisposition::Replaced
                    }
                };
                (
                    transaction,
                    PendingPointExpectation::UcsXAxisAngle(angle),
                    1,
                    disposition,
                )
            }
            DxfPointPatch::ResetUcsXAxisAngle => {
                let plan = match plan_point_ucs_x_axis_angle_reset(
                    self.document,
                    self.evidence,
                    key,
                    self.profile,
                    self.cancellation,
                )? {
                    Ok(plan) => plan,
                    Err(issue) => {
                        return Ok(DxfEntityEditOutcome::Unavailable(
                            DxfEntityEditIssue::Point(issue),
                        ));
                    }
                };
                match plan {
                    DxfPointUcsXAxisAngleResetPlan::AlreadyImplicit => {
                        return Ok(DxfEntityEditOutcome::PointApplied(DxfPointEditReceipt {
                            key,
                            kind,
                            disposition: DxfEntityEditDisposition::AlreadyImplicit,
                            queued_edit_count: u32::try_from(self.queued_len()?)
                                .map_err(|_| invalid_internal_data())?,
                        }));
                    }
                    DxfPointUcsXAxisAngleResetPlan::Planned(transaction) => (
                        transaction,
                        PendingPointExpectation::UcsXAxisAngleReset,
                        1,
                        DxfEntityEditDisposition::Reset,
                    ),
                }
            }
        };
        transaction.validate_source_precondition(self.document)?;
        if transaction.patches().len() != expected_patch_count {
            return Err(invalid_internal_data());
        }
        let next_count = self
            .queued_len()?
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        enforce_edit_limit(self.profile, next_count)?;
        self.pending_point_edits
            .try_reserve(1)
            .map_err(|_| out_of_memory())?;
        ensure_not_cancelled(self.cancellation)?;
        self.pending_point_edits.push(PendingPointEdit {
            key,
            kind,
            transaction,
            expectation,
        });
        Ok(DxfEntityEditOutcome::PointApplied(DxfPointEditReceipt {
            key,
            kind,
            disposition,
            queued_edit_count: u32::try_from(next_count).map_err(|_| invalid_internal_data())?,
        }))
    }

    fn update_common_field(
        &mut self,
        key: DxfEntityKey,
        patch: DxfEntityCommonFieldPatch<'_>,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        let field = patch.field();
        if self
            .pending
            .iter()
            .any(|edit| edit.key == key && edit.field == field)
        {
            return Ok(DxfEntityEditOutcome::Unavailable(
                DxfEntityEditIssue::DuplicateFieldEdit { key, field },
            ));
        }
        match patch {
            DxfEntityCommonFieldPatch::SetExplicit { value, .. } => {
                enforce_exact_text_limit(self.profile, value)?;
                if let DxfEntityCommonReferenceEditOutcome::Invalid(issue) =
                    self.classify_reference_edit(field, value)?
                {
                    return Ok(DxfEntityEditOutcome::Unavailable(
                        DxfEntityEditIssue::Reference(issue),
                    ));
                }
                if let DxfEntityCommonLayoutEditOutcome::Invalid(issue) =
                    self.classify_layout_edit(field, value)?
                {
                    return Ok(DxfEntityEditOutcome::Unavailable(
                        DxfEntityEditIssue::Layout(issue),
                    ));
                }
                if let DxfEntityCommonColorBookEditOutcome::Invalid(issue) =
                    self.classify_color_book_edit(key, field, value)?
                {
                    return Ok(DxfEntityEditOutcome::Unavailable(
                        DxfEntityEditIssue::ColorBook(issue),
                    ));
                }
                if let DxfEntityCommonSymbolEditOutcome::Invalid(issue) =
                    self.classify_symbol_edit(field, value)?
                {
                    return Ok(DxfEntityEditOutcome::Unavailable(
                        DxfEntityEditIssue::Symbol(issue),
                    ));
                }
                if let DxfEntityCommonFieldDomainOutcome::Invalid(issue) =
                    crate::classify_entity_common_field_edit_domain(field, value)
                {
                    return Ok(DxfEntityEditOutcome::Unavailable(
                        DxfEntityEditIssue::Domain(issue),
                    ));
                }
                self.set_explicit(key, field, value)
            }
            DxfEntityCommonFieldPatch::ResetToDefault { .. } => self.reset(key, field),
        }
    }

    fn update_color_book(
        &mut self,
        key: DxfEntityKey,
        patch: DxfEntityCommonColorBookPatch<'_>,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        enforce_exact_text_limit(self.profile, DxfEntityEditValue::ExactRawText(patch.name()))?;
        if self.document.acad_version_report().state()
            == DxfAcadVersionState::Supported(DxfAcadVersion::Ac1009)
        {
            return Ok(DxfEntityEditOutcome::Unavailable(
                DxfEntityEditIssue::ColorBook(
                    DxfEntityCommonColorBookEditIssue::DialectWireUnsupported {
                        field: DxfEntityField::COLOR_NAME,
                        version: DxfAcadVersion::Ac1009,
                    },
                ),
            ));
        }
        if let Err(issue) = crate::entity_common_color_book_edit::validate_proposed_name(
            patch.name(),
            self.cancellation,
        )? {
            return Ok(DxfEntityEditOutcome::Unavailable(
                DxfEntityEditIssue::ColorBook(issue),
            ));
        }
        for field in [
            DxfEntityField::COLOR,
            DxfEntityField::TRUE_COLOR,
            DxfEntityField::COLOR_NAME,
        ] {
            if self
                .pending
                .iter()
                .any(|edit| edit.key == key && edit.field == field)
            {
                return Ok(DxfEntityEditOutcome::Unavailable(
                    DxfEntityEditIssue::DuplicateFieldEdit { key, field },
                ));
            }
        }
        let checkpoint = self.pending.len();
        let values = [
            (
                DxfEntityField::COLOR,
                DxfEntityEditValue::Int16(patch.indexed_color().raw()),
            ),
            (
                DxfEntityField::TRUE_COLOR,
                DxfEntityEditValue::Int32(patch.true_color().raw()),
            ),
            (
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(patch.name()),
            ),
        ];
        for (field, value) in values {
            match self.set_explicit(key, field, value) {
                Ok(DxfEntityEditOutcome::Applied(_)) => {}
                Ok(unavailable @ DxfEntityEditOutcome::Unavailable(_)) => {
                    self.pending.truncate(checkpoint);
                    return Ok(unavailable);
                }
                Ok(DxfEntityEditOutcome::PointApplied(_)) => {
                    self.pending.truncate(checkpoint);
                    return Err(invalid_internal_data());
                }
                Err(error) => {
                    self.pending.truncate(checkpoint);
                    return Err(error);
                }
            }
        }
        applied(
            key,
            DxfEntityField::COLOR_NAME,
            DxfEntityEditDisposition::Composite,
            self.queued_len()?,
        )
    }

    fn reset_color_book(&mut self, key: DxfEntityKey) -> Result<DxfEntityEditOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        let fields = [
            DxfEntityField::COLOR,
            DxfEntityField::TRUE_COLOR,
            DxfEntityField::COLOR_NAME,
        ];
        for field in fields {
            if self
                .pending
                .iter()
                .any(|edit| edit.key == key && edit.field == field)
            {
                return Ok(DxfEntityEditOutcome::Unavailable(
                    DxfEntityEditIssue::DuplicateFieldEdit { key, field },
                ));
            }
        }

        let checkpoint = self.pending.len();
        for field in fields {
            match self.reset(key, field) {
                Ok(DxfEntityEditOutcome::Applied(_)) => {}
                Ok(unavailable @ DxfEntityEditOutcome::Unavailable(_)) => {
                    self.pending.truncate(checkpoint);
                    return Ok(unavailable);
                }
                Ok(DxfEntityEditOutcome::PointApplied(_)) => {
                    self.pending.truncate(checkpoint);
                    return Err(invalid_internal_data());
                }
                Err(error) => {
                    self.pending.truncate(checkpoint);
                    return Err(error);
                }
            }
        }
        let disposition = if self.pending.len() == checkpoint {
            DxfEntityEditDisposition::AlreadyImplicit
        } else {
            DxfEntityEditDisposition::Composite
        };
        applied(
            key,
            DxfEntityField::COLOR_NAME,
            disposition,
            self.queued_len()?,
        )
    }

    fn classify_reference_edit(
        &mut self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
    ) -> Result<DxfEntityCommonReferenceEditOutcome, DxfError> {
        if reviewed_common_reference_target_kind(field).is_some()
            && matches!(value, DxfEntityEditValue::Handle(handle) if !handle.is_null())
            && self.handle_identities.is_none()
        {
            self.handle_identities =
                Some(self.document.handle_identity_directory(self.cancellation)?);
        }
        classify_with_identities(
            self.document,
            self.handle_identities.as_ref(),
            field,
            value,
            self.cancellation,
        )
    }

    fn classify_symbol_edit(
        &mut self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
    ) -> Result<DxfEntityCommonSymbolEditOutcome, DxfError> {
        if reviewed_common_symbol_kind(field).is_some()
            && matches!(value, DxfEntityEditValue::ExactRawText(_))
            && self.named_symbols.is_none()
        {
            self.named_symbols = Some(
                self.document
                    .named_symbol_table_directory(self.cancellation)?,
            );
        }
        classify_with_symbols(
            self.document,
            self.named_symbols.as_ref(),
            field,
            value,
            self.cancellation,
        )
    }

    fn classify_color_book_edit(
        &mut self,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
    ) -> Result<DxfEntityCommonColorBookEditOutcome, DxfError> {
        let entity = if field == DxfEntityField::COLOR_NAME
            && matches!(value, DxfEntityEditValue::ExactRawText(_))
        {
            self.evidence.entity_directory().entity_for_key(key)?
        } else {
            None
        };
        if entity.is_some() && self.common_field_domains.is_none() {
            self.common_field_domains = Some(
                self.document
                    .entity_common_field_domain_directory(self.cancellation)?,
            );
        }
        if field == DxfEntityField::COLOR_NAME
            && matches!(value, DxfEntityEditValue::ExactRawText(_))
            && entity.is_none()
        {
            return Ok(DxfEntityCommonColorBookEditOutcome::NotColorBook { field });
        }
        classify_with_color_domains(
            self.common_field_domains.as_ref(),
            entity,
            field,
            value,
            self.cancellation,
        )
    }

    fn classify_layout_edit(
        &mut self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
    ) -> Result<DxfEntityCommonLayoutEditOutcome, DxfError> {
        if field == DxfEntityField::LAYOUT
            && matches!(value, DxfEntityEditValue::ExactRawText(_))
            && self.layout_objects.is_none()
        {
            self.layout_objects = Some(self.document.layout_object_directory(self.cancellation)?);
        }
        classify_with_layouts(
            self.document,
            self.layout_objects.as_ref(),
            field,
            value,
            self.cancellation,
        )
    }

    /// Freezes every accepted update into one source-order transaction plan.
    pub fn finish(self) -> Result<DxfTransactionPlan, DxfError> {
        self.finish_verifiable()
            .map(DxfEntityEditPlan::into_transaction)
    }

    /// Freezes the transaction together with its semantic postconditions.
    pub fn finish_verifiable(mut self) -> Result<DxfEntityEditPlan, DxfError> {
        let (mut transaction, pending, point_edits) = self.finish_parts()?;
        let mut expectations = Vec::new();
        expectations
            .try_reserve_exact(
                pending
                    .len()
                    .checked_add(point_edits.len())
                    .and_then(|count| count.checked_add(self.pending_inserts.len()))
                    .and_then(|count| count.checked_add(self.pending_deletes.len()))
                    .ok_or_else(invalid_internal_data)?,
            )
            .map_err(|_| out_of_memory())?;
        for edit in pending {
            expectations.push(DxfEntityEditExpectation::field(
                self.post_session_raw_record_ordinal(edit.key)?,
                edit.field,
                edit.expected,
            ));
        }
        for edit in point_edits {
            let raw_record_ordinal = self.post_session_raw_record_ordinal(edit.key)?;
            expectations.push(match edit.expectation {
                PendingPointExpectation::Location(location) => {
                    DxfEntityEditExpectation::point_location(raw_record_ordinal, location)
                }
                PendingPointExpectation::Thickness(thickness) => {
                    DxfEntityEditExpectation::point_thickness(raw_record_ordinal, thickness)
                }
                PendingPointExpectation::ThicknessReset => {
                    DxfEntityEditExpectation::point_thickness_reset(raw_record_ordinal)
                }
                PendingPointExpectation::Extrusion(extrusion) => {
                    DxfEntityEditExpectation::point_extrusion(raw_record_ordinal, extrusion)
                }
                PendingPointExpectation::ExtrusionReset => {
                    DxfEntityEditExpectation::point_extrusion_reset(raw_record_ordinal)
                }
                PendingPointExpectation::UcsXAxisAngle(angle) => {
                    DxfEntityEditExpectation::point_ucs_x_axis_angle(raw_record_ordinal, angle)
                }
                PendingPointExpectation::UcsXAxisAngleReset => {
                    DxfEntityEditExpectation::point_ucs_x_axis_angle_reset(raw_record_ordinal)
                }
            });
        }
        let inserted_entity_count = self.pending_inserts.len();
        if !self.pending_inserts.is_empty() {
            let (insertion, insert_expectations) = self.finish_insert_parts()?;
            expectations.extend(insert_expectations);
            transaction = if transaction.patches().is_empty() {
                insertion
            } else {
                compose_session_transactions(
                    self.document,
                    &transaction,
                    &insertion,
                    self.profile,
                    self.cancellation,
                )?
            };
        }
        if !self.pending_deletes.is_empty() {
            let (deletion, delete_expectations) =
                self.finish_delete_parts(inserted_entity_count)?;
            expectations.extend(delete_expectations);
            transaction = if transaction.patches().is_empty() {
                deletion
            } else {
                compose_session_transactions(
                    self.document,
                    &transaction,
                    &deletion,
                    self.profile,
                    self.cancellation,
                )?
            };
        }
        Ok(DxfEntityEditPlan::new(transaction, expectations))
    }

    fn finish_delete_parts(
        &mut self,
        inserted_entity_count: usize,
    ) -> Result<(DxfTransactionPlan, Vec<DxfEntityEditExpectation>), DxfError> {
        let deletes = std::mem::take(&mut self.pending_deletes);
        let expected_entity_count = self
            .evidence
            .entity_directory()
            .entities()
            .len()
            .checked_add(inserted_entity_count)
            .and_then(|count| count.checked_sub(deletes.len()))
            .ok_or_else(invalid_internal_data)?;
        let expected_entity_count =
            u64::try_from(expected_entity_count).map_err(|_| invalid_internal_data())?;
        let mut plans = Vec::new();
        plans
            .try_reserve_exact(deletes.len())
            .map_err(|_| out_of_memory())?;
        let mut expectations = Vec::new();
        expectations
            .try_reserve_exact(deletes.len())
            .map_err(|_| out_of_memory())?;
        for delete in &deletes {
            ensure_source(self.document.source_id(), delete.key.source_id())?;
            plans.push(&delete.transaction);
            expectations.push(match delete.expectation {
                PendingDeleteExpectation::Handle(handle) => {
                    DxfEntityEditExpectation::point_delete(handle)
                }
                PendingDeleteExpectation::Handleless => {
                    DxfEntityEditExpectation::handleless_point_delete(expected_entity_count)
                }
            });
        }
        let transaction =
            self.document
                .compose_transaction_plans(&plans, self.profile, self.cancellation)?;
        Ok((transaction, expectations))
    }

    fn finish_insert_parts(
        &mut self,
    ) -> Result<(DxfTransactionPlan, Vec<DxfEntityEditExpectation>), DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        let handle_count =
            u64::try_from(self.pending_inserts.len()).map_err(|_| invalid_internal_data())?;
        let policy = self
            .document
            .handle_allocation_policy_directory(self.cancellation)?;
        let reservation = match self.document.plan_handle_reservation(
            &policy,
            handle_count,
            self.profile,
            self.cancellation,
        )? {
            DxfHandleReservationPlanOutcome::Planned(plan) => plan,
            DxfHandleReservationPlanOutcome::PolicyUnavailable { .. }
            | DxfHandleReservationPlanOutcome::Exhausted { .. } => {
                return Err(invalid_internal_data());
            }
        };
        for (index, insert) in self.pending_inserts.iter().enumerate() {
            let index = u64::try_from(index).map_err(|_| invalid_internal_data())?;
            if reservation.allocation().handle_at(index) != Some(insert.handle) {
                return Err(invalid_internal_data());
            }
        }

        self.pending_inserts.sort_unstable_by_key(|insert| {
            (
                insert.placement.insertion_span().start(),
                insert.placement.insertion_span().end(),
                insert.handle.value(),
            )
        });
        let mut builder = self.document.transaction_plan_builder(self.profile)?;
        let mut cursor = 0_usize;
        while cursor < self.pending_inserts.len() {
            ensure_not_cancelled(self.cancellation)?;
            let first = self
                .pending_inserts
                .get(cursor)
                .ok_or_else(invalid_internal_data)?;
            let span = first.placement.insertion_span();
            let mut end = cursor + 1;
            while self
                .pending_inserts
                .get(end)
                .is_some_and(|insert| insert.placement.insertion_span() == span)
            {
                end += 1;
            }
            let mut fragments = Vec::new();
            fragments
                .try_reserve_exact(end - cursor)
                .map_err(|_| out_of_memory())?;
            for insert in &self.pending_inserts[cursor..end] {
                fragments.push(insert.bytes.as_ref());
            }
            builder.replace_raw_span_fragments(span, &fragments, self.cancellation)?;
            cursor = end;
        }
        let insertion = builder.finish(self.cancellation)?;
        let transaction = self.document.compose_transaction_plans(
            &[reservation.transaction(), &insertion],
            self.profile,
            self.cancellation,
        )?;
        let mut expectations = Vec::new();
        expectations
            .try_reserve_exact(self.pending_inserts.len())
            .map_err(|_| out_of_memory())?;
        for insert in self.pending_inserts.drain(..) {
            let owner = (insert.version >= DxfAcadVersion::Ac1012).then_some(insert.owner);
            expectations.push(DxfEntityEditExpectation::point_insert(
                insert.handle,
                owner,
                insert.placement.target(),
                insert.expectation,
            ));
        }
        Ok((transaction, expectations))
    }

    fn finish_parts(
        &mut self,
    ) -> Result<(DxfTransactionPlan, Vec<PendingEdit>, Vec<PendingPointEdit>), DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        self.pending.sort_unstable_by_key(|edit| {
            (
                edit.source_span.start(),
                edit.source_span.end(),
                edit.key.raw_record_ordinal(),
                edit.write_order,
            )
        });
        let mut builder = self.document.transaction_plan_builder(self.profile)?;
        let mut cursor = 0_usize;
        while cursor < self.pending.len() {
            ensure_not_cancelled(self.cancellation)?;
            let edit = self.pending.get(cursor).ok_or_else(invalid_internal_data)?;
            if !edit.source_span.is_empty() {
                builder.replace_raw_span(edit.source_span, &edit.replacement, self.cancellation)?;
                cursor += 1;
                continue;
            }
            let end = insertion_group_end(&self.pending, cursor)?;
            if end == cursor + 1 {
                builder.replace_raw_span(edit.source_span, &edit.replacement, self.cancellation)?;
            } else {
                let combined = combine_insertions(&self.pending[cursor..end])?;
                builder.replace_raw_span(edit.source_span, &combined, self.cancellation)?;
            }
            cursor = end;
        }
        let common = builder.finish(self.cancellation)?;
        let transaction = if self.pending_point_edits.is_empty() {
            common
        } else {
            let mut plans = Vec::new();
            plans
                .try_reserve_exact(self.pending_point_edits.len().saturating_add(1))
                .map_err(|_| out_of_memory())?;
            if !common.patches().is_empty() {
                plans.push(&common);
            }
            for edit in &self.pending_point_edits {
                plans.push(&edit.transaction);
            }
            self.document
                .compose_transaction_plans(&plans, self.profile, self.cancellation)?
        };
        Ok((
            transaction,
            std::mem::take(&mut self.pending),
            std::mem::take(&mut self.pending_point_edits),
        ))
    }

    fn post_session_raw_record_ordinal(&self, key: DxfEntityKey) -> Result<u64, DxfError> {
        ensure_source(self.document.source_id(), key.source_id())?;
        let entity = self
            .evidence
            .entity_directory()
            .entity_for_raw_ordinal(key.raw_record_ordinal())
            .ok_or_else(invalid_internal_data)?;
        let marker_start = entity.marker().full_span().start();
        let shift = self
            .pending_inserts
            .iter()
            .try_fold(0_u64, |count, insert| {
                if insert.placement.insertion_span().start() <= marker_start {
                    count.checked_add(1).ok_or_else(invalid_internal_data)
                } else {
                    Ok(count)
                }
            })?;
        let deleted_before = self
            .pending_deletes
            .iter()
            .try_fold(0_u64, |count, delete| {
                if delete.key.raw_record_ordinal() < key.raw_record_ordinal() {
                    count.checked_add(1).ok_or_else(invalid_internal_data)
                } else {
                    Ok(count)
                }
            })?;
        key.raw_record_ordinal()
            .checked_add(shift)
            .and_then(|ordinal| ordinal.checked_sub(deleted_before))
            .ok_or_else(invalid_internal_data)
    }

    fn set_explicit(
        &mut self,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        match self.document.plan_entity_field_replacement(
            self.evidence,
            key,
            field,
            value,
            self.profile,
            self.cancellation,
        )? {
            DxfEntityFieldReplacementOutcome::Planned(plan) => self.queue_transaction(
                key,
                field,
                DxfEntityEditDisposition::Replaced,
                plan.into_transaction(),
                DxfEntityExpectedField::explicit(value)?,
            ),
            DxfEntityFieldReplacementOutcome::Unavailable(
                DxfEntityFieldReplacementIssue::FieldAbsent { .. },
            ) => match self.document.plan_entity_field_insertion(
                self.evidence,
                key,
                field,
                value,
                self.profile,
                self.cancellation,
            )? {
                DxfEntityFieldInsertionOutcome::Planned(plan) => self.queue_transaction(
                    key,
                    field,
                    DxfEntityEditDisposition::Inserted,
                    plan.into_transaction(),
                    DxfEntityExpectedField::explicit(value)?,
                ),
                DxfEntityFieldInsertionOutcome::Unavailable(issue) => Ok(
                    DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Insertion(issue)),
                ),
            },
            DxfEntityFieldReplacementOutcome::Unavailable(issue) => Ok(
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Replacement(issue)),
            ),
        }
    }

    fn reset(
        &mut self,
        key: DxfEntityKey,
        field: DxfEntityField,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        match self.document.plan_entity_field_reset_to_default(
            self.evidence,
            key,
            field,
            self.profile,
            self.cancellation,
        )? {
            DxfEntityFieldResetOutcome::AlreadyImplicit { .. } => applied(
                key,
                field,
                DxfEntityEditDisposition::AlreadyImplicit,
                self.queued_len()?,
            ),
            DxfEntityFieldResetOutcome::Planned(plan) => self.queue_transaction(
                key,
                field,
                DxfEntityEditDisposition::Reset,
                plan.into_transaction(),
                DxfEntityExpectedField::Implicit,
            ),
            DxfEntityFieldResetOutcome::Unavailable(issue) => Ok(
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Reset(issue)),
            ),
        }
    }

    fn queue_transaction(
        &mut self,
        key: DxfEntityKey,
        field: DxfEntityField,
        disposition: DxfEntityEditDisposition,
        transaction: DxfTransactionPlan,
        expected: DxfEntityExpectedField,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        transaction.validate_source_precondition(self.document)?;
        let [patch] = transaction.patches() else {
            return Err(invalid_internal_data());
        };
        let replacement = transaction
            .replacement_bytes_for_patch_ordinal(patch.ordinal())
            .ok_or_else(invalid_internal_data)?;
        let next_count = self
            .queued_len()?
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        enforce_edit_limit(self.profile, next_count)?;
        let descriptor = field.descriptor().ok_or_else(invalid_internal_data)?;
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(replacement.len())
            .map_err(|_| out_of_memory())?;
        owned.extend_from_slice(replacement);
        self.pending.try_reserve(1).map_err(|_| out_of_memory())?;
        ensure_not_cancelled(self.cancellation)?;
        self.pending.push(PendingEdit {
            key,
            field,
            write_order: descriptor.write_order().ordinal(),
            source_span: patch.source_span(),
            replacement: owned.into_boxed_slice(),
            expected,
        });
        applied(key, field, disposition, self.queued_len()?)
    }

    fn queued_update_len(&self) -> Result<usize, DxfError> {
        self.pending
            .len()
            .checked_add(self.pending_point_edits.len())
            .ok_or_else(invalid_internal_data)
    }

    fn queued_len(&self) -> Result<usize, DxfError> {
        self.queued_update_len()?
            .checked_add(self.pending_inserts.len())
            .and_then(|count| count.checked_add(self.pending_deletes.len()))
            .ok_or_else(invalid_internal_data)
    }
}

impl<'document> DxfRawDocumentView<'document> {
    pub fn entity_edit_session<'evidence, 'cancellation>(
        self,
        evidence: &'evidence DxfEntityFieldEvidenceDirectory,
        profile: DxfResourceProfile,
        cancellation: &'cancellation DxfCancellationToken,
    ) -> Result<DxfEntityEditSession<'document, 'evidence, 'cancellation>, DxfError> {
        DxfEntityEditSession::new(self, evidence, profile, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_edit_session<'document, 'evidence, 'cancellation>(
        &'document self,
        evidence: &'evidence DxfEntityFieldEvidenceDirectory,
        profile: DxfResourceProfile,
        cancellation: &'cancellation DxfCancellationToken,
    ) -> Result<DxfEntityEditSession<'document, 'evidence, 'cancellation>, DxfError> {
        DxfRawDocumentView::from(self).entity_edit_session(evidence, profile, cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_edit_session<'document, 'evidence, 'cancellation>(
        &'document self,
        evidence: &'evidence DxfEntityFieldEvidenceDirectory,
        profile: DxfResourceProfile,
        cancellation: &'cancellation DxfCancellationToken,
    ) -> Result<DxfEntityEditSession<'document, 'evidence, 'cancellation>, DxfError> {
        DxfRawDocumentView::from(self).entity_edit_session(evidence, profile, cancellation)
    }
}

fn insertion_group_end(pending: &[PendingEdit], start: usize) -> Result<usize, DxfError> {
    let first = pending.get(start).ok_or_else(invalid_internal_data)?;
    let mut end = start + 1;
    while let Some(next) = pending.get(end) {
        if !next.source_span.is_empty()
            || next.source_span.start() != first.source_span.start()
            || next.key != first.key
        {
            break;
        }
        end += 1;
    }
    Ok(end)
}

fn combine_insertions(pending: &[PendingEdit]) -> Result<Vec<u8>, DxfError> {
    let capacity = pending.iter().try_fold(0_usize, |total, edit| {
        total
            .checked_add(edit.replacement.len())
            .ok_or_else(invalid_internal_data)
    })?;
    let mut combined = Vec::new();
    combined
        .try_reserve_exact(capacity)
        .map_err(|_| out_of_memory())?;
    for edit in pending {
        combined.extend_from_slice(&edit.replacement);
    }
    Ok(combined)
}

fn prepare_point_clone_draft(
    document: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    placement: DxfEntityPlacementTarget,
    owner: DxfHandle,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<OwnedPointCloneDraft, DxfEntityCloneIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    ensure_source(document.source_id(), evidence.source_id())?;
    let Some(entity) = evidence.entity_directory().entity_for_key(key)? else {
        return Ok(Err(DxfEntityCloneIssue::EntityMissing { key }));
    };
    if entity.classification() != DxfEntityClassification::Canonical(crate::DxfEntityTopic::POINT) {
        return Ok(Err(DxfEntityCloneIssue::WrongClassification {
            key,
            observed: entity.classification(),
        }));
    }
    let DxfAcadVersionState::Supported(version) = document.acad_version_report().state() else {
        return Ok(Err(DxfEntityCloneIssue::VersionUnavailable {
            state: document.acad_version_report().state(),
        }));
    };
    let expected_placement = match source_clone_placement(document, entity, cancellation)? {
        Some(expected) => expected,
        None => return Ok(Err(DxfEntityCloneIssue::SourcePlacementUnavailable { key })),
    };
    if placement != expected_placement {
        return Ok(Err(DxfEntityCloneIssue::PlacementMismatch {
            key,
            expected: expected_placement,
            requested: placement,
        }));
    }
    for occurrence in entity.record().group_range().start()..entity.record().group_range().end() {
        ensure_not_cancelled(cancellation)?;
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        let group_code = group.group_code().value();
        if !matches!(
            group_code,
            0 | 5
                | 6
                | 8
                | 10
                | 20
                | 30
                | 39
                | 48
                | 50
                | 60
                | 62
                | 67
                | 92
                | 100
                | 210
                | 220
                | 230
                | 284
                | 310
                | 330
                | 347
                | 370
                | 390
                | 410
                | 420
                | 430
                | 440
        ) {
            return Ok(Err(DxfEntityCloneIssue::UnsupportedSourceGroup {
                key,
                group_occurrence: occurrence,
                group_code,
            }));
        }
        if let Some(field) = DxfEntityField::from_group_code(group_code)
            && !evidence
                .occurrence_for_group(occurrence)
                .is_some_and(|entry| entry.entity() == entity && entry.field() == field)
        {
            return Ok(Err(DxfEntityCloneIssue::UnsupportedSourceGroup {
                key,
                group_occurrence: occurrence,
                group_code,
            }));
        }
    }

    let common = document.entity_field_semantic_directory(cancellation)?;
    if version >= DxfAcadVersion::Ac1012 {
        let owner_entry = common
            .entry_for_field(entity, DxfEntityField::OWNER)?
            .ok_or_else(invalid_internal_data)?;
        let DxfEntityFieldSemantics::Singleton(value) = owner_entry.semantics() else {
            return Err(invalid_internal_data());
        };
        let expected = match value.value().copied() {
            Some(DxfEntityFieldValue::Handle(expected))
                if value.state() == DxfSemanticValueState::Explicit =>
            {
                expected
            }
            _ => {
                return Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                    key,
                    field: DxfEntityField::OWNER,
                    state: owner_entry.card().state(),
                }));
            }
        };
        if owner != expected {
            return Ok(Err(DxfEntityCloneIssue::OwnerMismatch {
                key,
                expected,
                requested: owner,
            }));
        }
    }
    let layer = match clone_exact_text_field(
        document,
        &common,
        entity,
        key,
        DxfEntityField::LAYER,
        profile,
        false,
    )? {
        Ok(Some(layer)) => layer,
        Ok(None) => {
            return Ok(Err(common_clone_issue(
                key,
                DxfEntityField::LAYER,
                &common,
                entity,
            )?));
        }
        Err(issue) => return Ok(Err(issue)),
    };
    let layout = match clone_exact_text_field(
        document,
        &common,
        entity,
        key,
        DxfEntityField::LAYOUT,
        profile,
        version < DxfAcadVersion::Ac1015
            || entity.record().section_kind() == crate::DxfRawRecordSectionKind::Blocks,
    )? {
        Ok(layout) => layout,
        Err(issue) => return Ok(Err(issue)),
    };
    let linetype = match clone_exact_text_field(
        document,
        &common,
        entity,
        key,
        DxfEntityField::LINETYPE,
        profile,
        true,
    )? {
        Ok(linetype) => linetype,
        Err(issue) => return Ok(Err(issue)),
    };
    let material =
        match clone_common_scalar(&common, entity, key, DxfEntityField::MATERIAL, |value| {
            match value {
                DxfEntityFieldValue::Handle(handle) if !handle.is_null() => Some(handle),
                _ => None,
            }
        })? {
            Ok(value) => value,
            Err(issue) => return Ok(Err(issue)),
        };
    let plot_style =
        match clone_common_scalar(&common, entity, key, DxfEntityField::PLOT_STYLE, |value| {
            match value {
                DxfEntityFieldValue::Handle(handle) if !handle.is_null() => Some(handle),
                _ => None,
            }
        })? {
            Ok(value) => value,
            Err(issue) => return Ok(Err(issue)),
        };
    let lineweight_entry = common
        .entry_for_field(entity, DxfEntityField::LINEWEIGHT)?
        .ok_or_else(invalid_internal_data)?;
    let lineweight = match lineweight_entry.card().state() {
        DxfEntityFieldCardState::AbsentOptional => None,
        DxfEntityFieldCardState::AbsentRequired if version < DxfAcadVersion::Ac1015 => None,
        DxfEntityFieldCardState::Unique => {
            let DxfEntityFieldSemantics::Singleton(value) = lineweight_entry.semantics() else {
                return Err(invalid_internal_data());
            };
            match value.value().copied() {
                Some(DxfEntityFieldValue::Int16(raw))
                    if value.state() == DxfSemanticValueState::Explicit =>
                {
                    match DxfEntityLineweight::from_raw(raw) {
                        Some(lineweight) => Some(lineweight),
                        None => {
                            return Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                                key,
                                field: DxfEntityField::LINEWEIGHT,
                                state: lineweight_entry.card().state(),
                            }));
                        }
                    }
                }
                _ => {
                    return Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                        key,
                        field: DxfEntityField::LINEWEIGHT,
                        state: lineweight_entry.card().state(),
                    }));
                }
            }
        }
        state => {
            return Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                key,
                field: DxfEntityField::LINEWEIGHT,
                state,
            }));
        }
    };
    let space =
        match clone_common_scalar(&common, entity, key, DxfEntityField::PAPER_SPACE, |value| {
            match value {
                DxfEntityFieldValue::Int16(raw) => DxfEntitySpace::from_raw(raw),
                _ => None,
            }
        })? {
            Ok(value) => value,
            Err(issue) => return Ok(Err(issue)),
        };
    let indexed_color = match clone_common_scalar(
        &common,
        entity,
        key,
        DxfEntityField::COLOR,
        |value| match value {
            DxfEntityFieldValue::Int16(raw) => DxfEntityIndexedColor::from_raw(raw),
            _ => None,
        },
    )? {
        Ok(value) => value,
        Err(issue) => return Ok(Err(issue)),
    };
    let linetype_scale = match clone_common_scalar(
        &common,
        entity,
        key,
        DxfEntityField::LINETYPE_SCALE,
        |value| match value {
            DxfEntityFieldValue::Double(raw) if raw.is_finite() && raw.to_f64() >= 0.0 => Some(raw),
            _ => None,
        },
    )? {
        Ok(value) => value,
        Err(issue) => return Ok(Err(issue)),
    };
    let visibility =
        match clone_common_scalar(&common, entity, key, DxfEntityField::VISIBILITY, |value| {
            match value {
                DxfEntityFieldValue::Int16(raw) => DxfEntityVisibility::from_raw(raw),
                _ => None,
            }
        })? {
            Ok(value) => value,
            Err(issue) => return Ok(Err(issue)),
        };
    let proxy_directory = document.entity_proxy_graphics_directory(cancellation)?;
    let proxy_entry = proxy_directory
        .entry_for_entity(entity)?
        .ok_or_else(invalid_internal_data)?;
    let proxy_graphics = match proxy_entry.state() {
        DxfEntityProxyGraphicsState::Absent => None,
        DxfEntityProxyGraphicsState::Matched { .. } => Some(
            crate::entity_proxy_graphics_relation::copy_matched_proxy_graphics_payload(
                document,
                &proxy_directory,
                proxy_entry,
                profile,
                cancellation,
            )?,
        ),
        state => {
            return Ok(Err(DxfEntityCloneIssue::ProxyGraphicsUnavailable {
                key,
                state,
            }));
        }
    };
    let true_color =
        match clone_common_scalar(&common, entity, key, DxfEntityField::TRUE_COLOR, |value| {
            match value {
                DxfEntityFieldValue::Int32(raw) => DxfEntityTrueColor::from_raw(raw),
                _ => None,
            }
        })? {
            Ok(value) => value,
            Err(issue) => return Ok(Err(issue)),
        };
    let color_name = match clone_exact_text_field(
        document,
        &common,
        entity,
        key,
        DxfEntityField::COLOR_NAME,
        profile,
        true,
    )? {
        Ok(Some(color_name)) => color_name,
        Ok(None) => Box::default(),
        Err(issue) => return Ok(Err(issue)),
    };
    let transparency = match clone_common_scalar(
        &common,
        entity,
        key,
        DxfEntityField::TRANSPARENCY,
        |value| match value {
            DxfEntityFieldValue::Int32(raw) => DxfEntityTransparency::from_raw(raw),
            _ => None,
        },
    )? {
        Ok(value) => value,
        Err(issue) => return Ok(Err(issue)),
    };
    let shadow_mode = match clone_common_scalar(
        &common,
        entity,
        key,
        DxfEntityField::SHADOW,
        |value| match value {
            DxfEntityFieldValue::Int16(raw) => DxfEntityShadowMode::from_raw(raw),
            _ => None,
        },
    )? {
        Ok(value) => value,
        Err(issue) => return Ok(Err(issue)),
    };

    let geometry = document.basic_geometry_semantic_directory(cancellation)?;
    let Some(point) = geometry.point_for_raw_record(key.raw_record_ordinal())? else {
        return Ok(Err(DxfEntityCloneIssue::PointSemanticsUnavailable { key }));
    };
    let location = match point.location_value() {
        Some(location)
            if point
                .location()
                .iter()
                .all(|value| value.state() == DxfSemanticValueState::Explicit) =>
        {
            location
        }
        _ => {
            return Ok(Err(DxfEntityCloneIssue::PointFieldUnavailable {
                key,
                kind: DxfPointPatchKind::Location,
            }));
        }
    };
    let thickness = match point.thickness().state() {
        DxfSemanticValueState::Explicit => point.thickness_value(),
        DxfSemanticValueState::Defaulted => None,
        DxfSemanticValueState::Absent | DxfSemanticValueState::Invalid => {
            return Ok(Err(DxfEntityCloneIssue::PointFieldUnavailable {
                key,
                kind: DxfPointPatchKind::Thickness,
            }));
        }
    };
    if point.thickness().state() == DxfSemanticValueState::Explicit && thickness.is_none() {
        return Err(invalid_internal_data());
    }
    let extrusion = if point
        .extrusion()
        .iter()
        .all(|value| value.state() == DxfSemanticValueState::Explicit)
    {
        match point.extrusion_value() {
            Some(extrusion) => Some(extrusion),
            None => return Err(invalid_internal_data()),
        }
    } else if point
        .extrusion()
        .iter()
        .all(|value| value.state() == DxfSemanticValueState::Defaulted)
    {
        None
    } else {
        return Ok(Err(DxfEntityCloneIssue::PointFieldUnavailable {
            key,
            kind: DxfPointPatchKind::Extrusion,
        }));
    };
    let ucs_x_axis_angle = match point.ucs_x_axis_angle().state() {
        DxfSemanticValueState::Explicit => match point.ucs_x_axis_angle_value() {
            Some(angle) => Some(angle),
            None => return Err(invalid_internal_data()),
        },
        DxfSemanticValueState::Defaulted => None,
        DxfSemanticValueState::Absent | DxfSemanticValueState::Invalid => {
            return Ok(Err(DxfEntityCloneIssue::PointFieldUnavailable {
                key,
                kind: DxfPointPatchKind::UcsXAxisAngle,
            }));
        }
    };
    Ok(Ok(OwnedPointCloneDraft {
        layer,
        layout,
        linetype,
        material,
        plot_style,
        space,
        indexed_color,
        lineweight,
        linetype_scale,
        visibility,
        proxy_graphics,
        true_color,
        color_name,
        transparency,
        shadow_mode,
        location,
        thickness,
        extrusion,
        ucs_x_axis_angle,
    }))
}

fn clone_common_scalar<T>(
    common: &crate::DxfEntityFieldSemanticDirectory,
    entity: crate::DxfEntityRef,
    key: DxfEntityKey,
    field: DxfEntityField,
    classify: impl FnOnce(DxfEntityFieldValue) -> Option<T>,
) -> Result<Result<Option<T>, DxfEntityCloneIssue>, DxfError> {
    let entry = common
        .entry_for_field(entity, field)?
        .ok_or_else(invalid_internal_data)?;
    match entry.card().state() {
        DxfEntityFieldCardState::AbsentOptional | DxfEntityFieldCardState::AbsentRequired => {
            Ok(Ok(None))
        }
        DxfEntityFieldCardState::Unique => {
            let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
                return Err(invalid_internal_data());
            };
            match value.value().cloned() {
                Some(raw) if value.state() == DxfSemanticValueState::Explicit => {
                    match classify(raw) {
                        Some(typed) => Ok(Ok(Some(typed))),
                        None => Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                            key,
                            field,
                            state: entry.card().state(),
                        })),
                    }
                }
                _ => Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                    key,
                    field,
                    state: entry.card().state(),
                })),
            }
        }
        state => Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
            key,
            field,
            state,
        })),
    }
}

fn source_clone_placement(
    document: DxfRawDocumentView<'_>,
    entity: crate::DxfEntityRef,
    cancellation: &DxfCancellationToken,
) -> Result<Option<DxfEntityPlacementTarget>, DxfError> {
    Ok(match entity.record().section_kind() {
        crate::DxfRawRecordSectionKind::Entities => {
            Some(DxfEntityPlacementTarget::EntitiesSection {
                structure_section_ordinal: entity.record().structure_section_ordinal(),
            })
        }
        crate::DxfRawRecordSectionKind::Blocks => {
            let blocks = document.block_definition_directory(cancellation)?;
            let mut target = None;
            for definition in blocks.definitions().iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let block_ordinal = definition.block_record().ordinal();
                if blocks
                    .members_for_block_raw_ordinal(block_ordinal)
                    .is_some_and(|members| {
                        members
                            .iter()
                            .any(|member| member.ordinal() == entity.record().ordinal())
                    })
                {
                    target = Some(DxfEntityPlacementTarget::BlockDefinition {
                        raw_record_ordinal: block_ordinal,
                    });
                    break;
                }
            }
            target
        }
        crate::DxfRawRecordSectionKind::Classes
        | crate::DxfRawRecordSectionKind::Tables
        | crate::DxfRawRecordSectionKind::Objects => None,
    })
}

fn clone_exact_text_field(
    document: DxfRawDocumentView<'_>,
    common: &crate::DxfEntityFieldSemanticDirectory,
    entity: crate::DxfEntityRef,
    key: DxfEntityKey,
    field: DxfEntityField,
    profile: DxfResourceProfile,
    allow_absent_required: bool,
) -> Result<PointCloneTextResult, DxfError> {
    let entry = common
        .entry_for_field(entity, field)?
        .ok_or_else(invalid_internal_data)?;
    match entry.card().state() {
        DxfEntityFieldCardState::AbsentOptional => Ok(Ok(None)),
        DxfEntityFieldCardState::AbsentRequired if allow_absent_required => Ok(Ok(None)),
        DxfEntityFieldCardState::Unique => {
            let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
                return Err(invalid_internal_data());
            };
            let Some(DxfEntityFieldValue::ExactText(text)) = value.value().copied() else {
                return Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                    key,
                    field,
                    state: entry.card().state(),
                }));
            };
            if value.state() != DxfSemanticValueState::Explicit {
                return Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
                    key,
                    field,
                    state: entry.card().state(),
                }));
            }
            let len =
                usize::try_from(text.value_span().len()).map_err(|_| invalid_internal_data())?;
            let observed = u64::try_from(len).map_err(|_| invalid_internal_data())?;
            let limit = profile.limits().max_value_bytes();
            if observed > limit {
                return Err(DxfError::resource_limit(
                    DxfResource::ValueBytes,
                    limit,
                    observed,
                ));
            }
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(len).map_err(|_| out_of_memory())?;
            bytes.resize(len, 0);
            document.read_span(text.value_span(), &mut bytes)?;
            Ok(Ok(Some(bytes.into_boxed_slice())))
        }
        state => Ok(Err(DxfEntityCloneIssue::CommonFieldUnavailable {
            key,
            field,
            state,
        })),
    }
}

fn common_clone_issue(
    key: DxfEntityKey,
    field: DxfEntityField,
    common: &crate::DxfEntityFieldSemanticDirectory,
    entity: crate::DxfEntityRef,
) -> Result<DxfEntityCloneIssue, DxfError> {
    let entry = common
        .entry_for_field(entity, field)?
        .ok_or_else(invalid_internal_data)?;
    Ok(DxfEntityCloneIssue::CommonFieldUnavailable {
        key,
        field,
        state: entry.card().state(),
    })
}

fn compact_owner_issue(issue: DxfEntityPlacementOwnerIssue) -> DxfEntityInsertOwnerIssue {
    match issue {
        DxfEntityPlacementOwnerIssue::PlacementUnavailable { target } => {
            DxfEntityInsertOwnerIssue::PlacementUnavailable { target }
        }
        DxfEntityPlacementOwnerIssue::NullOwner => DxfEntityInsertOwnerIssue::NullOwner,
        DxfEntityPlacementOwnerIssue::OwnerMissing { handle } => {
            DxfEntityInsertOwnerIssue::OwnerMissing { handle }
        }
        DxfEntityPlacementOwnerIssue::OwnerAmbiguous {
            handle,
            target_count,
        } => DxfEntityInsertOwnerIssue::OwnerAmbiguous {
            handle,
            target_count,
        },
        DxfEntityPlacementOwnerIssue::OwnerNotBlockRecord { target } => {
            DxfEntityInsertOwnerIssue::OwnerNotBlockRecord {
                handle: target.handle(),
                raw_record_ordinal: target.record().ordinal(),
            }
        }
        DxfEntityPlacementOwnerIssue::BlockOwnerCardinality { state } => {
            DxfEntityInsertOwnerIssue::BlockOwnerCardinality { state }
        }
        DxfEntityPlacementOwnerIssue::BlockOwnerResolution { state } => {
            DxfEntityInsertOwnerIssue::BlockOwnerResolution { state }
        }
        DxfEntityPlacementOwnerIssue::BlockOwnerMismatch {
            declared,
            requested,
        } => DxfEntityInsertOwnerIssue::BlockOwnerMismatch {
            declared: declared.handle(),
            requested: requested.handle(),
        },
    }
}

fn applied(
    key: DxfEntityKey,
    field: DxfEntityField,
    disposition: DxfEntityEditDisposition,
    queued_edit_count: usize,
) -> Result<DxfEntityEditOutcome, DxfError> {
    Ok(DxfEntityEditOutcome::Applied(DxfEntityEditReceipt {
        key,
        field,
        disposition,
        queued_edit_count: u32::try_from(queued_edit_count).map_err(|_| invalid_internal_data())?,
    }))
}

fn enforce_edit_limit(profile: DxfResourceProfile, observed: usize) -> Result<(), DxfError> {
    let observed = u64::try_from(observed).map_err(|_| invalid_internal_data())?;
    let limit = profile.limits().max_records();
    if observed > limit {
        Err(DxfError::resource_limit(
            DxfResource::Records,
            limit,
            observed,
        ))
    } else {
        Ok(())
    }
}

fn compose_session_transactions(
    document: DxfRawDocumentView<'_>,
    updates: &DxfTransactionPlan,
    inserts: &DxfTransactionPlan,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<DxfTransactionPlan, DxfError> {
    updates.validate_source_precondition(document)?;
    inserts.validate_source_precondition(document)?;
    let patch_count = updates
        .patches()
        .len()
        .checked_add(inserts.patches().len())
        .ok_or_else(invalid_internal_data)?;
    let mut patches = Vec::new();
    patches
        .try_reserve_exact(patch_count)
        .map_err(|_| out_of_memory())?;
    for (order, transaction) in [(0_u8, updates), (1_u8, inserts)] {
        for patch in transaction.patches().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let replacement = transaction
                .replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or_else(invalid_internal_data)?;
            patches.push((patch.source_span(), order, replacement));
        }
    }
    patches.sort_unstable_by_key(|(span, order, _)| (span.start(), span.end(), *order));

    let mut builder = document.transaction_plan_builder(profile)?;
    let mut cursor = 0_usize;
    while cursor < patches.len() {
        ensure_not_cancelled(cancellation)?;
        let (span, _, replacement) = patches
            .get(cursor)
            .copied()
            .ok_or_else(invalid_internal_data)?;
        if !span.is_empty() {
            builder.replace_raw_span(span, replacement, cancellation)?;
            cursor += 1;
            continue;
        }
        let mut end = cursor + 1;
        while patches
            .get(end)
            .is_some_and(|(candidate, _, _)| *candidate == span)
        {
            end += 1;
        }
        if end == cursor + 1 {
            builder.replace_raw_span(span, replacement, cancellation)?;
        } else {
            let mut fragments = Vec::new();
            fragments
                .try_reserve_exact(end - cursor)
                .map_err(|_| out_of_memory())?;
            for (_, _, fragment) in &patches[cursor..end] {
                fragments.push(*fragment);
            }
            builder.replace_raw_span_fragments(span, &fragments, cancellation)?;
        }
        cursor = end;
    }
    builder.finish(cancellation)
}

fn enforce_exact_text_limit(
    profile: DxfResourceProfile,
    value: DxfEntityEditValue<'_>,
) -> Result<(), DxfError> {
    let DxfEntityEditValue::ExactRawText(bytes) = value else {
        return Ok(());
    };
    let observed = u64::try_from(bytes.len()).map_err(|_| invalid_internal_data())?;
    let limit = profile.limits().max_value_bytes();
    if observed > limit {
        Err(DxfError::resource_limit(
            DxfResource::ValueBytes,
            limit,
            observed,
        ))
    } else {
        Ok(())
    }
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

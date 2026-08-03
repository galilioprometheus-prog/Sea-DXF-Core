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
    DxfCancellationToken, DxfCommonOwnerCandidateState, DxfEntityCommonColorBookEditIssue,
    DxfEntityCommonColorBookEditOutcome, DxfEntityCommonFieldDomainDirectory,
    DxfEntityCommonFieldDomainIssue, DxfEntityCommonFieldDomainOutcome,
    DxfEntityCommonLayoutEditIssue, DxfEntityCommonLayoutEditOutcome,
    DxfEntityCommonReferenceEditIssue, DxfEntityCommonReferenceEditOutcome,
    DxfEntityCommonSymbolEditIssue, DxfEntityCommonSymbolEditOutcome, DxfEntityDraft,
    DxfEntityDraftApplicabilityIssue, DxfEntityDraftName, DxfEntityDraftRecordIssue,
    DxfEntityEditPlan, DxfEntityEditValue, DxfEntityField, DxfEntityFieldEvidenceDirectory,
    DxfEntityFieldInsertionIssue, DxfEntityFieldInsertionOutcome, DxfEntityFieldReplacementIssue,
    DxfEntityFieldReplacementOutcome, DxfEntityFieldResetIssue, DxfEntityFieldResetOutcome,
    DxfEntityKey, DxfEntityPlacement, DxfEntityPlacementOwnerIssue, DxfEntityPlacementOwnerOutcome,
    DxfEntityPlacementTarget, DxfError, DxfHandle, DxfHandleAllocationOutcome,
    DxfHandleAllocationPolicyState, DxfHandleIdentityDirectory, DxfHandleReservationPlanOutcome,
    DxfHandleResolutionState, DxfIoOperation, DxfLayoutObjectDirectory,
    DxfNamedSymbolTableDirectory, DxfPointEditIssue, DxfPointPatch, DxfPointPatchKind,
    DxfRawDocumentView, DxfResource, DxfResourceProfile, DxfSourceId, DxfTransactionPlan,
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
    }

    /// Adds one complete typed entity insertion without changing the source.
    ///
    /// Every admitted draft is copied into session-owned encoded evidence.
    /// Update/insert mixing remains fail-closed until ordinal-independent mixed
    /// verification lands.
    pub fn insert(
        &mut self,
        placement: DxfEntityPlacement,
        draft: DxfEntityDraft<'_>,
    ) -> Result<DxfEntityInsertOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        if !self.pending.is_empty() || !self.pending_point_edits.is_empty() {
            return Ok(DxfEntityInsertOutcome::Unavailable(
                DxfEntityInsertIssue::UpdatePending {
                    queued_update_count: u32::try_from(self.queued_update_len()?)
                        .map_err(|_| invalid_internal_data())?,
                },
            ));
        }
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

    /// Adds one typed update without changing the source document.
    pub fn update(
        &mut self,
        key: DxfEntityKey,
        patch: DxfEntityPatch<'_>,
    ) -> Result<DxfEntityEditOutcome, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        if !self.pending_inserts.is_empty() {
            return Ok(DxfEntityEditOutcome::Unavailable(
                DxfEntityEditIssue::InsertPending,
            ));
        }
        match patch {
            DxfEntityPatch::CommonField(patch) => self.update_common_field(key, patch),
            DxfEntityPatch::CommonColorBook(patch) => self.update_color_book(key, patch),
            DxfEntityPatch::ResetCommonColorBook => self.reset_color_book(key),
            DxfEntityPatch::Point(patch) => self.update_point(key, patch),
        }
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
                            queued_edit_count: u32::try_from(self.queued_update_len()?)
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
                            queued_edit_count: u32::try_from(self.queued_update_len()?)
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
                            queued_edit_count: u32::try_from(self.queued_update_len()?)
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
            .queued_update_len()?
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
            self.queued_update_len()?,
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
            self.queued_update_len()?,
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
        if !self.pending_inserts.is_empty() {
            return self
                .finish_insert_batch()
                .map(DxfEntityEditPlan::into_transaction);
        }
        self.finish_parts().map(|(transaction, _, _)| transaction)
    }

    /// Freezes the transaction together with its semantic postconditions.
    pub fn finish_verifiable(self) -> Result<DxfEntityEditPlan, DxfError> {
        if !self.pending_inserts.is_empty() {
            return self.finish_insert_batch();
        }
        let (transaction, pending, point_edits) = self.finish_parts()?;
        let mut expectations = Vec::new();
        expectations
            .try_reserve_exact(
                pending
                    .len()
                    .checked_add(point_edits.len())
                    .ok_or_else(invalid_internal_data)?,
            )
            .map_err(|_| out_of_memory())?;
        for edit in pending {
            expectations.push(DxfEntityEditExpectation::field(
                edit.key.raw_record_ordinal(),
                edit.field,
                edit.expected,
            ));
        }
        for edit in point_edits {
            expectations.push(match edit.expectation {
                PendingPointExpectation::Location(location) => {
                    DxfEntityEditExpectation::point_location(
                        edit.key.raw_record_ordinal(),
                        location,
                    )
                }
                PendingPointExpectation::Thickness(thickness) => {
                    DxfEntityEditExpectation::point_thickness(
                        edit.key.raw_record_ordinal(),
                        thickness,
                    )
                }
                PendingPointExpectation::ThicknessReset => {
                    DxfEntityEditExpectation::point_thickness_reset(edit.key.raw_record_ordinal())
                }
                PendingPointExpectation::Extrusion(extrusion) => {
                    DxfEntityEditExpectation::point_extrusion(
                        edit.key.raw_record_ordinal(),
                        extrusion,
                    )
                }
                PendingPointExpectation::ExtrusionReset => {
                    DxfEntityEditExpectation::point_extrusion_reset(edit.key.raw_record_ordinal())
                }
                PendingPointExpectation::UcsXAxisAngle(angle) => {
                    DxfEntityEditExpectation::point_ucs_x_axis_angle(
                        edit.key.raw_record_ordinal(),
                        angle,
                    )
                }
                PendingPointExpectation::UcsXAxisAngleReset => {
                    DxfEntityEditExpectation::point_ucs_x_axis_angle_reset(
                        edit.key.raw_record_ordinal(),
                    )
                }
            });
        }
        Ok(DxfEntityEditPlan::new(transaction, expectations))
    }

    fn finish_insert_batch(mut self) -> Result<DxfEntityEditPlan, DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        if !self.pending.is_empty() || !self.pending_point_edits.is_empty() {
            return Err(invalid_internal_data());
        }
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
        for insert in self.pending_inserts {
            let owner = (insert.version >= DxfAcadVersion::Ac1012).then_some(insert.owner);
            expectations.push(DxfEntityEditExpectation::point_insert(
                insert.handle,
                owner,
                insert.placement.target(),
                insert.expectation,
            ));
        }
        Ok(DxfEntityEditPlan::new(transaction, expectations))
    }

    fn finish_parts(
        mut self,
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
        Ok((transaction, self.pending, self.pending_point_edits))
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
                self.queued_update_len()?,
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
            .queued_update_len()?
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
        applied(key, field, disposition, self.queued_update_len()?)
    }

    fn queued_update_len(&self) -> Result<usize, DxfError> {
        self.pending
            .len()
            .checked_add(self.pending_point_edits.len())
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

//! Typed family-draft validation and canonical record-byte encoding.

use crate::{
    DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble,
    DxfEntityCommonColorBookEditIssue, DxfEntityCommonLayoutEditIssue,
    DxfEntityCommonLayoutEditOutcome, DxfEntityCommonReferenceEditIssue,
    DxfEntityCommonReferenceEditOutcome, DxfEntityCommonSymbolEditIssue,
    DxfEntityCommonSymbolEditOutcome, DxfEntityDraftApplicabilityPlan, DxfEntityDraftName,
    DxfEntityEditValue, DxfEntityField, DxfEntityFieldWireType, DxfEntityGroupEncodeIssue,
    DxfEntityGroupEncoder, DxfEntityIndexedColor, DxfEntityLineweight, DxfEntityNameClassification,
    DxfEntityPlacementTarget, DxfEntityShadowMode, DxfEntitySpace, DxfEntityTopic,
    DxfEntityTransparency, DxfEntityTrueColor, DxfEntityVisibility, DxfError, DxfHandle,
    DxfIoOperation, DxfRawDocumentView, DxfResource, DxfResourceProfile, DxfSourceId,
    DxfTransactionPlan,
};

const ACDB_ENTITY: &[u8] = b"AcDbEntity";
const ACDB_POINT: &[u8] = b"AcDbPoint";

/// Caller-provided exact-raw fields for a new POINT entity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPointDraft<'a> {
    layer: &'a [u8],
    layout: Option<&'a [u8]>,
    linetype: Option<&'a [u8]>,
    material: Option<DxfHandle>,
    plot_style: Option<DxfHandle>,
    space: Option<DxfEntitySpace>,
    indexed_color: Option<DxfEntityIndexedColor>,
    lineweight: Option<DxfEntityLineweight>,
    linetype_scale: Option<DxfDouble>,
    visibility: Option<DxfEntityVisibility>,
    true_color: Option<DxfEntityTrueColor>,
    color_name: Option<&'a [u8]>,
    transparency: Option<DxfEntityTransparency>,
    shadow_mode: Option<DxfEntityShadowMode>,
    location: [DxfDouble; 3],
    thickness: Option<DxfDouble>,
    extrusion: Option<[DxfDouble; 3]>,
    ucs_x_axis_angle: Option<DxfDouble>,
}

impl<'a> DxfPointDraft<'a> {
    #[must_use]
    pub const fn new(layer: &'a [u8], location: [DxfDouble; 3]) -> Self {
        Self {
            layer,
            layout: None,
            linetype: None,
            material: None,
            plot_style: None,
            space: None,
            indexed_color: None,
            lineweight: None,
            linetype_scale: None,
            visibility: None,
            true_color: None,
            color_name: None,
            transparency: None,
            shadow_mode: None,
            location,
            thickness: None,
            extrusion: None,
            ucs_x_axis_angle: None,
        }
    }

    /// Supplies the exact layout name required for AC1015+ ENTITIES placement.
    #[must_use]
    pub const fn with_layout(mut self, layout: &'a [u8]) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Emits an explicit model/paper-space group 67.
    #[must_use]
    pub const fn with_space(mut self, space: DxfEntitySpace) -> Self {
        self.space = Some(space);
        self
    }

    /// Emits an explicit indexed color group 62.
    #[must_use]
    pub const fn with_indexed_color(mut self, color: DxfEntityIndexedColor) -> Self {
        self.indexed_color = Some(color);
        self
    }

    /// Emits an exact same-document linetype name in group 6.
    #[must_use]
    pub const fn with_linetype(mut self, linetype: &'a [u8]) -> Self {
        self.linetype = Some(linetype);
        self
    }

    /// Emits a validated same-document MATERIAL object reference in group 347.
    #[must_use]
    pub const fn with_material(mut self, material: DxfHandle) -> Self {
        self.material = Some(material);
        self
    }

    /// Emits a validated same-document plot-style object reference in group 390.
    #[must_use]
    pub const fn with_plot_style(mut self, plot_style: DxfHandle) -> Self {
        self.plot_style = Some(plot_style);
        self
    }

    /// Supplies the non-omitted AC1015+ common lineweight.
    #[must_use]
    pub const fn with_lineweight(mut self, lineweight: DxfEntityLineweight) -> Self {
        self.lineweight = Some(lineweight);
        self
    }

    /// Emits an explicit non-negative common linetype scale group 48.
    #[must_use]
    pub const fn with_linetype_scale(mut self, scale: DxfDouble) -> Self {
        self.linetype_scale = Some(scale);
        self
    }

    /// Emits an explicit visibility group 60.
    #[must_use]
    pub const fn with_visibility(mut self, visibility: DxfEntityVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Emits an explicit true-color group 420.
    #[must_use]
    pub const fn with_true_color(mut self, color: DxfEntityTrueColor) -> Self {
        self.true_color = Some(color);
        self
    }

    /// Emits a color-book name in group 430 with explicit groups 62 and 420.
    #[must_use]
    pub const fn with_color_name(mut self, color_name: &'a [u8]) -> Self {
        self.color_name = Some(color_name);
        self
    }

    /// Emits an explicit transparency group 440.
    #[must_use]
    pub const fn with_transparency(mut self, transparency: DxfEntityTransparency) -> Self {
        self.transparency = Some(transparency);
        self
    }

    /// Emits an explicit common shadow-mode group 284.
    #[must_use]
    pub const fn with_shadow_mode(mut self, shadow_mode: DxfEntityShadowMode) -> Self {
        self.shadow_mode = Some(shadow_mode);
        self
    }

    /// Emits an explicit group 39 instead of the documented zero default.
    #[must_use]
    pub const fn with_thickness(mut self, thickness: DxfDouble) -> Self {
        self.thickness = Some(thickness);
        self
    }

    /// Emits the complete explicit group 210/220/230 direction tuple.
    #[must_use]
    pub const fn with_extrusion(mut self, extrusion: [DxfDouble; 3]) -> Self {
        self.extrusion = Some(extrusion);
        self
    }

    /// Emits the optional group 50 UCS X-axis angle in degrees.
    #[must_use]
    pub const fn with_ucs_x_axis_angle(mut self, angle: DxfDouble) -> Self {
        self.ucs_x_axis_angle = Some(angle);
        self
    }

    #[must_use]
    pub const fn layer(self) -> &'a [u8] {
        self.layer
    }

    #[must_use]
    pub const fn layout(self) -> Option<&'a [u8]> {
        self.layout
    }

    #[must_use]
    pub const fn space(self) -> Option<DxfEntitySpace> {
        self.space
    }

    #[must_use]
    pub const fn indexed_color(self) -> Option<DxfEntityIndexedColor> {
        self.indexed_color
    }

    #[must_use]
    pub const fn linetype(self) -> Option<&'a [u8]> {
        self.linetype
    }

    #[must_use]
    pub const fn material(self) -> Option<DxfHandle> {
        self.material
    }

    #[must_use]
    pub const fn plot_style(self) -> Option<DxfHandle> {
        self.plot_style
    }

    #[must_use]
    pub const fn lineweight(self) -> Option<DxfEntityLineweight> {
        self.lineweight
    }

    #[must_use]
    pub const fn linetype_scale(self) -> Option<DxfDouble> {
        self.linetype_scale
    }

    #[must_use]
    pub const fn visibility(self) -> Option<DxfEntityVisibility> {
        self.visibility
    }

    #[must_use]
    pub const fn true_color(self) -> Option<DxfEntityTrueColor> {
        self.true_color
    }

    #[must_use]
    pub const fn color_name(self) -> Option<&'a [u8]> {
        self.color_name
    }

    #[must_use]
    pub const fn transparency(self) -> Option<DxfEntityTransparency> {
        self.transparency
    }

    #[must_use]
    pub const fn shadow_mode(self) -> Option<DxfEntityShadowMode> {
        self.shadow_mode
    }

    #[must_use]
    pub const fn location(self) -> [DxfDouble; 3] {
        self.location
    }

    #[must_use]
    pub const fn thickness(self) -> Option<DxfDouble> {
        self.thickness
    }

    #[must_use]
    pub const fn extrusion(self) -> Option<[DxfDouble; 3]> {
        self.extrusion
    }

    #[must_use]
    pub const fn ucs_x_axis_angle(self) -> Option<DxfDouble> {
        self.ucs_x_axis_angle
    }
}

/// Typed payload for one entity family admitted by the generated registry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDraft<'a> {
    Point {
        draft: DxfPointDraft<'a>,
        owner: Option<DxfHandle>,
    },
}

impl<'a> DxfEntityDraft<'a> {
    #[must_use]
    pub const fn point(draft: DxfPointDraft<'a>) -> Self {
        Self::Point { draft, owner: None }
    }

    /// Supplies the exact BLOCK_RECORD owner selected by the caller.
    #[must_use]
    pub const fn with_owner(self, owner: DxfHandle) -> Self {
        match self {
            Self::Point { draft, .. } => Self::Point {
                draft,
                owner: Some(owner),
            },
        }
    }

    #[must_use]
    pub const fn owner(self) -> Option<DxfHandle> {
        match self {
            Self::Point { owner, .. } => owner,
        }
    }

    #[must_use]
    pub const fn name(self) -> DxfEntityDraftName {
        match self {
            Self::Point { .. } => DxfEntityDraftName::canonical(DxfEntityTopic::POINT),
        }
    }
}

/// Typed reason why an admitted identity cannot encode one family draft.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDraftRecordIssue {
    NameMismatch {
        admitted: DxfEntityNameClassification,
        draft: DxfEntityNameClassification,
    },
    EmptyLayerName,
    OwnerMismatch {
        admitted: DxfHandle,
        requested: DxfHandle,
    },
    LayoutRequired {
        target: DxfEntityPlacementTarget,
    },
    LayoutNotApplicable {
        version: DxfAcadVersion,
        target: DxfEntityPlacementTarget,
    },
    LineweightRequired {
        version: DxfAcadVersion,
    },
    LineweightNotApplicable {
        version: DxfAcadVersion,
    },
    CommonFieldNotApplicable {
        field: DxfEntityField,
        version: DxfAcadVersion,
    },
    InvalidLinetypeScale {
        value: DxfDouble,
    },
    ZeroExtrusion,
    LayerReference(DxfEntityCommonSymbolEditIssue),
    SymbolReference(DxfEntityCommonSymbolEditIssue),
    Reference(DxfEntityCommonReferenceEditIssue),
    ColorBook(DxfEntityCommonColorBookEditIssue),
    ColorBookRelatedFieldRequired {
        field: DxfEntityField,
    },
    LayoutReference(DxfEntityCommonLayoutEditIssue),
    GroupEncode {
        group_code: i16,
        issue: DxfEntityGroupEncodeIssue,
    },
}

/// Canonical encoded record bytes retaining their source-bound preparation.
pub struct DxfEntityDraftRecordPlan {
    applicability: DxfEntityDraftApplicabilityPlan,
    bytes: Box<[u8]>,
    expectation: DxfPointDraftRecordExpectation,
}

pub(crate) struct DxfPointDraftRecordExpectation {
    common: Box<DxfPointDraftCommonExpectation>,
    space: Option<DxfEntitySpace>,
    indexed_color: Option<DxfEntityIndexedColor>,
    lineweight: Option<DxfEntityLineweight>,
    linetype_scale: Option<DxfDouble>,
    visibility: Option<DxfEntityVisibility>,
    true_color: Option<DxfEntityTrueColor>,
    transparency: Option<DxfEntityTransparency>,
    shadow_mode: Option<DxfEntityShadowMode>,
    location: [DxfDouble; 3],
    thickness: Option<DxfDouble>,
    extrusion: Option<[DxfDouble; 3]>,
    ucs_x_axis_angle: Option<DxfDouble>,
}

struct DxfPointDraftCommonExpectation {
    layer: Box<[u8]>,
    layout: Box<[u8]>,
    linetype: Box<[u8]>,
    references: DxfPointDraftReferenceExpectation,
    color_name: Box<[u8]>,
}

#[derive(Clone, Copy)]
struct DxfPointDraftReferenceExpectation {
    handles: [DxfHandle; 2],
    explicit_mask: u8,
}

impl DxfPointDraftReferenceExpectation {
    const MATERIAL_MASK: u8 = 1;
    const PLOT_STYLE_MASK: u8 = 2;

    const fn new(material: Option<DxfHandle>, plot_style: Option<DxfHandle>) -> Self {
        Self {
            handles: [
                match material {
                    Some(handle) => handle,
                    None => DxfHandle::from_u64(0),
                },
                match plot_style {
                    Some(handle) => handle,
                    None => DxfHandle::from_u64(0),
                },
            ],
            explicit_mask: (if material.is_some() {
                Self::MATERIAL_MASK
            } else {
                0
            }) | (if plot_style.is_some() {
                Self::PLOT_STYLE_MASK
            } else {
                0
            }),
        }
    }

    const fn material(self) -> Option<DxfHandle> {
        if self.explicit_mask & Self::MATERIAL_MASK != 0 {
            Some(self.handles[0])
        } else {
            None
        }
    }

    const fn plot_style(self) -> Option<DxfHandle> {
        if self.explicit_mask & Self::PLOT_STYLE_MASK != 0 {
            Some(self.handles[1])
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DxfEntityDraftEncodingContext {
    name: DxfEntityDraftName,
    version: DxfAcadVersion,
    handle: DxfHandle,
    owner: DxfHandle,
    placement: DxfEntityPlacementTarget,
}

impl DxfEntityDraftEncodingContext {
    pub(crate) const fn new(
        name: DxfEntityDraftName,
        version: DxfAcadVersion,
        handle: DxfHandle,
        owner: DxfHandle,
        placement: DxfEntityPlacementTarget,
    ) -> Self {
        Self {
            name,
            version,
            handle,
            owner,
            placement,
        }
    }
}

pub(crate) struct DxfEncodedEntityDraftRecord {
    bytes: Box<[u8]>,
    expectation: DxfPointDraftRecordExpectation,
}

impl DxfEncodedEntityDraftRecord {
    pub(crate) fn into_parts(self) -> (Box<[u8]>, DxfPointDraftRecordExpectation) {
        (self.bytes, self.expectation)
    }
}

impl DxfPointDraftRecordExpectation {
    pub(crate) fn layer(&self) -> &[u8] {
        &self.common.layer
    }

    pub(crate) fn layout(&self) -> Option<&[u8]> {
        (!self.common.layout.is_empty()).then_some(self.common.layout.as_ref())
    }

    pub(crate) const fn space(&self) -> Option<DxfEntitySpace> {
        self.space
    }

    pub(crate) const fn indexed_color(&self) -> Option<DxfEntityIndexedColor> {
        self.indexed_color
    }

    pub(crate) fn linetype(&self) -> Option<&[u8]> {
        (!self.common.linetype.is_empty()).then_some(self.common.linetype.as_ref())
    }

    pub(crate) const fn material(&self) -> Option<DxfHandle> {
        self.common.references.material()
    }

    pub(crate) const fn plot_style(&self) -> Option<DxfHandle> {
        self.common.references.plot_style()
    }

    pub(crate) const fn lineweight(&self) -> Option<DxfEntityLineweight> {
        self.lineweight
    }

    pub(crate) const fn linetype_scale(&self) -> Option<DxfDouble> {
        self.linetype_scale
    }

    pub(crate) const fn visibility(&self) -> Option<DxfEntityVisibility> {
        self.visibility
    }

    pub(crate) const fn true_color(&self) -> Option<DxfEntityTrueColor> {
        self.true_color
    }

    pub(crate) fn color_name(&self) -> Option<&[u8]> {
        (!self.common.color_name.is_empty()).then_some(self.common.color_name.as_ref())
    }

    pub(crate) const fn transparency(&self) -> Option<DxfEntityTransparency> {
        self.transparency
    }

    pub(crate) const fn shadow_mode(&self) -> Option<DxfEntityShadowMode> {
        self.shadow_mode
    }

    pub(crate) const fn location(&self) -> [DxfDouble; 3] {
        self.location
    }

    pub(crate) const fn thickness(&self) -> Option<DxfDouble> {
        self.thickness
    }

    pub(crate) const fn extrusion(&self) -> Option<[DxfDouble; 3]> {
        self.extrusion
    }

    pub(crate) const fn ucs_x_axis_angle(&self) -> Option<DxfDouble> {
        self.ucs_x_axis_angle
    }
}

impl DxfEntityDraftRecordPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.applicability.source_id()
    }

    #[must_use]
    pub const fn name(&self) -> DxfEntityDraftName {
        self.applicability.name()
    }

    #[must_use]
    pub const fn version(&self) -> DxfAcadVersion {
        self.applicability.version()
    }

    #[must_use]
    pub const fn handle(&self) -> DxfHandle {
        self.applicability.handle()
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn transaction(&self) -> &DxfTransactionPlan {
        self.applicability.transaction()
    }

    #[must_use]
    pub const fn applicability(&self) -> &DxfEntityDraftApplicabilityPlan {
        &self.applicability
    }

    #[must_use]
    pub fn into_applicability(self) -> DxfEntityDraftApplicabilityPlan {
        self.applicability
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        DxfEntityDraftApplicabilityPlan,
        Box<[u8]>,
        DxfPointDraftRecordExpectation,
    ) {
        (self.applicability, self.bytes, self.expectation)
    }
}

impl std::fmt::Debug for DxfEntityDraftRecordPlan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DxfEntityDraftRecordPlan")
            .field("source_id", &self.source_id())
            .field("name", &self.name())
            .field("version", &self.version())
            .field("handle", &self.handle())
            .field("byte_count", &self.bytes.len())
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    /// Encodes one reviewed family draft without yet inserting it into a container.
    pub fn encode_entity_draft_record(
        self,
        applicability: DxfEntityDraftApplicabilityPlan,
        draft: DxfEntityDraft<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftRecordPlan, DxfEntityDraftRecordIssue>, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), applicability.source_id())?;
        applicability
            .transaction()
            .validate_source_precondition(self)?;
        let context = DxfEntityDraftEncodingContext::new(
            applicability.name(),
            applicability.version(),
            applicability.handle(),
            applicability.identity().owner_handle(),
            applicability.identity().placement().target(),
        );
        let encoded =
            match encode_entity_draft_record_parts(self, context, draft, profile, cancellation)? {
                Ok(encoded) => encoded,
                Err(issue) => return Ok(Err(issue)),
            };
        let (bytes, expectation) = encoded.into_parts();
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfEntityDraftRecordPlan {
            applicability,
            bytes,
            expectation,
        }))
    }
}

pub(crate) fn encode_entity_draft_record_parts(
    document: DxfRawDocumentView<'_>,
    context: DxfEntityDraftEncodingContext,
    draft: DxfEntityDraft<'_>,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfEncodedEntityDraftRecord, DxfEntityDraftRecordIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    if context.name != draft.name() {
        return Ok(Err(DxfEntityDraftRecordIssue::NameMismatch {
            admitted: context.name.classification(),
            draft: draft.name().classification(),
        }));
    }
    if let Some(requested) = draft.owner()
        && requested != context.owner
    {
        return Ok(Err(DxfEntityDraftRecordIssue::OwnerMismatch {
            admitted: context.owner,
            requested,
        }));
    }
    let (bytes, expectation) = match draft {
        DxfEntityDraft::Point { draft: point, .. } => {
            let bytes = match encode_point(document, context, point, profile, cancellation)? {
                Ok(bytes) => bytes,
                Err(issue) => return Ok(Err(issue)),
            };
            let expectation = point_expectation(point)?;
            (bytes, expectation)
        }
    };
    ensure_not_cancelled(cancellation)?;
    Ok(Ok(DxfEncodedEntityDraftRecord {
        bytes: bytes.into_boxed_slice(),
        expectation,
    }))
}

fn point_expectation(draft: DxfPointDraft<'_>) -> Result<DxfPointDraftRecordExpectation, DxfError> {
    Ok(DxfPointDraftRecordExpectation {
        common: Box::new(DxfPointDraftCommonExpectation {
            layer: copy_bytes(draft.layer())?,
            layout: copy_bytes(draft.layout().unwrap_or_default())?,
            linetype: copy_bytes(draft.linetype().unwrap_or_default())?,
            references: DxfPointDraftReferenceExpectation::new(
                draft.material(),
                draft.plot_style(),
            ),
            color_name: copy_bytes(draft.color_name().unwrap_or_default())?,
        }),
        space: draft.space(),
        indexed_color: draft.indexed_color(),
        lineweight: draft.lineweight(),
        linetype_scale: draft.linetype_scale(),
        visibility: draft.visibility(),
        true_color: draft.true_color(),
        transparency: draft.transparency(),
        shadow_mode: draft.shadow_mode(),
        location: draft.location(),
        thickness: draft.thickness(),
        extrusion: draft.extrusion(),
        ucs_x_axis_angle: draft.ucs_x_axis_angle(),
    })
}

fn copy_bytes(bytes: &[u8]) -> Result<Box<[u8]>, DxfError> {
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| out_of_memory())?;
    owned.extend_from_slice(bytes);
    Ok(owned.into_boxed_slice())
}

impl DxfAsciiRawDocument<'_> {
    pub fn encode_entity_draft_record(
        &self,
        applicability: DxfEntityDraftApplicabilityPlan,
        draft: DxfEntityDraft<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftRecordPlan, DxfEntityDraftRecordIssue>, DxfError> {
        DxfRawDocumentView::from(self).encode_entity_draft_record(
            applicability,
            draft,
            profile,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn encode_entity_draft_record(
        &self,
        applicability: DxfEntityDraftApplicabilityPlan,
        draft: DxfEntityDraft<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftRecordPlan, DxfEntityDraftRecordIssue>, DxfError> {
        DxfRawDocumentView::from(self).encode_entity_draft_record(
            applicability,
            draft,
            profile,
            cancellation,
        )
    }
}

fn encode_point(
    document: DxfRawDocumentView<'_>,
    context: DxfEntityDraftEncodingContext,
    draft: DxfPointDraft<'_>,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<Vec<u8>, DxfEntityDraftRecordIssue>, DxfError> {
    if draft.layer().is_empty() {
        return Ok(Err(DxfEntityDraftRecordIssue::EmptyLayerName));
    }
    let target = context.placement;
    let modern_common = context.version >= DxfAcadVersion::Ac1015;
    let layout = match (modern_common, target, draft.layout()) {
        (true, DxfEntityPlacementTarget::EntitiesSection { .. }, None) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LayoutRequired { target }));
        }
        (true, DxfEntityPlacementTarget::EntitiesSection { .. }, Some(layout)) => Some(layout),
        (false, _, Some(_)) | (true, DxfEntityPlacementTarget::BlockDefinition { .. }, Some(_)) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LayoutNotApplicable {
                version: context.version,
                target,
            }));
        }
        (false, _, None) | (true, DxfEntityPlacementTarget::BlockDefinition { .. }, None) => None,
    };
    let lineweight = match (modern_common, draft.lineweight()) {
        (true, Some(lineweight)) => Some(lineweight),
        (true, None) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LineweightRequired {
                version: context.version,
            }));
        }
        (false, Some(_)) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LineweightNotApplicable {
                version: context.version,
            }));
        }
        (false, None) => None,
    };
    if draft
        .linetype_scale()
        .is_some_and(|scale| !scale.is_finite() || scale.to_f64() < 0.0)
    {
        return Ok(Err(DxfEntityDraftRecordIssue::InvalidLinetypeScale {
            value: draft.linetype_scale().ok_or_else(invalid_internal_data)?,
        }));
    }
    if context.version == DxfAcadVersion::Ac1009 {
        for (field, present) in [
            (DxfEntityField::MATERIAL, draft.material().is_some()),
            (DxfEntityField::TRUE_COLOR, draft.true_color().is_some()),
            (DxfEntityField::COLOR_NAME, draft.color_name().is_some()),
            (DxfEntityField::TRANSPARENCY, draft.transparency().is_some()),
            (DxfEntityField::SHADOW, draft.shadow_mode().is_some()),
            (DxfEntityField::PLOT_STYLE, draft.plot_style().is_some()),
        ] {
            if present {
                return Ok(Err(DxfEntityDraftRecordIssue::CommonFieldNotApplicable {
                    field,
                    version: context.version,
                }));
            }
        }
    }
    if let Some(color_name) = draft.color_name() {
        for (field, present) in [
            (DxfEntityField::COLOR, draft.indexed_color().is_some()),
            (DxfEntityField::TRUE_COLOR, draft.true_color().is_some()),
        ] {
            if !present {
                return Ok(Err(
                    DxfEntityDraftRecordIssue::ColorBookRelatedFieldRequired { field },
                ));
            }
        }
        if let Err(issue) =
            crate::entity_common_color_book_edit::validate_proposed_name(color_name, cancellation)?
        {
            return Ok(Err(DxfEntityDraftRecordIssue::ColorBook(issue)));
        }
    }
    if draft
        .extrusion()
        .is_some_and(|extrusion| extrusion.iter().all(|component| component.to_f64() == 0.0))
    {
        return Ok(Err(DxfEntityDraftRecordIssue::ZeroExtrusion));
    }
    let encoder = DxfEntityGroupEncoder::new(document.format(), context.version, profile);
    let mut record = PointRecordEncoder::new(encoder, profile);
    if let Some(issue) = record.push(
        0,
        DxfEntityFieldWireType::ExactText,
        DxfEntityEditValue::ExactRawText(b"POINT"),
        cancellation,
    )? {
        return Ok(Err(issue));
    }
    if let Some(issue) = record.push(
        5,
        DxfEntityFieldWireType::Handle,
        DxfEntityEditValue::Handle(context.handle),
        cancellation,
    )? {
        return Ok(Err(issue));
    }
    if context.version >= DxfAcadVersion::Ac1012 {
        if let Some(issue) = record.push(
            330,
            DxfEntityFieldWireType::Handle,
            DxfEntityEditValue::Handle(context.owner),
            cancellation,
        )? {
            return Ok(Err(issue));
        }
        if let Some(issue) = record.push(
            100,
            DxfEntityFieldWireType::ExactText,
            DxfEntityEditValue::ExactRawText(ACDB_ENTITY),
            cancellation,
        )? {
            return Ok(Err(issue));
        }
    }
    if let Some(space) = draft.space()
        && let Some(issue) = record.push(
            67,
            DxfEntityFieldWireType::Int16,
            DxfEntityEditValue::Int16(space.raw()),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(layout) = layout {
        if let Some(issue) = record.push(
            410,
            DxfEntityFieldWireType::ExactText,
            DxfEntityEditValue::ExactRawText(layout),
            cancellation,
        )? {
            return Ok(Err(issue));
        }
        match document.classify_entity_common_layout_edit(
            DxfEntityField::LAYOUT,
            DxfEntityEditValue::ExactRawText(layout),
            cancellation,
        )? {
            DxfEntityCommonLayoutEditOutcome::Valid(_) => {}
            DxfEntityCommonLayoutEditOutcome::Invalid(issue) => {
                return Ok(Err(DxfEntityDraftRecordIssue::LayoutReference(issue)));
            }
            DxfEntityCommonLayoutEditOutcome::NotLayout { .. } => {
                return Err(invalid_internal_data());
            }
        }
    }
    if let Some(issue) = record.push(
        8,
        DxfEntityFieldWireType::ExactText,
        DxfEntityEditValue::ExactRawText(draft.layer()),
        cancellation,
    )? {
        return Ok(Err(issue));
    }
    match document.classify_entity_common_symbol_edit(
        DxfEntityField::LAYER,
        DxfEntityEditValue::ExactRawText(draft.layer()),
        cancellation,
    )? {
        DxfEntityCommonSymbolEditOutcome::Valid(_) => {}
        DxfEntityCommonSymbolEditOutcome::Invalid(issue) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LayerReference(issue)));
        }
        DxfEntityCommonSymbolEditOutcome::NotSymbol { .. } => {
            return Err(invalid_internal_data());
        }
    }
    if let Some(linetype) = draft.linetype() {
        if let Some(issue) = record.push(
            6,
            DxfEntityFieldWireType::ExactText,
            DxfEntityEditValue::ExactRawText(linetype),
            cancellation,
        )? {
            return Ok(Err(issue));
        }
        match document.classify_entity_common_symbol_edit(
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(linetype),
            cancellation,
        )? {
            DxfEntityCommonSymbolEditOutcome::Valid(_) => {}
            DxfEntityCommonSymbolEditOutcome::Invalid(issue) => {
                return Ok(Err(DxfEntityDraftRecordIssue::SymbolReference(issue)));
            }
            DxfEntityCommonSymbolEditOutcome::NotSymbol { .. } => {
                return Err(invalid_internal_data());
            }
        }
    }
    if let Some(material) = draft.material() {
        if let Some(issue) = record.push(
            347,
            DxfEntityFieldWireType::Handle,
            DxfEntityEditValue::Handle(material),
            cancellation,
        )? {
            return Ok(Err(issue));
        }
        match document.classify_entity_common_reference_edit(
            DxfEntityField::MATERIAL,
            DxfEntityEditValue::Handle(material),
            cancellation,
        )? {
            DxfEntityCommonReferenceEditOutcome::Valid(_) => {}
            DxfEntityCommonReferenceEditOutcome::Invalid(issue) => {
                return Ok(Err(DxfEntityDraftRecordIssue::Reference(issue)));
            }
            DxfEntityCommonReferenceEditOutcome::NotReference { .. } => {
                return Err(invalid_internal_data());
            }
        }
    }
    if let Some(color) = draft.indexed_color()
        && let Some(issue) = record.push(
            62,
            DxfEntityFieldWireType::Int16,
            DxfEntityEditValue::Int16(color.raw()),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(lineweight) = lineweight
        && let Some(issue) = record.push(
            370,
            DxfEntityFieldWireType::Int16,
            DxfEntityEditValue::Int16(lineweight.raw()),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(scale) = draft.linetype_scale()
        && let Some(issue) = record.push(
            48,
            DxfEntityFieldWireType::Double,
            DxfEntityEditValue::Double(scale),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(visibility) = draft.visibility()
        && let Some(issue) = record.push(
            60,
            DxfEntityFieldWireType::Int16,
            DxfEntityEditValue::Int16(visibility.raw()),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(color) = draft.true_color()
        && let Some(issue) = record.push(
            420,
            DxfEntityFieldWireType::Int32,
            DxfEntityEditValue::Int32(color.raw()),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(color_name) = draft.color_name()
        && let Some(issue) = record.push(
            430,
            DxfEntityFieldWireType::ExactText,
            DxfEntityEditValue::ExactRawText(color_name),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(transparency) = draft.transparency()
        && let Some(issue) = record.push(
            440,
            DxfEntityFieldWireType::Int32,
            DxfEntityEditValue::Int32(transparency.raw()),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(plot_style) = draft.plot_style() {
        if let Some(issue) = record.push(
            390,
            DxfEntityFieldWireType::Handle,
            DxfEntityEditValue::Handle(plot_style),
            cancellation,
        )? {
            return Ok(Err(issue));
        }
        match document.classify_entity_common_reference_edit(
            DxfEntityField::PLOT_STYLE,
            DxfEntityEditValue::Handle(plot_style),
            cancellation,
        )? {
            DxfEntityCommonReferenceEditOutcome::Valid(_) => {}
            DxfEntityCommonReferenceEditOutcome::Invalid(issue) => {
                return Ok(Err(DxfEntityDraftRecordIssue::Reference(issue)));
            }
            DxfEntityCommonReferenceEditOutcome::NotReference { .. } => {
                return Err(invalid_internal_data());
            }
        }
    }
    if let Some(shadow_mode) = draft.shadow_mode()
        && let Some(issue) = record.push(
            284,
            DxfEntityFieldWireType::Int16,
            DxfEntityEditValue::Int16(shadow_mode.raw()),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if context.version >= DxfAcadVersion::Ac1012
        && let Some(issue) = record.push(
            100,
            DxfEntityFieldWireType::ExactText,
            DxfEntityEditValue::ExactRawText(ACDB_POINT),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    for (group_code, value) in [10_i16, 20, 30].into_iter().zip(draft.location()) {
        if let Some(issue) = record.push(
            group_code,
            DxfEntityFieldWireType::Double,
            DxfEntityEditValue::Double(value),
            cancellation,
        )? {
            return Ok(Err(issue));
        }
    }
    if let Some(thickness) = draft.thickness()
        && let Some(issue) = record.push(
            39,
            DxfEntityFieldWireType::Double,
            DxfEntityEditValue::Double(thickness),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    if let Some(extrusion) = draft.extrusion() {
        for (group_code, value) in [210_i16, 220, 230].into_iter().zip(extrusion) {
            if let Some(issue) = record.push(
                group_code,
                DxfEntityFieldWireType::Double,
                DxfEntityEditValue::Double(value),
                cancellation,
            )? {
                return Ok(Err(issue));
            }
        }
    }
    if let Some(angle) = draft.ucs_x_axis_angle()
        && let Some(issue) = record.push(
            50,
            DxfEntityFieldWireType::Double,
            DxfEntityEditValue::Double(angle),
            cancellation,
        )?
    {
        return Ok(Err(issue));
    }
    Ok(Ok(record.finish()))
}

struct PointRecordEncoder {
    bytes: Vec<u8>,
    encoder: DxfEntityGroupEncoder,
    profile: DxfResourceProfile,
}

impl PointRecordEncoder {
    const fn new(encoder: DxfEntityGroupEncoder, profile: DxfResourceProfile) -> Self {
        Self {
            bytes: Vec::new(),
            encoder,
            profile,
        }
    }

    fn push(
        &mut self,
        group_code: i16,
        wire_type: DxfEntityFieldWireType,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Option<DxfEntityDraftRecordIssue>, DxfError> {
        let encoded = match self
            .encoder
            .encode_raw(group_code, wire_type, value, cancellation)?
        {
            Ok(encoded) => encoded,
            Err(issue) => {
                return Ok(Some(DxfEntityDraftRecordIssue::GroupEncode {
                    group_code,
                    issue,
                }));
            }
        };
        let next_len = self
            .bytes
            .len()
            .checked_add(encoded.len())
            .ok_or_else(offset_overflow)?;
        let observed = u64::try_from(next_len).map_err(|_| offset_overflow())?;
        let limit = self.profile.limits().max_value_bytes();
        if observed > limit {
            return Err(DxfError::resource_limit(
                DxfResource::ValueBytes,
                limit,
                observed,
            ));
        }
        self.bytes
            .try_reserve(encoded.len())
            .map_err(|_| out_of_memory())?;
        self.bytes.extend_from_slice(&encoded);
        Ok(None)
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
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

fn offset_overflow() -> DxfError {
    DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    }
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        crate::DxfIoOperation::Write,
        &std::io::Error::from(std::io::ErrorKind::OutOfMemory),
    )
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Write,
        &std::io::Error::from(std::io::ErrorKind::InvalidData),
    )
}

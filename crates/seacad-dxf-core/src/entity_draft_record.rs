//! Typed family-draft validation and canonical record-byte encoding.

use crate::{
    DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble,
    DxfEntityCommonLayoutEditIssue, DxfEntityCommonLayoutEditOutcome,
    DxfEntityCommonSymbolEditIssue, DxfEntityCommonSymbolEditOutcome,
    DxfEntityDraftApplicabilityPlan, DxfEntityDraftName, DxfEntityEditValue, DxfEntityField,
    DxfEntityFieldWireType, DxfEntityGroupEncodeIssue, DxfEntityGroupEncoder, DxfEntityLineweight,
    DxfEntityNameClassification, DxfEntityPlacementTarget, DxfEntityTopic, DxfError, DxfHandle,
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
    lineweight: Option<DxfEntityLineweight>,
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
            lineweight: None,
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

    /// Supplies the non-omitted AC1015+ common lineweight.
    #[must_use]
    pub const fn with_lineweight(mut self, lineweight: DxfEntityLineweight) -> Self {
        self.lineweight = Some(lineweight);
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
    pub const fn lineweight(self) -> Option<DxfEntityLineweight> {
        self.lineweight
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
    ZeroExtrusion,
    LayerReference(DxfEntityCommonSymbolEditIssue),
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
    layer: Box<[u8]>,
    layout: Option<Box<[u8]>>,
    lineweight: Option<DxfEntityLineweight>,
    location: [DxfDouble; 3],
    thickness: Option<DxfDouble>,
    extrusion: Option<[DxfDouble; 3]>,
    ucs_x_axis_angle: Option<DxfDouble>,
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
        &self.layer
    }

    pub(crate) fn layout(&self) -> Option<&[u8]> {
        self.layout.as_deref()
    }

    pub(crate) const fn lineweight(&self) -> Option<DxfEntityLineweight> {
        self.lineweight
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
        layer: copy_bytes(draft.layer())?,
        layout: draft.layout().map(copy_bytes).transpose()?,
        lineweight: draft.lineweight(),
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

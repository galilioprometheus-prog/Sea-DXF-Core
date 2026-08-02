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
}

impl<'a> DxfPointDraft<'a> {
    #[must_use]
    pub const fn new(layer: &'a [u8], location: [DxfDouble; 3]) -> Self {
        Self {
            layer,
            layout: None,
            lineweight: None,
            location,
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
}

/// Typed payload for one entity family admitted by the generated registry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDraft<'a> {
    Point(DxfPointDraft<'a>),
}

impl<'a> DxfEntityDraft<'a> {
    #[must_use]
    pub const fn point(draft: DxfPointDraft<'a>) -> Self {
        Self::Point(draft)
    }

    #[must_use]
    pub const fn name(self) -> DxfEntityDraftName {
        match self {
            Self::Point(_) => DxfEntityDraftName::canonical(DxfEntityTopic::POINT),
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
        if applicability.name() != draft.name() {
            return Ok(Err(DxfEntityDraftRecordIssue::NameMismatch {
                admitted: applicability.classification(),
                draft: draft.name().classification(),
            }));
        }
        let bytes = match draft {
            DxfEntityDraft::Point(point) => {
                match encode_point(self, &applicability, point, profile, cancellation)? {
                    Ok(bytes) => bytes,
                    Err(issue) => return Ok(Err(issue)),
                }
            }
        };
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfEntityDraftRecordPlan {
            applicability,
            bytes: bytes.into_boxed_slice(),
        }))
    }
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
    applicability: &DxfEntityDraftApplicabilityPlan,
    draft: DxfPointDraft<'_>,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<Vec<u8>, DxfEntityDraftRecordIssue>, DxfError> {
    if draft.layer().is_empty() {
        return Ok(Err(DxfEntityDraftRecordIssue::EmptyLayerName));
    }
    let target = applicability.identity().placement().target();
    let modern_common = applicability.version() >= DxfAcadVersion::Ac1015;
    let layout = match (modern_common, target, draft.layout()) {
        (true, DxfEntityPlacementTarget::EntitiesSection { .. }, None) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LayoutRequired { target }));
        }
        (true, DxfEntityPlacementTarget::EntitiesSection { .. }, Some(layout)) => Some(layout),
        (false, _, Some(_)) | (true, DxfEntityPlacementTarget::BlockDefinition { .. }, Some(_)) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LayoutNotApplicable {
                version: applicability.version(),
                target,
            }));
        }
        (false, _, None) | (true, DxfEntityPlacementTarget::BlockDefinition { .. }, None) => None,
    };
    let lineweight = match (modern_common, draft.lineweight()) {
        (true, Some(lineweight)) => Some(lineweight),
        (true, None) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LineweightRequired {
                version: applicability.version(),
            }));
        }
        (false, Some(_)) => {
            return Ok(Err(DxfEntityDraftRecordIssue::LineweightNotApplicable {
                version: applicability.version(),
            }));
        }
        (false, None) => None,
    };
    let encoder = DxfEntityGroupEncoder::new(document.format(), applicability.version(), profile);
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
        DxfEntityEditValue::Handle(applicability.handle()),
        cancellation,
    )? {
        return Ok(Err(issue));
    }
    if applicability.version() >= DxfAcadVersion::Ac1012 {
        if let Some(issue) = record.push(
            330,
            DxfEntityFieldWireType::Handle,
            DxfEntityEditValue::Handle(applicability.identity().owner_handle()),
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
    if applicability.version() >= DxfAcadVersion::Ac1012
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

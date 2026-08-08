//! Exact projection and composition of one POINT family draft with its source XDATA.

use std::io;

use crate::{
    DxfAcadVersion, DxfCancellationToken, DxfEntityKey, DxfEntityPlacementTarget, DxfEntityRef,
    DxfEntityXDataDraftRecordIssue, DxfEntityXDataDraftRecordPlan,
    DxfEntityXDataEncodedEntityDestinationDirectory, DxfEntityXDataEncodedEntityDestinationEntry,
    DxfEntityXDataEncodedEntityDestinationState, DxfError, DxfHandle, DxfIoOperation,
    DxfPointCloneDestinationBindings, DxfPointCloneDialectAdaptations,
    DxfPointCloneDraftProjectionIssue, DxfRawDocumentView, DxfResourceProfile, DxfSourceId,
    DxfTextTranscodeReceipt,
};

/// One exact encoded-XDATA source entry selected for POINT projection.
#[derive(Clone, Copy)]
pub struct DxfPointCloneXDataSource<'a> {
    directory: &'a DxfEntityXDataEncodedEntityDestinationDirectory,
    entry: DxfEntityXDataEncodedEntityDestinationEntry,
}

impl<'a> DxfPointCloneXDataSource<'a> {
    pub fn new(
        directory: &'a DxfEntityXDataEncodedEntityDestinationDirectory,
        entry: DxfEntityXDataEncodedEntityDestinationEntry,
    ) -> Result<Self, DxfError> {
        if directory.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        Ok(Self { directory, entry })
    }

    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.directory.source_id()
    }

    #[must_use]
    pub const fn destination_id(self) -> DxfSourceId {
        self.directory.destination_id()
    }

    #[must_use]
    pub const fn entry(self) -> DxfEntityXDataEncodedEntityDestinationEntry {
        self.entry
    }
}

impl std::fmt::Debug for DxfPointCloneXDataSource<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DxfPointCloneXDataSource")
            .field("source_id", &self.source_id())
            .field("destination_id", &self.destination_id())
            .field("entry", &self.entry)
            .finish()
    }
}

/// Typed reason why a source POINT and its encoded XDATA cannot become one
/// destination draft.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPointCloneXDataDraftIssue {
    Projection(DxfPointCloneDraftProjectionIssue),
    XData(DxfEntityXDataDraftRecordIssue),
}

/// One source-bound POINT projection composed with the exact encoded XDATA
/// payload owned by that source entity.
pub struct DxfPointCloneXDataDraftPlan {
    source_key: DxfEntityKey,
    source_version: DxfAcadVersion,
    source_placement: DxfEntityPlacementTarget,
    source_owner: Option<DxfHandle>,
    adaptations: DxfPointCloneDialectAdaptations,
    color_name_transcode: Option<DxfTextTranscodeReceipt>,
    xdata: DxfEntityXDataDraftRecordPlan,
}

impl DxfPointCloneXDataDraftPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.xdata.source_id()
    }

    #[must_use]
    pub const fn source_key(&self) -> DxfEntityKey {
        self.source_key
    }

    #[must_use]
    pub const fn source_version(&self) -> DxfAcadVersion {
        self.source_version
    }

    #[must_use]
    pub const fn source_placement(&self) -> DxfEntityPlacementTarget {
        self.source_placement
    }

    #[must_use]
    pub const fn source_owner(&self) -> Option<DxfHandle> {
        self.source_owner
    }

    #[must_use]
    pub const fn dialect_adaptations(&self) -> DxfPointCloneDialectAdaptations {
        self.adaptations
    }

    #[must_use]
    pub const fn color_name_transcode(&self) -> Option<DxfTextTranscodeReceipt> {
        self.color_name_transcode
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.xdata.destination_id()
    }

    #[must_use]
    pub const fn source_entity(&self) -> DxfEntityRef {
        self.xdata.source_entity()
    }

    #[must_use]
    pub const fn encoded_entry(&self) -> DxfEntityXDataEncodedEntityDestinationEntry {
        self.xdata.encoded_entry()
    }

    #[must_use]
    pub const fn encoded_state(&self) -> DxfEntityXDataEncodedEntityDestinationState {
        self.xdata.encoded_state()
    }

    #[must_use]
    pub const fn xdata_draft_record(&self) -> &DxfEntityXDataDraftRecordPlan {
        &self.xdata
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.xdata.bytes()
    }

    #[must_use]
    pub fn into_xdata_draft_record(self) -> DxfEntityXDataDraftRecordPlan {
        self.xdata
    }
}

impl std::fmt::Debug for DxfPointCloneXDataDraftPlan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DxfPointCloneXDataDraftPlan")
            .field("source_key", &self.source_key)
            .field("source_version", &self.source_version)
            .field("source_placement", &self.source_placement)
            .field("has_source_owner", &self.source_owner.is_some())
            .field("adaptations", &self.adaptations)
            .field(
                "has_color_name_transcode",
                &self.color_name_transcode.is_some(),
            )
            .field("xdata", &self.xdata)
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    /// Projects the exact POINT owning `xdata`, then immediately composes its
    /// encoded payload. No destination insertion occurs.
    pub fn project_point_clone_xdata_draft_from(
        self,
        source: DxfRawDocumentView<'_>,
        xdata: DxfPointCloneXDataSource<'_>,
        destination_applicability: crate::DxfEntityDraftApplicabilityPlan,
        bindings: DxfPointCloneDestinationBindings<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfPointCloneXDataDraftPlan, DxfPointCloneXDataDraftIssue>, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(source.source_id(), xdata.source_id())?;
        ensure_source(self.source_id(), xdata.destination_id())?;
        let source_entity = xdata.directory.entity_for_entry(xdata.entry())?;
        let projection = match self.project_point_clone_draft_from(
            source,
            source_entity.key(),
            destination_applicability,
            bindings.with_xdata_composition(),
            profile,
            cancellation,
        )? {
            Ok(plan) => plan,
            Err(issue) => return Ok(Err(DxfPointCloneXDataDraftIssue::Projection(issue))),
        };
        let source_key = projection.source_key();
        let source_version = projection.source_version();
        let source_placement = projection.source_placement();
        let source_owner = projection.source_owner();
        let adaptations = projection.dialect_adaptations();
        let color_name_transcode = projection.color_name_transcode();
        let composed = match xdata.directory.compose_entity_draft_record(
            xdata.entry(),
            projection.into_destination(),
            profile,
            cancellation,
        )? {
            Ok(plan) => plan,
            Err(issue) => return Ok(Err(DxfPointCloneXDataDraftIssue::XData(issue))),
        };
        if composed.source_entity() != source_entity || source_key != source_entity.key() {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfPointCloneXDataDraftPlan {
            source_key,
            source_version,
            source_placement,
            source_owner,
            adaptations,
            color_name_transcode,
            xdata: composed,
        }))
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

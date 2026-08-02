//! Typed entity name, handle reservation, placement, and owner preparation.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityAlias,
    DxfEntityNameClassification, DxfEntityPlacement, DxfEntityPlacementOwnerBinding,
    DxfEntityTopic, DxfError, DxfHandle, DxfHandleAllocationProposal, DxfHandleReservationPlan,
    DxfIoOperation, DxfNamedSymbolTableEntry, DxfRawDocumentView, DxfSourceId, DxfTransactionPlan,
};

/// Exact generated group-zero name selected for a future typed entity draft.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDraftName {
    Canonical(DxfEntityTopic),
    Alias(DxfEntityAlias),
}

impl DxfEntityDraftName {
    #[must_use]
    pub const fn canonical(topic: DxfEntityTopic) -> Self {
        Self::Canonical(topic)
    }

    #[must_use]
    pub const fn alias(alias: DxfEntityAlias) -> Self {
        Self::Alias(alias)
    }

    #[must_use]
    pub const fn classification(self) -> DxfEntityNameClassification {
        match self {
            Self::Canonical(topic) => DxfEntityNameClassification::Canonical(topic),
            Self::Alias(alias) => DxfEntityNameClassification::Alias(alias),
        }
    }

    #[must_use]
    pub fn canonical_topic(self) -> Option<DxfEntityTopic> {
        match self {
            Self::Canonical(topic) => Some(topic),
            Self::Alias(alias) => alias.descriptor().map(|descriptor| descriptor.topic()),
        }
    }

    #[must_use]
    pub fn exact_name(self) -> Option<&'static str> {
        match self {
            Self::Canonical(topic) => topic.descriptor().map(|descriptor| descriptor.dxf_name()),
            Self::Alias(alias) => alias.descriptor().map(|descriptor| descriptor.dxf_name()),
        }
    }

    #[must_use]
    pub const fn from_classification(classification: DxfEntityNameClassification) -> Option<Self> {
        match classification {
            DxfEntityNameClassification::Canonical(topic) => Some(Self::Canonical(topic)),
            DxfEntityNameClassification::Alias(alias) => Some(Self::Alias(alias)),
            DxfEntityNameClassification::Unknown => None,
        }
    }
}

/// Typed preparation failure before any draft bytes are encoded.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDraftIdentityIssue {
    ReservationCardinality { handle_count: u64 },
}

/// One exact name, placement-owner binding, and single-handle reservation.
#[derive(Debug)]
pub struct DxfEntityDraftIdentityPlan {
    source_id: DxfSourceId,
    name: DxfEntityDraftName,
    binding: DxfEntityPlacementOwnerBinding,
    reservation: DxfHandleReservationPlan,
}

impl DxfEntityDraftIdentityPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn name(&self) -> DxfEntityDraftName {
        self.name
    }

    #[must_use]
    pub const fn binding(&self) -> DxfEntityPlacementOwnerBinding {
        self.binding
    }

    #[must_use]
    pub const fn placement(&self) -> DxfEntityPlacement {
        self.binding.placement()
    }

    #[must_use]
    pub const fn handle(&self) -> DxfHandle {
        self.reservation.allocation().first_handle()
    }

    #[must_use]
    pub const fn owner_handle(&self) -> DxfHandle {
        self.binding.owner_handle()
    }

    #[must_use]
    pub const fn owner_block_record(&self) -> DxfNamedSymbolTableEntry {
        self.binding.block_record_entry()
    }

    #[must_use]
    pub const fn allocation(&self) -> DxfHandleAllocationProposal {
        self.reservation.allocation()
    }

    #[must_use]
    pub const fn reservation(&self) -> &DxfHandleReservationPlan {
        &self.reservation
    }

    #[must_use]
    pub const fn transaction(&self) -> &DxfTransactionPlan {
        self.reservation.transaction()
    }

    #[must_use]
    pub fn into_reservation(self) -> DxfHandleReservationPlan {
        self.reservation
    }
}

impl DxfRawDocumentView<'_> {
    pub fn prepare_entity_draft_identity(
        self,
        name: DxfEntityDraftName,
        binding: DxfEntityPlacementOwnerBinding,
        reservation: DxfHandleReservationPlan,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftIdentityPlan, DxfEntityDraftIdentityIssue>, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), binding.source_id())?;
        ensure_source(self.source_id(), reservation.source_id())?;
        reservation
            .transaction()
            .validate_source_precondition(self)?;
        let handle_count = reservation.allocation().handle_count();
        if handle_count != 1 {
            return Ok(Err(DxfEntityDraftIdentityIssue::ReservationCardinality {
                handle_count,
            }));
        }
        if reservation.allocation().handle_at(0).is_none() {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfEntityDraftIdentityPlan {
            source_id: self.source_id(),
            name,
            binding,
            reservation,
        }))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn prepare_entity_draft_identity(
        &self,
        name: DxfEntityDraftName,
        binding: DxfEntityPlacementOwnerBinding,
        reservation: DxfHandleReservationPlan,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftIdentityPlan, DxfEntityDraftIdentityIssue>, DxfError> {
        DxfRawDocumentView::from(self).prepare_entity_draft_identity(
            name,
            binding,
            reservation,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn prepare_entity_draft_identity(
        &self,
        name: DxfEntityDraftName,
        binding: DxfEntityPlacementOwnerBinding,
        reservation: DxfHandleReservationPlan,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftIdentityPlan, DxfEntityDraftIdentityIssue>, DxfError> {
        DxfRawDocumentView::from(self).prepare_entity_draft_identity(
            name,
            binding,
            reservation,
            cancellation,
        )
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

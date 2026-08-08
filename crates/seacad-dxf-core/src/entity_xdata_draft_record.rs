//! Destination-bound composition of canonical entity drafts with encoded XDATA payloads.

use std::{fmt, io};

use crate::{
    DxfCancellationToken, DxfEntityDraftRecordPlan, DxfEntityRef,
    DxfEntityXDataEncodedEntityDestinationDirectory, DxfEntityXDataEncodedEntityDestinationEntry,
    DxfEntityXDataEncodedEntityDestinationState, DxfError, DxfIoOperation, DxfResourceProfile,
    DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataDraftRecordIssue {
    EncodedPayloadUnavailable {
        state: DxfEntityXDataEncodedEntityDestinationState,
    },
}

pub struct DxfEntityXDataDraftRecordPlan {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    source_entity: DxfEntityRef,
    encoded_entry: DxfEntityXDataEncodedEntityDestinationEntry,
    record: DxfEntityDraftRecordPlan,
}

impl DxfEntityXDataDraftRecordPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn source_entity(&self) -> DxfEntityRef {
        self.source_entity
    }

    #[must_use]
    pub const fn encoded_entry(&self) -> DxfEntityXDataEncodedEntityDestinationEntry {
        self.encoded_entry
    }

    #[must_use]
    pub const fn encoded_state(&self) -> DxfEntityXDataEncodedEntityDestinationState {
        self.encoded_entry.state()
    }

    #[must_use]
    pub const fn draft_record(&self) -> &DxfEntityDraftRecordPlan {
        &self.record
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.record.bytes()
    }

    #[must_use]
    pub fn into_draft_record(self) -> DxfEntityDraftRecordPlan {
        self.record
    }
}

impl fmt::Debug for DxfEntityXDataDraftRecordPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityXDataDraftRecordPlan")
            .field("source_id", &self.source_id)
            .field("destination_id", &self.destination_id)
            .field("source_entity", &self.source_entity)
            .field("encoded_entry", &self.encoded_entry)
            .field("record", &self.record)
            .field("byte_count", &self.bytes().len())
            .finish()
    }
}

impl DxfEntityXDataEncodedEntityDestinationDirectory {
    pub fn compose_entity_draft_record(
        &self,
        entry: DxfEntityXDataEncodedEntityDestinationEntry,
        record: DxfEntityDraftRecordPlan,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityXDataDraftRecordPlan, DxfEntityXDataDraftRecordIssue>, DxfError>
    {
        ensure_not_cancelled(cancellation)?;
        if self.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        ensure_source(self.destination_id(), record.source_id())?;
        let source_entity = self.entity_for_entry(entry)?;
        let DxfEntityXDataEncodedEntityDestinationState::Ready { .. } = entry.state() else {
            return Ok(Err(
                DxfEntityXDataDraftRecordIssue::EncodedPayloadUnavailable {
                    state: entry.state(),
                },
            ));
        };
        let payload = self
            .encoded_bytes_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        let record = record.append_exact_groups(payload, profile)?;
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfEntityXDataDraftRecordPlan {
            source_id: self.source_id(),
            destination_id: self.destination_id(),
            source_entity,
            encoded_entry: entry,
            record,
        }))
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
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

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

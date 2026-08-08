//! Destination insertion planning for canonical entity drafts carrying encoded XDATA.

use std::{fmt, io};

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityEditPlan,
    DxfEntityRef, DxfEntityXDataDraftRecordPlan, DxfEntityXDataEncodedEntityDestinationEntry,
    DxfEntityXDataEncodedEntityDestinationState, DxfError, DxfHandle, DxfIoOperation,
    DxfRawDocumentView, DxfResourceProfile, DxfSourceId,
};

pub struct DxfEntityXDataDraftInsertPlan {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    source_entity: DxfEntityRef,
    encoded_entry: DxfEntityXDataEncodedEntityDestinationEntry,
    destination_handle: DxfHandle,
    expected_xdata: Box<[u8]>,
    edit: DxfEntityEditPlan,
}

impl DxfEntityXDataDraftInsertPlan {
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
    pub const fn destination_handle(&self) -> DxfHandle {
        self.destination_handle
    }

    #[must_use]
    pub fn expected_xdata_bytes(&self) -> &[u8] {
        &self.expected_xdata
    }

    #[must_use]
    pub const fn edit_plan(&self) -> &DxfEntityEditPlan {
        &self.edit
    }

    #[must_use]
    pub fn into_edit_plan(self) -> DxfEntityEditPlan {
        self.edit
    }
}

impl fmt::Debug for DxfEntityXDataDraftInsertPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityXDataDraftInsertPlan")
            .field("source_id", &self.source_id)
            .field("destination_id", &self.destination_id)
            .field("source_entity", &self.source_entity)
            .field("encoded_entry", &self.encoded_entry)
            .field("destination_handle", &self.destination_handle)
            .field("expected_xdata_byte_count", &self.expected_xdata.len())
            .field("edit", &self.edit)
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn plan_entity_xdata_draft_insert(
        self,
        record: DxfEntityXDataDraftRecordPlan,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataDraftInsertPlan, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), record.destination_id())?;
        ensure_source(self.source_id(), record.draft_record().source_id())?;
        let DxfEntityXDataEncodedEntityDestinationState::Ready {
            encoded_byte_count, ..
        } = record.encoded_state()
        else {
            return Err(invalid_internal_data());
        };
        let xdata_len = usize::try_from(encoded_byte_count).map_err(|_| invalid_internal_data())?;
        let xdata_start = record
            .bytes()
            .len()
            .checked_sub(xdata_len)
            .ok_or_else(invalid_internal_data)?;
        let expected = record
            .bytes()
            .get(xdata_start..)
            .ok_or_else(invalid_internal_data)?;
        let expected_xdata = copy_bytes(expected)?;
        let source_id = record.source_id();
        let destination_id = record.destination_id();
        let source_entity = record.source_entity();
        let encoded_entry = record.encoded_entry();
        let destination_handle = record.draft_record().handle();
        let edit =
            self.plan_entity_draft_insert(record.into_draft_record(), profile, cancellation)?;
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityXDataDraftInsertPlan {
            source_id,
            destination_id,
            source_entity,
            encoded_entry,
            destination_handle,
            expected_xdata,
            edit,
        })
    }
}

macro_rules! document_xdata_draft_insert {
    ($document:ty) => {
        impl $document {
            pub fn plan_entity_xdata_draft_insert(
                &self,
                record: DxfEntityXDataDraftRecordPlan,
                profile: DxfResourceProfile,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataDraftInsertPlan, DxfError> {
                DxfRawDocumentView::from(self).plan_entity_xdata_draft_insert(
                    record,
                    profile,
                    cancellation,
                )
            }
        }
    };
}

document_xdata_draft_insert!(DxfAsciiRawDocument<'_>);
document_xdata_draft_insert!(DxfBinaryRawDocument<'_>);

fn copy_bytes(bytes: &[u8]) -> Result<Box<[u8]>, DxfError> {
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| out_of_memory())?;
    owned.extend_from_slice(bytes);
    Ok(owned.into_boxed_slice())
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

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

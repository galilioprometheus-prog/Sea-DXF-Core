//! Canonical destination-byte XDATA payloads composed per indexed entity.

use std::{fmt, io};

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataEncodedApplicationDestinationDirectory,
    DxfEntityXDataEncodedApplicationDestinationEntry,
    DxfEntityXDataEncodedApplicationDestinationState, DxfEntityXDataHandleRemap,
    DxfEntityXDataPayloadDestinationEntry, DxfEntityXDataPayloadDestinationState, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfResourceProfile, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataEncodedEntityDestinationState {
    Ready {
        application_count: u32,
        member_count: u32,
        encoded_byte_count: u64,
    },
    Unavailable {
        payload_state: DxfEntityXDataPayloadDestinationState,
        application_count: u32,
        unavailable_application_count: u32,
        first_unavailable_application_ordinal: Option<u32>,
        member_count: u32,
        unavailable_member_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataEncodedEntityDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    payload_ordinal: u32,
    byte_start: u32,
    byte_end: u32,
    state: DxfEntityXDataEncodedEntityDestinationState,
}

impl DxfEntityXDataEncodedEntityDestinationEntry {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn payload_ordinal(self) -> u64 {
        self.payload_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataEncodedEntityDestinationState {
        self.state
    }
}

pub struct DxfEntityXDataEncodedEntityDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    applications: DxfEntityXDataEncodedApplicationDestinationDirectory,
    entries: Box<[DxfEntityXDataEncodedEntityDestinationEntry]>,
    bytes: Box<[u8]>,
}

impl DxfEntityXDataEncodedEntityDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let applications = source.entity_xdata_encoded_application_destination_directory(
            destination,
            transform,
            mappings,
            profile,
            cancellation,
        )?;
        ensure_source(source.source_id(), applications.source_id())?;
        ensure_source(destination.source_id(), applications.destination_id())?;
        let payloads = applications
            .encoded_destination_directory()
            .logical_destination_directory()
            .payload_destination_directory();
        let mut entries = Vec::new();
        let mut bytes = Vec::new();
        entries
            .try_reserve_exact(payloads.entries().len())
            .map_err(|_| out_of_memory())?;
        for payload in payloads.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let entity = payloads.entity_for_entry(payload)?;
            let application_entries = applications.entries_for_entity(entity)?;
            validate_applications(&applications, payload, application_entries)?;
            let byte_start = compact_len(bytes.len())?;
            let state = compose_state(&applications, payload, application_entries, &mut bytes)?;
            entries.push(DxfEntityXDataEncodedEntityDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                payload_ordinal: compact_u64(payload.ordinal())?,
                byte_start,
                byte_end: compact_len(bytes.len())?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            applications,
            entries: entries.into_boxed_slice(),
            bytes: bytes.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn encoded_application_destination_directory(
        &self,
    ) -> &DxfEntityXDataEncodedApplicationDestinationDirectory {
        &self.applications
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataEncodedEntityDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataEncodedEntityDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn payload_for_entry(
        &self,
        entry: DxfEntityXDataEncodedEntityDestinationEntry,
    ) -> Option<DxfEntityXDataPayloadDestinationEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.applications
            .encoded_destination_directory()
            .logical_destination_directory()
            .payload_destination_directory()
            .entry(entry.payload_ordinal())
    }

    pub fn entity_for_entry(
        &self,
        entry: DxfEntityXDataEncodedEntityDestinationEntry,
    ) -> Result<DxfEntityRef, DxfError> {
        let payload = self
            .payload_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.applications
            .encoded_destination_directory()
            .logical_destination_directory()
            .payload_destination_directory()
            .entity_for_entry(payload)
    }

    pub fn entry_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<Option<DxfEntityXDataEncodedEntityDestinationEntry>, DxfError> {
        let payload = self
            .applications
            .encoded_destination_directory()
            .logical_destination_directory()
            .payload_destination_directory()
            .entry_for_entity(entity)?;
        Ok(payload.and_then(|payload| {
            self.entry(payload.ordinal())
                .filter(|entry| self.payload_for_entry(*entry) == Some(payload))
        }))
    }

    pub fn applications_for_entry(
        &self,
        entry: DxfEntityXDataEncodedEntityDestinationEntry,
    ) -> Result<&[DxfEntityXDataEncodedApplicationDestinationEntry], DxfError> {
        let entity = self.entity_for_entry(entry)?;
        self.applications.entries_for_entity(entity)
    }

    #[must_use]
    pub fn encoded_bytes_for_entry(
        &self,
        entry: DxfEntityXDataEncodedEntityDestinationEntry,
    ) -> Option<&[u8]> {
        if self.entry(entry.ordinal()) != Some(entry)
            || !matches!(
                entry.state(),
                DxfEntityXDataEncodedEntityDestinationState::Ready { .. }
            )
        {
            return None;
        }
        self.bytes
            .get(usize::try_from(entry.byte_start).ok()?..usize::try_from(entry.byte_end).ok()?)
    }
}

impl fmt::Debug for DxfEntityXDataEncodedEntityDestinationDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityXDataEncodedEntityDestinationDirectory")
            .field("source_id", &self.source_id)
            .field("destination_id", &self.destination_id)
            .field("applications", &self.applications)
            .field("entries", &self.entries)
            .field("encoded_byte_count", &self.bytes.len())
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_encoded_entity_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataEncodedEntityDestinationDirectory, DxfError> {
        DxfEntityXDataEncodedEntityDestinationDirectory::from_documents(
            self,
            destination,
            transform,
            mappings,
            profile,
            cancellation,
        )
    }
}

macro_rules! document_encoded_entity_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_encoded_entity_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                transform: DxfEntityXDataCoordinateTransform,
                mappings: &[DxfEntityXDataHandleRemap],
                profile: DxfResourceProfile,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataEncodedEntityDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_encoded_entity_destination_directory(
                    destination,
                    transform,
                    mappings,
                    profile,
                    cancellation,
                )
            }
        }
    };
}

document_encoded_entity_destination_directory!(DxfAsciiRawDocument<'_>);
document_encoded_entity_destination_directory!(DxfBinaryRawDocument<'_>);

fn validate_applications(
    applications: &DxfEntityXDataEncodedApplicationDestinationDirectory,
    payload: DxfEntityXDataPayloadDestinationEntry,
    entries: &[DxfEntityXDataEncodedApplicationDestinationEntry],
) -> Result<(), DxfError> {
    for entry in entries.iter().copied() {
        if applications.payload_for_entry(entry) != Some(payload) {
            return Err(invalid_internal_data());
        }
    }
    Ok(())
}

fn compose_state(
    applications: &DxfEntityXDataEncodedApplicationDestinationDirectory,
    payload: DxfEntityXDataPayloadDestinationEntry,
    entries: &[DxfEntityXDataEncodedApplicationDestinationEntry],
    bytes: &mut Vec<u8>,
) -> Result<DxfEntityXDataEncodedEntityDestinationState, DxfError> {
    let application_count = compact_len(entries.len())?;
    let mut unavailable_application_count = 0_u32;
    let mut first_unavailable_application_ordinal = None;
    let mut member_count = 0_u32;
    let mut unavailable_member_count = 0_u32;
    let mut encoded_byte_count = 0_u64;
    for (ordinal, entry) in entries.iter().copied().enumerate() {
        match entry.state() {
            DxfEntityXDataEncodedApplicationDestinationState::Ready {
                member_count: members,
                encoded_byte_count: encoded,
            } => {
                member_count = member_count
                    .checked_add(members)
                    .ok_or_else(invalid_internal_data)?;
                encoded_byte_count = encoded_byte_count
                    .checked_add(encoded)
                    .ok_or_else(invalid_internal_data)?;
            }
            DxfEntityXDataEncodedApplicationDestinationState::Unavailable {
                member_count: members,
                unavailable_member_count: unavailable_members,
                ..
            } => {
                unavailable_application_count = unavailable_application_count
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
                if first_unavailable_application_ordinal.is_none() {
                    first_unavailable_application_ordinal = Some(compact_len(ordinal)?);
                }
                member_count = member_count
                    .checked_add(members)
                    .ok_or_else(invalid_internal_data)?;
                unavailable_member_count = unavailable_member_count
                    .checked_add(unavailable_members)
                    .ok_or_else(invalid_internal_data)?;
            }
        }
    }
    if matches!(
        payload.state(),
        DxfEntityXDataPayloadDestinationState::Ready { .. }
    ) && unavailable_application_count == 0
    {
        let reserve = usize::try_from(encoded_byte_count).map_err(|_| invalid_internal_data())?;
        bytes.try_reserve(reserve).map_err(|_| out_of_memory())?;
        for entry in entries.iter().copied() {
            let application_bytes = applications
                .encoded_bytes_for_entry(entry)
                .ok_or_else(invalid_internal_data)?;
            bytes.extend_from_slice(application_bytes);
        }
        Ok(DxfEntityXDataEncodedEntityDestinationState::Ready {
            application_count,
            member_count,
            encoded_byte_count,
        })
    } else {
        Ok(DxfEntityXDataEncodedEntityDestinationState::Unavailable {
            payload_state: payload.state(),
            application_count,
            unavailable_application_count,
            first_unavailable_application_ordinal,
            member_count,
            unavailable_member_count,
        })
    }
}

fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

fn compact_u64(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
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
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

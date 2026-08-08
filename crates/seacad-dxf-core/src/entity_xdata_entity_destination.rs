//! Per-entity generic XDATA destination readiness for applications and source capacity.
//!
//! This composes destination-ready applications with exact source 16-KiB capacity evidence. It
//! does not validate transformed coordinates, mapped handles, payload semantics, or encoding.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplicationDestinationDirectory, DxfEntityXDataApplicationDestinationEntry,
    DxfEntityXDataApplicationDestinationState, DxfEntityXDataCapacityDirectory,
    DxfEntityXDataCapacityEntry, DxfEntityXDataCapacityIssue, DxfEntityXDataCapacityState,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// Exact application and source-capacity readiness for one indexed entity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataEntityDestinationState {
    Ready {
        application_count: u32,
        used_bytes: u64,
        remaining_bytes: u64,
    },
    Unavailable {
        application_count: u32,
        unavailable_application_count: u32,
        capacity: DxfEntityXDataCapacityState,
    },
}

/// One indexed entity composed with application destination and source capacity evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataEntityDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    raw_record_ordinal: u32,
    capacity_ordinal: u32,
    state: DxfEntityXDataEntityDestinationState,
}

impl DxfEntityXDataEntityDestinationEntry {
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
    pub const fn raw_record_ordinal(self) -> u64 {
        self.raw_record_ordinal as u64
    }

    #[must_use]
    pub const fn capacity_ordinal(self) -> u64 {
        self.capacity_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataEntityDestinationState {
        self.state
    }
}

/// Source-order entity readiness for destination applications and exact source capacity.
#[derive(Debug)]
pub struct DxfEntityXDataEntityDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    applications: DxfEntityXDataApplicationDestinationDirectory,
    capacity: DxfEntityXDataCapacityDirectory,
    entries: Box<[DxfEntityXDataEntityDestinationEntry]>,
}

impl DxfEntityXDataEntityDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let applications =
            source.entity_xdata_application_destination_directory(destination, cancellation)?;
        let capacity = source.entity_xdata_capacity_directory(cancellation)?;
        for observed in [applications.source_id(), capacity.source_id()] {
            ensure_source(source.source_id(), observed)?;
        }
        ensure_source(destination.source_id(), applications.destination_id())?;

        let mut entries = Vec::new();
        entries
            .try_reserve_exact(capacity.entries().len())
            .map_err(|_| out_of_memory())?;
        for capacity_entry in capacity.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let entity = capacity.entity_for_entry(capacity_entry)?;
            let application_entries = applications.entries_for_entity(entity)?;
            entries.push(DxfEntityXDataEntityDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                raw_record_ordinal: compact_u64(entity.record().ordinal())?,
                capacity_ordinal: compact_u64(capacity_entry.ordinal())?,
                state: readiness_state(application_entries, capacity_entry)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            applications,
            capacity,
            entries: entries.into_boxed_slice(),
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
    pub const fn application_destination_directory(
        &self,
    ) -> &DxfEntityXDataApplicationDestinationDirectory {
        &self.applications
    }

    #[must_use]
    pub const fn capacity_directory(&self) -> &DxfEntityXDataCapacityDirectory {
        &self.capacity
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataEntityDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataEntityDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entry_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<Option<DxfEntityXDataEntityDestinationEntry>, DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        Ok(self
            .entries
            .binary_search_by_key(&entity.record().ordinal(), |entry| {
                entry.raw_record_ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied()))
    }

    pub fn entity_for_entry(
        &self,
        entry: DxfEntityXDataEntityDestinationEntry,
    ) -> Result<DxfEntityRef, DxfError> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        let capacity = self
            .capacity_entry_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        let entity = self.capacity.entity_for_entry(capacity)?;
        if entity.record().ordinal() != entry.raw_record_ordinal() {
            return Err(invalid_internal_data());
        }
        Ok(entity)
    }

    #[must_use]
    pub fn capacity_entry_for_entry(
        &self,
        entry: DxfEntityXDataEntityDestinationEntry,
    ) -> Option<DxfEntityXDataCapacityEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.capacity
            .entry(entry.capacity_ordinal())
            .filter(|capacity| capacity.raw_record_ordinal() == entry.raw_record_ordinal())
    }

    pub fn capacity_issues_for_entry(
        &self,
        entry: DxfEntityXDataEntityDestinationEntry,
    ) -> Result<&[DxfEntityXDataCapacityIssue], DxfError> {
        let capacity = self
            .capacity_entry_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.capacity.issues_for_entry(capacity)
    }

    pub fn application_entries_for_entry(
        &self,
        entry: DxfEntityXDataEntityDestinationEntry,
    ) -> Result<&[DxfEntityXDataApplicationDestinationEntry], DxfError> {
        let entity = self.entity_for_entry(entry)?;
        self.applications.entries_for_entity(entity)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_entity_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataEntityDestinationDirectory, DxfError> {
        DxfEntityXDataEntityDestinationDirectory::from_documents(self, destination, cancellation)
    }
}

macro_rules! document_entity_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_entity_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataEntityDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_entity_destination_directory(destination, cancellation)
            }
        }
    };
}

document_entity_destination_directory!(DxfAsciiRawDocument<'_>);
document_entity_destination_directory!(DxfBinaryRawDocument<'_>);

fn readiness_state(
    applications: &[DxfEntityXDataApplicationDestinationEntry],
    capacity: DxfEntityXDataCapacityEntry,
) -> Result<DxfEntityXDataEntityDestinationState, DxfError> {
    let application_count = compact_len(applications.len())?;
    let mut unavailable_application_count = 0_u32;
    for application in applications.iter().copied() {
        if !matches!(
            application.state(),
            DxfEntityXDataApplicationDestinationState::Ready { .. }
        ) {
            unavailable_application_count = unavailable_application_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
        }
    }
    match (unavailable_application_count, capacity.state()) {
        (
            0,
            DxfEntityXDataCapacityState::WithinLimit {
                used_bytes,
                remaining_bytes,
            },
        ) => Ok(DxfEntityXDataEntityDestinationState::Ready {
            application_count,
            used_bytes,
            remaining_bytes,
        }),
        (_, capacity) => Ok(DxfEntityXDataEntityDestinationState::Unavailable {
            application_count,
            unavailable_application_count,
            capacity,
        }),
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
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

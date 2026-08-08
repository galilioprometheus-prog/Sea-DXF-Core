//! Per-entity destination readiness composed with XDATA coordinate transformation.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataCoordinateTransformDirectory,
    DxfEntityXDataEntityDestinationDirectory, DxfEntityXDataEntityDestinationEntry,
    DxfEntityXDataEntityDestinationState, DxfEntityXDataTransformedPointEntry,
    DxfEntityXDataTransformedPointState, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataCoordinateDestinationState {
    Ready {
        application_count: u32,
        used_bytes: u64,
        remaining_bytes: u64,
        transformed_tuple_count: u32,
    },
    Unavailable {
        entity_state: DxfEntityXDataEntityDestinationState,
        tuple_count: u32,
        unavailable_tuple_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataCoordinateDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    entity_destination_ordinal: u32,
    state: DxfEntityXDataCoordinateDestinationState,
}

impl DxfEntityXDataCoordinateDestinationEntry {
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
    pub const fn entity_destination_ordinal(self) -> u64 {
        self.entity_destination_ordinal as u64
    }
    #[must_use]
    pub const fn state(self) -> DxfEntityXDataCoordinateDestinationState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataCoordinateDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    entities: DxfEntityXDataEntityDestinationDirectory,
    coordinates: DxfEntityXDataCoordinateTransformDirectory,
    entries: Box<[DxfEntityXDataCoordinateDestinationEntry]>,
}

impl DxfEntityXDataCoordinateDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let entities =
            source.entity_xdata_entity_destination_directory(destination, cancellation)?;
        let coordinates =
            source.entity_xdata_coordinate_transform_directory(transform, cancellation)?;
        ensure_source(source.source_id(), entities.source_id())?;
        ensure_source(source.source_id(), coordinates.source_id())?;
        ensure_source(destination.source_id(), entities.destination_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(entities.entries().len())
            .map_err(|_| out_of_memory())?;
        for entity_entry in entities.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let entity = entities.entity_for_entry(entity_entry)?;
            let tuples = coordinates
                .point_tuple_directory()
                .tuples_for_entity(entity)?;
            let transformed = transformed_slice(&coordinates, tuples)?;
            entries.push(DxfEntityXDataCoordinateDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                entity_destination_ordinal: compact_u64(entity_entry.ordinal())?,
                state: readiness_state(entity_entry, transformed)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            entities,
            coordinates,
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
    pub const fn entity_destination_directory(&self) -> &DxfEntityXDataEntityDestinationDirectory {
        &self.entities
    }
    #[must_use]
    pub const fn coordinate_transform_directory(
        &self,
    ) -> &DxfEntityXDataCoordinateTransformDirectory {
        &self.coordinates
    }
    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataCoordinateDestinationEntry] {
        &self.entries
    }
    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataCoordinateDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }
    #[must_use]
    pub fn entity_destination_for_entry(
        &self,
        entry: DxfEntityXDataCoordinateDestinationEntry,
    ) -> Option<DxfEntityXDataEntityDestinationEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.entities.entry(entry.entity_destination_ordinal())
    }

    pub fn entry_for_entity(
        &self,
        entity: crate::DxfEntityRef,
    ) -> Result<Option<DxfEntityXDataCoordinateDestinationEntry>, DxfError> {
        let Some(entity_entry) = self.entities.entry_for_entity(entity)? else {
            return Ok(None);
        };
        Ok(self
            .entry(entity_entry.ordinal())
            .filter(|entry| self.entity_destination_for_entry(*entry) == Some(entity_entry)))
    }

    pub fn entity_for_entry(
        &self,
        entry: DxfEntityXDataCoordinateDestinationEntry,
    ) -> Result<crate::DxfEntityRef, DxfError> {
        let entity_entry = self
            .entity_destination_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.entities.entity_for_entry(entity_entry)
    }
    pub fn transformed_entries_for_entry(
        &self,
        entry: DxfEntityXDataCoordinateDestinationEntry,
    ) -> Result<&[DxfEntityXDataTransformedPointEntry], DxfError> {
        let entity_entry = self
            .entity_destination_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        let entity = self.entities.entity_for_entry(entity_entry)?;
        let tuples = self
            .coordinates
            .point_tuple_directory()
            .tuples_for_entity(entity)?;
        transformed_slice(&self.coordinates, tuples)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_coordinate_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataCoordinateDestinationDirectory, DxfError> {
        DxfEntityXDataCoordinateDestinationDirectory::from_documents(
            self,
            destination,
            transform,
            cancellation,
        )
    }
}

macro_rules! document_coordinate_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_coordinate_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                transform: DxfEntityXDataCoordinateTransform,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataCoordinateDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_coordinate_destination_directory(
                    destination,
                    transform,
                    cancellation,
                )
            }
        }
    };
}

document_coordinate_destination_directory!(DxfAsciiRawDocument<'_>);
document_coordinate_destination_directory!(DxfBinaryRawDocument<'_>);

fn transformed_slice<'a>(
    directory: &'a DxfEntityXDataCoordinateTransformDirectory,
    tuples: &[crate::DxfEntityXDataPointTuple],
) -> Result<&'a [DxfEntityXDataTransformedPointEntry], DxfError> {
    let Some(first) = tuples.first() else {
        return directory
            .entries()
            .get(0..0)
            .ok_or_else(invalid_internal_data);
    };
    let start = usize::try_from(first.ordinal()).map_err(|_| invalid_internal_data())?;
    let end = start
        .checked_add(tuples.len())
        .ok_or_else(invalid_internal_data)?;
    directory
        .entries()
        .get(start..end)
        .ok_or_else(invalid_internal_data)
}

fn readiness_state(
    entity: DxfEntityXDataEntityDestinationEntry,
    transformed: &[DxfEntityXDataTransformedPointEntry],
) -> Result<DxfEntityXDataCoordinateDestinationState, DxfError> {
    let tuple_count = compact_len(transformed.len())?;
    let unavailable_tuple_count = compact_len(
        transformed
            .iter()
            .filter(|entry| {
                !matches!(
                    entry.state(),
                    DxfEntityXDataTransformedPointState::Available { .. }
                )
            })
            .count(),
    )?;
    match (entity.state(), unavailable_tuple_count) {
        (
            DxfEntityXDataEntityDestinationState::Ready {
                application_count,
                used_bytes,
                remaining_bytes,
            },
            0,
        ) => Ok(DxfEntityXDataCoordinateDestinationState::Ready {
            application_count,
            used_bytes,
            remaining_bytes,
            transformed_tuple_count: tuple_count,
        }),
        (entity_state, _) => Ok(DxfEntityXDataCoordinateDestinationState::Unavailable {
            entity_state,
            tuple_count,
            unavailable_tuple_count,
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

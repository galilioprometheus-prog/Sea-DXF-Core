//! Per-entity destination readiness composed with XDATA handle remapping.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataCoordinateDestinationDirectory, DxfEntityXDataCoordinateDestinationEntry,
    DxfEntityXDataCoordinateDestinationState, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataHandleDestinationDirectory, DxfEntityXDataHandleDestinationEntry,
    DxfEntityXDataHandleDestinationState, DxfEntityXDataHandleRemap, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleComposedDestinationState {
    Ready {
        application_count: u32,
        used_bytes: u64,
        remaining_bytes: u64,
        transformed_tuple_count: u32,
        destination_handle_count: u32,
    },
    Unavailable {
        coordinate_state: DxfEntityXDataCoordinateDestinationState,
        handle_count: u32,
        unavailable_handle_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataHandleComposedDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    coordinate_destination_ordinal: u32,
    state: DxfEntityXDataHandleComposedDestinationState,
}

impl DxfEntityXDataHandleComposedDestinationEntry {
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
    pub const fn coordinate_destination_ordinal(self) -> u64 {
        self.coordinate_destination_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataHandleComposedDestinationState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataHandleComposedDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    coordinates: DxfEntityXDataCoordinateDestinationDirectory,
    handles: DxfEntityXDataHandleDestinationDirectory,
    entries: Box<[DxfEntityXDataHandleComposedDestinationEntry]>,
}

impl DxfEntityXDataHandleComposedDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let coordinates = source.entity_xdata_coordinate_destination_directory(
            destination,
            transform,
            cancellation,
        )?;
        let handles = source.entity_xdata_handle_destination_directory(
            destination,
            mappings,
            cancellation,
        )?;
        ensure_source(source.source_id(), coordinates.source_id())?;
        ensure_source(source.source_id(), handles.source_id())?;
        ensure_source(destination.source_id(), coordinates.destination_id())?;
        ensure_source(destination.source_id(), handles.destination_id())?;

        let mut entries = Vec::new();
        entries
            .try_reserve_exact(coordinates.entries().len())
            .map_err(|_| out_of_memory())?;
        for coordinate in coordinates.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let entity = coordinates.entity_for_entry(coordinate)?;
            let entity_handles = handles.entries_for_entity(entity)?;
            entries.push(DxfEntityXDataHandleComposedDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                coordinate_destination_ordinal: compact_u64(coordinate.ordinal())?,
                state: readiness_state(coordinate, entity_handles)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            coordinates,
            handles,
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
    pub const fn coordinate_destination_directory(
        &self,
    ) -> &DxfEntityXDataCoordinateDestinationDirectory {
        &self.coordinates
    }

    #[must_use]
    pub const fn handle_destination_directory(&self) -> &DxfEntityXDataHandleDestinationDirectory {
        &self.handles
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataHandleComposedDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataHandleComposedDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn coordinate_destination_for_entry(
        &self,
        entry: DxfEntityXDataHandleComposedDestinationEntry,
    ) -> Option<DxfEntityXDataCoordinateDestinationEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.coordinates
            .entry(entry.coordinate_destination_ordinal())
    }

    pub fn entry_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<Option<DxfEntityXDataHandleComposedDestinationEntry>, DxfError> {
        let Some(coordinate) = self.coordinates.entry_for_entity(entity)? else {
            return Ok(None);
        };
        Ok(self
            .entry(coordinate.ordinal())
            .filter(|entry| self.coordinate_destination_for_entry(*entry) == Some(coordinate)))
    }

    pub fn entity_for_entry(
        &self,
        entry: DxfEntityXDataHandleComposedDestinationEntry,
    ) -> Result<DxfEntityRef, DxfError> {
        let coordinate = self
            .coordinate_destination_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.coordinates.entity_for_entry(coordinate)
    }

    pub fn handle_entries_for_entry(
        &self,
        entry: DxfEntityXDataHandleComposedDestinationEntry,
    ) -> Result<&[DxfEntityXDataHandleDestinationEntry], DxfError> {
        let entity = self.entity_for_entry(entry)?;
        self.handles.entries_for_entity(entity)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_handle_composed_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleComposedDestinationDirectory, DxfError> {
        DxfEntityXDataHandleComposedDestinationDirectory::from_documents(
            self,
            destination,
            transform,
            mappings,
            cancellation,
        )
    }
}

macro_rules! document_handle_composed_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_handle_composed_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                transform: DxfEntityXDataCoordinateTransform,
                mappings: &[DxfEntityXDataHandleRemap],
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataHandleComposedDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_handle_composed_destination_directory(
                    destination,
                    transform,
                    mappings,
                    cancellation,
                )
            }
        }
    };
}

document_handle_composed_destination_directory!(DxfAsciiRawDocument<'_>);
document_handle_composed_destination_directory!(DxfBinaryRawDocument<'_>);

fn readiness_state(
    coordinate: DxfEntityXDataCoordinateDestinationEntry,
    handles: &[DxfEntityXDataHandleDestinationEntry],
) -> Result<DxfEntityXDataHandleComposedDestinationState, DxfError> {
    let handle_count = compact_len(handles.len())?;
    let unavailable_handle_count = compact_len(
        handles
            .iter()
            .filter(|entry| {
                !matches!(
                    entry.state(),
                    DxfEntityXDataHandleDestinationState::Unique { .. }
                )
            })
            .count(),
    )?;
    match (coordinate.state(), unavailable_handle_count) {
        (
            DxfEntityXDataCoordinateDestinationState::Ready {
                application_count,
                used_bytes,
                remaining_bytes,
                transformed_tuple_count,
            },
            0,
        ) => Ok(DxfEntityXDataHandleComposedDestinationState::Ready {
            application_count,
            used_bytes,
            remaining_bytes,
            transformed_tuple_count,
            destination_handle_count: handle_count,
        }),
        (coordinate_state, _) => Ok(DxfEntityXDataHandleComposedDestinationState::Unavailable {
            coordinate_state,
            handle_count,
            unavailable_handle_count,
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

//! Per-entity destination readiness for structurally enclosed XDATA payloads.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplication, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataHandleComposedDestinationDirectory, DxfEntityXDataHandleComposedDestinationEntry,
    DxfEntityXDataHandleComposedDestinationState, DxfEntityXDataHandleRemap,
    DxfEntityXDataOccurrenceKind, DxfEntityXDataTypedDirectory, DxfEntityXDataTypedEntry, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataPayloadDestinationState {
    Ready {
        application_count: u32,
        payload_value_count: u32,
        used_bytes: u64,
        remaining_bytes: u64,
        transformed_tuple_count: u32,
        destination_handle_count: u32,
    },
    Unavailable {
        composition_state: DxfEntityXDataHandleComposedDestinationState,
        payload_value_count: u32,
        orphan_value_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataPayloadDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    composition_ordinal: u32,
    state: DxfEntityXDataPayloadDestinationState,
}

impl DxfEntityXDataPayloadDestinationEntry {
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
    pub const fn composition_ordinal(self) -> u64 {
        self.composition_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataPayloadDestinationState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataPayloadDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    compositions: DxfEntityXDataHandleComposedDestinationDirectory,
    entries: Box<[DxfEntityXDataPayloadDestinationEntry]>,
}

impl DxfEntityXDataPayloadDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let compositions = source.entity_xdata_handle_composed_destination_directory(
            destination,
            transform,
            mappings,
            cancellation,
        )?;
        ensure_source(source.source_id(), compositions.source_id())?;
        ensure_source(destination.source_id(), compositions.destination_id())?;

        let mut entries = Vec::new();
        entries
            .try_reserve_exact(compositions.entries().len())
            .map_err(|_| out_of_memory())?;
        for composition in compositions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let entity = compositions.entity_for_entry(composition)?;
            let typed = typed_directory(&compositions).entries_for_entity(entity)?;
            let applications = typed_directory(&compositions)
                .xdata_directory()
                .applications_for_entity(entity)?;
            entries.push(DxfEntityXDataPayloadDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                composition_ordinal: compact_u64(composition.ordinal())?,
                state: readiness_state(composition, typed, applications)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            compositions,
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
    pub const fn handle_composed_destination_directory(
        &self,
    ) -> &DxfEntityXDataHandleComposedDestinationDirectory {
        &self.compositions
    }

    #[must_use]
    pub fn typed_directory(&self) -> &DxfEntityXDataTypedDirectory {
        typed_directory(&self.compositions)
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataPayloadDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataPayloadDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn composition_for_entry(
        &self,
        entry: DxfEntityXDataPayloadDestinationEntry,
    ) -> Option<DxfEntityXDataHandleComposedDestinationEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.compositions.entry(entry.composition_ordinal())
    }

    pub fn entry_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<Option<DxfEntityXDataPayloadDestinationEntry>, DxfError> {
        let Some(composition) = self.compositions.entry_for_entity(entity)? else {
            return Ok(None);
        };
        Ok(self
            .entry(composition.ordinal())
            .filter(|entry| self.composition_for_entry(*entry) == Some(composition)))
    }

    pub fn entity_for_entry(
        &self,
        entry: DxfEntityXDataPayloadDestinationEntry,
    ) -> Result<DxfEntityRef, DxfError> {
        let composition = self
            .composition_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.compositions.entity_for_entry(composition)
    }

    pub fn typed_entries_for_entry(
        &self,
        entry: DxfEntityXDataPayloadDestinationEntry,
    ) -> Result<&[DxfEntityXDataTypedEntry], DxfError> {
        let entity = self.entity_for_entry(entry)?;
        typed_directory(&self.compositions).entries_for_entity(entity)
    }

    pub fn applications_for_entry(
        &self,
        entry: DxfEntityXDataPayloadDestinationEntry,
    ) -> Result<&[DxfEntityXDataApplication], DxfError> {
        let entity = self.entity_for_entry(entry)?;
        typed_directory(&self.compositions)
            .xdata_directory()
            .applications_for_entity(entity)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_payload_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataPayloadDestinationDirectory, DxfError> {
        DxfEntityXDataPayloadDestinationDirectory::from_documents(
            self,
            destination,
            transform,
            mappings,
            cancellation,
        )
    }
}

macro_rules! document_payload_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_payload_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                transform: DxfEntityXDataCoordinateTransform,
                mappings: &[DxfEntityXDataHandleRemap],
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataPayloadDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_payload_destination_directory(
                    destination,
                    transform,
                    mappings,
                    cancellation,
                )
            }
        }
    };
}

document_payload_destination_directory!(DxfAsciiRawDocument<'_>);
document_payload_destination_directory!(DxfBinaryRawDocument<'_>);

fn typed_directory(
    compositions: &DxfEntityXDataHandleComposedDestinationDirectory,
) -> &DxfEntityXDataTypedDirectory {
    compositions
        .coordinate_destination_directory()
        .coordinate_transform_directory()
        .point_tuple_directory()
        .typed_directory()
}

fn readiness_state(
    composition: DxfEntityXDataHandleComposedDestinationEntry,
    typed: &[DxfEntityXDataTypedEntry],
    applications: &[DxfEntityXDataApplication],
) -> Result<DxfEntityXDataPayloadDestinationState, DxfError> {
    let payload_value_count = typed
        .len()
        .checked_sub(applications.len())
        .ok_or_else(invalid_internal_data)
        .and_then(compact_len)?;
    let orphan_value_count = compact_len(
        typed
            .iter()
            .filter(|entry| {
                matches!(
                    entry.occurrence().kind(),
                    DxfEntityXDataOccurrenceKind::Orphan
                )
            })
            .count(),
    )?;
    match (composition.state(), orphan_value_count) {
        (
            DxfEntityXDataHandleComposedDestinationState::Ready {
                application_count,
                used_bytes,
                remaining_bytes,
                transformed_tuple_count,
                destination_handle_count,
            },
            0,
        ) => Ok(DxfEntityXDataPayloadDestinationState::Ready {
            application_count,
            payload_value_count,
            used_bytes,
            remaining_bytes,
            transformed_tuple_count,
            destination_handle_count,
        }),
        (composition_state, _) => Ok(DxfEntityXDataPayloadDestinationState::Unavailable {
            composition_state,
            payload_value_count,
            orphan_value_count,
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

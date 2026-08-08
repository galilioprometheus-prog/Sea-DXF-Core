//! Destination-document validation for mapped generic XDATA group-1005 handles.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataHandleRemap, DxfEntityXDataHandleRemapDirectory, DxfEntityXDataHandleRemapEntry,
    DxfEntityXDataHandleRemapState, DxfError, DxfHandle, DxfHandleIdentityDirectory,
    DxfHandleIdentityLookup, DxfHandleIdentityMatch, DxfIoOperation, DxfRawDocumentView,
    DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleDestinationState {
    RemapUnavailable(DxfEntityXDataHandleRemapState),
    Missing {
        target: DxfHandle,
    },
    Unique {
        target: DxfHandle,
    },
    Ambiguous {
        target: DxfHandle,
        target_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataHandleDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    remap_ordinal: u32,
    state: DxfEntityXDataHandleDestinationState,
}

impl DxfEntityXDataHandleDestinationEntry {
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
    pub const fn remap_ordinal(self) -> u64 {
        self.remap_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataHandleDestinationState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataHandleDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    remaps: DxfEntityXDataHandleRemapDirectory,
    destination_identities: DxfHandleIdentityDirectory,
    entries: Box<[DxfEntityXDataHandleDestinationEntry]>,
}

impl DxfEntityXDataHandleDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let remaps = source.entity_xdata_handle_remap_directory(mappings, cancellation)?;
        let destination_identities = destination.handle_identity_directory(cancellation)?;
        ensure_source(source.source_id(), remaps.source_id())?;
        ensure_source(destination.source_id(), destination_identities.source_id())?;

        let mut entries = Vec::new();
        entries
            .try_reserve_exact(remaps.entries().len())
            .map_err(|_| out_of_memory())?;
        for remap in remaps.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = destination_state(remap, &destination_identities)?;
            entries.push(DxfEntityXDataHandleDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                remap_ordinal: compact_u64(remap.ordinal())?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            remaps,
            destination_identities,
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
    pub const fn remap_directory(&self) -> &DxfEntityXDataHandleRemapDirectory {
        &self.remaps
    }

    #[must_use]
    pub const fn destination_identity_directory(&self) -> &DxfHandleIdentityDirectory {
        &self.destination_identities
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataHandleDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataHandleDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn remap_for_entry(
        &self,
        entry: DxfEntityXDataHandleDestinationEntry,
    ) -> Option<DxfEntityXDataHandleRemapEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.remaps.entry(entry.remap_ordinal())
    }

    #[must_use]
    pub fn destination_target_for_entry(
        &self,
        entry: DxfEntityXDataHandleDestinationEntry,
    ) -> Option<DxfHandleIdentityMatch> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        let DxfEntityXDataHandleDestinationState::Unique { target } = entry.state() else {
            return None;
        };
        match self.destination_identities.lookup(target) {
            DxfHandleIdentityLookup::Unique(identity) => Some(identity),
            DxfHandleIdentityLookup::Missing | DxfHandleIdentityLookup::Ambiguous(_) => None,
        }
    }

    #[must_use]
    pub fn source_entity_for_entry(
        &self,
        entry: DxfEntityXDataHandleDestinationEntry,
    ) -> Option<DxfEntityRef> {
        let remap = self.remap_for_entry(entry)?;
        let resolutions = self.remaps.resolution_directory();
        let resolution = self.remaps.resolution_for_entry(remap)?;
        Some(
            resolutions
                .typed_for_entry(resolution)?
                .occurrence()
                .entity(),
        )
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataHandleDestinationEntry], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let ordinal = entity.record().ordinal();
        let start = self.entries.partition_point(|entry| {
            self.source_entity_for_entry(*entry)
                .is_some_and(|source| source.record().ordinal() < ordinal)
        });
        let end = self.entries.partition_point(|entry| {
            self.source_entity_for_entry(*entry)
                .is_some_and(|source| source.record().ordinal() <= ordinal)
        });
        let entries = self
            .entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)?;
        if entries.iter().copied().all(|entry| {
            self.source_entity_for_entry(entry)
                .is_some_and(|source| source == entity)
        }) {
            Ok(entries)
        } else {
            Err(invalid_internal_data())
        }
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_handle_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleDestinationDirectory, DxfError> {
        DxfEntityXDataHandleDestinationDirectory::from_documents(
            self,
            destination,
            mappings,
            cancellation,
        )
    }
}

macro_rules! document_handle_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_handle_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                mappings: &[DxfEntityXDataHandleRemap],
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataHandleDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_handle_destination_directory(
                    destination,
                    mappings,
                    cancellation,
                )
            }
        }
    };
}

document_handle_destination_directory!(DxfAsciiRawDocument<'_>);
document_handle_destination_directory!(DxfBinaryRawDocument<'_>);

fn destination_state(
    remap: DxfEntityXDataHandleRemapEntry,
    identities: &DxfHandleIdentityDirectory,
) -> Result<DxfEntityXDataHandleDestinationState, DxfError> {
    let DxfEntityXDataHandleRemapState::Mapped { target, .. } = remap.state() else {
        return Ok(DxfEntityXDataHandleDestinationState::RemapUnavailable(
            remap.state(),
        ));
    };
    Ok(match identities.lookup(target) {
        DxfHandleIdentityLookup::Missing => {
            DxfEntityXDataHandleDestinationState::Missing { target }
        }
        DxfHandleIdentityLookup::Unique(_) => {
            DxfEntityXDataHandleDestinationState::Unique { target }
        }
        DxfHandleIdentityLookup::Ambiguous(identities) => {
            DxfEntityXDataHandleDestinationState::Ambiguous {
                target,
                target_count: compact_len(identities.len())?,
            }
        }
    })
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

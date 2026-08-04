//! Entity-level all-or-nothing readiness for generic XDATA handle replacements.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataHandleRemap, DxfEntityXDataHandleReplacementDirectory,
    DxfEntityXDataHandleReplacementEntry, DxfEntityXDataHandleReplacementPatch,
    DxfEntityXDataHandleReplacementState, DxfError, DxfIoOperation, DxfRawDocumentView,
    DxfResourceProfile, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleReplacementSetState {
    Ready {
        replacement_count: u32,
    },
    Unavailable {
        replacement_count: u32,
        unavailable_count: u32,
        first_unavailable_ordinal: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataHandleReplacementSetEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    entity: DxfEntityRef,
    replacement_start: u32,
    replacement_end: u32,
    state: DxfEntityXDataHandleReplacementSetState,
}

impl DxfEntityXDataHandleReplacementSetEntry {
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
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataHandleReplacementSetState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataHandleReplacementSetDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    replacements: DxfEntityXDataHandleReplacementDirectory,
    entries: Box<[DxfEntityXDataHandleReplacementSetEntry]>,
}

impl DxfEntityXDataHandleReplacementSetDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let replacements = source.entity_xdata_handle_replacement_directory(
            destination,
            mappings,
            profile,
            cancellation,
        )?;
        ensure_source(source.source_id(), replacements.source_id())?;
        ensure_source(destination.source_id(), replacements.destination_id())?;
        let mut entries = Vec::new();
        let mut start = 0_usize;
        while start < replacements.entries().len() {
            ensure_not_cancelled(cancellation)?;
            let first = replacements
                .entries()
                .get(start)
                .copied()
                .ok_or_else(invalid_internal_data)?;
            let entity = replacement_entity(&replacements, first)?;
            let mut end = start + 1;
            while end < replacements.entries().len() {
                let candidate = replacements
                    .entries()
                    .get(end)
                    .copied()
                    .ok_or_else(invalid_internal_data)?;
                if replacement_entity(&replacements, candidate)? != entity {
                    break;
                }
                end += 1;
            }
            let state = set_state(&replacements.entries()[start..end])?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfEntityXDataHandleReplacementSetEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                entity,
                replacement_start: compact_len(start)?,
                replacement_end: compact_len(end)?,
                state,
            });
            start = end;
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            replacements,
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
    pub const fn replacement_directory(&self) -> &DxfEntityXDataHandleReplacementDirectory {
        &self.replacements
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataHandleReplacementSetEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataHandleReplacementSetEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn replacements_for_entry(
        &self,
        entry: DxfEntityXDataHandleReplacementSetEntry,
    ) -> Option<&[DxfEntityXDataHandleReplacementEntry]> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.replacements.entries().get(
            usize::try_from(entry.replacement_start).ok()?
                ..usize::try_from(entry.replacement_end).ok()?,
        )
    }

    #[must_use]
    pub fn patch_for_replacement(
        &self,
        entry: DxfEntityXDataHandleReplacementSetEntry,
        replacement: DxfEntityXDataHandleReplacementEntry,
    ) -> Option<DxfEntityXDataHandleReplacementPatch> {
        let replacements = self.replacements_for_entry(entry)?;
        if !replacements.contains(&replacement) {
            return None;
        }
        self.replacements.patch_for_entry(replacement)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_handle_replacement_set_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleReplacementSetDirectory, DxfError> {
        DxfEntityXDataHandleReplacementSetDirectory::from_documents(
            self,
            destination,
            mappings,
            profile,
            cancellation,
        )
    }
}

macro_rules! document_handle_replacement_set_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_handle_replacement_set_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                mappings: &[DxfEntityXDataHandleRemap],
                profile: DxfResourceProfile,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataHandleReplacementSetDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_handle_replacement_set_directory(
                    destination,
                    mappings,
                    profile,
                    cancellation,
                )
            }
        }
    };
}

document_handle_replacement_set_directory!(DxfAsciiRawDocument<'_>);
document_handle_replacement_set_directory!(DxfBinaryRawDocument<'_>);

fn replacement_entity(
    replacements: &DxfEntityXDataHandleReplacementDirectory,
    entry: DxfEntityXDataHandleReplacementEntry,
) -> Result<DxfEntityRef, DxfError> {
    let destination = replacements
        .destination_for_entry(entry)
        .ok_or_else(invalid_internal_data)?;
    let remaps = replacements.destination_directory().remap_directory();
    let remap = remaps
        .entry(destination.remap_ordinal())
        .ok_or_else(invalid_internal_data)?;
    let resolutions = remaps.resolution_directory();
    let resolution = resolutions
        .entry(remap.resolution_ordinal())
        .ok_or_else(invalid_internal_data)?;
    let typed = resolutions
        .typed_for_entry(resolution)
        .ok_or_else(invalid_internal_data)?;
    Ok(typed.occurrence().entity())
}

fn set_state(
    replacements: &[DxfEntityXDataHandleReplacementEntry],
) -> Result<DxfEntityXDataHandleReplacementSetState, DxfError> {
    let replacement_count = compact_len(replacements.len())?;
    let mut unavailable_count = 0_u32;
    let mut first_unavailable_ordinal = None;
    for replacement in replacements.iter().copied() {
        if !matches!(
            replacement.state(),
            DxfEntityXDataHandleReplacementState::Ready { .. }
        ) {
            unavailable_count = unavailable_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            first_unavailable_ordinal.get_or_insert(compact_u64(replacement.ordinal())?);
        }
    }
    match first_unavailable_ordinal {
        Some(first_unavailable_ordinal) => {
            Ok(DxfEntityXDataHandleReplacementSetState::Unavailable {
                replacement_count,
                unavailable_count,
                first_unavailable_ordinal,
            })
        }
        None => Ok(DxfEntityXDataHandleReplacementSetState::Ready { replacement_count }),
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

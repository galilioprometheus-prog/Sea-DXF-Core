//! Fail-closed cross-document remap planning for generic XDATA group-1005 handles.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityXDataHandleResolutionDirectory, DxfEntityXDataHandleResolutionEntry, DxfError,
    DxfHandle, DxfHandleIdentityMatch, DxfHandleParseIssue, DxfHandleResolutionState,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleRemapInputIssue {
    NullSource,
    NullTarget,
}

/// One caller-supplied source-handle to destination-handle candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DxfEntityXDataHandleRemap {
    source: DxfHandle,
    target: DxfHandle,
}

impl DxfEntityXDataHandleRemap {
    pub const fn new(
        source: DxfHandle,
        target: DxfHandle,
    ) -> Result<Self, DxfEntityXDataHandleRemapInputIssue> {
        if source.is_null() {
            Err(DxfEntityXDataHandleRemapInputIssue::NullSource)
        } else if target.is_null() {
            Err(DxfEntityXDataHandleRemapInputIssue::NullTarget)
        } else {
            Ok(Self { source, target })
        }
    }

    #[must_use]
    pub const fn source(self) -> DxfHandle {
        self.source
    }

    #[must_use]
    pub const fn target(self) -> DxfHandle {
        self.target
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleRemapState {
    SourceInvalid(DxfHandleParseIssue),
    SourceNull,
    SourceMissing,
    SourceAmbiguous {
        target_count: u32,
    },
    Unmapped {
        source: DxfHandle,
    },
    Mapped {
        source: DxfHandle,
        target: DxfHandle,
    },
    AmbiguousMapping {
        source: DxfHandle,
        candidate_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataHandleRemapEntry {
    source_id: DxfSourceId,
    ordinal: u32,
    resolution_ordinal: u32,
    state: DxfEntityXDataHandleRemapState,
}

impl DxfEntityXDataHandleRemapEntry {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn resolution_ordinal(self) -> u64 {
        self.resolution_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataHandleRemapState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataHandleRemapDirectory {
    source_id: DxfSourceId,
    resolutions: DxfEntityXDataHandleResolutionDirectory,
    mappings: Box<[DxfEntityXDataHandleRemap]>,
    entries: Box<[DxfEntityXDataHandleRemapEntry]>,
}

impl DxfEntityXDataHandleRemapDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let resolutions = document.entity_xdata_handle_resolution_directory(cancellation)?;
        ensure_source(document.source_id(), resolutions.source_id())?;

        let mut owned_mappings = Vec::new();
        owned_mappings
            .try_reserve_exact(mappings.len())
            .map_err(|_| out_of_memory())?;
        for mapping in mappings.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            owned_mappings.push(mapping);
        }
        owned_mappings.sort_unstable();

        let mut entries = Vec::new();
        entries
            .try_reserve_exact(resolutions.entries().len())
            .map_err(|_| out_of_memory())?;
        for resolution in resolutions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfEntityXDataHandleRemapEntry {
                source_id: document.source_id(),
                ordinal: compact_len(entries.len())?,
                resolution_ordinal: compact_u64(resolution.ordinal())?,
                state: remap_state(resolution, &owned_mappings)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            resolutions,
            mappings: owned_mappings.into_boxed_slice(),
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn resolution_directory(&self) -> &DxfEntityXDataHandleResolutionDirectory {
        &self.resolutions
    }

    #[must_use]
    pub fn mappings(&self) -> &[DxfEntityXDataHandleRemap] {
        &self.mappings
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataHandleRemapEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataHandleRemapEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn resolution_for_entry(
        &self,
        entry: DxfEntityXDataHandleRemapEntry,
    ) -> Option<DxfEntityXDataHandleResolutionEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.resolutions.entry(entry.resolution_ordinal())
    }

    #[must_use]
    pub fn source_target_for_entry(
        &self,
        entry: DxfEntityXDataHandleRemapEntry,
    ) -> Option<DxfHandleIdentityMatch> {
        self.resolution_for_entry(entry)?.target()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_handle_remap_directory(
        self,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleRemapDirectory, DxfError> {
        DxfEntityXDataHandleRemapDirectory::from_document(self, mappings, cancellation)
    }
}

macro_rules! document_handle_remap_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_handle_remap_directory(
                &self,
                mappings: &[DxfEntityXDataHandleRemap],
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataHandleRemapDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_handle_remap_directory(mappings, cancellation)
            }
        }
    };
}

document_handle_remap_directory!(DxfAsciiRawDocument<'_>);
document_handle_remap_directory!(DxfBinaryRawDocument<'_>);

fn remap_state(
    resolution: DxfEntityXDataHandleResolutionEntry,
    mappings: &[DxfEntityXDataHandleRemap],
) -> Result<DxfEntityXDataHandleRemapState, DxfError> {
    let source = match resolution.state() {
        DxfHandleResolutionState::Invalid(issue) => {
            return Ok(DxfEntityXDataHandleRemapState::SourceInvalid(issue));
        }
        DxfHandleResolutionState::Null => return Ok(DxfEntityXDataHandleRemapState::SourceNull),
        DxfHandleResolutionState::Missing => {
            return Ok(DxfEntityXDataHandleRemapState::SourceMissing);
        }
        DxfHandleResolutionState::Ambiguous { target_count } => {
            return Ok(DxfEntityXDataHandleRemapState::SourceAmbiguous { target_count });
        }
        DxfHandleResolutionState::Unique => resolution
            .target()
            .ok_or_else(invalid_internal_data)?
            .handle(),
    };
    let start = mappings.partition_point(|mapping| mapping.source() < source);
    let end = mappings.partition_point(|mapping| mapping.source() <= source);
    match mappings.get(start..end).ok_or_else(invalid_internal_data)? {
        [] => Ok(DxfEntityXDataHandleRemapState::Unmapped { source }),
        [mapping] => Ok(DxfEntityXDataHandleRemapState::Mapped {
            source,
            target: mapping.target(),
        }),
        multiple => Ok(DxfEntityXDataHandleRemapState::AmbiguousMapping {
            source,
            candidate_count: compact_len(multiple.len())?,
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

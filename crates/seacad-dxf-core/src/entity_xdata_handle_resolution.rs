//! Document-local target resolution for generic entity XDATA group-1005 values.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityXDataTypedDirectory,
    DxfEntityXDataTypedEntry, DxfEntityXDataValue, DxfError, DxfHandleIdentityMatch,
    DxfHandleResolutionDirectory, DxfHandleResolutionEntry, DxfHandleResolutionState,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// Compact resolution result for one retained XDATA group-1005 occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataHandleResolutionEntry {
    ordinal: u32,
    typed_ordinal: u32,
    state: DxfHandleResolutionState,
    target: Option<DxfHandleIdentityMatch>,
}

impl DxfEntityXDataHandleResolutionEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn typed_ordinal(self) -> u64 {
        self.typed_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfHandleResolutionState {
        self.state
    }

    #[must_use]
    pub const fn target(self) -> Option<DxfHandleIdentityMatch> {
        self.target
    }
}

/// Source-bound XDATA group-1005 projection over the generic handle index.
#[derive(Debug)]
pub struct DxfEntityXDataHandleResolutionDirectory {
    source_id: DxfSourceId,
    typed: DxfEntityXDataTypedDirectory,
    handles: DxfHandleResolutionDirectory,
    entries: Box<[DxfEntityXDataHandleResolutionEntry]>,
}

impl DxfEntityXDataHandleResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let typed = document.entity_xdata_typed_directory(cancellation)?;
        let handles = document.handle_resolution_directory(cancellation)?;
        ensure_source(document.source_id(), typed.source_id())?;
        ensure_source(document.source_id(), handles.source_id())?;

        let count = typed
            .entries()
            .iter()
            .filter(|entry| entry.occurrence().group().group_code().value() == 1005)
            .count();
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| out_of_memory())?;
        for typed_entry in typed.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if typed_entry.occurrence().group().group_code().value() != 1005 {
                continue;
            }
            let generic = handles
                .resolution_for_group(typed_entry.occurrence().group().occurrence())
                .ok_or_else(invalid_internal_data)?;
            verify_typed_state(typed_entry, generic)?;
            entries.push(DxfEntityXDataHandleResolutionEntry {
                ordinal: compact_len(entries.len())?,
                typed_ordinal: compact_u64(typed_entry.ordinal())?,
                state: generic.state(),
                target: unique_target(&handles, generic)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            typed,
            handles,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn typed_directory(&self) -> &DxfEntityXDataTypedDirectory {
        &self.typed
    }

    #[must_use]
    pub const fn handle_resolution_directory(&self) -> &DxfHandleResolutionDirectory {
        &self.handles
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataHandleResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataHandleResolutionEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_typed(
        &self,
        typed: DxfEntityXDataTypedEntry,
    ) -> Option<DxfEntityXDataHandleResolutionEntry> {
        if self.typed.entry(typed.ordinal()) != Some(typed) {
            return None;
        }
        let index = self
            .entries
            .binary_search_by_key(&typed.ordinal(), |entry| entry.typed_ordinal())
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn typed_for_entry(
        &self,
        entry: DxfEntityXDataHandleResolutionEntry,
    ) -> Option<DxfEntityXDataTypedEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.typed.entry(entry.typed_ordinal())
    }

    #[must_use]
    pub fn generic_for_entry(
        &self,
        entry: DxfEntityXDataHandleResolutionEntry,
    ) -> Option<DxfHandleResolutionEntry> {
        let typed = self.typed_for_entry(entry)?;
        self.handles
            .resolution_for_group(typed.occurrence().group().occurrence())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_handle_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleResolutionDirectory, DxfError> {
        DxfEntityXDataHandleResolutionDirectory::from_document(self, cancellation)
    }
}

macro_rules! document_handle_resolution_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_handle_resolution_directory(
                &self,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataHandleResolutionDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_handle_resolution_directory(cancellation)
            }
        }
    };
}

document_handle_resolution_directory!(DxfAsciiRawDocument<'_>);
document_handle_resolution_directory!(DxfBinaryRawDocument<'_>);

fn verify_typed_state(
    typed: DxfEntityXDataTypedEntry,
    generic: DxfHandleResolutionEntry,
) -> Result<(), DxfError> {
    match (typed.value(), generic.state()) {
        (DxfEntityXDataValue::Handle { value, .. }, DxfHandleResolutionState::Null)
            if value.is_null() =>
        {
            Ok(())
        }
        (DxfEntityXDataValue::Handle { value, .. }, state)
            if !value.is_null()
                && matches!(
                    state,
                    DxfHandleResolutionState::Missing
                        | DxfHandleResolutionState::Unique
                        | DxfHandleResolutionState::Ambiguous { .. }
                ) =>
        {
            Ok(())
        }
        (
            DxfEntityXDataValue::Invalid {
                issue: crate::DxfEntityXDataValueIssue::InvalidHandle(typed_issue),
                ..
            },
            DxfHandleResolutionState::Invalid(generic_issue),
        ) if typed_issue == generic_issue => Ok(()),
        _ => Err(invalid_internal_data()),
    }
}

fn unique_target(
    handles: &DxfHandleResolutionDirectory,
    generic: DxfHandleResolutionEntry,
) -> Result<Option<DxfHandleIdentityMatch>, DxfError> {
    if generic.state() != DxfHandleResolutionState::Unique {
        return Ok(None);
    }
    let handle = generic
        .reference()
        .value()
        .parse_result()
        .map_err(|_| invalid_internal_data())?;
    let [target] = handles.identity_directory().matches_for_handle(handle) else {
        return Err(invalid_internal_data());
    };
    Ok(Some(*target))
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

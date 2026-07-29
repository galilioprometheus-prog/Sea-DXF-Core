//! Typed target resolution over record identities and pointer/owner occurrences.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHandleIdentityDirectory, DxfHandleIdentityMatch, DxfHandleParseIssue,
    DxfHandleReferenceDirectory, DxfHandleReferenceEntry, DxfIoOperation, DxfRawDocumentView,
    DxfSourceId,
};

/// Target state for one record-scoped pointer or owner occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleResolutionState {
    Invalid(DxfHandleParseIssue),
    Null,
    Missing,
    Unique,
    Ambiguous { target_count: u32 },
}

/// One pointer or owner occurrence and its document-local target state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandleResolutionEntry {
    reference: DxfHandleReferenceEntry,
    state: DxfHandleResolutionState,
}

impl DxfHandleResolutionEntry {
    #[must_use]
    pub const fn reference(self) -> DxfHandleReferenceEntry {
        self.reference
    }

    #[must_use]
    pub const fn state(self) -> DxfHandleResolutionState {
        self.state
    }
}

/// Immutable, linear-storage resolution directory for pointer and owner handles.
///
/// Identity and reference evidence remains available without copying target
/// lists per reference. Resolution is document-local and makes no graph,
/// ownership-validity, purge, clone, INSERT, or XREF claim.
#[derive(Debug)]
pub struct DxfHandleResolutionDirectory {
    source_id: DxfSourceId,
    identities: DxfHandleIdentityDirectory,
    references: DxfHandleReferenceDirectory,
    entries: Box<[DxfHandleResolutionEntry]>,
}

impl DxfHandleResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let identities = document.handle_identity_directory(cancellation)?;
        let references = document.handle_reference_directory(cancellation)?;
        if identities.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: identities.source_id(),
            });
        }
        if references.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: references.source_id(),
            });
        }

        let mut entries = Vec::new();
        entries
            .try_reserve(references.entries().len())
            .map_err(|_| out_of_memory())?;
        for reference in references.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = match reference.value().parse_result() {
                Err(issue) => DxfHandleResolutionState::Invalid(issue),
                Ok(handle) if handle.is_null() => DxfHandleResolutionState::Null,
                Ok(handle) => match identities.matches_for_handle(handle) {
                    [] => DxfHandleResolutionState::Missing,
                    [_] => DxfHandleResolutionState::Unique,
                    targets => DxfHandleResolutionState::Ambiguous {
                        target_count: compact_len(targets.len())?,
                    },
                },
            };
            entries.push(DxfHandleResolutionEntry { reference, state });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            identities,
            references,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn identity_directory(&self) -> &DxfHandleIdentityDirectory {
        &self.identities
    }

    #[must_use]
    pub const fn reference_directory(&self) -> &DxfHandleReferenceDirectory {
        &self.references
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHandleResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, reference_ordinal: u64) -> Option<DxfHandleResolutionEntry> {
        let index = usize::try_from(reference_ordinal).ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn targets_for_reference(
        &self,
        reference_ordinal: u64,
    ) -> Option<&[DxfHandleIdentityMatch]> {
        let entry = self.entry(reference_ordinal)?;
        let handle = match entry.reference().value().parse_result() {
            Ok(handle) if !handle.is_null() => handle,
            Ok(_) | Err(_) => return Some(&[]),
        };
        Some(self.identities.matches_for_handle(handle))
    }

    #[must_use]
    pub fn resolution_for_group(&self, occurrence: u64) -> Option<DxfHandleResolutionEntry> {
        let index = self
            .entries
            .partition_point(|entry| entry.reference().value().group().occurrence() < occurrence);
        self.entries
            .get(index)
            .copied()
            .filter(|entry| entry.reference().value().group().occurrence() == occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    /// Resolves pointer and owner handles against uniquely parsed record identities.
    pub fn handle_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleResolutionDirectory, DxfError> {
        DxfHandleResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn handle_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn handle_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_resolution_directory(cancellation)
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
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

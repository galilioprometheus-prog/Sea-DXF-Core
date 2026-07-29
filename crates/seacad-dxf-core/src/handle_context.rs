//! Application-group context joined to every resolved handle reference.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfApplicationGroupEntry, DxfApplicationGroupKind,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHandleResolutionDirectory, DxfHandleResolutionEntry, DxfIoOperation, DxfRawDocumentView,
    DxfSourceId,
};

/// Lexical application-group context for one pointer or owner occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleReferenceContext {
    OutsideApplicationGroup,
    AcadReactors,
    AcadXDictionary,
    OtherApplicationGroup,
}

/// One resolved handle reference and its optional application-group container.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfContextualHandleReferenceEntry {
    resolution: DxfHandleResolutionEntry,
    application_group_ordinal: Option<u32>,
    context: DxfHandleReferenceContext,
}

impl DxfContextualHandleReferenceEntry {
    #[must_use]
    pub const fn resolution(self) -> DxfHandleResolutionEntry {
        self.resolution
    }

    #[must_use]
    pub const fn application_group_ordinal(self) -> Option<u64> {
        match self.application_group_ordinal {
            Some(ordinal) => Some(ordinal as u64),
            None => None,
        }
    }

    #[must_use]
    pub const fn context(self) -> DxfHandleReferenceContext {
        self.context
    }
}

/// Immutable source-order join of handle resolution and application groups.
///
/// Every M7.3b resolution has exactly one entry. Context is lexical only and
/// does not change the numeric reference class or target-resolution state.
#[derive(Debug)]
pub struct DxfContextualHandleReferenceDirectory {
    source_id: DxfSourceId,
    resolutions: DxfHandleResolutionDirectory,
    application_groups: DxfApplicationGroupDirectory,
    entries: Box<[DxfContextualHandleReferenceEntry]>,
}

impl DxfContextualHandleReferenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let resolutions = document.handle_resolution_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        for observed in [resolutions.source_id(), application_groups.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        entries
            .try_reserve(resolutions.entries().len())
            .map_err(|_| out_of_memory())?;
        for resolution in resolutions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let occurrence = resolution.reference().value().group().occurrence();
            let application_group_ordinal =
                application_group_ordinal_for_occurrence(&application_groups, occurrence)?;
            let context = match application_group_ordinal {
                None => DxfHandleReferenceContext::OutsideApplicationGroup,
                Some(ordinal) => match application_groups
                    .group(ordinal as u64)
                    .ok_or_else(invalid_internal_data)?
                    .kind()
                {
                    DxfApplicationGroupKind::AcadReactors => {
                        DxfHandleReferenceContext::AcadReactors
                    }
                    DxfApplicationGroupKind::AcadXDictionary => {
                        DxfHandleReferenceContext::AcadXDictionary
                    }
                    DxfApplicationGroupKind::Other => {
                        DxfHandleReferenceContext::OtherApplicationGroup
                    }
                },
            };
            entries.push(DxfContextualHandleReferenceEntry {
                resolution,
                application_group_ordinal,
                context,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            resolutions,
            application_groups,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn resolution_directory(&self) -> &DxfHandleResolutionDirectory {
        &self.resolutions
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        &self.application_groups
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfContextualHandleReferenceEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, reference_ordinal: u64) -> Option<DxfContextualHandleReferenceEntry> {
        let index = usize::try_from(reference_ordinal).ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_record(
        &self,
        record_ordinal: u64,
    ) -> Option<&[DxfContextualHandleReferenceEntry]> {
        self.resolutions
            .identity_directory()
            .entry(record_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry.resolution().reference().record().ordinal() < record_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry.resolution().reference().record().ordinal() <= record_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn entry_for_group(&self, occurrence: u64) -> Option<DxfContextualHandleReferenceEntry> {
        let index = self.entries.partition_point(|entry| {
            entry.resolution().reference().value().group().occurrence() < occurrence
        });
        self.entries.get(index).copied().filter(|entry| {
            entry.resolution().reference().value().group().occurrence() == occurrence
        })
    }

    #[must_use]
    pub fn application_group_for_entry(
        &self,
        reference_ordinal: u64,
    ) -> Option<DxfApplicationGroupEntry> {
        let ordinal = self.entry(reference_ordinal)?.application_group_ordinal()?;
        self.application_groups.group(ordinal)
    }
}

impl DxfRawDocumentView<'_> {
    /// Joins every pointer/owner resolution to its lexical application group.
    pub fn contextual_handle_reference_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfContextualHandleReferenceDirectory, DxfError> {
        DxfContextualHandleReferenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn contextual_handle_reference_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfContextualHandleReferenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).contextual_handle_reference_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn contextual_handle_reference_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfContextualHandleReferenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).contextual_handle_reference_directory(cancellation)
    }
}

fn application_group_ordinal_for_occurrence(
    directory: &DxfApplicationGroupDirectory,
    occurrence: u64,
) -> Result<Option<u32>, DxfError> {
    let index = directory
        .groups()
        .partition_point(|entry| entry.content_group_range().end() <= occurrence);
    let Some(entry) = directory.groups().get(index).copied() else {
        return Ok(None);
    };
    if occurrence < entry.content_group_range().start()
        || occurrence >= entry.content_group_range().end()
    {
        return Ok(None);
    }
    Ok(Some(compact_len(index)?))
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

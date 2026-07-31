//! Document-local resolution for exact MTEXT linked-column XDATA evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandleParseIssue,
    DxfHandleResolutionDirectory, DxfHandleResolutionState, DxfIoOperation,
    DxfMTextXDataColumnDirectory, DxfMTextXDataColumnEntry, DxfMTextXDataLinkedColumnDirectory,
    DxfMTextXDataLinkedColumnEntry, DxfRawDocumentView, DxfRawHandleValue, DxfRawRecord,
    DxfSourceId,
};

/// Document-local target classification for one linked-column handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextXDataLinkedColumnTargetState {
    Invalid(DxfHandleParseIssue),
    Null,
    Missing,
    Ambiguous { target_count: u32 },
    UniqueMText,
    UniqueOtherRecord,
}

/// One source handle, its resolution state, and optional unique target record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextXDataLinkedColumnTarget {
    handle: DxfRawHandleValue,
    state: DxfMTextXDataLinkedColumnTargetState,
    target_record: Option<DxfRawRecord>,
}

impl DxfMTextXDataLinkedColumnTarget {
    #[must_use]
    pub const fn handle(self) -> DxfRawHandleValue {
        self.handle
    }

    #[must_use]
    pub const fn state(self) -> DxfMTextXDataLinkedColumnTargetState {
        self.state
    }

    #[must_use]
    pub const fn target_record(self) -> Option<DxfRawRecord> {
        self.target_record
    }
}

/// One linked-column block, optional preceding column-info block, and targets.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextXDataLinkedColumnResolutionEntry {
    linked: DxfMTextXDataLinkedColumnEntry,
    column_info: Option<DxfMTextXDataColumnEntry>,
    target_start: u32,
    target_end: u32,
}

impl DxfMTextXDataLinkedColumnResolutionEntry {
    #[must_use]
    pub const fn linked(self) -> DxfMTextXDataLinkedColumnEntry {
        self.linked
    }

    #[must_use]
    pub const fn column_info(self) -> Option<DxfMTextXDataColumnEntry> {
        self.column_info
    }

    #[must_use]
    pub const fn target_count(self) -> u64 {
        (self.target_end - self.target_start) as u64
    }
}

/// Linked-column evidence plus document-local generic handle resolution.
#[derive(Debug)]
pub struct DxfMTextXDataLinkedColumnResolutionDirectory {
    source_id: DxfSourceId,
    column_info: DxfMTextXDataColumnDirectory,
    linked: DxfMTextXDataLinkedColumnDirectory,
    handle_resolution: DxfHandleResolutionDirectory,
    entries: Box<[DxfMTextXDataLinkedColumnResolutionEntry]>,
    targets: Box<[DxfMTextXDataLinkedColumnTarget]>,
}

impl DxfMTextXDataLinkedColumnResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let column_info = document.mtext_xdata_column_directory(cancellation)?;
        let linked = document.mtext_xdata_linked_column_directory(cancellation)?;
        let handle_resolution = document.handle_resolution_directory(cancellation)?;
        for observed in [
            column_info.source_id(),
            linked.source_id(),
            handle_resolution.source_id(),
        ] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        let mut targets = Vec::new();
        entries
            .try_reserve(linked.entries().len())
            .map_err(|_| out_of_memory())?;
        for linked_entry in linked.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let target_start = compact_len(targets.len())?;
            let handles = linked
                .handles_for_entry(linked_entry)
                .ok_or_else(invalid_internal_data)?;
            for handle in handles.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                targets.try_reserve(1).map_err(|_| out_of_memory())?;
                targets.push(resolve_target(document, &handle_resolution, handle)?);
            }
            entries.push(DxfMTextXDataLinkedColumnResolutionEntry {
                linked: linked_entry,
                column_info: preceding_column_info(&column_info, linked_entry),
                target_start,
                target_end: compact_len(targets.len())?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            column_info,
            linked,
            handle_resolution,
            entries: entries.into_boxed_slice(),
            targets: targets.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn column_info_directory(&self) -> &DxfMTextXDataColumnDirectory {
        &self.column_info
    }

    #[must_use]
    pub const fn linked_directory(&self) -> &DxfMTextXDataLinkedColumnDirectory {
        &self.linked
    }

    #[must_use]
    pub const fn handle_resolution_directory(&self) -> &DxfHandleResolutionDirectory {
        &self.handle_resolution
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfMTextXDataLinkedColumnResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn targets(&self) -> &[DxfMTextXDataLinkedColumnTarget] {
        &self.targets
    }

    #[must_use]
    pub fn targets_for_entry(
        &self,
        entry: DxfMTextXDataLinkedColumnResolutionEntry,
    ) -> Option<&[DxfMTextXDataLinkedColumnTarget]> {
        self.targets.get(
            usize::try_from(entry.target_start).ok()?..usize::try_from(entry.target_end).ok()?,
        )
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_xdata_linked_column_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataLinkedColumnResolutionDirectory, DxfError> {
        DxfMTextXDataLinkedColumnResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_xdata_linked_column_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataLinkedColumnResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_xdata_linked_column_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_xdata_linked_column_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataLinkedColumnResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_xdata_linked_column_resolution_directory(cancellation)
    }
}

fn preceding_column_info(
    directory: &DxfMTextXDataColumnDirectory,
    linked: DxfMTextXDataLinkedColumnEntry,
) -> Option<DxfMTextXDataColumnEntry> {
    let occurrence = linked.begin().occurrence();
    let end = directory
        .entries()
        .partition_point(|entry| entry.begin().occurrence() < occurrence);
    end.checked_sub(1)
        .and_then(|index| directory.entries().get(index))
        .copied()
        .filter(|entry| entry.record() == linked.record() && entry.end().occurrence() < occurrence)
}

fn resolve_target(
    document: DxfRawDocumentView<'_>,
    resolution: &DxfHandleResolutionDirectory,
    handle: DxfRawHandleValue,
) -> Result<DxfMTextXDataLinkedColumnTarget, DxfError> {
    let generic = resolution
        .resolution_for_group(handle.group().occurrence())
        .ok_or_else(invalid_internal_data)?;
    let (state, target_record) = match generic.state() {
        DxfHandleResolutionState::Invalid(issue) => {
            (DxfMTextXDataLinkedColumnTargetState::Invalid(issue), None)
        }
        DxfHandleResolutionState::Null => (DxfMTextXDataLinkedColumnTargetState::Null, None),
        DxfHandleResolutionState::Missing => (DxfMTextXDataLinkedColumnTargetState::Missing, None),
        DxfHandleResolutionState::Ambiguous { target_count } => (
            DxfMTextXDataLinkedColumnTargetState::Ambiguous { target_count },
            None,
        ),
        DxfHandleResolutionState::Unique => {
            let parsed = handle.parse_result().map_err(|_| invalid_internal_data())?;
            let [target] = resolution.identity_directory().matches_for_handle(parsed) else {
                return Err(invalid_internal_data());
            };
            let target_record = target.record();
            let marker = document
                .group(target_record.marker_occurrence())
                .ok_or_else(invalid_internal_data)?;
            let is_mtext = document.raw_span_equals_exact(marker.value_payload_span(), b"MTEXT")?;
            (
                if is_mtext {
                    DxfMTextXDataLinkedColumnTargetState::UniqueMText
                } else {
                    DxfMTextXDataLinkedColumnTargetState::UniqueOtherRecord
                },
                Some(target_record),
            )
        }
    };
    Ok(DxfMTextXDataLinkedColumnTarget {
        handle,
        state,
        target_record,
    })
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

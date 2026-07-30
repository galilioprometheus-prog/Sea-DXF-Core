//! Exact INSERT block-name resolution over matched BLOCK definitions.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockNameIndexDirectory, DxfBlockNameIndexMatch,
    DxfCancellationToken, DxfError, DxfInsertRecordSemanticDirectory, DxfInsertRecordValueEntry,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// Exact resolution outcome for one INSERT block-name field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertBlockResolutionState {
    UnusableName,
    Missing,
    Unique,
    Ambiguous { target_count: u32 },
}

/// Half-open range in the directory's retained target-match array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertBlockTargetRange {
    start: u32,
    end: u32,
}

impl DxfInsertBlockTargetRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// One exact INSERT and its duplicate-preserving BLOCK resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertBlockResolutionEntry {
    insert: DxfInsertRecordValueEntry,
    target_range: DxfInsertBlockTargetRange,
    state: DxfInsertBlockResolutionState,
}

impl DxfInsertBlockResolutionEntry {
    #[must_use]
    pub const fn insert(self) -> DxfInsertRecordValueEntry {
        self.insert
    }

    #[must_use]
    pub const fn target_range(self) -> DxfInsertBlockTargetRange {
        self.target_range
    }

    #[must_use]
    pub const fn state(self) -> DxfInsertBlockResolutionState {
        self.state
    }
}

/// Immutable exact INSERT-to-BLOCK resolution evidence.
#[derive(Debug)]
pub struct DxfInsertBlockResolutionDirectory {
    source_id: DxfSourceId,
    insert_semantics: DxfInsertRecordSemanticDirectory,
    block_names: DxfBlockNameIndexDirectory,
    entries: Box<[DxfInsertBlockResolutionEntry]>,
    targets: Box<[DxfBlockNameIndexMatch]>,
}

impl DxfInsertBlockResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let insert_semantics = document.insert_record_semantic_directory(cancellation)?;
        let block_names = document.block_name_index_directory(cancellation)?;
        for observed in [insert_semantics.source_id(), block_names.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        let mut targets = Vec::new();
        for insert in insert_semantics.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let semantics = insert_semantics
                .semantics_for_entry(insert)?
                .ok_or_else(invalid_internal_data)?;
            let target_start = compact_len(targets.len())?;
            let state = if let Some(name) = semantics.block_name().value().copied() {
                if name.source_id() != document.source_id() {
                    return Err(DxfError::SourceIdentityMismatch {
                        expected: document.source_id(),
                        observed: name.source_id(),
                    });
                }
                let matches = block_names.matches_for_exact_source_span(
                    document,
                    name.value_span(),
                    cancellation,
                )?;
                targets
                    .try_reserve(matches.len())
                    .map_err(|_| out_of_memory())?;
                targets.extend_from_slice(matches);
                match matches {
                    [] => DxfInsertBlockResolutionState::Missing,
                    [_] => DxfInsertBlockResolutionState::Unique,
                    multiple => DxfInsertBlockResolutionState::Ambiguous {
                        target_count: compact_len(multiple.len())?,
                    },
                }
            } else {
                DxfInsertBlockResolutionState::UnusableName
            };
            let target_end = compact_len(targets.len())?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfInsertBlockResolutionEntry {
                insert,
                target_range: DxfInsertBlockTargetRange::new(target_start, target_end)?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            insert_semantics,
            block_names,
            entries: entries.into_boxed_slice(),
            targets: targets.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn insert_semantic_directory(&self) -> &DxfInsertRecordSemanticDirectory {
        &self.insert_semantics
    }

    #[must_use]
    pub const fn block_name_index_directory(&self) -> &DxfBlockNameIndexDirectory {
        &self.block_names
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfInsertBlockResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn targets(&self) -> &[DxfBlockNameIndexMatch] {
        &self.targets
    }

    #[must_use]
    pub fn entry_for_insert_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertBlockResolutionEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.insert().record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn targets_for_insert_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBlockNameIndexMatch]> {
        let entry = self.entry_for_insert_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.target_range().start()).ok()?;
        let end = usize::try_from(entry.target_range().end()).ok()?;
        self.targets.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_block_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertBlockResolutionDirectory, DxfError> {
        DxfInsertBlockResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_block_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertBlockResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_block_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_block_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertBlockResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_block_resolution_directory(cancellation)
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

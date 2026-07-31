//! Exact primary/secondary BLOCK-name consistency evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockRecordSemanticDirectory,
    DxfBlockRecordSemantics, DxfBlockRecordValueEntry, DxfCancellationToken, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId, source_span::spans_equal,
};

/// Exact relationship between one BLOCK record's groups `2` and `3`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockNameConsistencyState {
    Matched,
    Conflicting,
    NotComparable,
}

/// One BLOCK record and its exact primary/secondary name comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockNameConsistencyEntry {
    record: DxfBlockRecordValueEntry,
    state: DxfBlockNameConsistencyState,
}

impl DxfBlockNameConsistencyEntry {
    #[must_use]
    pub const fn record(self) -> DxfBlockRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn state(self) -> DxfBlockNameConsistencyState {
        self.state
    }
}

/// Immutable exact-name consistency directory retaining M10.1d semantics.
#[derive(Debug)]
pub struct DxfBlockNameConsistencyDirectory {
    source_id: DxfSourceId,
    semantics: DxfBlockRecordSemanticDirectory,
    entries: Box<[DxfBlockNameConsistencyEntry]>,
}

impl DxfBlockNameConsistencyDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.block_record_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }

        let mut entries = Vec::new();
        for record in semantics.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let projected = semantics
                .semantics_for_entry(record)?
                .ok_or_else(invalid_internal_data)?;
            let state = compare_names(document, &projected, cancellation)?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfBlockNameConsistencyEntry { record, state });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfBlockRecordSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfBlockNameConsistencyEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfBlockNameConsistencyEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.record().definition().block_record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_name_consistency_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockNameConsistencyDirectory, DxfError> {
        DxfBlockNameConsistencyDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_name_consistency_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockNameConsistencyDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_name_consistency_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_name_consistency_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockNameConsistencyDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_name_consistency_directory(cancellation)
    }
}

fn compare_names(
    document: DxfRawDocumentView<'_>,
    semantics: &DxfBlockRecordSemantics,
    cancellation: &DxfCancellationToken,
) -> Result<DxfBlockNameConsistencyState, DxfError> {
    let (Some(primary), Some(secondary)) = (
        semantics.primary_name().value().copied(),
        semantics.secondary_name().value().copied(),
    ) else {
        return Ok(DxfBlockNameConsistencyState::NotComparable);
    };
    for observed in [primary.source_id(), secondary.source_id()] {
        if observed != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed,
            });
        }
    }
    if spans_equal(
        document,
        primary.value_span(),
        secondary.value_span(),
        cancellation,
    )? {
        Ok(DxfBlockNameConsistencyState::Matched)
    } else {
        Ok(DxfBlockNameConsistencyState::Conflicting)
    }
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

//! Exact TOLERANCE group-3 to DIMSTYLE table-record resolution.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDimStyleTableDirectory,
    DxfDimStyleTableEntry, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
    DxfTextSymbolKind, DxfTextSymbolRecordEntry, DxfTextSymbolTextDirectory,
    source_span::spans_equal,
};

/// Exact resolution outcome for one TOLERANCE dimension-style name.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfToleranceDimStyleResolutionState {
    UnusableName,
    Missing,
    Unique,
    Ambiguous { target_count: u32 },
}

/// Half-open range in the resolution directory's retained target array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfToleranceDimStyleTargetRange {
    start: u32,
    end: u32,
}

impl DxfToleranceDimStyleTargetRange {
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

/// One TOLERANCE record and its exact DIMSTYLE target disposition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfToleranceDimStyleResolutionEntry {
    tolerance: DxfTextSymbolRecordEntry,
    target_range: DxfToleranceDimStyleTargetRange,
    state: DxfToleranceDimStyleResolutionState,
}

impl DxfToleranceDimStyleResolutionEntry {
    #[must_use]
    pub const fn tolerance(self) -> DxfTextSymbolRecordEntry {
        self.tolerance
    }

    #[must_use]
    pub const fn target_range(self) -> DxfToleranceDimStyleTargetRange {
        self.target_range
    }

    #[must_use]
    pub const fn state(self) -> DxfToleranceDimStyleResolutionState {
        self.state
    }
}

/// Immutable exact TOLERANCE-to-DIMSTYLE resolution evidence.
#[derive(Debug)]
pub struct DxfToleranceDimStyleResolutionDirectory {
    source_id: DxfSourceId,
    fields: DxfTextSymbolTextDirectory,
    dimstyles: DxfDimStyleTableDirectory,
    entries: Box<[DxfToleranceDimStyleResolutionEntry]>,
    targets: Box<[DxfDimStyleTableEntry]>,
}

impl DxfToleranceDimStyleResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let fields = document.text_symbol_text_directory(cancellation)?;
        let dimstyles = document.dimstyle_table_directory(cancellation)?;
        for observed in [fields.source_id(), dimstyles.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        let mut targets = Vec::new();
        for tolerance in fields.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if tolerance.kind() != DxfTextSymbolKind::Tolerance {
                continue;
            }
            let semantics = fields
                .tolerance_semantics_for_record(tolerance)?
                .ok_or_else(invalid_internal_data)?;
            let start = compact_len(targets.len())?;
            let state = if let Some(name) = semantics.dimension_style_name().value().copied() {
                let span = name.group().value_payload_span();
                for target in dimstyles.entries().iter().copied() {
                    if spans_equal(document, span, target.name().value_span(), cancellation)? {
                        targets.try_reserve(1).map_err(|_| out_of_memory())?;
                        targets.push(target);
                    }
                }
                let count = compact_len(targets.len())?
                    .checked_sub(start)
                    .ok_or_else(invalid_internal_data)?;
                match count {
                    0 => DxfToleranceDimStyleResolutionState::Missing,
                    1 => DxfToleranceDimStyleResolutionState::Unique,
                    target_count => DxfToleranceDimStyleResolutionState::Ambiguous { target_count },
                }
            } else {
                DxfToleranceDimStyleResolutionState::UnusableName
            };
            let end = compact_len(targets.len())?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfToleranceDimStyleResolutionEntry {
                tolerance,
                target_range: DxfToleranceDimStyleTargetRange::new(start, end)?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            fields,
            dimstyles,
            entries: entries.into_boxed_slice(),
            targets: targets.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn field_directory(&self) -> &DxfTextSymbolTextDirectory {
        &self.fields
    }

    #[must_use]
    pub const fn dimstyle_table_directory(&self) -> &DxfDimStyleTableDirectory {
        &self.dimstyles
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfToleranceDimStyleResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn targets(&self) -> &[DxfDimStyleTableEntry] {
        &self.targets
    }

    #[must_use]
    pub fn entry_for_tolerance_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfToleranceDimStyleResolutionEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.tolerance().record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn targets_for_tolerance_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfDimStyleTableEntry]> {
        let entry = self.entry_for_tolerance_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.target_range().start()).ok()?;
        let end = usize::try_from(entry.target_range().end()).ok()?;
        self.targets.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn tolerance_dimstyle_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfToleranceDimStyleResolutionDirectory, DxfError> {
        DxfToleranceDimStyleResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn tolerance_dimstyle_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfToleranceDimStyleResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).tolerance_dimstyle_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn tolerance_dimstyle_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfToleranceDimStyleResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).tolerance_dimstyle_resolution_directory(cancellation)
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

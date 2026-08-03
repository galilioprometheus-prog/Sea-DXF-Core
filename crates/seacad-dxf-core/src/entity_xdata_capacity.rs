//! AutoCAD-compatible, fail-closed entity XDATA capacity accounting.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataTypedDirectory, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// Maximum XDATA storage reported by `xdroom` for an otherwise empty entity.
pub const DXF_XDATA_ENTITY_CAPACITY_BYTES: u64 = 16_383;

/// Why a validated group-1000 string has no exact capacity contribution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataCapacityTextIssue {
    EncodingUnavailable,
    StorageMalformed,
    StorageOutputLimit,
    EscapeMalformed,
    EscapeOutputLimit,
}

/// One reason an entity's exact AutoCAD XDATA size cannot be established.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataCapacityIssueKind {
    OrphanValue {
        source_entry_ordinal: u32,
    },
    ApplicationResolution {
        application_ordinal: u32,
        target_count: u32,
    },
    InvalidApplicationStructure {
        application_ordinal: u32,
        issue_count: u32,
    },
    InvalidValue {
        source_entry_ordinal: u32,
    },
    PartialPointTuple {
        tuple_ordinal: u32,
    },
    LayerResolution {
        source_entry_ordinal: u32,
        target_count: u32,
    },
    Text {
        source_entry_ordinal: u32,
        issue: DxfEntityXDataCapacityTextIssue,
    },
}

/// One compact, entity-bound capacity issue in deterministic source order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataCapacityIssue {
    kind: DxfEntityXDataCapacityIssueKind,
}

impl DxfEntityXDataCapacityIssue {
    pub(crate) const fn new(kind: DxfEntityXDataCapacityIssueKind) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(self) -> DxfEntityXDataCapacityIssueKind {
        self.kind
    }
}

/// Half-open range in the directory's capacity-issue array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataCapacityIssueRange {
    start: u32,
    end: u32,
}

impl DxfEntityXDataCapacityIssueRange {
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

/// Exact capacity relation, or a fail-closed accounted lower bound.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataCapacityState {
    WithinLimit {
        used_bytes: u64,
        remaining_bytes: u64,
    },
    Exceeded {
        used_bytes: u64,
        excess_bytes: u64,
    },
    Indeterminate {
        accounted_bytes: u64,
        issue_count: u32,
    },
}

/// One indexed entity and its per-entity XDATA capacity result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataCapacityEntry {
    ordinal: u32,
    raw_record_ordinal: u32,
    state: DxfEntityXDataCapacityState,
    issues: DxfEntityXDataCapacityIssueRange,
}

impl DxfEntityXDataCapacityEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn raw_record_ordinal(self) -> u64 {
        self.raw_record_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataCapacityState {
        self.state
    }

    #[must_use]
    pub const fn issue_range(self) -> DxfEntityXDataCapacityIssueRange {
        self.issues
    }
}

/// Immutable capacity results for every indexed entity, including zero-XDATA entities.
#[derive(Debug)]
pub struct DxfEntityXDataCapacityDirectory {
    source_id: DxfSourceId,
    typed: DxfEntityXDataTypedDirectory,
    entries: Box<[DxfEntityXDataCapacityEntry]>,
    issues: Box<[DxfEntityXDataCapacityIssue]>,
}

impl DxfEntityXDataCapacityDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let typed = document.entity_xdata_typed_directory(cancellation)?;
        let appids = document.entity_xdata_appid_resolution_directory(cancellation)?;
        let structure = document.entity_xdata_structure_directory(cancellation)?;
        let tuples = document.entity_xdata_point_tuple_directory(cancellation)?;
        let layers = document.entity_xdata_layer_resolution_directory(cancellation)?;
        for observed in [
            typed.source_id(),
            appids.source_id(),
            structure.source_id(),
            tuples.source_id(),
            layers.source_id(),
        ] {
            ensure_source(document.source_id(), observed)?;
        }
        let entities = typed.xdata_directory().entity_directory().entities();
        let mut entries = Vec::new();
        let mut issues = Vec::new();
        entries
            .try_reserve_exact(entities.len())
            .map_err(|_| out_of_memory())?;
        for entity in entities.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let issue_start = compact_len(issues.len())?;
            let accounted = crate::entity_xdata_capacity_measure::measure_entity(
                document,
                &typed,
                &appids,
                &structure,
                &tuples,
                &layers,
                entity,
                cancellation,
                &mut issues,
            )?;
            let issue_end = compact_len(issues.len())?;
            let issue_count = issue_end
                .checked_sub(issue_start)
                .ok_or_else(invalid_internal_data)?;
            let state = if issue_count != 0 {
                DxfEntityXDataCapacityState::Indeterminate {
                    accounted_bytes: accounted,
                    issue_count,
                }
            } else if accounted <= DXF_XDATA_ENTITY_CAPACITY_BYTES {
                DxfEntityXDataCapacityState::WithinLimit {
                    used_bytes: accounted,
                    remaining_bytes: DXF_XDATA_ENTITY_CAPACITY_BYTES - accounted,
                }
            } else {
                DxfEntityXDataCapacityState::Exceeded {
                    used_bytes: accounted,
                    excess_bytes: accounted - DXF_XDATA_ENTITY_CAPACITY_BYTES,
                }
            };
            entries.push(DxfEntityXDataCapacityEntry {
                ordinal: compact_len(entries.len())?,
                raw_record_ordinal: compact_u64(entity.record().ordinal())?,
                state,
                issues: DxfEntityXDataCapacityIssueRange::new(issue_start, issue_end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            typed,
            entries: entries.into_boxed_slice(),
            issues: issues.into_boxed_slice(),
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
    pub fn entries(&self) -> &[DxfEntityXDataCapacityEntry] {
        &self.entries
    }

    #[must_use]
    pub fn issues(&self) -> &[DxfEntityXDataCapacityIssue] {
        &self.issues
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataCapacityEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entry_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<Option<DxfEntityXDataCapacityEntry>, DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        Ok(self
            .entries
            .binary_search_by_key(&entity.record().ordinal(), |entry| {
                entry.raw_record_ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied()))
    }

    pub fn entity_for_entry(
        &self,
        entry: DxfEntityXDataCapacityEntry,
    ) -> Result<DxfEntityRef, DxfError> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        self.typed
            .xdata_directory()
            .entity_directory()
            .entity_for_raw_ordinal(entry.raw_record_ordinal())
            .ok_or_else(invalid_internal_data)
    }

    pub fn issues_for_entry(
        &self,
        entry: DxfEntityXDataCapacityEntry,
    ) -> Result<&[DxfEntityXDataCapacityIssue], DxfError> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        let range = entry.issue_range();
        self.issues
            .get(
                usize::try_from(range.start()).map_err(|_| invalid_internal_data())?
                    ..usize::try_from(range.end()).map_err(|_| invalid_internal_data())?,
            )
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_capacity_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataCapacityDirectory, DxfError> {
        DxfEntityXDataCapacityDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_xdata_capacity_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataCapacityDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_capacity_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_xdata_capacity_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataCapacityDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_capacity_directory(cancellation)
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

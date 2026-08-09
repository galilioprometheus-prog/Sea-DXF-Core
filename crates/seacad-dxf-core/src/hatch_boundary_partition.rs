//! Fail-closed HATCH boundary-envelope partitioning over exact raw evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfFillMeshEvidenceDirectory, DxfFillMeshFamily, DxfFillMeshField, DxfFillMeshRange,
    DxfFillMeshSubclassEntry, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryPartitionIssue {
    BoundaryPathCountAbsent,
    BoundaryPathCountMultiple { count: u32 },
    HatchStyleAbsent,
    HatchStyleMultiple { count: u32 },
    AnchorOrderInvalid,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPartition {
    boundary_path_count: DxfFillMeshField,
    hatch_style: DxfFillMeshField,
    header_range: DxfFillMeshRange,
    boundary_range: DxfFillMeshRange,
    trailing_range: DxfFillMeshRange,
}

impl DxfHatchBoundaryPartition {
    #[must_use]
    pub const fn boundary_path_count(self) -> DxfFillMeshField {
        self.boundary_path_count
    }

    #[must_use]
    pub const fn hatch_style(self) -> DxfFillMeshField {
        self.hatch_style
    }

    /// Top-level fields through and including group 91.
    #[must_use]
    pub const fn header_range(self) -> DxfFillMeshRange {
        self.header_range
    }

    /// Opaque boundary-path payload after group 91 and before group 75.
    #[must_use]
    pub const fn boundary_range(self) -> DxfFillMeshRange {
        self.boundary_range
    }

    /// Top-level and later nested fields beginning with group 75.
    #[must_use]
    pub const fn trailing_range(self) -> DxfFillMeshRange {
        self.trailing_range
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPartitionEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    subclass: DxfFillMeshSubclassEntry,
    partition: Result<DxfHatchBoundaryPartition, DxfHatchBoundaryPartitionIssue>,
}

impl DxfHatchBoundaryPartitionEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn subclass(self) -> DxfFillMeshSubclassEntry {
        self.subclass
    }

    pub const fn partition(
        self,
    ) -> Result<DxfHatchBoundaryPartition, DxfHatchBoundaryPartitionIssue> {
        self.partition
    }
}

/// One boundary-envelope result per exact `AcDbHatch` subclass.
#[derive(Debug)]
pub struct DxfHatchBoundaryPartitionDirectory {
    source_id: DxfSourceId,
    evidence: DxfFillMeshEvidenceDirectory,
    entries: Box<[DxfHatchBoundaryPartitionEntry]>,
}

impl DxfHatchBoundaryPartitionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.fill_mesh_evidence_directory(cancellation)?;
        ensure_source(document.source_id(), evidence.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(evidence.subclasses().len())
            .map_err(|_| out_of_memory())?;

        for (subclass_index, subclass) in evidence.subclasses().iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            if subclass.family() != DxfFillMeshFamily::Hatch {
                continue;
            }
            let fields = evidence
                .fields_for_subclass(subclass_index as u64)
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfHatchBoundaryPartitionEntry {
                ordinal: compact_len(entries.len())?,
                subclass_ordinal: compact_len(subclass_index)?,
                subclass,
                partition: partition_for(subclass, fields)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            evidence,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn evidence_directory(&self) -> &DxfFillMeshEvidenceDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryPartitionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryPartitionEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_subclass(&self, ordinal: u64) -> Option<DxfHatchBoundaryPartitionEntry> {
        self.entries
            .binary_search_by_key(&ordinal, |entry| entry.subclass_ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchBoundaryPartitionEntry] {
        let start = self
            .entries
            .partition_point(|entry| entry.subclass().entity().record().ordinal() < raw);
        let end = self
            .entries
            .partition_point(|entry| entry.subclass().entity().record().ordinal() <= raw);
        self.entries.get(start..end).unwrap_or_default()
    }

    #[must_use]
    pub fn header_fields_for_subclass(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let partition = self.entry_for_subclass(ordinal)?.partition().ok()?;
        slice_for_range(self.evidence.fields(), partition.header_range())
    }

    #[must_use]
    pub fn boundary_fields_for_subclass(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let partition = self.entry_for_subclass(ordinal)?.partition().ok()?;
        slice_for_range(self.evidence.fields(), partition.boundary_range())
    }

    #[must_use]
    pub fn trailing_fields_for_subclass(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let partition = self.entry_for_subclass(ordinal)?.partition().ok()?;
        slice_for_range(self.evidence.fields(), partition.trailing_range())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_partition_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPartitionDirectory, DxfError> {
        DxfHatchBoundaryPartitionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_partition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPartitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_partition_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_partition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPartitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_partition_directory(cancellation)
    }
}

fn partition_for(
    subclass: DxfFillMeshSubclassEntry,
    fields: &[DxfFillMeshField],
) -> Result<Result<DxfHatchBoundaryPartition, DxfHatchBoundaryPartitionIssue>, DxfError> {
    let path_counts = anchor_indexes(fields, 91)?;
    let hatch_styles = anchor_indexes(fields, 75)?;
    let path_count = match only_anchor(path_counts, AnchorKind::BoundaryPathCount) {
        Ok(index) => index,
        Err(issue) => return Ok(Err(issue)),
    };
    let hatch_style = match only_anchor(hatch_styles, AnchorKind::HatchStyle) {
        Ok(index) => index,
        Err(issue) => return Ok(Err(issue)),
    };
    if path_count >= hatch_style {
        return Ok(Err(DxfHatchBoundaryPartitionIssue::AnchorOrderInvalid));
    }

    let start = compact_u64(subclass.field_range().start())?;
    let end = compact_u64(subclass.field_range().end())?;
    let path_count_global = compact_add(start, path_count)?;
    let hatch_style_global = compact_add(start, hatch_style)?;
    let boundary_start = path_count_global
        .checked_add(1)
        .ok_or_else(invalid_internal_data)?;
    let boundary_path_count = fields
        .get(path_count)
        .copied()
        .ok_or_else(invalid_internal_data)?;
    let hatch_style = fields
        .get(hatch_style)
        .copied()
        .ok_or_else(invalid_internal_data)?;
    Ok(Ok(DxfHatchBoundaryPartition {
        boundary_path_count,
        hatch_style,
        header_range: DxfFillMeshRange::new(start, boundary_start)?,
        boundary_range: DxfFillMeshRange::new(boundary_start, hatch_style_global)?,
        trailing_range: DxfFillMeshRange::new(hatch_style_global, end)?,
    }))
}

#[derive(Clone, Copy)]
enum AnchorKind {
    BoundaryPathCount,
    HatchStyle,
}

fn only_anchor(
    indexes: Vec<usize>,
    kind: AnchorKind,
) -> Result<usize, DxfHatchBoundaryPartitionIssue> {
    match indexes.as_slice() {
        [index] => Ok(*index),
        [] => Err(match kind {
            AnchorKind::BoundaryPathCount => {
                DxfHatchBoundaryPartitionIssue::BoundaryPathCountAbsent
            }
            AnchorKind::HatchStyle => DxfHatchBoundaryPartitionIssue::HatchStyleAbsent,
        }),
        values => {
            let count = u32::try_from(values.len()).unwrap_or(u32::MAX);
            Err(match kind {
                AnchorKind::BoundaryPathCount => {
                    DxfHatchBoundaryPartitionIssue::BoundaryPathCountMultiple { count }
                }
                AnchorKind::HatchStyle => {
                    DxfHatchBoundaryPartitionIssue::HatchStyleMultiple { count }
                }
            })
        }
    }
}

fn anchor_indexes(fields: &[DxfFillMeshField], code: i16) -> Result<Vec<usize>, DxfError> {
    let mut indexes = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        if field.group().group_code().value() == code {
            indexes.try_reserve(1).map_err(|_| out_of_memory())?;
            indexes.push(index);
        }
    }
    Ok(indexes)
}

fn slice_for_range<T>(values: &[T], range: DxfFillMeshRange) -> Option<&[T]> {
    let start = usize::try_from(range.start()).ok()?;
    let end = usize::try_from(range.end()).ok()?;
    values.get(start..end)
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn compact_u64(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

fn compact_add(base: u32, offset: usize) -> Result<u32, DxfError> {
    base.checked_add(compact_len(offset)?)
        .ok_or_else(invalid_internal_data)
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
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

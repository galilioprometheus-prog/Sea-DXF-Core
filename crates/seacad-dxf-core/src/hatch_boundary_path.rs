//! Conservative HATCH boundary-path grouping and declared-count comparison.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfFillMeshField, DxfFillMeshRange, DxfHatchBoundaryPartitionDirectory,
    DxfHatchBoundaryPartitionIssue, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    raw_integer::decode_raw_i32,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPathRange {
    start: u32,
    end: u32,
}

impl DxfHatchBoundaryPathRange {
    const fn new(start: u32, end: u32) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPathEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    subclass_path_ordinal: u32,
    marker: DxfFillMeshField,
    payload_range: DxfFillMeshRange,
}

impl DxfHatchBoundaryPathEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn subclass_path_ordinal(self) -> u64 {
        self.subclass_path_ordinal as u64
    }

    #[must_use]
    pub const fn marker(self) -> DxfFillMeshField {
        self.marker
    }

    #[must_use]
    pub const fn payload_range(self) -> DxfFillMeshRange {
        self.payload_range
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryPathCountIssue {
    InvalidAsciiNumber {
        group: DxfRawGroup,
        issue: DxfAsciiNumericIssue,
    },
    Negative {
        group: DxfRawGroup,
        value: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryPathCountRelation {
    Matched { count: u32 },
    Mismatched { declared: u32, observed: u32 },
    DeclaredUnavailable(DxfHatchBoundaryPathCountIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPathTopology {
    path_range: DxfHatchBoundaryPathRange,
    orphan_range: DxfFillMeshRange,
    count_relation: DxfHatchBoundaryPathCountRelation,
}

impl DxfHatchBoundaryPathTopology {
    #[must_use]
    pub const fn path_range(self) -> DxfHatchBoundaryPathRange {
        self.path_range
    }

    #[must_use]
    pub const fn path_count(self) -> u64 {
        self.path_range.len()
    }

    #[must_use]
    pub const fn orphan_range(self) -> DxfFillMeshRange {
        self.orphan_range
    }

    #[must_use]
    pub const fn orphan_field_count(self) -> u64 {
        self.orphan_range.len()
    }

    #[must_use]
    pub const fn count_relation(self) -> DxfHatchBoundaryPathCountRelation {
        self.count_relation
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryPathTopologyIssue {
    PartitionUnavailable(DxfHatchBoundaryPartitionIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPathTopologyEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    raw_record_ordinal: u32,
    topology: Result<DxfHatchBoundaryPathTopology, DxfHatchBoundaryPathTopologyIssue>,
}

impl DxfHatchBoundaryPathTopologyEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn raw_record_ordinal(self) -> u64 {
        self.raw_record_ordinal as u64
    }

    pub const fn topology(
        self,
    ) -> Result<DxfHatchBoundaryPathTopology, DxfHatchBoundaryPathTopologyIssue> {
        self.topology
    }
}

/// Boundary-path anchors, payload ranges, orphan fields, and count relations.
#[derive(Debug)]
pub struct DxfHatchBoundaryPathDirectory {
    source_id: DxfSourceId,
    partitions: DxfHatchBoundaryPartitionDirectory,
    entries: Box<[DxfHatchBoundaryPathTopologyEntry]>,
    paths: Box<[DxfHatchBoundaryPathEntry]>,
}

impl DxfHatchBoundaryPathDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let partitions = document.hatch_boundary_partition_directory(cancellation)?;
        ensure_source(document.source_id(), partitions.source_id())?;
        let mut entries = Vec::new();
        let mut paths = Vec::new();
        entries
            .try_reserve(partitions.entries().len())
            .map_err(|_| out_of_memory())?;

        for partition_entry in partitions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let topology = match partition_entry.partition() {
                Ok(partition) => Ok(group_paths(
                    document,
                    &partitions,
                    partition_entry.subclass_ordinal(),
                    partition,
                    cancellation,
                    &mut paths,
                )?),
                Err(issue) => Err(DxfHatchBoundaryPathTopologyIssue::PartitionUnavailable(
                    issue,
                )),
            };
            entries.push(DxfHatchBoundaryPathTopologyEntry {
                ordinal: compact_len(entries.len())?,
                subclass_ordinal: compact_u64(partition_entry.subclass_ordinal())?,
                raw_record_ordinal: compact_u64(
                    partition_entry.subclass().entity().record().ordinal(),
                )?,
                topology,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            partitions,
            entries: entries.into_boxed_slice(),
            paths: paths.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn partition_directory(&self) -> &DxfHatchBoundaryPartitionDirectory {
        &self.partitions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryPathTopologyEntry] {
        &self.entries
    }

    #[must_use]
    pub fn paths(&self) -> &[DxfHatchBoundaryPathEntry] {
        &self.paths
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryPathTopologyEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_subclass(&self, ordinal: u64) -> Option<DxfHatchBoundaryPathTopologyEntry> {
        self.entries
            .binary_search_by_key(&ordinal, |entry| entry.subclass_ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchBoundaryPathTopologyEntry] {
        let start = self
            .entries
            .partition_point(|entry| entry.raw_record_ordinal() < raw);
        let end = self
            .entries
            .partition_point(|entry| entry.raw_record_ordinal() <= raw);
        self.entries.get(start..end).unwrap_or_default()
    }

    #[must_use]
    pub fn path(&self, ordinal: u64) -> Option<DxfHatchBoundaryPathEntry> {
        self.paths.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn paths_for_subclass(&self, ordinal: u64) -> Option<&[DxfHatchBoundaryPathEntry]> {
        let topology = self.entry_for_subclass(ordinal)?.topology().ok()?;
        slice_for_path_range(&self.paths, topology.path_range())
    }

    #[must_use]
    pub fn orphan_fields_for_subclass(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let topology = self.entry_for_subclass(ordinal)?.topology().ok()?;
        slice_for_field_range(
            self.partitions.evidence_directory().fields(),
            topology.orphan_range(),
        )
    }

    #[must_use]
    pub fn payload_fields_for_path(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let path = self.path(ordinal)?;
        slice_for_field_range(
            self.partitions.evidence_directory().fields(),
            path.payload_range(),
        )
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_path_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPathDirectory, DxfError> {
        DxfHatchBoundaryPathDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_path_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPathDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_path_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_path_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPathDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_path_directory(cancellation)
    }
}

fn group_paths(
    document: DxfRawDocumentView<'_>,
    partitions: &DxfHatchBoundaryPartitionDirectory,
    subclass_ordinal: u64,
    partition: crate::DxfHatchBoundaryPartition,
    cancellation: &DxfCancellationToken,
    paths: &mut Vec<DxfHatchBoundaryPathEntry>,
) -> Result<DxfHatchBoundaryPathTopology, DxfError> {
    let fields = partitions
        .boundary_fields_for_subclass(subclass_ordinal)
        .ok_or_else(invalid_internal_data)?;
    let mut anchors = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        ensure_not_cancelled(cancellation)?;
        if field.group().group_code().value() == 92 {
            anchors.try_reserve(1).map_err(|_| out_of_memory())?;
            anchors.push(index);
        }
    }
    let path_start = compact_len(paths.len())?;
    let boundary_start = compact_u64(partition.boundary_range().start())?;
    let boundary_end = compact_u64(partition.boundary_range().end())?;
    let first_anchor = anchors.first().copied().unwrap_or(fields.len());
    let orphan_end = compact_add(boundary_start, first_anchor)?;

    for (path_ordinal, anchor) in anchors.iter().copied().enumerate() {
        ensure_not_cancelled(cancellation)?;
        let marker = fields
            .get(anchor)
            .copied()
            .ok_or_else(invalid_internal_data)?;
        let marker_global = compact_add(boundary_start, anchor)?;
        let payload_start = marker_global
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        let payload_end = match anchors.get(path_ordinal.saturating_add(1)).copied() {
            Some(next) => compact_add(boundary_start, next)?,
            None => boundary_end,
        };
        paths.try_reserve(1).map_err(|_| out_of_memory())?;
        paths.push(DxfHatchBoundaryPathEntry {
            ordinal: compact_len(paths.len())?,
            subclass_ordinal: compact_u64(subclass_ordinal)?,
            subclass_path_ordinal: compact_len(path_ordinal)?,
            marker,
            payload_range: DxfFillMeshRange::new(payload_start, payload_end)?,
        });
    }
    let observed = compact_len(anchors.len())?;
    let path_end = compact_len(paths.len())?;
    Ok(DxfHatchBoundaryPathTopology {
        path_range: DxfHatchBoundaryPathRange::new(path_start, path_end)
            .ok_or_else(invalid_internal_data)?,
        orphan_range: DxfFillMeshRange::new(boundary_start, orphan_end)?,
        count_relation: count_relation(
            document,
            partition.boundary_path_count().group(),
            observed,
            cancellation,
        )?,
    })
}

fn count_relation(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    observed: u32,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundaryPathCountRelation, DxfError> {
    let declared = match decode_raw_i32(document, group, cancellation)? {
        Ok(value) if value >= 0 => u32::try_from(value).map_err(|_| invalid_internal_data())?,
        Ok(value) => {
            return Ok(DxfHatchBoundaryPathCountRelation::DeclaredUnavailable(
                DxfHatchBoundaryPathCountIssue::Negative { group, value },
            ));
        }
        Err(issue) => {
            return Ok(DxfHatchBoundaryPathCountRelation::DeclaredUnavailable(
                DxfHatchBoundaryPathCountIssue::InvalidAsciiNumber { group, issue },
            ));
        }
    };
    if declared == observed {
        Ok(DxfHatchBoundaryPathCountRelation::Matched { count: declared })
    } else {
        Ok(DxfHatchBoundaryPathCountRelation::Mismatched { declared, observed })
    }
}

fn slice_for_path_range(
    values: &[DxfHatchBoundaryPathEntry],
    range: DxfHatchBoundaryPathRange,
) -> Option<&[DxfHatchBoundaryPathEntry]> {
    let start = usize::try_from(range.start()).ok()?;
    let end = usize::try_from(range.end()).ok()?;
    values.get(start..end)
}

fn slice_for_field_range<T>(values: &[T], range: DxfFillMeshRange) -> Option<&[T]> {
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

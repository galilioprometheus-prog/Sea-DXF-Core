//! Counted edge-anchor grouping for HATCH boundary paths of kind Edges.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfFillMeshField, DxfFillMeshRange, DxfHatchBoundaryPathFlagDirectory,
    DxfHatchBoundaryPathFlagEntry, DxfHatchBoundaryPathFlagIssue, DxfHatchBoundaryPathKind,
    DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    raw_integer::decode_raw_i32,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgeCount {
    group: DxfRawGroup,
    value: u32,
}

impl DxfHatchBoundaryEdgeCount {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEdgeCountRelation {
    Matched { count: u32 },
    Mismatched { declared: u32, observed: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgeRange {
    start: u32,
    end: u32,
}

impl DxfHatchBoundaryEdgeRange {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgePath {
    count: DxfHatchBoundaryEdgeCount,
    count_relation: DxfHatchBoundaryEdgeCountRelation,
    edges: DxfHatchBoundaryEdgeRange,
}

impl DxfHatchBoundaryEdgePath {
    #[must_use]
    pub const fn edge_count(self) -> DxfHatchBoundaryEdgeCount {
        self.count
    }

    #[must_use]
    pub const fn count_relation(self) -> DxfHatchBoundaryEdgeCountRelation {
        self.count_relation
    }

    #[must_use]
    pub const fn edge_range(self) -> DxfHatchBoundaryEdgeRange {
        self.edges
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEdgePathIssue {
    PathFlagsUnavailable(DxfHatchBoundaryPathFlagIssue),
    EdgeCountAbsent,
    EdgeCountMultiple {
        occurrence_count: u32,
    },
    InvalidAsciiNumber {
        group: DxfRawGroup,
        issue: DxfAsciiNumericIssue,
    },
    ValueOutOfDomain {
        group: DxfRawGroup,
        value: i32,
    },
    EdgeMarkerBeforeCount {
        group: DxfRawGroup,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEdgePathState {
    NotEdges,
    Grouped(DxfHatchBoundaryEdgePath),
    Unavailable(DxfHatchBoundaryEdgePathIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgePathEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    raw_record_ordinal: u32,
    flags: DxfHatchBoundaryPathFlagEntry,
    state: DxfHatchBoundaryEdgePathState,
}

impl DxfHatchBoundaryEdgePathEntry {
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

    #[must_use]
    pub const fn flag_entry(self) -> DxfHatchBoundaryPathFlagEntry {
        self.flags
    }

    #[must_use]
    pub const fn state(self) -> DxfHatchBoundaryEdgePathState {
        self.state
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgeEntry {
    ordinal: u32,
    path_ordinal: u32,
    path_edge_ordinal: u32,
    marker: DxfFillMeshField,
    payload_range: DxfFillMeshRange,
}

impl DxfHatchBoundaryEdgeEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn path_ordinal(self) -> u64 {
        self.path_ordinal as u64
    }

    #[must_use]
    pub const fn path_edge_ordinal(self) -> u64 {
        self.path_edge_ordinal as u64
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

/// Edge count/header states and exact group-72 anchored payload slices.
#[derive(Debug)]
pub struct DxfHatchBoundaryEdgeDirectory {
    source_id: DxfSourceId,
    flags: DxfHatchBoundaryPathFlagDirectory,
    paths: Box<[DxfHatchBoundaryEdgePathEntry]>,
    edges: Box<[DxfHatchBoundaryEdgeEntry]>,
}

impl DxfHatchBoundaryEdgeDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let flags = document.hatch_boundary_path_flag_directory(cancellation)?;
        ensure_source(document.source_id(), flags.source_id())?;
        let mut paths = Vec::new();
        let mut edges = Vec::new();
        paths
            .try_reserve(flags.entries().len())
            .map_err(|_| out_of_memory())?;
        for flag_entry in flags.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let path_ordinal = flag_entry.ordinal();
            let path = flags
                .path_directory()
                .path(path_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let edge_start = compact_len(edges.len())?;
            let state = match flag_entry.state() {
                Err(issue) => DxfHatchBoundaryEdgePathState::Unavailable(
                    DxfHatchBoundaryEdgePathIssue::PathFlagsUnavailable(issue),
                ),
                Ok(value) if value.kind() != DxfHatchBoundaryPathKind::Edges => {
                    DxfHatchBoundaryEdgePathState::NotEdges
                }
                Ok(_) => match group_edges(
                    document,
                    &flags,
                    path_ordinal,
                    path.payload_range(),
                    edge_start,
                    cancellation,
                    &mut edges,
                )? {
                    Ok(grouped) => DxfHatchBoundaryEdgePathState::Grouped(grouped),
                    Err(issue) => DxfHatchBoundaryEdgePathState::Unavailable(issue),
                },
            };
            paths.push(DxfHatchBoundaryEdgePathEntry {
                ordinal: compact_len(paths.len())?,
                subclass_ordinal: compact_u64(flag_entry.subclass_ordinal())?,
                raw_record_ordinal: compact_u64(flag_entry.raw_record_ordinal())?,
                flags: flag_entry,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            flags,
            paths: paths.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn flag_directory(&self) -> &DxfHatchBoundaryPathFlagDirectory {
        &self.flags
    }

    #[must_use]
    pub fn paths(&self) -> &[DxfHatchBoundaryEdgePathEntry] {
        &self.paths
    }

    #[must_use]
    pub fn edges(&self) -> &[DxfHatchBoundaryEdgeEntry] {
        &self.edges
    }

    #[must_use]
    pub fn path(&self, ordinal: u64) -> Option<DxfHatchBoundaryEdgePathEntry> {
        self.paths.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn edge(&self, ordinal: u64) -> Option<DxfHatchBoundaryEdgeEntry> {
        self.edges.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn edges_for_path(&self, ordinal: u64) -> Option<&[DxfHatchBoundaryEdgeEntry]> {
        let DxfHatchBoundaryEdgePathState::Grouped(path) = self.path(ordinal)?.state() else {
            return None;
        };
        let start = usize::try_from(path.edge_range().start()).ok()?;
        let end = usize::try_from(path.edge_range().end()).ok()?;
        self.edges.get(start..end)
    }

    #[must_use]
    pub fn payload_fields_for_edge(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let edge = self.edge(ordinal)?;
        let fields = self
            .flags
            .path_directory()
            .partition_directory()
            .evidence_directory()
            .fields();
        slice_for_range(fields, edge.payload_range())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_edge_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgeDirectory, DxfError> {
        DxfHatchBoundaryEdgeDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_edge_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_edge_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_edge_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_edge_directory(cancellation)
    }
}

fn group_edges(
    document: DxfRawDocumentView<'_>,
    flags: &DxfHatchBoundaryPathFlagDirectory,
    path_ordinal: u64,
    payload_range: DxfFillMeshRange,
    edge_start: u32,
    cancellation: &DxfCancellationToken,
    edges: &mut Vec<DxfHatchBoundaryEdgeEntry>,
) -> Result<Result<DxfHatchBoundaryEdgePath, DxfHatchBoundaryEdgePathIssue>, DxfError> {
    let fields = flags
        .path_directory()
        .payload_fields_for_path(path_ordinal)
        .ok_or_else(invalid_internal_data)?;
    let mut count_indices = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        ensure_not_cancelled(cancellation)?;
        if field.group().group_code().value() == 93 {
            count_indices.try_reserve(1).map_err(|_| out_of_memory())?;
            count_indices.push(index);
        }
    }
    let count_index = match count_indices.as_slice() {
        [] => return Ok(Err(DxfHatchBoundaryEdgePathIssue::EdgeCountAbsent)),
        [index] => *index,
        values => {
            return Ok(Err(DxfHatchBoundaryEdgePathIssue::EdgeCountMultiple {
                occurrence_count: compact_len(values.len())?,
            }));
        }
    };
    if let Some(group) = fields[..count_index]
        .iter()
        .find(|field| field.group().group_code().value() == 72)
        .map(|field| field.group())
    {
        return Ok(Err(DxfHatchBoundaryEdgePathIssue::EdgeMarkerBeforeCount {
            group,
        }));
    }
    let count_group = fields
        .get(count_index)
        .map(|field| field.group())
        .ok_or_else(invalid_internal_data)?;
    let declared = match decode_raw_i32(document, count_group, cancellation)? {
        Err(issue) => {
            return Ok(Err(DxfHatchBoundaryEdgePathIssue::InvalidAsciiNumber {
                group: count_group,
                issue,
            }));
        }
        Ok(value) if value < 0 => {
            return Ok(Err(DxfHatchBoundaryEdgePathIssue::ValueOutOfDomain {
                group: count_group,
                value,
            }));
        }
        Ok(value) => u32::try_from(value).map_err(|_| invalid_internal_data())?,
    };
    let mut anchors = Vec::new();
    for (index, field) in fields
        .iter()
        .enumerate()
        .skip(count_index.saturating_add(1))
    {
        ensure_not_cancelled(cancellation)?;
        if field.group().group_code().value() == 72 {
            anchors.try_reserve(1).map_err(|_| out_of_memory())?;
            anchors.push(index);
        }
    }
    let payload_start = compact_u64(payload_range.start())?;
    let payload_end = compact_u64(payload_range.end())?;
    for (path_edge_ordinal, anchor) in anchors.iter().copied().enumerate() {
        ensure_not_cancelled(cancellation)?;
        let marker = fields
            .get(anchor)
            .copied()
            .ok_or_else(invalid_internal_data)?;
        let marker_global = compact_add(payload_start, anchor)?;
        let edge_payload_start = marker_global
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        let edge_payload_end = match anchors.get(path_edge_ordinal.saturating_add(1)).copied() {
            Some(next) => compact_add(payload_start, next)?,
            None => payload_end,
        };
        edges.try_reserve(1).map_err(|_| out_of_memory())?;
        edges.push(DxfHatchBoundaryEdgeEntry {
            ordinal: compact_len(edges.len())?,
            path_ordinal: compact_u64(path_ordinal)?,
            path_edge_ordinal: compact_len(path_edge_ordinal)?,
            marker,
            payload_range: DxfFillMeshRange::new(edge_payload_start, edge_payload_end)?,
        });
    }
    let observed = compact_len(anchors.len())?;
    let edge_end = compact_len(edges.len())?;
    Ok(Ok(DxfHatchBoundaryEdgePath {
        count: DxfHatchBoundaryEdgeCount {
            group: count_group,
            value: declared,
        },
        count_relation: if declared == observed {
            DxfHatchBoundaryEdgeCountRelation::Matched { count: declared }
        } else {
            DxfHatchBoundaryEdgeCountRelation::Mismatched { declared, observed }
        },
        edges: DxfHatchBoundaryEdgeRange::new(edge_start, edge_end)?,
    }))
}

fn slice_for_range<T>(values: &[T], range: DxfFillMeshRange) -> Option<&[T]> {
    let start = usize::try_from(range.start()).ok()?;
    let end = usize::try_from(range.end()).ok()?;
    values.get(start..end)
}

fn compact_add(base: u32, offset: usize) -> Result<u32, DxfError> {
    base.checked_add(compact_len(offset)?)
        .ok_or_else(invalid_internal_data)
}

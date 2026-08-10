//! Grammar phases for repeated HATCH boundary Spline-edge data.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfFillMeshField,
    DxfFillMeshRange, DxfHatchBoundaryEdgePayloadPartitionDirectory,
    DxfHatchBoundaryEdgePayloadPartitionEntry, DxfHatchBoundaryEdgePayloadPartitionIssue,
    DxfHatchBoundaryEdgeType, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    hatch_boundary_edge_payload_partition::slice_subrange,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

const PHASE_COUNT: usize = 7;

/// Ordered grammar phase selected for one Spline edge-data field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum DxfHatchBoundarySplineEdgeSequencePhase {
    Header = 0,
    KnotValues = 1,
    ControlPoints = 2,
    FitDataCount = 3,
    FitPoints = 4,
    StartTangent = 5,
    EndTangent = 6,
}

impl DxfHatchBoundarySplineEdgeSequencePhase {
    const fn index(self) -> usize {
        self as usize
    }
}

/// Why one partitioned Spline edge cannot expose ordered sequence ranges.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgeSequenceIssue {
    EdgePayloadUnavailable(DxfHatchBoundaryEdgePayloadPartitionIssue),
    FitDataCountAbsent {
        group: DxfRawGroup,
    },
    DuplicateFitDataCount {
        group: DxfRawGroup,
    },
    FieldOutOfOrder {
        group: DxfRawGroup,
        phase: DxfHatchBoundarySplineEdgeSequencePhase,
    },
    UnexpectedField {
        group: DxfRawGroup,
    },
}

/// Exact contiguous grammar ranges inside one partitioned Spline edge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgeSequencePartition {
    edge_data_range: DxfFillMeshRange,
    header_range: DxfFillMeshRange,
    knot_range: DxfFillMeshRange,
    control_point_range: DxfFillMeshRange,
    fit_data_count: Option<DxfFillMeshField>,
    fit_point_range: DxfFillMeshRange,
    start_tangent_range: DxfFillMeshRange,
    end_tangent_range: DxfFillMeshRange,
}

impl DxfHatchBoundarySplineEdgeSequencePartition {
    #[must_use]
    pub const fn edge_data_range(self) -> DxfFillMeshRange {
        self.edge_data_range
    }

    #[must_use]
    pub const fn header_range(self) -> DxfFillMeshRange {
        self.header_range
    }

    #[must_use]
    pub const fn knot_range(self) -> DxfFillMeshRange {
        self.knot_range
    }

    #[must_use]
    pub const fn control_point_range(self) -> DxfFillMeshRange {
        self.control_point_range
    }

    #[must_use]
    pub const fn fit_data_count(self) -> Option<DxfFillMeshField> {
        self.fit_data_count
    }

    #[must_use]
    pub const fn fit_point_range(self) -> DxfFillMeshRange {
        self.fit_point_range
    }

    #[must_use]
    pub const fn start_tangent_range(self) -> DxfFillMeshRange {
        self.start_tangent_range
    }

    #[must_use]
    pub const fn end_tangent_range(self) -> DxfFillMeshRange {
        self.end_tangent_range
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgeSequenceEntry {
    ordinal: u32,
    payload_partition: DxfHatchBoundaryEdgePayloadPartitionEntry,
    sequence: Result<
        DxfHatchBoundarySplineEdgeSequencePartition,
        DxfHatchBoundarySplineEdgeSequenceIssue,
    >,
}

impl DxfHatchBoundarySplineEdgeSequenceEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn payload_partition_entry(self) -> DxfHatchBoundaryEdgePayloadPartitionEntry {
        self.payload_partition
    }

    pub const fn sequence(
        self,
    ) -> Result<DxfHatchBoundarySplineEdgeSequencePartition, DxfHatchBoundarySplineEdgeSequenceIssue>
    {
        self.sequence
    }
}

/// One ordered sequence partition for every edge typed as Spline.
#[derive(Debug)]
pub struct DxfHatchBoundarySplineEdgeSequenceDirectory {
    source_id: DxfSourceId,
    payload_partitions: DxfHatchBoundaryEdgePayloadPartitionDirectory,
    entries: Box<[DxfHatchBoundarySplineEdgeSequenceEntry]>,
}

impl DxfHatchBoundarySplineEdgeSequenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let payload_partitions =
            document.hatch_boundary_edge_payload_partition_directory(cancellation)?;
        ensure_source(document.source_id(), payload_partitions.source_id())?;
        let mut entries = Vec::new();
        for payload_partition in payload_partitions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if payload_partition.edge_type_entry().edge_type()
                != Ok(DxfHatchBoundaryEdgeType::Spline)
            {
                continue;
            }
            let sequence = match payload_partition.partition() {
                Err(issue) => {
                    Err(DxfHatchBoundarySplineEdgeSequenceIssue::EdgePayloadUnavailable(issue))
                }
                Ok(partition) => {
                    let edge = payload_partition.edge_type_entry().edge();
                    let fields = payload_partitions
                        .edge_data_fields_for_edge(edge.ordinal())
                        .ok_or_else(invalid_internal_data)?;
                    partition_sequence(partition.edge_data_range(), fields, cancellation)?
                }
            };
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfHatchBoundarySplineEdgeSequenceEntry {
                ordinal: compact_len(entries.len())?,
                payload_partition,
                sequence,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            payload_partitions,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn payload_partition_directory(
        &self,
    ) -> &DxfHatchBoundaryEdgePayloadPartitionDirectory {
        &self.payload_partitions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundarySplineEdgeSequenceEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgeSequenceEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundarySplineEdgeSequenceEntry> {
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| {
                entry
                    .payload_partition_entry()
                    .edge_type_entry()
                    .edge()
                    .ordinal()
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn header_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        self.fields_for_range(edge_ordinal, |sequence| sequence.header_range())
    }

    #[must_use]
    pub fn knot_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        self.fields_for_range(edge_ordinal, |sequence| sequence.knot_range())
    }

    #[must_use]
    pub fn control_point_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        self.fields_for_range(edge_ordinal, |sequence| sequence.control_point_range())
    }

    #[must_use]
    pub fn fit_point_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        self.fields_for_range(edge_ordinal, |sequence| sequence.fit_point_range())
    }

    #[must_use]
    pub fn start_tangent_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        self.fields_for_range(edge_ordinal, |sequence| sequence.start_tangent_range())
    }

    #[must_use]
    pub fn end_tangent_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        self.fields_for_range(edge_ordinal, |sequence| sequence.end_tangent_range())
    }

    fn fields_for_range(
        &self,
        edge_ordinal: u64,
        range: fn(DxfHatchBoundarySplineEdgeSequencePartition) -> DxfFillMeshRange,
    ) -> Option<&[DxfFillMeshField]> {
        let entry = self.entry_for_edge(edge_ordinal)?;
        let sequence = entry.sequence().ok()?;
        let fields = self
            .payload_partitions
            .edge_data_fields_for_edge(edge_ordinal)?;
        slice_subrange(fields, sequence.edge_data_range(), range(sequence))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_spline_edge_sequence_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeSequenceDirectory, DxfError> {
        DxfHatchBoundarySplineEdgeSequenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_sequence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeSequenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_spline_edge_sequence_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_sequence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeSequenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_spline_edge_sequence_directory(cancellation)
    }
}

fn partition_sequence(
    edge_data_range: DxfFillMeshRange,
    fields: &[DxfFillMeshField],
    cancellation: &DxfCancellationToken,
) -> Result<
    Result<DxfHatchBoundarySplineEdgeSequencePartition, DxfHatchBoundarySplineEdgeSequenceIssue>,
    DxfError,
> {
    let mut starts = [None; PHASE_COUNT];
    starts[DxfHatchBoundarySplineEdgeSequencePhase::Header.index()] = Some(0);
    let mut current = DxfHatchBoundarySplineEdgeSequencePhase::Header;
    let mut fit_data_count_index = None;
    for (index, field) in fields.iter().copied().enumerate() {
        ensure_not_cancelled(cancellation)?;
        let Some(phase) = phase_for_code(field.group().group_code().value()) else {
            return Ok(Err(
                DxfHatchBoundarySplineEdgeSequenceIssue::UnexpectedField {
                    group: field.group(),
                },
            ));
        };
        if phase.index() >= DxfHatchBoundarySplineEdgeSequencePhase::FitPoints.index()
            && fit_data_count_index.is_none()
        {
            return Ok(Err(
                DxfHatchBoundarySplineEdgeSequenceIssue::FitDataCountAbsent {
                    group: field.group(),
                },
            ));
        }
        if phase == DxfHatchBoundarySplineEdgeSequencePhase::FitDataCount {
            if fit_data_count_index.is_some() {
                return Ok(Err(
                    DxfHatchBoundarySplineEdgeSequenceIssue::DuplicateFitDataCount {
                        group: field.group(),
                    },
                ));
            }
            fit_data_count_index = Some(index);
        }
        if phase.index() < current.index() {
            return Ok(Err(
                DxfHatchBoundarySplineEdgeSequenceIssue::FieldOutOfOrder {
                    group: field.group(),
                    phase,
                },
            ));
        }
        if starts[phase.index()].is_none() {
            starts[phase.index()] = Some(index);
        }
        current = phase;
    }
    ensure_not_cancelled(cancellation)?;
    let fit_data_count = fit_data_count_index
        .map(|index| fields.get(index).copied().ok_or_else(invalid_internal_data))
        .transpose()?;
    Ok(Ok(DxfHatchBoundarySplineEdgeSequencePartition {
        edge_data_range,
        header_range: phase_range(
            edge_data_range,
            &starts,
            DxfHatchBoundarySplineEdgeSequencePhase::Header,
            fields.len(),
        )?,
        knot_range: phase_range(
            edge_data_range,
            &starts,
            DxfHatchBoundarySplineEdgeSequencePhase::KnotValues,
            fields.len(),
        )?,
        control_point_range: phase_range(
            edge_data_range,
            &starts,
            DxfHatchBoundarySplineEdgeSequencePhase::ControlPoints,
            fields.len(),
        )?,
        fit_data_count,
        fit_point_range: phase_range(
            edge_data_range,
            &starts,
            DxfHatchBoundarySplineEdgeSequencePhase::FitPoints,
            fields.len(),
        )?,
        start_tangent_range: phase_range(
            edge_data_range,
            &starts,
            DxfHatchBoundarySplineEdgeSequencePhase::StartTangent,
            fields.len(),
        )?,
        end_tangent_range: phase_range(
            edge_data_range,
            &starts,
            DxfHatchBoundarySplineEdgeSequencePhase::EndTangent,
            fields.len(),
        )?,
    }))
}

fn phase_range(
    outer: DxfFillMeshRange,
    starts: &[Option<usize>; PHASE_COUNT],
    phase: DxfHatchBoundarySplineEdgeSequencePhase,
    field_count: usize,
) -> Result<DxfFillMeshRange, DxfError> {
    let index = phase.index();
    let later_start = starts
        .iter()
        .skip(index.saturating_add(1))
        .flatten()
        .copied()
        .min()
        .unwrap_or(field_count);
    let start = starts[index].unwrap_or(later_start);
    global_range(outer, start, later_start)
}

fn global_range(
    outer: DxfFillMeshRange,
    local_start: usize,
    local_end: usize,
) -> Result<DxfFillMeshRange, DxfError> {
    let base = compact_u64(outer.start())?;
    let start = base
        .checked_add(compact_len(local_start)?)
        .ok_or_else(invalid_internal_data)?;
    let end = base
        .checked_add(compact_len(local_end)?)
        .ok_or_else(invalid_internal_data)?;
    DxfFillMeshRange::new(start, end)
}

const fn phase_for_code(code: i16) -> Option<DxfHatchBoundarySplineEdgeSequencePhase> {
    match code {
        94 | 73 | 74 | 95 | 96 => Some(DxfHatchBoundarySplineEdgeSequencePhase::Header),
        40 => Some(DxfHatchBoundarySplineEdgeSequencePhase::KnotValues),
        10 | 20 | 42 => Some(DxfHatchBoundarySplineEdgeSequencePhase::ControlPoints),
        97 => Some(DxfHatchBoundarySplineEdgeSequencePhase::FitDataCount),
        11 | 21 => Some(DxfHatchBoundarySplineEdgeSequencePhase::FitPoints),
        12 | 22 => Some(DxfHatchBoundarySplineEdgeSequencePhase::StartTangent),
        13 | 23 => Some(DxfHatchBoundarySplineEdgeSequencePhase::EndTangent),
        _ => None,
    }
}

//! Source-anchored LWPOLYLINE segment topology and local field semantics.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfLightweightPolylineGroupedRecordEntry, DxfLightweightPolylineInteger,
    DxfLightweightPolylineIntegerIssue, DxfLightweightPolylineIntegerRole,
    DxfLightweightPolylineVertexDirectory, DxfLightweightPolylineVertexEntry,
    DxfLightweightPolylineVertexSemanticDirectory, DxfLightweightPolylineVertexSemantics,
    DxfRawDocumentView, DxfSourceId,
};

/// Whether a closing last-to-first segment can be established from flags.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineClosureState {
    DefaultedOpen,
    ExplicitOpen { flags: i16 },
    ExplicitClosed { flags: i16 },
    IndeterminateInvalid,
    IndeterminateMultiple { occurrence_count: u32 },
}

impl DxfLightweightPolylineClosureState {
    #[must_use]
    pub const fn is_closed(self) -> Option<bool> {
        match self {
            Self::DefaultedOpen | Self::ExplicitOpen { .. } => Some(false),
            Self::ExplicitClosed { .. } => Some(true),
            Self::IndeterminateInvalid | Self::IndeterminateMultiple { .. } => None,
        }
    }

    #[must_use]
    pub const fn explicit_flags(self) -> Option<i16> {
        match self {
            Self::ExplicitOpen { flags } | Self::ExplicitClosed { flags } => Some(flags),
            Self::DefaultedOpen
            | Self::IndeterminateInvalid
            | Self::IndeterminateMultiple { .. } => None,
        }
    }
}

/// Topological source of one segment endpoint pairing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineSegmentTopology {
    Consecutive,
    Closing,
}

/// Shape implied only by the start vertex's usable bulge factor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineSegmentShape {
    Straight,
    Arc { bulge: DxfDouble },
    Indeterminate,
}

/// Half-open segment range owned by one LWPOLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineSegmentRange {
    start: u32,
    end: u32,
}

impl DxfLightweightPolylineSegmentRange {
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

/// One recognized LWPOLYLINE record and its guaranteed segment range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineSegmentRecordEntry {
    grouped_record: DxfLightweightPolylineGroupedRecordEntry,
    closure: DxfLightweightPolylineClosureState,
    segment_range: DxfLightweightPolylineSegmentRange,
}

impl DxfLightweightPolylineSegmentRecordEntry {
    #[must_use]
    pub const fn grouped_record(self) -> DxfLightweightPolylineGroupedRecordEntry {
        self.grouped_record
    }

    #[must_use]
    pub const fn closure(self) -> DxfLightweightPolylineClosureState {
        self.closure
    }

    #[must_use]
    pub const fn segment_range(self) -> DxfLightweightPolylineSegmentRange {
        self.segment_range
    }
}

/// One consecutive or closing segment referencing exact grouped vertices.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineSegmentEntry {
    ordinal: u32,
    raw_record_ordinal: u32,
    record_segment_ordinal: u32,
    start_vertex_ordinal: u32,
    end_vertex_ordinal: u32,
    topology: DxfLightweightPolylineSegmentTopology,
}

impl DxfLightweightPolylineSegmentEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn raw_record_ordinal(self) -> u64 {
        self.raw_record_ordinal as u64
    }

    #[must_use]
    pub const fn record_segment_ordinal(self) -> u64 {
        self.record_segment_ordinal as u64
    }

    #[must_use]
    pub const fn start_vertex_ordinal(self) -> u64 {
        self.start_vertex_ordinal as u64
    }

    #[must_use]
    pub const fn end_vertex_ordinal(self) -> u64 {
        self.end_vertex_ordinal as u64
    }

    #[must_use]
    pub const fn topology(self) -> DxfLightweightPolylineSegmentTopology {
        self.topology
    }
}

/// Lazy segment-local view; widths and bulge belong to the start vertex.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineSegmentSemantics {
    segment: DxfLightweightPolylineSegmentEntry,
    start_vertex: DxfLightweightPolylineVertexSemantics,
    end_vertex: DxfLightweightPolylineVertexSemantics,
}

impl DxfLightweightPolylineSegmentSemantics {
    #[must_use]
    pub const fn segment(self) -> DxfLightweightPolylineSegmentEntry {
        self.segment
    }

    #[must_use]
    pub const fn start_vertex(&self) -> &DxfLightweightPolylineVertexSemantics {
        &self.start_vertex
    }

    #[must_use]
    pub const fn end_vertex(&self) -> &DxfLightweightPolylineVertexSemantics {
        &self.end_vertex
    }

    #[must_use]
    pub fn start_ocs_position(&self) -> Option<[DxfDouble; 2]> {
        self.start_vertex.ocs_position_value()
    }

    #[must_use]
    pub fn end_ocs_position(&self) -> Option<[DxfDouble; 2]> {
        self.end_vertex.ocs_position_value()
    }

    #[must_use]
    pub fn start_vertex_local_widths(&self) -> Option<[DxfDouble; 2]> {
        self.start_vertex.local_width_values()
    }

    #[must_use]
    pub fn bulge(&self) -> Option<DxfDouble> {
        self.start_vertex.bulge_value()
    }

    #[must_use]
    pub fn shape(&self) -> DxfLightweightPolylineSegmentShape {
        match self.bulge() {
            None => DxfLightweightPolylineSegmentShape::Indeterminate,
            Some(bulge) if bulge.to_f64() == 0.0 => DxfLightweightPolylineSegmentShape::Straight,
            Some(bulge) => DxfLightweightPolylineSegmentShape::Arc { bulge },
        }
    }
}

/// Immutable segment topology retaining the complete M9.1c/d vertex graph.
#[derive(Debug)]
pub struct DxfLightweightPolylineSegmentDirectory {
    source_id: DxfSourceId,
    vertices: DxfLightweightPolylineVertexSemanticDirectory,
    records: Box<[DxfLightweightPolylineSegmentRecordEntry]>,
    segments: Box<[DxfLightweightPolylineSegmentEntry]>,
}

impl DxfLightweightPolylineSegmentDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let vertices = document.lightweight_polyline_vertex_semantic_directory(cancellation)?;
        if vertices.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: vertices.source_id(),
            });
        }

        let mut records = Vec::new();
        let mut segments = Vec::new();
        for grouped_record in vertices.vertex_directory().records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw_record_ordinal = grouped_record.record().ordinal();
            let closure = closure_state(vertices.vertex_directory(), raw_record_ordinal)?;
            let record_vertices = vertices
                .vertex_directory()
                .vertices_for_raw_record(raw_record_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let start = compact_len(segments.len())?;
            for pair in record_vertices.windows(2) {
                let [start_vertex, end_vertex] = pair else {
                    return Err(invalid_internal_data());
                };
                push_segment(
                    &mut segments,
                    raw_record_ordinal,
                    start,
                    *start_vertex,
                    *end_vertex,
                    DxfLightweightPolylineSegmentTopology::Consecutive,
                )?;
            }
            if closure.is_closed() == Some(true) && !record_vertices.is_empty() {
                let first = record_vertices
                    .first()
                    .copied()
                    .ok_or_else(invalid_internal_data)?;
                let last = record_vertices
                    .last()
                    .copied()
                    .ok_or_else(invalid_internal_data)?;
                push_segment(
                    &mut segments,
                    raw_record_ordinal,
                    start,
                    last,
                    first,
                    DxfLightweightPolylineSegmentTopology::Closing,
                )?;
            }
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfLightweightPolylineSegmentRecordEntry {
                grouped_record,
                closure,
                segment_range: DxfLightweightPolylineSegmentRange::new(
                    start,
                    compact_len(segments.len())?,
                )?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            vertices,
            records: records.into_boxed_slice(),
            segments: segments.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn vertex_semantic_directory(
        &self,
    ) -> &DxfLightweightPolylineVertexSemanticDirectory {
        &self.vertices
    }

    #[must_use]
    pub fn records(&self) -> &[DxfLightweightPolylineSegmentRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn segments(&self) -> &[DxfLightweightPolylineSegmentEntry] {
        &self.segments
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfLightweightPolylineSegmentRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.grouped_record().record().ordinal()
            })
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn segment(&self, ordinal: u64) -> Option<DxfLightweightPolylineSegmentEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.segments.get(index).copied()
    }

    #[must_use]
    pub fn segments_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfLightweightPolylineSegmentEntry]> {
        let record = self.record_for_raw_ordinal(raw_record_ordinal)?;
        slice(&self.segments, record.segment_range())
    }

    pub fn semantics_for_segment(
        &self,
        segment_ordinal: u64,
    ) -> Result<Option<DxfLightweightPolylineSegmentSemantics>, DxfError> {
        let Some(segment) = self.segment(segment_ordinal) else {
            return Ok(None);
        };
        let start_vertex = self
            .vertices
            .semantics_for_vertex(segment.start_vertex_ordinal())?
            .ok_or_else(invalid_internal_data)?;
        let end_vertex = self
            .vertices
            .semantics_for_vertex(segment.end_vertex_ordinal())?
            .ok_or_else(invalid_internal_data)?;
        Ok(Some(DxfLightweightPolylineSegmentSemantics {
            segment,
            start_vertex,
            end_vertex,
        }))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn lightweight_polyline_segment_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineSegmentDirectory, DxfError> {
        DxfLightweightPolylineSegmentDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_segment_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineSegmentDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_segment_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_segment_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineSegmentDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_segment_directory(cancellation)
    }
}

fn closure_state(
    vertices: &DxfLightweightPolylineVertexDirectory,
    raw_record_ordinal: u64,
) -> Result<DxfLightweightPolylineClosureState, DxfError> {
    let values = vertices
        .integer_evidence_directory()
        .values_for_raw_record(raw_record_ordinal)
        .ok_or_else(invalid_internal_data)?;
    let mut flags = values
        .iter()
        .copied()
        .filter(|value| value.role() == DxfLightweightPolylineIntegerRole::Flags);
    let Some(first) = flags.next() else {
        return Ok(DxfLightweightPolylineClosureState::DefaultedOpen);
    };
    let additional = flags.count();
    if additional != 0 {
        return Ok(DxfLightweightPolylineClosureState::IndeterminateMultiple {
            occurrence_count: u32::try_from(additional.saturating_add(1))
                .map_err(|_| invalid_internal_data())?,
        });
    }
    match first.value() {
        Ok(DxfLightweightPolylineInteger::I16(flags)) if flags & 1 != 0 => {
            Ok(DxfLightweightPolylineClosureState::ExplicitClosed { flags })
        }
        Ok(DxfLightweightPolylineInteger::I16(flags)) => {
            Ok(DxfLightweightPolylineClosureState::ExplicitOpen { flags })
        }
        Ok(DxfLightweightPolylineInteger::I32(_)) => Err(invalid_internal_data()),
        Err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber(_)) => {
            Ok(DxfLightweightPolylineClosureState::IndeterminateInvalid)
        }
    }
}

fn push_segment(
    segments: &mut Vec<DxfLightweightPolylineSegmentEntry>,
    raw_record_ordinal: u64,
    record_start: u32,
    start_vertex: DxfLightweightPolylineVertexEntry,
    end_vertex: DxfLightweightPolylineVertexEntry,
    topology: DxfLightweightPolylineSegmentTopology,
) -> Result<(), DxfError> {
    let ordinal = compact_len(segments.len())?;
    segments.try_reserve(1).map_err(|_| out_of_memory())?;
    segments.push(DxfLightweightPolylineSegmentEntry {
        ordinal,
        raw_record_ordinal: u32::try_from(raw_record_ordinal)
            .map_err(|_| invalid_internal_data())?,
        record_segment_ordinal: ordinal
            .checked_sub(record_start)
            .ok_or_else(invalid_internal_data)?,
        start_vertex_ordinal: vertex_ordinal(start_vertex)?,
        end_vertex_ordinal: vertex_ordinal(end_vertex)?,
        topology,
    });
    Ok(())
}

fn vertex_ordinal(vertex: DxfLightweightPolylineVertexEntry) -> Result<u32, DxfError> {
    u32::try_from(vertex.cards()[0].vertex_ordinal()).map_err(|_| invalid_internal_data())
}

fn slice<T>(values: &[T], range: DxfLightweightPolylineSegmentRange) -> Option<&[T]> {
    let start = usize::try_from(range.start()).ok()?;
    let end = usize::try_from(range.end()).ok()?;
    values.get(start..end)
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

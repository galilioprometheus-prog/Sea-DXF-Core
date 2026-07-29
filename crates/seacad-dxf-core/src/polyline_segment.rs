//! Fail-closed segment topology for consistent classic 2D/3D POLYLINE paths.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPolylineFamily, DxfPolylineFamilySemanticDirectory, DxfPolylineFamilyState,
    DxfPolylineRecordValueEntry, DxfPolylineSequenceState, DxfPolylineVertexFamilyComparison,
    DxfPolylineVertexValueEntry, DxfRawDocumentView, DxfSourceId,
};

/// Why one classic POLYLINE record does or does not expose path segments.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineSegmentRecordState {
    Available {
        family: DxfPolylineFamily,
        closed: bool,
    },
    IncompleteSequence {
        sequence_state: DxfPolylineSequenceState,
    },
    UnsupportedFamily {
        family: DxfPolylineFamily,
    },
    IndeterminateFamily,
    InconsistentVertex {
        sequence_vertex_ordinal: u32,
    },
}

/// Topological source of one classic POLYLINE segment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineSegmentTopology {
    Consecutive,
    Closing,
}

/// Half-open segment range owned by one classic POLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineSegmentRange {
    start: u32,
    end: u32,
}

impl DxfPolylineSegmentRange {
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

/// One classic POLYLINE record and its guaranteed segment range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineSegmentRecordEntry {
    record: DxfPolylineRecordValueEntry,
    state: DxfPolylineSegmentRecordState,
    segment_range: DxfPolylineSegmentRange,
}

impl DxfPolylineSegmentRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylineSegmentRecordState {
        self.state
    }

    #[must_use]
    pub const fn segment_range(self) -> DxfPolylineSegmentRange {
        self.segment_range
    }
}

/// One consecutive or closing segment referencing exact retained VERTEX entries.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineSegmentEntry {
    ordinal: u32,
    record_segment_ordinal: u32,
    record: DxfPolylineRecordValueEntry,
    start_vertex: DxfPolylineVertexValueEntry,
    end_vertex: DxfPolylineVertexValueEntry,
    topology: DxfPolylineSegmentTopology,
}

impl DxfPolylineSegmentEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record_segment_ordinal(self) -> u64 {
        self.record_segment_ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn start_vertex(self) -> DxfPolylineVertexValueEntry {
        self.start_vertex
    }

    #[must_use]
    pub const fn end_vertex(self) -> DxfPolylineVertexValueEntry {
        self.end_vertex
    }

    #[must_use]
    pub const fn topology(self) -> DxfPolylineSegmentTopology {
        self.topology
    }
}

/// Immutable path topology retaining the complete M9.2i family graph.
#[derive(Debug)]
pub struct DxfPolylineSegmentDirectory {
    source_id: DxfSourceId,
    families: DxfPolylineFamilySemanticDirectory,
    records: Box<[DxfPolylineSegmentRecordEntry]>,
    segments: Box<[DxfPolylineSegmentEntry]>,
}

impl DxfPolylineSegmentDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let families = document.polyline_family_semantic_directory(cancellation)?;
        if families.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: families.source_id(),
            });
        }

        let mut records = Vec::new();
        let mut segments = Vec::new();
        for record in families.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(segments.len())?;
            let state = record_state(&families, record)?;
            if let DxfPolylineSegmentRecordState::Available { closed, .. } = state {
                let vertices = families
                    .vertex_integer_semantic_directory()
                    .card_directory()
                    .evidence_directory()
                    .vertices_for_polyline_raw_ordinal(
                        record.sequence().polyline_record().ordinal(),
                    )
                    .ok_or_else(invalid_internal_data)?;
                for pair in vertices.windows(2) {
                    let [start_vertex, end_vertex] = pair else {
                        return Err(invalid_internal_data());
                    };
                    push_segment(
                        &mut segments,
                        record,
                        start,
                        *start_vertex,
                        *end_vertex,
                        DxfPolylineSegmentTopology::Consecutive,
                    )?;
                }
                if closed && !vertices.is_empty() {
                    push_segment(
                        &mut segments,
                        record,
                        start,
                        *vertices.last().ok_or_else(invalid_internal_data)?,
                        *vertices.first().ok_or_else(invalid_internal_data)?,
                        DxfPolylineSegmentTopology::Closing,
                    )?;
                }
            }
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfPolylineSegmentRecordEntry {
                record,
                state,
                segment_range: DxfPolylineSegmentRange::new(start, compact_len(segments.len())?)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            families,
            records: records.into_boxed_slice(),
            segments: segments.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn family_semantic_directory(&self) -> &DxfPolylineFamilySemanticDirectory {
        &self.families
    }

    #[must_use]
    pub fn records(&self) -> &[DxfPolylineSegmentRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn segments(&self) -> &[DxfPolylineSegmentEntry] {
        &self.segments
    }

    #[must_use]
    pub fn record_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfPolylineSegmentRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.record().sequence().polyline_record().ordinal()
            })
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn segment(&self, ordinal: u64) -> Option<DxfPolylineSegmentEntry> {
        self.segments.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn segments_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfPolylineSegmentEntry]> {
        let range = self
            .record_for_polyline_raw_ordinal(raw_record_ordinal)?
            .segment_range();
        self.segments
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_segment_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentDirectory, DxfError> {
        DxfPolylineSegmentDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_segment_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_segment_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_directory(cancellation)
    }
}

fn record_state(
    families: &DxfPolylineFamilySemanticDirectory,
    record: DxfPolylineRecordValueEntry,
) -> Result<DxfPolylineSegmentRecordState, DxfError> {
    let sequence_state = record.sequence().state();
    if sequence_state != DxfPolylineSequenceState::Closed {
        return Ok(DxfPolylineSegmentRecordState::IncompleteSequence { sequence_state });
    }
    let family = families
        .polyline_semantics_for_raw_ordinal(record.sequence().polyline_record().ordinal())?
        .ok_or_else(invalid_internal_data)?
        .family();
    let family = match family {
        DxfPolylineFamilyState::Classified(
            family @ (DxfPolylineFamily::TwoDimensional | DxfPolylineFamily::ThreeDimensional),
        ) => family,
        DxfPolylineFamilyState::Classified(family) => {
            return Ok(DxfPolylineSegmentRecordState::UnsupportedFamily { family });
        }
        DxfPolylineFamilyState::Unavailable | DxfPolylineFamilyState::Conflicting { .. } => {
            return Ok(DxfPolylineSegmentRecordState::IndeterminateFamily);
        }
    };
    let vertices = families.vertices().iter().filter(|vertex| {
        vertex.polyline_record().ordinal() == record.sequence().polyline_record().ordinal()
    });
    for vertex in vertices {
        let comparison = families
            .vertex_semantics_for_raw_ordinal(vertex.vertex_record().ordinal())?
            .ok_or_else(invalid_internal_data)?
            .comparison();
        if !matches!(
            comparison,
            DxfPolylineVertexFamilyComparison::Matched { .. }
        ) {
            return Ok(DxfPolylineSegmentRecordState::InconsistentVertex {
                sequence_vertex_ordinal: u32::try_from(vertex.sequence_vertex_ordinal())
                    .map_err(|_| invalid_internal_data())?,
            });
        }
    }
    let closed = families
        .record_semantic_directory()
        .semantics_for_polyline_raw_ordinal(record.sequence().polyline_record().ordinal())?
        .ok_or_else(invalid_internal_data)?
        .is_closed_or_mesh_closed_m()
        .ok_or_else(invalid_internal_data)?;
    Ok(DxfPolylineSegmentRecordState::Available { family, closed })
}

fn push_segment(
    segments: &mut Vec<DxfPolylineSegmentEntry>,
    record: DxfPolylineRecordValueEntry,
    record_start: u32,
    start_vertex: DxfPolylineVertexValueEntry,
    end_vertex: DxfPolylineVertexValueEntry,
    topology: DxfPolylineSegmentTopology,
) -> Result<(), DxfError> {
    let ordinal = compact_len(segments.len())?;
    segments.try_reserve(1).map_err(|_| out_of_memory())?;
    segments.push(DxfPolylineSegmentEntry {
        ordinal,
        record_segment_ordinal: ordinal
            .checked_sub(record_start)
            .ok_or_else(invalid_internal_data)?,
        record,
        start_vertex,
        end_vertex,
        topology,
    });
    Ok(())
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

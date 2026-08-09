//! Fail-closed segment topology for HATCH polyline-boundary paths.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchPolylineHeaderIssue, DxfHatchPolylineHeaderState,
    DxfHatchPolylineVertexCoordinateDirectory, DxfHatchPolylineVertexCoordinateEntry,
    DxfHatchPolylineVertexCountRelation, DxfHatchPolylineVertexGroupingState,
    DxfHatchPolylineVertexPathEntry, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineSegmentPathState {
    Available { closed: bool },
    NotPolyline,
    HeaderUnavailable(DxfHatchPolylineHeaderIssue),
    VertexCountMismatched { declared: u32, observed: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineSegmentTopology {
    Consecutive,
    Closing,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineSegmentRange {
    start: u32,
    end: u32,
}

impl DxfHatchPolylineSegmentRange {
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
pub struct DxfHatchPolylineSegmentPathEntry {
    path: DxfHatchPolylineVertexPathEntry,
    state: DxfHatchPolylineSegmentPathState,
    segments: DxfHatchPolylineSegmentRange,
}

impl DxfHatchPolylineSegmentPathEntry {
    #[must_use]
    pub const fn path(self) -> DxfHatchPolylineVertexPathEntry {
        self.path
    }

    #[must_use]
    pub const fn state(self) -> DxfHatchPolylineSegmentPathState {
        self.state
    }

    #[must_use]
    pub const fn segment_range(self) -> DxfHatchPolylineSegmentRange {
        self.segments
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineSegmentEntry {
    ordinal: u32,
    path_ordinal: u32,
    path_segment_ordinal: u32,
    start_vertex_ordinal: u32,
    end_vertex_ordinal: u32,
    topology: DxfHatchPolylineSegmentTopology,
}

impl DxfHatchPolylineSegmentEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn path_ordinal(self) -> u64 {
        self.path_ordinal as u64
    }

    #[must_use]
    pub const fn path_segment_ordinal(self) -> u64 {
        self.path_segment_ordinal as u64
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
    pub const fn topology(self) -> DxfHatchPolylineSegmentTopology {
        self.topology
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineSegmentEndpoints {
    segment: DxfHatchPolylineSegmentEntry,
    start: DxfHatchPolylineVertexCoordinateEntry,
    end: DxfHatchPolylineVertexCoordinateEntry,
}

impl DxfHatchPolylineSegmentEndpoints {
    #[must_use]
    pub const fn segment(self) -> DxfHatchPolylineSegmentEntry {
        self.segment
    }

    #[must_use]
    pub const fn start(self) -> DxfHatchPolylineVertexCoordinateEntry {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> DxfHatchPolylineVertexCoordinateEntry {
        self.end
    }
}

/// Path topology retaining the complete M14.4n coordinate/bulge/header chain.
#[derive(Debug)]
pub struct DxfHatchPolylineSegmentDirectory {
    source_id: DxfSourceId,
    coordinates: DxfHatchPolylineVertexCoordinateDirectory,
    paths: Box<[DxfHatchPolylineSegmentPathEntry]>,
    segments: Box<[DxfHatchPolylineSegmentEntry]>,
}

impl DxfHatchPolylineSegmentDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let coordinates = document.hatch_polyline_vertex_coordinate_directory(cancellation)?;
        ensure_source(document.source_id(), coordinates.source_id())?;
        let vertices = coordinates
            .bulge_directory()
            .numeric_directory()
            .vertex_directory();
        let mut paths = Vec::new();
        let mut segments = Vec::new();
        paths
            .try_reserve(vertices.paths().len())
            .map_err(|_| out_of_memory())?;
        for path in vertices.paths().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(segments.len())?;
            let state = match path.state() {
                DxfHatchPolylineVertexGroupingState::NotPolyline => {
                    DxfHatchPolylineSegmentPathState::NotPolyline
                }
                DxfHatchPolylineVertexGroupingState::HeaderUnavailable(issue) => {
                    DxfHatchPolylineSegmentPathState::HeaderUnavailable(issue)
                }
                DxfHatchPolylineVertexGroupingState::Grouped(grouping) => {
                    match grouping.count_relation() {
                        DxfHatchPolylineVertexCountRelation::Mismatched { declared, observed } => {
                            DxfHatchPolylineSegmentPathState::VertexCountMismatched {
                                declared,
                                observed,
                            }
                        }
                        DxfHatchPolylineVertexCountRelation::Matched { .. } => {
                            let entries = coordinates
                                .entries_for_path(path.ordinal())
                                .ok_or_else(invalid_internal_data)?;
                            let header = vertices
                                .header_directory()
                                .entry(path.ordinal())
                                .ok_or_else(invalid_internal_data)?;
                            let DxfHatchPolylineHeaderState::Explicit(header) = header.state()
                            else {
                                return Err(invalid_internal_data());
                            };
                            append_segments(
                                &mut segments,
                                path.ordinal(),
                                start,
                                entries,
                                header.is_closed().value(),
                            )?;
                            DxfHatchPolylineSegmentPathState::Available {
                                closed: header.is_closed().value(),
                            }
                        }
                    }
                }
            };
            paths.push(DxfHatchPolylineSegmentPathEntry {
                path,
                state,
                segments: DxfHatchPolylineSegmentRange::new(start, compact_len(segments.len())?)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            coordinates,
            paths: paths.into_boxed_slice(),
            segments: segments.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn coordinate_directory(&self) -> &DxfHatchPolylineVertexCoordinateDirectory {
        &self.coordinates
    }

    #[must_use]
    pub fn paths(&self) -> &[DxfHatchPolylineSegmentPathEntry] {
        &self.paths
    }

    #[must_use]
    pub fn segments(&self) -> &[DxfHatchPolylineSegmentEntry] {
        &self.segments
    }

    #[must_use]
    pub fn path(&self, ordinal: u64) -> Option<DxfHatchPolylineSegmentPathEntry> {
        self.paths.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn segment(&self, ordinal: u64) -> Option<DxfHatchPolylineSegmentEntry> {
        self.segments.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn segments_for_path(&self, ordinal: u64) -> Option<&[DxfHatchPolylineSegmentEntry]> {
        let path = self.path(ordinal)?;
        let start = usize::try_from(path.segment_range().start()).ok()?;
        let end = usize::try_from(path.segment_range().end()).ok()?;
        self.segments.get(start..end)
    }

    #[must_use]
    pub fn endpoints_for_segment(&self, ordinal: u64) -> Option<DxfHatchPolylineSegmentEndpoints> {
        let segment = self.segment(ordinal)?;
        let start = self.coordinates.entry(segment.start_vertex_ordinal())?;
        let end = self.coordinates.entry(segment.end_vertex_ordinal())?;
        Some(DxfHatchPolylineSegmentEndpoints {
            segment,
            start,
            end,
        })
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_segment_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentDirectory, DxfError> {
        DxfHatchPolylineSegmentDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_segment_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_segment_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_segment_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_segment_directory(cancellation)
    }
}

fn append_segments(
    segments: &mut Vec<DxfHatchPolylineSegmentEntry>,
    path_ordinal: u64,
    path_start: u32,
    vertices: &[DxfHatchPolylineVertexCoordinateEntry],
    closed: bool,
) -> Result<(), DxfError> {
    for pair in vertices.windows(2) {
        let [start, end] = pair else {
            return Err(invalid_internal_data());
        };
        push_segment(
            segments,
            path_ordinal,
            path_start,
            *start,
            *end,
            DxfHatchPolylineSegmentTopology::Consecutive,
        )?;
    }
    if closed && !vertices.is_empty() {
        let first = vertices
            .first()
            .copied()
            .ok_or_else(invalid_internal_data)?;
        let last = vertices.last().copied().ok_or_else(invalid_internal_data)?;
        push_segment(
            segments,
            path_ordinal,
            path_start,
            last,
            first,
            DxfHatchPolylineSegmentTopology::Closing,
        )?;
    }
    Ok(())
}

fn push_segment(
    segments: &mut Vec<DxfHatchPolylineSegmentEntry>,
    path_ordinal: u64,
    path_start: u32,
    start: DxfHatchPolylineVertexCoordinateEntry,
    end: DxfHatchPolylineVertexCoordinateEntry,
    topology: DxfHatchPolylineSegmentTopology,
) -> Result<(), DxfError> {
    let ordinal = compact_len(segments.len())?;
    segments.try_reserve(1).map_err(|_| out_of_memory())?;
    segments.push(DxfHatchPolylineSegmentEntry {
        ordinal,
        path_ordinal: compact_u64(path_ordinal)?,
        path_segment_ordinal: ordinal
            .checked_sub(path_start)
            .ok_or_else(invalid_internal_data)?,
        start_vertex_ordinal: compact_u64(start.ordinal())?,
        end_vertex_ordinal: compact_u64(end.ordinal())?,
        topology,
    });
    Ok(())
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn compact_u64(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
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

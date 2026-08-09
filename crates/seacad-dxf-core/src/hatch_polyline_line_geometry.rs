//! Exact-endpoint OCS line geometry for straight HATCH polyline segments.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchPolylineBulgeIssue, DxfHatchPolylineSegmentShape,
    DxfHatchPolylineSegmentShapeDirectory, DxfHatchPolylineSegmentShapeEntry,
    DxfHatchPolylineVertexOcsPosition, DxfHatchPolylineVertexPositionIssue, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineOcsLineSegment {
    start: DxfHatchPolylineVertexOcsPosition,
    end: DxfHatchPolylineVertexOcsPosition,
}

impl DxfHatchPolylineOcsLineSegment {
    #[must_use]
    pub const fn start(self) -> DxfHatchPolylineVertexOcsPosition {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> DxfHatchPolylineVertexOcsPosition {
        self.end
    }

    #[must_use]
    pub const fn values(self) -> [[DxfDouble; 2]; 2] {
        [self.start.values(), self.end.values()]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineLineGeometryIssue {
    ArcSegment { bulge: DxfDouble },
    ShapeIndeterminate(DxfHatchPolylineBulgeIssue),
    StartPositionUnavailable(DxfHatchPolylineVertexPositionIssue),
    EndPositionUnavailable(DxfHatchPolylineVertexPositionIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineLineGeometryEntry {
    ordinal: u32,
    shape: DxfHatchPolylineSegmentShapeEntry,
    geometry: Result<DxfHatchPolylineOcsLineSegment, DxfHatchPolylineLineGeometryIssue>,
}

impl DxfHatchPolylineLineGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn shape(self) -> DxfHatchPolylineSegmentShapeEntry {
        self.shape
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfHatchPolylineOcsLineSegment, DxfHatchPolylineLineGeometryIssue> {
        self.geometry
    }
}

/// OCS line results retaining the complete M14.4p shape/topology directory.
#[derive(Debug)]
pub struct DxfHatchPolylineLineGeometryDirectory {
    source_id: DxfSourceId,
    shapes: DxfHatchPolylineSegmentShapeDirectory,
    entries: Box<[DxfHatchPolylineLineGeometryEntry]>,
}

impl DxfHatchPolylineLineGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let shapes = document.hatch_polyline_segment_shape_directory(cancellation)?;
        ensure_source(document.source_id(), shapes.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(shapes.entries().len())
            .map_err(|_| out_of_memory())?;
        for shape in shapes.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfHatchPolylineLineGeometryEntry {
                ordinal: compact_len(entries.len())?,
                shape,
                geometry: geometry(&shapes, shape)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            shapes,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn shape_directory(&self) -> &DxfHatchPolylineSegmentShapeDirectory {
        &self.shapes
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineLineGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineLineGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchPolylineLineGeometryEntry]> {
        self.shapes.entries_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.shape().segment().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.shape().segment().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_line_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineLineGeometryDirectory, DxfError> {
        DxfHatchPolylineLineGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_line_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineLineGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_line_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_line_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineLineGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_line_geometry_directory(cancellation)
    }
}

fn geometry(
    shapes: &DxfHatchPolylineSegmentShapeDirectory,
    shape: DxfHatchPolylineSegmentShapeEntry,
) -> Result<Result<DxfHatchPolylineOcsLineSegment, DxfHatchPolylineLineGeometryIssue>, DxfError> {
    match shape.shape() {
        DxfHatchPolylineSegmentShape::Arc { bulge } => {
            Ok(Err(DxfHatchPolylineLineGeometryIssue::ArcSegment { bulge }))
        }
        DxfHatchPolylineSegmentShape::Indeterminate(issue) => Ok(Err(
            DxfHatchPolylineLineGeometryIssue::ShapeIndeterminate(issue),
        )),
        DxfHatchPolylineSegmentShape::Straight => {
            let endpoints = shapes
                .segment_directory()
                .endpoints_for_segment(shape.segment().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let start = endpoints
                .start()
                .ocs_position()
                .map_err(DxfHatchPolylineLineGeometryIssue::StartPositionUnavailable);
            let end = endpoints
                .end()
                .ocs_position()
                .map_err(DxfHatchPolylineLineGeometryIssue::EndPositionUnavailable);
            Ok(match (start, end) {
                (Ok(start), Ok(end)) => Ok(DxfHatchPolylineOcsLineSegment { start, end }),
                (Err(issue), _) | (_, Err(issue)) => Err(issue),
            })
        }
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
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

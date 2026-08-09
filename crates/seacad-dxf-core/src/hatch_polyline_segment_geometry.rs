//! Finite OCS line/arc geometry for HATCH polyline-boundary segments.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchPolylineBulgeIssue, DxfHatchPolylineLineGeometryDirectory,
    DxfHatchPolylineLineGeometryEntry, DxfHatchPolylineLineGeometryIssue,
    DxfHatchPolylineOcsLineSegment, DxfHatchPolylineVertexOcsPosition,
    DxfHatchPolylineVertexPositionIssue, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineSegmentGeometryIssue {
    ShapeIndeterminate(DxfHatchPolylineBulgeIssue),
    StartPositionUnavailable(DxfHatchPolylineVertexPositionIssue),
    EndPositionUnavailable(DxfHatchPolylineVertexPositionIssue),
    DegenerateArcChord,
    NonFiniteDerivedGeometry,
}

/// Finite derived OCS arc geometry retaining exact source endpoints and bulge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineOcsArcSegment {
    start: DxfHatchPolylineVertexOcsPosition,
    end: DxfHatchPolylineVertexOcsPosition,
    center: [DxfDouble; 2],
    radius: DxfDouble,
    signed_sweep_radians: DxfDouble,
    bulge: DxfDouble,
}

impl DxfHatchPolylineOcsArcSegment {
    #[must_use]
    pub const fn start(self) -> DxfHatchPolylineVertexOcsPosition {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> DxfHatchPolylineVertexOcsPosition {
        self.end
    }

    #[must_use]
    pub const fn center(self) -> [DxfDouble; 2] {
        self.center
    }

    #[must_use]
    pub const fn radius(self) -> DxfDouble {
        self.radius
    }

    #[must_use]
    pub const fn signed_sweep_radians(self) -> DxfDouble {
        self.signed_sweep_radians
    }

    #[must_use]
    pub const fn bulge(self) -> DxfDouble {
        self.bulge
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineOcsSegmentGeometry {
    Straight(DxfHatchPolylineOcsLineSegment),
    Arc(DxfHatchPolylineOcsArcSegment),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineSegmentGeometryEntry {
    ordinal: u32,
    line_geometry: DxfHatchPolylineLineGeometryEntry,
    geometry: Result<DxfHatchPolylineOcsSegmentGeometry, DxfHatchPolylineSegmentGeometryIssue>,
}

impl DxfHatchPolylineSegmentGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn line_geometry(self) -> DxfHatchPolylineLineGeometryEntry {
        self.line_geometry
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfHatchPolylineOcsSegmentGeometry, DxfHatchPolylineSegmentGeometryIssue> {
        self.geometry
    }
}

/// OCS segment geometry retaining the complete M14.4q line/shape/topology chain.
#[derive(Debug)]
pub struct DxfHatchPolylineSegmentGeometryDirectory {
    source_id: DxfSourceId,
    line_geometries: DxfHatchPolylineLineGeometryDirectory,
    entries: Box<[DxfHatchPolylineSegmentGeometryEntry]>,
}

impl DxfHatchPolylineSegmentGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let line_geometries = document.hatch_polyline_line_geometry_directory(cancellation)?;
        ensure_source(document.source_id(), line_geometries.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(line_geometries.entries().len())
            .map_err(|_| out_of_memory())?;
        for line_geometry in line_geometries.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfHatchPolylineSegmentGeometryEntry {
                ordinal: compact_len(entries.len())?,
                line_geometry,
                geometry: geometry(&line_geometries, line_geometry)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            line_geometries,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn line_geometry_directory(&self) -> &DxfHatchPolylineLineGeometryDirectory {
        &self.line_geometries
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineSegmentGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineSegmentGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    /// Resolve the HATCH subclass owning one locally indexed geometry entry.
    pub(crate) fn subclass_ordinal_for_entry(&self, entry_ordinal: u64) -> Option<u64> {
        let entry = self.entry(entry_ordinal)?;
        let segment = entry.line_geometry().shape().segment();
        self.line_geometries
            .shape_directory()
            .segment_directory()
            .path(segment.path_ordinal())
            .map(|path| path.path().subclass_ordinal())
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchPolylineSegmentGeometryEntry]> {
        self.line_geometries.entries_for_path(path_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry.line_geometry().shape().segment().path_ordinal() < path_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry.line_geometry().shape().segment().path_ordinal() <= path_ordinal
        });
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_segment_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentGeometryDirectory, DxfError> {
        DxfHatchPolylineSegmentGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_segment_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_segment_geometry_directory(cancellation)
    }
}

fn geometry(
    directory: &DxfHatchPolylineLineGeometryDirectory,
    entry: DxfHatchPolylineLineGeometryEntry,
) -> Result<
    Result<DxfHatchPolylineOcsSegmentGeometry, DxfHatchPolylineSegmentGeometryIssue>,
    DxfError,
> {
    match entry.geometry() {
        Ok(line) => Ok(Ok(DxfHatchPolylineOcsSegmentGeometry::Straight(line))),
        Err(DxfHatchPolylineLineGeometryIssue::ArcSegment { bulge }) => {
            arc_geometry(directory, entry, bulge)
        }
        Err(DxfHatchPolylineLineGeometryIssue::ShapeIndeterminate(issue)) => Ok(Err(
            DxfHatchPolylineSegmentGeometryIssue::ShapeIndeterminate(issue),
        )),
        Err(DxfHatchPolylineLineGeometryIssue::StartPositionUnavailable(issue)) => Ok(Err(
            DxfHatchPolylineSegmentGeometryIssue::StartPositionUnavailable(issue),
        )),
        Err(DxfHatchPolylineLineGeometryIssue::EndPositionUnavailable(issue)) => Ok(Err(
            DxfHatchPolylineSegmentGeometryIssue::EndPositionUnavailable(issue),
        )),
    }
}

fn arc_geometry(
    directory: &DxfHatchPolylineLineGeometryDirectory,
    entry: DxfHatchPolylineLineGeometryEntry,
    bulge: DxfDouble,
) -> Result<
    Result<DxfHatchPolylineOcsSegmentGeometry, DxfHatchPolylineSegmentGeometryIssue>,
    DxfError,
> {
    let endpoints = directory
        .shape_directory()
        .segment_directory()
        .endpoints_for_segment(entry.shape().segment().ordinal())
        .ok_or_else(invalid_internal_data)?;
    let start = endpoints
        .start()
        .ocs_position()
        .map_err(DxfHatchPolylineSegmentGeometryIssue::StartPositionUnavailable);
    let end = endpoints
        .end()
        .ocs_position()
        .map_err(DxfHatchPolylineSegmentGeometryIssue::EndPositionUnavailable);
    Ok(match (start, end) {
        (Ok(start), Ok(end)) => {
            derive_arc(start, end, bulge).map(DxfHatchPolylineOcsSegmentGeometry::Arc)
        }
        (Err(issue), _) | (_, Err(issue)) => Err(issue),
    })
}

fn derive_arc(
    start: DxfHatchPolylineVertexOcsPosition,
    end: DxfHatchPolylineVertexOcsPosition,
    bulge: DxfDouble,
) -> Result<DxfHatchPolylineOcsArcSegment, DxfHatchPolylineSegmentGeometryIssue> {
    let x0 = start.x().to_f64();
    let y0 = start.y().to_f64();
    let x1 = end.x().to_f64();
    let y1 = end.y().to_f64();
    let dx = x1 - x0;
    let dy = y1 - y0;
    let chord = dx.hypot(dy);
    if chord == 0.0 {
        return Err(DxfHatchPolylineSegmentGeometryIssue::DegenerateArcChord);
    }

    let bulge_value = bulge.to_f64();
    let quarter_chord = chord * 0.25;
    let center_offset = quarter_chord * (1.0 / bulge_value - bulge_value);
    let radius = quarter_chord * (bulge_value.abs() + 1.0 / bulge_value.abs());
    let midpoint_x = x0 * 0.5 + x1 * 0.5;
    let midpoint_y = y0 * 0.5 + y1 * 0.5;
    let center_x = midpoint_x + (-dy / chord) * center_offset;
    let center_y = midpoint_y + (dx / chord) * center_offset;
    let signed_sweep_radians = 4.0 * bulge_value.atan();
    if ![
        dx,
        dy,
        chord,
        center_offset,
        radius,
        center_x,
        center_y,
        signed_sweep_radians,
    ]
    .iter()
    .all(|value| value.is_finite())
        || radius <= 0.0
        || signed_sweep_radians == 0.0
    {
        return Err(DxfHatchPolylineSegmentGeometryIssue::NonFiniteDerivedGeometry);
    }
    Ok(DxfHatchPolylineOcsArcSegment {
        start,
        end,
        center: [DxfDouble::from_f64(center_x), DxfDouble::from_f64(center_y)],
        radius: DxfDouble::from_f64(radius),
        signed_sweep_radians: DxfDouble::from_f64(signed_sweep_radians),
        bulge,
    })
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

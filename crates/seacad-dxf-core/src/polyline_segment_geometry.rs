//! Lazy straight and circular geometry for classic POLYLINE segments.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfPolylineSegmentCoordinateSystem, DxfPolylineSegmentDirectory, DxfPolylineSegmentEntry,
    DxfPolylineSegmentSemanticDirectory, DxfPolylineSegmentSemantics, DxfRawDocumentView,
    DxfSourceId,
};

/// Why one topology-proven classic POLYLINE segment cannot yield finite geometry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineSegmentGeometryIssue {
    StartPositionUnavailable,
    EndPositionUnavailable,
    ElevationUnavailable,
    BulgeUnavailable,
    BulgeUnsupportedForThreeDimensional,
    DegenerateArcChord,
    NonFiniteDerivedGeometry,
}

/// Exact semantic endpoints for one straight 2D segment in polyline OCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineOcsLineSegment {
    start: [DxfDouble; 2],
    end: [DxfDouble; 2],
    elevation: DxfDouble,
}

impl DxfPolylineOcsLineSegment {
    #[must_use]
    pub const fn start(self) -> [DxfDouble; 2] {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> [DxfDouble; 2] {
        self.end
    }

    #[must_use]
    pub const fn elevation(self) -> DxfDouble {
        self.elevation
    }
}

/// Finite derived 2D arc in polyline OCS, retaining its authoritative bulge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineOcsArcSegment {
    start: [DxfDouble; 2],
    end: [DxfDouble; 2],
    center: [DxfDouble; 2],
    elevation: DxfDouble,
    radius: DxfDouble,
    signed_sweep_radians: DxfDouble,
    bulge: DxfDouble,
}

impl DxfPolylineOcsArcSegment {
    #[must_use]
    pub const fn start(self) -> [DxfDouble; 2] {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> [DxfDouble; 2] {
        self.end
    }

    #[must_use]
    pub const fn center(self) -> [DxfDouble; 2] {
        self.center
    }

    #[must_use]
    pub const fn elevation(self) -> DxfDouble {
        self.elevation
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

/// Exact semantic endpoints for one straight 3D segment in WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineWcsLineSegment {
    start: [DxfDouble; 3],
    end: [DxfDouble; 3],
}

impl DxfPolylineWcsLineSegment {
    #[must_use]
    pub const fn start(self) -> [DxfDouble; 3] {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> [DxfDouble; 3] {
        self.end
    }
}

/// One usable classic POLYLINE segment in its documented coordinate system.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineSegmentGeometry {
    TwoDimensionalStraight(DxfPolylineOcsLineSegment),
    TwoDimensionalArc(DxfPolylineOcsArcSegment),
    ThreeDimensionalStraight(DxfPolylineWcsLineSegment),
}

/// Lazy geometry result anchored to one M9.2j segment entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineSegmentGeometrySemantics {
    segment: DxfPolylineSegmentEntry,
    geometry: Result<DxfPolylineSegmentGeometry, DxfPolylineSegmentGeometryIssue>,
}

impl DxfPolylineSegmentGeometrySemantics {
    #[must_use]
    pub const fn segment(self) -> DxfPolylineSegmentEntry {
        self.segment
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfPolylineSegmentGeometry, DxfPolylineSegmentGeometryIssue> {
        self.geometry
    }
}

/// Immutable lazy geometry retaining the complete M9.2j segment graph.
#[derive(Debug)]
pub struct DxfPolylineSegmentGeometryDirectory {
    source_id: DxfSourceId,
    semantics: DxfPolylineSegmentSemanticDirectory,
}

impl DxfPolylineSegmentGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.polyline_segment_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn segment_semantic_directory(&self) -> &DxfPolylineSegmentSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub const fn segment_directory(&self) -> &DxfPolylineSegmentDirectory {
        self.semantics.segment_directory()
    }

    #[must_use]
    pub fn segments(&self) -> &[DxfPolylineSegmentEntry] {
        self.segment_directory().segments()
    }

    pub fn geometry_for_segment(
        &self,
        segment_ordinal: u64,
    ) -> Result<Option<DxfPolylineSegmentGeometrySemantics>, DxfError> {
        let Some(semantics) = self.semantics.semantics_for_segment(segment_ordinal)? else {
            return Ok(None);
        };
        Ok(Some(DxfPolylineSegmentGeometrySemantics {
            segment: semantics.segment(),
            geometry: segment_geometry(&semantics),
        }))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_segment_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentGeometryDirectory, DxfError> {
        DxfPolylineSegmentGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_geometry_directory(cancellation)
    }
}

fn segment_geometry(
    semantics: &DxfPolylineSegmentSemantics,
) -> Result<DxfPolylineSegmentGeometry, DxfPolylineSegmentGeometryIssue> {
    let start = semantics
        .start_position()
        .ok_or(DxfPolylineSegmentGeometryIssue::StartPositionUnavailable)?;
    let end = semantics
        .end_position()
        .ok_or(DxfPolylineSegmentGeometryIssue::EndPositionUnavailable)?;
    let bulge = semantics
        .bulge()
        .ok_or(DxfPolylineSegmentGeometryIssue::BulgeUnavailable)?;
    match semantics.coordinate_system() {
        DxfPolylineSegmentCoordinateSystem::Object => {
            let elevation = semantics
                .parent()
                .dummy_point_value()
                .map(|point| point[2])
                .ok_or(DxfPolylineSegmentGeometryIssue::ElevationUnavailable)?;
            let start = [start[0], start[1]];
            let end = [end[0], end[1]];
            if bulge.to_f64() == 0.0 {
                Ok(DxfPolylineSegmentGeometry::TwoDimensionalStraight(
                    DxfPolylineOcsLineSegment {
                        start,
                        end,
                        elevation,
                    },
                ))
            } else {
                derive_arc(start, end, elevation, bulge)
                    .map(DxfPolylineSegmentGeometry::TwoDimensionalArc)
            }
        }
        DxfPolylineSegmentCoordinateSystem::World => {
            if bulge.to_f64() != 0.0 {
                Err(DxfPolylineSegmentGeometryIssue::BulgeUnsupportedForThreeDimensional)
            } else {
                Ok(DxfPolylineSegmentGeometry::ThreeDimensionalStraight(
                    DxfPolylineWcsLineSegment { start, end },
                ))
            }
        }
    }
}

fn derive_arc(
    start: [DxfDouble; 2],
    end: [DxfDouble; 2],
    elevation: DxfDouble,
    bulge: DxfDouble,
) -> Result<DxfPolylineOcsArcSegment, DxfPolylineSegmentGeometryIssue> {
    let x0 = start[0].to_f64();
    let y0 = start[1].to_f64();
    let x1 = end[0].to_f64();
    let y1 = end[1].to_f64();
    let dx = x1 - x0;
    let dy = y1 - y0;
    let chord = dx.hypot(dy);
    if chord == 0.0 {
        return Err(DxfPolylineSegmentGeometryIssue::DegenerateArcChord);
    }

    let bulge_value = bulge.to_f64();
    let quarter_chord = chord * 0.25;
    let center_offset = quarter_chord * (1.0 / bulge_value - bulge_value);
    let radius = quarter_chord * (bulge_value.abs() + 1.0 / bulge_value.abs());
    let center_x = x0 * 0.5 + x1 * 0.5 + (-dy / chord) * center_offset;
    let center_y = y0 * 0.5 + y1 * 0.5 + (dx / chord) * center_offset;
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
        elevation.to_f64(),
    ]
    .iter()
    .all(|value| value.is_finite())
        || radius <= 0.0
        || signed_sweep_radians == 0.0
    {
        return Err(DxfPolylineSegmentGeometryIssue::NonFiniteDerivedGeometry);
    }
    Ok(DxfPolylineOcsArcSegment {
        start,
        end,
        center: [DxfDouble::from_f64(center_x), DxfDouble::from_f64(center_y)],
        elevation,
        radius: DxfDouble::from_f64(radius),
        signed_sweep_radians: DxfDouble::from_f64(signed_sweep_radians),
        bulge,
    })
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

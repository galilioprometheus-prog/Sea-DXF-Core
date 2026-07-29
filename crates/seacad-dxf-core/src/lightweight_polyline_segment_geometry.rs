//! Lazy OCS line/arc geometry derived from LWPOLYLINE segment semantics.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfLightweightPolylineSegmentDirectory, DxfLightweightPolylineSegmentEntry,
    DxfLightweightPolylineSegmentSemantics, DxfRawDocumentView, DxfSourceId,
};

/// Why one retained segment cannot yield finite OCS geometry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineSegmentGeometryIssue {
    StartPositionUnavailable,
    EndPositionUnavailable,
    BulgeUnavailable,
    DegenerateArcChord,
    NonFiniteDerivedGeometry,
}

/// Exact semantic endpoints for one straight OCS segment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineOcsLineSegment {
    start: [DxfDouble; 2],
    end: [DxfDouble; 2],
}

impl DxfLightweightPolylineOcsLineSegment {
    #[must_use]
    pub const fn start(self) -> [DxfDouble; 2] {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> [DxfDouble; 2] {
        self.end
    }
}

/// Finite derived OCS arc geometry retaining its authoritative bulge factor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineOcsArcSegment {
    start: [DxfDouble; 2],
    end: [DxfDouble; 2],
    center: [DxfDouble; 2],
    radius: DxfDouble,
    signed_sweep_radians: DxfDouble,
    bulge: DxfDouble,
}

impl DxfLightweightPolylineOcsArcSegment {
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

/// One usable finite straight or circular segment in polyline OCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineOcsSegmentGeometry {
    Straight(DxfLightweightPolylineOcsLineSegment),
    Arc(DxfLightweightPolylineOcsArcSegment),
}

/// Lazy geometry result anchored to one M9.1g segment entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineSegmentGeometrySemantics {
    segment: DxfLightweightPolylineSegmentEntry,
    geometry: Result<
        DxfLightweightPolylineOcsSegmentGeometry,
        DxfLightweightPolylineSegmentGeometryIssue,
    >,
}

impl DxfLightweightPolylineSegmentGeometrySemantics {
    #[must_use]
    pub const fn segment(self) -> DxfLightweightPolylineSegmentEntry {
        self.segment
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfLightweightPolylineOcsSegmentGeometry, DxfLightweightPolylineSegmentGeometryIssue>
    {
        self.geometry
    }
}

/// Immutable lazy OCS geometry retaining the complete M9.1g segment graph.
#[derive(Debug)]
pub struct DxfLightweightPolylineSegmentGeometryDirectory {
    source_id: DxfSourceId,
    segments: DxfLightweightPolylineSegmentDirectory,
}

impl DxfLightweightPolylineSegmentGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let segments = document.lightweight_polyline_segment_directory(cancellation)?;
        if segments.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: segments.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            segments,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn segment_directory(&self) -> &DxfLightweightPolylineSegmentDirectory {
        &self.segments
    }

    #[must_use]
    pub fn segments(&self) -> &[DxfLightweightPolylineSegmentEntry] {
        self.segments.segments()
    }

    pub fn geometry_for_segment(
        &self,
        segment_ordinal: u64,
    ) -> Result<Option<DxfLightweightPolylineSegmentGeometrySemantics>, DxfError> {
        let Some(semantics) = self.segments.semantics_for_segment(segment_ordinal)? else {
            return Ok(None);
        };
        Ok(Some(DxfLightweightPolylineSegmentGeometrySemantics {
            segment: semantics.segment(),
            geometry: segment_geometry(&semantics),
        }))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn lightweight_polyline_segment_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineSegmentGeometryDirectory, DxfError> {
        DxfLightweightPolylineSegmentGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_segment_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_segment_geometry_directory(cancellation)
    }
}

fn segment_geometry(
    semantics: &DxfLightweightPolylineSegmentSemantics,
) -> Result<DxfLightweightPolylineOcsSegmentGeometry, DxfLightweightPolylineSegmentGeometryIssue> {
    let start = semantics
        .start_ocs_position()
        .ok_or(DxfLightweightPolylineSegmentGeometryIssue::StartPositionUnavailable)?;
    let end = semantics
        .end_ocs_position()
        .ok_or(DxfLightweightPolylineSegmentGeometryIssue::EndPositionUnavailable)?;
    let bulge = semantics
        .bulge()
        .ok_or(DxfLightweightPolylineSegmentGeometryIssue::BulgeUnavailable)?;
    if bulge.to_f64() == 0.0 {
        return Ok(DxfLightweightPolylineOcsSegmentGeometry::Straight(
            DxfLightweightPolylineOcsLineSegment { start, end },
        ));
    }
    derive_arc(start, end, bulge).map(DxfLightweightPolylineOcsSegmentGeometry::Arc)
}

fn derive_arc(
    start: [DxfDouble; 2],
    end: [DxfDouble; 2],
    bulge: DxfDouble,
) -> Result<DxfLightweightPolylineOcsArcSegment, DxfLightweightPolylineSegmentGeometryIssue> {
    let x0 = start[0].to_f64();
    let y0 = start[1].to_f64();
    let x1 = end[0].to_f64();
    let y1 = end[1].to_f64();
    let dx = x1 - x0;
    let dy = y1 - y0;
    let chord = dx.hypot(dy);
    if chord == 0.0 {
        return Err(DxfLightweightPolylineSegmentGeometryIssue::DegenerateArcChord);
    }

    let bulge_value = bulge.to_f64();
    let reciprocal = 1.0 / bulge_value;
    let quarter_chord = chord * 0.25;
    let center_offset = quarter_chord * (reciprocal - bulge_value);
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
        return Err(DxfLightweightPolylineSegmentGeometryIssue::NonFiniteDerivedGeometry);
    }
    Ok(DxfLightweightPolylineOcsArcSegment {
        start,
        end,
        center: [DxfDouble::from_f64(center_x), DxfDouble::from_f64(center_y)],
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

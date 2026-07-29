//! Lazy WCS projection for classic POLYLINE segment geometry.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfPolylineSegmentEntry, DxfPolylineSegmentGeometry, DxfPolylineSegmentGeometryDirectory,
    DxfPolylineSegmentGeometryIssue, DxfPolylineWcsLineSegment, DxfRawDocumentView, DxfSourceId,
};

/// Why one retained segment cannot yield finite WCS geometry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineWcsSegmentGeometryIssue {
    SourceGeometry(DxfPolylineSegmentGeometryIssue),
    ExtrusionUnavailable,
    ZeroLengthExtrusion,
    NonFiniteDerivedGeometry,
}

/// A classic 2D straight segment transformed from OCS into WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineTransformedWcsLineSegment {
    start: [DxfDouble; 3],
    end: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfPolylineTransformedWcsLineSegment {
    #[must_use]
    pub const fn start(self) -> [DxfDouble; 3] {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> [DxfDouble; 3] {
        self.end
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }
}

/// A classic 2D circular segment transformed from OCS into WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineTransformedWcsArcSegment {
    start: [DxfDouble; 3],
    end: [DxfDouble; 3],
    center: [DxfDouble; 3],
    normal: [DxfDouble; 3],
    radius: DxfDouble,
    signed_sweep_radians: DxfDouble,
    bulge: DxfDouble,
}

impl DxfPolylineTransformedWcsArcSegment {
    #[must_use]
    pub const fn start(self) -> [DxfDouble; 3] {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> [DxfDouble; 3] {
        self.end
    }

    #[must_use]
    pub const fn center(self) -> [DxfDouble; 3] {
        self.center
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
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

/// One usable classic POLYLINE segment expressed in WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineWcsSegmentGeometry {
    TransformedTwoDimensionalStraight(DxfPolylineTransformedWcsLineSegment),
    TransformedTwoDimensionalArc(DxfPolylineTransformedWcsArcSegment),
    NativeThreeDimensionalStraight(DxfPolylineWcsLineSegment),
}

/// Lazy WCS result anchored to one M9.2j segment entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineWcsSegmentGeometrySemantics {
    segment: DxfPolylineSegmentEntry,
    geometry: Result<DxfPolylineWcsSegmentGeometry, DxfPolylineWcsSegmentGeometryIssue>,
}

impl DxfPolylineWcsSegmentGeometrySemantics {
    #[must_use]
    pub const fn segment(self) -> DxfPolylineSegmentEntry {
        self.segment
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfPolylineWcsSegmentGeometry, DxfPolylineWcsSegmentGeometryIssue> {
        self.geometry
    }
}

/// Immutable lazy WCS geometry retaining the complete M9.2l geometry layer.
#[derive(Debug)]
pub struct DxfPolylineWcsSegmentGeometryDirectory {
    source_id: DxfSourceId,
    geometry: DxfPolylineSegmentGeometryDirectory,
}

impl DxfPolylineWcsSegmentGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let geometry = document.polyline_segment_geometry_directory(cancellation)?;
        if geometry.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: geometry.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            geometry,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_geometry_directory(&self) -> &DxfPolylineSegmentGeometryDirectory {
        &self.geometry
    }

    #[must_use]
    pub fn segments(&self) -> &[DxfPolylineSegmentEntry] {
        self.geometry.segments()
    }

    pub fn geometry_for_segment(
        &self,
        segment_ordinal: u64,
    ) -> Result<Option<DxfPolylineWcsSegmentGeometrySemantics>, DxfError> {
        let Some(source) = self.geometry.geometry_for_segment(segment_ordinal)? else {
            return Ok(None);
        };
        let geometry = match source.geometry() {
            Ok(source_geometry) => self.project_geometry(segment_ordinal, source_geometry)?,
            Err(issue) => Err(DxfPolylineWcsSegmentGeometryIssue::SourceGeometry(issue)),
        };
        Ok(Some(DxfPolylineWcsSegmentGeometrySemantics {
            segment: source.segment(),
            geometry,
        }))
    }

    fn project_geometry(
        &self,
        segment_ordinal: u64,
        geometry: DxfPolylineSegmentGeometry,
    ) -> Result<Result<DxfPolylineWcsSegmentGeometry, DxfPolylineWcsSegmentGeometryIssue>, DxfError>
    {
        match geometry {
            DxfPolylineSegmentGeometry::ThreeDimensionalStraight(line) => Ok(Ok(
                DxfPolylineWcsSegmentGeometry::NativeThreeDimensionalStraight(line),
            )),
            DxfPolylineSegmentGeometry::TwoDimensionalStraight(line) => {
                let extrusion = self.extrusion_for_segment(segment_ordinal)?;
                Ok(project_basis(extrusion).and_then(|basis| {
                    Ok(
                        DxfPolylineWcsSegmentGeometry::TransformedTwoDimensionalStraight(
                            DxfPolylineTransformedWcsLineSegment {
                                start: basis.transform(line.start(), line.elevation())?,
                                end: basis.transform(line.end(), line.elevation())?,
                                normal: basis.normal(),
                            },
                        ),
                    )
                }))
            }
            DxfPolylineSegmentGeometry::TwoDimensionalArc(arc) => {
                let extrusion = self.extrusion_for_segment(segment_ordinal)?;
                Ok(project_basis(extrusion).and_then(|basis| {
                    Ok(DxfPolylineWcsSegmentGeometry::TransformedTwoDimensionalArc(
                        DxfPolylineTransformedWcsArcSegment {
                            start: basis.transform(arc.start(), arc.elevation())?,
                            end: basis.transform(arc.end(), arc.elevation())?,
                            center: basis.transform(arc.center(), arc.elevation())?,
                            normal: basis.normal(),
                            radius: arc.radius(),
                            signed_sweep_radians: arc.signed_sweep_radians(),
                            bulge: arc.bulge(),
                        },
                    ))
                }))
            }
        }
    }

    fn extrusion_for_segment(
        &self,
        segment_ordinal: u64,
    ) -> Result<Option<[DxfDouble; 3]>, DxfError> {
        Ok(self
            .geometry
            .segment_semantic_directory()
            .semantics_for_segment(segment_ordinal)?
            .and_then(|semantics| semantics.parent().extrusion_value()))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_wcs_segment_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineWcsSegmentGeometryDirectory, DxfError> {
        DxfPolylineWcsSegmentGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_wcs_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineWcsSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_wcs_segment_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_wcs_segment_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineWcsSegmentGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_wcs_segment_geometry_directory(cancellation)
    }
}

#[derive(Clone, Copy)]
struct OcsBasis {
    x: [f64; 3],
    y: [f64; 3],
    z: [f64; 3],
}

impl OcsBasis {
    fn normal(self) -> [DxfDouble; 3] {
        self.z.map(DxfDouble::from_f64)
    }

    fn transform(
        self,
        point: [DxfDouble; 2],
        elevation: DxfDouble,
    ) -> Result<[DxfDouble; 3], DxfPolylineWcsSegmentGeometryIssue> {
        let x = point[0].to_f64();
        let y = point[1].to_f64();
        let z = elevation.to_f64();
        let value = [
            x * self.x[0] + y * self.y[0] + z * self.z[0],
            x * self.x[1] + y * self.y[1] + z * self.z[1],
            x * self.x[2] + y * self.y[2] + z * self.z[2],
        ];
        if value.iter().all(|component| component.is_finite()) {
            Ok(value.map(DxfDouble::from_f64))
        } else {
            Err(DxfPolylineWcsSegmentGeometryIssue::NonFiniteDerivedGeometry)
        }
    }
}

fn project_basis(
    extrusion: Option<[DxfDouble; 3]>,
) -> Result<OcsBasis, DxfPolylineWcsSegmentGeometryIssue> {
    let raw = extrusion.ok_or(DxfPolylineWcsSegmentGeometryIssue::ExtrusionUnavailable)?;
    let z = normalize(raw.map(DxfDouble::to_f64))?;
    let x = if z[0].abs() < 1.0 / 64.0 && z[1].abs() < 1.0 / 64.0 {
        normalize([z[2], 0.0, -z[0]])?
    } else {
        normalize([-z[1], z[0], 0.0])?
    };
    let y = normalize(cross(z, x))?;
    Ok(OcsBasis { x, y, z })
}

fn normalize(vector: [f64; 3]) -> Result<[f64; 3], DxfPolylineWcsSegmentGeometryIssue> {
    if !vector.iter().all(|component| component.is_finite()) {
        return Err(DxfPolylineWcsSegmentGeometryIssue::NonFiniteDerivedGeometry);
    }
    let scale = vector
        .iter()
        .map(|component| component.abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Err(DxfPolylineWcsSegmentGeometryIssue::ZeroLengthExtrusion);
    }
    let scaled = vector.map(|component| component / scale);
    let length = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
    let normalized = scaled.map(|component| component / length);
    if normalized.iter().all(|component| component.is_finite()) {
        Ok(normalized)
    } else {
        Err(DxfPolylineWcsSegmentGeometryIssue::NonFiniteDerivedGeometry)
    }
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

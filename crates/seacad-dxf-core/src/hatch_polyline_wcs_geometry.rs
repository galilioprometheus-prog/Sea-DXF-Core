//! WCS projection for finite HATCH polyline-boundary segment geometry.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchElevation, DxfHatchElevationDirectory, DxfHatchElevationEntry, DxfHatchElevationIssue,
    DxfHatchExtrusion, DxfHatchExtrusionDirectory, DxfHatchExtrusionEntry, DxfHatchExtrusionIssue,
    DxfHatchPolylineOcsArcSegment, DxfHatchPolylineOcsSegmentGeometry,
    DxfHatchPolylineSegmentGeometryDirectory, DxfHatchPolylineSegmentGeometryEntry,
    DxfHatchPolylineSegmentGeometryIssue, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineWcsGeometryIssue {
    SourceGeometry(DxfHatchPolylineSegmentGeometryIssue),
    ElevationUnavailable(DxfHatchElevationIssue),
    ExtrusionUnavailable(DxfHatchExtrusionIssue),
    NonFiniteDerivedGeometry,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineWcsLineSegment {
    start: [DxfDouble; 3],
    end: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfHatchPolylineWcsLineSegment {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineWcsArcSegment {
    start: [DxfDouble; 3],
    end: [DxfDouble; 3],
    center: [DxfDouble; 3],
    normal: [DxfDouble; 3],
    radius: DxfDouble,
    signed_sweep_radians: DxfDouble,
    bulge: DxfDouble,
}

impl DxfHatchPolylineWcsArcSegment {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineWcsSegmentGeometry {
    Straight(DxfHatchPolylineWcsLineSegment),
    Arc(DxfHatchPolylineWcsArcSegment),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineWcsGeometryEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    source: DxfHatchPolylineSegmentGeometryEntry,
    geometry: Result<DxfHatchPolylineWcsSegmentGeometry, DxfHatchPolylineWcsGeometryIssue>,
}

impl DxfHatchPolylineWcsGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn source(self) -> DxfHatchPolylineSegmentGeometryEntry {
        self.source
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfHatchPolylineWcsSegmentGeometry, DxfHatchPolylineWcsGeometryIssue> {
        self.geometry
    }
}

/// WCS results retaining the complete OCS geometry, elevation, and extrusion chain.
#[derive(Debug)]
pub struct DxfHatchPolylineWcsGeometryDirectory {
    source_id: DxfSourceId,
    source_geometries: DxfHatchPolylineSegmentGeometryDirectory,
    elevations: DxfHatchElevationDirectory,
    extrusions: DxfHatchExtrusionDirectory,
    entries: Box<[DxfHatchPolylineWcsGeometryEntry]>,
}

impl DxfHatchPolylineWcsGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source_geometries = document.hatch_polyline_segment_geometry_directory(cancellation)?;
        let elevations = document.hatch_elevation_directory(cancellation)?;
        let extrusions = document.hatch_extrusion_directory(cancellation)?;
        ensure_source(document.source_id(), source_geometries.source_id())?;
        ensure_source(document.source_id(), elevations.source_id())?;
        ensure_source(document.source_id(), extrusions.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(source_geometries.entries().len())
            .map_err(|_| out_of_memory())?;
        for source in source_geometries.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let segment = source.line_geometry().shape().segment();
            let path = source_geometries
                .line_geometry_directory()
                .shape_directory()
                .segment_directory()
                .path(segment.path_ordinal())
                .ok_or_else(invalid_internal_data)?;
            let subclass_ordinal = path.path().subclass_ordinal();
            let elevation = elevations
                .entry_for_subclass(subclass_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let extrusion = extrusions
                .entry_for_subclass(subclass_ordinal)
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfHatchPolylineWcsGeometryEntry {
                ordinal: compact_len(entries.len())?,
                subclass_ordinal: compact_u64(subclass_ordinal)?,
                source,
                geometry: project(source, elevation, extrusion),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            source_geometries,
            elevations,
            extrusions,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_geometry_directory(&self) -> &DxfHatchPolylineSegmentGeometryDirectory {
        &self.source_geometries
    }

    #[must_use]
    pub const fn elevation_directory(&self) -> &DxfHatchElevationDirectory {
        &self.elevations
    }

    #[must_use]
    pub const fn extrusion_directory(&self) -> &DxfHatchExtrusionDirectory {
        &self.extrusions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineWcsGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineWcsGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchPolylineWcsGeometryEntry]> {
        self.source_geometries.entries_for_path(path_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry
                .source()
                .line_geometry()
                .shape()
                .segment()
                .path_ordinal()
                < path_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry
                .source()
                .line_geometry()
                .shape()
                .segment()
                .path_ordinal()
                <= path_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn elevation_for_entry(
        &self,
        entry: DxfHatchPolylineWcsGeometryEntry,
    ) -> Option<DxfHatchElevationEntry> {
        self.elevations.entry_for_subclass(entry.subclass_ordinal())
    }

    #[must_use]
    pub fn extrusion_for_entry(
        &self,
        entry: DxfHatchPolylineWcsGeometryEntry,
    ) -> Option<DxfHatchExtrusionEntry> {
        self.extrusions.entry_for_subclass(entry.subclass_ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_wcs_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineWcsGeometryDirectory, DxfError> {
        DxfHatchPolylineWcsGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_wcs_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_wcs_geometry_directory(cancellation)
    }
}

fn project(
    source: DxfHatchPolylineSegmentGeometryEntry,
    elevation_entry: DxfHatchElevationEntry,
    extrusion_entry: DxfHatchExtrusionEntry,
) -> Result<DxfHatchPolylineWcsSegmentGeometry, DxfHatchPolylineWcsGeometryIssue> {
    let geometry = source
        .geometry()
        .map_err(DxfHatchPolylineWcsGeometryIssue::SourceGeometry)?;
    let elevation = elevation_entry
        .elevation()
        .map_err(DxfHatchPolylineWcsGeometryIssue::ElevationUnavailable)?;
    let extrusion = extrusion_entry
        .extrusion()
        .map_err(DxfHatchPolylineWcsGeometryIssue::ExtrusionUnavailable)?;
    let basis = OcsBasis::new(extrusion)?;
    match geometry {
        DxfHatchPolylineOcsSegmentGeometry::Straight(line) => Ok(
            DxfHatchPolylineWcsSegmentGeometry::Straight(DxfHatchPolylineWcsLineSegment {
                start: basis.transform(line.start().values(), elevation)?,
                end: basis.transform(line.end().values(), elevation)?,
                normal: basis.normal(),
            }),
        ),
        DxfHatchPolylineOcsSegmentGeometry::Arc(arc) => Ok(
            DxfHatchPolylineWcsSegmentGeometry::Arc(project_arc(basis, elevation, arc)?),
        ),
    }
}

fn project_arc(
    basis: OcsBasis,
    elevation: DxfHatchElevation,
    arc: DxfHatchPolylineOcsArcSegment,
) -> Result<DxfHatchPolylineWcsArcSegment, DxfHatchPolylineWcsGeometryIssue> {
    Ok(DxfHatchPolylineWcsArcSegment {
        start: basis.transform(arc.start().values(), elevation)?,
        end: basis.transform(arc.end().values(), elevation)?,
        center: basis.transform(arc.center(), elevation)?,
        normal: basis.normal(),
        radius: arc.radius(),
        signed_sweep_radians: arc.signed_sweep_radians(),
        bulge: arc.bulge(),
    })
}

#[derive(Clone, Copy)]
struct OcsBasis {
    x: [f64; 3],
    y: [f64; 3],
    z: [f64; 3],
}

impl OcsBasis {
    fn new(extrusion: DxfHatchExtrusion) -> Result<Self, DxfHatchPolylineWcsGeometryIssue> {
        let z = normalize(extrusion.values().map(DxfDouble::to_f64))?;
        let x = if z[0].abs() < 1.0 / 64.0 && z[1].abs() < 1.0 / 64.0 {
            normalize([z[2], 0.0, -z[0]])?
        } else {
            normalize([-z[1], z[0], 0.0])?
        };
        let y = normalize(cross(z, x))?;
        Ok(Self { x, y, z })
    }

    fn normal(self) -> [DxfDouble; 3] {
        self.z.map(DxfDouble::from_f64)
    }

    fn transform(
        self,
        point: [DxfDouble; 2],
        elevation: DxfHatchElevation,
    ) -> Result<[DxfDouble; 3], DxfHatchPolylineWcsGeometryIssue> {
        let x = point[0].to_f64();
        let y = point[1].to_f64();
        let z = elevation.z().value().to_f64();
        let value = [
            x * self.x[0] + y * self.y[0] + z * self.z[0],
            x * self.x[1] + y * self.y[1] + z * self.z[1],
            x * self.x[2] + y * self.y[2] + z * self.z[2],
        ];
        if value.iter().all(|component| component.is_finite()) {
            Ok(value.map(DxfDouble::from_f64))
        } else {
            Err(DxfHatchPolylineWcsGeometryIssue::NonFiniteDerivedGeometry)
        }
    }
}

fn normalize(vector: [f64; 3]) -> Result<[f64; 3], DxfHatchPolylineWcsGeometryIssue> {
    if !vector.iter().all(|component| component.is_finite()) {
        return Err(DxfHatchPolylineWcsGeometryIssue::NonFiniteDerivedGeometry);
    }
    let scale = vector
        .iter()
        .map(|component| component.abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Err(DxfHatchPolylineWcsGeometryIssue::NonFiniteDerivedGeometry);
    }
    let scaled = vector.map(|component| component / scale);
    let length = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
    let normalized = scaled.map(|component| component / length);
    if normalized.iter().all(|component| component.is_finite()) {
        Ok(normalized)
    } else {
        Err(DxfHatchPolylineWcsGeometryIssue::NonFiniteDerivedGeometry)
    }
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
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

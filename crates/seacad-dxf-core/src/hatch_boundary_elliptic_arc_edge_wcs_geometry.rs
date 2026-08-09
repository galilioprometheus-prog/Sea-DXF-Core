//! WCS projection for exact HATCH boundary EllipticArc-edge geometry.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryEllipticArcEdgeDirection, DxfHatchBoundaryEllipticArcEdgeGeometryDirectory,
    DxfHatchBoundaryEllipticArcEdgeGeometryEntry, DxfHatchBoundaryEllipticArcEdgeGeometryIssue,
    DxfHatchElevationDirectory, DxfHatchElevationEntry, DxfHatchElevationIssue,
    DxfHatchExtrusionDirectory, DxfHatchExtrusionEntry, DxfHatchExtrusionIssue, DxfRawDocumentView,
    DxfSourceId,
    hatch_polyline_wcs_geometry::OcsBasis,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue {
    SourceGeometry(DxfHatchBoundaryEllipticArcEdgeGeometryIssue),
    ElevationUnavailable(DxfHatchElevationIssue),
    ExtrusionUnavailable(DxfHatchExtrusionIssue),
    NonFiniteDerivedGeometry,
}

/// WCS EllipticArc frame without minor-axis, endpoint, or sweep derivation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeWcsSegment {
    center: [DxfDouble; 3],
    major_axis_endpoint_relative_to_center: [DxfDouble; 3],
    normal: [DxfDouble; 3],
    minor_to_major_ratio: DxfDouble,
    start_angle_degrees: DxfDouble,
    end_angle_degrees: DxfDouble,
    direction: DxfHatchBoundaryEllipticArcEdgeDirection,
}

impl DxfHatchBoundaryEllipticArcEdgeWcsSegment {
    #[must_use]
    pub const fn center(self) -> [DxfDouble; 3] {
        self.center
    }

    #[must_use]
    pub const fn major_axis_endpoint_relative_to_center(self) -> [DxfDouble; 3] {
        self.major_axis_endpoint_relative_to_center
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }

    #[must_use]
    pub const fn minor_to_major_ratio(self) -> DxfDouble {
        self.minor_to_major_ratio
    }

    #[must_use]
    pub const fn start_angle_degrees(self) -> DxfDouble {
        self.start_angle_degrees
    }

    #[must_use]
    pub const fn end_angle_degrees(self) -> DxfDouble {
        self.end_angle_degrees
    }

    #[must_use]
    pub const fn direction(self) -> DxfHatchBoundaryEllipticArcEdgeDirection {
        self.direction
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    source: DxfHatchBoundaryEllipticArcEdgeGeometryEntry,
    geometry: Result<
        DxfHatchBoundaryEllipticArcEdgeWcsSegment,
        DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue,
    >,
}

impl DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn source(self) -> DxfHatchBoundaryEllipticArcEdgeGeometryEntry {
        self.source
    }

    pub const fn geometry(
        self,
    ) -> Result<
        DxfHatchBoundaryEllipticArcEdgeWcsSegment,
        DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue,
    > {
        self.geometry
    }
}

/// WCS results retaining OCS geometry, elevation, and extrusion evidence.
#[derive(Debug)]
pub struct DxfHatchBoundaryEllipticArcEdgeWcsGeometryDirectory {
    source_id: DxfSourceId,
    source_geometries: DxfHatchBoundaryEllipticArcEdgeGeometryDirectory,
    elevations: DxfHatchElevationDirectory,
    extrusions: DxfHatchExtrusionDirectory,
    entries: Box<[DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry]>,
}

impl DxfHatchBoundaryEllipticArcEdgeWcsGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source_geometries =
            document.hatch_boundary_elliptic_arc_edge_geometry_directory(cancellation)?;
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
            let subclass_ordinal = source_geometries
                .subclass_ordinal_for_entry(source.ordinal())
                .ok_or_else(invalid_internal_data)?;
            let elevation = elevations
                .entry_for_subclass(subclass_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let extrusion = extrusions
                .entry_for_subclass(subclass_ordinal)
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry {
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
    pub const fn source_geometry_directory(
        &self,
    ) -> &DxfHatchBoundaryEllipticArcEdgeGeometryDirectory {
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
    pub fn entries(&self) -> &[DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry> {
        self.source_geometries.entry_for_edge(edge_ordinal)?;
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| {
                entry.source().semantics().numeric().edge_ordinal()
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry]> {
        self.source_geometries.entries_for_path(path_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry.source().semantics().numeric().path_ordinal() < path_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry.source().semantics().numeric().path_ordinal() <= path_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn elevation_for_entry(
        &self,
        entry: DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry,
    ) -> Option<DxfHatchElevationEntry> {
        self.elevations.entry_for_subclass(entry.subclass_ordinal())
    }

    #[must_use]
    pub fn extrusion_for_entry(
        &self,
        entry: DxfHatchBoundaryEllipticArcEdgeWcsGeometryEntry,
    ) -> Option<DxfHatchExtrusionEntry> {
        self.extrusions.entry_for_subclass(entry.subclass_ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_wcs_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeWcsGeometryDirectory, DxfError> {
        DxfHatchBoundaryEllipticArcEdgeWcsGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_wcs_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_wcs_geometry_directory(cancellation)
    }
}

fn project(
    source: DxfHatchBoundaryEllipticArcEdgeGeometryEntry,
    elevation_entry: DxfHatchElevationEntry,
    extrusion_entry: DxfHatchExtrusionEntry,
) -> Result<
    DxfHatchBoundaryEllipticArcEdgeWcsSegment,
    DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue,
> {
    let geometry = source
        .geometry()
        .map_err(DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue::SourceGeometry)?;
    let elevation = elevation_entry
        .elevation()
        .map_err(DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue::ElevationUnavailable)?;
    let extrusion = extrusion_entry
        .extrusion()
        .map_err(DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue::ExtrusionUnavailable)?;
    let basis = OcsBasis::new(extrusion).map_err(|_| nonfinite())?;
    Ok(DxfHatchBoundaryEllipticArcEdgeWcsSegment {
        center: basis
            .transform(geometry.center(), elevation)
            .map_err(|_| nonfinite())?,
        major_axis_endpoint_relative_to_center: basis
            .transform_vector(geometry.major_axis_endpoint_relative_to_center())
            .map_err(|_| nonfinite())?,
        normal: basis.normal(),
        minor_to_major_ratio: geometry.minor_to_major_ratio(),
        start_angle_degrees: geometry.start_angle_degrees(),
        end_angle_degrees: geometry.end_angle_degrees(),
        direction: geometry.direction(),
    })
}

const fn nonfinite() -> DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue {
    DxfHatchBoundaryEllipticArcEdgeWcsGeometryIssue::NonFiniteDerivedGeometry
}

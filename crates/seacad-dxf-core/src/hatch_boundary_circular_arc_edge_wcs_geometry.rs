//! WCS projection for exact HATCH boundary CircularArc-edge geometry.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryCircularArcEdgeDirection, DxfHatchBoundaryCircularArcEdgeGeometryDirectory,
    DxfHatchBoundaryCircularArcEdgeGeometryEntry, DxfHatchBoundaryCircularArcEdgeGeometryIssue,
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
pub enum DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue {
    SourceGeometry(DxfHatchBoundaryCircularArcEdgeGeometryIssue),
    ElevationUnavailable(DxfHatchElevationIssue),
    ExtrusionUnavailable(DxfHatchExtrusionIssue),
    NonFiniteDerivedGeometry,
}

/// WCS CircularArc frame retaining exact source radius, angles, and direction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeWcsSegment {
    center: [DxfDouble; 3],
    x_axis: [DxfDouble; 3],
    y_axis: [DxfDouble; 3],
    normal: [DxfDouble; 3],
    radius: DxfDouble,
    start_angle_degrees: DxfDouble,
    end_angle_degrees: DxfDouble,
    direction: DxfHatchBoundaryCircularArcEdgeDirection,
}

impl DxfHatchBoundaryCircularArcEdgeWcsSegment {
    #[must_use]
    pub const fn center(self) -> [DxfDouble; 3] {
        self.center
    }

    #[must_use]
    pub const fn x_axis(self) -> [DxfDouble; 3] {
        self.x_axis
    }

    #[must_use]
    pub const fn y_axis(self) -> [DxfDouble; 3] {
        self.y_axis
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
    pub const fn start_angle_degrees(self) -> DxfDouble {
        self.start_angle_degrees
    }

    #[must_use]
    pub const fn end_angle_degrees(self) -> DxfDouble {
        self.end_angle_degrees
    }

    #[must_use]
    pub const fn direction(self) -> DxfHatchBoundaryCircularArcEdgeDirection {
        self.direction
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    source: DxfHatchBoundaryCircularArcEdgeGeometryEntry,
    geometry: Result<
        DxfHatchBoundaryCircularArcEdgeWcsSegment,
        DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue,
    >,
}

impl DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn source(self) -> DxfHatchBoundaryCircularArcEdgeGeometryEntry {
        self.source
    }

    pub const fn geometry(
        self,
    ) -> Result<
        DxfHatchBoundaryCircularArcEdgeWcsSegment,
        DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue,
    > {
        self.geometry
    }
}

/// WCS results retaining OCS geometry, elevation, and extrusion evidence.
#[derive(Debug)]
pub struct DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory {
    source_id: DxfSourceId,
    source_geometries: DxfHatchBoundaryCircularArcEdgeGeometryDirectory,
    elevations: DxfHatchElevationDirectory,
    extrusions: DxfHatchExtrusionDirectory,
    entries: Box<[DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry]>,
}

impl DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source_geometries =
            document.hatch_boundary_circular_arc_edge_geometry_directory(cancellation)?;
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
            entries.push(DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry {
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
    ) -> &DxfHatchBoundaryCircularArcEdgeGeometryDirectory {
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
    pub fn entries(&self) -> &[DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry> {
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
    ) -> Option<&[DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry]> {
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
        entry: DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry,
    ) -> Option<DxfHatchElevationEntry> {
        self.elevations.entry_for_subclass(entry.subclass_ordinal())
    }

    #[must_use]
    pub fn extrusion_for_entry(
        &self,
        entry: DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry,
    ) -> Option<DxfHatchExtrusionEntry> {
        self.extrusions.entry_for_subclass(entry.subclass_ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_circular_arc_edge_wcs_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory, DxfError> {
        DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_wcs_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_wcs_geometry_directory(cancellation)
    }
}

fn project(
    source: DxfHatchBoundaryCircularArcEdgeGeometryEntry,
    elevation_entry: DxfHatchElevationEntry,
    extrusion_entry: DxfHatchExtrusionEntry,
) -> Result<
    DxfHatchBoundaryCircularArcEdgeWcsSegment,
    DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue,
> {
    let geometry = source
        .geometry()
        .map_err(DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::SourceGeometry)?;
    let elevation = elevation_entry
        .elevation()
        .map_err(DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::ElevationUnavailable)?;
    let extrusion = extrusion_entry
        .extrusion()
        .map_err(DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::ExtrusionUnavailable)?;
    let basis = OcsBasis::new(extrusion).map_err(|_| nonfinite())?;
    Ok(DxfHatchBoundaryCircularArcEdgeWcsSegment {
        center: basis
            .transform(geometry.center(), elevation)
            .map_err(|_| nonfinite())?,
        x_axis: basis.x_axis(),
        y_axis: basis.y_axis(),
        normal: basis.normal(),
        radius: geometry.radius(),
        start_angle_degrees: geometry.start_angle_degrees(),
        end_angle_degrees: geometry.end_angle_degrees(),
        direction: geometry.direction(),
    })
}

const fn nonfinite() -> DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue {
    DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::NonFiniteDerivedGeometry
}

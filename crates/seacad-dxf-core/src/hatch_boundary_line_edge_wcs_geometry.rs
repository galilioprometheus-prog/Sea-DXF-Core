//! WCS projection for finite HATCH boundary Line-edge geometry.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryLineEdgeGeometryDirectory, DxfHatchBoundaryLineEdgeGeometryEntry,
    DxfHatchBoundaryLineEdgeGeometryIssue, DxfHatchElevationDirectory, DxfHatchElevationEntry,
    DxfHatchElevationIssue, DxfHatchExtrusionDirectory, DxfHatchExtrusionEntry,
    DxfHatchExtrusionIssue, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
    hatch_polyline_wcs_geometry::OcsBasis,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeWcsGeometryIssue {
    SourceGeometry(DxfHatchBoundaryLineEdgeGeometryIssue),
    ElevationUnavailable(DxfHatchElevationIssue),
    ExtrusionUnavailable(DxfHatchExtrusionIssue),
    NonFiniteDerivedGeometry,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeWcsSegment {
    start: [DxfDouble; 3],
    end: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfHatchBoundaryLineEdgeWcsSegment {
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
pub struct DxfHatchBoundaryLineEdgeWcsGeometryEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    source: DxfHatchBoundaryLineEdgeGeometryEntry,
    geometry: Result<DxfHatchBoundaryLineEdgeWcsSegment, DxfHatchBoundaryLineEdgeWcsGeometryIssue>,
}

impl DxfHatchBoundaryLineEdgeWcsGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn source(self) -> DxfHatchBoundaryLineEdgeGeometryEntry {
        self.source
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfHatchBoundaryLineEdgeWcsSegment, DxfHatchBoundaryLineEdgeWcsGeometryIssue> {
        self.geometry
    }
}

/// WCS results retaining OCS geometry, elevation, and extrusion evidence.
#[derive(Debug)]
pub struct DxfHatchBoundaryLineEdgeWcsGeometryDirectory {
    source_id: DxfSourceId,
    source_geometries: DxfHatchBoundaryLineEdgeGeometryDirectory,
    elevations: DxfHatchElevationDirectory,
    extrusions: DxfHatchExtrusionDirectory,
    entries: Box<[DxfHatchBoundaryLineEdgeWcsGeometryEntry]>,
}

impl DxfHatchBoundaryLineEdgeWcsGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source_geometries =
            document.hatch_boundary_line_edge_geometry_directory(cancellation)?;
        let elevations = document.hatch_elevation_directory(cancellation)?;
        let extrusions = document.hatch_extrusion_directory(cancellation)?;
        ensure_source(document.source_id(), source_geometries.source_id())?;
        ensure_source(document.source_id(), elevations.source_id())?;
        ensure_source(document.source_id(), extrusions.source_id())?;
        let edge_directory = source_geometries
            .coordinate_directory()
            .numeric_directory()
            .card_directory()
            .edge_type_directory()
            .edge_directory();
        let mut entries = Vec::new();
        entries
            .try_reserve(source_geometries.entries().len())
            .map_err(|_| out_of_memory())?;
        for source in source_geometries.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let path_ordinal = source.coordinates().numeric().path_ordinal();
            let path = edge_directory
                .path(path_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let subclass_ordinal = path.subclass_ordinal();
            let elevation = elevations
                .entry_for_subclass(subclass_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let extrusion = extrusions
                .entry_for_subclass(subclass_ordinal)
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfHatchBoundaryLineEdgeWcsGeometryEntry {
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
    pub const fn source_geometry_directory(&self) -> &DxfHatchBoundaryLineEdgeGeometryDirectory {
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
    pub fn entries(&self) -> &[DxfHatchBoundaryLineEdgeWcsGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryLineEdgeWcsGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryLineEdgeWcsGeometryEntry> {
        self.source_geometries.entry_for_edge(edge_ordinal)?;
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| {
                entry.source().coordinates().numeric().edge_ordinal()
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchBoundaryLineEdgeWcsGeometryEntry]> {
        self.source_geometries.entries_for_path(path_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry.source().coordinates().numeric().path_ordinal() < path_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry.source().coordinates().numeric().path_ordinal() <= path_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn elevation_for_entry(
        &self,
        entry: DxfHatchBoundaryLineEdgeWcsGeometryEntry,
    ) -> Option<DxfHatchElevationEntry> {
        self.elevations.entry_for_subclass(entry.subclass_ordinal())
    }

    #[must_use]
    pub fn extrusion_for_entry(
        &self,
        entry: DxfHatchBoundaryLineEdgeWcsGeometryEntry,
    ) -> Option<DxfHatchExtrusionEntry> {
        self.extrusions.entry_for_subclass(entry.subclass_ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_line_edge_wcs_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeWcsGeometryDirectory, DxfError> {
        DxfHatchBoundaryLineEdgeWcsGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_line_edge_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_wcs_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_line_edge_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_wcs_geometry_directory(cancellation)
    }
}

fn project(
    source: DxfHatchBoundaryLineEdgeGeometryEntry,
    elevation_entry: DxfHatchElevationEntry,
    extrusion_entry: DxfHatchExtrusionEntry,
) -> Result<DxfHatchBoundaryLineEdgeWcsSegment, DxfHatchBoundaryLineEdgeWcsGeometryIssue> {
    let geometry = source
        .geometry()
        .map_err(DxfHatchBoundaryLineEdgeWcsGeometryIssue::SourceGeometry)?;
    let elevation = elevation_entry
        .elevation()
        .map_err(DxfHatchBoundaryLineEdgeWcsGeometryIssue::ElevationUnavailable)?;
    let extrusion = extrusion_entry
        .extrusion()
        .map_err(DxfHatchBoundaryLineEdgeWcsGeometryIssue::ExtrusionUnavailable)?;
    let basis = OcsBasis::new(extrusion).map_err(|_| nonfinite())?;
    Ok(DxfHatchBoundaryLineEdgeWcsSegment {
        start: basis
            .transform(geometry.start().values(), elevation)
            .map_err(|_| nonfinite())?,
        end: basis
            .transform(geometry.end().values(), elevation)
            .map_err(|_| nonfinite())?,
        normal: basis.normal(),
    })
}

const fn nonfinite() -> DxfHatchBoundaryLineEdgeWcsGeometryIssue {
    DxfHatchBoundaryLineEdgeWcsGeometryIssue::NonFiniteDerivedGeometry
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

//! Exact-endpoint OCS line geometry for HATCH boundary Line edges.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryLineEdgeCoordinateDirectory, DxfHatchBoundaryLineEdgeCoordinateEntry,
    DxfHatchBoundaryLineEdgeEndpointIssue, DxfHatchBoundaryLineEdgeOcsPoint, DxfRawDocumentView,
    DxfSourceId,
    read_support::{compact_len, ensure_not_cancelled, ensure_source, out_of_memory},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeOcsSegment {
    start: DxfHatchBoundaryLineEdgeOcsPoint,
    end: DxfHatchBoundaryLineEdgeOcsPoint,
}

impl DxfHatchBoundaryLineEdgeOcsSegment {
    #[must_use]
    pub const fn start(self) -> DxfHatchBoundaryLineEdgeOcsPoint {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> DxfHatchBoundaryLineEdgeOcsPoint {
        self.end
    }

    #[must_use]
    pub const fn values(self) -> [[DxfDouble; 2]; 2] {
        [self.start.values(), self.end.values()]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeGeometryIssue {
    EndpointsUnavailable(DxfHatchBoundaryLineEdgeEndpointIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeGeometryEntry {
    ordinal: u32,
    coordinates: DxfHatchBoundaryLineEdgeCoordinateEntry,
    geometry: Result<DxfHatchBoundaryLineEdgeOcsSegment, DxfHatchBoundaryLineEdgeGeometryIssue>,
}

impl DxfHatchBoundaryLineEdgeGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn coordinates(self) -> DxfHatchBoundaryLineEdgeCoordinateEntry {
        self.coordinates
    }

    pub const fn geometry(
        self,
    ) -> Result<DxfHatchBoundaryLineEdgeOcsSegment, DxfHatchBoundaryLineEdgeGeometryIssue> {
        self.geometry
    }
}

/// OCS line results retaining the complete M14.4x coordinate directory.
#[derive(Debug)]
pub struct DxfHatchBoundaryLineEdgeGeometryDirectory {
    source_id: DxfSourceId,
    coordinates: DxfHatchBoundaryLineEdgeCoordinateDirectory,
    entries: Box<[DxfHatchBoundaryLineEdgeGeometryEntry]>,
}

impl DxfHatchBoundaryLineEdgeGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let coordinates = document.hatch_boundary_line_edge_coordinate_directory(cancellation)?;
        ensure_source(document.source_id(), coordinates.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(coordinates.entries().len())
            .map_err(|_| out_of_memory())?;
        for coordinate in coordinates.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfHatchBoundaryLineEdgeGeometryEntry {
                ordinal: compact_len(entries.len())?,
                coordinates: coordinate,
                geometry: geometry(coordinate),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            coordinates,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn coordinate_directory(&self) -> &DxfHatchBoundaryLineEdgeCoordinateDirectory {
        &self.coordinates
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryLineEdgeGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryLineEdgeGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryLineEdgeGeometryEntry> {
        self.coordinates.entry_for_edge(edge_ordinal)?;
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| {
                entry.coordinates().numeric().edge_ordinal()
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchBoundaryLineEdgeGeometryEntry]> {
        self.coordinates.entries_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.coordinates().numeric().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.coordinates().numeric().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_line_edge_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeGeometryDirectory, DxfError> {
        DxfHatchBoundaryLineEdgeGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_line_edge_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_line_edge_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_geometry_directory(cancellation)
    }
}

fn geometry(
    coordinate: DxfHatchBoundaryLineEdgeCoordinateEntry,
) -> Result<DxfHatchBoundaryLineEdgeOcsSegment, DxfHatchBoundaryLineEdgeGeometryIssue> {
    let endpoints = coordinate
        .ocs_endpoints()
        .map_err(DxfHatchBoundaryLineEdgeGeometryIssue::EndpointsUnavailable)?;
    Ok(DxfHatchBoundaryLineEdgeOcsSegment {
        start: endpoints.start(),
        end: endpoints.end(),
    })
}

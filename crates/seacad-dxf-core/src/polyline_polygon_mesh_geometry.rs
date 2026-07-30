//! Exact WCS corner assembly for classic POLYLINE polygon-mesh cells.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfPolylinePolygonMeshCellEntry, DxfPolylinePolygonMeshDirectory,
    DxfPolylineVertexSemanticDirectory, DxfPolylineVertexValueEntry, DxfRawDocumentView,
    DxfSemanticValueState, DxfSourceId,
};

/// One named row-major polygon-mesh cell corner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolygonMeshCellCorner {
    M0N0,
    M0N1,
    M1N1,
    M1N0,
}

/// Exact WCS geometry or a typed coordinate failure for one mesh cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolygonMeshCellGeometryState {
    Available {
        m0_n0: [DxfDouble; 3],
        m0_n1: [DxfDouble; 3],
        m1_n1: [DxfDouble; 3],
        m1_n0: [DxfDouble; 3],
    },
    CoordinateUnavailable {
        corner: DxfPolylinePolygonMeshCellCorner,
        component_states: [DxfSemanticValueState; 3],
    },
}

/// One exact M9.2o cell and its WCS corner state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolygonMeshCellGeometryEntry {
    cell: DxfPolylinePolygonMeshCellEntry,
    state: DxfPolylinePolygonMeshCellGeometryState,
}

impl DxfPolylinePolygonMeshCellGeometryEntry {
    #[must_use]
    pub const fn cell(self) -> DxfPolylinePolygonMeshCellEntry {
        self.cell
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylinePolygonMeshCellGeometryState {
        self.state
    }
}

/// Immutable exact WCS assembly retaining topology and vertex semantics.
#[derive(Debug)]
pub struct DxfPolylinePolygonMeshCellGeometryDirectory {
    source_id: DxfSourceId,
    topology: DxfPolylinePolygonMeshDirectory,
    vertices: DxfPolylineVertexSemanticDirectory,
    cells: Box<[DxfPolylinePolygonMeshCellGeometryEntry]>,
}

impl DxfPolylinePolygonMeshCellGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let topology = document.polyline_polygon_mesh_directory(cancellation)?;
        let vertices = document.polyline_vertex_semantic_directory(cancellation)?;
        if topology.source_id() != document.source_id()
            || vertices.source_id() != document.source_id()
        {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: topology.source_id(),
            });
        }
        let mut cells = Vec::new();
        cells
            .try_reserve(topology.cells().len())
            .map_err(|_| out_of_memory())?;
        for cell in topology.cells().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            cells.push(DxfPolylinePolygonMeshCellGeometryEntry {
                cell,
                state: resolve_cell(&vertices, cell)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            topology,
            vertices,
            cells: cells.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn topology_directory(&self) -> &DxfPolylinePolygonMeshDirectory {
        &self.topology
    }

    #[must_use]
    pub const fn vertex_semantic_directory(&self) -> &DxfPolylineVertexSemanticDirectory {
        &self.vertices
    }

    #[must_use]
    pub fn cells(&self) -> &[DxfPolylinePolygonMeshCellGeometryEntry] {
        &self.cells
    }

    #[must_use]
    pub fn cell(&self, ordinal: u64) -> Option<DxfPolylinePolygonMeshCellGeometryEntry> {
        self.cells.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cells_for_polyline_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<&[DxfPolylinePolygonMeshCellGeometryEntry]> {
        let range = self
            .topology
            .record_for_polyline_raw_ordinal(raw)?
            .cell_range();
        self.cells
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_polygon_mesh_cell_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshCellGeometryDirectory, DxfError> {
        DxfPolylinePolygonMeshCellGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_polygon_mesh_cell_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshCellGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polygon_mesh_cell_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_polygon_mesh_cell_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshCellGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polygon_mesh_cell_geometry_directory(cancellation)
    }
}

fn resolve_cell(
    vertices: &DxfPolylineVertexSemanticDirectory,
    cell: DxfPolylinePolygonMeshCellEntry,
) -> Result<DxfPolylinePolygonMeshCellGeometryState, DxfError> {
    let m0_n0 = match position(
        vertices,
        cell.m0_n0(),
        DxfPolylinePolygonMeshCellCorner::M0N0,
    )? {
        Ok(value) => value,
        Err(state) => return Ok(state),
    };
    let m0_n1 = match position(
        vertices,
        cell.m0_n1(),
        DxfPolylinePolygonMeshCellCorner::M0N1,
    )? {
        Ok(value) => value,
        Err(state) => return Ok(state),
    };
    let m1_n1 = match position(
        vertices,
        cell.m1_n1(),
        DxfPolylinePolygonMeshCellCorner::M1N1,
    )? {
        Ok(value) => value,
        Err(state) => return Ok(state),
    };
    let m1_n0 = match position(
        vertices,
        cell.m1_n0(),
        DxfPolylinePolygonMeshCellCorner::M1N0,
    )? {
        Ok(value) => value,
        Err(state) => return Ok(state),
    };
    Ok(DxfPolylinePolygonMeshCellGeometryState::Available {
        m0_n0,
        m0_n1,
        m1_n1,
        m1_n0,
    })
}

fn position(
    vertices: &DxfPolylineVertexSemanticDirectory,
    vertex: DxfPolylineVertexValueEntry,
    corner: DxfPolylinePolygonMeshCellCorner,
) -> Result<Result<[DxfDouble; 3], DxfPolylinePolygonMeshCellGeometryState>, DxfError> {
    let semantics = vertices
        .semantics_for_entry(vertex)?
        .ok_or_else(invalid_internal_data)?;
    Ok(match semantics.position_value() {
        Some(position) => Ok(position),
        None => Err(
            DxfPolylinePolygonMeshCellGeometryState::CoordinateUnavailable {
                corner,
                component_states: [
                    semantics.position()[0].state(),
                    semantics.position()[1].state(),
                    semantics.position()[2].state(),
                ],
            },
        ),
    })
}

fn ensure_not_cancelled(token: &DxfCancellationToken) -> Result<(), DxfError> {
    if token.is_cancelled() {
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

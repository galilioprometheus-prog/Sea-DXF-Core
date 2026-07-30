//! Typed smooth-surface metadata for classic POLYLINE polygon meshes.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPolylinePolygonMeshDirectory, DxfPolylinePolygonMeshRecordEntry,
    DxfPolylinePolygonMeshRecordState, DxfRawDocumentView, DxfSemanticValueState, DxfSourceId,
};

/// One documented classic polygon-mesh smooth-surface type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolygonMeshSmoothSurfaceType {
    None,
    QuadraticBSpline,
    CubicBSpline,
    Bezier,
}

/// Typed smoothing metadata or its exact failure boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolygonMeshSmoothingState {
    Available {
        m_density: i16,
        n_density: i16,
        surface_type: DxfPolylinePolygonMeshSmoothSurfaceType,
    },
    TopologyUnavailable {
        state: DxfPolylinePolygonMeshRecordState,
    },
    MetadataUnavailable {
        component_states: [DxfSemanticValueState; 3],
    },
    UnsupportedSurfaceType {
        value: i16,
        m_density: i16,
        n_density: i16,
    },
}

/// One M9.2o record and its reviewed smooth-surface metadata.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolygonMeshSmoothingEntry {
    record: DxfPolylinePolygonMeshRecordEntry,
    state: DxfPolylinePolygonMeshSmoothingState,
}

impl DxfPolylinePolygonMeshSmoothingEntry {
    #[must_use]
    pub const fn record(self) -> DxfPolylinePolygonMeshRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylinePolygonMeshSmoothingState {
        self.state
    }
}

/// Immutable smoothing metadata retaining M9.2o topology and record semantics.
#[derive(Debug)]
pub struct DxfPolylinePolygonMeshSmoothingDirectory {
    source_id: DxfSourceId,
    topology: DxfPolylinePolygonMeshDirectory,
    entries: Box<[DxfPolylinePolygonMeshSmoothingEntry]>,
}

impl DxfPolylinePolygonMeshSmoothingDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let topology = document.polyline_polygon_mesh_directory(cancellation)?;
        if topology.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: topology.source_id(),
            });
        }
        let mut entries = Vec::new();
        entries
            .try_reserve(topology.records().len())
            .map_err(|_| out_of_memory())?;
        for record in topology.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfPolylinePolygonMeshSmoothingEntry {
                record,
                state: smoothing_state(&topology, record)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            topology,
            entries: entries.into_boxed_slice(),
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
    pub fn entries(&self) -> &[DxfPolylinePolygonMeshSmoothingEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_polyline_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<DxfPolylinePolygonMeshSmoothingEntry> {
        self.entries
            .binary_search_by_key(&raw, |entry| {
                entry
                    .record()
                    .record()
                    .sequence()
                    .polyline_record()
                    .ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_polygon_mesh_smoothing_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshSmoothingDirectory, DxfError> {
        DxfPolylinePolygonMeshSmoothingDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_polygon_mesh_smoothing_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshSmoothingDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polygon_mesh_smoothing_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_polygon_mesh_smoothing_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshSmoothingDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polygon_mesh_smoothing_directory(cancellation)
    }
}

fn smoothing_state(
    topology: &DxfPolylinePolygonMeshDirectory,
    record: DxfPolylinePolygonMeshRecordEntry,
) -> Result<DxfPolylinePolygonMeshSmoothingState, DxfError> {
    if !matches!(
        record.state(),
        DxfPolylinePolygonMeshRecordState::Available { .. }
    ) {
        return Ok(DxfPolylinePolygonMeshSmoothingState::TopologyUnavailable {
            state: record.state(),
        });
    }
    let semantics = topology
        .family_semantic_directory()
        .record_semantic_directory()
        .semantics_for_polyline_raw_ordinal(record.record().sequence().polyline_record().ordinal())?
        .ok_or_else(invalid_internal_data)?;
    let m = semantics.smooth_surface_m_density();
    let n = semantics.smooth_surface_n_density();
    let kind = semantics.smooth_surface_type();
    let (Some(m_density), Some(n_density), Some(kind_value)) = (
        m.value().copied(),
        n.value().copied(),
        kind.value().copied(),
    ) else {
        return Ok(DxfPolylinePolygonMeshSmoothingState::MetadataUnavailable {
            component_states: [m.state(), n.state(), kind.state()],
        });
    };
    let surface_type = match kind_value {
        0 => DxfPolylinePolygonMeshSmoothSurfaceType::None,
        5 => DxfPolylinePolygonMeshSmoothSurfaceType::QuadraticBSpline,
        6 => DxfPolylinePolygonMeshSmoothSurfaceType::CubicBSpline,
        8 => DxfPolylinePolygonMeshSmoothSurfaceType::Bezier,
        value => {
            return Ok(
                DxfPolylinePolygonMeshSmoothingState::UnsupportedSurfaceType {
                    value,
                    m_density,
                    n_density,
                },
            );
        }
    };
    Ok(DxfPolylinePolygonMeshSmoothingState::Available {
        m_density,
        n_density,
        surface_type,
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

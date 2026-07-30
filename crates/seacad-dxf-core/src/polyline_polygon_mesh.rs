//! Fail-closed row-major topology for classic POLYLINE polygon meshes.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPolylineFamily, DxfPolylineFamilySemanticDirectory, DxfPolylineFamilyState,
    DxfPolylineRecordValueEntry, DxfPolylineSequenceState, DxfPolylineVertexFamilyComparison,
    DxfPolylineVertexValueEntry, DxfRawDocumentView, DxfSourceId,
};

/// Why one classic POLYLINE record does or does not expose polygon-mesh cells.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolygonMeshRecordState {
    Available {
        m_vertex_count: u16,
        n_vertex_count: u16,
        closed_m: bool,
        closed_n: bool,
    },
    IncompleteSequence {
        sequence_state: DxfPolylineSequenceState,
    },
    UnsupportedFamily {
        family: DxfPolylineFamily,
    },
    IndeterminateFamily,
    InconsistentVertex {
        sequence_vertex_ordinal: u32,
    },
    GridCountsUnavailable,
    NonPositiveGridCount {
        m_vertex_count: i16,
        n_vertex_count: i16,
    },
    VertexCountMismatch {
        expected: u32,
        observed: u32,
    },
}

/// Half-open cell range owned by one classic polygon-mesh record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolygonMeshCellRange {
    start: u32,
    end: u32,
}

impl DxfPolylinePolygonMeshCellRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// One classic POLYLINE record and its guaranteed polygon-mesh cell range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolygonMeshRecordEntry {
    record: DxfPolylineRecordValueEntry,
    state: DxfPolylinePolygonMeshRecordState,
    cell_range: DxfPolylinePolygonMeshCellRange,
}

impl DxfPolylinePolygonMeshRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylinePolygonMeshRecordState {
        self.state
    }

    #[must_use]
    pub const fn cell_range(self) -> DxfPolylinePolygonMeshCellRange {
        self.cell_range
    }
}

/// One row-major quadrilateral cell referencing exact retained VERTEX entries.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolygonMeshCellEntry {
    ordinal: u32,
    record_cell_ordinal: u32,
    record: DxfPolylineRecordValueEntry,
    m_index: u16,
    n_index: u16,
    wraps_m: bool,
    wraps_n: bool,
    m0_n0: DxfPolylineVertexValueEntry,
    m0_n1: DxfPolylineVertexValueEntry,
    m1_n1: DxfPolylineVertexValueEntry,
    m1_n0: DxfPolylineVertexValueEntry,
}

impl DxfPolylinePolygonMeshCellEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }
    #[must_use]
    pub const fn record_cell_ordinal(self) -> u64 {
        self.record_cell_ordinal as u64
    }
    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }
    #[must_use]
    pub const fn m_index(self) -> u16 {
        self.m_index
    }
    #[must_use]
    pub const fn n_index(self) -> u16 {
        self.n_index
    }
    #[must_use]
    pub const fn wraps_m(self) -> bool {
        self.wraps_m
    }
    #[must_use]
    pub const fn wraps_n(self) -> bool {
        self.wraps_n
    }
    #[must_use]
    pub const fn m0_n0(self) -> DxfPolylineVertexValueEntry {
        self.m0_n0
    }
    #[must_use]
    pub const fn m0_n1(self) -> DxfPolylineVertexValueEntry {
        self.m0_n1
    }
    #[must_use]
    pub const fn m1_n1(self) -> DxfPolylineVertexValueEntry {
        self.m1_n1
    }
    #[must_use]
    pub const fn m1_n0(self) -> DxfPolylineVertexValueEntry {
        self.m1_n0
    }
}

/// Immutable polygon-mesh topology retaining the M9.2i family graph.
#[derive(Debug)]
pub struct DxfPolylinePolygonMeshDirectory {
    source_id: DxfSourceId,
    families: DxfPolylineFamilySemanticDirectory,
    records: Box<[DxfPolylinePolygonMeshRecordEntry]>,
    cells: Box<[DxfPolylinePolygonMeshCellEntry]>,
}

impl DxfPolylinePolygonMeshDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let families = document.polyline_family_semantic_directory(cancellation)?;
        if families.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: families.source_id(),
            });
        }
        let mut records = Vec::new();
        let mut cells = Vec::new();
        for record in families.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(cells.len())?;
            let state = record_state(&families, record)?;
            if matches!(state, DxfPolylinePolygonMeshRecordState::Available { .. }) {
                let vertices = vertices_for_record(&families, record)?;
                build_cells(&mut cells, record, start, vertices, state)?;
            }
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfPolylinePolygonMeshRecordEntry {
                record,
                state,
                cell_range: DxfPolylinePolygonMeshCellRange::new(start, compact_len(cells.len())?)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            families,
            records: records.into_boxed_slice(),
            cells: cells.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }
    #[must_use]
    pub const fn family_semantic_directory(&self) -> &DxfPolylineFamilySemanticDirectory {
        &self.families
    }
    #[must_use]
    pub fn records(&self) -> &[DxfPolylinePolygonMeshRecordEntry] {
        &self.records
    }
    #[must_use]
    pub fn cells(&self) -> &[DxfPolylinePolygonMeshCellEntry] {
        &self.cells
    }
    #[must_use]
    pub fn record_for_polyline_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<DxfPolylinePolygonMeshRecordEntry> {
        self.records
            .binary_search_by_key(&raw, |entry| {
                entry.record().sequence().polyline_record().ordinal()
            })
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }
    #[must_use]
    pub fn cell(&self, ordinal: u64) -> Option<DxfPolylinePolygonMeshCellEntry> {
        self.cells.get(usize::try_from(ordinal).ok()?).copied()
    }
    #[must_use]
    pub fn cells_for_polyline_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<&[DxfPolylinePolygonMeshCellEntry]> {
        let range = self.record_for_polyline_raw_ordinal(raw)?.cell_range();
        self.cells
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_polygon_mesh_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshDirectory, DxfError> {
        DxfPolylinePolygonMeshDirectory::from_document(self, cancellation)
    }
}
impl DxfAsciiRawDocument<'_> {
    pub fn polyline_polygon_mesh_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polygon_mesh_directory(cancellation)
    }
}
impl DxfBinaryRawDocument<'_> {
    pub fn polyline_polygon_mesh_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolygonMeshDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polygon_mesh_directory(cancellation)
    }
}

fn record_state(
    families: &DxfPolylineFamilySemanticDirectory,
    record: DxfPolylineRecordValueEntry,
) -> Result<DxfPolylinePolygonMeshRecordState, DxfError> {
    if record.sequence().state() != DxfPolylineSequenceState::Closed {
        return Ok(DxfPolylinePolygonMeshRecordState::IncompleteSequence {
            sequence_state: record.sequence().state(),
        });
    }
    let family = families
        .polyline_semantics_for_raw_ordinal(record.sequence().polyline_record().ordinal())?
        .ok_or_else(invalid_internal_data)?
        .family();
    match family {
        DxfPolylineFamilyState::Classified(DxfPolylineFamily::PolygonMesh) => {}
        DxfPolylineFamilyState::Classified(family) => {
            return Ok(DxfPolylinePolygonMeshRecordState::UnsupportedFamily { family });
        }
        DxfPolylineFamilyState::Unavailable | DxfPolylineFamilyState::Conflicting { .. } => {
            return Ok(DxfPolylinePolygonMeshRecordState::IndeterminateFamily);
        }
    }
    for vertex in families.vertices().iter().filter(|vertex| {
        vertex.polyline_record().ordinal() == record.sequence().polyline_record().ordinal()
    }) {
        if !matches!(
            families
                .vertex_semantics_for_raw_ordinal(vertex.vertex_record().ordinal())?
                .ok_or_else(invalid_internal_data)?
                .comparison(),
            DxfPolylineVertexFamilyComparison::Matched { .. }
        ) {
            return Ok(DxfPolylinePolygonMeshRecordState::InconsistentVertex {
                sequence_vertex_ordinal: u32::try_from(vertex.sequence_vertex_ordinal())
                    .map_err(|_| invalid_internal_data())?,
            });
        }
    }
    let semantics = families
        .record_semantic_directory()
        .semantics_for_polyline_raw_ordinal(record.sequence().polyline_record().ordinal())?
        .ok_or_else(invalid_internal_data)?;
    let Some(m) = semantics.mesh_m_vertex_count().value().copied() else {
        return Ok(DxfPolylinePolygonMeshRecordState::GridCountsUnavailable);
    };
    let Some(n) = semantics.mesh_n_vertex_count().value().copied() else {
        return Ok(DxfPolylinePolygonMeshRecordState::GridCountsUnavailable);
    };
    if m <= 0 || n <= 0 {
        return Ok(DxfPolylinePolygonMeshRecordState::NonPositiveGridCount {
            m_vertex_count: m,
            n_vertex_count: n,
        });
    }
    let expected = u32::from(u16::try_from(m).map_err(|_| invalid_internal_data())?)
        .checked_mul(u32::from(
            u16::try_from(n).map_err(|_| invalid_internal_data())?,
        ))
        .ok_or_else(invalid_internal_data)?;
    let observed = compact_len(vertices_for_record(families, record)?.len())?;
    if expected != observed {
        return Ok(DxfPolylinePolygonMeshRecordState::VertexCountMismatch { expected, observed });
    }
    Ok(DxfPolylinePolygonMeshRecordState::Available {
        m_vertex_count: u16::try_from(m).map_err(|_| invalid_internal_data())?,
        n_vertex_count: u16::try_from(n).map_err(|_| invalid_internal_data())?,
        closed_m: semantics
            .is_closed_or_mesh_closed_m()
            .ok_or_else(invalid_internal_data)?,
        closed_n: semantics
            .is_mesh_closed_n()
            .ok_or_else(invalid_internal_data)?,
    })
}

fn vertices_for_record(
    families: &DxfPolylineFamilySemanticDirectory,
    record: DxfPolylineRecordValueEntry,
) -> Result<&[DxfPolylineVertexValueEntry], DxfError> {
    families
        .vertex_integer_semantic_directory()
        .card_directory()
        .evidence_directory()
        .vertices_for_polyline_raw_ordinal(record.sequence().polyline_record().ordinal())
        .ok_or_else(invalid_internal_data)
}

fn build_cells(
    cells: &mut Vec<DxfPolylinePolygonMeshCellEntry>,
    record: DxfPolylineRecordValueEntry,
    start: u32,
    vertices: &[DxfPolylineVertexValueEntry],
    state: DxfPolylinePolygonMeshRecordState,
) -> Result<(), DxfError> {
    let DxfPolylinePolygonMeshRecordState::Available {
        m_vertex_count: m_count,
        n_vertex_count: n_count,
        closed_m,
        closed_n,
    } = state
    else {
        return Err(invalid_internal_data());
    };
    let m_cells = if closed_m {
        m_count
    } else {
        m_count.saturating_sub(1)
    };
    let n_cells = if closed_n {
        n_count
    } else {
        n_count.saturating_sub(1)
    };
    for m in 0..m_cells {
        for n in 0..n_cells {
            let m1 = if m + 1 == m_count { 0 } else { m + 1 };
            let n1 = if n + 1 == n_count { 0 } else { n + 1 };
            let at = |mi: u16, ni: u16| -> Result<DxfPolylineVertexValueEntry, DxfError> {
                vertices
                    .get(usize::from(mi) * usize::from(n_count) + usize::from(ni))
                    .copied()
                    .ok_or_else(invalid_internal_data)
            };
            let ordinal = compact_len(cells.len())?;
            cells.try_reserve(1).map_err(|_| out_of_memory())?;
            cells.push(DxfPolylinePolygonMeshCellEntry {
                ordinal,
                record_cell_ordinal: ordinal
                    .checked_sub(start)
                    .ok_or_else(invalid_internal_data)?,
                record,
                m_index: m,
                n_index: n,
                wraps_m: m1 == 0,
                wraps_n: n1 == 0,
                m0_n0: at(m, n)?,
                m0_n1: at(m, n1)?,
                m1_n1: at(m1, n1)?,
                m1_n0: at(m1, n)?,
            });
        }
    }
    Ok(())
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
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

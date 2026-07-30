//! Tolerant coordinate/face partitioning for classic POLYLINE polyface meshes.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPolylineFamily, DxfPolylineFamilySemanticDirectory, DxfPolylineFamilyState,
    DxfPolylineRecordValueEntry, DxfPolylineSequenceState, DxfPolylineVertexFamily,
    DxfPolylineVertexFamilyComparison, DxfPolylineVertexValueEntry, DxfRawDocumentView,
    DxfSourceId,
};

/// Whether retained coordinate and face records use the conventional order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolyfaceOrdering {
    CoordinatesThenFaces,
    Odd,
}

/// Why one classic POLYLINE record does or does not expose a polyface partition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolyfaceRecordState {
    Available {
        reported_coordinate_count: Option<i16>,
        reported_face_count: Option<i16>,
        observed_coordinate_count: u32,
        observed_face_count: u32,
        ordering: DxfPolylinePolyfaceOrdering,
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
}

/// Half-open member range owned by one classic polyface record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceMemberRange {
    start: u32,
    end: u32,
}

impl DxfPolylinePolyfaceMemberRange {
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

/// One classic POLYLINE record and its coordinate/face member ranges.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceRecordEntry {
    record: DxfPolylineRecordValueEntry,
    state: DxfPolylinePolyfaceRecordState,
    coordinate_range: DxfPolylinePolyfaceMemberRange,
    face_range: DxfPolylinePolyfaceMemberRange,
}

impl DxfPolylinePolyfaceRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylinePolyfaceRecordState {
        self.state
    }

    #[must_use]
    pub const fn coordinate_range(self) -> DxfPolylinePolyfaceMemberRange {
        self.coordinate_range
    }

    #[must_use]
    pub const fn face_range(self) -> DxfPolylinePolyfaceMemberRange {
        self.face_range
    }
}

/// One exact retained VERTEX classified as a polyface coordinate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceCoordinateEntry {
    ordinal: u32,
    record_coordinate_ordinal: u32,
    record: DxfPolylineRecordValueEntry,
    vertex: DxfPolylineVertexValueEntry,
}

impl DxfPolylinePolyfaceCoordinateEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record_coordinate_ordinal(self) -> u64 {
        self.record_coordinate_ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn vertex(self) -> DxfPolylineVertexValueEntry {
        self.vertex
    }
}

/// One exact retained VERTEX classified as a polyface face definition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceFaceEntry {
    ordinal: u32,
    record_face_ordinal: u32,
    record: DxfPolylineRecordValueEntry,
    vertex: DxfPolylineVertexValueEntry,
}

impl DxfPolylinePolyfaceFaceEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record_face_ordinal(self) -> u64 {
        self.record_face_ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn vertex(self) -> DxfPolylineVertexValueEntry {
        self.vertex
    }
}

/// Immutable tolerant polyface partition retaining the M9.2i family graph.
#[derive(Debug)]
pub struct DxfPolylinePolyfaceTopologyDirectory {
    source_id: DxfSourceId,
    families: DxfPolylineFamilySemanticDirectory,
    records: Box<[DxfPolylinePolyfaceRecordEntry]>,
    coordinates: Box<[DxfPolylinePolyfaceCoordinateEntry]>,
    faces: Box<[DxfPolylinePolyfaceFaceEntry]>,
}

impl DxfPolylinePolyfaceTopologyDirectory {
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
        let mut coordinates = Vec::new();
        let mut faces = Vec::new();
        for record in families.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let coordinate_start = compact_len(coordinates.len())?;
            let face_start = compact_len(faces.len())?;
            let state = record_state(&families, record)?;
            if matches!(state, DxfPolylinePolyfaceRecordState::Available { .. }) {
                build_partition(
                    &families,
                    record,
                    coordinate_start,
                    face_start,
                    &mut coordinates,
                    &mut faces,
                )?;
            }
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfPolylinePolyfaceRecordEntry {
                record,
                state,
                coordinate_range: DxfPolylinePolyfaceMemberRange::new(
                    coordinate_start,
                    compact_len(coordinates.len())?,
                )?,
                face_range: DxfPolylinePolyfaceMemberRange::new(
                    face_start,
                    compact_len(faces.len())?,
                )?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            families,
            records: records.into_boxed_slice(),
            coordinates: coordinates.into_boxed_slice(),
            faces: faces.into_boxed_slice(),
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
    pub fn records(&self) -> &[DxfPolylinePolyfaceRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn coordinates(&self) -> &[DxfPolylinePolyfaceCoordinateEntry] {
        &self.coordinates
    }

    #[must_use]
    pub fn faces(&self) -> &[DxfPolylinePolyfaceFaceEntry] {
        &self.faces
    }

    #[must_use]
    pub fn record_for_polyline_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<DxfPolylinePolyfaceRecordEntry> {
        self.records
            .binary_search_by_key(&raw, |entry| {
                entry.record().sequence().polyline_record().ordinal()
            })
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn coordinate(&self, ordinal: u64) -> Option<DxfPolylinePolyfaceCoordinateEntry> {
        self.coordinates
            .get(usize::try_from(ordinal).ok()?)
            .copied()
    }

    #[must_use]
    pub fn face(&self, ordinal: u64) -> Option<DxfPolylinePolyfaceFaceEntry> {
        self.faces.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn coordinates_for_polyline_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<&[DxfPolylinePolyfaceCoordinateEntry]> {
        let range = self
            .record_for_polyline_raw_ordinal(raw)?
            .coordinate_range();
        self.coordinates
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }

    #[must_use]
    pub fn faces_for_polyline_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<&[DxfPolylinePolyfaceFaceEntry]> {
        let range = self.record_for_polyline_raw_ordinal(raw)?.face_range();
        self.faces
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_polyface_topology_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceTopologyDirectory, DxfError> {
        DxfPolylinePolyfaceTopologyDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_polyface_topology_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceTopologyDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polyface_topology_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_polyface_topology_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceTopologyDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polyface_topology_directory(cancellation)
    }
}

fn record_state(
    families: &DxfPolylineFamilySemanticDirectory,
    record: DxfPolylineRecordValueEntry,
) -> Result<DxfPolylinePolyfaceRecordState, DxfError> {
    if record.sequence().state() != DxfPolylineSequenceState::Closed {
        return Ok(DxfPolylinePolyfaceRecordState::IncompleteSequence {
            sequence_state: record.sequence().state(),
        });
    }
    let family = families
        .polyline_semantics_for_raw_ordinal(record.sequence().polyline_record().ordinal())?
        .ok_or_else(invalid_internal_data)?
        .family();
    match family {
        DxfPolylineFamilyState::Classified(DxfPolylineFamily::PolyfaceMesh) => {}
        DxfPolylineFamilyState::Classified(family) => {
            return Ok(DxfPolylinePolyfaceRecordState::UnsupportedFamily { family });
        }
        DxfPolylineFamilyState::Unavailable | DxfPolylineFamilyState::Conflicting { .. } => {
            return Ok(DxfPolylinePolyfaceRecordState::IndeterminateFamily);
        }
    }
    let vertices = vertices_for_record(families, record)?;
    let mut coordinate_count = 0_u32;
    let mut face_count = 0_u32;
    let mut saw_face = false;
    let mut ordering = DxfPolylinePolyfaceOrdering::CoordinatesThenFaces;
    for vertex in vertices {
        match matched_vertex_family(families, *vertex)? {
            Some(DxfPolylineVertexFamily::PolyfaceCoordinate) => {
                coordinate_count = coordinate_count
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
                if saw_face {
                    ordering = DxfPolylinePolyfaceOrdering::Odd;
                }
            }
            Some(DxfPolylineVertexFamily::PolyfaceFace) => {
                face_count = face_count
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
                saw_face = true;
            }
            _ => {
                return Ok(DxfPolylinePolyfaceRecordState::InconsistentVertex {
                    sequence_vertex_ordinal: u32::try_from(vertex.sequence_vertex_ordinal())
                        .map_err(|_| invalid_internal_data())?,
                });
            }
        }
    }
    let semantics = families
        .record_semantic_directory()
        .semantics_for_polyline_raw_ordinal(record.sequence().polyline_record().ordinal())?
        .ok_or_else(invalid_internal_data)?;
    let reported_coordinate_count = semantics.mesh_m_vertex_count();
    let reported_face_count = semantics.mesh_n_vertex_count();
    Ok(DxfPolylinePolyfaceRecordState::Available {
        reported_coordinate_count: reported_coordinate_count
            .raw_provenance()
            .and_then(|_| reported_coordinate_count.value().copied()),
        reported_face_count: reported_face_count
            .raw_provenance()
            .and_then(|_| reported_face_count.value().copied()),
        observed_coordinate_count: coordinate_count,
        observed_face_count: face_count,
        ordering,
    })
}

fn build_partition(
    families: &DxfPolylineFamilySemanticDirectory,
    record: DxfPolylineRecordValueEntry,
    coordinate_start: u32,
    face_start: u32,
    coordinates: &mut Vec<DxfPolylinePolyfaceCoordinateEntry>,
    faces: &mut Vec<DxfPolylinePolyfaceFaceEntry>,
) -> Result<(), DxfError> {
    for vertex in vertices_for_record(families, record)? {
        match matched_vertex_family(families, *vertex)? {
            Some(DxfPolylineVertexFamily::PolyfaceCoordinate) => {
                let ordinal = compact_len(coordinates.len())?;
                coordinates.try_reserve(1).map_err(|_| out_of_memory())?;
                coordinates.push(DxfPolylinePolyfaceCoordinateEntry {
                    ordinal,
                    record_coordinate_ordinal: ordinal
                        .checked_sub(coordinate_start)
                        .ok_or_else(invalid_internal_data)?,
                    record,
                    vertex: *vertex,
                });
            }
            Some(DxfPolylineVertexFamily::PolyfaceFace) => {
                let ordinal = compact_len(faces.len())?;
                faces.try_reserve(1).map_err(|_| out_of_memory())?;
                faces.push(DxfPolylinePolyfaceFaceEntry {
                    ordinal,
                    record_face_ordinal: ordinal
                        .checked_sub(face_start)
                        .ok_or_else(invalid_internal_data)?,
                    record,
                    vertex: *vertex,
                });
            }
            _ => return Err(invalid_internal_data()),
        }
    }
    Ok(())
}

fn matched_vertex_family(
    families: &DxfPolylineFamilySemanticDirectory,
    vertex: DxfPolylineVertexValueEntry,
) -> Result<Option<DxfPolylineVertexFamily>, DxfError> {
    let semantics = families
        .vertex_semantics_for_raw_ordinal(vertex.vertex_record().ordinal())?
        .ok_or_else(invalid_internal_data)?;
    match semantics.comparison() {
        DxfPolylineVertexFamilyComparison::Matched {
            polyline: DxfPolylineFamily::PolyfaceMesh,
            vertex,
        } => Ok(Some(vertex)),
        _ => Ok(None),
    }
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

//! Fail-closed classic POLYLINE polyface index resolution.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPolylinePolyfaceCoordinateEntry, DxfPolylinePolyfaceFaceEntry,
    DxfPolylinePolyfaceTopologyDirectory, DxfRawDocumentView, DxfSemanticValueState, DxfSourceId,
};

/// Why one polyface face does or does not expose resolved corners.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolyfaceFaceResolutionState {
    Available {
        corner_count: u8,
    },
    InvalidIndex {
        slot: u8,
        state: DxfSemanticValueState,
    },
    NonZeroAfterTerminator {
        slot: u8,
        value: i16,
    },
    IndexMagnitudeOverflow {
        slot: u8,
    },
    CoordinateIndexOutOfRange {
        slot: u8,
        index: u16,
        coordinate_count: u32,
    },
}

/// Half-open resolved-corner range owned by one polyface face.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceCornerRange {
    start: u32,
    end: u32,
}

impl DxfPolylinePolyfaceCornerRange {
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

/// One exact polyface face VERTEX and its resolution state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceResolvedFaceEntry {
    face: DxfPolylinePolyfaceFaceEntry,
    state: DxfPolylinePolyfaceFaceResolutionState,
    corner_range: DxfPolylinePolyfaceCornerRange,
}

impl DxfPolylinePolyfaceResolvedFaceEntry {
    #[must_use]
    pub const fn face(self) -> DxfPolylinePolyfaceFaceEntry {
        self.face
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylinePolyfaceFaceResolutionState {
        self.state
    }

    #[must_use]
    pub const fn corner_range(self) -> DxfPolylinePolyfaceCornerRange {
        self.corner_range
    }
}

/// One resolved face corner retaining the signed source index and edge state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceResolvedCornerEntry {
    ordinal: u32,
    face_corner_ordinal: u8,
    face: DxfPolylinePolyfaceFaceEntry,
    coordinate: DxfPolylinePolyfaceCoordinateEntry,
    source_index: i16,
    edge_visible: bool,
}

impl DxfPolylinePolyfaceResolvedCornerEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn face_corner_ordinal(self) -> u8 {
        self.face_corner_ordinal
    }

    #[must_use]
    pub const fn face(self) -> DxfPolylinePolyfaceFaceEntry {
        self.face
    }

    #[must_use]
    pub const fn coordinate(self) -> DxfPolylinePolyfaceCoordinateEntry {
        self.coordinate
    }

    #[must_use]
    pub const fn source_index(self) -> i16 {
        self.source_index
    }

    #[must_use]
    pub const fn edge_visible(self) -> bool {
        self.edge_visible
    }
}

/// Immutable face resolution retaining M9.2p topology and exact VERTEX evidence.
#[derive(Debug)]
pub struct DxfPolylinePolyfaceFaceResolutionDirectory {
    source_id: DxfSourceId,
    topology: DxfPolylinePolyfaceTopologyDirectory,
    faces: Box<[DxfPolylinePolyfaceResolvedFaceEntry]>,
    corners: Box<[DxfPolylinePolyfaceResolvedCornerEntry]>,
}

impl DxfPolylinePolyfaceFaceResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let topology = document.polyline_polyface_topology_directory(cancellation)?;
        if topology.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: topology.source_id(),
            });
        }
        let mut faces = Vec::new();
        let mut corners = Vec::new();
        for face in topology.faces().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(corners.len())?;
            let coordinates = topology
                .coordinates_for_polyline_raw_ordinal(
                    face.record().sequence().polyline_record().ordinal(),
                )
                .ok_or_else(invalid_internal_data)?;
            let (state, resolved) = resolve_face(&topology, face, coordinates)?;
            if matches!(
                state,
                DxfPolylinePolyfaceFaceResolutionState::Available { .. }
            ) {
                append_corners(&mut corners, face, resolved)?;
            }
            faces.try_reserve(1).map_err(|_| out_of_memory())?;
            faces.push(DxfPolylinePolyfaceResolvedFaceEntry {
                face,
                state,
                corner_range: DxfPolylinePolyfaceCornerRange::new(
                    start,
                    compact_len(corners.len())?,
                )?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            topology,
            faces: faces.into_boxed_slice(),
            corners: corners.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn topology_directory(&self) -> &DxfPolylinePolyfaceTopologyDirectory {
        &self.topology
    }

    #[must_use]
    pub fn faces(&self) -> &[DxfPolylinePolyfaceResolvedFaceEntry] {
        &self.faces
    }

    #[must_use]
    pub fn corners(&self) -> &[DxfPolylinePolyfaceResolvedCornerEntry] {
        &self.corners
    }

    #[must_use]
    pub fn face(&self, ordinal: u64) -> Option<DxfPolylinePolyfaceResolvedFaceEntry> {
        self.faces.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn corner(&self, ordinal: u64) -> Option<DxfPolylinePolyfaceResolvedCornerEntry> {
        self.corners.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn face_for_vertex_raw_ordinal(
        &self,
        raw: u64,
    ) -> Option<DxfPolylinePolyfaceResolvedFaceEntry> {
        self.faces
            .binary_search_by_key(&raw, |entry| {
                entry.face().vertex().vertex_record().ordinal()
            })
            .ok()
            .and_then(|index| self.faces.get(index).copied())
    }

    #[must_use]
    pub fn corners_for_face(
        &self,
        face_ordinal: u64,
    ) -> Option<&[DxfPolylinePolyfaceResolvedCornerEntry]> {
        let range = self.face(face_ordinal)?.corner_range();
        self.corners
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_polyface_face_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceFaceResolutionDirectory, DxfError> {
        DxfPolylinePolyfaceFaceResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_polyface_face_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceFaceResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polyface_face_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_polyface_face_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceFaceResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polyface_face_resolution_directory(cancellation)
    }
}

type ResolvedCorner = (DxfPolylinePolyfaceCoordinateEntry, i16);

fn resolve_face(
    topology: &DxfPolylinePolyfaceTopologyDirectory,
    face: DxfPolylinePolyfaceFaceEntry,
    coordinates: &[DxfPolylinePolyfaceCoordinateEntry],
) -> Result<
    (
        DxfPolylinePolyfaceFaceResolutionState,
        [Option<ResolvedCorner>; 4],
    ),
    DxfError,
> {
    let semantics = topology
        .family_semantic_directory()
        .vertex_integer_semantic_directory()
        .semantics_for_entry(face.vertex())?
        .ok_or_else(invalid_internal_data)?;
    let mut resolved = [None; 4];
    let mut terminated = false;
    let mut count = 0_u8;
    for (slot, index) in semantics.polyface_vertex_indices().iter().enumerate() {
        let slot = u8::try_from(slot).map_err(|_| invalid_internal_data())?;
        let value = match index.state() {
            DxfSemanticValueState::Explicit => *index.value().ok_or_else(invalid_internal_data)?,
            DxfSemanticValueState::Absent => {
                terminated = true;
                continue;
            }
            state => {
                return Ok((
                    DxfPolylinePolyfaceFaceResolutionState::InvalidIndex { slot, state },
                    resolved,
                ));
            }
        };
        if value == 0 {
            terminated = true;
            continue;
        }
        if terminated {
            return Ok((
                DxfPolylinePolyfaceFaceResolutionState::NonZeroAfterTerminator { slot, value },
                resolved,
            ));
        }
        let Some(magnitude) = value.checked_abs() else {
            return Ok((
                DxfPolylinePolyfaceFaceResolutionState::IndexMagnitudeOverflow { slot },
                resolved,
            ));
        };
        let index = u16::try_from(magnitude).map_err(|_| invalid_internal_data())?;
        let Some(coordinate) = usize::from(index)
            .checked_sub(1)
            .and_then(|at| coordinates.get(at))
            .copied()
        else {
            return Ok((
                DxfPolylinePolyfaceFaceResolutionState::CoordinateIndexOutOfRange {
                    slot,
                    index,
                    coordinate_count: compact_len(coordinates.len())?,
                },
                resolved,
            ));
        };
        resolved[usize::from(slot)] = Some((coordinate, value));
        count = count.checked_add(1).ok_or_else(invalid_internal_data)?;
    }
    Ok((
        DxfPolylinePolyfaceFaceResolutionState::Available {
            corner_count: count,
        },
        resolved,
    ))
}

fn append_corners(
    corners: &mut Vec<DxfPolylinePolyfaceResolvedCornerEntry>,
    face: DxfPolylinePolyfaceFaceEntry,
    resolved: [Option<ResolvedCorner>; 4],
) -> Result<(), DxfError> {
    for (face_corner_ordinal, corner) in resolved.into_iter().flatten().enumerate() {
        let ordinal = compact_len(corners.len())?;
        corners.try_reserve(1).map_err(|_| out_of_memory())?;
        corners.push(DxfPolylinePolyfaceResolvedCornerEntry {
            ordinal,
            face_corner_ordinal: u8::try_from(face_corner_ordinal)
                .map_err(|_| invalid_internal_data())?,
            face,
            coordinate: corner.0,
            source_index: corner.1,
            edge_visible: corner.1 > 0,
        });
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

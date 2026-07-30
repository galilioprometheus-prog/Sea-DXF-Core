//! Exact WCS coordinate assembly for resolved classic POLYLINE polyface faces.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfPolylinePolyfaceFaceResolutionDirectory,
    DxfPolylinePolyfaceFaceResolutionState, DxfPolylinePolyfaceResolvedCornerEntry,
    DxfPolylinePolyfaceResolvedFaceEntry, DxfPolylineVertexSemanticDirectory, DxfRawDocumentView,
    DxfSemanticValueState, DxfSourceId,
};

/// Why one resolved polyface face does or does not expose WCS points.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylinePolyfaceFaceGeometryState {
    Available {
        point_count: u8,
    },
    FaceResolutionFailure {
        state: DxfPolylinePolyfaceFaceResolutionState,
    },
    CoordinateUnavailable {
        face_corner_ordinal: u8,
        component_states: [DxfSemanticValueState; 3],
    },
}

/// Half-open WCS point range owned by one polyface face.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfacePointRange {
    start: u32,
    end: u32,
}

impl DxfPolylinePolyfacePointRange {
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

/// One resolved polyface face and its exact WCS point range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfaceFaceGeometryEntry {
    face: DxfPolylinePolyfaceResolvedFaceEntry,
    state: DxfPolylinePolyfaceFaceGeometryState,
    point_range: DxfPolylinePolyfacePointRange,
}

impl DxfPolylinePolyfaceFaceGeometryEntry {
    #[must_use]
    pub const fn face(self) -> DxfPolylinePolyfaceResolvedFaceEntry {
        self.face
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylinePolyfaceFaceGeometryState {
        self.state
    }

    #[must_use]
    pub const fn point_range(self) -> DxfPolylinePolyfacePointRange {
        self.point_range
    }
}

/// One exact WCS point attached to its resolved face corner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylinePolyfacePointEntry {
    ordinal: u32,
    face_point_ordinal: u8,
    corner: DxfPolylinePolyfaceResolvedCornerEntry,
    position: [DxfDouble; 3],
}

impl DxfPolylinePolyfacePointEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn face_point_ordinal(self) -> u8 {
        self.face_point_ordinal
    }

    #[must_use]
    pub const fn corner(self) -> DxfPolylinePolyfaceResolvedCornerEntry {
        self.corner
    }

    #[must_use]
    pub const fn position(self) -> [DxfDouble; 3] {
        self.position
    }
}

/// Immutable exact WCS assembly retaining face and vertex semantic evidence.
#[derive(Debug)]
pub struct DxfPolylinePolyfaceFaceGeometryDirectory {
    source_id: DxfSourceId,
    resolution: DxfPolylinePolyfaceFaceResolutionDirectory,
    vertices: DxfPolylineVertexSemanticDirectory,
    faces: Box<[DxfPolylinePolyfaceFaceGeometryEntry]>,
    points: Box<[DxfPolylinePolyfacePointEntry]>,
}

impl DxfPolylinePolyfaceFaceGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let resolution = document.polyline_polyface_face_resolution_directory(cancellation)?;
        let vertices = document.polyline_vertex_semantic_directory(cancellation)?;
        if resolution.source_id() != document.source_id()
            || vertices.source_id() != document.source_id()
        {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: resolution.source_id(),
            });
        }
        let mut faces = Vec::new();
        let mut points = Vec::new();
        for face in resolution.faces().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(points.len())?;
            let (state, resolved) = resolve_points(&resolution, &vertices, face)?;
            if matches!(
                state,
                DxfPolylinePolyfaceFaceGeometryState::Available { .. }
            ) {
                append_points(&mut points, resolved)?;
            }
            faces.try_reserve(1).map_err(|_| out_of_memory())?;
            faces.push(DxfPolylinePolyfaceFaceGeometryEntry {
                face,
                state,
                point_range: DxfPolylinePolyfacePointRange::new(start, compact_len(points.len())?)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            resolution,
            vertices,
            faces: faces.into_boxed_slice(),
            points: points.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn face_resolution_directory(&self) -> &DxfPolylinePolyfaceFaceResolutionDirectory {
        &self.resolution
    }

    #[must_use]
    pub const fn vertex_semantic_directory(&self) -> &DxfPolylineVertexSemanticDirectory {
        &self.vertices
    }

    #[must_use]
    pub fn faces(&self) -> &[DxfPolylinePolyfaceFaceGeometryEntry] {
        &self.faces
    }

    #[must_use]
    pub fn points(&self) -> &[DxfPolylinePolyfacePointEntry] {
        &self.points
    }

    #[must_use]
    pub fn face(&self, ordinal: u64) -> Option<DxfPolylinePolyfaceFaceGeometryEntry> {
        self.faces.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn point(&self, ordinal: u64) -> Option<DxfPolylinePolyfacePointEntry> {
        self.points.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn points_for_face(&self, face_ordinal: u64) -> Option<&[DxfPolylinePolyfacePointEntry]> {
        let range = self.face(face_ordinal)?.point_range();
        self.points
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_polyface_face_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceFaceGeometryDirectory, DxfError> {
        DxfPolylinePolyfaceFaceGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_polyface_face_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceFaceGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polyface_face_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_polyface_face_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylinePolyfaceFaceGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_polyface_face_geometry_directory(cancellation)
    }
}

type ResolvedPoint = (DxfPolylinePolyfaceResolvedCornerEntry, [DxfDouble; 3]);

fn resolve_points(
    resolution: &DxfPolylinePolyfaceFaceResolutionDirectory,
    vertices: &DxfPolylineVertexSemanticDirectory,
    face: DxfPolylinePolyfaceResolvedFaceEntry,
) -> Result<
    (
        DxfPolylinePolyfaceFaceGeometryState,
        [Option<ResolvedPoint>; 4],
    ),
    DxfError,
> {
    let DxfPolylinePolyfaceFaceResolutionState::Available { corner_count } = face.state() else {
        return Ok((
            DxfPolylinePolyfaceFaceGeometryState::FaceResolutionFailure {
                state: face.state(),
            },
            [None; 4],
        ));
    };
    let corners = resolution
        .corners_for_face(face.face().ordinal())
        .ok_or_else(invalid_internal_data)?;
    if usize::from(corner_count) != corners.len() {
        return Err(invalid_internal_data());
    }
    let mut resolved = [None; 4];
    for corner in corners.iter().copied() {
        let semantics = vertices
            .semantics_for_entry(corner.coordinate().vertex())?
            .ok_or_else(invalid_internal_data)?;
        let Some(position) = semantics.position_value() else {
            return Ok((
                DxfPolylinePolyfaceFaceGeometryState::CoordinateUnavailable {
                    face_corner_ordinal: corner.face_corner_ordinal(),
                    component_states: [
                        semantics.position()[0].state(),
                        semantics.position()[1].state(),
                        semantics.position()[2].state(),
                    ],
                },
                resolved,
            ));
        };
        let at = usize::from(corner.face_corner_ordinal());
        let Some(slot) = resolved.get_mut(at) else {
            return Err(invalid_internal_data());
        };
        *slot = Some((corner, position));
    }
    Ok((
        DxfPolylinePolyfaceFaceGeometryState::Available {
            point_count: corner_count,
        },
        resolved,
    ))
}

fn append_points(
    points: &mut Vec<DxfPolylinePolyfacePointEntry>,
    resolved: [Option<ResolvedPoint>; 4],
) -> Result<(), DxfError> {
    for (face_point_ordinal, point) in resolved.into_iter().flatten().enumerate() {
        let ordinal = compact_len(points.len())?;
        points.try_reserve(1).map_err(|_| out_of_memory())?;
        points.push(DxfPolylinePolyfacePointEntry {
            ordinal,
            face_point_ordinal: u8::try_from(face_point_ordinal)
                .map_err(|_| invalid_internal_data())?,
            corner: point.0,
            position: point.1,
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

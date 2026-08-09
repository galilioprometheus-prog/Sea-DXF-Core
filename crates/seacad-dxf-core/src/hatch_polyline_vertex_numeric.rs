//! Finite numeric semantics for grouped HATCH polyline-boundary vertices.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfHatchPolylineVertexCardState, DxfHatchPolylineVertexDirectory,
    DxfHatchPolylineVertexEntry, DxfHatchPolylineVertexMember, DxfHatchPolylineVertexRole,
    DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId, raw_double::decode_raw_double,
};

const HATCH_NAMESPACE: &str = "entity.hatch";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineVertexNumericIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
}

pub type DxfHatchPolylineVertexNumericValue =
    DxfSemanticValue<DxfDouble, DxfHatchPolylineVertexNumericIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexNumericComponents {
    x: DxfHatchPolylineVertexNumericValue,
    y: DxfHatchPolylineVertexNumericValue,
    bulge: DxfHatchPolylineVertexNumericValue,
}

impl DxfHatchPolylineVertexNumericComponents {
    #[must_use]
    pub const fn x(&self) -> &DxfHatchPolylineVertexNumericValue {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &DxfHatchPolylineVertexNumericValue {
        &self.y
    }

    #[must_use]
    pub const fn bulge(&self) -> &DxfHatchPolylineVertexNumericValue {
        &self.bulge
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexNumericEntry {
    ordinal: u32,
    vertex: DxfHatchPolylineVertexEntry,
    components: DxfHatchPolylineVertexNumericComponents,
}

impl DxfHatchPolylineVertexNumericEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn vertex(self) -> DxfHatchPolylineVertexEntry {
        self.vertex
    }

    #[must_use]
    pub const fn components(&self) -> &DxfHatchPolylineVertexNumericComponents {
        &self.components
    }
}

/// One source-stable numeric entry per grouped M14.4k vertex.
#[derive(Debug)]
pub struct DxfHatchPolylineVertexNumericDirectory {
    source_id: DxfSourceId,
    vertices: DxfHatchPolylineVertexDirectory,
    entries: Box<[DxfHatchPolylineVertexNumericEntry]>,
}

impl DxfHatchPolylineVertexNumericDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let vertices = document.hatch_polyline_vertex_directory(cancellation)?;
        ensure_source(document.source_id(), vertices.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(vertices.vertices().len())
            .map_err(|_| out_of_memory())?;
        for vertex in vertices.vertices().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let members = vertices
                .members_for_vertex(vertex.ordinal())
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfHatchPolylineVertexNumericEntry {
                ordinal: compact_len(entries.len())?,
                vertex,
                components: DxfHatchPolylineVertexNumericComponents {
                    x: component(
                        document,
                        vertices.source_id(),
                        DxfHatchPolylineVertexRole::X,
                        vertex.x_state(),
                        Some(vertex.anchor()),
                        cancellation,
                    )?,
                    y: member_component(
                        document,
                        vertices.source_id(),
                        vertex.y_state(),
                        members,
                        DxfHatchPolylineVertexRole::Y,
                        cancellation,
                    )?,
                    bulge: member_component(
                        document,
                        vertices.source_id(),
                        vertex.bulge_state(),
                        members,
                        DxfHatchPolylineVertexRole::Bulge,
                        cancellation,
                    )?,
                },
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            vertices,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn vertex_directory(&self) -> &DxfHatchPolylineVertexDirectory {
        &self.vertices
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineVertexNumericEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineVertexNumericEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchPolylineVertexNumericEntry]> {
        self.vertices.vertices_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.vertex().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.vertex().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_vertex_numeric_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexNumericDirectory, DxfError> {
        DxfHatchPolylineVertexNumericDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_vertex_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_vertex_numeric_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_vertex_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_vertex_numeric_directory(cancellation)
    }
}

fn member_component(
    document: DxfRawDocumentView<'_>,
    source_id: DxfSourceId,
    state: DxfHatchPolylineVertexCardState,
    members: &[DxfHatchPolylineVertexMember],
    role: DxfHatchPolylineVertexRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchPolylineVertexNumericValue, DxfError> {
    let group = if state == DxfHatchPolylineVertexCardState::Unique {
        Some(
            members
                .iter()
                .copied()
                .find(|member| member.role() == role)
                .ok_or_else(invalid_internal_data)?
                .group(),
        )
    } else {
        None
    };
    component(document, source_id, role, state, group, cancellation)
}

fn component(
    document: DxfRawDocumentView<'_>,
    source_id: DxfSourceId,
    role: DxfHatchPolylineVertexRole,
    state: DxfHatchPolylineVertexCardState,
    group: Option<DxfRawGroup>,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchPolylineVertexNumericValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(source_id, HATCH_NAMESPACE, field_id(role));
    match state {
        DxfHatchPolylineVertexCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchPolylineVertexCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchPolylineVertexNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchPolylineVertexCardState::Unique => {
            let group = group.ok_or_else(invalid_internal_data)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_double(document, group, cancellation)? {
                Ok(value) if value.is_finite() => DxfSemanticValue::explicit(value, field, raw),
                Ok(value) => DxfSemanticValue::invalid(
                    DxfHatchPolylineVertexNumericIssue::NonFiniteDouble(value),
                    field,
                    Some(raw),
                ),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchPolylineVertexNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn raw_provenance(group: DxfRawGroup) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
        .ok_or_else(invalid_internal_data)
}

const fn field_id(role: DxfHatchPolylineVertexRole) -> &'static str {
    match role {
        DxfHatchPolylineVertexRole::X => "polyline_vertex_x",
        DxfHatchPolylineVertexRole::Y => "polyline_vertex_y",
        DxfHatchPolylineVertexRole::Bulge => "polyline_vertex_bulge",
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
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

//! Lazy typed LWPOLYLINE vertex semantics over M9.1c cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfLightweightPolylineInteger,
    DxfLightweightPolylineIntegerIssue, DxfLightweightPolylineNumericIssue,
    DxfLightweightPolylineVertexCard, DxfLightweightPolylineVertexCardState,
    DxfLightweightPolylineVertexDirectory, DxfLightweightPolylineVertexEntry,
    DxfLightweightPolylineVertexMember, DxfLightweightPolylineVertexRole, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "lightweight_polyline.vertex";
const DEFAULT_ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());

/// Why one reviewed LWPOLYLINE vertex field has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineVertexSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored state for one LWPOLYLINE vertex double.
pub type DxfLightweightPolylineVertexSemanticDouble =
    DxfSemanticValue<DxfDouble, DxfLightweightPolylineVertexSemanticIssue>;

/// Source-anchored state for one optional LWPOLYLINE vertex identifier.
pub type DxfLightweightPolylineVertexSemanticIdentifier =
    DxfSemanticValue<i32, DxfLightweightPolylineVertexSemanticIssue>;

/// Lazy reviewed semantics for one conservatively grouped vertex.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineVertexSemantics {
    vertex: DxfLightweightPolylineVertexEntry,
    ocs_position: [DxfLightweightPolylineVertexSemanticDouble; 2],
    local_start_width: DxfLightweightPolylineVertexSemanticDouble,
    local_end_width: DxfLightweightPolylineVertexSemanticDouble,
    bulge: DxfLightweightPolylineVertexSemanticDouble,
    identifier: DxfLightweightPolylineVertexSemanticIdentifier,
}

impl DxfLightweightPolylineVertexSemantics {
    #[must_use]
    pub const fn vertex(self) -> DxfLightweightPolylineVertexEntry {
        self.vertex
    }

    #[must_use]
    pub const fn ocs_position(&self) -> &[DxfLightweightPolylineVertexSemanticDouble; 2] {
        &self.ocs_position
    }

    #[must_use]
    pub const fn local_start_width(&self) -> &DxfLightweightPolylineVertexSemanticDouble {
        &self.local_start_width
    }

    #[must_use]
    pub const fn local_end_width(&self) -> &DxfLightweightPolylineVertexSemanticDouble {
        &self.local_end_width
    }

    #[must_use]
    pub const fn bulge(&self) -> &DxfLightweightPolylineVertexSemanticDouble {
        &self.bulge
    }

    #[must_use]
    pub const fn identifier(&self) -> &DxfLightweightPolylineVertexSemanticIdentifier {
        &self.identifier
    }

    #[must_use]
    pub fn ocs_position_value(&self) -> Option<[DxfDouble; 2]> {
        Some([
            self.ocs_position[0].value().copied()?,
            self.ocs_position[1].value().copied()?,
        ])
    }

    #[must_use]
    pub fn local_width_values(&self) -> Option<[DxfDouble; 2]> {
        Some([
            self.local_start_width.value().copied()?,
            self.local_end_width.value().copied()?,
        ])
    }

    #[must_use]
    pub fn bulge_value(&self) -> Option<DxfDouble> {
        self.bulge.value().copied()
    }

    #[must_use]
    pub fn identifier_value(&self) -> Option<i32> {
        self.identifier.value().copied()
    }
}

/// Immutable lazy vertex semantics with all grouping and raw evidence retained.
#[derive(Debug)]
pub struct DxfLightweightPolylineVertexSemanticDirectory {
    source_id: DxfSourceId,
    vertices: DxfLightweightPolylineVertexDirectory,
}

impl DxfLightweightPolylineVertexSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let vertices = document.lightweight_polyline_vertex_directory(cancellation)?;
        if vertices.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: vertices.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            vertices,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn vertex_directory(&self) -> &DxfLightweightPolylineVertexDirectory {
        &self.vertices
    }

    #[must_use]
    pub fn vertices(&self) -> &[DxfLightweightPolylineVertexEntry] {
        self.vertices.vertices()
    }

    pub fn semantics_for_vertex(
        &self,
        vertex_ordinal: u64,
    ) -> Result<Option<DxfLightweightPolylineVertexSemantics>, DxfError> {
        let Ok(index) = usize::try_from(vertex_ordinal) else {
            return Ok(None);
        };
        let Some(vertex) = self.vertices().get(index).copied() else {
            return Ok(None);
        };
        vertex_semantics(&self.vertices, vertex_ordinal, vertex).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        vertex: DxfLightweightPolylineVertexEntry,
    ) -> Result<Option<DxfLightweightPolylineVertexSemantics>, DxfError> {
        let vertex_ordinal = vertex.cards()[0].vertex_ordinal();
        let Ok(index) = usize::try_from(vertex_ordinal) else {
            return Ok(None);
        };
        if self.vertices().get(index) != Some(&vertex) {
            return Ok(None);
        }
        vertex_semantics(&self.vertices, vertex_ordinal, vertex).map(Some)
    }

    pub fn semantics_for_raw_record_vertex(
        &self,
        raw_record_ordinal: u64,
        record_vertex_ordinal: u64,
    ) -> Result<Option<DxfLightweightPolylineVertexSemantics>, DxfError> {
        let Some(vertices) = self.vertices.vertices_for_raw_record(raw_record_ordinal) else {
            return Ok(None);
        };
        let Some(vertex) = usize::try_from(record_vertex_ordinal)
            .ok()
            .and_then(|index| vertices.get(index))
            .copied()
        else {
            return Ok(None);
        };
        self.semantics_for_entry(vertex)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn lightweight_polyline_vertex_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineVertexSemanticDirectory, DxfError> {
        DxfLightweightPolylineVertexSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_vertex_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineVertexSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_vertex_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_vertex_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineVertexSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_vertex_semantic_directory(cancellation)
    }
}

fn vertex_semantics(
    directory: &DxfLightweightPolylineVertexDirectory,
    vertex_ordinal: u64,
    vertex: DxfLightweightPolylineVertexEntry,
) -> Result<DxfLightweightPolylineVertexSemantics, DxfError> {
    Ok(DxfLightweightPolylineVertexSemantics {
        vertex,
        ocs_position: [
            double_semantic(
                directory,
                vertex_ordinal,
                DxfLightweightPolylineVertexRole::OcsX,
                "ocs_x",
                Absence::Required,
            )?,
            double_semantic(
                directory,
                vertex_ordinal,
                DxfLightweightPolylineVertexRole::OcsY,
                "ocs_y",
                Absence::Required,
            )?,
        ],
        local_start_width: double_semantic(
            directory,
            vertex_ordinal,
            DxfLightweightPolylineVertexRole::StartWidth,
            "local_start_width",
            Absence::DefaultZero,
        )?,
        local_end_width: double_semantic(
            directory,
            vertex_ordinal,
            DxfLightweightPolylineVertexRole::EndWidth,
            "local_end_width",
            Absence::DefaultZero,
        )?,
        bulge: double_semantic(
            directory,
            vertex_ordinal,
            DxfLightweightPolylineVertexRole::Bulge,
            "bulge",
            Absence::DefaultZero,
        )?,
        identifier: identifier_semantic(directory, vertex_ordinal)?,
    })
}

fn double_semantic(
    directory: &DxfLightweightPolylineVertexDirectory,
    vertex_ordinal: u64,
    role: DxfLightweightPolylineVertexRole,
    field_id: &'static str,
    absence: Absence,
) -> Result<DxfLightweightPolylineVertexSemanticDouble, DxfError> {
    let field = DxfSemanticFieldProvenance::new(directory.source_id(), NAMESPACE, field_id);
    let card = directory
        .card_for_role(vertex_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfLightweightPolylineVertexCardState::Absent => Ok(match absence {
            Absence::Required => DxfSemanticValue::invalid(
                DxfLightweightPolylineVertexSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
            Absence::DefaultZero => DxfSemanticValue::defaulted(DEFAULT_ZERO, field),
        }),
        DxfLightweightPolylineVertexCardState::Unique => {
            let member = unique_member(directory, card)?;
            let value = directory
                .floating_value_for_member(member)
                .ok_or_else(invalid_internal_data)?;
            let raw = value_provenance(
                value.group().occurrence(),
                value.group().value_payload_span(),
            )?;
            Ok(match value.value() {
                Ok(number) => DxfSemanticValue::explicit(number, field, raw),
                Err(DxfLightweightPolylineNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfLightweightPolylineVertexSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfLightweightPolylineVertexCardState::Multiple { occurrence_count } => {
            let member = first_member(directory, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfLightweightPolylineVertexSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(member_provenance(directory, member)?),
            ))
        }
    }
}

fn identifier_semantic(
    directory: &DxfLightweightPolylineVertexDirectory,
    vertex_ordinal: u64,
) -> Result<DxfLightweightPolylineVertexSemanticIdentifier, DxfError> {
    let field = DxfSemanticFieldProvenance::new(directory.source_id(), NAMESPACE, "identifier");
    let card = directory
        .card_for_role(vertex_ordinal, DxfLightweightPolylineVertexRole::Identifier)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfLightweightPolylineVertexCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfLightweightPolylineVertexCardState::Unique => {
            let member = unique_member(directory, card)?;
            let value = directory
                .integer_value_for_member(member)
                .ok_or_else(invalid_internal_data)?;
            let raw = value_provenance(
                value.group().occurrence(),
                value.group().value_payload_span(),
            )?;
            Ok(match value.value() {
                Ok(DxfLightweightPolylineInteger::I32(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfLightweightPolylineInteger::I16(_)) => return Err(invalid_internal_data()),
                Err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfLightweightPolylineVertexSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfLightweightPolylineVertexCardState::Multiple { occurrence_count } => {
            let member = first_member(directory, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfLightweightPolylineVertexSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(member_provenance(directory, member)?),
            ))
        }
    }
}

fn unique_member(
    directory: &DxfLightweightPolylineVertexDirectory,
    card: DxfLightweightPolylineVertexCard,
) -> Result<DxfLightweightPolylineVertexMember, DxfError> {
    let members = directory
        .members_for_card(card)
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
        return Err(invalid_internal_data());
    };
    Ok(*member)
}

fn first_member(
    directory: &DxfLightweightPolylineVertexDirectory,
    card: DxfLightweightPolylineVertexCard,
) -> Result<DxfLightweightPolylineVertexMember, DxfError> {
    directory
        .members_for_card(card)
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)
}

fn member_provenance(
    directory: &DxfLightweightPolylineVertexDirectory,
    member: DxfLightweightPolylineVertexMember,
) -> Result<DxfRawValueProvenance, DxfError> {
    if let Some(value) = directory.floating_value_for_member(member) {
        return value_provenance(
            value.group().occurrence(),
            value.group().value_payload_span(),
        );
    }
    let value = directory
        .integer_value_for_member(member)
        .ok_or_else(invalid_internal_data)?;
    value_provenance(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
}

fn value_provenance(
    occurrence: u64,
    span: crate::ByteSpan,
) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(occurrence, span).ok_or_else(invalid_internal_data)
}

#[derive(Clone, Copy)]
enum Absence {
    Required,
    DefaultZero,
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

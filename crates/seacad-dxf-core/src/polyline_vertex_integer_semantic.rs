//! Lazy typed integer semantics for classic POLYLINE VERTEX records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfPolylineVertexCardDirectory, DxfPolylineVertexCardMember,
    DxfPolylineVertexNumber, DxfPolylineVertexNumericIssue, DxfPolylineVertexValueCard,
    DxfPolylineVertexValueCardState, DxfPolylineVertexValueEntry, DxfPolylineVertexValueRole,
    DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfSourceId,
};

const NAMESPACE: &str = "polyline.vertex.integer";

/// Why one classic VERTEX integer has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexIntegerSemanticIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored state for one classic VERTEX signed-i16 field.
pub type DxfPolylineVertexSemanticI16 =
    DxfSemanticValue<i16, DxfPolylineVertexIntegerSemanticIssue>;

/// Source-anchored state for one classic VERTEX signed-i32 field.
pub type DxfPolylineVertexSemanticI32 =
    DxfSemanticValue<i32, DxfPolylineVertexIntegerSemanticIssue>;

/// Lazy integer semantics for one retained classic VERTEX.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexIntegerSemantics {
    vertex: DxfPolylineVertexValueEntry,
    flags: DxfPolylineVertexSemanticI16,
    polyface_vertex_indices: [DxfPolylineVertexSemanticI16; 4],
    identifier: DxfPolylineVertexSemanticI32,
}

impl DxfPolylineVertexIntegerSemantics {
    #[must_use]
    pub const fn vertex(self) -> DxfPolylineVertexValueEntry {
        self.vertex
    }

    #[must_use]
    pub const fn flags(&self) -> &DxfPolylineVertexSemanticI16 {
        &self.flags
    }

    #[must_use]
    pub const fn polyface_vertex_indices(&self) -> &[DxfPolylineVertexSemanticI16; 4] {
        &self.polyface_vertex_indices
    }

    #[must_use]
    pub const fn identifier(&self) -> &DxfPolylineVertexSemanticI32 {
        &self.identifier
    }

    #[must_use]
    pub fn flags_value(&self) -> Option<i16> {
        self.flags.value().copied()
    }

    #[must_use]
    pub fn is_extra_curve_fit_vertex(&self) -> Option<bool> {
        self.flag_bit(1)
    }

    #[must_use]
    pub fn has_curve_fit_tangent(&self) -> Option<bool> {
        self.flag_bit(2)
    }

    #[must_use]
    pub fn is_spline_fit_vertex(&self) -> Option<bool> {
        self.flag_bit(8)
    }

    #[must_use]
    pub fn is_spline_frame_control_point(&self) -> Option<bool> {
        self.flag_bit(16)
    }

    #[must_use]
    pub fn is_3d_polyline_vertex(&self) -> Option<bool> {
        self.flag_bit(32)
    }

    #[must_use]
    pub fn is_3d_polygon_mesh_vertex(&self) -> Option<bool> {
        self.flag_bit(64)
    }

    #[must_use]
    pub fn is_polyface_mesh_vertex(&self) -> Option<bool> {
        self.flag_bit(128)
    }

    #[must_use]
    pub fn identifier_value(&self) -> Option<i32> {
        self.identifier.value().copied()
    }

    fn flag_bit(&self, bit: i16) -> Option<bool> {
        Some(self.flags_value()? & bit != 0)
    }
}

/// Immutable lazy classic VERTEX integer semantics retaining M9.2d evidence.
#[derive(Debug)]
pub struct DxfPolylineVertexIntegerSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfPolylineVertexCardDirectory,
}

impl DxfPolylineVertexIntegerSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.polyline_vertex_card_directory(cancellation)?;
        if cards.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: cards.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfPolylineVertexCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn vertices(&self) -> &[DxfPolylineVertexValueEntry] {
        self.cards.evidence_directory().vertices()
    }

    pub fn semantics_for_vertex_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfPolylineVertexIntegerSemantics>, DxfError> {
        let Some(vertex) = self
            .cards
            .evidence_directory()
            .vertex_for_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        vertex_semantics(&self.cards, vertex).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        vertex: DxfPolylineVertexValueEntry,
    ) -> Result<Option<DxfPolylineVertexIntegerSemantics>, DxfError> {
        if self
            .cards
            .evidence_directory()
            .vertex_for_raw_ordinal(vertex.vertex_record().ordinal())
            != Some(vertex)
        {
            return Ok(None);
        }
        vertex_semantics(&self.cards, vertex).map(Some)
    }

    pub fn semantics_for_polyline_sequence_vertex(
        &self,
        polyline_raw_ordinal: u64,
        sequence_vertex_ordinal: u64,
    ) -> Result<Option<DxfPolylineVertexIntegerSemantics>, DxfError> {
        let Some(vertices) = self
            .cards
            .evidence_directory()
            .vertices_for_polyline_raw_ordinal(polyline_raw_ordinal)
        else {
            return Ok(None);
        };
        let Some(vertex) = usize::try_from(sequence_vertex_ordinal)
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
    pub fn polyline_vertex_integer_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexIntegerSemanticDirectory, DxfError> {
        DxfPolylineVertexIntegerSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_vertex_integer_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexIntegerSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_integer_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_vertex_integer_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexIntegerSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_integer_semantic_directory(cancellation)
    }
}

fn vertex_semantics(
    cards: &DxfPolylineVertexCardDirectory,
    vertex: DxfPolylineVertexValueEntry,
) -> Result<DxfPolylineVertexIntegerSemantics, DxfError> {
    let raw_ordinal = vertex.vertex_record().ordinal();
    Ok(DxfPolylineVertexIntegerSemantics {
        vertex,
        flags: i16_semantic(
            cards,
            raw_ordinal,
            DxfPolylineVertexValueRole::Flags,
            "flags",
        )?,
        polyface_vertex_indices: [
            i16_semantic(
                cards,
                raw_ordinal,
                DxfPolylineVertexValueRole::PolyfaceVertexIndex1,
                "polyface_vertex_index_1",
            )?,
            i16_semantic(
                cards,
                raw_ordinal,
                DxfPolylineVertexValueRole::PolyfaceVertexIndex2,
                "polyface_vertex_index_2",
            )?,
            i16_semantic(
                cards,
                raw_ordinal,
                DxfPolylineVertexValueRole::PolyfaceVertexIndex3,
                "polyface_vertex_index_3",
            )?,
            i16_semantic(
                cards,
                raw_ordinal,
                DxfPolylineVertexValueRole::PolyfaceVertexIndex4,
                "polyface_vertex_index_4",
            )?,
        ],
        identifier: i32_semantic(
            cards,
            raw_ordinal,
            DxfPolylineVertexValueRole::Identifier,
            "identifier",
        )?,
    })
}

fn i16_semantic(
    cards: &DxfPolylineVertexCardDirectory,
    raw_record_ordinal: u64,
    role: DxfPolylineVertexValueRole,
    field_id: &'static str,
) -> Result<DxfPolylineVertexSemanticI16, DxfError> {
    let value = integer_semantic(cards, raw_record_ordinal, role, field_id)?;
    Ok(match value {
        DxfSemanticValue::Explicit {
            value: DxfPolylineVertexNumber::Int16(value),
            field,
            raw,
        } => DxfSemanticValue::explicit(value, field, raw),
        DxfSemanticValue::Explicit { .. } => return Err(invalid_internal_data()),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(issue, field, raw)
        }
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

fn i32_semantic(
    cards: &DxfPolylineVertexCardDirectory,
    raw_record_ordinal: u64,
    role: DxfPolylineVertexValueRole,
    field_id: &'static str,
) -> Result<DxfPolylineVertexSemanticI32, DxfError> {
    let value = integer_semantic(cards, raw_record_ordinal, role, field_id)?;
    Ok(match value {
        DxfSemanticValue::Explicit {
            value: DxfPolylineVertexNumber::Int32(value),
            field,
            raw,
        } => DxfSemanticValue::explicit(value, field, raw),
        DxfSemanticValue::Explicit { .. } => return Err(invalid_internal_data()),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(issue, field, raw)
        }
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

fn integer_semantic(
    cards: &DxfPolylineVertexCardDirectory,
    raw_record_ordinal: u64,
    role: DxfPolylineVertexValueRole,
    field_id: &'static str,
) -> Result<
    DxfSemanticValue<DxfPolylineVertexNumber, DxfPolylineVertexIntegerSemanticIssue>,
    DxfError,
> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = cards
        .card_for_role(raw_record_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfPolylineVertexValueCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfPolylineVertexValueCardState::Unique => {
            let value = value_for_member(cards, unique_member(cards, card)?)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(
                    number
                    @ (DxfPolylineVertexNumber::Int16(_) | DxfPolylineVertexNumber::Int32(_)),
                ) => DxfSemanticValue::explicit(number, field, raw),
                Ok(DxfPolylineVertexNumber::Double(_)) => return Err(invalid_internal_data()),
                Err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfPolylineVertexIntegerSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfPolylineVertexValueCardState::Multiple { occurrence_count } => {
            let value = value_for_member(cards, first_member(cards, card)?)?;
            Ok(DxfSemanticValue::invalid(
                DxfPolylineVertexIntegerSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(value)?),
            ))
        }
    }
}

fn unique_member(
    cards: &DxfPolylineVertexCardDirectory,
    card: DxfPolylineVertexValueCard,
) -> Result<DxfPolylineVertexCardMember, DxfError> {
    let members = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
        return Err(invalid_internal_data());
    };
    Ok(*member)
}

fn first_member(
    cards: &DxfPolylineVertexCardDirectory,
    card: DxfPolylineVertexValueCard,
) -> Result<DxfPolylineVertexCardMember, DxfError> {
    cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)
}

fn value_for_member(
    cards: &DxfPolylineVertexCardDirectory,
    member: DxfPolylineVertexCardMember,
) -> Result<crate::DxfPolylineVertexValue, DxfError> {
    cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(
    value: crate::DxfPolylineVertexValue,
) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
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

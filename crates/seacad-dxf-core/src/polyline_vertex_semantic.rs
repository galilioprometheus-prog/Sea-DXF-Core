//! Lazy typed double semantics for classic POLYLINE VERTEX records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfPolylineVertexCardDirectory, DxfPolylineVertexNumber,
    DxfPolylineVertexNumericIssue, DxfPolylineVertexValueCard, DxfPolylineVertexValueCardState,
    DxfPolylineVertexValueEntry, DxfPolylineVertexValueRole, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "polyline.vertex";
const DEFAULT_ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());

/// Why one reviewed classic VERTEX double has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored state for one reviewed classic VERTEX double.
pub type DxfPolylineVertexSemanticDouble =
    DxfSemanticValue<DxfDouble, DxfPolylineVertexSemanticIssue>;

/// Lazy reviewed double semantics for one retained classic VERTEX.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexSemantics {
    vertex: DxfPolylineVertexValueEntry,
    position: [DxfPolylineVertexSemanticDouble; 3],
    start_width: DxfPolylineVertexSemanticDouble,
    end_width: DxfPolylineVertexSemanticDouble,
    bulge: DxfPolylineVertexSemanticDouble,
    curve_fit_tangent_direction: DxfPolylineVertexSemanticDouble,
}

impl DxfPolylineVertexSemantics {
    #[must_use]
    pub const fn vertex(self) -> DxfPolylineVertexValueEntry {
        self.vertex
    }

    #[must_use]
    pub const fn position(&self) -> &[DxfPolylineVertexSemanticDouble; 3] {
        &self.position
    }

    #[must_use]
    pub const fn start_width(&self) -> &DxfPolylineVertexSemanticDouble {
        &self.start_width
    }

    #[must_use]
    pub const fn end_width(&self) -> &DxfPolylineVertexSemanticDouble {
        &self.end_width
    }

    #[must_use]
    pub const fn bulge(&self) -> &DxfPolylineVertexSemanticDouble {
        &self.bulge
    }

    #[must_use]
    pub const fn curve_fit_tangent_direction(&self) -> &DxfPolylineVertexSemanticDouble {
        &self.curve_fit_tangent_direction
    }

    #[must_use]
    pub fn position_value(&self) -> Option<[DxfDouble; 3]> {
        Some([
            self.position[0].value().copied()?,
            self.position[1].value().copied()?,
            self.position[2].value().copied()?,
        ])
    }

    #[must_use]
    pub fn width_values(&self) -> Option<[DxfDouble; 2]> {
        Some([
            self.start_width.value().copied()?,
            self.end_width.value().copied()?,
        ])
    }

    #[must_use]
    pub fn bulge_value(&self) -> Option<DxfDouble> {
        self.bulge.value().copied()
    }

    #[must_use]
    pub fn curve_fit_tangent_direction_value(&self) -> Option<DxfDouble> {
        self.curve_fit_tangent_direction.value().copied()
    }
}

/// Immutable lazy classic VERTEX double semantics with all evidence retained.
#[derive(Debug)]
pub struct DxfPolylineVertexSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfPolylineVertexCardDirectory,
}

impl DxfPolylineVertexSemanticDirectory {
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
    ) -> Result<Option<DxfPolylineVertexSemantics>, DxfError> {
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
    ) -> Result<Option<DxfPolylineVertexSemantics>, DxfError> {
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
    ) -> Result<Option<DxfPolylineVertexSemantics>, DxfError> {
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
    pub fn polyline_vertex_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexSemanticDirectory, DxfError> {
        DxfPolylineVertexSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_vertex_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_vertex_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_semantic_directory(cancellation)
    }
}

fn vertex_semantics(
    directory: &DxfPolylineVertexCardDirectory,
    vertex: DxfPolylineVertexValueEntry,
) -> Result<DxfPolylineVertexSemantics, DxfError> {
    let raw_ordinal = vertex.vertex_record().ordinal();
    Ok(DxfPolylineVertexSemantics {
        vertex,
        position: [
            double_semantic(
                directory,
                raw_ordinal,
                DxfPolylineVertexValueRole::LocationX,
                "location_x",
                Absence::Required,
            )?,
            double_semantic(
                directory,
                raw_ordinal,
                DxfPolylineVertexValueRole::LocationY,
                "location_y",
                Absence::Required,
            )?,
            double_semantic(
                directory,
                raw_ordinal,
                DxfPolylineVertexValueRole::LocationZ,
                "location_z",
                Absence::Required,
            )?,
        ],
        start_width: double_semantic(
            directory,
            raw_ordinal,
            DxfPolylineVertexValueRole::StartWidth,
            "start_width",
            Absence::DefaultZero,
        )?,
        end_width: double_semantic(
            directory,
            raw_ordinal,
            DxfPolylineVertexValueRole::EndWidth,
            "end_width",
            Absence::DefaultZero,
        )?,
        bulge: double_semantic(
            directory,
            raw_ordinal,
            DxfPolylineVertexValueRole::Bulge,
            "bulge",
            Absence::DefaultZero,
        )?,
        curve_fit_tangent_direction: double_semantic(
            directory,
            raw_ordinal,
            DxfPolylineVertexValueRole::CurveFitTangentDirection,
            "curve_fit_tangent_direction",
            Absence::Optional,
        )?,
    })
}

fn double_semantic(
    directory: &DxfPolylineVertexCardDirectory,
    raw_record_ordinal: u64,
    role: DxfPolylineVertexValueRole,
    field_id: &'static str,
    absence: Absence,
) -> Result<DxfPolylineVertexSemanticDouble, DxfError> {
    let field = DxfSemanticFieldProvenance::new(directory.source_id(), NAMESPACE, field_id);
    let card = directory
        .card_for_role(raw_record_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfPolylineVertexValueCardState::Absent => Ok(match absence {
            Absence::Required => DxfSemanticValue::invalid(
                DxfPolylineVertexSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
            Absence::DefaultZero => DxfSemanticValue::defaulted(DEFAULT_ZERO, field),
            Absence::Optional => DxfSemanticValue::absent(field),
        }),
        DxfPolylineVertexValueCardState::Unique => {
            let member = unique_member(directory, card)?;
            let value = directory
                .value_for_member(member)
                .ok_or_else(invalid_internal_data)?;
            let raw = value_provenance(
                value.group().occurrence(),
                value.group().value_payload_span(),
            )?;
            Ok(match value.value() {
                Ok(DxfPolylineVertexNumber::Double(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfPolylineVertexNumber::Int16(_) | DxfPolylineVertexNumber::Int32(_)) => {
                    return Err(invalid_internal_data());
                }
                Err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfPolylineVertexSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfPolylineVertexValueCardState::Multiple { occurrence_count } => {
            let member = first_member(directory, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfPolylineVertexSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(member_provenance(directory, member)?),
            ))
        }
    }
}

fn unique_member(
    directory: &DxfPolylineVertexCardDirectory,
    card: DxfPolylineVertexValueCard,
) -> Result<crate::DxfPolylineVertexCardMember, DxfError> {
    let members = directory
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
        return Err(invalid_internal_data());
    };
    Ok(*member)
}

fn first_member(
    directory: &DxfPolylineVertexCardDirectory,
    card: DxfPolylineVertexValueCard,
) -> Result<crate::DxfPolylineVertexCardMember, DxfError> {
    directory
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)
}

fn member_provenance(
    directory: &DxfPolylineVertexCardDirectory,
    member: crate::DxfPolylineVertexCardMember,
) -> Result<DxfRawValueProvenance, DxfError> {
    let value = directory
        .value_for_member(member)
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
    Optional,
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

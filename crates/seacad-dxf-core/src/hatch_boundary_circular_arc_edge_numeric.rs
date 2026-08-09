//! Source-anchored numeric semantics for HATCH boundary CircularArc edge fields.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfHatchBoundaryCircularArcEdgeCard,
    DxfHatchBoundaryCircularArcEdgeCardDirectory, DxfHatchBoundaryCircularArcEdgeCardState,
    DxfHatchBoundaryCircularArcEdgeRole, DxfHatchBoundaryEdgeType, DxfRawDocumentView, DxfRawGroup,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    raw_double::decode_raw_double,
    raw_integer::decode_raw_i16,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

const HATCH_NAMESPACE: &str = "entity.hatch";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryCircularArcEdgeNumericIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
}

pub type DxfHatchBoundaryCircularArcEdgeNumericDoubleValue =
    DxfSemanticValue<DxfDouble, DxfHatchBoundaryCircularArcEdgeNumericIssue>;

pub type DxfHatchBoundaryCircularArcEdgeNumericIntegerValue =
    DxfSemanticValue<i16, DxfHatchBoundaryCircularArcEdgeNumericIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeNumericComponents {
    center_x: DxfHatchBoundaryCircularArcEdgeNumericDoubleValue,
    center_y: DxfHatchBoundaryCircularArcEdgeNumericDoubleValue,
    radius: DxfHatchBoundaryCircularArcEdgeNumericDoubleValue,
    start_angle: DxfHatchBoundaryCircularArcEdgeNumericDoubleValue,
    end_angle: DxfHatchBoundaryCircularArcEdgeNumericDoubleValue,
    counterclockwise: DxfHatchBoundaryCircularArcEdgeNumericIntegerValue,
}

impl DxfHatchBoundaryCircularArcEdgeNumericComponents {
    #[must_use]
    pub const fn center_x(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericDoubleValue {
        &self.center_x
    }

    #[must_use]
    pub const fn center_y(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericDoubleValue {
        &self.center_y
    }

    #[must_use]
    pub const fn radius(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericDoubleValue {
        &self.radius
    }

    #[must_use]
    pub const fn start_angle(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericDoubleValue {
        &self.start_angle
    }

    #[must_use]
    pub const fn end_angle(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericDoubleValue {
        &self.end_angle
    }

    #[must_use]
    pub const fn counterclockwise(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericIntegerValue {
        &self.counterclockwise
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeNumericEntry {
    ordinal: u32,
    edge_ordinal: u32,
    path_ordinal: u32,
    components: DxfHatchBoundaryCircularArcEdgeNumericComponents,
}

impl DxfHatchBoundaryCircularArcEdgeNumericEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn edge_ordinal(self) -> u64 {
        self.edge_ordinal as u64
    }

    #[must_use]
    pub const fn path_ordinal(self) -> u64 {
        self.path_ordinal as u64
    }

    #[must_use]
    pub const fn components(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericComponents {
        &self.components
    }
}

/// One source-stable numeric entry per grouped edge typed as CircularArc.
#[derive(Debug)]
pub struct DxfHatchBoundaryCircularArcEdgeNumericDirectory {
    source_id: DxfSourceId,
    cards: DxfHatchBoundaryCircularArcEdgeCardDirectory,
    entries: Box<[DxfHatchBoundaryCircularArcEdgeNumericEntry]>,
}

impl DxfHatchBoundaryCircularArcEdgeNumericDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.hatch_boundary_circular_arc_edge_card_directory(cancellation)?;
        ensure_source(document.source_id(), cards.source_id())?;
        if cards.cards().len() % 6 != 0 {
            return Err(invalid_internal_data());
        }
        let circular_arc_count = cards.cards().len() / 6;
        let mut entries = Vec::new();
        entries
            .try_reserve(circular_arc_count)
            .map_err(|_| out_of_memory())?;
        for edge in cards.edge_type_directory().entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if edge.edge_type() != Ok(DxfHatchBoundaryEdgeType::CircularArc) {
                continue;
            }
            entries.push(DxfHatchBoundaryCircularArcEdgeNumericEntry {
                ordinal: compact_len(entries.len())?,
                edge_ordinal: compact_u64(edge.ordinal())?,
                path_ordinal: compact_u64(edge.edge().path_ordinal())?,
                components: DxfHatchBoundaryCircularArcEdgeNumericComponents {
                    center_x: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryCircularArcEdgeRole::CenterX,
                        cancellation,
                    )?,
                    center_y: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryCircularArcEdgeRole::CenterY,
                        cancellation,
                    )?,
                    radius: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryCircularArcEdgeRole::Radius,
                        cancellation,
                    )?,
                    start_angle: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryCircularArcEdgeRole::StartAngle,
                        cancellation,
                    )?,
                    end_angle: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryCircularArcEdgeRole::EndAngle,
                        cancellation,
                    )?,
                    counterclockwise: integer_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryCircularArcEdgeRole::Counterclockwise,
                        cancellation,
                    )?,
                },
            });
        }
        if entries.len() != circular_arc_count {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfHatchBoundaryCircularArcEdgeCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryCircularArcEdgeNumericEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryCircularArcEdgeNumericEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryCircularArcEdgeNumericEntry> {
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| entry.edge_ordinal())
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchBoundaryCircularArcEdgeNumericEntry]> {
        self.cards
            .edge_type_directory()
            .entries_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_circular_arc_edge_numeric_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeNumericDirectory, DxfError> {
        DxfHatchBoundaryCircularArcEdgeNumericDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_numeric_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_numeric_directory(cancellation)
    }
}

fn double_component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundaryCircularArcEdgeCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundaryCircularArcEdgeRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundaryCircularArcEdgeNumericDoubleValue, DxfError> {
    let (card, field) = card_and_field(cards, edge_ordinal, role)?;
    match card.state() {
        DxfHatchBoundaryCircularArcEdgeCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundaryCircularArcEdgeCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundaryCircularArcEdgeNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundaryCircularArcEdgeCardState::Unique => {
            let group = unique_group(cards, card)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_double(document, group, cancellation)? {
                Ok(value) if value.is_finite() => DxfSemanticValue::explicit(value, field, raw),
                Ok(value) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryCircularArcEdgeNumericIssue::NonFiniteDouble(value),
                    field,
                    Some(raw),
                ),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryCircularArcEdgeNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn integer_component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundaryCircularArcEdgeCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundaryCircularArcEdgeRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundaryCircularArcEdgeNumericIntegerValue, DxfError> {
    let (card, field) = card_and_field(cards, edge_ordinal, role)?;
    match card.state() {
        DxfHatchBoundaryCircularArcEdgeCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundaryCircularArcEdgeCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundaryCircularArcEdgeNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundaryCircularArcEdgeCardState::Unique => {
            let group = unique_group(cards, card)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_i16(document, group, cancellation)? {
                Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryCircularArcEdgeNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn card_and_field(
    cards: &DxfHatchBoundaryCircularArcEdgeCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundaryCircularArcEdgeRole,
) -> Result<
    (
        DxfHatchBoundaryCircularArcEdgeCard,
        DxfSemanticFieldProvenance,
    ),
    DxfError,
> {
    let card = cards
        .card_for_role(edge_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), HATCH_NAMESPACE, field_id(role));
    Ok((card, field))
}

fn unique_group(
    cards: &DxfHatchBoundaryCircularArcEdgeCardDirectory,
    card: DxfHatchBoundaryCircularArcEdgeCard,
) -> Result<DxfRawGroup, DxfError> {
    let [member] = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    Ok(member.field().group())
}

fn raw_provenance(group: DxfRawGroup) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
        .ok_or_else(invalid_internal_data)
}

const fn field_id(role: DxfHatchBoundaryCircularArcEdgeRole) -> &'static str {
    match role {
        DxfHatchBoundaryCircularArcEdgeRole::CenterX => "boundary_circular_arc_center_x",
        DxfHatchBoundaryCircularArcEdgeRole::CenterY => "boundary_circular_arc_center_y",
        DxfHatchBoundaryCircularArcEdgeRole::Radius => "boundary_circular_arc_radius",
        DxfHatchBoundaryCircularArcEdgeRole::StartAngle => "boundary_circular_arc_start_angle",
        DxfHatchBoundaryCircularArcEdgeRole::EndAngle => "boundary_circular_arc_end_angle",
        DxfHatchBoundaryCircularArcEdgeRole::Counterclockwise => {
            "boundary_circular_arc_counterclockwise"
        }
    }
}

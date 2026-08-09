//! Source-anchored numeric semantics for HATCH boundary EllipticArc edge fields.

use crate::{
    DXF_HATCH_BOUNDARY_ELLIPTIC_ARC_EDGE_ROLES, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError, DxfHatchBoundaryEdgeType,
    DxfHatchBoundaryEllipticArcEdgeCard, DxfHatchBoundaryEllipticArcEdgeCardDirectory,
    DxfHatchBoundaryEllipticArcEdgeCardState, DxfHatchBoundaryEllipticArcEdgeRole,
    DxfRawDocumentView, DxfRawGroup, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId,
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
pub enum DxfHatchBoundaryEllipticArcEdgeNumericIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
}

pub type DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue =
    DxfSemanticValue<DxfDouble, DxfHatchBoundaryEllipticArcEdgeNumericIssue>;

pub type DxfHatchBoundaryEllipticArcEdgeNumericIntegerValue =
    DxfSemanticValue<i16, DxfHatchBoundaryEllipticArcEdgeNumericIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeNumericComponents {
    center_x: DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
    center_y: DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
    major_axis_endpoint_x: DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
    major_axis_endpoint_y: DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
    minor_to_major_ratio: DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
    start_angle: DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
    end_angle: DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
    counterclockwise: DxfHatchBoundaryEllipticArcEdgeNumericIntegerValue,
}

impl DxfHatchBoundaryEllipticArcEdgeNumericComponents {
    #[must_use]
    pub const fn center_x(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue {
        &self.center_x
    }

    #[must_use]
    pub const fn center_y(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue {
        &self.center_y
    }

    #[must_use]
    pub const fn major_axis_endpoint_x(
        &self,
    ) -> &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue {
        &self.major_axis_endpoint_x
    }

    #[must_use]
    pub const fn major_axis_endpoint_y(
        &self,
    ) -> &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue {
        &self.major_axis_endpoint_y
    }

    #[must_use]
    pub const fn minor_to_major_ratio(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue {
        &self.minor_to_major_ratio
    }

    #[must_use]
    pub const fn start_angle(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue {
        &self.start_angle
    }

    #[must_use]
    pub const fn end_angle(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue {
        &self.end_angle
    }

    #[must_use]
    pub const fn counterclockwise(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericIntegerValue {
        &self.counterclockwise
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeNumericEntry {
    ordinal: u32,
    edge_ordinal: u32,
    path_ordinal: u32,
    components: DxfHatchBoundaryEllipticArcEdgeNumericComponents,
}

impl DxfHatchBoundaryEllipticArcEdgeNumericEntry {
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
    pub const fn components(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericComponents {
        &self.components
    }
}

/// One source-stable numeric entry per grouped edge typed as EllipticArc.
#[derive(Debug)]
pub struct DxfHatchBoundaryEllipticArcEdgeNumericDirectory {
    source_id: DxfSourceId,
    cards: DxfHatchBoundaryEllipticArcEdgeCardDirectory,
    entries: Box<[DxfHatchBoundaryEllipticArcEdgeNumericEntry]>,
}

impl DxfHatchBoundaryEllipticArcEdgeNumericDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.hatch_boundary_elliptic_arc_edge_card_directory(cancellation)?;
        ensure_source(document.source_id(), cards.source_id())?;
        let role_count = DXF_HATCH_BOUNDARY_ELLIPTIC_ARC_EDGE_ROLES.len();
        if cards.cards().len() % role_count != 0 {
            return Err(invalid_internal_data());
        }
        let elliptic_arc_count = cards.cards().len() / role_count;
        let mut entries = Vec::new();
        entries
            .try_reserve(elliptic_arc_count)
            .map_err(|_| out_of_memory())?;
        for edge in cards.edge_type_directory().entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if edge.edge_type() != Ok(DxfHatchBoundaryEdgeType::EllipticArc) {
                continue;
            }
            entries.push(DxfHatchBoundaryEllipticArcEdgeNumericEntry {
                ordinal: compact_len(entries.len())?,
                edge_ordinal: compact_u64(edge.ordinal())?,
                path_ordinal: compact_u64(edge.edge().path_ordinal())?,
                components: DxfHatchBoundaryEllipticArcEdgeNumericComponents {
                    center_x: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::CenterX,
                        cancellation,
                    )?,
                    center_y: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::CenterY,
                        cancellation,
                    )?,
                    major_axis_endpoint_x: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::MajorAxisEndpointX,
                        cancellation,
                    )?,
                    major_axis_endpoint_y: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::MajorAxisEndpointY,
                        cancellation,
                    )?,
                    minor_to_major_ratio: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::MinorToMajorRatio,
                        cancellation,
                    )?,
                    start_angle: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::StartAngle,
                        cancellation,
                    )?,
                    end_angle: double_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::EndAngle,
                        cancellation,
                    )?,
                    counterclockwise: integer_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryEllipticArcEdgeRole::Counterclockwise,
                        cancellation,
                    )?,
                },
            });
        }
        if entries.len() != elliptic_arc_count {
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
    pub const fn card_directory(&self) -> &DxfHatchBoundaryEllipticArcEdgeCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryEllipticArcEdgeNumericEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryEllipticArcEdgeNumericEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryEllipticArcEdgeNumericEntry> {
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
    ) -> Option<&[DxfHatchBoundaryEllipticArcEdgeNumericEntry]> {
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
    pub fn hatch_boundary_elliptic_arc_edge_numeric_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeNumericDirectory, DxfError> {
        DxfHatchBoundaryEllipticArcEdgeNumericDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_numeric_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_numeric_directory(cancellation)
    }
}

fn double_component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundaryEllipticArcEdgeCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundaryEllipticArcEdgeRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue, DxfError> {
    let (card, field) = card_and_field(cards, edge_ordinal, role)?;
    match card.state() {
        DxfHatchBoundaryEllipticArcEdgeCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundaryEllipticArcEdgeCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundaryEllipticArcEdgeNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundaryEllipticArcEdgeCardState::Unique => {
            let group = unique_group(cards, card)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_double(document, group, cancellation)? {
                Ok(value) if value.is_finite() => DxfSemanticValue::explicit(value, field, raw),
                Ok(value) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryEllipticArcEdgeNumericIssue::NonFiniteDouble(value),
                    field,
                    Some(raw),
                ),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn integer_component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundaryEllipticArcEdgeCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundaryEllipticArcEdgeRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundaryEllipticArcEdgeNumericIntegerValue, DxfError> {
    let (card, field) = card_and_field(cards, edge_ordinal, role)?;
    match card.state() {
        DxfHatchBoundaryEllipticArcEdgeCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundaryEllipticArcEdgeCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundaryEllipticArcEdgeNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundaryEllipticArcEdgeCardState::Unique => {
            let group = unique_group(cards, card)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_i16(document, group, cancellation)? {
                Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn card_and_field(
    cards: &DxfHatchBoundaryEllipticArcEdgeCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundaryEllipticArcEdgeRole,
) -> Result<
    (
        DxfHatchBoundaryEllipticArcEdgeCard,
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
    cards: &DxfHatchBoundaryEllipticArcEdgeCardDirectory,
    card: DxfHatchBoundaryEllipticArcEdgeCard,
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

const fn field_id(role: DxfHatchBoundaryEllipticArcEdgeRole) -> &'static str {
    match role {
        DxfHatchBoundaryEllipticArcEdgeRole::CenterX => "boundary_elliptic_arc_center_x",
        DxfHatchBoundaryEllipticArcEdgeRole::CenterY => "boundary_elliptic_arc_center_y",
        DxfHatchBoundaryEllipticArcEdgeRole::MajorAxisEndpointX => {
            "boundary_elliptic_arc_major_axis_endpoint_x"
        }
        DxfHatchBoundaryEllipticArcEdgeRole::MajorAxisEndpointY => {
            "boundary_elliptic_arc_major_axis_endpoint_y"
        }
        DxfHatchBoundaryEllipticArcEdgeRole::MinorToMajorRatio => {
            "boundary_elliptic_arc_minor_to_major_ratio"
        }
        DxfHatchBoundaryEllipticArcEdgeRole::StartAngle => "boundary_elliptic_arc_start_angle",
        DxfHatchBoundaryEllipticArcEdgeRole::EndAngle => "boundary_elliptic_arc_end_angle",
        DxfHatchBoundaryEllipticArcEdgeRole::Counterclockwise => {
            "boundary_elliptic_arc_counterclockwise"
        }
    }
}

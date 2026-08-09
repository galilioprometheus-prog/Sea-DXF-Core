//! Source-anchored numeric values for fixed HATCH boundary Spline-edge headers.

use crate::{
    DXF_HATCH_BOUNDARY_SPLINE_EDGE_HEADER_ROLES, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHatchBoundaryEdgeType,
    DxfHatchBoundarySplineEdgeHeaderCard, DxfHatchBoundarySplineEdgeHeaderCardDirectory,
    DxfHatchBoundarySplineEdgeHeaderCardState, DxfHatchBoundarySplineEdgeHeaderRole,
    DxfRawDocumentView, DxfRawGroup, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId,
    raw_integer::{decode_raw_i16, decode_raw_i32},
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

const HATCH_NAMESPACE: &str = "entity.hatch";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgeHeaderNumericIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

pub type DxfHatchBoundarySplineEdgeHeaderNumericI16Value =
    DxfSemanticValue<i16, DxfHatchBoundarySplineEdgeHeaderNumericIssue>;

pub type DxfHatchBoundarySplineEdgeHeaderNumericI32Value =
    DxfSemanticValue<i32, DxfHatchBoundarySplineEdgeHeaderNumericIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgeHeaderNumericComponents {
    degree: DxfHatchBoundarySplineEdgeHeaderNumericI32Value,
    rational: DxfHatchBoundarySplineEdgeHeaderNumericI16Value,
    periodic: DxfHatchBoundarySplineEdgeHeaderNumericI16Value,
    knot_count: DxfHatchBoundarySplineEdgeHeaderNumericI32Value,
    control_point_count: DxfHatchBoundarySplineEdgeHeaderNumericI32Value,
}

impl DxfHatchBoundarySplineEdgeHeaderNumericComponents {
    #[must_use]
    pub const fn degree(&self) -> &DxfHatchBoundarySplineEdgeHeaderNumericI32Value {
        &self.degree
    }

    #[must_use]
    pub const fn rational(&self) -> &DxfHatchBoundarySplineEdgeHeaderNumericI16Value {
        &self.rational
    }

    #[must_use]
    pub const fn periodic(&self) -> &DxfHatchBoundarySplineEdgeHeaderNumericI16Value {
        &self.periodic
    }

    #[must_use]
    pub const fn knot_count(&self) -> &DxfHatchBoundarySplineEdgeHeaderNumericI32Value {
        &self.knot_count
    }

    #[must_use]
    pub const fn control_point_count(&self) -> &DxfHatchBoundarySplineEdgeHeaderNumericI32Value {
        &self.control_point_count
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgeHeaderNumericEntry {
    ordinal: u32,
    edge_ordinal: u32,
    path_ordinal: u32,
    components: DxfHatchBoundarySplineEdgeHeaderNumericComponents,
}

impl DxfHatchBoundarySplineEdgeHeaderNumericEntry {
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
    pub const fn components(&self) -> &DxfHatchBoundarySplineEdgeHeaderNumericComponents {
        &self.components
    }
}

/// One source-stable numeric entry per grouped edge typed as Spline.
#[derive(Debug)]
pub struct DxfHatchBoundarySplineEdgeHeaderNumericDirectory {
    source_id: DxfSourceId,
    cards: DxfHatchBoundarySplineEdgeHeaderCardDirectory,
    entries: Box<[DxfHatchBoundarySplineEdgeHeaderNumericEntry]>,
}

impl DxfHatchBoundarySplineEdgeHeaderNumericDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.hatch_boundary_spline_edge_header_card_directory(cancellation)?;
        ensure_source(document.source_id(), cards.source_id())?;
        let role_count = DXF_HATCH_BOUNDARY_SPLINE_EDGE_HEADER_ROLES.len();
        if cards.cards().len() % role_count != 0 {
            return Err(invalid_internal_data());
        }
        let spline_count = cards.cards().len() / role_count;
        let mut entries = Vec::new();
        entries
            .try_reserve(spline_count)
            .map_err(|_| out_of_memory())?;
        for edge in cards.edge_type_directory().entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if edge.edge_type() != Ok(DxfHatchBoundaryEdgeType::Spline) {
                continue;
            }
            entries.push(DxfHatchBoundarySplineEdgeHeaderNumericEntry {
                ordinal: compact_len(entries.len())?,
                edge_ordinal: compact_u64(edge.ordinal())?,
                path_ordinal: compact_u64(edge.edge().path_ordinal())?,
                components: DxfHatchBoundarySplineEdgeHeaderNumericComponents {
                    degree: i32_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundarySplineEdgeHeaderRole::Degree,
                        cancellation,
                    )?,
                    rational: i16_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundarySplineEdgeHeaderRole::Rational,
                        cancellation,
                    )?,
                    periodic: i16_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundarySplineEdgeHeaderRole::Periodic,
                        cancellation,
                    )?,
                    knot_count: i32_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundarySplineEdgeHeaderRole::KnotCount,
                        cancellation,
                    )?,
                    control_point_count: i32_component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundarySplineEdgeHeaderRole::ControlPointCount,
                        cancellation,
                    )?,
                },
            });
        }
        if entries.len() != spline_count {
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
    pub const fn card_directory(&self) -> &DxfHatchBoundarySplineEdgeHeaderCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundarySplineEdgeHeaderNumericEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgeHeaderNumericEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundarySplineEdgeHeaderNumericEntry> {
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
    ) -> Option<&[DxfHatchBoundarySplineEdgeHeaderNumericEntry]> {
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
    pub fn hatch_boundary_spline_edge_header_numeric_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderNumericDirectory, DxfError> {
        DxfHatchBoundarySplineEdgeHeaderNumericDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_header_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_header_numeric_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_header_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_header_numeric_directory(cancellation)
    }
}

fn i16_component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundarySplineEdgeHeaderCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundarySplineEdgeHeaderRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundarySplineEdgeHeaderNumericI16Value, DxfError> {
    let (card, field) = card_and_field(cards, edge_ordinal, role)?;
    match card.state() {
        DxfHatchBoundarySplineEdgeHeaderCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundarySplineEdgeHeaderCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundarySplineEdgeHeaderNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundarySplineEdgeHeaderCardState::Unique => {
            let group = unique_group(cards, card)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_i16(document, group, cancellation)? {
                Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchBoundarySplineEdgeHeaderNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn i32_component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundarySplineEdgeHeaderCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundarySplineEdgeHeaderRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundarySplineEdgeHeaderNumericI32Value, DxfError> {
    let (card, field) = card_and_field(cards, edge_ordinal, role)?;
    match card.state() {
        DxfHatchBoundarySplineEdgeHeaderCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundarySplineEdgeHeaderCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundarySplineEdgeHeaderNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundarySplineEdgeHeaderCardState::Unique => {
            let group = unique_group(cards, card)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_i32(document, group, cancellation)? {
                Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchBoundarySplineEdgeHeaderNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn card_and_field(
    cards: &DxfHatchBoundarySplineEdgeHeaderCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundarySplineEdgeHeaderRole,
) -> Result<
    (
        DxfHatchBoundarySplineEdgeHeaderCard,
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
    cards: &DxfHatchBoundarySplineEdgeHeaderCardDirectory,
    card: DxfHatchBoundarySplineEdgeHeaderCard,
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

const fn field_id(role: DxfHatchBoundarySplineEdgeHeaderRole) -> &'static str {
    match role {
        DxfHatchBoundarySplineEdgeHeaderRole::Degree => "boundary_spline_degree",
        DxfHatchBoundarySplineEdgeHeaderRole::Rational => "boundary_spline_rational",
        DxfHatchBoundarySplineEdgeHeaderRole::Periodic => "boundary_spline_periodic",
        DxfHatchBoundarySplineEdgeHeaderRole::KnotCount => "boundary_spline_knot_count",
        DxfHatchBoundarySplineEdgeHeaderRole::ControlPointCount => {
            "boundary_spline_control_point_count"
        }
    }
}

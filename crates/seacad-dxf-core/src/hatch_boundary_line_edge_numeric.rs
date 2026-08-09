//! Source-anchored numeric semantics for HATCH boundary Line edge fields.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfHatchBoundaryEdgeType, DxfHatchBoundaryLineEdgeCard,
    DxfHatchBoundaryLineEdgeCardDirectory, DxfHatchBoundaryLineEdgeCardState,
    DxfHatchBoundaryLineEdgeRole, DxfRawDocumentView, DxfRawGroup, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    raw_double::decode_raw_double,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

const HATCH_NAMESPACE: &str = "entity.hatch";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeNumericIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
}

pub type DxfHatchBoundaryLineEdgeNumericValue =
    DxfSemanticValue<DxfDouble, DxfHatchBoundaryLineEdgeNumericIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeNumericComponents {
    start_x: DxfHatchBoundaryLineEdgeNumericValue,
    start_y: DxfHatchBoundaryLineEdgeNumericValue,
    end_x: DxfHatchBoundaryLineEdgeNumericValue,
    end_y: DxfHatchBoundaryLineEdgeNumericValue,
}

impl DxfHatchBoundaryLineEdgeNumericComponents {
    #[must_use]
    pub const fn start_x(&self) -> &DxfHatchBoundaryLineEdgeNumericValue {
        &self.start_x
    }

    #[must_use]
    pub const fn start_y(&self) -> &DxfHatchBoundaryLineEdgeNumericValue {
        &self.start_y
    }

    #[must_use]
    pub const fn end_x(&self) -> &DxfHatchBoundaryLineEdgeNumericValue {
        &self.end_x
    }

    #[must_use]
    pub const fn end_y(&self) -> &DxfHatchBoundaryLineEdgeNumericValue {
        &self.end_y
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeNumericEntry {
    ordinal: u32,
    edge_ordinal: u32,
    path_ordinal: u32,
    components: DxfHatchBoundaryLineEdgeNumericComponents,
}

impl DxfHatchBoundaryLineEdgeNumericEntry {
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
    pub const fn components(&self) -> &DxfHatchBoundaryLineEdgeNumericComponents {
        &self.components
    }
}

/// One source-stable numeric entry per grouped edge typed as Line.
#[derive(Debug)]
pub struct DxfHatchBoundaryLineEdgeNumericDirectory {
    source_id: DxfSourceId,
    cards: DxfHatchBoundaryLineEdgeCardDirectory,
    entries: Box<[DxfHatchBoundaryLineEdgeNumericEntry]>,
}

impl DxfHatchBoundaryLineEdgeNumericDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.hatch_boundary_line_edge_card_directory(cancellation)?;
        ensure_source(document.source_id(), cards.source_id())?;
        if cards.cards().len() % 4 != 0 {
            return Err(invalid_internal_data());
        }
        let line_count = cards.cards().len() / 4;
        let mut entries = Vec::new();
        entries
            .try_reserve(line_count)
            .map_err(|_| out_of_memory())?;
        for edge in cards.edge_type_directory().entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if edge.edge_type() != Ok(DxfHatchBoundaryEdgeType::Line) {
                continue;
            }
            entries.push(DxfHatchBoundaryLineEdgeNumericEntry {
                ordinal: compact_len(entries.len())?,
                edge_ordinal: compact_u64(edge.ordinal())?,
                path_ordinal: compact_u64(edge.edge().path_ordinal())?,
                components: DxfHatchBoundaryLineEdgeNumericComponents {
                    start_x: component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryLineEdgeRole::StartX,
                        cancellation,
                    )?,
                    start_y: component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryLineEdgeRole::StartY,
                        cancellation,
                    )?,
                    end_x: component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryLineEdgeRole::EndX,
                        cancellation,
                    )?,
                    end_y: component(
                        document,
                        &cards,
                        edge.ordinal(),
                        DxfHatchBoundaryLineEdgeRole::EndY,
                        cancellation,
                    )?,
                },
            });
        }
        if entries.len() != line_count {
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
    pub const fn card_directory(&self) -> &DxfHatchBoundaryLineEdgeCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryLineEdgeNumericEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryLineEdgeNumericEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryLineEdgeNumericEntry> {
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
    ) -> Option<&[DxfHatchBoundaryLineEdgeNumericEntry]> {
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
    pub fn hatch_boundary_line_edge_numeric_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeNumericDirectory, DxfError> {
        DxfHatchBoundaryLineEdgeNumericDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_line_edge_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_numeric_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_line_edge_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_numeric_directory(cancellation)
    }
}

fn component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundaryLineEdgeCardDirectory,
    edge_ordinal: u64,
    role: DxfHatchBoundaryLineEdgeRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundaryLineEdgeNumericValue, DxfError> {
    let card = cards
        .card_for_role(edge_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), HATCH_NAMESPACE, field_id(role));
    match card.state() {
        DxfHatchBoundaryLineEdgeCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundaryLineEdgeCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundaryLineEdgeNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundaryLineEdgeCardState::Unique => {
            let group = unique_group(cards, card)?;
            let raw = raw_provenance(group)?;
            Ok(match decode_raw_double(document, group, cancellation)? {
                Ok(value) if value.is_finite() => DxfSemanticValue::explicit(value, field, raw),
                Ok(value) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryLineEdgeNumericIssue::NonFiniteDouble(value),
                    field,
                    Some(raw),
                ),
                Err(issue) => DxfSemanticValue::invalid(
                    DxfHatchBoundaryLineEdgeNumericIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn unique_group(
    cards: &DxfHatchBoundaryLineEdgeCardDirectory,
    card: DxfHatchBoundaryLineEdgeCard,
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

const fn field_id(role: DxfHatchBoundaryLineEdgeRole) -> &'static str {
    match role {
        DxfHatchBoundaryLineEdgeRole::StartX => "boundary_line_start_x",
        DxfHatchBoundaryLineEdgeRole::StartY => "boundary_line_start_y",
        DxfHatchBoundaryLineEdgeRole::EndX => "boundary_line_end_x",
        DxfHatchBoundaryLineEdgeRole::EndY => "boundary_line_end_y",
    }
}

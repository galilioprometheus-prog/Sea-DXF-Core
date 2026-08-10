//! Source-anchored numeric values for grouped HATCH Spline edge points.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfFillMeshField, DxfHatchBoundarySplineEdgePointCard,
    DxfHatchBoundarySplineEdgePointCardDirectory, DxfHatchBoundarySplineEdgePointCardState,
    DxfHatchBoundarySplineEdgePointKind, DxfHatchBoundarySplineEdgePointMemberRole,
    DxfHatchBoundarySplineEdgePointTuple, DxfRawDocumentView, DxfRawGroup, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    raw_double::decode_raw_double,
    read_support::{
        compact_len, ensure_not_cancelled, ensure_source, invalid_internal_data, out_of_memory,
    },
};

const HATCH_NAMESPACE: &str = "entity.hatch";

/// Why one Spline point component has no selected finite numeric value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgePointNumericIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
}

pub type DxfHatchBoundarySplineEdgePointNumericValue =
    DxfSemanticValue<DxfDouble, DxfHatchBoundarySplineEdgePointNumericIssue>;

/// Numeric X/Y evidence and the applicable control-point Weight evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointNumericComponents {
    x: DxfHatchBoundarySplineEdgePointNumericValue,
    y: DxfHatchBoundarySplineEdgePointNumericValue,
    weight: Option<DxfHatchBoundarySplineEdgePointNumericValue>,
}

impl DxfHatchBoundarySplineEdgePointNumericComponents {
    #[must_use]
    pub const fn x(&self) -> &DxfHatchBoundarySplineEdgePointNumericValue {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &DxfHatchBoundarySplineEdgePointNumericValue {
        &self.y
    }

    #[must_use]
    pub const fn weight(&self) -> Option<&DxfHatchBoundarySplineEdgePointNumericValue> {
        self.weight.as_ref()
    }
}

/// One source-stable numeric entry for one exact M14.4ap point tuple.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointNumericEntry {
    ordinal: u32,
    point_tuple: DxfHatchBoundarySplineEdgePointTuple,
    components: DxfHatchBoundarySplineEdgePointNumericComponents,
}

impl DxfHatchBoundarySplineEdgePointNumericEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn point_tuple(self) -> DxfHatchBoundarySplineEdgePointTuple {
        self.point_tuple
    }

    #[must_use]
    pub const fn components(&self) -> &DxfHatchBoundarySplineEdgePointNumericComponents {
        &self.components
    }
}

/// Numeric point entries retaining all lower card and tuple evidence.
#[derive(Debug)]
pub struct DxfHatchBoundarySplineEdgePointNumericDirectory {
    source_id: DxfSourceId,
    cards: DxfHatchBoundarySplineEdgePointCardDirectory,
    entries: Box<[DxfHatchBoundarySplineEdgePointNumericEntry]>,
}

impl DxfHatchBoundarySplineEdgePointNumericDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.hatch_boundary_spline_edge_point_card_directory(cancellation)?;
        ensure_source(document.source_id(), cards.source_id())?;
        let tuples = cards.point_tuple_directory().tuples();
        let mut entries = Vec::new();
        entries
            .try_reserve(tuples.len())
            .map_err(|_| out_of_memory())?;
        for point_tuple in tuples.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let kind = point_tuple.kind();
            let x = direct_component(
                document,
                cards.source_id(),
                point_tuple.x_field(),
                kind,
                DxfHatchBoundarySplineEdgePointMemberRole::X,
                cancellation,
            )?;
            let y = card_component(
                document,
                &cards,
                point_tuple.ordinal(),
                kind,
                DxfHatchBoundarySplineEdgePointMemberRole::Y,
                cancellation,
            )?;
            let weight = match kind {
                DxfHatchBoundarySplineEdgePointKind::ControlPoint => Some(card_component(
                    document,
                    &cards,
                    point_tuple.ordinal(),
                    kind,
                    DxfHatchBoundarySplineEdgePointMemberRole::Weight,
                    cancellation,
                )?),
                DxfHatchBoundarySplineEdgePointKind::FitPoint => None,
            };
            entries.push(DxfHatchBoundarySplineEdgePointNumericEntry {
                ordinal: compact_len(entries.len())?,
                point_tuple,
                components: DxfHatchBoundarySplineEdgePointNumericComponents { x, y, weight },
            });
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
    pub const fn card_directory(&self) -> &DxfHatchBoundarySplineEdgePointCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundarySplineEdgePointNumericEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgePointNumericEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_tuple(
        &self,
        tuple_ordinal: u64,
    ) -> Option<DxfHatchBoundarySplineEdgePointNumericEntry> {
        let index = self
            .entries
            .binary_search_by_key(&tuple_ordinal, |entry| entry.point_tuple().ordinal())
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_edge(
        &self,
        edge_ordinal: u64,
        kind: DxfHatchBoundarySplineEdgePointKind,
    ) -> Option<&[DxfHatchBoundarySplineEdgePointNumericEntry]> {
        self.cards
            .point_tuple_directory()
            .tuples_for_edge(edge_ordinal, kind)?;
        let key = (edge_ordinal, kind);
        let start = self.entries.partition_point(|entry| {
            (
                entry.point_tuple().edge_ordinal(),
                entry.point_tuple().kind(),
            ) < key
        });
        let end = self.entries.partition_point(|entry| {
            (
                entry.point_tuple().edge_ordinal(),
                entry.point_tuple().kind(),
            ) <= key
        });
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_spline_edge_point_numeric_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointNumericDirectory, DxfError> {
        DxfHatchBoundarySplineEdgePointNumericDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_point_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_point_numeric_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_point_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_point_numeric_directory(cancellation)
    }
}

fn direct_component(
    document: DxfRawDocumentView<'_>,
    source_id: DxfSourceId,
    source: DxfFillMeshField,
    kind: DxfHatchBoundarySplineEdgePointKind,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundarySplineEdgePointNumericValue, DxfError> {
    let field = field_provenance(source_id, kind, role)?;
    decode_component(document, source.group(), field, cancellation)
}

fn card_component(
    document: DxfRawDocumentView<'_>,
    cards: &DxfHatchBoundarySplineEdgePointCardDirectory,
    tuple_ordinal: u64,
    kind: DxfHatchBoundarySplineEdgePointKind,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundarySplineEdgePointNumericValue, DxfError> {
    let card = cards
        .card_for_role(tuple_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    let field = field_provenance(cards.source_id(), kind, role)?;
    match card.state() {
        DxfHatchBoundarySplineEdgePointCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHatchBoundarySplineEdgePointCardState::Multiple { occurrence_count } => {
            Ok(DxfSemanticValue::invalid(
                DxfHatchBoundarySplineEdgePointNumericIssue::MultipleValues { occurrence_count },
                field,
                None,
            ))
        }
        DxfHatchBoundarySplineEdgePointCardState::Unique => {
            let group = unique_group(cards, card)?;
            decode_component(document, group, field, cancellation)
        }
    }
}

fn decode_component(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    field: DxfSemanticFieldProvenance,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundarySplineEdgePointNumericValue, DxfError> {
    let raw = raw_provenance(group)?;
    Ok(match decode_raw_double(document, group, cancellation)? {
        Ok(value) if value.is_finite() => DxfSemanticValue::explicit(value, field, raw),
        Ok(value) => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgePointNumericIssue::NonFiniteDouble(value),
            field,
            Some(raw),
        ),
        Err(issue) => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgePointNumericIssue::InvalidAsciiNumber(issue),
            field,
            Some(raw),
        ),
    })
}

fn unique_group(
    cards: &DxfHatchBoundarySplineEdgePointCardDirectory,
    card: DxfHatchBoundarySplineEdgePointCard,
) -> Result<DxfRawGroup, DxfError> {
    let [member] = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    Ok(member.field().group())
}

fn field_provenance(
    source_id: DxfSourceId,
    kind: DxfHatchBoundarySplineEdgePointKind,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
) -> Result<DxfSemanticFieldProvenance, DxfError> {
    let field_id = field_id(kind, role).ok_or_else(invalid_internal_data)?;
    Ok(DxfSemanticFieldProvenance::new(
        source_id,
        HATCH_NAMESPACE,
        field_id,
    ))
}

fn raw_provenance(group: DxfRawGroup) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
        .ok_or_else(invalid_internal_data)
}

const fn field_id(
    kind: DxfHatchBoundarySplineEdgePointKind,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
) -> Option<&'static str> {
    match (kind, role) {
        (
            DxfHatchBoundarySplineEdgePointKind::ControlPoint,
            DxfHatchBoundarySplineEdgePointMemberRole::X,
        ) => Some("boundary_spline_control_point_x"),
        (
            DxfHatchBoundarySplineEdgePointKind::ControlPoint,
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
        ) => Some("boundary_spline_control_point_y"),
        (
            DxfHatchBoundarySplineEdgePointKind::ControlPoint,
            DxfHatchBoundarySplineEdgePointMemberRole::Weight,
        ) => Some("boundary_spline_control_point_weight"),
        (
            DxfHatchBoundarySplineEdgePointKind::FitPoint,
            DxfHatchBoundarySplineEdgePointMemberRole::X,
        ) => Some("boundary_spline_fit_point_x"),
        (
            DxfHatchBoundarySplineEdgePointKind::FitPoint,
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
        ) => Some("boundary_spline_fit_point_y"),
        (
            DxfHatchBoundarySplineEdgePointKind::FitPoint,
            DxfHatchBoundarySplineEdgePointMemberRole::Weight,
        ) => None,
    }
}

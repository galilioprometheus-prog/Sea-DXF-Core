//! Per-component cardinality cards for grouped HATCH Spline edge points.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchBoundarySplineEdgePointKind, DxfHatchBoundarySplineEdgePointMemberRole,
    DxfHatchBoundarySplineEdgePointTuple, DxfHatchBoundarySplineEdgePointTupleDirectory,
    DxfHatchBoundarySplineEdgePointTupleMember, DxfRawDocumentView, DxfSourceId,
    read_support::{
        compact_len, ensure_not_cancelled, ensure_source, invalid_internal_data, out_of_memory,
    },
};

const CONTROL_POINT_ROLES: [DxfHatchBoundarySplineEdgePointMemberRole; 2] = [
    DxfHatchBoundarySplineEdgePointMemberRole::Y,
    DxfHatchBoundarySplineEdgePointMemberRole::Weight,
];
const FIT_POINT_ROLES: [DxfHatchBoundarySplineEdgePointMemberRole; 1] =
    [DxfHatchBoundarySplineEdgePointMemberRole::Y];

/// Cardinality of one documented non-anchor point component.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgePointCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Exact lower-layer member retained by one point-component card.
pub type DxfHatchBoundarySplineEdgePointCardMember = DxfHatchBoundarySplineEdgePointTupleMember;

/// One component role and all of its exact members for one point tuple.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointCard {
    ordinal: u32,
    point_tuple: DxfHatchBoundarySplineEdgePointTuple,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
    member_start: u32,
    member_end: u32,
    state: DxfHatchBoundarySplineEdgePointCardState,
}

impl DxfHatchBoundarySplineEdgePointCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn point_tuple(self) -> DxfHatchBoundarySplineEdgePointTuple {
        self.point_tuple
    }

    #[must_use]
    pub const fn role(self) -> DxfHatchBoundarySplineEdgePointMemberRole {
        self.role
    }

    #[must_use]
    pub const fn member_count(self) -> u64 {
        (self.member_end - self.member_start) as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfHatchBoundarySplineEdgePointCardState {
        self.state
    }
}

/// Source-stable point cards retaining the complete lower tuple directory.
#[derive(Debug)]
pub struct DxfHatchBoundarySplineEdgePointCardDirectory {
    source_id: DxfSourceId,
    point_tuples: DxfHatchBoundarySplineEdgePointTupleDirectory,
    cards: Box<[DxfHatchBoundarySplineEdgePointCard]>,
    members: Box<[DxfHatchBoundarySplineEdgePointCardMember]>,
}

impl DxfHatchBoundarySplineEdgePointCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let point_tuples =
            document.hatch_boundary_spline_edge_point_tuple_directory(cancellation)?;
        ensure_source(document.source_id(), point_tuples.source_id())?;
        let mut cards = Vec::new();
        let mut members = Vec::new();
        for point_tuple in point_tuples.tuples().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let tuple_members = point_tuples
                .members_for_tuple(point_tuple.ordinal())
                .ok_or_else(invalid_internal_data)?;
            let roles = roles_for_kind(point_tuple.kind());
            cards
                .try_reserve(roles.len())
                .map_err(|_| out_of_memory())?;
            for role in roles.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                for member in tuple_members.iter().copied() {
                    ensure_not_cancelled(cancellation)?;
                    if member.role() == role {
                        members.try_reserve(1).map_err(|_| out_of_memory())?;
                        members.push(member);
                    }
                }
                let member_end = compact_len(members.len())?;
                let state = card_state(member_end - member_start);
                cards.push(DxfHatchBoundarySplineEdgePointCard {
                    ordinal: compact_len(cards.len())?,
                    point_tuple,
                    role,
                    member_start,
                    member_end,
                    state,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            point_tuples,
            cards: cards.into_boxed_slice(),
            members: members.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn point_tuple_directory(&self) -> &DxfHatchBoundarySplineEdgePointTupleDirectory {
        &self.point_tuples
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfHatchBoundarySplineEdgePointCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfHatchBoundarySplineEdgePointCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgePointCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_tuple(
        &self,
        tuple_ordinal: u64,
    ) -> Option<&[DxfHatchBoundarySplineEdgePointCard]> {
        self.point_tuples.tuple(tuple_ordinal)?;
        let start = self
            .cards
            .partition_point(|card| card.point_tuple().ordinal() < tuple_ordinal);
        let end = self
            .cards
            .partition_point(|card| card.point_tuple().ordinal() <= tuple_ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn cards_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<&[DxfHatchBoundarySplineEdgePointCard]> {
        self.point_tuples
            .entry_for_edge(edge_ordinal)?
            .grouping()
            .ok()?;
        let start = self
            .cards
            .partition_point(|card| card.point_tuple().edge_ordinal() < edge_ordinal);
        let end = self
            .cards
            .partition_point(|card| card.point_tuple().edge_ordinal() <= edge_ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        tuple_ordinal: u64,
        role: DxfHatchBoundarySplineEdgePointMemberRole,
    ) -> Option<DxfHatchBoundarySplineEdgePointCard> {
        self.cards_for_tuple(tuple_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(
        &self,
        card_ordinal: u64,
    ) -> Option<&[DxfHatchBoundarySplineEdgePointCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_start).ok()?;
        let end = usize::try_from(card.member_end).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn tuple_for_card(
        &self,
        card_ordinal: u64,
    ) -> Option<DxfHatchBoundarySplineEdgePointTuple> {
        Some(self.card(card_ordinal)?.point_tuple())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_spline_edge_point_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointCardDirectory, DxfError> {
        DxfHatchBoundarySplineEdgePointCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_point_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_spline_edge_point_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_point_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_spline_edge_point_card_directory(cancellation)
    }
}

const fn roles_for_kind(
    kind: DxfHatchBoundarySplineEdgePointKind,
) -> &'static [DxfHatchBoundarySplineEdgePointMemberRole] {
    match kind {
        DxfHatchBoundarySplineEdgePointKind::ControlPoint => &CONTROL_POINT_ROLES,
        DxfHatchBoundarySplineEdgePointKind::FitPoint => &FIT_POINT_ROLES,
    }
}

const fn card_state(occurrence_count: u32) -> DxfHatchBoundarySplineEdgePointCardState {
    match occurrence_count {
        0 => DxfHatchBoundarySplineEdgePointCardState::Absent,
        1 => DxfHatchBoundarySplineEdgePointCardState::Unique,
        occurrence_count => DxfHatchBoundarySplineEdgePointCardState::Multiple { occurrence_count },
    }
}

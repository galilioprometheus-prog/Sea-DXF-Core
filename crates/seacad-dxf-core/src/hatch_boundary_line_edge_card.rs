//! Per-role cardinality cards for HATCH boundary Line edge fields.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfFillMeshField,
    DxfHatchBoundaryEdgeType, DxfHatchBoundaryEdgeTypeDirectory, DxfHatchBoundaryEdgeTypeEntry,
    DxfRawDocumentView, DxfSourceId,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

pub const DXF_HATCH_BOUNDARY_LINE_EDGE_ROLES: [DxfHatchBoundaryLineEdgeRole; 4] = [
    DxfHatchBoundaryLineEdgeRole::StartX,
    DxfHatchBoundaryLineEdgeRole::StartY,
    DxfHatchBoundaryLineEdgeRole::EndX,
    DxfHatchBoundaryLineEdgeRole::EndY,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeRole {
    StartX,
    StartY,
    EndX,
    EndY,
}

impl DxfHatchBoundaryLineEdgeRole {
    #[must_use]
    pub const fn group_code(self) -> i16 {
        match self {
            Self::StartX => 10,
            Self::StartY => 20,
            Self::EndX => 11,
            Self::EndY => 21,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeMemberRange {
    start: u32,
    end: u32,
}

impl DxfHatchBoundaryLineEdgeMemberRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeMember {
    field: DxfFillMeshField,
}

impl DxfHatchBoundaryLineEdgeMember {
    #[must_use]
    pub const fn field(self) -> DxfFillMeshField {
        self.field
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeCard {
    ordinal: u32,
    edge_ordinal: u32,
    role: DxfHatchBoundaryLineEdgeRole,
    member_range: DxfHatchBoundaryLineEdgeMemberRange,
    state: DxfHatchBoundaryLineEdgeCardState,
}

impl DxfHatchBoundaryLineEdgeCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn edge_ordinal(self) -> u64 {
        self.edge_ordinal as u64
    }

    #[must_use]
    pub const fn role(self) -> DxfHatchBoundaryLineEdgeRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfHatchBoundaryLineEdgeMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfHatchBoundaryLineEdgeCardState {
        self.state
    }
}

/// Four stable field cards for each grouped edge typed as Line.
#[derive(Debug)]
pub struct DxfHatchBoundaryLineEdgeCardDirectory {
    source_id: DxfSourceId,
    edge_types: DxfHatchBoundaryEdgeTypeDirectory,
    cards: Box<[DxfHatchBoundaryLineEdgeCard]>,
    members: Box<[DxfHatchBoundaryLineEdgeMember]>,
}

impl DxfHatchBoundaryLineEdgeCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let edge_types = document.hatch_boundary_edge_type_directory(cancellation)?;
        ensure_source(document.source_id(), edge_types.source_id())?;
        let mut cards = Vec::new();
        let mut members = Vec::new();
        for entry in edge_types.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if entry.edge_type() != Ok(DxfHatchBoundaryEdgeType::Line) {
                continue;
            }
            let fields = edge_types
                .payload_fields_for_entry(entry.ordinal())
                .ok_or_else(invalid_internal_data)?;
            cards
                .try_reserve(DXF_HATCH_BOUNDARY_LINE_EDGE_ROLES.len())
                .map_err(|_| out_of_memory())?;
            for role in DXF_HATCH_BOUNDARY_LINE_EDGE_ROLES {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                for field in fields.iter().copied() {
                    ensure_not_cancelled(cancellation)?;
                    if field.group().group_code().value() != role.group_code() {
                        continue;
                    }
                    members.try_reserve(1).map_err(|_| out_of_memory())?;
                    members.push(DxfHatchBoundaryLineEdgeMember { field });
                }
                let member_end = compact_len(members.len())?;
                let member_range =
                    DxfHatchBoundaryLineEdgeMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfHatchBoundaryLineEdgeCardState::Absent,
                    1 => DxfHatchBoundaryLineEdgeCardState::Unique,
                    occurrence_count => DxfHatchBoundaryLineEdgeCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.push(DxfHatchBoundaryLineEdgeCard {
                    ordinal: compact_len(cards.len())?,
                    edge_ordinal: compact_u64(entry.ordinal())?,
                    role,
                    member_range,
                    state,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            edge_types,
            cards: cards.into_boxed_slice(),
            members: members.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn edge_type_directory(&self) -> &DxfHatchBoundaryEdgeTypeDirectory {
        &self.edge_types
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfHatchBoundaryLineEdgeCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfHatchBoundaryLineEdgeMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfHatchBoundaryLineEdgeCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfHatchBoundaryLineEdgeCard]> {
        self.edge_types.entry(edge_ordinal)?;
        let start = self
            .cards
            .partition_point(|card| card.edge_ordinal() < edge_ordinal);
        let end = self
            .cards
            .partition_point(|card| card.edge_ordinal() <= edge_ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        edge_ordinal: u64,
        role: DxfHatchBoundaryLineEdgeRole,
    ) -> Option<DxfHatchBoundaryLineEdgeCard> {
        self.cards_for_edge(edge_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfHatchBoundaryLineEdgeMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn edge_entry_for_card(&self, card_ordinal: u64) -> Option<DxfHatchBoundaryEdgeTypeEntry> {
        let card = self.card(card_ordinal)?;
        self.edge_types.entry(card.edge_ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_line_edge_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCardDirectory, DxfError> {
        DxfHatchBoundaryLineEdgeCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_line_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_line_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_card_directory(cancellation)
    }
}

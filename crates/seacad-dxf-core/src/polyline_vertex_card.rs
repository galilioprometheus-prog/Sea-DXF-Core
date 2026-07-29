//! Per-role cardinality cards over classic POLYLINE VERTEX value evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPolylineVertexValue, DxfPolylineVertexValueDirectory, DxfPolylineVertexValueEntry,
    DxfPolylineVertexValueRole, DxfRawDocumentView, DxfSourceId,
};

const VERTEX_ROLES: [DxfPolylineVertexValueRole; 13] = [
    DxfPolylineVertexValueRole::LocationX,
    DxfPolylineVertexValueRole::LocationY,
    DxfPolylineVertexValueRole::LocationZ,
    DxfPolylineVertexValueRole::StartWidth,
    DxfPolylineVertexValueRole::EndWidth,
    DxfPolylineVertexValueRole::Bulge,
    DxfPolylineVertexValueRole::CurveFitTangentDirection,
    DxfPolylineVertexValueRole::Flags,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex1,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex2,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex3,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex4,
    DxfPolylineVertexValueRole::Identifier,
];

/// Cardinality of one documented classic VERTEX value role in one raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open range of member ordinals owned by one classic VERTEX value card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfPolylineVertexCardMemberRange {
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

/// Compact reference from one card back to an M9.2c value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexCardMember {
    value_ordinal: u32,
}

impl DxfPolylineVertexCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One fixed role card for one exact classic VERTEX record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexValueCard {
    ordinal: u32,
    vertex: DxfPolylineVertexValueEntry,
    role: DxfPolylineVertexValueRole,
    member_range: DxfPolylineVertexCardMemberRange,
    state: DxfPolylineVertexValueCardState,
}

impl DxfPolylineVertexValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn vertex(self) -> DxfPolylineVertexValueEntry {
        self.vertex
    }

    #[must_use]
    pub const fn role(self) -> DxfPolylineVertexValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfPolylineVertexCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylineVertexValueCardState {
        self.state
    }
}

/// Immutable per-role cardinality index over M9.2c classic VERTEX evidence.
///
/// Every retained VERTEX receives thirteen cards in stable role order. Members
/// point back into the evidence directory; no value is copied, selected,
/// defaulted, interpreted, classified, validated, or transformed.
#[derive(Debug)]
pub struct DxfPolylineVertexCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfPolylineVertexValueDirectory,
    cards: Box<[DxfPolylineVertexValueCard]>,
    members: Box<[DxfPolylineVertexCardMember]>,
}

impl DxfPolylineVertexCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.polyline_vertex_value_directory(cancellation)?;
        if evidence.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: evidence.source_id(),
            });
        }

        let mut cards = Vec::new();
        let mut members = Vec::new();
        for vertex in evidence.vertices().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let values = evidence
                .values_for_vertex_raw_ordinal(vertex.vertex_record().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let value_base = vertex.value_range().start();

            for role in VERTEX_ROLES.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                for (local_index, value) in values.iter().copied().enumerate() {
                    if value.role() != role {
                        continue;
                    }
                    let value_ordinal = value_base
                        .checked_add(local_index as u64)
                        .ok_or_else(invalid_internal_data)?;
                    members.try_reserve(1).map_err(|_| out_of_memory())?;
                    members.push(DxfPolylineVertexCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range = DxfPolylineVertexCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfPolylineVertexValueCardState::Absent,
                    1 => DxfPolylineVertexValueCardState::Unique,
                    occurrence_count => DxfPolylineVertexValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfPolylineVertexValueCard {
                    ordinal: compact_len(cards.len())?,
                    vertex,
                    role,
                    member_range,
                    state,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            evidence,
            cards: cards.into_boxed_slice(),
            members: members.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn evidence_directory(&self) -> &DxfPolylineVertexValueDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfPolylineVertexValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfPolylineVertexCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfPolylineVertexValueCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }

    #[must_use]
    pub fn cards_for_vertex_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfPolylineVertexValueCard]> {
        self.evidence.vertex_for_raw_ordinal(raw_record_ordinal)?;
        let start = self
            .cards
            .partition_point(|card| card.vertex().vertex_record().ordinal() < raw_record_ordinal);
        let end = self
            .cards
            .partition_point(|card| card.vertex().vertex_record().ordinal() <= raw_record_ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        raw_record_ordinal: u64,
        role: DxfPolylineVertexValueRole,
    ) -> Option<DxfPolylineVertexValueCard> {
        self.cards_for_vertex_raw_ordinal(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfPolylineVertexCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(
        &self,
        member: DxfPolylineVertexCardMember,
    ) -> Option<DxfPolylineVertexValue> {
        let index = usize::try_from(member.value_ordinal()).ok()?;
        self.evidence.values().get(index).copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_vertex_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexCardDirectory, DxfError> {
        DxfPolylineVertexCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_vertex_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_vertex_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_card_directory(cancellation)
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

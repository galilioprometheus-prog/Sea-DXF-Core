//! Per-role cardinality cards over 3DFACE, SOLID, and TRACE evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPlanarFaceDirectory, DxfPlanarFaceKind, DxfPlanarFaceRecordEntry, DxfPlanarFaceValue,
    DxfPlanarFaceValueRole, DxfRawDocumentView, DxfSourceId,
};

const FACE3D_ROLES: [DxfPlanarFaceValueRole; 13] = [
    DxfPlanarFaceValueRole::FirstCornerX,
    DxfPlanarFaceValueRole::FirstCornerY,
    DxfPlanarFaceValueRole::FirstCornerZ,
    DxfPlanarFaceValueRole::SecondCornerX,
    DxfPlanarFaceValueRole::SecondCornerY,
    DxfPlanarFaceValueRole::SecondCornerZ,
    DxfPlanarFaceValueRole::ThirdCornerX,
    DxfPlanarFaceValueRole::ThirdCornerY,
    DxfPlanarFaceValueRole::ThirdCornerZ,
    DxfPlanarFaceValueRole::FourthCornerX,
    DxfPlanarFaceValueRole::FourthCornerY,
    DxfPlanarFaceValueRole::FourthCornerZ,
    DxfPlanarFaceValueRole::InvisibleEdgeFlags,
];

const PLANAR_SOLID_ROLES: [DxfPlanarFaceValueRole; 16] = [
    DxfPlanarFaceValueRole::FirstCornerX,
    DxfPlanarFaceValueRole::FirstCornerY,
    DxfPlanarFaceValueRole::FirstCornerZ,
    DxfPlanarFaceValueRole::SecondCornerX,
    DxfPlanarFaceValueRole::SecondCornerY,
    DxfPlanarFaceValueRole::SecondCornerZ,
    DxfPlanarFaceValueRole::ThirdCornerX,
    DxfPlanarFaceValueRole::ThirdCornerY,
    DxfPlanarFaceValueRole::ThirdCornerZ,
    DxfPlanarFaceValueRole::FourthCornerX,
    DxfPlanarFaceValueRole::FourthCornerY,
    DxfPlanarFaceValueRole::FourthCornerZ,
    DxfPlanarFaceValueRole::Thickness,
    DxfPlanarFaceValueRole::ExtrusionX,
    DxfPlanarFaceValueRole::ExtrusionY,
    DxfPlanarFaceValueRole::ExtrusionZ,
];

/// Cardinality of one documented planar-face value role in one raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPlanarFaceValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open range of member ordinals owned by one planar-face value card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfPlanarFaceCardMemberRange {
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

/// Reference from one card back to an M14.1a value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceCardMember {
    value_ordinal: u32,
}

impl DxfPlanarFaceCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One fixed role card for an exact 3DFACE, SOLID, or TRACE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceValueCard {
    ordinal: u32,
    record: DxfPlanarFaceRecordEntry,
    role: DxfPlanarFaceValueRole,
    member_range: DxfPlanarFaceCardMemberRange,
    state: DxfPlanarFaceValueCardState,
}

impl DxfPlanarFaceValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfPlanarFaceRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfPlanarFaceValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfPlanarFaceCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfPlanarFaceValueCardState {
        self.state
    }
}

/// Immutable per-role cardinality index over M14.1a planar-face evidence.
///
/// Every 3DFACE receives thirteen cards and every SOLID/TRACE receives sixteen
/// cards in stable role order. Members point into retained evidence; no value
/// is copied, selected, defaulted, validated, reordered, or transformed.
#[derive(Debug)]
pub struct DxfPlanarFaceCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfPlanarFaceDirectory,
    cards: Box<[DxfPlanarFaceValueCard]>,
    members: Box<[DxfPlanarFaceCardMember]>,
}

impl DxfPlanarFaceCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.planar_face_directory(cancellation)?;
        if evidence.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: evidence.source_id(),
            });
        }

        let mut cards = Vec::new();
        let mut members = Vec::new();
        for record in evidence.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let values = evidence
                .values_for_raw_record(record.record().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let value_base = record.value_range().start();
            for role in roles_for_kind(record.kind()).iter().copied() {
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
                    members.push(DxfPlanarFaceCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range = DxfPlanarFaceCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfPlanarFaceValueCardState::Absent,
                    1 => DxfPlanarFaceValueCardState::Unique,
                    occurrence_count => DxfPlanarFaceValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfPlanarFaceValueCard {
                    ordinal: compact_len(cards.len())?,
                    record,
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
    pub const fn evidence_directory(&self) -> &DxfPlanarFaceDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfPlanarFaceValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfPlanarFaceCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfPlanarFaceValueCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfPlanarFaceValueCard]> {
        self.evidence.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = self
            .cards
            .partition_point(|card| card.record().record().ordinal() < raw_record_ordinal);
        let end = self
            .cards
            .partition_point(|card| card.record().record().ordinal() <= raw_record_ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        raw_record_ordinal: u64,
        role: DxfPlanarFaceValueRole,
    ) -> Option<DxfPlanarFaceValueCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfPlanarFaceCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(&self, member: DxfPlanarFaceCardMember) -> Option<DxfPlanarFaceValue> {
        let index = usize::try_from(member.value_ordinal()).ok()?;
        self.evidence.values().get(index).copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn planar_face_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceCardDirectory, DxfError> {
        DxfPlanarFaceCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn planar_face_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn planar_face_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_card_directory(cancellation)
    }
}

const fn roles_for_kind(kind: DxfPlanarFaceKind) -> &'static [DxfPlanarFaceValueRole] {
    match kind {
        DxfPlanarFaceKind::Face3d => &FACE3D_ROLES,
        DxfPlanarFaceKind::Solid | DxfPlanarFaceKind::Trace => &PLANAR_SOLID_ROLES,
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

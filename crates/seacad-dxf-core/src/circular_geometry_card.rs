//! Per-role cardinality cards over CIRCLE and ARC defining-value evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfCircularGeometryDirectory,
    DxfCircularGeometryKind, DxfCircularGeometryRecordEntry, DxfCircularGeometryValue,
    DxfCircularGeometryValueRole, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

const CIRCLE_ROLES: [DxfCircularGeometryValueRole; 7] = [
    DxfCircularGeometryValueRole::OcsCenterX,
    DxfCircularGeometryValueRole::OcsCenterY,
    DxfCircularGeometryValueRole::OcsCenterZ,
    DxfCircularGeometryValueRole::Radius,
    DxfCircularGeometryValueRole::ExtrusionX,
    DxfCircularGeometryValueRole::ExtrusionY,
    DxfCircularGeometryValueRole::ExtrusionZ,
];

const ARC_ROLES: [DxfCircularGeometryValueRole; 9] = [
    DxfCircularGeometryValueRole::OcsCenterX,
    DxfCircularGeometryValueRole::OcsCenterY,
    DxfCircularGeometryValueRole::OcsCenterZ,
    DxfCircularGeometryValueRole::Radius,
    DxfCircularGeometryValueRole::StartAngle,
    DxfCircularGeometryValueRole::EndAngle,
    DxfCircularGeometryValueRole::ExtrusionX,
    DxfCircularGeometryValueRole::ExtrusionY,
    DxfCircularGeometryValueRole::ExtrusionZ,
];

/// Cardinality of one documented circular value role in one raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCircularGeometryValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open range of member ordinals owned by one circular value card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCircularGeometryCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfCircularGeometryCardMemberRange {
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

/// Reference from one card back to an M8.2a value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCircularGeometryCardMember {
    value_ordinal: u32,
}

impl DxfCircularGeometryCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One fixed role card for an exact CIRCLE or ARC record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCircularGeometryValueCard {
    ordinal: u32,
    record: DxfCircularGeometryRecordEntry,
    role: DxfCircularGeometryValueRole,
    member_range: DxfCircularGeometryCardMemberRange,
    state: DxfCircularGeometryValueCardState,
}

impl DxfCircularGeometryValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfCircularGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfCircularGeometryValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfCircularGeometryCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfCircularGeometryValueCardState {
        self.state
    }
}

/// Immutable per-role cardinality index over M8.2a circular evidence.
///
/// Every CIRCLE receives seven cards and every ARC receives nine cards in a
/// stable role order. Members point back into the retained evidence directory;
/// no value is copied, selected, defaulted, validated, or transformed.
#[derive(Debug)]
pub struct DxfCircularGeometryCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfCircularGeometryDirectory,
    cards: Box<[DxfCircularGeometryValueCard]>,
    members: Box<[DxfCircularGeometryCardMember]>,
}

impl DxfCircularGeometryCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.circular_geometry_directory(cancellation)?;
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
            let roles = roles_for_kind(record.kind());
            let values = evidence
                .values_for_raw_record(record.record().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let value_base = record.value_range().start();

            for role in roles.iter().copied() {
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
                    members.push(DxfCircularGeometryCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range =
                    DxfCircularGeometryCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfCircularGeometryValueCardState::Absent,
                    1 => DxfCircularGeometryValueCardState::Unique,
                    occurrence_count => DxfCircularGeometryValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfCircularGeometryValueCard {
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
    pub const fn evidence_directory(&self) -> &DxfCircularGeometryDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfCircularGeometryValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfCircularGeometryCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfCircularGeometryValueCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfCircularGeometryValueCard]> {
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
        role: DxfCircularGeometryValueRole,
    ) -> Option<DxfCircularGeometryValueCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfCircularGeometryCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(
        &self,
        member: DxfCircularGeometryCardMember,
    ) -> Option<DxfCircularGeometryValue> {
        let index = usize::try_from(member.value_ordinal()).ok()?;
        self.evidence.values().get(index).copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn circular_geometry_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCircularGeometryCardDirectory, DxfError> {
        DxfCircularGeometryCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn circular_geometry_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCircularGeometryCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).circular_geometry_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn circular_geometry_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCircularGeometryCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).circular_geometry_card_directory(cancellation)
    }
}

const fn roles_for_kind(kind: DxfCircularGeometryKind) -> &'static [DxfCircularGeometryValueRole] {
    match kind {
        DxfCircularGeometryKind::Circle => &CIRCLE_ROLES,
        DxfCircularGeometryKind::Arc => &ARC_ROLES,
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

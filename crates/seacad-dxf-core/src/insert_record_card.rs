//! Per-role cardinality cards over exact INSERT record-value evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfInsertRecordValue, DxfInsertRecordValueDirectory, DxfInsertRecordValueEntry,
    DxfInsertRecordValueRole, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

const INSERT_ROLES: [DxfInsertRecordValueRole; 16] = [
    DxfInsertRecordValueRole::BlockName,
    DxfInsertRecordValueRole::InsertionPointX,
    DxfInsertRecordValueRole::InsertionPointY,
    DxfInsertRecordValueRole::InsertionPointZ,
    DxfInsertRecordValueRole::ScaleFactorX,
    DxfInsertRecordValueRole::ScaleFactorY,
    DxfInsertRecordValueRole::ScaleFactorZ,
    DxfInsertRecordValueRole::RotationAngle,
    DxfInsertRecordValueRole::ColumnCount,
    DxfInsertRecordValueRole::RowCount,
    DxfInsertRecordValueRole::ColumnSpacing,
    DxfInsertRecordValueRole::RowSpacing,
    DxfInsertRecordValueRole::AttributesFollow,
    DxfInsertRecordValueRole::ExtrusionX,
    DxfInsertRecordValueRole::ExtrusionY,
    DxfInsertRecordValueRole::ExtrusionZ,
];

/// Cardinality of one documented INSERT value role in one record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertRecordValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open member range owned by one INSERT value card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfInsertRecordCardMemberRange {
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

/// Compact reference from one card back to an M10.1g value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordCardMember {
    value_ordinal: u32,
}

impl DxfInsertRecordCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One fixed role card for one exact INSERT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordValueCard {
    ordinal: u32,
    record: DxfInsertRecordValueEntry,
    role: DxfInsertRecordValueRole,
    member_range: DxfInsertRecordCardMemberRange,
    state: DxfInsertRecordValueCardState,
}

impl DxfInsertRecordValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfInsertRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfInsertRecordValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfInsertRecordCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfInsertRecordValueCardState {
        self.state
    }
}

/// Immutable fixed-card directory retaining every exact INSERT value.
#[derive(Debug)]
pub struct DxfInsertRecordCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfInsertRecordValueDirectory,
    cards: Box<[DxfInsertRecordValueCard]>,
    members: Box<[DxfInsertRecordCardMember]>,
}

impl DxfInsertRecordCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.insert_record_value_directory(cancellation)?;
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
            let raw_ordinal = record.record().ordinal();
            let values = evidence
                .values_for_raw_record(raw_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let value_base = record.value_range().start();

            for role in INSERT_ROLES.iter().copied() {
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
                    members.push(DxfInsertRecordCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range = DxfInsertRecordCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfInsertRecordValueCardState::Absent,
                    1 => DxfInsertRecordValueCardState::Unique,
                    occurrence_count => DxfInsertRecordValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfInsertRecordValueCard {
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
    pub const fn evidence_directory(&self) -> &DxfInsertRecordValueDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfInsertRecordValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfInsertRecordCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfInsertRecordValueCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfInsertRecordValueCard]> {
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
        role: DxfInsertRecordValueRole,
    ) -> Option<DxfInsertRecordValueCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfInsertRecordCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(
        &self,
        member: DxfInsertRecordCardMember,
    ) -> Option<DxfInsertRecordValue> {
        let index = usize::try_from(member.value_ordinal()).ok()?;
        self.evidence.values().get(index).copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_record_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordCardDirectory, DxfError> {
        DxfInsertRecordCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_record_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_record_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_record_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_record_card_directory(cancellation)
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

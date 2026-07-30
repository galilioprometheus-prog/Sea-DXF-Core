//! Per-role cardinality cards over BLOCK record-value evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockRecordValue, DxfBlockRecordValueDirectory,
    DxfBlockRecordValueEntry, DxfBlockRecordValueRole, DxfCancellationToken, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

const RECORD_ROLES: [DxfBlockRecordValueRole; 8] = [
    DxfBlockRecordValueRole::PrimaryName,
    DxfBlockRecordValueRole::Flags,
    DxfBlockRecordValueRole::BasePointX,
    DxfBlockRecordValueRole::BasePointY,
    DxfBlockRecordValueRole::BasePointZ,
    DxfBlockRecordValueRole::SecondaryName,
    DxfBlockRecordValueRole::XrefPath,
    DxfBlockRecordValueRole::Description,
];

/// Cardinality of one documented BLOCK value role in one record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockRecordValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open member range owned by one BLOCK record card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfBlockRecordCardMemberRange {
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

/// Compact reference from one card back to an M10.1b value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordCardMember {
    value_ordinal: u32,
}

impl DxfBlockRecordCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One fixed role card for one exact BLOCK record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordValueCard {
    ordinal: u32,
    record: DxfBlockRecordValueEntry,
    role: DxfBlockRecordValueRole,
    member_range: DxfBlockRecordCardMemberRange,
    state: DxfBlockRecordValueCardState,
}

impl DxfBlockRecordValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfBlockRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfBlockRecordValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfBlockRecordCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfBlockRecordValueCardState {
        self.state
    }
}

/// Immutable per-role cardinality index over M10.1b BLOCK evidence.
#[derive(Debug)]
pub struct DxfBlockRecordCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfBlockRecordValueDirectory,
    cards: Box<[DxfBlockRecordValueCard]>,
    members: Box<[DxfBlockRecordCardMember]>,
}

impl DxfBlockRecordCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.block_record_value_directory(cancellation)?;
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
            let raw_ordinal = record.definition().block_record().ordinal();
            let values = evidence
                .values_for_block_raw_ordinal(raw_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let value_base = record.value_range().start();

            for role in RECORD_ROLES.iter().copied() {
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
                    members.push(DxfBlockRecordCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range = DxfBlockRecordCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfBlockRecordValueCardState::Absent,
                    1 => DxfBlockRecordValueCardState::Unique,
                    occurrence_count => DxfBlockRecordValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfBlockRecordValueCard {
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
    pub const fn evidence_directory(&self) -> &DxfBlockRecordValueDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfBlockRecordValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfBlockRecordCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfBlockRecordValueCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }

    #[must_use]
    pub fn cards_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBlockRecordValueCard]> {
        self.evidence
            .record_for_block_raw_ordinal(raw_record_ordinal)?;
        let start = self.cards.partition_point(|card| {
            card.record().definition().block_record().ordinal() < raw_record_ordinal
        });
        let end = self.cards.partition_point(|card| {
            card.record().definition().block_record().ordinal() <= raw_record_ordinal
        });
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        raw_record_ordinal: u64,
        role: DxfBlockRecordValueRole,
    ) -> Option<DxfBlockRecordValueCard> {
        self.cards_for_block_raw_ordinal(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfBlockRecordCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(
        &self,
        member: DxfBlockRecordCardMember,
    ) -> Option<DxfBlockRecordValue> {
        let index = usize::try_from(member.value_ordinal()).ok()?;
        self.evidence.values().get(index).copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_record_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordCardDirectory, DxfError> {
        DxfBlockRecordCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_record_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_record_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_record_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_record_card_directory(cancellation)
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

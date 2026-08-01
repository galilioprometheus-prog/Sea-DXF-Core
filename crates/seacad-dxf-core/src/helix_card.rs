//! Per-role cardinality cards over HELIX source evidence.

use std::io;

use crate::{
    DXF_HELIX_ROLES, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHelixDirectory, DxfHelixRecordEntry, DxfHelixValue, DxfHelixValueRole, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfHelixCardMemberRange {
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
pub struct DxfHelixCardMember {
    value_ordinal: u32,
}

impl DxfHelixCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixValueCard {
    ordinal: u32,
    record: DxfHelixRecordEntry,
    role: DxfHelixValueRole,
    member_range: DxfHelixCardMemberRange,
    state: DxfHelixCardState,
}

impl DxfHelixValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfHelixRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfHelixValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfHelixCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfHelixCardState {
        self.state
    }
}

/// Sixteen stable cards per HELIX record without value selection.
#[derive(Debug)]
pub struct DxfHelixCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfHelixDirectory,
    cards: Box<[DxfHelixValueCard]>,
    members: Box<[DxfHelixCardMember]>,
}

impl DxfHelixCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.helix_directory(cancellation)?;
        ensure_source(document.source_id(), evidence.source_id())?;
        let card_capacity = evidence
            .records()
            .len()
            .checked_mul(DXF_HELIX_ROLES.len())
            .ok_or_else(invalid_internal_data)?;
        let mut cards = Vec::new();
        cards
            .try_reserve(card_capacity)
            .map_err(|_| out_of_memory())?;
        let mut members = Vec::new();
        members
            .try_reserve(evidence.values().len())
            .map_err(|_| out_of_memory())?;
        for record in evidence.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw = record.entity().record().ordinal();
            let values = evidence
                .values_for_raw_record(raw)
                .ok_or_else(invalid_internal_data)?;
            append_record_cards(
                record,
                values,
                record.value_range().start(),
                cancellation,
                &mut cards,
                &mut members,
            )?;
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
    pub const fn evidence_directory(&self) -> &DxfHelixDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfHelixValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfHelixCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfHelixValueCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(&self, raw: u64) -> Option<&[DxfHelixValueCard]> {
        self.evidence.record_for_raw_ordinal(raw)?;
        let start = self
            .cards
            .partition_point(|card| card.record().entity().record().ordinal() < raw);
        let end = self
            .cards
            .partition_point(|card| card.record().entity().record().ordinal() <= raw);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(&self, raw: u64, role: DxfHelixValueRole) -> Option<DxfHelixValueCard> {
        self.cards_for_raw_record(raw)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, ordinal: u64) -> Option<&[DxfHelixCardMember]> {
        let card = self.card(ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(&self, member: DxfHelixCardMember) -> Option<DxfHelixValue> {
        self.evidence
            .values()
            .get(usize::try_from(member.value_ordinal()).ok()?)
            .copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn helix_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixCardDirectory, DxfError> {
        DxfHelixCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn helix_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn helix_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_card_directory(cancellation)
    }
}

fn append_record_cards(
    record: DxfHelixRecordEntry,
    values: &[DxfHelixValue],
    value_base: u64,
    cancellation: &DxfCancellationToken,
    cards: &mut Vec<DxfHelixValueCard>,
    members: &mut Vec<DxfHelixCardMember>,
) -> Result<(), DxfError> {
    for role in DXF_HELIX_ROLES {
        ensure_not_cancelled(cancellation)?;
        let member_start = compact_len(members.len())?;
        for (local_index, value) in values.iter().copied().enumerate() {
            if value.role() != role {
                continue;
            }
            let value_ordinal = value_base
                .checked_add(local_index as u64)
                .and_then(|ordinal| u32::try_from(ordinal).ok())
                .ok_or_else(invalid_internal_data)?;
            members.push(DxfHelixCardMember { value_ordinal });
        }
        let member_end = compact_len(members.len())?;
        let member_range = DxfHelixCardMemberRange::new(member_start, member_end)?;
        let state = match member_range.len() {
            0 => DxfHelixCardState::Absent,
            1 => DxfHelixCardState::Unique,
            occurrence_count => DxfHelixCardState::Multiple {
                occurrence_count: u32::try_from(occurrence_count)
                    .map_err(|_| invalid_internal_data())?,
            },
        };
        cards.push(DxfHelixValueCard {
            ordinal: compact_len(cards.len())?,
            record,
            role,
            member_range,
            state,
        });
    }
    Ok(())
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
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

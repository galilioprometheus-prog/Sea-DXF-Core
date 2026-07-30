//! Per-role cardinality cards over text-and-symbol source evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId, DxfTextSymbolDirectory, DxfTextSymbolRecordEntry,
    DxfTextSymbolValue, DxfTextSymbolValueRole, text_symbol_role::roles_for_kind,
};

/// Cardinality of one documented role in one text-and-symbol record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open member range owned by one role card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextSymbolCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfTextSymbolCardMemberRange {
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

/// Reference from one card to one retained M14.2a value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextSymbolCardMember {
    value_ordinal: u32,
}

impl DxfTextSymbolCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One stable role card for an exact text-and-symbol record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextSymbolValueCard {
    ordinal: u32,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    member_range: DxfTextSymbolCardMemberRange,
    state: DxfTextSymbolValueCardState,
}

impl DxfTextSymbolValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfTextSymbolValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfTextSymbolCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfTextSymbolValueCardState {
        self.state
    }
}

/// Immutable fixed-role cardinality index retaining M14.2a evidence.
#[derive(Debug)]
pub struct DxfTextSymbolCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfTextSymbolDirectory,
    cards: Box<[DxfTextSymbolValueCard]>,
    members: Box<[DxfTextSymbolCardMember]>,
}

impl DxfTextSymbolCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.text_symbol_directory(cancellation)?;
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
            for role in roles_for_kind(record.kind()).iter().copied() {
                append_card(record, role, values, &mut cards, &mut members)?;
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
    pub const fn evidence_directory(&self) -> &DxfTextSymbolDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfTextSymbolValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfTextSymbolCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfTextSymbolValueCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(&self, raw: u64) -> Option<&[DxfTextSymbolValueCard]> {
        self.evidence.record_for_raw_ordinal(raw)?;
        let start = self
            .cards
            .partition_point(|card| card.record().record().ordinal() < raw);
        let end = self
            .cards
            .partition_point(|card| card.record().record().ordinal() <= raw);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        raw: u64,
        role: DxfTextSymbolValueRole,
    ) -> Option<DxfTextSymbolValueCard> {
        self.cards_for_raw_record(raw)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, ordinal: u64) -> Option<&[DxfTextSymbolCardMember]> {
        let range = self.card(ordinal)?.member_range();
        self.members
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }

    #[must_use]
    pub fn value_for_member(&self, member: DxfTextSymbolCardMember) -> Option<DxfTextSymbolValue> {
        self.evidence
            .values()
            .get(usize::try_from(member.value_ordinal()).ok()?)
            .copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn text_symbol_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolCardDirectory, DxfError> {
        DxfTextSymbolCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_symbol_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_symbol_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_symbol_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_symbol_card_directory(cancellation)
    }
}

fn append_card(
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    values: &[DxfTextSymbolValue],
    cards: &mut Vec<DxfTextSymbolValueCard>,
    members: &mut Vec<DxfTextSymbolCardMember>,
) -> Result<(), DxfError> {
    let member_start = compact_len(members.len())?;
    for (local_index, value) in values.iter().copied().enumerate() {
        if value.role() != role {
            continue;
        }
        let value_ordinal = record
            .value_range()
            .start()
            .checked_add(local_index as u64)
            .ok_or_else(invalid_internal_data)?;
        members.try_reserve(1).map_err(|_| out_of_memory())?;
        members.push(DxfTextSymbolCardMember {
            value_ordinal: u32::try_from(value_ordinal).map_err(|_| invalid_internal_data())?,
        });
    }
    let member_end = compact_len(members.len())?;
    let member_range = DxfTextSymbolCardMemberRange::new(member_start, member_end)?;
    let state = match member_range.len() {
        0 => DxfTextSymbolValueCardState::Absent,
        1 => DxfTextSymbolValueCardState::Unique,
        count => DxfTextSymbolValueCardState::Multiple {
            occurrence_count: u32::try_from(count).map_err(|_| invalid_internal_data())?,
        },
    };
    cards.try_reserve(1).map_err(|_| out_of_memory())?;
    cards.push(DxfTextSymbolValueCard {
        ordinal: compact_len(cards.len())?,
        record,
        role,
        member_range,
        state,
    });
    Ok(())
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

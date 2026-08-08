//! Fixed per-role cardinality cards over unambiguous HATCH scalar evidence.

use std::io;

use crate::{
    DXF_HATCH_SCALAR_ROLES, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfHatchScalarDirectory, DxfHatchScalarEntry, DxfHatchScalarOccurrence,
    DxfHatchScalarRole, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchScalarCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchScalarCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfHatchScalarCardMemberRange {
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
pub struct DxfHatchScalarCardMember {
    occurrence_ordinal: u32,
}

impl DxfHatchScalarCardMember {
    #[must_use]
    pub const fn occurrence_ordinal(self) -> u64 {
        self.occurrence_ordinal as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchScalarCard {
    ordinal: u32,
    entry: DxfHatchScalarEntry,
    role: DxfHatchScalarRole,
    member_range: DxfHatchScalarCardMemberRange,
    state: DxfHatchScalarCardState,
}

impl DxfHatchScalarCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn entry(self) -> DxfHatchScalarEntry {
        self.entry
    }

    #[must_use]
    pub const fn role(self) -> DxfHatchScalarRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfHatchScalarCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfHatchScalarCardState {
        self.state
    }
}

/// Twenty-five stable cardinality cards per exact `AcDbHatch` subclass.
#[derive(Debug)]
pub struct DxfHatchScalarCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfHatchScalarDirectory,
    cards: Box<[DxfHatchScalarCard]>,
    members: Box<[DxfHatchScalarCardMember]>,
}

impl DxfHatchScalarCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.hatch_scalar_directory(cancellation)?;
        ensure_source(document.source_id(), evidence.source_id())?;
        let mut cards = Vec::new();
        let mut members = Vec::new();
        for entry in evidence.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let occurrences = evidence
                .occurrences_for_subclass(entry.subclass_ordinal())
                .ok_or_else(invalid_internal_data)?;
            append_entry_cards(
                entry,
                occurrences,
                entry.occurrence_range().start(),
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
    pub const fn evidence_directory(&self) -> &DxfHatchScalarDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfHatchScalarCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfHatchScalarCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfHatchScalarCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_subclass(&self, ordinal: u64) -> Option<&[DxfHatchScalarCard]> {
        self.evidence.entry_for_subclass(ordinal)?;
        let start = self
            .cards
            .partition_point(|card| card.entry().subclass_ordinal() < ordinal);
        let end = self
            .cards
            .partition_point(|card| card.entry().subclass_ordinal() <= ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn cards_for_raw_record(&self, raw: u64) -> &[DxfHatchScalarCard] {
        let start = self
            .cards
            .partition_point(|card| card.entry().subclass().entity().record().ordinal() < raw);
        let end = self
            .cards
            .partition_point(|card| card.entry().subclass().entity().record().ordinal() <= raw);
        self.cards.get(start..end).unwrap_or_default()
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        subclass_ordinal: u64,
        role: DxfHatchScalarRole,
    ) -> Option<DxfHatchScalarCard> {
        self.cards_for_subclass(subclass_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, ordinal: u64) -> Option<&[DxfHatchScalarCardMember]> {
        let card = self.card(ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn occurrence_for_member(
        &self,
        member: DxfHatchScalarCardMember,
    ) -> Option<DxfHatchScalarOccurrence> {
        self.evidence
            .occurrences()
            .get(usize::try_from(member.occurrence_ordinal()).ok()?)
            .copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_scalar_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarCardDirectory, DxfError> {
        DxfHatchScalarCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_scalar_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_scalar_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_scalar_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_scalar_card_directory(cancellation)
    }
}

fn append_entry_cards(
    entry: DxfHatchScalarEntry,
    occurrences: &[DxfHatchScalarOccurrence],
    occurrence_base: u64,
    cancellation: &DxfCancellationToken,
    cards: &mut Vec<DxfHatchScalarCard>,
    members: &mut Vec<DxfHatchScalarCardMember>,
) -> Result<(), DxfError> {
    for role in DXF_HATCH_SCALAR_ROLES {
        ensure_not_cancelled(cancellation)?;
        let start = compact_len(members.len())?;
        for (local_index, occurrence) in occurrences.iter().copied().enumerate() {
            if occurrence.role() != role {
                continue;
            }
            let occurrence_ordinal = occurrence_base
                .checked_add(local_index as u64)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(invalid_internal_data)?;
            members.try_reserve(1).map_err(|_| out_of_memory())?;
            members.push(DxfHatchScalarCardMember { occurrence_ordinal });
        }
        let end = compact_len(members.len())?;
        let member_range = DxfHatchScalarCardMemberRange::new(start, end)?;
        let state = match member_range.len() {
            0 => DxfHatchScalarCardState::Absent,
            1 => DxfHatchScalarCardState::Unique,
            occurrence_count => DxfHatchScalarCardState::Multiple {
                occurrence_count: u32::try_from(occurrence_count)
                    .map_err(|_| invalid_internal_data())?,
            },
        };
        cards.try_reserve(1).map_err(|_| out_of_memory())?;
        cards.push(DxfHatchScalarCard {
            ordinal: compact_len(cards.len())?,
            entry,
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

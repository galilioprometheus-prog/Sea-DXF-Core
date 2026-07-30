//! Fixed per-role cardinality cards over classic ATTRIB values.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfInsertAttributeValue, DxfInsertAttributeValueDirectory, DxfInsertAttributeValueEntry,
    DxfInsertAttributeValueRole, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

const ATTRIBUTE_ROLES: [DxfInsertAttributeValueRole; 23] = [
    DxfInsertAttributeValueRole::Thickness,
    DxfInsertAttributeValueRole::TextStartX,
    DxfInsertAttributeValueRole::TextStartY,
    DxfInsertAttributeValueRole::TextStartZ,
    DxfInsertAttributeValueRole::TextHeight,
    DxfInsertAttributeValueRole::TextValue,
    DxfInsertAttributeValueRole::AttributeTag,
    DxfInsertAttributeValueRole::AttributeFlags,
    DxfInsertAttributeValueRole::FieldLength,
    DxfInsertAttributeValueRole::RotationAngle,
    DxfInsertAttributeValueRole::RelativeXScale,
    DxfInsertAttributeValueRole::ObliqueAngle,
    DxfInsertAttributeValueRole::TextStyleName,
    DxfInsertAttributeValueRole::TextGenerationFlags,
    DxfInsertAttributeValueRole::HorizontalJustification,
    DxfInsertAttributeValueRole::VerticalJustification,
    DxfInsertAttributeValueRole::AlignmentPointX,
    DxfInsertAttributeValueRole::AlignmentPointY,
    DxfInsertAttributeValueRole::AlignmentPointZ,
    DxfInsertAttributeValueRole::ExtrusionX,
    DxfInsertAttributeValueRole::ExtrusionY,
    DxfInsertAttributeValueRole::ExtrusionZ,
    DxfInsertAttributeValueRole::VersionOrLockPosition,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfInsertAttributeCardMemberRange {
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
pub struct DxfInsertAttributeCardMember {
    value_ordinal: u32,
}

impl DxfInsertAttributeCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeValueCard {
    ordinal: u32,
    record: DxfInsertAttributeValueEntry,
    role: DxfInsertAttributeValueRole,
    member_range: DxfInsertAttributeCardMemberRange,
    state: DxfInsertAttributeValueCardState,
}

impl DxfInsertAttributeValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfInsertAttributeValueEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfInsertAttributeValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfInsertAttributeCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfInsertAttributeValueCardState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributeCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfInsertAttributeValueDirectory,
    cards: Box<[DxfInsertAttributeValueCard]>,
    members: Box<[DxfInsertAttributeCardMember]>,
}

impl DxfInsertAttributeCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.insert_attribute_value_directory(cancellation)?;
        if evidence.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: evidence.source_id(),
            });
        }
        let card_capacity = evidence
            .records()
            .len()
            .checked_mul(ATTRIBUTE_ROLES.len())
            .ok_or_else(out_of_memory)?;
        let mut cards = Vec::new();
        cards
            .try_reserve(card_capacity)
            .map_err(|_| out_of_memory())?;
        let mut members = Vec::new();
        for record in evidence.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let values = evidence
                .values_for_raw_record(record.record().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let value_base = record.value_range().start();
            for role in ATTRIBUTE_ROLES.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                for (local_index, value) in values.iter().copied().enumerate() {
                    if value.role() != role {
                        continue;
                    }
                    let local_index =
                        u64::try_from(local_index).map_err(|_| invalid_internal_data())?;
                    let value_ordinal = value_base
                        .checked_add(local_index)
                        .ok_or_else(invalid_internal_data)?;
                    members.try_reserve(1).map_err(|_| out_of_memory())?;
                    members.push(DxfInsertAttributeCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range =
                    DxfInsertAttributeCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfInsertAttributeValueCardState::Absent,
                    1 => DxfInsertAttributeValueCardState::Unique,
                    occurrence_count => DxfInsertAttributeValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.push(DxfInsertAttributeValueCard {
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
    pub const fn evidence_directory(&self) -> &DxfInsertAttributeValueDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfInsertAttributeValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfInsertAttributeCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfInsertAttributeValueCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfInsertAttributeValueCard]> {
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
        role: DxfInsertAttributeValueRole,
    ) -> Option<DxfInsertAttributeValueCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfInsertAttributeCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(
        &self,
        member: DxfInsertAttributeCardMember,
    ) -> Option<DxfInsertAttributeValue> {
        self.evidence
            .values()
            .get(usize::try_from(member.value_ordinal()).ok()?)
            .copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_attribute_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeCardDirectory, DxfError> {
        DxfInsertAttributeCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_card_directory(cancellation)
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
    io_error(std::io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(std::io::ErrorKind::OutOfMemory)
}

fn io_error(kind: std::io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &std::io::Error::from(kind))
}

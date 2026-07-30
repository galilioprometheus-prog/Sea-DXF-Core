//! Fixed per-role cardinality cards over classic ATTDEF values.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockAttributeDefinitionValue,
    DxfBlockAttributeDefinitionValueDirectory, DxfBlockAttributeDefinitionValueEntry,
    DxfBlockAttributeDefinitionValueRole, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

const ATTRIBUTE_DEFINITION_ROLES: [DxfBlockAttributeDefinitionValueRole; 24] = [
    DxfBlockAttributeDefinitionValueRole::Thickness,
    DxfBlockAttributeDefinitionValueRole::TextStartX,
    DxfBlockAttributeDefinitionValueRole::TextStartY,
    DxfBlockAttributeDefinitionValueRole::TextStartZ,
    DxfBlockAttributeDefinitionValueRole::TextHeight,
    DxfBlockAttributeDefinitionValueRole::DefaultValue,
    DxfBlockAttributeDefinitionValueRole::Prompt,
    DxfBlockAttributeDefinitionValueRole::AttributeTag,
    DxfBlockAttributeDefinitionValueRole::AttributeFlags,
    DxfBlockAttributeDefinitionValueRole::FieldLength,
    DxfBlockAttributeDefinitionValueRole::RotationAngle,
    DxfBlockAttributeDefinitionValueRole::RelativeXScale,
    DxfBlockAttributeDefinitionValueRole::ObliqueAngle,
    DxfBlockAttributeDefinitionValueRole::TextStyleName,
    DxfBlockAttributeDefinitionValueRole::TextGenerationFlags,
    DxfBlockAttributeDefinitionValueRole::HorizontalJustification,
    DxfBlockAttributeDefinitionValueRole::VerticalJustification,
    DxfBlockAttributeDefinitionValueRole::AlignmentPointX,
    DxfBlockAttributeDefinitionValueRole::AlignmentPointY,
    DxfBlockAttributeDefinitionValueRole::AlignmentPointZ,
    DxfBlockAttributeDefinitionValueRole::ExtrusionX,
    DxfBlockAttributeDefinitionValueRole::ExtrusionY,
    DxfBlockAttributeDefinitionValueRole::ExtrusionZ,
    DxfBlockAttributeDefinitionValueRole::VersionOrLockPosition,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfBlockAttributeDefinitionCardMemberRange {
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
pub struct DxfBlockAttributeDefinitionCardMember {
    value_ordinal: u32,
}

impl DxfBlockAttributeDefinitionCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionValueCard {
    ordinal: u32,
    record: DxfBlockAttributeDefinitionValueEntry,
    role: DxfBlockAttributeDefinitionValueRole,
    member_range: DxfBlockAttributeDefinitionCardMemberRange,
    state: DxfBlockAttributeDefinitionValueCardState,
}

impl DxfBlockAttributeDefinitionValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfBlockAttributeDefinitionValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfBlockAttributeDefinitionCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfBlockAttributeDefinitionValueCardState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfBlockAttributeDefinitionValueDirectory,
    cards: Box<[DxfBlockAttributeDefinitionValueCard]>,
    members: Box<[DxfBlockAttributeDefinitionCardMember]>,
}

impl DxfBlockAttributeDefinitionCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.block_attribute_definition_value_directory(cancellation)?;
        if evidence.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: evidence.source_id(),
            });
        }
        let card_capacity = evidence
            .records()
            .len()
            .checked_mul(ATTRIBUTE_DEFINITION_ROLES.len())
            .ok_or_else(out_of_memory)?;
        let mut cards = Vec::new();
        cards
            .try_reserve(card_capacity)
            .map_err(|_| out_of_memory())?;
        let mut members = Vec::new();
        for record in evidence.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw_ordinal = record.definition().record().ordinal();
            let values = evidence
                .values_for_raw_record(raw_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let value_base = record.value_range().start();
            for role in ATTRIBUTE_DEFINITION_ROLES.iter().copied() {
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
                    members.push(DxfBlockAttributeDefinitionCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range =
                    DxfBlockAttributeDefinitionCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfBlockAttributeDefinitionValueCardState::Absent,
                    1 => DxfBlockAttributeDefinitionValueCardState::Unique,
                    occurrence_count => DxfBlockAttributeDefinitionValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.push(DxfBlockAttributeDefinitionValueCard {
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
    pub const fn evidence_directory(&self) -> &DxfBlockAttributeDefinitionValueDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfBlockAttributeDefinitionValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfBlockAttributeDefinitionCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfBlockAttributeDefinitionValueCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBlockAttributeDefinitionValueCard]> {
        self.evidence.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = self.cards.partition_point(|card| {
            card.record().definition().record().ordinal() < raw_record_ordinal
        });
        let end = self.cards.partition_point(|card| {
            card.record().definition().record().ordinal() <= raw_record_ordinal
        });
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        raw_record_ordinal: u64,
        role: DxfBlockAttributeDefinitionValueRole,
    ) -> Option<DxfBlockAttributeDefinitionValueCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(
        &self,
        card_ordinal: u64,
    ) -> Option<&[DxfBlockAttributeDefinitionCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(
        &self,
        member: DxfBlockAttributeDefinitionCardMember,
    ) -> Option<DxfBlockAttributeDefinitionValue> {
        self.evidence
            .values()
            .get(usize::try_from(member.value_ordinal()).ok()?)
            .copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_attribute_definition_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionCardDirectory, DxfError> {
        DxfBlockAttributeDefinitionCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_card_directory(cancellation)
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

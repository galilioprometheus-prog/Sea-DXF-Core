//! Fixed-field cardinality cards over typed DIMSTYLE evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDimStyleField,
    DxfDimStyleValue, DxfDimStyleValueDirectory, DxfDimStyleValueEntry, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId, dxf_dimstyle_fields,
};

/// Cardinality of one documented field within one DIMSTYLE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleFieldCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open member range owned by one field card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfDimStyleCardMemberRange {
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

/// Compact reference from one card to one retained field occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleCardMember {
    value_ordinal: u32,
}

impl DxfDimStyleCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One stable field card for one exact named DIMSTYLE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleFieldCard {
    ordinal: u32,
    record: DxfDimStyleValueEntry,
    field: DxfDimStyleField,
    member_range: DxfDimStyleCardMemberRange,
    state: DxfDimStyleFieldCardState,
}

impl DxfDimStyleFieldCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfDimStyleValueEntry {
        self.record
    }

    #[must_use]
    pub const fn field(self) -> DxfDimStyleField {
        self.field
    }

    #[must_use]
    pub const fn member_range(self) -> DxfDimStyleCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfDimStyleFieldCardState {
        self.state
    }
}

/// Immutable fixed-field cardinality index retaining typed DIMSTYLE evidence.
#[derive(Debug)]
pub struct DxfDimStyleFieldCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfDimStyleValueDirectory,
    cards: Box<[DxfDimStyleFieldCard]>,
    members: Box<[DxfDimStyleCardMember]>,
}

impl DxfDimStyleFieldCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.dimstyle_value_directory(cancellation)?;
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
            let raw_ordinal = record.table_entry().record().ordinal();
            let values = evidence
                .values_for_raw_ordinal(raw_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let value_base = record.value_range().start();
            for field in dxf_dimstyle_fields().iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                for (local_index, value) in values.iter().copied().enumerate() {
                    if value.field() != field {
                        continue;
                    }
                    let value_ordinal = value_base
                        .checked_add(local_index as u64)
                        .ok_or_else(invalid_internal_data)?;
                    members.try_reserve(1).map_err(|_| out_of_memory())?;
                    members.push(DxfDimStyleCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range = DxfDimStyleCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfDimStyleFieldCardState::Absent,
                    1 => DxfDimStyleFieldCardState::Unique,
                    occurrence_count => DxfDimStyleFieldCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfDimStyleFieldCard {
                    ordinal: compact_len(cards.len())?,
                    record,
                    field,
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
    pub const fn evidence_directory(&self) -> &DxfDimStyleValueDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfDimStyleFieldCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfDimStyleCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfDimStyleFieldCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_raw_ordinal(&self, raw_ordinal: u64) -> Option<&[DxfDimStyleFieldCard]> {
        self.evidence.record_for_raw_ordinal(raw_ordinal)?;
        let start = self
            .cards
            .partition_point(|card| card.record().table_entry().record().ordinal() < raw_ordinal);
        let end = self
            .cards
            .partition_point(|card| card.record().table_entry().record().ordinal() <= raw_ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_field(
        &self,
        raw_ordinal: u64,
        field: DxfDimStyleField,
    ) -> Option<DxfDimStyleFieldCard> {
        self.cards_for_raw_ordinal(raw_ordinal)?
            .iter()
            .copied()
            .find(|card| card.field() == field)
    }

    #[must_use]
    pub fn members_for_card(&self, ordinal: u64) -> Option<&[DxfDimStyleCardMember]> {
        let card = self.card(ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(&self, member: DxfDimStyleCardMember) -> Option<DxfDimStyleValue> {
        self.evidence
            .values()
            .get(usize::try_from(member.value_ordinal()).ok()?)
            .copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn dimstyle_field_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleFieldCardDirectory, DxfError> {
        DxfDimStyleFieldCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn dimstyle_field_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleFieldCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_field_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn dimstyle_field_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleFieldCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_field_card_directory(cancellation)
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

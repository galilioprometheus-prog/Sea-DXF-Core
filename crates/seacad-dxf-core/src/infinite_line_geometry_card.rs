//! Per-role cardinality cards over RAY and XLINE defining-value evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfInfiniteLineGeometryDirectory, DxfInfiniteLineGeometryRecordEntry,
    DxfInfiniteLineGeometryValue, DxfInfiniteLineGeometryValueRole, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

const INFINITE_LINE_ROLES: [DxfInfiniteLineGeometryValueRole; 6] = [
    DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointX,
    DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointY,
    DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointZ,
    DxfInfiniteLineGeometryValueRole::WcsUnitDirectionX,
    DxfInfiniteLineGeometryValueRole::WcsUnitDirectionY,
    DxfInfiniteLineGeometryValueRole::WcsUnitDirectionZ,
];

/// Cardinality of one documented RAY/XLINE value role in one raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInfiniteLineGeometryValueCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open range of member ordinals owned by one RAY/XLINE value card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInfiniteLineGeometryCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfInfiniteLineGeometryCardMemberRange {
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

/// Reference from one card back to an M8.4a value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInfiniteLineGeometryCardMember {
    value_ordinal: u32,
}

impl DxfInfiniteLineGeometryCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

/// One fixed role card for an exact RAY or XLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInfiniteLineGeometryValueCard {
    ordinal: u32,
    record: DxfInfiniteLineGeometryRecordEntry,
    role: DxfInfiniteLineGeometryValueRole,
    member_range: DxfInfiniteLineGeometryCardMemberRange,
    state: DxfInfiniteLineGeometryValueCardState,
}

impl DxfInfiniteLineGeometryValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfInfiniteLineGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfInfiniteLineGeometryValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfInfiniteLineGeometryCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfInfiniteLineGeometryValueCardState {
        self.state
    }
}

/// Immutable per-role cardinality index over M8.4a RAY/XLINE evidence.
///
/// Every record receives six cards in a stable role order. Members point back
/// into the retained evidence directory; no value is copied, selected,
/// defaulted, validated, normalized, or transformed.
#[derive(Debug)]
pub struct DxfInfiniteLineGeometryCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfInfiniteLineGeometryDirectory,
    cards: Box<[DxfInfiniteLineGeometryValueCard]>,
    members: Box<[DxfInfiniteLineGeometryCardMember]>,
}

impl DxfInfiniteLineGeometryCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.infinite_line_geometry_directory(cancellation)?;
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

            for role in INFINITE_LINE_ROLES.iter().copied() {
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
                    members.push(DxfInfiniteLineGeometryCardMember {
                        value_ordinal: u32::try_from(value_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range =
                    DxfInfiniteLineGeometryCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfInfiniteLineGeometryValueCardState::Absent,
                    1 => DxfInfiniteLineGeometryValueCardState::Unique,
                    occurrence_count => DxfInfiniteLineGeometryValueCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfInfiniteLineGeometryValueCard {
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
    pub const fn evidence_directory(&self) -> &DxfInfiniteLineGeometryDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfInfiniteLineGeometryValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfInfiniteLineGeometryCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfInfiniteLineGeometryValueCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfInfiniteLineGeometryValueCard]> {
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
        role: DxfInfiniteLineGeometryValueRole,
    ) -> Option<DxfInfiniteLineGeometryValueCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(
        &self,
        card_ordinal: u64,
    ) -> Option<&[DxfInfiniteLineGeometryCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(
        &self,
        member: DxfInfiniteLineGeometryCardMember,
    ) -> Option<DxfInfiniteLineGeometryValue> {
        let index = usize::try_from(member.value_ordinal()).ok()?;
        self.evidence.values().get(index).copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn infinite_line_geometry_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometryCardDirectory, DxfError> {
        DxfInfiniteLineGeometryCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn infinite_line_geometry_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometryCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).infinite_line_geometry_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn infinite_line_geometry_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometryCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).infinite_line_geometry_card_directory(cancellation)
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

//! Per-role cardinality cards over SPLINE source evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId, DxfSplineDirectory, DxfSplineRecordEntry, DxfSplineValue,
    DxfSplineValueRole,
};

pub const DXF_SPLINE_ROLES: [DxfSplineValueRole; 24] = [
    DxfSplineValueRole::Flags,
    DxfSplineValueRole::Degree,
    DxfSplineValueRole::KnotCount,
    DxfSplineValueRole::ControlPointCount,
    DxfSplineValueRole::FitPointCount,
    DxfSplineValueRole::KnotTolerance,
    DxfSplineValueRole::ControlPointTolerance,
    DxfSplineValueRole::FitTolerance,
    DxfSplineValueRole::StartTangentX,
    DxfSplineValueRole::StartTangentY,
    DxfSplineValueRole::StartTangentZ,
    DxfSplineValueRole::EndTangentX,
    DxfSplineValueRole::EndTangentY,
    DxfSplineValueRole::EndTangentZ,
    DxfSplineValueRole::KnotValue,
    DxfSplineValueRole::ControlPointX,
    DxfSplineValueRole::ControlPointY,
    DxfSplineValueRole::ControlPointZ,
    DxfSplineValueRole::FitPointX,
    DxfSplineValueRole::FitPointY,
    DxfSplineValueRole::FitPointZ,
    DxfSplineValueRole::NormalX,
    DxfSplineValueRole::NormalY,
    DxfSplineValueRole::NormalZ,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfSplineCardMemberRange {
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
pub struct DxfSplineCardMember {
    value_ordinal: u32,
}

impl DxfSplineCardMember {
    #[must_use]
    pub const fn value_ordinal(self) -> u64 {
        self.value_ordinal as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineValueCard {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    role: DxfSplineValueRole,
    member_range: DxfSplineCardMemberRange,
    state: DxfSplineCardState,
}

impl DxfSplineValueCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfSplineValueRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfSplineCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineCardState {
        self.state
    }
}

/// Twenty-four stable cards per SPLINE record without value selection.
#[derive(Debug)]
pub struct DxfSplineCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfSplineDirectory,
    cards: Box<[DxfSplineValueCard]>,
    members: Box<[DxfSplineCardMember]>,
}

impl DxfSplineCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.spline_directory(cancellation)?;
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
    pub const fn evidence_directory(&self) -> &DxfSplineDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfSplineValueCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfSplineCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfSplineValueCard> {
        self.cards.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(&self, raw_ordinal: u64) -> Option<&[DxfSplineValueCard]> {
        self.evidence.record_for_raw_ordinal(raw_ordinal)?;
        let start = self
            .cards
            .partition_point(|card| card.record().record().ordinal() < raw_ordinal);
        let end = self
            .cards
            .partition_point(|card| card.record().record().ordinal() <= raw_ordinal);
        self.cards.get(start..end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        raw_ordinal: u64,
        role: DxfSplineValueRole,
    ) -> Option<DxfSplineValueCard> {
        self.cards_for_raw_record(raw_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, ordinal: u64) -> Option<&[DxfSplineCardMember]> {
        let card = self.card(ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn value_for_member(&self, member: DxfSplineCardMember) -> Option<DxfSplineValue> {
        self.evidence
            .values()
            .get(usize::try_from(member.value_ordinal()).ok()?)
            .copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineCardDirectory, DxfError> {
        DxfSplineCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_card_directory(cancellation)
    }
}

fn append_record_cards(
    record: DxfSplineRecordEntry,
    values: &[DxfSplineValue],
    value_base: u64,
    cancellation: &DxfCancellationToken,
    cards: &mut Vec<DxfSplineValueCard>,
    members: &mut Vec<DxfSplineCardMember>,
) -> Result<(), DxfError> {
    for role in DXF_SPLINE_ROLES {
        ensure_not_cancelled(cancellation)?;
        let member_start = compact_len(members.len())?;
        for (local_index, value) in values.iter().copied().enumerate() {
            if value.role() != role {
                continue;
            }
            let value_ordinal = value_base
                .checked_add(local_index as u64)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(invalid_internal_data)?;
            members.try_reserve(1).map_err(|_| out_of_memory())?;
            members.push(DxfSplineCardMember { value_ordinal });
        }
        let member_end = compact_len(members.len())?;
        let member_range = DxfSplineCardMemberRange::new(member_start, member_end)?;
        let state = match member_range.len() {
            0 => DxfSplineCardState::Absent,
            1 => DxfSplineCardState::Unique,
            occurrence_count => DxfSplineCardState::Multiple {
                occurrence_count: u32::try_from(occurrence_count)
                    .map_err(|_| invalid_internal_data())?,
            },
        };
        cards.try_reserve(1).map_err(|_| out_of_memory())?;
        cards.push(DxfSplineValueCard {
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

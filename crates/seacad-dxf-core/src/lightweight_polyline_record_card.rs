//! Record-level cardinality cards over LWPOLYLINE numeric evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfLightweightPolylineDirectory, DxfLightweightPolylineIntegerDirectory,
    DxfLightweightPolylineIntegerRole, DxfLightweightPolylineIntegerValue,
    DxfLightweightPolylineValue, DxfLightweightPolylineValueRole, DxfRawDocumentView, DxfRawRecord,
    DxfSourceId,
};

const RECORD_ROLES: [DxfLightweightPolylineRecordRole; 8] = [
    DxfLightweightPolylineRecordRole::VertexCount,
    DxfLightweightPolylineRecordRole::Flags,
    DxfLightweightPolylineRecordRole::OcsElevation,
    DxfLightweightPolylineRecordRole::Thickness,
    DxfLightweightPolylineRecordRole::ConstantWidth,
    DxfLightweightPolylineRecordRole::ExtrusionX,
    DxfLightweightPolylineRecordRole::ExtrusionY,
    DxfLightweightPolylineRecordRole::ExtrusionZ,
];

/// One documented record-level numeric role in a LWPOLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineRecordRole {
    VertexCount,
    Flags,
    OcsElevation,
    Thickness,
    ConstantWidth,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
}

/// Occurrence cardinality for one record-level LWPOLYLINE role.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineRecordCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open range of compact members owned by one record-level card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineRecordCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfLightweightPolylineRecordCardMemberRange {
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

/// Reference from one record-level card to its original numeric evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineRecordCardMember {
    role: DxfLightweightPolylineRecordRole,
    group_occurrence: u32,
}

impl DxfLightweightPolylineRecordCardMember {
    #[must_use]
    pub const fn role(self) -> DxfLightweightPolylineRecordRole {
        self.role
    }
    #[must_use]
    pub const fn group_occurrence(self) -> u64 {
        self.group_occurrence as u64
    }
}

/// One fixed record-level role card for an exact LWPOLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineRecordCard {
    ordinal: u32,
    record: DxfRawRecord,
    role: DxfLightweightPolylineRecordRole,
    member_range: DxfLightweightPolylineRecordCardMemberRange,
    state: DxfLightweightPolylineRecordCardState,
}

impl DxfLightweightPolylineRecordCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }
    #[must_use]
    pub const fn role(self) -> DxfLightweightPolylineRecordRole {
        self.role
    }
    #[must_use]
    pub const fn member_range(self) -> DxfLightweightPolylineRecordCardMemberRange {
        self.member_range
    }
    #[must_use]
    pub const fn state(self) -> DxfLightweightPolylineRecordCardState {
        self.state
    }
}

/// One exact LWPOLYLINE record and its eight fixed record-level cards.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineRecordCardEntry {
    record: DxfRawRecord,
    first_card_ordinal: u32,
}

impl DxfLightweightPolylineRecordCardEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }
    #[must_use]
    pub const fn first_card_ordinal(self) -> u64 {
        self.first_card_ordinal as u64
    }
}

/// Immutable cardinality index for record-level LWPOLYLINE numeric roles.
///
/// Every recognized record receives eight cards in stable role order. Members
/// point to M9.1a/M9.1b evidence without copying, selecting, defaulting,
/// interpreting, reconciling, or transforming any value.
#[derive(Debug)]
pub struct DxfLightweightPolylineRecordCardDirectory {
    source_id: DxfSourceId,
    floating_evidence: DxfLightweightPolylineDirectory,
    integer_evidence: DxfLightweightPolylineIntegerDirectory,
    records: Box<[DxfLightweightPolylineRecordCardEntry]>,
    cards: Box<[DxfLightweightPolylineRecordCard]>,
    members: Box<[DxfLightweightPolylineRecordCardMember]>,
}

impl DxfLightweightPolylineRecordCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let floating_evidence = document.lightweight_polyline_directory(cancellation)?;
        let integer_evidence = document.lightweight_polyline_integer_directory(cancellation)?;
        validate_evidence(document.source_id(), &floating_evidence, &integer_evidence)?;

        let mut records = Vec::new();
        let mut cards = Vec::new();
        let mut members = Vec::new();
        for (floating_record, integer_record) in floating_evidence
            .records()
            .iter()
            .copied()
            .zip(integer_evidence.records().iter().copied())
        {
            ensure_not_cancelled(cancellation)?;
            let record = floating_record.record();
            if record != integer_record.record() {
                return Err(invalid_internal_data());
            }
            let floating_values = floating_evidence
                .values_for_raw_record(record.ordinal())
                .ok_or_else(invalid_internal_data)?;
            let integer_values = integer_evidence
                .values_for_raw_record(record.ordinal())
                .ok_or_else(invalid_internal_data)?;
            let card_start = compact_len(cards.len())?;

            for role in RECORD_ROLES {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                collect_members(role, floating_values, integer_values, &mut members)?;
                let member_end = compact_len(members.len())?;
                let member_range =
                    DxfLightweightPolylineRecordCardMemberRange::new(member_start, member_end)?;
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfLightweightPolylineRecordCard {
                    ordinal: compact_len(cards.len())?,
                    record,
                    role,
                    member_range,
                    state: state_for_count(member_range.len())?,
                });
            }

            let card_end = compact_len(cards.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfLightweightPolylineRecordCardEntry {
                record,
                first_card_ordinal: card_start,
            });
            if card_end.checked_sub(card_start) != Some(RECORD_ROLES.len() as u32) {
                return Err(invalid_internal_data());
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            floating_evidence,
            integer_evidence,
            records: records.into_boxed_slice(),
            cards: cards.into_boxed_slice(),
            members: members.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }
    #[must_use]
    pub const fn floating_evidence_directory(&self) -> &DxfLightweightPolylineDirectory {
        &self.floating_evidence
    }
    #[must_use]
    pub const fn integer_evidence_directory(&self) -> &DxfLightweightPolylineIntegerDirectory {
        &self.integer_evidence
    }
    #[must_use]
    pub fn records(&self) -> &[DxfLightweightPolylineRecordCardEntry] {
        &self.records
    }
    #[must_use]
    pub fn cards(&self) -> &[DxfLightweightPolylineRecordCard] {
        &self.cards
    }
    #[must_use]
    pub fn members(&self) -> &[DxfLightweightPolylineRecordCardMember] {
        &self.members
    }
    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfLightweightPolylineRecordCardEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }
    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfLightweightPolylineRecordCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }
    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfLightweightPolylineRecordCard]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.first_card_ordinal()).ok()?;
        let end = start.checked_add(RECORD_ROLES.len())?;
        self.cards.get(start..end)
    }
    #[must_use]
    pub fn card_for_role(
        &self,
        raw_record_ordinal: u64,
        role: DxfLightweightPolylineRecordRole,
    ) -> Option<DxfLightweightPolylineRecordCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(
        &self,
        card: DxfLightweightPolylineRecordCard,
    ) -> Option<&[DxfLightweightPolylineRecordCardMember]> {
        if self.card(card.ordinal()) != Some(card) {
            return None;
        }
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn floating_value_for_member(
        &self,
        member: DxfLightweightPolylineRecordCardMember,
    ) -> Option<DxfLightweightPolylineValue> {
        let expected = floating_role(member.role())?;
        self.floating_evidence
            .value_for_group(member.group_occurrence())
            .filter(|value| value.role() == expected)
    }

    #[must_use]
    pub fn integer_value_for_member(
        &self,
        member: DxfLightweightPolylineRecordCardMember,
    ) -> Option<DxfLightweightPolylineIntegerValue> {
        let expected = integer_role(member.role())?;
        self.integer_evidence
            .value_for_group(member.group_occurrence())
            .filter(|value| value.role() == expected)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn lightweight_polyline_record_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineRecordCardDirectory, DxfError> {
        DxfLightweightPolylineRecordCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_record_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineRecordCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_record_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_record_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineRecordCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_record_card_directory(cancellation)
    }
}

fn validate_evidence(
    source_id: DxfSourceId,
    floating: &DxfLightweightPolylineDirectory,
    integer: &DxfLightweightPolylineIntegerDirectory,
) -> Result<(), DxfError> {
    for observed in [floating.source_id(), integer.source_id()] {
        if observed != source_id {
            return Err(DxfError::SourceIdentityMismatch {
                expected: source_id,
                observed,
            });
        }
    }
    if floating.raw_record_count() != integer.raw_record_count()
        || floating.records().len() != integer.records().len()
    {
        return Err(invalid_internal_data());
    }
    Ok(())
}

fn collect_members(
    role: DxfLightweightPolylineRecordRole,
    floating_values: &[DxfLightweightPolylineValue],
    integer_values: &[DxfLightweightPolylineIntegerValue],
    members: &mut Vec<DxfLightweightPolylineRecordCardMember>,
) -> Result<(), DxfError> {
    if let Some(expected) = floating_role(role) {
        for value in floating_values
            .iter()
            .copied()
            .filter(|value| value.role() == expected)
        {
            push_member(members, role, value.group().occurrence())?;
        }
    } else if let Some(expected) = integer_role(role) {
        for value in integer_values
            .iter()
            .copied()
            .filter(|value| value.role() == expected)
        {
            push_member(members, role, value.group().occurrence())?;
        }
    } else {
        return Err(invalid_internal_data());
    }
    Ok(())
}

fn push_member(
    members: &mut Vec<DxfLightweightPolylineRecordCardMember>,
    role: DxfLightweightPolylineRecordRole,
    occurrence: u64,
) -> Result<(), DxfError> {
    members.try_reserve(1).map_err(|_| out_of_memory())?;
    members.push(DxfLightweightPolylineRecordCardMember {
        role,
        group_occurrence: u32::try_from(occurrence).map_err(|_| invalid_internal_data())?,
    });
    Ok(())
}

const fn floating_role(
    role: DxfLightweightPolylineRecordRole,
) -> Option<DxfLightweightPolylineValueRole> {
    match role {
        DxfLightweightPolylineRecordRole::OcsElevation => {
            Some(DxfLightweightPolylineValueRole::OcsElevation)
        }
        DxfLightweightPolylineRecordRole::Thickness => {
            Some(DxfLightweightPolylineValueRole::Thickness)
        }
        DxfLightweightPolylineRecordRole::ConstantWidth => {
            Some(DxfLightweightPolylineValueRole::ConstantWidth)
        }
        DxfLightweightPolylineRecordRole::ExtrusionX => {
            Some(DxfLightweightPolylineValueRole::ExtrusionX)
        }
        DxfLightweightPolylineRecordRole::ExtrusionY => {
            Some(DxfLightweightPolylineValueRole::ExtrusionY)
        }
        DxfLightweightPolylineRecordRole::ExtrusionZ => {
            Some(DxfLightweightPolylineValueRole::ExtrusionZ)
        }
        DxfLightweightPolylineRecordRole::VertexCount | DxfLightweightPolylineRecordRole::Flags => {
            None
        }
    }
}

const fn integer_role(
    role: DxfLightweightPolylineRecordRole,
) -> Option<DxfLightweightPolylineIntegerRole> {
    match role {
        DxfLightweightPolylineRecordRole::VertexCount => {
            Some(DxfLightweightPolylineIntegerRole::VertexCount)
        }
        DxfLightweightPolylineRecordRole::Flags => Some(DxfLightweightPolylineIntegerRole::Flags),
        DxfLightweightPolylineRecordRole::OcsElevation
        | DxfLightweightPolylineRecordRole::Thickness
        | DxfLightweightPolylineRecordRole::ConstantWidth
        | DxfLightweightPolylineRecordRole::ExtrusionX
        | DxfLightweightPolylineRecordRole::ExtrusionY
        | DxfLightweightPolylineRecordRole::ExtrusionZ => None,
    }
}

fn state_for_count(count: u64) -> Result<DxfLightweightPolylineRecordCardState, DxfError> {
    match count {
        0 => Ok(DxfLightweightPolylineRecordCardState::Absent),
        1 => Ok(DxfLightweightPolylineRecordCardState::Unique),
        occurrence_count => Ok(DxfLightweightPolylineRecordCardState::Multiple {
            occurrence_count: u32::try_from(occurrence_count)
                .map_err(|_| invalid_internal_data())?,
        }),
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

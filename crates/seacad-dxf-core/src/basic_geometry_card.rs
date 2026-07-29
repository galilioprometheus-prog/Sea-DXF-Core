//! Per-role cardinality cards over basic-geometry component evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBasicGeometryComponent, DxfBasicGeometryComponentRole,
    DxfBasicGeometryDirectory, DxfBasicGeometryKind, DxfBasicGeometryRecordEntry,
    DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation, DxfRawDocumentView,
    DxfSourceId,
};

const POINT_ROLES: [DxfBasicGeometryComponentRole; 6] = [
    DxfBasicGeometryComponentRole::WcsLocationOrStartX,
    DxfBasicGeometryComponentRole::WcsLocationOrStartY,
    DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
    DxfBasicGeometryComponentRole::ExtrusionX,
    DxfBasicGeometryComponentRole::ExtrusionY,
    DxfBasicGeometryComponentRole::ExtrusionZ,
];

const LINE_ROLES: [DxfBasicGeometryComponentRole; 9] = [
    DxfBasicGeometryComponentRole::WcsLocationOrStartX,
    DxfBasicGeometryComponentRole::WcsLocationOrStartY,
    DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
    DxfBasicGeometryComponentRole::WcsEndpointX,
    DxfBasicGeometryComponentRole::WcsEndpointY,
    DxfBasicGeometryComponentRole::WcsEndpointZ,
    DxfBasicGeometryComponentRole::ExtrusionX,
    DxfBasicGeometryComponentRole::ExtrusionY,
    DxfBasicGeometryComponentRole::ExtrusionZ,
];

/// Cardinality of one documented component role within one raw entity record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBasicGeometryComponentCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// Half-open range of member ordinals owned by one component card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBasicGeometryCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfBasicGeometryCardMemberRange {
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

/// Reference from a cardinality card back to one M8.1a component occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBasicGeometryCardMember {
    component_ordinal: u32,
}

impl DxfBasicGeometryCardMember {
    #[must_use]
    pub const fn component_ordinal(self) -> u64 {
        self.component_ordinal as u64
    }
}

/// One fixed role card for an exact POINT or LINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBasicGeometryComponentCard {
    ordinal: u32,
    record: DxfBasicGeometryRecordEntry,
    role: DxfBasicGeometryComponentRole,
    member_range: DxfBasicGeometryCardMemberRange,
    state: DxfBasicGeometryComponentCardState,
}

impl DxfBasicGeometryComponentCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfBasicGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfBasicGeometryComponentRole {
        self.role
    }

    #[must_use]
    pub const fn member_range(self) -> DxfBasicGeometryCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfBasicGeometryComponentCardState {
        self.state
    }
}

/// Immutable per-role cardinality index over M8.1a occurrence evidence.
///
/// Every POINT receives six cards and every LINE receives nine cards in a
/// stable role order. Members point back into the retained evidence directory;
/// no component is copied, selected, defaulted, or transformed.
#[derive(Debug)]
pub struct DxfBasicGeometryCardDirectory {
    source_id: DxfSourceId,
    evidence: DxfBasicGeometryDirectory,
    cards: Box<[DxfBasicGeometryComponentCard]>,
    members: Box<[DxfBasicGeometryCardMember]>,
}

impl DxfBasicGeometryCardDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.basic_geometry_directory(cancellation)?;
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
            let roles = roles_for_kind(record.kind());
            let components = evidence
                .components_for_raw_record(record.record().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let component_base = record.component_range().start();

            for role in roles.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                for (local_index, component) in components.iter().copied().enumerate() {
                    if component.role() != role {
                        continue;
                    }
                    let component_ordinal = component_base
                        .checked_add(local_index as u64)
                        .ok_or_else(invalid_internal_data)?;
                    members.try_reserve(1).map_err(|_| out_of_memory())?;
                    members.push(DxfBasicGeometryCardMember {
                        component_ordinal: u32::try_from(component_ordinal)
                            .map_err(|_| invalid_internal_data())?,
                    });
                }
                let member_end = compact_len(members.len())?;
                let member_range = DxfBasicGeometryCardMemberRange::new(member_start, member_end)?;
                let state = match member_range.len() {
                    0 => DxfBasicGeometryComponentCardState::Absent,
                    1 => DxfBasicGeometryComponentCardState::Unique,
                    occurrence_count => DxfBasicGeometryComponentCardState::Multiple {
                        occurrence_count: u32::try_from(occurrence_count)
                            .map_err(|_| invalid_internal_data())?,
                    },
                };
                cards.try_reserve(1).map_err(|_| out_of_memory())?;
                cards.push(DxfBasicGeometryComponentCard {
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
    pub const fn evidence_directory(&self) -> &DxfBasicGeometryDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfBasicGeometryComponentCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfBasicGeometryCardMember] {
        &self.members
    }

    #[must_use]
    pub fn card(&self, ordinal: u64) -> Option<DxfBasicGeometryComponentCard> {
        let index = usize::try_from(ordinal).ok()?;
        self.cards.get(index).copied()
    }

    #[must_use]
    pub fn cards_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBasicGeometryComponentCard]> {
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
        role: DxfBasicGeometryComponentRole,
    ) -> Option<DxfBasicGeometryComponentCard> {
        self.cards_for_raw_record(raw_record_ordinal)?
            .iter()
            .copied()
            .find(|card| card.role() == role)
    }

    #[must_use]
    pub fn members_for_card(&self, card_ordinal: u64) -> Option<&[DxfBasicGeometryCardMember]> {
        let card = self.card(card_ordinal)?;
        let start = usize::try_from(card.member_range().start()).ok()?;
        let end = usize::try_from(card.member_range().end()).ok()?;
        self.members.get(start..end)
    }

    #[must_use]
    pub fn component_for_member(
        &self,
        member: DxfBasicGeometryCardMember,
    ) -> Option<DxfBasicGeometryComponent> {
        let index = usize::try_from(member.component_ordinal()).ok()?;
        self.evidence.components().get(index).copied()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn basic_geometry_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometryCardDirectory, DxfError> {
        DxfBasicGeometryCardDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn basic_geometry_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometryCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).basic_geometry_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn basic_geometry_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometryCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).basic_geometry_card_directory(cancellation)
    }
}

const fn roles_for_kind(kind: DxfBasicGeometryKind) -> &'static [DxfBasicGeometryComponentRole] {
    match kind {
        DxfBasicGeometryKind::Point => &POINT_ROLES,
        DxfBasicGeometryKind::Line => &LINE_ROLES,
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

//! Conservative LWPOLYLINE vertex grouping and per-role cardinality.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfLightweightPolylineDirectory, DxfLightweightPolylineIntegerDirectory,
    DxfLightweightPolylineIntegerRole, DxfLightweightPolylineIntegerValue,
    DxfLightweightPolylineValue, DxfLightweightPolylineValueRole, DxfRawDocumentView, DxfRawRecord,
    DxfSourceId,
};

const ROLE_COUNT: usize = 6;
const ROLES: [DxfLightweightPolylineVertexRole; ROLE_COUNT] = [
    DxfLightweightPolylineVertexRole::OcsX,
    DxfLightweightPolylineVertexRole::OcsY,
    DxfLightweightPolylineVertexRole::StartWidth,
    DxfLightweightPolylineVertexRole::EndWidth,
    DxfLightweightPolylineVertexRole::Bulge,
    DxfLightweightPolylineVertexRole::Identifier,
];

/// One vertex-scoped role in LWPOLYLINE source order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineVertexRole {
    OcsX,
    OcsY,
    StartWidth,
    EndWidth,
    Bulge,
    Identifier,
}

/// Compact reference back to one M9.1a or M9.1b value occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineVertexMember {
    role: DxfLightweightPolylineVertexRole,
    group_occurrence: u32,
}

impl DxfLightweightPolylineVertexMember {
    #[must_use]
    pub const fn role(self) -> DxfLightweightPolylineVertexRole {
        self.role
    }

    #[must_use]
    pub const fn group_occurrence(self) -> u64 {
        self.group_occurrence as u64
    }
}

/// Occurrence count for one vertex role; lexical validity stays independent.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineVertexCardState {
    Absent,
    Unique,
    Multiple { occurrence_count: u32 },
}

/// One fixed role card for one conservatively grouped vertex.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineVertexCard {
    vertex_ordinal: u32,
    role: DxfLightweightPolylineVertexRole,
    state: DxfLightweightPolylineVertexCardState,
    member_start: u32,
    member_end: u32,
}

impl DxfLightweightPolylineVertexCard {
    #[must_use]
    pub const fn vertex_ordinal(self) -> u64 {
        self.vertex_ordinal as u64
    }

    #[must_use]
    pub const fn role(self) -> DxfLightweightPolylineVertexRole {
        self.role
    }

    #[must_use]
    pub const fn state(self) -> DxfLightweightPolylineVertexCardState {
        self.state
    }

    #[must_use]
    pub const fn member_count(self) -> u64 {
        (self.member_end - self.member_start) as u64
    }
}

/// One vertex started by an exact group-10 occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineVertexEntry {
    record: DxfRawRecord,
    record_vertex_ordinal: u32,
    anchor_group_occurrence: u32,
    cards: [DxfLightweightPolylineVertexCard; ROLE_COUNT],
}

impl DxfLightweightPolylineVertexEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn record_vertex_ordinal(self) -> u64 {
        self.record_vertex_ordinal as u64
    }

    #[must_use]
    pub const fn anchor_group_occurrence(self) -> u64 {
        self.anchor_group_occurrence as u64
    }

    #[must_use]
    pub const fn cards(&self) -> &[DxfLightweightPolylineVertexCard; ROLE_COUNT] {
        &self.cards
    }
}

/// Vertex and orphan slices for one recognized LWPOLYLINE raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineGroupedRecordEntry {
    record: DxfRawRecord,
    vertex_start: u32,
    vertex_end: u32,
    orphan_start: u32,
    orphan_end: u32,
}

impl DxfLightweightPolylineGroupedRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn vertex_count(self) -> u64 {
        (self.vertex_end - self.vertex_start) as u64
    }

    #[must_use]
    pub const fn orphan_count(self) -> u64 {
        (self.orphan_end - self.orphan_start) as u64
    }
}

/// Immutable LWPOLYLINE vertex grouping over retained M9.1a/b evidence.
#[derive(Debug)]
pub struct DxfLightweightPolylineVertexDirectory {
    floating: DxfLightweightPolylineDirectory,
    integers: DxfLightweightPolylineIntegerDirectory,
    records: Box<[DxfLightweightPolylineGroupedRecordEntry]>,
    vertices: Box<[DxfLightweightPolylineVertexEntry]>,
    members: Box<[DxfLightweightPolylineVertexMember]>,
    orphans: Box<[DxfLightweightPolylineVertexMember]>,
}

impl DxfLightweightPolylineVertexDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let floating = document.lightweight_polyline_directory(cancellation)?;
        let integers = document.lightweight_polyline_integer_directory(cancellation)?;
        if floating.source_id() != document.source_id()
            || integers.source_id() != document.source_id()
            || floating.records().len() != integers.records().len()
        {
            return Err(invalid_internal_data());
        }

        let mut records = Vec::new();
        let mut vertices = Vec::new();
        let mut members = Vec::new();
        let mut orphans = Vec::new();
        for floating_record in floating.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let record = floating_record.record();
            let integer_record = integers
                .record_for_raw_ordinal(record.ordinal())
                .ok_or_else(invalid_internal_data)?;
            if integer_record.record() != record {
                return Err(invalid_internal_data());
            }
            let vertex_start = compact_len(vertices.len())?;
            let orphan_start = compact_len(orphans.len())?;
            group_record(
                record,
                &floating,
                &integers,
                cancellation,
                &mut vertices,
                &mut members,
                &mut orphans,
            )?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfLightweightPolylineGroupedRecordEntry {
                record,
                vertex_start,
                vertex_end: compact_len(vertices.len())?,
                orphan_start,
                orphan_end: compact_len(orphans.len())?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            floating,
            integers,
            records: records.into_boxed_slice(),
            vertices: vertices.into_boxed_slice(),
            members: members.into_boxed_slice(),
            orphans: orphans.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.floating.source_id()
    }

    #[must_use]
    pub fn records(&self) -> &[DxfLightweightPolylineGroupedRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn vertices(&self) -> &[DxfLightweightPolylineVertexEntry] {
        &self.vertices
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfLightweightPolylineGroupedRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn vertices_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfLightweightPolylineVertexEntry]> {
        let record = self.record_for_raw_ordinal(raw_record_ordinal)?;
        slice(&self.vertices, record.vertex_start, record.vertex_end)
    }

    #[must_use]
    pub fn card_for_role(
        &self,
        vertex_ordinal: u64,
        role: DxfLightweightPolylineVertexRole,
    ) -> Option<DxfLightweightPolylineVertexCard> {
        let index = usize::try_from(vertex_ordinal).ok()?;
        self.vertices
            .get(index)?
            .cards()
            .get(role_index(role))
            .copied()
    }

    #[must_use]
    pub fn members_for_card(
        &self,
        card: DxfLightweightPolylineVertexCard,
    ) -> Option<&[DxfLightweightPolylineVertexMember]> {
        let known = self.card_for_role(card.vertex_ordinal(), card.role())?;
        (known == card)
            .then(|| slice(&self.members, card.member_start, card.member_end))
            .flatten()
    }

    #[must_use]
    pub fn orphan_members_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfLightweightPolylineVertexMember]> {
        let record = self.record_for_raw_ordinal(raw_record_ordinal)?;
        slice(&self.orphans, record.orphan_start, record.orphan_end)
    }

    #[must_use]
    pub fn floating_value_for_member(
        &self,
        member: DxfLightweightPolylineVertexMember,
    ) -> Option<DxfLightweightPolylineValue> {
        (member.role() != DxfLightweightPolylineVertexRole::Identifier)
            .then(|| self.floating.value_for_group(member.group_occurrence()))
            .flatten()
    }

    #[must_use]
    pub fn integer_value_for_member(
        &self,
        member: DxfLightweightPolylineVertexMember,
    ) -> Option<DxfLightweightPolylineIntegerValue> {
        (member.role() == DxfLightweightPolylineVertexRole::Identifier)
            .then(|| self.integers.value_for_group(member.group_occurrence()))
            .flatten()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn lightweight_polyline_vertex_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineVertexDirectory, DxfError> {
        DxfLightweightPolylineVertexDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_vertex_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineVertexDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_vertex_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_vertex_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineVertexDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_vertex_directory(cancellation)
    }
}

fn group_record(
    record: DxfRawRecord,
    floating: &DxfLightweightPolylineDirectory,
    integers: &DxfLightweightPolylineIntegerDirectory,
    cancellation: &DxfCancellationToken,
    vertices: &mut Vec<DxfLightweightPolylineVertexEntry>,
    members: &mut Vec<DxfLightweightPolylineVertexMember>,
    orphans: &mut Vec<DxfLightweightPolylineVertexMember>,
) -> Result<(), DxfError> {
    let mut grouped: [Vec<DxfLightweightPolylineVertexMember>; ROLE_COUNT] =
        std::array::from_fn(|_| Vec::new());
    let mut anchor = None;
    let mut record_vertex_ordinal = 0_u32;
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
        ensure_not_cancelled(cancellation)?;
        let Some((role, member)) = vertex_member(floating, integers, occurrence)? else {
            continue;
        };
        if role == DxfLightweightPolylineVertexRole::OcsX {
            if let Some(anchor_group_occurrence) = anchor {
                append_vertex(
                    record,
                    record_vertex_ordinal,
                    anchor_group_occurrence,
                    &mut grouped,
                    vertices,
                    members,
                )?;
                record_vertex_ordinal = record_vertex_ordinal
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
            }
            anchor = Some(member.group_occurrence);
        }
        if anchor.is_some() {
            let role_index = role_index(role);
            grouped[role_index]
                .try_reserve(1)
                .map_err(|_| out_of_memory())?;
            grouped[role_index].push(member);
        } else {
            orphans.try_reserve(1).map_err(|_| out_of_memory())?;
            orphans.push(member);
        }
    }
    if let Some(anchor_group_occurrence) = anchor {
        append_vertex(
            record,
            record_vertex_ordinal,
            anchor_group_occurrence,
            &mut grouped,
            vertices,
            members,
        )?;
    }
    Ok(())
}

fn append_vertex(
    record: DxfRawRecord,
    record_vertex_ordinal: u32,
    anchor_group_occurrence: u32,
    grouped: &mut [Vec<DxfLightweightPolylineVertexMember>; ROLE_COUNT],
    vertices: &mut Vec<DxfLightweightPolylineVertexEntry>,
    members: &mut Vec<DxfLightweightPolylineVertexMember>,
) -> Result<(), DxfError> {
    let vertex_ordinal = compact_len(vertices.len())?;
    let mut cards = Vec::new();
    cards
        .try_reserve_exact(ROLE_COUNT)
        .map_err(|_| out_of_memory())?;
    for (role, role_members) in ROLES.into_iter().zip(grouped.iter_mut()) {
        let member_start = compact_len(members.len())?;
        members
            .try_reserve(role_members.len())
            .map_err(|_| out_of_memory())?;
        members.extend_from_slice(role_members);
        let occurrence_count = compact_len(role_members.len())?;
        role_members.clear();
        let state = match occurrence_count {
            0 => DxfLightweightPolylineVertexCardState::Absent,
            1 => DxfLightweightPolylineVertexCardState::Unique,
            occurrence_count => {
                DxfLightweightPolylineVertexCardState::Multiple { occurrence_count }
            }
        };
        cards.push(DxfLightweightPolylineVertexCard {
            vertex_ordinal,
            role,
            state,
            member_start,
            member_end: compact_len(members.len())?,
        });
    }
    let cards = cards.try_into().map_err(|_| invalid_internal_data())?;
    vertices.try_reserve(1).map_err(|_| out_of_memory())?;
    vertices.push(DxfLightweightPolylineVertexEntry {
        record,
        record_vertex_ordinal,
        anchor_group_occurrence,
        cards,
    });
    Ok(())
}

fn vertex_member(
    floating: &DxfLightweightPolylineDirectory,
    integers: &DxfLightweightPolylineIntegerDirectory,
    occurrence: u64,
) -> Result<
    Option<(
        DxfLightweightPolylineVertexRole,
        DxfLightweightPolylineVertexMember,
    )>,
    DxfError,
> {
    if let Some(value) = floating.value_for_group(occurrence) {
        let role = match value.role() {
            DxfLightweightPolylineValueRole::OcsVertexX => DxfLightweightPolylineVertexRole::OcsX,
            DxfLightweightPolylineValueRole::OcsVertexY => DxfLightweightPolylineVertexRole::OcsY,
            DxfLightweightPolylineValueRole::StartWidth => {
                DxfLightweightPolylineVertexRole::StartWidth
            }
            DxfLightweightPolylineValueRole::EndWidth => DxfLightweightPolylineVertexRole::EndWidth,
            DxfLightweightPolylineValueRole::Bulge => DxfLightweightPolylineVertexRole::Bulge,
            DxfLightweightPolylineValueRole::OcsElevation
            | DxfLightweightPolylineValueRole::Thickness
            | DxfLightweightPolylineValueRole::ConstantWidth
            | DxfLightweightPolylineValueRole::ExtrusionX
            | DxfLightweightPolylineValueRole::ExtrusionY
            | DxfLightweightPolylineValueRole::ExtrusionZ => return Ok(None),
        };
        return Ok(Some((role, member(role, occurrence)?)));
    }
    let Some(value) = integers.value_for_group(occurrence) else {
        return Ok(None);
    };
    if value.role() != DxfLightweightPolylineIntegerRole::VertexIdentifier {
        return Ok(None);
    }
    Ok(Some((
        DxfLightweightPolylineVertexRole::Identifier,
        member(DxfLightweightPolylineVertexRole::Identifier, occurrence)?,
    )))
}

fn member(
    role: DxfLightweightPolylineVertexRole,
    occurrence: u64,
) -> Result<DxfLightweightPolylineVertexMember, DxfError> {
    Ok(DxfLightweightPolylineVertexMember {
        role,
        group_occurrence: u32::try_from(occurrence).map_err(|_| invalid_internal_data())?,
    })
}

const fn role_index(role: DxfLightweightPolylineVertexRole) -> usize {
    match role {
        DxfLightweightPolylineVertexRole::OcsX => 0,
        DxfLightweightPolylineVertexRole::OcsY => 1,
        DxfLightweightPolylineVertexRole::StartWidth => 2,
        DxfLightweightPolylineVertexRole::EndWidth => 3,
        DxfLightweightPolylineVertexRole::Bulge => 4,
        DxfLightweightPolylineVertexRole::Identifier => 5,
    }
}

fn slice<T>(values: &[T], start: u32, end: u32) -> Option<&[T]> {
    let start = usize::try_from(start).ok()?;
    let end = usize::try_from(end).ok()?;
    values.get(start..end)
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

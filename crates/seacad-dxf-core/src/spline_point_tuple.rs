//! Order-independent SPLINE control-point and fit-point tuple evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId, DxfSplineCardDirectory, DxfSplineCardMember,
    DxfSplineRecordEntry, DxfSplineValueRole,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplinePointKind {
    ControlPoint,
    FitPoint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplinePointComponents {
    bits: u8,
}

impl DxfSplinePointComponents {
    const X: u8 = 1;
    const Y: u8 = 2;
    const Z: u8 = 4;
    const COMPLETE: u8 = Self::X | Self::Y | Self::Z;

    const fn from_presence(x: bool, y: bool, z: bool) -> Self {
        Self {
            bits: ((x as u8) * Self::X) | ((y as u8) * Self::Y) | ((z as u8) * Self::Z),
        }
    }

    #[must_use]
    pub const fn has_x(self) -> bool {
        self.bits & Self::X != 0
    }

    #[must_use]
    pub const fn has_y(self) -> bool {
        self.bits & Self::Y != 0
    }

    #[must_use]
    pub const fn has_z(self) -> bool {
        self.bits & Self::Z != 0
    }

    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.bits == Self::COMPLETE
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplinePointTupleState {
    Complete,
    Partial(DxfSplinePointComponents),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplinePointComponentCounts {
    x: u32,
    y: u32,
    z: u32,
}

impl DxfSplinePointComponentCounts {
    #[must_use]
    pub const fn x(self) -> u64 {
        self.x as u64
    }

    #[must_use]
    pub const fn y(self) -> u64 {
        self.y as u64
    }

    #[must_use]
    pub const fn z(self) -> u64 {
        self.z as u64
    }

    #[must_use]
    pub const fn maximum(self) -> u64 {
        let maximum = if self.x > self.y { self.x } else { self.y };
        if maximum > self.z {
            maximum as u64
        } else {
            self.z as u64
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplinePointTupleRange {
    start: u32,
    end: u32,
}

impl DxfSplinePointTupleRange {
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
pub struct DxfSplinePointTuple {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    kind: DxfSplinePointKind,
    point_index: u32,
    x_member: Option<DxfSplineCardMember>,
    y_member: Option<DxfSplineCardMember>,
    z_member: Option<DxfSplineCardMember>,
}

impl DxfSplinePointTuple {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfSplinePointKind {
        self.kind
    }

    #[must_use]
    pub const fn point_index(self) -> u64 {
        self.point_index as u64
    }

    #[must_use]
    pub const fn x_member(self) -> Option<DxfSplineCardMember> {
        self.x_member
    }

    #[must_use]
    pub const fn y_member(self) -> Option<DxfSplineCardMember> {
        self.y_member
    }

    #[must_use]
    pub const fn z_member(self) -> Option<DxfSplineCardMember> {
        self.z_member
    }

    #[must_use]
    pub const fn components(self) -> DxfSplinePointComponents {
        DxfSplinePointComponents::from_presence(
            self.x_member.is_some(),
            self.y_member.is_some(),
            self.z_member.is_some(),
        )
    }

    #[must_use]
    pub const fn state(self) -> DxfSplinePointTupleState {
        let components = self.components();
        if components.is_complete() {
            DxfSplinePointTupleState::Complete
        } else {
            DxfSplinePointTupleState::Partial(components)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplinePointTupleEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    kind: DxfSplinePointKind,
    component_counts: DxfSplinePointComponentCounts,
    tuple_range: DxfSplinePointTupleRange,
}

impl DxfSplinePointTupleEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfSplinePointKind {
        self.kind
    }

    #[must_use]
    pub const fn component_counts(self) -> DxfSplinePointComponentCounts {
        self.component_counts
    }

    #[must_use]
    pub const fn tuple_range(self) -> DxfSplinePointTupleRange {
        self.tuple_range
    }
}

/// Two stable point-sequence entries per SPLINE record, backed by exact cards.
#[derive(Debug)]
pub struct DxfSplinePointTupleDirectory {
    source_id: DxfSourceId,
    cards: DxfSplineCardDirectory,
    entries: Box<[DxfSplinePointTupleEntry]>,
    tuples: Box<[DxfSplinePointTuple]>,
}

impl DxfSplinePointTupleDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.spline_card_directory(cancellation)?;
        if cards.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: cards.source_id(),
            });
        }
        let mut entries = Vec::new();
        let mut tuples = Vec::new();
        for record in cards.evidence_directory().records().iter().copied() {
            for kind in [
                DxfSplinePointKind::ControlPoint,
                DxfSplinePointKind::FitPoint,
            ] {
                ensure_not_cancelled(cancellation)?;
                append_entry(
                    &cards,
                    record,
                    kind,
                    cancellation,
                    &mut entries,
                    &mut tuples,
                )?;
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            entries: entries.into_boxed_slice(),
            tuples: tuples.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfSplineCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfSplinePointTupleEntry] {
        &self.entries
    }

    #[must_use]
    pub fn tuples(&self) -> &[DxfSplinePointTuple] {
        &self.tuples
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfSplinePointTupleEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn tuple(&self, ordinal: u64) -> Option<DxfSplinePointTuple> {
        self.tuples.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw_ordinal: u64) -> Option<&[DxfSplinePointTupleEntry]> {
        self.cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.record().record().ordinal() < raw_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.record().record().ordinal() <= raw_ordinal);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn entry_for_kind(
        &self,
        raw_ordinal: u64,
        kind: DxfSplinePointKind,
    ) -> Option<DxfSplinePointTupleEntry> {
        self.entries_for_raw_record(raw_ordinal)?
            .iter()
            .copied()
            .find(|entry| entry.kind() == kind)
    }

    #[must_use]
    pub fn tuples_for_entry(&self, ordinal: u64) -> Option<&[DxfSplinePointTuple]> {
        let entry = self.entry(ordinal)?;
        let start = usize::try_from(entry.tuple_range().start()).ok()?;
        let end = usize::try_from(entry.tuple_range().end()).ok()?;
        self.tuples.get(start..end)
    }

    #[must_use]
    pub fn tuples_for_raw_record(
        &self,
        raw_ordinal: u64,
        kind: DxfSplinePointKind,
    ) -> Option<&[DxfSplinePointTuple]> {
        let entry = self.entry_for_kind(raw_ordinal, kind)?;
        self.tuples_for_entry(entry.ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_point_tuple_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplinePointTupleDirectory, DxfError> {
        DxfSplinePointTupleDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_point_tuple_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplinePointTupleDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_point_tuple_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_point_tuple_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplinePointTupleDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_point_tuple_directory(cancellation)
    }
}

fn append_entry(
    cards: &DxfSplineCardDirectory,
    record: DxfSplineRecordEntry,
    kind: DxfSplinePointKind,
    cancellation: &DxfCancellationToken,
    entries: &mut Vec<DxfSplinePointTupleEntry>,
    tuples: &mut Vec<DxfSplinePointTuple>,
) -> Result<(), DxfError> {
    let (x_role, y_role, z_role) = roles(kind);
    let x_members = members(cards, record, x_role)?;
    let y_members = members(cards, record, y_role)?;
    let z_members = members(cards, record, z_role)?;
    let component_counts = DxfSplinePointComponentCounts {
        x: compact_len(x_members.len())?,
        y: compact_len(y_members.len())?,
        z: compact_len(z_members.len())?,
    };
    let tuple_count = component_counts.maximum();
    let start = compact_len(tuples.len())?;
    tuples
        .try_reserve(usize::try_from(tuple_count).map_err(|_| invalid_internal_data())?)
        .map_err(|_| out_of_memory())?;
    for point_index in 0..tuple_count {
        ensure_not_cancelled(cancellation)?;
        let index = usize::try_from(point_index).map_err(|_| invalid_internal_data())?;
        tuples.push(DxfSplinePointTuple {
            ordinal: compact_len(tuples.len())?,
            record,
            kind,
            point_index: u32::try_from(point_index).map_err(|_| invalid_internal_data())?,
            x_member: x_members.get(index).copied(),
            y_member: y_members.get(index).copied(),
            z_member: z_members.get(index).copied(),
        });
    }
    let end = compact_len(tuples.len())?;
    entries.try_reserve(1).map_err(|_| out_of_memory())?;
    entries.push(DxfSplinePointTupleEntry {
        ordinal: compact_len(entries.len())?,
        record,
        kind,
        component_counts,
        tuple_range: DxfSplinePointTupleRange::new(start, end)?,
    });
    Ok(())
}

fn members(
    cards: &DxfSplineCardDirectory,
    record: DxfSplineRecordEntry,
    role: DxfSplineValueRole,
) -> Result<&[DxfSplineCardMember], DxfError> {
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)
}

const fn roles(
    kind: DxfSplinePointKind,
) -> (DxfSplineValueRole, DxfSplineValueRole, DxfSplineValueRole) {
    match kind {
        DxfSplinePointKind::ControlPoint => (
            DxfSplineValueRole::ControlPointX,
            DxfSplineValueRole::ControlPointY,
            DxfSplineValueRole::ControlPointZ,
        ),
        DxfSplinePointKind::FitPoint => (
            DxfSplineValueRole::FitPointX,
            DxfSplineValueRole::FitPointY,
            DxfSplineValueRole::FitPointZ,
        ),
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

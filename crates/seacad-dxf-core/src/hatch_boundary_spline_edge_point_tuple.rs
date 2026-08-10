//! Exact ordered point tuples inside HATCH boundary Spline-edge sequences.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfFillMeshField,
    DxfHatchBoundarySplineEdgeSequenceDirectory, DxfHatchBoundarySplineEdgeSequenceEntry,
    DxfHatchBoundarySplineEdgeSequenceIssue, DxfRawDocumentView, DxfSourceId,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
#[repr(u8)]
pub enum DxfHatchBoundarySplineEdgePointKind {
    ControlPoint = 0,
    FitPoint = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgePointMemberRole {
    X,
    Y,
    Weight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointGrouping {
    control_point_count: u32,
    control_orphan_count: u32,
    fit_point_count: u32,
    fit_orphan_count: u32,
}

impl DxfHatchBoundarySplineEdgePointGrouping {
    #[must_use]
    pub const fn control_point_count(self) -> u64 {
        self.control_point_count as u64
    }

    #[must_use]
    pub const fn control_orphan_count(self) -> u64 {
        self.control_orphan_count as u64
    }

    #[must_use]
    pub const fn fit_point_count(self) -> u64 {
        self.fit_point_count as u64
    }

    #[must_use]
    pub const fn fit_orphan_count(self) -> u64 {
        self.fit_orphan_count as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgePointTupleIssue {
    SequenceUnavailable(DxfHatchBoundarySplineEdgeSequenceIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointTupleMember {
    role: DxfHatchBoundarySplineEdgePointMemberRole,
    field: DxfFillMeshField,
}

impl DxfHatchBoundarySplineEdgePointTupleMember {
    #[must_use]
    pub const fn role(self) -> DxfHatchBoundarySplineEdgePointMemberRole {
        self.role
    }

    #[must_use]
    pub const fn field(self) -> DxfFillMeshField {
        self.field
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointTuple {
    ordinal: u32,
    edge_ordinal: u32,
    kind: DxfHatchBoundarySplineEdgePointKind,
    point_index: u32,
    x_field: DxfFillMeshField,
    member_start: u32,
    member_end: u32,
    y_count: u32,
    weight_count: u32,
}

impl DxfHatchBoundarySplineEdgePointTuple {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn edge_ordinal(self) -> u64 {
        self.edge_ordinal as u64
    }

    #[must_use]
    pub const fn kind(self) -> DxfHatchBoundarySplineEdgePointKind {
        self.kind
    }

    #[must_use]
    pub const fn point_index(self) -> u64 {
        self.point_index as u64
    }

    #[must_use]
    pub const fn x_field(self) -> DxfFillMeshField {
        self.x_field
    }

    #[must_use]
    pub const fn member_count(self) -> u64 {
        (self.member_end - self.member_start) as u64
    }

    #[must_use]
    pub const fn y_count(self) -> u64 {
        self.y_count as u64
    }

    #[must_use]
    pub const fn weight_count(self) -> u64 {
        self.weight_count as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointOrphan {
    ordinal: u32,
    edge_ordinal: u32,
    kind: DxfHatchBoundarySplineEdgePointKind,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
    field: DxfFillMeshField,
}

impl DxfHatchBoundarySplineEdgePointOrphan {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn edge_ordinal(self) -> u64 {
        self.edge_ordinal as u64
    }

    #[must_use]
    pub const fn kind(self) -> DxfHatchBoundarySplineEdgePointKind {
        self.kind
    }

    #[must_use]
    pub const fn role(self) -> DxfHatchBoundarySplineEdgePointMemberRole {
        self.role
    }

    #[must_use]
    pub const fn field(self) -> DxfFillMeshField {
        self.field
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgePointTupleEntry {
    ordinal: u32,
    sequence: DxfHatchBoundarySplineEdgeSequenceEntry,
    grouping:
        Result<DxfHatchBoundarySplineEdgePointGrouping, DxfHatchBoundarySplineEdgePointTupleIssue>,
}

impl DxfHatchBoundarySplineEdgePointTupleEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn sequence_entry(self) -> DxfHatchBoundarySplineEdgeSequenceEntry {
        self.sequence
    }

    pub const fn grouping(
        self,
    ) -> Result<DxfHatchBoundarySplineEdgePointGrouping, DxfHatchBoundarySplineEdgePointTupleIssue>
    {
        self.grouping
    }
}

/// Point tuples and pre-anchor orphans for every Spline sequence entry.
#[derive(Debug)]
pub struct DxfHatchBoundarySplineEdgePointTupleDirectory {
    source_id: DxfSourceId,
    sequences: DxfHatchBoundarySplineEdgeSequenceDirectory,
    entries: Box<[DxfHatchBoundarySplineEdgePointTupleEntry]>,
    tuples: Box<[DxfHatchBoundarySplineEdgePointTuple]>,
    members: Box<[DxfHatchBoundarySplineEdgePointTupleMember]>,
    orphans: Box<[DxfHatchBoundarySplineEdgePointOrphan]>,
}

impl DxfHatchBoundarySplineEdgePointTupleDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let sequences = document.hatch_boundary_spline_edge_sequence_directory(cancellation)?;
        ensure_source(document.source_id(), sequences.source_id())?;
        let mut entries = Vec::new();
        let mut tuples = Vec::new();
        let mut members = Vec::new();
        let mut orphans = Vec::new();
        for sequence in sequences.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let grouping = match sequence.sequence() {
                Err(issue) => {
                    Err(DxfHatchBoundarySplineEdgePointTupleIssue::SequenceUnavailable(issue))
                }
                Ok(_) => Ok(group_sequence(
                    &sequences,
                    sequence,
                    cancellation,
                    &mut tuples,
                    &mut members,
                    &mut orphans,
                )?),
            };
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfHatchBoundarySplineEdgePointTupleEntry {
                ordinal: compact_len(entries.len())?,
                sequence,
                grouping,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            sequences,
            entries: entries.into_boxed_slice(),
            tuples: tuples.into_boxed_slice(),
            members: members.into_boxed_slice(),
            orphans: orphans.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn sequence_directory(&self) -> &DxfHatchBoundarySplineEdgeSequenceDirectory {
        &self.sequences
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundarySplineEdgePointTupleEntry] {
        &self.entries
    }

    #[must_use]
    pub fn tuples(&self) -> &[DxfHatchBoundarySplineEdgePointTuple] {
        &self.tuples
    }

    #[must_use]
    pub fn members(&self) -> &[DxfHatchBoundarySplineEdgePointTupleMember] {
        &self.members
    }

    #[must_use]
    pub fn orphans(&self) -> &[DxfHatchBoundarySplineEdgePointOrphan] {
        &self.orphans
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgePointTupleEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn tuple(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgePointTuple> {
        self.tuples.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn orphan(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgePointOrphan> {
        self.orphans.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundarySplineEdgePointTupleEntry> {
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| {
                edge_ordinal_for_sequence(entry.sequence_entry())
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn tuples_for_edge(
        &self,
        edge_ordinal: u64,
        kind: DxfHatchBoundarySplineEdgePointKind,
    ) -> Option<&[DxfHatchBoundarySplineEdgePointTuple]> {
        self.entry_for_edge(edge_ordinal)?.grouping().ok()?;
        let key = (edge_ordinal, kind);
        let start = self
            .tuples
            .partition_point(|tuple| (tuple.edge_ordinal(), tuple.kind()) < key);
        let end = self
            .tuples
            .partition_point(|tuple| (tuple.edge_ordinal(), tuple.kind()) <= key);
        self.tuples.get(start..end)
    }

    #[must_use]
    pub fn orphans_for_edge(
        &self,
        edge_ordinal: u64,
        kind: DxfHatchBoundarySplineEdgePointKind,
    ) -> Option<&[DxfHatchBoundarySplineEdgePointOrphan]> {
        self.entry_for_edge(edge_ordinal)?.grouping().ok()?;
        let key = (edge_ordinal, kind);
        let start = self
            .orphans
            .partition_point(|orphan| (orphan.edge_ordinal(), orphan.kind()) < key);
        let end = self
            .orphans
            .partition_point(|orphan| (orphan.edge_ordinal(), orphan.kind()) <= key);
        self.orphans.get(start..end)
    }

    #[must_use]
    pub fn members_for_tuple(
        &self,
        tuple_ordinal: u64,
    ) -> Option<&[DxfHatchBoundarySplineEdgePointTupleMember]> {
        let tuple = self.tuple(tuple_ordinal)?;
        let start = usize::try_from(tuple.member_start).ok()?;
        let end = usize::try_from(tuple.member_end).ok()?;
        self.members.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_spline_edge_point_tuple_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointTupleDirectory, DxfError> {
        DxfHatchBoundarySplineEdgePointTupleDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_point_tuple_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointTupleDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_point_tuple_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_point_tuple_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgePointTupleDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_point_tuple_directory(cancellation)
    }
}

fn group_sequence(
    sequences: &DxfHatchBoundarySplineEdgeSequenceDirectory,
    sequence: DxfHatchBoundarySplineEdgeSequenceEntry,
    cancellation: &DxfCancellationToken,
    tuples: &mut Vec<DxfHatchBoundarySplineEdgePointTuple>,
    members: &mut Vec<DxfHatchBoundarySplineEdgePointTupleMember>,
    orphans: &mut Vec<DxfHatchBoundarySplineEdgePointOrphan>,
) -> Result<DxfHatchBoundarySplineEdgePointGrouping, DxfError> {
    let edge_ordinal = edge_ordinal_for_sequence(sequence);
    let control_tuple_start = compact_len(tuples.len())?;
    let control_orphan_start = compact_len(orphans.len())?;
    let control = sequences
        .control_point_fields_for_edge(edge_ordinal)
        .ok_or_else(invalid_internal_data)?;
    append_points(
        edge_ordinal,
        DxfHatchBoundarySplineEdgePointKind::ControlPoint,
        control,
        cancellation,
        tuples,
        members,
        orphans,
    )?;
    let control_point_count = compact_len(tuples.len())?
        .checked_sub(control_tuple_start)
        .ok_or_else(invalid_internal_data)?;
    let control_orphan_count = compact_len(orphans.len())?
        .checked_sub(control_orphan_start)
        .ok_or_else(invalid_internal_data)?;

    let fit_tuple_start = compact_len(tuples.len())?;
    let fit_orphan_start = compact_len(orphans.len())?;
    let fit = sequences
        .fit_point_fields_for_edge(edge_ordinal)
        .ok_or_else(invalid_internal_data)?;
    append_points(
        edge_ordinal,
        DxfHatchBoundarySplineEdgePointKind::FitPoint,
        fit,
        cancellation,
        tuples,
        members,
        orphans,
    )?;
    Ok(DxfHatchBoundarySplineEdgePointGrouping {
        control_point_count,
        control_orphan_count,
        fit_point_count: compact_len(tuples.len())?
            .checked_sub(fit_tuple_start)
            .ok_or_else(invalid_internal_data)?,
        fit_orphan_count: compact_len(orphans.len())?
            .checked_sub(fit_orphan_start)
            .ok_or_else(invalid_internal_data)?,
    })
}

struct OpenTuple {
    point_index: u32,
    x_field: DxfFillMeshField,
    member_start: u32,
    y_count: u32,
    weight_count: u32,
}

fn append_points(
    edge_ordinal: u64,
    kind: DxfHatchBoundarySplineEdgePointKind,
    fields: &[DxfFillMeshField],
    cancellation: &DxfCancellationToken,
    tuples: &mut Vec<DxfHatchBoundarySplineEdgePointTuple>,
    members: &mut Vec<DxfHatchBoundarySplineEdgePointTupleMember>,
    orphans: &mut Vec<DxfHatchBoundarySplineEdgePointOrphan>,
) -> Result<(), DxfError> {
    let mut current = None;
    let mut point_count = 0_u32;
    for field in fields.iter().copied() {
        ensure_not_cancelled(cancellation)?;
        let role = role_for_code(kind, field.group().group_code().value())
            .ok_or_else(invalid_internal_data)?;
        if role == DxfHatchBoundarySplineEdgePointMemberRole::X {
            flush_tuple(edge_ordinal, kind, current.take(), tuples, members)?;
            let member_start = compact_len(members.len())?;
            push_member(role, field, members)?;
            current = Some(OpenTuple {
                point_index: point_count,
                x_field: field,
                member_start,
                y_count: 0,
                weight_count: 0,
            });
            point_count = point_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
        } else if let Some(open) = current.as_mut() {
            match role {
                DxfHatchBoundarySplineEdgePointMemberRole::Y => {
                    open.y_count = open
                        .y_count
                        .checked_add(1)
                        .ok_or_else(invalid_internal_data)?;
                }
                DxfHatchBoundarySplineEdgePointMemberRole::Weight => {
                    open.weight_count = open
                        .weight_count
                        .checked_add(1)
                        .ok_or_else(invalid_internal_data)?;
                }
                DxfHatchBoundarySplineEdgePointMemberRole::X => {
                    return Err(invalid_internal_data());
                }
            }
            push_member(role, field, members)?;
        } else {
            orphans.try_reserve(1).map_err(|_| out_of_memory())?;
            orphans.push(DxfHatchBoundarySplineEdgePointOrphan {
                ordinal: compact_len(orphans.len())?,
                edge_ordinal: compact_u64(edge_ordinal)?,
                kind,
                role,
                field,
            });
        }
    }
    flush_tuple(edge_ordinal, kind, current, tuples, members)
}

fn push_member(
    role: DxfHatchBoundarySplineEdgePointMemberRole,
    field: DxfFillMeshField,
    members: &mut Vec<DxfHatchBoundarySplineEdgePointTupleMember>,
) -> Result<(), DxfError> {
    members.try_reserve(1).map_err(|_| out_of_memory())?;
    members.push(DxfHatchBoundarySplineEdgePointTupleMember { role, field });
    Ok(())
}

fn flush_tuple(
    edge_ordinal: u64,
    kind: DxfHatchBoundarySplineEdgePointKind,
    current: Option<OpenTuple>,
    tuples: &mut Vec<DxfHatchBoundarySplineEdgePointTuple>,
    members: &[DxfHatchBoundarySplineEdgePointTupleMember],
) -> Result<(), DxfError> {
    let Some(open) = current else {
        return Ok(());
    };
    tuples.try_reserve(1).map_err(|_| out_of_memory())?;
    tuples.push(DxfHatchBoundarySplineEdgePointTuple {
        ordinal: compact_len(tuples.len())?,
        edge_ordinal: compact_u64(edge_ordinal)?,
        kind,
        point_index: open.point_index,
        x_field: open.x_field,
        member_start: open.member_start,
        member_end: compact_len(members.len())?,
        y_count: open.y_count,
        weight_count: open.weight_count,
    });
    Ok(())
}

const fn role_for_code(
    kind: DxfHatchBoundarySplineEdgePointKind,
    code: i16,
) -> Option<DxfHatchBoundarySplineEdgePointMemberRole> {
    match (kind, code) {
        (DxfHatchBoundarySplineEdgePointKind::ControlPoint, 10)
        | (DxfHatchBoundarySplineEdgePointKind::FitPoint, 11) => {
            Some(DxfHatchBoundarySplineEdgePointMemberRole::X)
        }
        (DxfHatchBoundarySplineEdgePointKind::ControlPoint, 20)
        | (DxfHatchBoundarySplineEdgePointKind::FitPoint, 21) => {
            Some(DxfHatchBoundarySplineEdgePointMemberRole::Y)
        }
        (DxfHatchBoundarySplineEdgePointKind::ControlPoint, 42) => {
            Some(DxfHatchBoundarySplineEdgePointMemberRole::Weight)
        }
        _ => None,
    }
}

const fn edge_ordinal_for_sequence(sequence: DxfHatchBoundarySplineEdgeSequenceEntry) -> u64 {
    sequence
        .payload_partition_entry()
        .edge_type_entry()
        .edge()
        .ordinal()
}

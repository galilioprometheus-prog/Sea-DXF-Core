//! Source-ordered 3D point and vector tuples for generic entity XDATA.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplication, DxfEntityXDataOccurrenceKind, DxfEntityXDataTypedDirectory,
    DxfEntityXDataTypedEntry, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// Documented transformation role of one XDATA 3D tuple.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataPointKind {
    Point,
    WorldPosition,
    WorldDisplacement,
    WorldDirection,
}

#[derive(Clone, Copy)]
enum PointAxis {
    X,
    Y,
    Z,
}

/// Exact set of axes retained by one tuple candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataPointComponents {
    bits: u8,
}

impl DxfEntityXDataPointComponents {
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

/// Structural completeness of one source-ordered XDATA tuple candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataPointTupleState {
    Complete,
    Partial(DxfEntityXDataPointComponents),
}

/// Compact link from one tuple axis to its generic typed XDATA entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataPointMember {
    entry_ordinal: u32,
}

impl DxfEntityXDataPointMember {
    #[must_use]
    pub const fn entry_ordinal(self) -> u64 {
        self.entry_ordinal as u64
    }
}

/// One maximal, same-kind, source-ordered X/Y/Z tuple candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataPointTuple {
    ordinal: u32,
    kind: DxfEntityXDataPointKind,
    context: DxfEntityXDataOccurrenceKind,
    entity: DxfEntityRef,
    x: Option<DxfEntityXDataPointMember>,
    y: Option<DxfEntityXDataPointMember>,
    z: Option<DxfEntityXDataPointMember>,
}

impl DxfEntityXDataPointTuple {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn kind(self) -> DxfEntityXDataPointKind {
        self.kind
    }

    #[must_use]
    pub const fn context(self) -> DxfEntityXDataOccurrenceKind {
        self.context
    }

    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn x(self) -> Option<DxfEntityXDataPointMember> {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> Option<DxfEntityXDataPointMember> {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> Option<DxfEntityXDataPointMember> {
        self.z
    }

    #[must_use]
    pub const fn components(self) -> DxfEntityXDataPointComponents {
        DxfEntityXDataPointComponents::from_presence(
            self.x.is_some(),
            self.y.is_some(),
            self.z.is_some(),
        )
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataPointTupleState {
        let components = self.components();
        if components.is_complete() {
            DxfEntityXDataPointTupleState::Complete
        } else {
            DxfEntityXDataPointTupleState::Partial(components)
        }
    }

    const fn first_entry_ordinal(self) -> u64 {
        if let Some(entry) = self.x {
            entry.entry_ordinal()
        } else if let Some(entry) = self.y {
            entry.entry_ordinal()
        } else if let Some(entry) = self.z {
            entry.entry_ordinal()
        } else {
            u64::MAX
        }
    }

    const fn last_entry_ordinal(self) -> u64 {
        if let Some(entry) = self.z {
            entry.entry_ordinal()
        } else if let Some(entry) = self.y {
            entry.entry_ordinal()
        } else if let Some(entry) = self.x {
            entry.entry_ordinal()
        } else {
            u64::MAX
        }
    }

    fn contains(self, entry: DxfEntityXDataTypedEntry) -> bool {
        let ordinal = entry.ordinal();
        self.x
            .is_some_and(|member| member.entry_ordinal() == ordinal)
            || self
                .y
                .is_some_and(|member| member.entry_ordinal() == ordinal)
            || self
                .z
                .is_some_and(|member| member.entry_ordinal() == ordinal)
    }
}

/// Bounded tuple projection over the generic typed XDATA directory.
#[derive(Debug)]
pub struct DxfEntityXDataPointTupleDirectory {
    source_id: DxfSourceId,
    typed: DxfEntityXDataTypedDirectory,
    tuples: Box<[DxfEntityXDataPointTuple]>,
}

impl DxfEntityXDataPointTupleDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let typed = document.entity_xdata_typed_directory(cancellation)?;
        ensure_source(document.source_id(), typed.source_id())?;
        let mut tuples = Vec::new();
        tuples
            .try_reserve(typed.entries().len())
            .map_err(|_| out_of_memory())?;
        let mut pending = None;
        for entry in typed.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            append_entry(entry, &mut pending, &mut tuples)?;
        }
        flush(&mut pending, &mut tuples)?;
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            typed,
            tuples: tuples.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn typed_directory(&self) -> &DxfEntityXDataTypedDirectory {
        &self.typed
    }

    #[must_use]
    pub fn tuples(&self) -> &[DxfEntityXDataPointTuple] {
        &self.tuples
    }

    #[must_use]
    pub fn tuple(&self, ordinal: u64) -> Option<DxfEntityXDataPointTuple> {
        self.tuples.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn tuple_for_entry(
        &self,
        entry: DxfEntityXDataTypedEntry,
    ) -> Option<DxfEntityXDataPointTuple> {
        if self.typed.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        let index = self
            .tuples
            .partition_point(|tuple| tuple.last_entry_ordinal() < entry.ordinal());
        self.tuples
            .get(index)
            .copied()
            .filter(|tuple| tuple.contains(entry))
    }

    #[must_use]
    pub fn entry_for_member(
        &self,
        tuple: DxfEntityXDataPointTuple,
        member: DxfEntityXDataPointMember,
    ) -> Option<DxfEntityXDataTypedEntry> {
        if self.tuple(tuple.ordinal()) != Some(tuple)
            || ![tuple.x(), tuple.y(), tuple.z()].contains(&Some(member))
        {
            return None;
        }
        self.typed.entry(member.entry_ordinal())
    }

    pub fn tuples_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<&[DxfEntityXDataPointTuple], DxfError> {
        ensure_source(self.source_id, application.entity().source_id())?;
        if self
            .typed
            .xdata_directory()
            .application(application.ordinal())
            != Some(application)
        {
            return Err(invalid_internal_data());
        }
        let range = application.occurrence_range();
        let start = self
            .tuples
            .partition_point(|tuple| tuple.last_entry_ordinal() < range.start());
        let end = self
            .tuples
            .partition_point(|tuple| tuple.first_entry_ordinal() < range.end());
        self.tuples
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }

    pub fn tuples_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataPointTuple], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let ordinal = entity.record().ordinal();
        let start = self
            .tuples
            .partition_point(|tuple| tuple.entity().record().ordinal() < ordinal);
        let end = self
            .tuples
            .partition_point(|tuple| tuple.entity().record().ordinal() <= ordinal);
        self.tuples
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_point_tuple_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataPointTupleDirectory, DxfError> {
        DxfEntityXDataPointTupleDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_xdata_point_tuple_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataPointTupleDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_point_tuple_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_xdata_point_tuple_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataPointTupleDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_point_tuple_directory(cancellation)
    }
}

#[derive(Clone, Copy)]
struct PendingTuple {
    kind: DxfEntityXDataPointKind,
    context: DxfEntityXDataOccurrenceKind,
    entity: DxfEntityRef,
    x: Option<DxfEntityXDataPointMember>,
    y: Option<DxfEntityXDataPointMember>,
    z: Option<DxfEntityXDataPointMember>,
    last_group_occurrence: u64,
}

fn append_entry(
    entry: DxfEntityXDataTypedEntry,
    pending: &mut Option<PendingTuple>,
    tuples: &mut Vec<DxfEntityXDataPointTuple>,
) -> Result<(), DxfError> {
    let Some((kind, axis)) = component(entry) else {
        return flush(pending, tuples);
    };
    let context = entry.occurrence().kind();
    let entity = entry.occurrence().entity();
    let group_occurrence = entry.occurrence().group().occurrence();
    if pending.as_ref().is_some_and(|current| {
        current.kind != kind
            || current.context != context
            || current.entity != entity
            || current.last_group_occurrence.checked_add(1) != Some(group_occurrence)
    }) {
        flush(pending, tuples)?;
    }
    match axis {
        PointAxis::X => {
            flush(pending, tuples)?;
            *pending = Some(PendingTuple::new(kind, context, entity, axis, entry)?);
        }
        PointAxis::Y => {
            if pending.as_ref().is_some_and(|current| {
                current.x.is_some() && current.y.is_none() && current.z.is_none()
            }) {
                let current = pending.as_mut().ok_or_else(invalid_internal_data)?;
                current.y = Some(member(entry)?);
                current.last_group_occurrence = group_occurrence;
            } else {
                flush(pending, tuples)?;
                *pending = Some(PendingTuple::new(kind, context, entity, axis, entry)?);
            }
        }
        PointAxis::Z => {
            if pending.as_ref().is_some_and(|current| {
                current.z.is_none() && (current.x.is_some() || current.y.is_some())
            }) {
                let current = pending.as_mut().ok_or_else(invalid_internal_data)?;
                current.z = Some(member(entry)?);
                current.last_group_occurrence = group_occurrence;
            } else {
                flush(pending, tuples)?;
                *pending = Some(PendingTuple::new(kind, context, entity, axis, entry)?);
            }
            flush(pending, tuples)?;
        }
    }
    Ok(())
}

impl PendingTuple {
    fn new(
        kind: DxfEntityXDataPointKind,
        context: DxfEntityXDataOccurrenceKind,
        entity: DxfEntityRef,
        axis: PointAxis,
        entry: DxfEntityXDataTypedEntry,
    ) -> Result<Self, DxfError> {
        let member = member(entry)?;
        Ok(Self {
            kind,
            context,
            entity,
            x: if matches!(axis, PointAxis::X) {
                Some(member)
            } else {
                None
            },
            y: if matches!(axis, PointAxis::Y) {
                Some(member)
            } else {
                None
            },
            z: if matches!(axis, PointAxis::Z) {
                Some(member)
            } else {
                None
            },
            last_group_occurrence: entry.occurrence().group().occurrence(),
        })
    }
}

fn member(entry: DxfEntityXDataTypedEntry) -> Result<DxfEntityXDataPointMember, DxfError> {
    Ok(DxfEntityXDataPointMember {
        entry_ordinal: u32::try_from(entry.ordinal()).map_err(|_| invalid_internal_data())?,
    })
}

fn flush(
    pending: &mut Option<PendingTuple>,
    tuples: &mut Vec<DxfEntityXDataPointTuple>,
) -> Result<(), DxfError> {
    let Some(value) = pending.take() else {
        return Ok(());
    };
    tuples.push(DxfEntityXDataPointTuple {
        ordinal: compact_len(tuples.len())?,
        kind: value.kind,
        context: value.context,
        entity: value.entity,
        x: value.x,
        y: value.y,
        z: value.z,
    });
    Ok(())
}

const fn component(
    entry: DxfEntityXDataTypedEntry,
) -> Option<(DxfEntityXDataPointKind, PointAxis)> {
    let code = entry.occurrence().group().group_code().value();
    let axis = match code {
        1010..=1013 => PointAxis::X,
        1020..=1023 => PointAxis::Y,
        1030..=1033 => PointAxis::Z,
        _ => return None,
    };
    let suffix = code % 10;
    let kind = match suffix {
        0 => DxfEntityXDataPointKind::Point,
        1 => DxfEntityXDataPointKind::WorldPosition,
        2 => DxfEntityXDataPointKind::WorldDisplacement,
        3 => DxfEntityXDataPointKind::WorldDirection,
        _ => return None,
    };
    Some((kind, axis))
}

fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
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

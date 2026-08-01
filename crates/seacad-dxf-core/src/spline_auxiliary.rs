//! SPLINE weight cardinality and optional tangent/normal vector structure.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId, DxfSplineCardMember, DxfSplineCardState, DxfSplinePointKind,
    DxfSplinePointTupleDirectory, DxfSplineRecordEntry, DxfSplineValue, DxfSplineValueRole,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineWeightCountDisposition {
    Matched,
    Mismatched,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineWeightState {
    ImplicitUnit {
        control_point_count: u32,
    },
    Explicit {
        weight_count: u32,
        control_point_count: u32,
        disposition: DxfSplineWeightCountDisposition,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineWeightEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    card_ordinal: u32,
    state: DxfSplineWeightState,
}

impl DxfSplineWeightEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineWeightState {
        self.state
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineVectorKind {
    StartTangent,
    EndTangent,
    Normal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineVectorComponents {
    bits: u8,
}

impl DxfSplineVectorComponents {
    const X: u8 = 1;
    const Y: u8 = 2;
    const Z: u8 = 4;
    const COMPLETE: u8 = Self::X | Self::Y | Self::Z;

    pub(crate) const fn from_bits(bits: u8) -> Self {
        Self {
            bits: bits & Self::COMPLETE,
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

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineVectorComponentState {
    Absent,
    Unique(DxfSplineCardMember),
    Multiple { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineVectorState {
    Absent,
    Present {
        components: DxfSplineVectorComponents,
    },
    Ambiguous {
        duplicate_components: DxfSplineVectorComponents,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineVectorEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    kind: DxfSplineVectorKind,
    x: DxfSplineVectorComponentState,
    y: DxfSplineVectorComponentState,
    z: DxfSplineVectorComponentState,
}

impl DxfSplineVectorEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfSplineVectorKind {
        self.kind
    }

    #[must_use]
    pub const fn x(self) -> DxfSplineVectorComponentState {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> DxfSplineVectorComponentState {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> DxfSplineVectorComponentState {
        self.z
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineVectorState {
        let duplicate_components = component_mask(self.x, self.y, self.z, true);
        if !duplicate_components.is_empty() {
            return DxfSplineVectorState::Ambiguous {
                duplicate_components,
            };
        }
        let components = component_mask(self.x, self.y, self.z, false);
        if components.is_empty() {
            DxfSplineVectorState::Absent
        } else {
            DxfSplineVectorState::Present { components }
        }
    }
}

/// Weight sequence and three optional vectors per retained SPLINE record.
#[derive(Debug)]
pub struct DxfSplineAuxiliaryDirectory {
    source_id: DxfSourceId,
    points: DxfSplinePointTupleDirectory,
    weights: Box<[DxfSplineWeightEntry]>,
    vectors: Box<[DxfSplineVectorEntry]>,
}

impl DxfSplineAuxiliaryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let points = document.spline_point_tuple_directory(cancellation)?;
        if points.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: points.source_id(),
            });
        }
        let mut weights = Vec::new();
        let mut vectors = Vec::new();
        for record in points
            .card_directory()
            .evidence_directory()
            .records()
            .iter()
            .copied()
        {
            ensure_not_cancelled(cancellation)?;
            append_weight(&points, record, &mut weights)?;
            for kind in [
                DxfSplineVectorKind::StartTangent,
                DxfSplineVectorKind::EndTangent,
                DxfSplineVectorKind::Normal,
            ] {
                ensure_not_cancelled(cancellation)?;
                append_vector(&points, record, kind, &mut vectors)?;
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            points,
            weights: weights.into_boxed_slice(),
            vectors: vectors.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn point_directory(&self) -> &DxfSplinePointTupleDirectory {
        &self.points
    }

    #[must_use]
    pub fn weights(&self) -> &[DxfSplineWeightEntry] {
        &self.weights
    }

    #[must_use]
    pub fn vectors(&self) -> &[DxfSplineVectorEntry] {
        &self.vectors
    }

    #[must_use]
    pub fn weight(&self, ordinal: u64) -> Option<DxfSplineWeightEntry> {
        self.weights.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn vector(&self, ordinal: u64) -> Option<DxfSplineVectorEntry> {
        self.vectors.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn weight_for_raw_record(&self, raw_ordinal: u64) -> Option<DxfSplineWeightEntry> {
        self.weights
            .binary_search_by_key(&raw_ordinal, |entry| entry.record().record().ordinal())
            .ok()
            .and_then(|index| self.weights.get(index).copied())
    }

    #[must_use]
    pub fn vectors_for_raw_record(&self, raw_ordinal: u64) -> Option<&[DxfSplineVectorEntry]> {
        self.points
            .card_directory()
            .evidence_directory()
            .record_for_raw_ordinal(raw_ordinal)?;
        let start = self
            .vectors
            .partition_point(|entry| entry.record().record().ordinal() < raw_ordinal);
        let end = self
            .vectors
            .partition_point(|entry| entry.record().record().ordinal() <= raw_ordinal);
        self.vectors.get(start..end)
    }

    #[must_use]
    pub fn vector_for_kind(
        &self,
        raw_ordinal: u64,
        kind: DxfSplineVectorKind,
    ) -> Option<DxfSplineVectorEntry> {
        self.vectors_for_raw_record(raw_ordinal)?
            .iter()
            .copied()
            .find(|entry| entry.kind() == kind)
    }

    #[must_use]
    pub fn members_for_weight(&self, ordinal: u64) -> Option<&[DxfSplineCardMember]> {
        let entry = self.weight(ordinal)?;
        self.points
            .card_directory()
            .members_for_card(entry.card_ordinal as u64)
    }

    #[must_use]
    pub fn value_for_member(&self, member: DxfSplineCardMember) -> Option<DxfSplineValue> {
        self.points.card_directory().value_for_member(member)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_auxiliary_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAuxiliaryDirectory, DxfError> {
        DxfSplineAuxiliaryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_auxiliary_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAuxiliaryDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_auxiliary_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_auxiliary_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAuxiliaryDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_auxiliary_directory(cancellation)
    }
}

fn append_weight(
    points: &DxfSplinePointTupleDirectory,
    record: DxfSplineRecordEntry,
    weights: &mut Vec<DxfSplineWeightEntry>,
) -> Result<(), DxfError> {
    let cards = points.card_directory();
    let card = cards
        .card_for_role(record.record().ordinal(), DxfSplineValueRole::Weight)
        .ok_or_else(invalid_internal_data)?;
    let weight_count = compact_len(
        cards
            .members_for_card(card.ordinal())
            .ok_or_else(invalid_internal_data)?
            .len(),
    )?;
    let control_point_count = u32::try_from(
        points
            .entry_for_kind(record.record().ordinal(), DxfSplinePointKind::ControlPoint)
            .ok_or_else(invalid_internal_data)?
            .component_counts()
            .x(),
    )
    .map_err(|_| invalid_internal_data())?;
    let state = if weight_count == 0 {
        DxfSplineWeightState::ImplicitUnit {
            control_point_count,
        }
    } else {
        DxfSplineWeightState::Explicit {
            weight_count,
            control_point_count,
            disposition: if weight_count == control_point_count {
                DxfSplineWeightCountDisposition::Matched
            } else {
                DxfSplineWeightCountDisposition::Mismatched
            },
        }
    };
    weights.try_reserve(1).map_err(|_| out_of_memory())?;
    weights.push(DxfSplineWeightEntry {
        ordinal: compact_len(weights.len())?,
        record,
        card_ordinal: u32::try_from(card.ordinal()).map_err(|_| invalid_internal_data())?,
        state,
    });
    Ok(())
}

fn append_vector(
    points: &DxfSplinePointTupleDirectory,
    record: DxfSplineRecordEntry,
    kind: DxfSplineVectorKind,
    vectors: &mut Vec<DxfSplineVectorEntry>,
) -> Result<(), DxfError> {
    let (x_role, y_role, z_role) = vector_roles(kind);
    vectors.try_reserve(1).map_err(|_| out_of_memory())?;
    vectors.push(DxfSplineVectorEntry {
        ordinal: compact_len(vectors.len())?,
        record,
        kind,
        x: component_state(points, record, x_role)?,
        y: component_state(points, record, y_role)?,
        z: component_state(points, record, z_role)?,
    });
    Ok(())
}

fn component_state(
    points: &DxfSplinePointTupleDirectory,
    record: DxfSplineRecordEntry,
    role: DxfSplineValueRole,
) -> Result<DxfSplineVectorComponentState, DxfError> {
    let cards = points.card_directory();
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfSplineCardState::Absent => Ok(DxfSplineVectorComponentState::Absent),
        DxfSplineCardState::Multiple { occurrence_count } => {
            Ok(DxfSplineVectorComponentState::Multiple { occurrence_count })
        }
        DxfSplineCardState::Unique => {
            let [member] = cards
                .members_for_card(card.ordinal())
                .ok_or_else(invalid_internal_data)?
            else {
                return Err(invalid_internal_data());
            };
            Ok(DxfSplineVectorComponentState::Unique(*member))
        }
    }
}

const fn component_mask(
    x: DxfSplineVectorComponentState,
    y: DxfSplineVectorComponentState,
    z: DxfSplineVectorComponentState,
    duplicates: bool,
) -> DxfSplineVectorComponents {
    let x_set = matches!(x, DxfSplineVectorComponentState::Multiple { .. }) == duplicates
        && !matches!(x, DxfSplineVectorComponentState::Absent);
    let y_set = matches!(y, DxfSplineVectorComponentState::Multiple { .. }) == duplicates
        && !matches!(y, DxfSplineVectorComponentState::Absent);
    let z_set = matches!(z, DxfSplineVectorComponentState::Multiple { .. }) == duplicates
        && !matches!(z, DxfSplineVectorComponentState::Absent);
    DxfSplineVectorComponents::from_bits(
        ((x_set as u8) * DxfSplineVectorComponents::X)
            | ((y_set as u8) * DxfSplineVectorComponents::Y)
            | ((z_set as u8) * DxfSplineVectorComponents::Z),
    )
}

const fn vector_roles(
    kind: DxfSplineVectorKind,
) -> (DxfSplineValueRole, DxfSplineValueRole, DxfSplineValueRole) {
    match kind {
        DxfSplineVectorKind::StartTangent => (
            DxfSplineValueRole::StartTangentX,
            DxfSplineValueRole::StartTangentY,
            DxfSplineValueRole::StartTangentZ,
        ),
        DxfSplineVectorKind::EndTangent => (
            DxfSplineValueRole::EndTangentX,
            DxfSplineValueRole::EndTangentY,
            DxfSplineValueRole::EndTangentZ,
        ),
        DxfSplineVectorKind::Normal => (
            DxfSplineValueRole::NormalX,
            DxfSplineValueRole::NormalY,
            DxfSplineValueRole::NormalZ,
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

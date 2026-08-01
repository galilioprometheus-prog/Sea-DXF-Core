//! Typed WCS component semantics for HELIX vectors.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfHelixCardDirectory, DxfHelixCardState, DxfHelixNumber,
    DxfHelixNumericIssue, DxfHelixRecordEntry, DxfHelixValue, DxfHelixValueCard, DxfHelixValueRole,
    DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId,
};

const HELIX_NAMESPACE: &str = "entity.helix";

pub const DXF_HELIX_VECTOR_KINDS: [DxfHelixVectorKind; 3] = [
    DxfHelixVectorKind::AxisBase,
    DxfHelixVectorKind::StartPoint,
    DxfHelixVectorKind::AxisVector,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixVectorKind {
    AxisBase,
    StartPoint,
    AxisVector,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixCoordinateIssue {
    MultipleComponents { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
}

pub type DxfHelixCoordinateSemanticValue = DxfSemanticValue<DxfDouble, DxfHelixCoordinateIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixVectorSemantics {
    ordinal: u32,
    record: DxfHelixRecordEntry,
    kind: DxfHelixVectorKind,
    components: [DxfHelixCoordinateSemanticValue; 3],
}

impl DxfHelixVectorSemantics {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfHelixRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfHelixVectorKind {
        self.kind
    }

    #[must_use]
    pub const fn components(&self) -> &[DxfHelixCoordinateSemanticValue; 3] {
        &self.components
    }

    #[must_use]
    pub fn vector_value(&self) -> Option<[DxfDouble; 3]> {
        Some([
            *self.components[0].value()?,
            *self.components[1].value()?,
            *self.components[2].value()?,
        ])
    }
}

/// Three stable WCS vector entries per retained HELIX record.
#[derive(Debug)]
pub struct DxfHelixVectorDirectory {
    source_id: DxfSourceId,
    cards: DxfHelixCardDirectory,
    entries: Box<[DxfHelixVectorSemantics]>,
}

impl DxfHelixVectorDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.helix_card_directory(cancellation)?;
        ensure_source(document.source_id(), cards.source_id())?;
        let capacity = cards
            .evidence_directory()
            .records()
            .len()
            .checked_mul(DXF_HELIX_VECTOR_KINDS.len())
            .ok_or_else(invalid_internal_data)?;
        let mut entries = Vec::new();
        entries.try_reserve(capacity).map_err(|_| out_of_memory())?;
        for record in cards.evidence_directory().records().iter().copied() {
            for kind in DXF_HELIX_VECTOR_KINDS {
                ensure_not_cancelled(cancellation)?;
                entries.push(DxfHelixVectorSemantics {
                    ordinal: compact_len(entries.len())?,
                    record,
                    kind,
                    components: vector_components(&cards, record, kind)?,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfHelixCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHelixVectorSemantics] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHelixVectorSemantics> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> Option<&[DxfHelixVectorSemantics]> {
        self.cards
            .evidence_directory()
            .record_for_raw_ordinal(raw)?;
        let start = self
            .entries
            .partition_point(|entry| entry.record().entity().record().ordinal() < raw);
        let end = self
            .entries
            .partition_point(|entry| entry.record().entity().record().ordinal() <= raw);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn vector_for_kind(
        &self,
        raw: u64,
        kind: DxfHelixVectorKind,
    ) -> Option<DxfHelixVectorSemantics> {
        self.entries_for_raw_record(raw)?
            .iter()
            .copied()
            .find(|entry| entry.kind() == kind)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn helix_vector_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixVectorDirectory, DxfError> {
        DxfHelixVectorDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn helix_vector_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixVectorDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_vector_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn helix_vector_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixVectorDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_vector_directory(cancellation)
    }
}

fn vector_components(
    cards: &DxfHelixCardDirectory,
    record: DxfHelixRecordEntry,
    kind: DxfHelixVectorKind,
) -> Result<[DxfHelixCoordinateSemanticValue; 3], DxfError> {
    let roles = component_roles(kind);
    Ok([
        component_semantic(cards, record, roles[0])?,
        component_semantic(cards, record, roles[1])?,
        component_semantic(cards, record, roles[2])?,
    ])
}

fn component_semantic(
    cards: &DxfHelixCardDirectory,
    record: DxfHelixRecordEntry,
    role: DxfHelixValueRole,
) -> Result<DxfHelixCoordinateSemanticValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), HELIX_NAMESPACE, field_id(role));
    let card = cards
        .card_for_role(record.entity().record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfHelixCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHelixCardState::Multiple { occurrence_count } => Ok(DxfSemanticValue::invalid(
            DxfHelixCoordinateIssue::MultipleComponents { occurrence_count },
            field,
            None,
        )),
        DxfHelixCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = raw_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfHelixNumber::Double(number)) if number.is_finite() => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfHelixNumber::Double(number)) => DxfSemanticValue::invalid(
                    DxfHelixCoordinateIssue::NonFiniteDouble(number),
                    field,
                    Some(raw),
                ),
                Err(DxfHelixNumericIssue::InvalidAsciiNumber(issue)) => DxfSemanticValue::invalid(
                    DxfHelixCoordinateIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
                Ok(_) => return Err(invalid_internal_data()),
            })
        }
    }
}

fn unique_value(
    cards: &DxfHelixCardDirectory,
    card: DxfHelixValueCard,
) -> Result<DxfHelixValue, DxfError> {
    let [member] = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    cards
        .value_for_member(*member)
        .ok_or_else(invalid_internal_data)
}

fn raw_provenance(value: DxfHelixValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

const fn component_roles(kind: DxfHelixVectorKind) -> [DxfHelixValueRole; 3] {
    match kind {
        DxfHelixVectorKind::AxisBase => [
            DxfHelixValueRole::AxisBaseX,
            DxfHelixValueRole::AxisBaseY,
            DxfHelixValueRole::AxisBaseZ,
        ],
        DxfHelixVectorKind::StartPoint => [
            DxfHelixValueRole::StartPointX,
            DxfHelixValueRole::StartPointY,
            DxfHelixValueRole::StartPointZ,
        ],
        DxfHelixVectorKind::AxisVector => [
            DxfHelixValueRole::AxisVectorX,
            DxfHelixValueRole::AxisVectorY,
            DxfHelixValueRole::AxisVectorZ,
        ],
    }
}

const fn field_id(role: DxfHelixValueRole) -> &'static str {
    match role {
        DxfHelixValueRole::AxisBaseX => "axis_base_x",
        DxfHelixValueRole::AxisBaseY => "axis_base_y",
        DxfHelixValueRole::AxisBaseZ => "axis_base_z",
        DxfHelixValueRole::StartPointX => "start_point_x",
        DxfHelixValueRole::StartPointY => "start_point_y",
        DxfHelixValueRole::StartPointZ => "start_point_z",
        DxfHelixValueRole::AxisVectorX => "axis_vector_x",
        DxfHelixValueRole::AxisVectorY => "axis_vector_y",
        DxfHelixValueRole::AxisVectorZ => "axis_vector_z",
        _ => "unreviewed_non_coordinate",
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
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

//! Typed scalar semantics for HELIX subclass fields.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfHelixCardDirectory, DxfHelixCardState, DxfHelixNumber,
    DxfHelixNumericIssue, DxfHelixRecordEntry, DxfHelixValue, DxfHelixValueCard, DxfHelixValueRole,
    DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId,
};

const HELIX_NAMESPACE: &str = "entity.helix";

pub const DXF_HELIX_SCALAR_ROLES: [DxfHelixValueRole; 7] = [
    DxfHelixValueRole::MajorVersion,
    DxfHelixValueRole::MaintenanceVersion,
    DxfHelixValueRole::Radius,
    DxfHelixValueRole::Turns,
    DxfHelixValueRole::TurnHeight,
    DxfHelixValueRole::Handedness,
    DxfHelixValueRole::ConstraintType,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixHandedness {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixConstraintType {
    TurnHeight,
    Turns,
    Height,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixScalarValue {
    MajorVersion(i32),
    MaintenanceVersion(i32),
    Radius(DxfDouble),
    Turns(DxfDouble),
    TurnHeight(DxfDouble),
    Handedness(DxfHelixHandedness),
    ConstraintType(DxfHelixConstraintType),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixScalarIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
    InvalidHandedness(i16),
    InvalidConstraintType(i16),
}

pub type DxfHelixScalarSemanticValue = DxfSemanticValue<DxfHelixScalarValue, DxfHelixScalarIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixScalarEntry {
    ordinal: u32,
    record: DxfHelixRecordEntry,
    role: DxfHelixValueRole,
    semantic: DxfHelixScalarSemanticValue,
}

impl DxfHelixScalarEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfHelixRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfHelixValueRole {
        self.role
    }

    #[must_use]
    pub const fn semantic(&self) -> &DxfHelixScalarSemanticValue {
        &self.semantic
    }
}

/// Seven stable scalar semantics per retained HELIX record.
#[derive(Debug)]
pub struct DxfHelixScalarDirectory {
    source_id: DxfSourceId,
    cards: DxfHelixCardDirectory,
    entries: Box<[DxfHelixScalarEntry]>,
}

impl DxfHelixScalarDirectory {
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
            .checked_mul(DXF_HELIX_SCALAR_ROLES.len())
            .ok_or_else(invalid_internal_data)?;
        let mut entries = Vec::new();
        entries.try_reserve(capacity).map_err(|_| out_of_memory())?;
        for record in cards.evidence_directory().records().iter().copied() {
            for role in DXF_HELIX_SCALAR_ROLES {
                ensure_not_cancelled(cancellation)?;
                entries.push(DxfHelixScalarEntry {
                    ordinal: compact_len(entries.len())?,
                    record,
                    role,
                    semantic: scalar_semantic(&cards, record, role)?,
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
    pub fn entries(&self) -> &[DxfHelixScalarEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHelixScalarEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> Option<&[DxfHelixScalarEntry]> {
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
    pub fn entry_for_role(&self, raw: u64, role: DxfHelixValueRole) -> Option<DxfHelixScalarEntry> {
        self.entries_for_raw_record(raw)?
            .iter()
            .copied()
            .find(|entry| entry.role() == role)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn helix_scalar_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixScalarDirectory, DxfError> {
        DxfHelixScalarDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn helix_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_scalar_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn helix_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_scalar_directory(cancellation)
    }
}

fn scalar_semantic(
    cards: &DxfHelixCardDirectory,
    record: DxfHelixRecordEntry,
    role: DxfHelixValueRole,
) -> Result<DxfHelixScalarSemanticValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), HELIX_NAMESPACE, field_id(role));
    let card = cards
        .card_for_role(record.entity().record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfHelixCardState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHelixCardState::Multiple { occurrence_count } => Ok(DxfSemanticValue::invalid(
            DxfHelixScalarIssue::MultipleValues { occurrence_count },
            field,
            None,
        )),
        DxfHelixCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = raw_provenance(value)?;
            Ok(match value.value() {
                Ok(number) => match semantic_value(role, number)? {
                    Ok(semantic) => DxfSemanticValue::explicit(semantic, field, raw),
                    Err(issue) => DxfSemanticValue::invalid(issue, field, Some(raw)),
                },
                Err(DxfHelixNumericIssue::InvalidAsciiNumber(issue)) => DxfSemanticValue::invalid(
                    DxfHelixScalarIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn semantic_value(
    role: DxfHelixValueRole,
    number: DxfHelixNumber,
) -> Result<Result<DxfHelixScalarValue, DxfHelixScalarIssue>, DxfError> {
    use DxfHelixScalarValue as Value;
    let semantic = match (role, number) {
        (DxfHelixValueRole::MajorVersion, DxfHelixNumber::Int32(value)) => {
            Ok(Value::MajorVersion(value))
        }
        (DxfHelixValueRole::MaintenanceVersion, DxfHelixNumber::Int32(value)) => {
            Ok(Value::MaintenanceVersion(value))
        }
        (DxfHelixValueRole::Radius, DxfHelixNumber::Double(value)) => finite(value, Value::Radius),
        (DxfHelixValueRole::Turns, DxfHelixNumber::Double(value)) => finite(value, Value::Turns),
        (DxfHelixValueRole::TurnHeight, DxfHelixNumber::Double(value)) => {
            finite(value, Value::TurnHeight)
        }
        (DxfHelixValueRole::Handedness, DxfHelixNumber::Int16(0)) => {
            Ok(Value::Handedness(DxfHelixHandedness::Left))
        }
        (DxfHelixValueRole::Handedness, DxfHelixNumber::Int16(1)) => {
            Ok(Value::Handedness(DxfHelixHandedness::Right))
        }
        (DxfHelixValueRole::Handedness, DxfHelixNumber::Int16(value)) => {
            Err(DxfHelixScalarIssue::InvalidHandedness(value))
        }
        (DxfHelixValueRole::ConstraintType, DxfHelixNumber::Int16(0)) => {
            Ok(Value::ConstraintType(DxfHelixConstraintType::TurnHeight))
        }
        (DxfHelixValueRole::ConstraintType, DxfHelixNumber::Int16(1)) => {
            Ok(Value::ConstraintType(DxfHelixConstraintType::Turns))
        }
        (DxfHelixValueRole::ConstraintType, DxfHelixNumber::Int16(2)) => {
            Ok(Value::ConstraintType(DxfHelixConstraintType::Height))
        }
        (DxfHelixValueRole::ConstraintType, DxfHelixNumber::Int16(value)) => {
            Err(DxfHelixScalarIssue::InvalidConstraintType(value))
        }
        _ => return Err(invalid_internal_data()),
    };
    Ok(semantic)
}

fn finite(
    value: DxfDouble,
    wrap: fn(DxfDouble) -> DxfHelixScalarValue,
) -> Result<DxfHelixScalarValue, DxfHelixScalarIssue> {
    if value.is_finite() {
        Ok(wrap(value))
    } else {
        Err(DxfHelixScalarIssue::NonFiniteDouble(value))
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

const fn field_id(role: DxfHelixValueRole) -> &'static str {
    match role {
        DxfHelixValueRole::MajorVersion => "major_version",
        DxfHelixValueRole::MaintenanceVersion => "maintenance_version",
        DxfHelixValueRole::Radius => "radius",
        DxfHelixValueRole::Turns => "turns",
        DxfHelixValueRole::TurnHeight => "turn_height",
        DxfHelixValueRole::Handedness => "handedness",
        DxfHelixValueRole::ConstraintType => "constraint_type",
        _ => "unreviewed_non_scalar",
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

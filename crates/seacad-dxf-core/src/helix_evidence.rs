//! Source-anchored HELIX subclass evidence.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfDouble, DxfEntityClassification, DxfEntityDirectory, DxfEntityRef,
    DxfEntityTopic, DxfError, DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfRawGroup, DxfSourceId, ascii_numeric::parse_i16 as parse_ascii_i16,
    raw_double::decode_raw_ascii_numeric, raw_double::decode_raw_double,
    raw_double::read_raw_fixed_payload, raw_integer::decode_raw_i16, raw_integer::decode_raw_i32,
};

pub const DXF_HELIX_ROLES: [DxfHelixValueRole; 16] = [
    DxfHelixValueRole::MajorVersion,
    DxfHelixValueRole::MaintenanceVersion,
    DxfHelixValueRole::AxisBaseX,
    DxfHelixValueRole::AxisBaseY,
    DxfHelixValueRole::AxisBaseZ,
    DxfHelixValueRole::StartPointX,
    DxfHelixValueRole::StartPointY,
    DxfHelixValueRole::StartPointZ,
    DxfHelixValueRole::AxisVectorX,
    DxfHelixValueRole::AxisVectorY,
    DxfHelixValueRole::AxisVectorZ,
    DxfHelixValueRole::Radius,
    DxfHelixValueRole::Turns,
    DxfHelixValueRole::TurnHeight,
    DxfHelixValueRole::Handedness,
    DxfHelixValueRole::ConstraintType,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixValueRole {
    MajorVersion,
    MaintenanceVersion,
    AxisBaseX,
    AxisBaseY,
    AxisBaseZ,
    StartPointX,
    StartPointY,
    StartPointZ,
    AxisVectorX,
    AxisVectorY,
    AxisVectorZ,
    Radius,
    Turns,
    TurnHeight,
    Handedness,
    ConstraintType,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixNumber {
    Double(DxfDouble),
    Int16(i16),
    Int32(i32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixValueRange {
    start: u32,
    end: u32,
}

impl DxfHelixValueRange {
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
pub struct DxfHelixValue {
    group: DxfRawGroup,
    role: DxfHelixValueRole,
    value: Result<DxfHelixNumber, DxfHelixNumericIssue>,
}

impl DxfHelixValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfHelixValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfHelixNumber, DxfHelixNumericIssue> {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixRecordEntry {
    entity: DxfEntityRef,
    value_range: DxfHelixValueRange,
}

impl DxfHelixRecordEntry {
    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn value_range(self) -> DxfHelixValueRange {
        self.value_range
    }
}

/// Exact `AcDbHelix` values without selection, defaults, or validation.
#[derive(Debug)]
pub struct DxfHelixDirectory {
    source_id: DxfSourceId,
    entities: DxfEntityDirectory,
    application_groups: DxfApplicationGroupDirectory,
    records: Box<[DxfHelixRecordEntry]>,
    values: Box<[DxfHelixValue]>,
}

impl DxfHelixDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let entities = document.entity_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        ensure_source(document.source_id(), entities.source_id())?;
        ensure_source(document.source_id(), application_groups.source_id())?;

        let mut records = Vec::new();
        let mut values = Vec::new();
        for entity in entities.entities().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if entity.classification() != DxfEntityClassification::Canonical(DxfEntityTopic::HELIX)
            {
                continue;
            }
            let start = compact_len(values.len())?;
            append_values(
                document,
                entity,
                &application_groups,
                cancellation,
                &mut values,
            )?;
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfHelixRecordEntry {
                entity,
                value_range: DxfHelixValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entities,
            application_groups,
            records: records.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn entity_directory(&self) -> &DxfEntityDirectory {
        &self.entities
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        &self.application_groups
    }

    #[must_use]
    pub fn records(&self) -> &[DxfHelixRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfHelixValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(&self, raw: u64) -> Option<DxfHelixRecordEntry> {
        self.records
            .binary_search_by_key(&raw, |entry| entry.entity().record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(&self, raw: u64) -> Option<&[DxfHelixValue]> {
        let entry = self.record_for_raw_ordinal(raw)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfHelixValue> {
        let index = self
            .values
            .partition_point(|entry| entry.group().occurrence() < occurrence);
        self.values
            .get(index)
            .copied()
            .filter(|entry| entry.group().occurrence() == occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn helix_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixDirectory, DxfError> {
        DxfHelixDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn helix_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn helix_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_directory(cancellation)
    }
}

fn append_values(
    document: DxfRawDocumentView<'_>,
    entity: DxfEntityRef,
    application_groups: &DxfApplicationGroupDirectory,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfHelixValue>,
) -> Result<(), DxfError> {
    let mut in_helix_subclass = false;
    for occurrence in
        entity.record().marker_occurrence().saturating_add(1)..entity.record().group_range().end()
    {
        ensure_not_cancelled(cancellation)?;
        if application_groups
            .group_for_content_occurrence(occurrence)
            .is_some()
        {
            continue;
        }
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if group.group_code().value() == 100 {
            in_helix_subclass =
                document.raw_span_equals_exact(group.value_payload_span(), b"AcDbHelix")?;
            continue;
        }
        if !in_helix_subclass {
            continue;
        }
        let Some(role) = value_role(group.group_code().value()) else {
            continue;
        };
        let value = decode_number(document, group, role, cancellation)?;
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(DxfHelixValue { group, role, value });
    }
    Ok(())
}

fn decode_number(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfHelixValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfHelixNumber, DxfHelixNumericIssue>, DxfError> {
    let result = match role {
        DxfHelixValueRole::MajorVersion | DxfHelixValueRole::MaintenanceVersion => {
            decode_raw_i32(document, group, cancellation)?.map(DxfHelixNumber::Int32)
        }
        DxfHelixValueRole::ConstraintType => {
            decode_raw_i16(document, group, cancellation)?.map(DxfHelixNumber::Int16)
        }
        DxfHelixValueRole::Handedness => {
            decode_boolean(document, group, cancellation)?.map(DxfHelixNumber::Int16)
        }
        _ => decode_raw_double(document, group, cancellation)?.map(DxfHelixNumber::Double),
    };
    Ok(result.map_err(DxfHelixNumericIssue::InvalidAsciiNumber))
}

fn decode_boolean(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Result<i16, DxfAsciiNumericIssue>, DxfError> {
    match document.format() {
        DxfRawDocumentFormat::Ascii => decode_raw_ascii_numeric(
            document,
            group.value_payload_span(),
            cancellation,
            parse_ascii_i16,
        ),
        DxfRawDocumentFormat::Binary => read_raw_fixed_payload(document, group, cancellation)
            .map(|bytes: [u8; 1]| Ok(i16::from(bytes[0]))),
    }
}

const fn value_role(group_code: i16) -> Option<DxfHelixValueRole> {
    use DxfHelixValueRole::*;
    match group_code {
        90 => Some(MajorVersion),
        91 => Some(MaintenanceVersion),
        10 => Some(AxisBaseX),
        20 => Some(AxisBaseY),
        30 => Some(AxisBaseZ),
        11 => Some(StartPointX),
        21 => Some(StartPointY),
        31 => Some(StartPointZ),
        12 => Some(AxisVectorX),
        22 => Some(AxisVectorY),
        32 => Some(AxisVectorZ),
        40 => Some(Radius),
        41 => Some(Turns),
        42 => Some(TurnHeight),
        290 => Some(Handedness),
        280 => Some(ConstraintType),
        _ => None,
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

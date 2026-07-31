//! Source-anchored SPLINE defining-value evidence.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup,
    DxfRawRecord, DxfRawRecordSectionKind, DxfSourceId, raw_double::decode_raw_double,
    raw_integer::decode_raw_i16,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineValueRole {
    Flags,
    Degree,
    KnotCount,
    ControlPointCount,
    FitPointCount,
    KnotTolerance,
    ControlPointTolerance,
    FitTolerance,
    StartTangentX,
    StartTangentY,
    StartTangentZ,
    EndTangentX,
    EndTangentY,
    EndTangentZ,
    KnotValue,
    ControlPointX,
    ControlPointY,
    ControlPointZ,
    FitPointX,
    FitPointY,
    FitPointZ,
    NormalX,
    NormalY,
    NormalZ,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineNumber {
    Double(DxfDouble),
    Int16(i16),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineValueRange {
    start: u32,
    end: u32,
}

impl DxfSplineValueRange {
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
pub struct DxfSplineValue {
    group: DxfRawGroup,
    role: DxfSplineValueRole,
    value: Result<DxfSplineNumber, DxfSplineNumericIssue>,
}

impl DxfSplineValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfSplineValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfSplineNumber, DxfSplineNumericIssue> {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineRecordEntry {
    record: DxfRawRecord,
    value_range: DxfSplineValueRange,
}

impl DxfSplineRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn value_range(self) -> DxfSplineValueRange {
        self.value_range
    }
}

/// Exact SPLINE values without selection, defaults, validation, or geometry.
#[derive(Debug)]
pub struct DxfSplineDirectory {
    source_id: DxfSourceId,
    raw_record_count: u32,
    records: Box<[DxfSplineRecordEntry]>,
    values: Box<[DxfSplineValue]>,
}

impl DxfSplineDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        for observed in [raw_records.source_id(), application_groups.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let raw_record_count = compact_len(raw_records.records().len())?;
        let mut records = Vec::new();
        let mut values = Vec::new();
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !matches!(
                record.section_kind(),
                DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
            ) || !is_spline(document, record)?
            {
                continue;
            }
            let start = compact_len(values.len())?;
            append_values(
                document,
                record,
                &application_groups,
                cancellation,
                &mut values,
            )?;
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfSplineRecordEntry {
                record,
                value_range: DxfSplineValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            raw_record_count,
            records: records.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn raw_record_count(&self) -> u64 {
        self.raw_record_count as u64
    }

    #[must_use]
    pub fn records(&self) -> &[DxfSplineRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfSplineValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(&self, raw_ordinal: u64) -> Option<DxfSplineRecordEntry> {
        self.records
            .binary_search_by_key(&raw_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(&self, raw_ordinal: u64) -> Option<&[DxfSplineValue]> {
        let entry = self.record_for_raw_ordinal(raw_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfSplineValue> {
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
    pub fn spline_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineDirectory, DxfError> {
        DxfSplineDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_directory(cancellation)
    }
}

fn append_values(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    application_groups: &DxfApplicationGroupDirectory,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfSplineValue>,
) -> Result<(), DxfError> {
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
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
        let Some(role) = value_role(group.group_code().value()) else {
            continue;
        };
        let value = decode_number(document, group, role, cancellation)?;
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(DxfSplineValue { group, role, value });
    }
    Ok(())
}

fn is_spline(document: DxfRawDocumentView<'_>, record: DxfRawRecord) -> Result<bool, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    document.raw_span_equals_exact(marker.value_payload_span(), b"SPLINE")
}

fn decode_number(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfSplineValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfSplineNumber, DxfSplineNumericIssue>, DxfError> {
    if is_integer(role) {
        decode_raw_i16(document, group, cancellation).map(|value| {
            value
                .map(DxfSplineNumber::Int16)
                .map_err(DxfSplineNumericIssue::InvalidAsciiNumber)
        })
    } else {
        decode_raw_double(document, group, cancellation).map(|value| {
            value
                .map(DxfSplineNumber::Double)
                .map_err(DxfSplineNumericIssue::InvalidAsciiNumber)
        })
    }
}

const fn is_integer(role: DxfSplineValueRole) -> bool {
    matches!(
        role,
        DxfSplineValueRole::Flags
            | DxfSplineValueRole::Degree
            | DxfSplineValueRole::KnotCount
            | DxfSplineValueRole::ControlPointCount
            | DxfSplineValueRole::FitPointCount
    )
}

const fn value_role(group_code: i16) -> Option<DxfSplineValueRole> {
    use DxfSplineValueRole::*;
    match group_code {
        70 => Some(Flags),
        71 => Some(Degree),
        72 => Some(KnotCount),
        73 => Some(ControlPointCount),
        74 => Some(FitPointCount),
        42 => Some(KnotTolerance),
        43 => Some(ControlPointTolerance),
        44 => Some(FitTolerance),
        12 => Some(StartTangentX),
        22 => Some(StartTangentY),
        32 => Some(StartTangentZ),
        13 => Some(EndTangentX),
        23 => Some(EndTangentY),
        33 => Some(EndTangentZ),
        40 => Some(KnotValue),
        10 => Some(ControlPointX),
        20 => Some(ControlPointY),
        30 => Some(ControlPointZ),
        11 => Some(FitPointX),
        21 => Some(FitPointY),
        31 => Some(FitPointZ),
        210 => Some(NormalX),
        220 => Some(NormalY),
        230 => Some(NormalZ),
        _ => None,
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

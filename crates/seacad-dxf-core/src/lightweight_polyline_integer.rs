//! Source-anchored LWPOLYLINE signed-integer evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfSourceId,
    raw_integer::{decode_raw_i16, decode_raw_i32},
};

/// Documented signed-integer role retained from one LWPOLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineIntegerRole {
    VertexCount,
    Flags,
    VertexIdentifier,
}

/// Exact signed wire value without merging the i16 and i32 domains.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineInteger {
    I16(i16),
    I32(i32),
}

/// Lexical reason why one LWPOLYLINE integer cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineIntegerIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open integer-value range owned by one LWPOLYLINE record entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineIntegerRange {
    start: u32,
    end: u32,
}

impl DxfLightweightPolylineIntegerRange {
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

/// One source-order LWPOLYLINE signed integer and its raw evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineIntegerValue {
    group: DxfRawGroup,
    role: DxfLightweightPolylineIntegerRole,
    value: Result<DxfLightweightPolylineInteger, DxfLightweightPolylineIntegerIssue>,
}

impl DxfLightweightPolylineIntegerValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfLightweightPolylineIntegerRole {
        self.role
    }

    pub const fn value(
        self,
    ) -> Result<DxfLightweightPolylineInteger, DxfLightweightPolylineIntegerIssue> {
        self.value
    }
}

/// One exact LWPOLYLINE marker and its source-order integer-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineIntegerRecordEntry {
    record: DxfRawRecord,
    value_range: DxfLightweightPolylineIntegerRange,
}

impl DxfLightweightPolylineIntegerRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn value_range(self) -> DxfLightweightPolylineIntegerRange {
        self.value_range
    }
}

/// Immutable evidence directory for exact LWPOLYLINE signed integers.
///
/// Entries preserve source order, duplicates, lexical invalidity, raw spans,
/// and exact i16/i32 wire domains. They do not select canonical values,
/// interpret flags, validate counts, group vertices, or assemble geometry.
#[derive(Debug)]
pub struct DxfLightweightPolylineIntegerDirectory {
    source_id: DxfSourceId,
    raw_record_count: u32,
    records: Box<[DxfLightweightPolylineIntegerRecordEntry]>,
    values: Box<[DxfLightweightPolylineIntegerValue]>,
}

impl DxfLightweightPolylineIntegerDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        if raw_records.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: raw_records.source_id(),
            });
        }

        let raw_record_count = compact_len(raw_records.records().len())?;
        let mut records = Vec::new();
        let mut values = Vec::new();
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !matches!(
                record.section_kind(),
                DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
            ) || !is_lightweight_polyline(document, record)?
            {
                continue;
            }

            let start = compact_len(values.len())?;
            for occurrence in
                record.marker_occurrence().saturating_add(1)..record.group_range().end()
            {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                let Some(role) = integer_role(group.group_code().value()) else {
                    continue;
                };
                let value = decode_integer(document, group, role, cancellation)?;
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfLightweightPolylineIntegerValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfLightweightPolylineIntegerRecordEntry {
                record,
                value_range: DxfLightweightPolylineIntegerRange::new(start, end)?,
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
    pub fn records(&self) -> &[DxfLightweightPolylineIntegerRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfLightweightPolylineIntegerValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfLightweightPolylineIntegerRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfLightweightPolylineIntegerValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfLightweightPolylineIntegerValue> {
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
    pub fn lightweight_polyline_integer_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineIntegerDirectory, DxfError> {
        DxfLightweightPolylineIntegerDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_integer_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineIntegerDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_integer_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_integer_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineIntegerDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_integer_directory(cancellation)
    }
}

fn decode_integer(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfLightweightPolylineIntegerRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfLightweightPolylineInteger, DxfLightweightPolylineIntegerIssue>, DxfError> {
    match role {
        DxfLightweightPolylineIntegerRole::Flags => decode_raw_i16(document, group, cancellation)
            .map(|value| {
                value
                    .map(DxfLightweightPolylineInteger::I16)
                    .map_err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber)
            }),
        DxfLightweightPolylineIntegerRole::VertexCount
        | DxfLightweightPolylineIntegerRole::VertexIdentifier => {
            decode_raw_i32(document, group, cancellation).map(|value| {
                value
                    .map(DxfLightweightPolylineInteger::I32)
                    .map_err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber)
            })
        }
    }
}

fn is_lightweight_polyline(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<bool, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    document.raw_span_equals_exact(marker.value_payload_span(), b"LWPOLYLINE")
}

const fn integer_role(group_code: i16) -> Option<DxfLightweightPolylineIntegerRole> {
    match group_code {
        90 => Some(DxfLightweightPolylineIntegerRole::VertexCount),
        70 => Some(DxfLightweightPolylineIntegerRole::Flags),
        91 => Some(DxfLightweightPolylineIntegerRole::VertexIdentifier),
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

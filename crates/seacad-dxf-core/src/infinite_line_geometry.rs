//! Source-anchored RAY and XLINE defining-value evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfSourceId, raw_double::decode_raw_double,
};

/// Exact infinite-line entity family recognized by this evidence directory.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInfiniteLineGeometryKind {
    Ray,
    Xline,
}

/// Documented RAY/XLINE value role without vector normalization.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInfiniteLineGeometryValueRole {
    WcsStartOrFirstPointX,
    WcsStartOrFirstPointY,
    WcsStartOrFirstPointZ,
    WcsUnitDirectionX,
    WcsUnitDirectionY,
    WcsUnitDirectionZ,
}

/// Lexical reason why one RAY/XLINE value cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInfiniteLineGeometryNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open value range owned by one RAY or XLINE record entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInfiniteLineGeometryValueRange {
    start: u32,
    end: u32,
}

impl DxfInfiniteLineGeometryValueRange {
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

/// One source-order RAY/XLINE value and its exact binary64 evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInfiniteLineGeometryValue {
    group: DxfRawGroup,
    role: DxfInfiniteLineGeometryValueRole,
    value: Result<DxfDouble, DxfInfiniteLineGeometryNumericIssue>,
}

impl DxfInfiniteLineGeometryValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfInfiniteLineGeometryValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfDouble, DxfInfiniteLineGeometryNumericIssue> {
        self.value
    }
}

/// One exact RAY/XLINE marker and its source-order defining-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInfiniteLineGeometryRecordEntry {
    record: DxfRawRecord,
    kind: DxfInfiniteLineGeometryKind,
    value_range: DxfInfiniteLineGeometryValueRange,
}

impl DxfInfiniteLineGeometryRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfInfiniteLineGeometryKind {
        self.kind
    }

    #[must_use]
    pub const fn value_range(self) -> DxfInfiniteLineGeometryValueRange {
        self.value_range
    }
}

/// Immutable evidence directory for exact RAY/XLINE defining values.
///
/// Entries preserve source order, duplicates, lexical invalidity, raw spans,
/// and original binary64 bits. They do not choose canonical values, require
/// components, validate a unit direction, or assemble infinite geometry.
#[derive(Debug)]
pub struct DxfInfiniteLineGeometryDirectory {
    source_id: DxfSourceId,
    record_count: u32,
    records: Box<[DxfInfiniteLineGeometryRecordEntry]>,
    values: Box<[DxfInfiniteLineGeometryValue]>,
}

impl DxfInfiniteLineGeometryDirectory {
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

        let record_count = compact_len(raw_records.records().len())?;
        let mut records = Vec::new();
        let mut values = Vec::new();
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !matches!(
                record.section_kind(),
                DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
            ) {
                continue;
            }
            let Some(kind) = infinite_line_kind(document, record)? else {
                continue;
            };

            let start = compact_len(values.len())?;
            for occurrence in
                record.marker_occurrence().saturating_add(1)..record.group_range().end()
            {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                let Some(role) = value_role(group.group_code().value()) else {
                    continue;
                };
                let value = decode_raw_double(document, group, cancellation)?
                    .map_err(DxfInfiniteLineGeometryNumericIssue::InvalidAsciiNumber);
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfInfiniteLineGeometryValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfInfiniteLineGeometryRecordEntry {
                record,
                kind,
                value_range: DxfInfiniteLineGeometryValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            record_count,
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
        self.record_count as u64
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInfiniteLineGeometryRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfInfiniteLineGeometryValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInfiniteLineGeometryRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfInfiniteLineGeometryValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfInfiniteLineGeometryValue> {
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
    pub fn infinite_line_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometryDirectory, DxfError> {
        DxfInfiniteLineGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn infinite_line_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).infinite_line_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn infinite_line_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).infinite_line_geometry_directory(cancellation)
    }
}

fn infinite_line_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<Option<DxfInfiniteLineGeometryKind>, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    if document.raw_span_equals_exact(marker.value_payload_span(), b"RAY")? {
        Ok(Some(DxfInfiniteLineGeometryKind::Ray))
    } else if document.raw_span_equals_exact(marker.value_payload_span(), b"XLINE")? {
        Ok(Some(DxfInfiniteLineGeometryKind::Xline))
    } else {
        Ok(None)
    }
}

const fn value_role(group_code: i16) -> Option<DxfInfiniteLineGeometryValueRole> {
    use DxfInfiniteLineGeometryValueRole::{
        WcsStartOrFirstPointX, WcsStartOrFirstPointY, WcsStartOrFirstPointZ, WcsUnitDirectionX,
        WcsUnitDirectionY, WcsUnitDirectionZ,
    };
    match group_code {
        10 => Some(WcsStartOrFirstPointX),
        20 => Some(WcsStartOrFirstPointY),
        30 => Some(WcsStartOrFirstPointZ),
        11 => Some(WcsUnitDirectionX),
        21 => Some(WcsUnitDirectionY),
        31 => Some(WcsUnitDirectionZ),
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

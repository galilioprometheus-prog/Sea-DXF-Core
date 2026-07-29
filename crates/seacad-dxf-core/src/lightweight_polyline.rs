//! Source-anchored LWPOLYLINE floating-point evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfSourceId, raw_double::decode_raw_double,
};

/// Documented floating-point role retained from one LWPOLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineValueRole {
    OcsElevation,
    Thickness,
    ConstantWidth,
    OcsVertexX,
    OcsVertexY,
    StartWidth,
    EndWidth,
    Bulge,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
}

/// Lexical reason why one LWPOLYLINE value cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open value range owned by one LWPOLYLINE record entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineValueRange {
    start: u32,
    end: u32,
}

impl DxfLightweightPolylineValueRange {
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

/// One source-order LWPOLYLINE value and its exact binary64 evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineValue {
    group: DxfRawGroup,
    role: DxfLightweightPolylineValueRole,
    value: Result<DxfDouble, DxfLightweightPolylineNumericIssue>,
}

impl DxfLightweightPolylineValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfLightweightPolylineValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfDouble, DxfLightweightPolylineNumericIssue> {
        self.value
    }
}

/// One exact LWPOLYLINE marker and its source-order floating-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineRecordEntry {
    record: DxfRawRecord,
    value_range: DxfLightweightPolylineValueRange,
}

impl DxfLightweightPolylineRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn value_range(self) -> DxfLightweightPolylineValueRange {
        self.value_range
    }
}

/// Immutable evidence directory for exact LWPOLYLINE floating-point values.
///
/// Entries preserve source order, duplicates, lexical invalidity, raw spans,
/// and original binary64 bits. They do not group vertices, apply defaults,
/// validate counts or flags, transform OCS, or assemble polyline geometry.
#[derive(Debug)]
pub struct DxfLightweightPolylineDirectory {
    source_id: DxfSourceId,
    raw_record_count: u32,
    records: Box<[DxfLightweightPolylineRecordEntry]>,
    values: Box<[DxfLightweightPolylineValue]>,
}

impl DxfLightweightPolylineDirectory {
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
                let Some(role) = value_role(group.group_code().value()) else {
                    continue;
                };
                let value = decode_raw_double(document, group, cancellation)?
                    .map_err(DxfLightweightPolylineNumericIssue::InvalidAsciiNumber);
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfLightweightPolylineValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfLightweightPolylineRecordEntry {
                record,
                value_range: DxfLightweightPolylineValueRange::new(start, end)?,
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
    pub fn records(&self) -> &[DxfLightweightPolylineRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfLightweightPolylineValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfLightweightPolylineRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfLightweightPolylineValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfLightweightPolylineValue> {
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
    pub fn lightweight_polyline_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineDirectory, DxfError> {
        DxfLightweightPolylineDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_directory(cancellation)
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

const fn value_role(group_code: i16) -> Option<DxfLightweightPolylineValueRole> {
    use DxfLightweightPolylineValueRole::{
        Bulge, ConstantWidth, EndWidth, ExtrusionX, ExtrusionY, ExtrusionZ, OcsElevation,
        OcsVertexX, OcsVertexY, StartWidth, Thickness,
    };
    match group_code {
        38 => Some(OcsElevation),
        39 => Some(Thickness),
        43 => Some(ConstantWidth),
        10 => Some(OcsVertexX),
        20 => Some(OcsVertexY),
        40 => Some(StartWidth),
        41 => Some(EndWidth),
        42 => Some(Bulge),
        210 => Some(ExtrusionX),
        220 => Some(ExtrusionY),
        230 => Some(ExtrusionZ),
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

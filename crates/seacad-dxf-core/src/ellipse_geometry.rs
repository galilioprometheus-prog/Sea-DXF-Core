//! Source-anchored ELLIPSE defining-value evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfSourceId, raw_double::decode_raw_double,
};

/// Documented ELLIPSE value role without semantic normalization.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEllipseGeometryValueRole {
    WcsCenterX,
    WcsCenterY,
    WcsCenterZ,
    WcsMajorAxisEndpointX,
    WcsMajorAxisEndpointY,
    WcsMajorAxisEndpointZ,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
    MinorToMajorAxisRatio,
    StartParameter,
    EndParameter,
}

/// Lexical reason why one ELLIPSE value cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEllipseGeometryNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open value range owned by one ELLIPSE record entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEllipseGeometryValueRange {
    start: u32,
    end: u32,
}

impl DxfEllipseGeometryValueRange {
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

/// One source-order ELLIPSE value and its exact binary64 evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEllipseGeometryValue {
    group: DxfRawGroup,
    role: DxfEllipseGeometryValueRole,
    value: Result<DxfDouble, DxfEllipseGeometryNumericIssue>,
}

impl DxfEllipseGeometryValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfEllipseGeometryValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfDouble, DxfEllipseGeometryNumericIssue> {
        self.value
    }
}

/// One exact ELLIPSE marker and its source-order defining-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEllipseGeometryRecordEntry {
    record: DxfRawRecord,
    value_range: DxfEllipseGeometryValueRange,
}

impl DxfEllipseGeometryRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn value_range(self) -> DxfEllipseGeometryValueRange {
        self.value_range
    }
}

/// Immutable evidence directory for exact ELLIPSE defining values.
///
/// Entries preserve source order, duplicates, lexical invalidity, raw spans,
/// and original binary64 bits. They do not choose canonical values, apply
/// defaults, validate parameters or ratio, or assemble ellipse geometry.
#[derive(Debug)]
pub struct DxfEllipseGeometryDirectory {
    source_id: DxfSourceId,
    record_count: u32,
    records: Box<[DxfEllipseGeometryRecordEntry]>,
    values: Box<[DxfEllipseGeometryValue]>,
}

impl DxfEllipseGeometryDirectory {
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
            ) || !is_ellipse(document, record)?
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
                    .map_err(DxfEllipseGeometryNumericIssue::InvalidAsciiNumber);
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfEllipseGeometryValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfEllipseGeometryRecordEntry {
                record,
                value_range: DxfEllipseGeometryValueRange::new(start, end)?,
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
    pub fn records(&self) -> &[DxfEllipseGeometryRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfEllipseGeometryValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfEllipseGeometryRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfEllipseGeometryValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfEllipseGeometryValue> {
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
    pub fn ellipse_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEllipseGeometryDirectory, DxfError> {
        DxfEllipseGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn ellipse_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEllipseGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).ellipse_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn ellipse_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEllipseGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).ellipse_geometry_directory(cancellation)
    }
}

fn is_ellipse(document: DxfRawDocumentView<'_>, record: DxfRawRecord) -> Result<bool, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    document.raw_span_equals_exact(marker.value_payload_span(), b"ELLIPSE")
}

const fn value_role(group_code: i16) -> Option<DxfEllipseGeometryValueRole> {
    use DxfEllipseGeometryValueRole::{
        EndParameter, ExtrusionX, ExtrusionY, ExtrusionZ, MinorToMajorAxisRatio, StartParameter,
        WcsCenterX, WcsCenterY, WcsCenterZ, WcsMajorAxisEndpointX, WcsMajorAxisEndpointY,
        WcsMajorAxisEndpointZ,
    };
    match group_code {
        10 => Some(WcsCenterX),
        20 => Some(WcsCenterY),
        30 => Some(WcsCenterZ),
        11 => Some(WcsMajorAxisEndpointX),
        21 => Some(WcsMajorAxisEndpointY),
        31 => Some(WcsMajorAxisEndpointZ),
        210 => Some(ExtrusionX),
        220 => Some(ExtrusionY),
        230 => Some(ExtrusionZ),
        40 => Some(MinorToMajorAxisRatio),
        41 => Some(StartParameter),
        42 => Some(EndParameter),
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

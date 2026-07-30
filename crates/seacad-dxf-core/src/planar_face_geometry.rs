//! Source-anchored 3DFACE, SOLID, and TRACE numeric evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfSourceId, raw_double::decode_raw_double,
    raw_integer::decode_raw_i16,
};

/// Reviewed planar-face entity family.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPlanarFaceKind {
    Face3d,
    Solid,
    Trace,
}

/// Documented numeric role retained without selecting or transforming values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPlanarFaceValueRole {
    FirstCornerX,
    FirstCornerY,
    FirstCornerZ,
    SecondCornerX,
    SecondCornerY,
    SecondCornerZ,
    ThirdCornerX,
    ThirdCornerY,
    ThirdCornerZ,
    FourthCornerX,
    FourthCornerY,
    FourthCornerZ,
    Thickness,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
    InvisibleEdgeFlags,
}

/// Exact signed or binary64 wire value without merging numeric domains.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPlanarFaceNumber {
    Double(DxfDouble),
    Int16(i16),
}

/// Lexical reason why one planar-face number cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPlanarFaceNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open numeric-value range owned by one planar-face record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceValueRange {
    start: u32,
    end: u32,
}

impl DxfPlanarFaceValueRange {
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

/// One source-order documented numeric group and its exact evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceValue {
    group: DxfRawGroup,
    role: DxfPlanarFaceValueRole,
    value: Result<DxfPlanarFaceNumber, DxfPlanarFaceNumericIssue>,
}

impl DxfPlanarFaceValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfPlanarFaceValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfPlanarFaceNumber, DxfPlanarFaceNumericIssue> {
        self.value
    }
}

/// One exact entity marker and its source-order numeric-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceRecordEntry {
    record: DxfRawRecord,
    kind: DxfPlanarFaceKind,
    value_range: DxfPlanarFaceValueRange,
}

impl DxfPlanarFaceRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfPlanarFaceKind {
        self.kind
    }

    #[must_use]
    pub const fn value_range(self) -> DxfPlanarFaceValueRange {
        self.value_range
    }
}

/// Immutable evidence directory for exact 3DFACE, SOLID, and TRACE records.
///
/// Entries preserve source order, duplicates, lexical invalidity, and raw
/// group spans. They do not choose canonical values, apply defaults, validate
/// entity completeness, reorder corners, or transform coordinate systems.
#[derive(Debug)]
pub struct DxfPlanarFaceDirectory {
    source_id: DxfSourceId,
    raw_record_count: u32,
    records: Box<[DxfPlanarFaceRecordEntry]>,
    values: Box<[DxfPlanarFaceValue]>,
}

impl DxfPlanarFaceDirectory {
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
            ) {
                continue;
            }
            let Some(kind) = record_kind(document, record)? else {
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
                let Some(role) = value_role(kind, group.group_code().value()) else {
                    continue;
                };
                let value = decode_number(document, group, role, cancellation)?;
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfPlanarFaceValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfPlanarFaceRecordEntry {
                record,
                kind,
                value_range: DxfPlanarFaceValueRange::new(start, end)?,
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
    pub fn records(&self) -> &[DxfPlanarFaceRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfPlanarFaceValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfPlanarFaceRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(&self, raw_record_ordinal: u64) -> Option<&[DxfPlanarFaceValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfPlanarFaceValue> {
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
    pub fn planar_face_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceDirectory, DxfError> {
        DxfPlanarFaceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn planar_face_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn planar_face_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_directory(cancellation)
    }
}

fn record_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<Option<DxfPlanarFaceKind>, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    if document.raw_span_equals_exact(marker.value_payload_span(), b"3DFACE")? {
        Ok(Some(DxfPlanarFaceKind::Face3d))
    } else if document.raw_span_equals_exact(marker.value_payload_span(), b"SOLID")? {
        Ok(Some(DxfPlanarFaceKind::Solid))
    } else if document.raw_span_equals_exact(marker.value_payload_span(), b"TRACE")? {
        Ok(Some(DxfPlanarFaceKind::Trace))
    } else {
        Ok(None)
    }
}

fn decode_number(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfPlanarFaceValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfPlanarFaceNumber, DxfPlanarFaceNumericIssue>, DxfError> {
    if role == DxfPlanarFaceValueRole::InvisibleEdgeFlags {
        decode_raw_i16(document, group, cancellation).map(|value| {
            value
                .map(DxfPlanarFaceNumber::Int16)
                .map_err(DxfPlanarFaceNumericIssue::InvalidAsciiNumber)
        })
    } else {
        decode_raw_double(document, group, cancellation).map(|value| {
            value
                .map(DxfPlanarFaceNumber::Double)
                .map_err(DxfPlanarFaceNumericIssue::InvalidAsciiNumber)
        })
    }
}

const fn value_role(kind: DxfPlanarFaceKind, group_code: i16) -> Option<DxfPlanarFaceValueRole> {
    use DxfPlanarFaceValueRole::{
        ExtrusionX, ExtrusionY, ExtrusionZ, FirstCornerX, FirstCornerY, FirstCornerZ,
        FourthCornerX, FourthCornerY, FourthCornerZ, InvisibleEdgeFlags, SecondCornerX,
        SecondCornerY, SecondCornerZ, Thickness, ThirdCornerX, ThirdCornerY, ThirdCornerZ,
    };
    match (kind, group_code) {
        (_, 10) => Some(FirstCornerX),
        (_, 20) => Some(FirstCornerY),
        (_, 30) => Some(FirstCornerZ),
        (_, 11) => Some(SecondCornerX),
        (_, 21) => Some(SecondCornerY),
        (_, 31) => Some(SecondCornerZ),
        (_, 12) => Some(ThirdCornerX),
        (_, 22) => Some(ThirdCornerY),
        (_, 32) => Some(ThirdCornerZ),
        (_, 13) => Some(FourthCornerX),
        (_, 23) => Some(FourthCornerY),
        (_, 33) => Some(FourthCornerZ),
        (DxfPlanarFaceKind::Solid | DxfPlanarFaceKind::Trace, 39) => Some(Thickness),
        (DxfPlanarFaceKind::Solid | DxfPlanarFaceKind::Trace, 210) => Some(ExtrusionX),
        (DxfPlanarFaceKind::Solid | DxfPlanarFaceKind::Trace, 220) => Some(ExtrusionY),
        (DxfPlanarFaceKind::Solid | DxfPlanarFaceKind::Trace, 230) => Some(ExtrusionZ),
        (DxfPlanarFaceKind::Face3d, 70) => Some(InvisibleEdgeFlags),
        (_, _) => None,
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

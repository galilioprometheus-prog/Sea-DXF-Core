//! Source-anchored documented INSERT defining-value evidence.

use std::io;

use crate::{
    ByteSpan, DxfApplicationGroupDirectory, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawGroup, DxfRawRecord, DxfRawRecordDirectory, DxfRawRecordSectionKind,
    DxfSourceId, DxfTextEncodingResolution, DxfTextValueDecodeReceipt,
    raw_double::decode_raw_double, raw_integer::decode_raw_i16,
};

/// Documented defining-value role retained from one exact INSERT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertRecordValueRole {
    BlockName,
    InsertionPointX,
    InsertionPointY,
    InsertionPointZ,
    ScaleFactorX,
    ScaleFactorY,
    ScaleFactorZ,
    RotationAngle,
    ColumnCount,
    RowCount,
    ColumnSpacing,
    RowSpacing,
    AttributesFollow,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
}

/// Borrowed-by-identity reference to one exact raw INSERT block-name value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordTextValue {
    source_id: DxfSourceId,
    group_occurrence: u32,
    value_span: ByteSpan,
    encoding: DxfTextEncodingResolution,
}

impl DxfInsertRecordTextValue {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn group_occurrence(self) -> u64 {
        self.group_occurrence as u64
    }

    #[must_use]
    pub const fn value_span(self) -> ByteSpan {
        self.value_span
    }

    #[must_use]
    pub const fn encoding(self) -> DxfTextEncodingResolution {
        self.encoding
    }

    /// Decodes from the original source without replacement or normalization.
    pub fn decode_to_utf8_without_replacement(
        self,
        document: DxfRawDocumentView<'_>,
        destination: &mut [u8],
    ) -> Result<DxfTextValueDecodeReceipt, DxfError> {
        if document.source_id() != self.source_id {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: document.source_id(),
            });
        }
        let receipt = document
            .decode_group_value_to_utf8_without_replacement(self.group_occurrence(), destination)?;
        if receipt.source_id() != self.source_id
            || receipt.group_occurrence() != self.group_occurrence()
            || receipt.value_span() != self.value_span
            || receipt.encoding() != self.encoding
        {
            return Err(invalid_internal_data());
        }
        Ok(receipt)
    }
}

/// Exact text, binary64, or signed-16-bit INSERT value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertRecordValueData {
    Text(DxfInsertRecordTextValue),
    Double(DxfDouble),
    Int16(i16),
}

/// Lexical reason why one INSERT number cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertRecordValueIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open defining-value range owned by one INSERT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordValueRange {
    start: u32,
    end: u32,
}

impl DxfInsertRecordValueRange {
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

/// One source-order documented group and its exact value evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordValue {
    group: DxfRawGroup,
    role: DxfInsertRecordValueRole,
    value: Result<DxfInsertRecordValueData, DxfInsertRecordValueIssue>,
}

impl DxfInsertRecordValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfInsertRecordValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfInsertRecordValueData, DxfInsertRecordValueIssue> {
        self.value
    }
}

/// One exact INSERT record and its source-order defining-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordValueEntry {
    record: DxfRawRecord,
    value_range: DxfInsertRecordValueRange,
}

impl DxfInsertRecordValueEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn value_range(self) -> DxfInsertRecordValueRange {
        self.value_range
    }
}

/// Immutable defining-value evidence directory for exact INSERT records.
#[derive(Debug)]
pub struct DxfInsertRecordValueDirectory {
    source_id: DxfSourceId,
    raw_records: DxfRawRecordDirectory,
    application_groups: DxfApplicationGroupDirectory,
    records: Box<[DxfInsertRecordValueEntry]>,
    values: Box<[DxfInsertRecordValue]>,
}

impl DxfInsertRecordValueDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        for observed in [
            raw_records.source_id(),
            application_groups.source_id(),
            document.text_encoding_report().source_id(),
        ] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut records = Vec::new();
        let mut values = Vec::new();
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !matches!(
                record.section_kind(),
                DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
            ) || !is_insert_record(document, record)?
            {
                continue;
            }

            let start = compact_len(values.len())?;
            for occurrence in
                record.marker_occurrence().saturating_add(1)..record.group_range().end()
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
                let Some(role) = value_role(group.group_code().value()) else {
                    continue;
                };
                let value = decode_value(document, group, role, cancellation)?;
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfInsertRecordValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfInsertRecordValueEntry {
                record,
                value_range: DxfInsertRecordValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            raw_records,
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
    pub const fn raw_record_directory(&self) -> &DxfRawRecordDirectory {
        &self.raw_records
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        &self.application_groups
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInsertRecordValueEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfInsertRecordValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertRecordValueEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfInsertRecordValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfInsertRecordValue> {
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
    pub fn insert_record_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordValueDirectory, DxfError> {
        DxfInsertRecordValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_record_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_record_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_record_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_record_value_directory(cancellation)
    }
}

fn is_insert_record(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<bool, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    document.raw_span_equals_exact(marker.value_payload_span(), b"INSERT")
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfInsertRecordValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfInsertRecordValueData, DxfInsertRecordValueIssue>, DxfError> {
    match wire_kind(role) {
        DxfInsertRecordWireKind::Text => {
            let group_occurrence =
                u32::try_from(group.occurrence()).map_err(|_| invalid_internal_data())?;
            Ok(Ok(DxfInsertRecordValueData::Text(
                DxfInsertRecordTextValue {
                    source_id: document.source_id(),
                    group_occurrence,
                    value_span: group.value_payload_span(),
                    encoding: document.text_encoding_report().resolution(),
                },
            )))
        }
        DxfInsertRecordWireKind::Double => {
            decode_raw_double(document, group, cancellation).map(|value| {
                value
                    .map(DxfInsertRecordValueData::Double)
                    .map_err(DxfInsertRecordValueIssue::InvalidAsciiNumber)
            })
        }
        DxfInsertRecordWireKind::Int16 => {
            decode_raw_i16(document, group, cancellation).map(|value| {
                value
                    .map(DxfInsertRecordValueData::Int16)
                    .map_err(DxfInsertRecordValueIssue::InvalidAsciiNumber)
            })
        }
    }
}

#[derive(Clone, Copy)]
enum DxfInsertRecordWireKind {
    Text,
    Double,
    Int16,
}

const fn wire_kind(role: DxfInsertRecordValueRole) -> DxfInsertRecordWireKind {
    use DxfInsertRecordValueRole::{
        AttributesFollow, BlockName, ColumnCount, ColumnSpacing, ExtrusionX, ExtrusionY,
        ExtrusionZ, InsertionPointX, InsertionPointY, InsertionPointZ, RotationAngle, RowCount,
        RowSpacing, ScaleFactorX, ScaleFactorY, ScaleFactorZ,
    };
    match role {
        BlockName => DxfInsertRecordWireKind::Text,
        InsertionPointX | InsertionPointY | InsertionPointZ | ScaleFactorX | ScaleFactorY
        | ScaleFactorZ | RotationAngle | ColumnSpacing | RowSpacing | ExtrusionX | ExtrusionY
        | ExtrusionZ => DxfInsertRecordWireKind::Double,
        AttributesFollow | ColumnCount | RowCount => DxfInsertRecordWireKind::Int16,
    }
}

const fn value_role(group_code: i16) -> Option<DxfInsertRecordValueRole> {
    use DxfInsertRecordValueRole::{
        AttributesFollow, BlockName, ColumnCount, ColumnSpacing, ExtrusionX, ExtrusionY,
        ExtrusionZ, InsertionPointX, InsertionPointY, InsertionPointZ, RotationAngle, RowCount,
        RowSpacing, ScaleFactorX, ScaleFactorY, ScaleFactorZ,
    };
    match group_code {
        2 => Some(BlockName),
        10 => Some(InsertionPointX),
        20 => Some(InsertionPointY),
        30 => Some(InsertionPointZ),
        41 => Some(ScaleFactorX),
        42 => Some(ScaleFactorY),
        43 => Some(ScaleFactorZ),
        50 => Some(RotationAngle),
        70 => Some(ColumnCount),
        71 => Some(RowCount),
        44 => Some(ColumnSpacing),
        45 => Some(RowSpacing),
        66 => Some(AttributesFollow),
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

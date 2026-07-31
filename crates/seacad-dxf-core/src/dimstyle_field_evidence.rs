//! Typed, source-anchored values from exact named DIMSTYLE records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDimStyleField, DxfDimStyleTableDirectory, DxfDimStyleTableEntry, DxfDimStyleWireKind,
    DxfDouble, DxfError, DxfHandle, DxfHandleParseIssue, DxfIoOperation, DxfRawDocumentView,
    DxfRawGroup, DxfRawValueProvenance, DxfSourceId, dimstyle_field::field_for_group_code,
    raw_double::decode_raw_double, raw_handle::parse_raw_group_handle, raw_integer::decode_raw_i16,
};

/// Exact typed payload retained from one DIMSTYLE field occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleValueData {
    Text(DxfRawValueProvenance),
    Double(DxfDouble),
    Int16(i16),
    Handle(DxfHandle),
}

/// Lexical reason why one DIMSTYLE field cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleValueIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    InvalidHandle(DxfHandleParseIssue),
}

/// Half-open value range owned by one exact named DIMSTYLE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleValueRange {
    start: u32,
    end: u32,
}

impl DxfDimStyleValueRange {
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

/// One source-order DIMSTYLE-specific group and its typed value evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleValue {
    group: DxfRawGroup,
    field: DxfDimStyleField,
    value: Result<DxfDimStyleValueData, DxfDimStyleValueIssue>,
}

impl DxfDimStyleValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn field(self) -> DxfDimStyleField {
        self.field
    }

    pub const fn value(self) -> Result<DxfDimStyleValueData, DxfDimStyleValueIssue> {
        self.value
    }
}

/// One admitted table record and its DIMSTYLE-specific value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleValueEntry {
    table_entry: DxfDimStyleTableEntry,
    value_range: DxfDimStyleValueRange,
}

impl DxfDimStyleValueEntry {
    #[must_use]
    pub const fn table_entry(self) -> DxfDimStyleTableEntry {
        self.table_entry
    }

    #[must_use]
    pub const fn value_range(self) -> DxfDimStyleValueRange {
        self.value_range
    }
}

/// Immutable typed evidence for documented DIMSTYLE-specific fields.
#[derive(Debug)]
pub struct DxfDimStyleValueDirectory {
    source_id: DxfSourceId,
    table: DxfDimStyleTableDirectory,
    records: Box<[DxfDimStyleValueEntry]>,
    values: Box<[DxfDimStyleValue]>,
}

impl DxfDimStyleValueDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let table = document.dimstyle_table_directory(cancellation)?;
        if table.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: table.source_id(),
            });
        }

        let mut records = Vec::new();
        let mut values = Vec::new();
        for table_entry in table.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let record = table_entry.record();
            let start = compact_len(values.len())?;
            for occurrence in
                record.marker_occurrence().saturating_add(1)..record.group_range().end()
            {
                ensure_not_cancelled(cancellation)?;
                if table
                    .application_group_directory()
                    .group_for_content_occurrence(occurrence)
                    .is_some()
                {
                    continue;
                }
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                let Some(field) = field_for_group_code(group.group_code().value()) else {
                    continue;
                };
                let value = decode_value(document, group, field, cancellation)?;
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfDimStyleValue {
                    group,
                    field,
                    value,
                });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfDimStyleValueEntry {
                table_entry,
                value_range: DxfDimStyleValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            table,
            records: records.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn table_directory(&self) -> &DxfDimStyleTableDirectory {
        &self.table
    }

    #[must_use]
    pub fn records(&self) -> &[DxfDimStyleValueEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfDimStyleValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(&self, raw_ordinal: u64) -> Option<DxfDimStyleValueEntry> {
        self.records
            .binary_search_by_key(&raw_ordinal, |entry| entry.table_entry().record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_ordinal(&self, raw_ordinal: u64) -> Option<&[DxfDimStyleValue]> {
        let entry = self.record_for_raw_ordinal(raw_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfDimStyleValue> {
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
    pub fn dimstyle_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleValueDirectory, DxfError> {
        DxfDimStyleValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn dimstyle_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn dimstyle_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_value_directory(cancellation)
    }
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    field: DxfDimStyleField,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfDimStyleValueData, DxfDimStyleValueIssue>, DxfError> {
    match field.wire_kind() {
        DxfDimStyleWireKind::Text => Ok(Ok(DxfDimStyleValueData::Text(
            DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
                .ok_or_else(invalid_internal_data)?,
        ))),
        DxfDimStyleWireKind::Double => Ok(decode_raw_double(document, group, cancellation)?
            .map(DxfDimStyleValueData::Double)
            .map_err(DxfDimStyleValueIssue::InvalidAsciiNumber)),
        DxfDimStyleWireKind::Int16 => Ok(decode_raw_i16(document, group, cancellation)?
            .map(DxfDimStyleValueData::Int16)
            .map_err(DxfDimStyleValueIssue::InvalidAsciiNumber)),
        DxfDimStyleWireKind::Handle => Ok(parse_raw_group_handle(document, group, cancellation)?
            .map(DxfDimStyleValueData::Handle)
            .map_err(DxfDimStyleValueIssue::InvalidHandle)),
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

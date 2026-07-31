//! Exact MTEXT column evidence filed after a group-101 embedded-object marker.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawGroup, DxfRawRecord, DxfSourceId, DxfTextSymbolKind,
    DxfTextSymbolNumericIssue, DxfTextSymbolValueData, raw_double::decode_raw_double,
    raw_integer::decode_raw_i16,
};

/// Source role observed in AutoCAD's MTEXT embedded column object.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextEmbeddedColumnRole {
    Version,
    SharedHeight,
    ColumnHeight,
    ColumnType,
    ColumnCount,
    ColumnWidth,
    ColumnGutter,
    ColumnAutoHeight,
    ColumnFlowReversed,
}

/// One source-order value owned by an MTEXT embedded column object.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextEmbeddedColumnValue {
    group: DxfRawGroup,
    role: DxfMTextEmbeddedColumnRole,
    data: DxfTextSymbolValueData,
}

impl DxfMTextEmbeddedColumnValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfMTextEmbeddedColumnRole {
        self.role
    }

    #[must_use]
    pub const fn data(self) -> DxfTextSymbolValueData {
        self.data
    }
}

/// One exact embedded-object marker and its retained column-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextEmbeddedColumnEntry {
    record: DxfRawRecord,
    marker: DxfRawGroup,
    value_start: u32,
    value_end: u32,
}

impl DxfMTextEmbeddedColumnEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn marker(self) -> DxfRawGroup {
        self.marker
    }

    #[must_use]
    pub const fn value_count(self) -> u64 {
        (self.value_end - self.value_start) as u64
    }
}

/// Immutable source evidence for modern MTEXT embedded column objects.
#[derive(Debug)]
pub struct DxfMTextEmbeddedColumnDirectory {
    source_id: DxfSourceId,
    entries: Box<[DxfMTextEmbeddedColumnEntry]>,
    values: Box<[DxfMTextEmbeddedColumnValue]>,
}

impl DxfMTextEmbeddedColumnDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let text = document.text_symbol_directory(cancellation)?;
        if text.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: text.source_id(),
            });
        }
        let mut entries = Vec::new();
        let mut values = Vec::new();
        for text_record in text.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if text_record.kind() != DxfTextSymbolKind::MText {
                continue;
            }
            append_record_entries(
                document,
                text_record.record(),
                cancellation,
                &mut entries,
                &mut values,
            )?;
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entries: entries.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfMTextEmbeddedColumnEntry] {
        &self.entries
    }

    #[must_use]
    pub fn values(&self) -> &[DxfMTextEmbeddedColumnValue] {
        &self.values
    }

    #[must_use]
    pub fn values_for_entry(
        &self,
        entry: DxfMTextEmbeddedColumnEntry,
    ) -> Option<&[DxfMTextEmbeddedColumnValue]> {
        self.values
            .get(usize::try_from(entry.value_start).ok()?..usize::try_from(entry.value_end).ok()?)
    }

    #[must_use]
    pub fn entry_for_raw_record(&self, raw: u64) -> Option<DxfMTextEmbeddedColumnEntry> {
        self.entries
            .binary_search_by_key(&raw, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_embedded_column_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextEmbeddedColumnDirectory, DxfError> {
        DxfMTextEmbeddedColumnDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_embedded_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextEmbeddedColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_embedded_column_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_embedded_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextEmbeddedColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_embedded_column_directory(cancellation)
    }
}

fn append_record_entries(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    cancellation: &DxfCancellationToken,
    entries: &mut Vec<DxfMTextEmbeddedColumnEntry>,
    values: &mut Vec<DxfMTextEmbeddedColumnValue>,
) -> Result<(), DxfError> {
    let mut marker = None;
    let mut value_start = compact_len(values.len())?;
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
        ensure_not_cancelled(cancellation)?;
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if is_embedded_marker(document, group)? {
            if let Some(previous) = marker {
                append_entry(record, previous, value_start, entries, values)?;
            }
            marker = Some(group);
            value_start = compact_len(values.len())?;
            continue;
        }
        if marker.is_none() {
            continue;
        }
        let Some((role, wire)) = embedded_role(group.group_code().value()) else {
            continue;
        };
        let data = decode_value(document, group, wire, cancellation)?;
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(DxfMTextEmbeddedColumnValue { group, role, data });
    }
    if let Some(last) = marker {
        append_entry(record, last, value_start, entries, values)?;
    }
    Ok(())
}

fn append_entry(
    record: DxfRawRecord,
    marker: DxfRawGroup,
    value_start: u32,
    entries: &mut Vec<DxfMTextEmbeddedColumnEntry>,
    values: &[DxfMTextEmbeddedColumnValue],
) -> Result<(), DxfError> {
    entries.try_reserve(1).map_err(|_| out_of_memory())?;
    entries.push(DxfMTextEmbeddedColumnEntry {
        record,
        marker,
        value_start,
        value_end: compact_len(values.len())?,
    });
    Ok(())
}

fn is_embedded_marker(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
) -> Result<bool, DxfError> {
    Ok(group.group_code().value() == 101
        && document.raw_span_equals_exact(group.value_payload_span(), b"Embedded Object")?)
}

#[derive(Clone, Copy)]
enum Wire {
    Double,
    Int16,
}

const fn embedded_role(code: i16) -> Option<(DxfMTextEmbeddedColumnRole, Wire)> {
    use DxfMTextEmbeddedColumnRole as R;
    match code {
        70 => Some((R::Version, Wire::Int16)),
        41 => Some((R::SharedHeight, Wire::Double)),
        46 => Some((R::ColumnHeight, Wire::Double)),
        71 => Some((R::ColumnType, Wire::Int16)),
        72 => Some((R::ColumnCount, Wire::Int16)),
        44 => Some((R::ColumnWidth, Wire::Double)),
        45 => Some((R::ColumnGutter, Wire::Double)),
        73 => Some((R::ColumnAutoHeight, Wire::Int16)),
        74 => Some((R::ColumnFlowReversed, Wire::Int16)),
        _ => None,
    }
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    wire: Wire,
    cancellation: &DxfCancellationToken,
) -> Result<DxfTextSymbolValueData, DxfError> {
    let issue = DxfTextSymbolNumericIssue::InvalidAsciiNumber;
    match wire {
        Wire::Double => decode_raw_double(document, group, cancellation)
            .map(|value| DxfTextSymbolValueData::Double(value.map_err(issue))),
        Wire::Int16 => decode_raw_i16(document, group, cancellation)
            .map(|value| DxfTextSymbolValueData::Int16(value.map_err(issue))),
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

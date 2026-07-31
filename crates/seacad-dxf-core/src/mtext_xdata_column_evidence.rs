//! R2007-era MTEXT column evidence stored in ACAD XDATA.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawGroup, DxfRawRecord, DxfSourceId, DxfTextSymbolKind,
    DxfTextSymbolNumericIssue, DxfTextSymbolValueData, raw_double::decode_raw_double,
    raw_integer::decode_raw_i16,
};

const APP_NAME: &[u8] = b"ACAD";
const BEGIN: &[u8] = b"ACAD_MTEXT_COLUMN_INFO_BEGIN";
const END: &[u8] = b"ACAD_MTEXT_COLUMN_INFO_END";

/// Role encoded by a field identifier inside MTEXT column-info XDATA.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextXDataColumnRole {
    ColumnType,
    ColumnAutoHeight,
    ColumnCount,
    ColumnFlowReversed,
    ColumnWidth,
    ColumnGutter,
    ColumnHeightCount,
    ColumnHeight,
}

/// One source-order value decoded from an ACAD MTEXT column-info block.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextXDataColumnValue {
    field_id_group: DxfRawGroup,
    value_group: DxfRawGroup,
    role: DxfMTextXDataColumnRole,
    data: DxfTextSymbolValueData,
}

impl DxfMTextXDataColumnValue {
    #[must_use]
    pub const fn field_id_group(self) -> DxfRawGroup {
        self.field_id_group
    }

    #[must_use]
    pub const fn value_group(self) -> DxfRawGroup {
        self.value_group
    }

    #[must_use]
    pub const fn role(self) -> DxfMTextXDataColumnRole {
        self.role
    }

    #[must_use]
    pub const fn data(self) -> DxfTextSymbolValueData {
        self.data
    }
}

/// One complete exact ACAD MTEXT column-info XDATA block.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextXDataColumnEntry {
    record: DxfRawRecord,
    begin: DxfRawGroup,
    end: DxfRawGroup,
    value_start: u32,
    value_end: u32,
}

impl DxfMTextXDataColumnEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn begin(self) -> DxfRawGroup {
        self.begin
    }

    #[must_use]
    pub const fn end(self) -> DxfRawGroup {
        self.end
    }

    #[must_use]
    pub const fn value_count(self) -> u64 {
        (self.value_end - self.value_start) as u64
    }
}

/// Immutable R2007-era ACAD XDATA column evidence.
#[derive(Debug)]
pub struct DxfMTextXDataColumnDirectory {
    source_id: DxfSourceId,
    entries: Box<[DxfMTextXDataColumnEntry]>,
    values: Box<[DxfMTextXDataColumnValue]>,
}

impl DxfMTextXDataColumnDirectory {
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
            if text_record.kind() == DxfTextSymbolKind::MText {
                append_record(
                    document,
                    text_record.record(),
                    cancellation,
                    &mut entries,
                    &mut values,
                )?;
            }
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
    pub fn entries(&self) -> &[DxfMTextXDataColumnEntry] {
        &self.entries
    }

    #[must_use]
    pub fn values(&self) -> &[DxfMTextXDataColumnValue] {
        &self.values
    }

    #[must_use]
    pub fn values_for_entry(
        &self,
        entry: DxfMTextXDataColumnEntry,
    ) -> Option<&[DxfMTextXDataColumnValue]> {
        self.values
            .get(usize::try_from(entry.value_start).ok()?..usize::try_from(entry.value_end).ok()?)
    }

    #[must_use]
    pub fn entry_for_begin_occurrence(&self, occurrence: u64) -> Option<DxfMTextXDataColumnEntry> {
        self.entries
            .binary_search_by_key(&occurrence, |entry| entry.begin().occurrence())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_xdata_column_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataColumnDirectory, DxfError> {
        DxfMTextXDataColumnDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_xdata_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_xdata_column_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_xdata_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_xdata_column_directory(cancellation)
    }
}

fn append_record(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    cancellation: &DxfCancellationToken,
    entries: &mut Vec<DxfMTextXDataColumnEntry>,
    values: &mut Vec<DxfMTextXDataColumnValue>,
) -> Result<(), DxfError> {
    let mut acad_scope = false;
    let mut occurrence = record.marker_occurrence().saturating_add(1);
    while occurrence < record.group_range().end() {
        ensure_not_cancelled(cancellation)?;
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if group.group_code().value() == 1001 {
            acad_scope = document.raw_span_equals_exact(group.value_payload_span(), APP_NAME)?;
        } else if acad_scope && exact_text(document, group, 1000, BEGIN)? {
            occurrence = append_block(
                document,
                record,
                group,
                occurrence.saturating_add(1),
                cancellation,
                entries,
                values,
            )?;
            continue;
        }
        occurrence = occurrence.saturating_add(1);
    }
    Ok(())
}

fn append_block(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    begin: DxfRawGroup,
    mut occurrence: u64,
    cancellation: &DxfCancellationToken,
    entries: &mut Vec<DxfMTextXDataColumnEntry>,
    values: &mut Vec<DxfMTextXDataColumnValue>,
) -> Result<u64, DxfError> {
    let value_start = values.len();
    while occurrence < record.group_range().end() {
        ensure_not_cancelled(cancellation)?;
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if exact_text(document, group, 1000, END)? {
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfMTextXDataColumnEntry {
                record,
                begin,
                end: group,
                value_start: compact_len(value_start)?,
                value_end: compact_len(values.len())?,
            });
            return Ok(occurrence.saturating_add(1));
        }
        if group.group_code().value() == 1001 {
            values.truncate(value_start);
            return Ok(occurrence);
        }
        if group.group_code().value() != 1070 {
            occurrence = occurrence.saturating_add(1);
            continue;
        }
        let selector = decode_raw_i16(document, group, cancellation)?;
        let Ok(selector) = selector else {
            occurrence = occurrence.saturating_add(1);
            continue;
        };
        occurrence = append_field(
            document,
            group,
            selector,
            occurrence.saturating_add(1),
            record.group_range().end(),
            cancellation,
            values,
        )?;
    }
    values.truncate(value_start);
    Ok(occurrence)
}

fn append_field(
    document: DxfRawDocumentView<'_>,
    field_id_group: DxfRawGroup,
    selector: i16,
    occurrence: u64,
    record_end: u64,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfMTextXDataColumnValue>,
) -> Result<u64, DxfError> {
    let Some((role, wire)) = field_role(selector) else {
        return Ok(occurrence.saturating_add(1));
    };
    let Some(value_group) = document.group(occurrence) else {
        return Ok(record_end);
    };
    if value_group.group_code().value() != wire.code() {
        return Ok(occurrence.saturating_add(1));
    }
    append_value(
        document,
        field_id_group,
        value_group,
        role,
        wire,
        cancellation,
        values,
    )?;
    let mut next = occurrence.saturating_add(1);
    if role != DxfMTextXDataColumnRole::ColumnHeightCount {
        return Ok(next);
    }
    let height_count = match values.last().map(|value| value.data()) {
        Some(DxfTextSymbolValueData::Int16(Ok(count))) if count >= 0 => count as u16,
        _ => return Ok(next),
    };
    for _ in 0..height_count {
        let Some(height) = document.group(next) else {
            return Ok(record_end);
        };
        if height.group_code().value() != 1040 {
            break;
        }
        append_value(
            document,
            field_id_group,
            height,
            DxfMTextXDataColumnRole::ColumnHeight,
            Wire::Double,
            cancellation,
            values,
        )?;
        next = next.saturating_add(1);
    }
    Ok(next)
}

fn append_value(
    document: DxfRawDocumentView<'_>,
    field_id_group: DxfRawGroup,
    value_group: DxfRawGroup,
    role: DxfMTextXDataColumnRole,
    wire: Wire,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfMTextXDataColumnValue>,
) -> Result<(), DxfError> {
    let issue = DxfTextSymbolNumericIssue::InvalidAsciiNumber;
    let data = match wire {
        Wire::Int16 => decode_raw_i16(document, value_group, cancellation)
            .map(|value| DxfTextSymbolValueData::Int16(value.map_err(issue)))?,
        Wire::Double => decode_raw_double(document, value_group, cancellation)
            .map(|value| DxfTextSymbolValueData::Double(value.map_err(issue)))?,
    };
    values.try_reserve(1).map_err(|_| out_of_memory())?;
    values.push(DxfMTextXDataColumnValue {
        field_id_group,
        value_group,
        role,
        data,
    });
    Ok(())
}

#[derive(Clone, Copy)]
enum Wire {
    Int16,
    Double,
}

impl Wire {
    const fn code(self) -> i16 {
        match self {
            Self::Int16 => 1070,
            Self::Double => 1040,
        }
    }
}

const fn field_role(selector: i16) -> Option<(DxfMTextXDataColumnRole, Wire)> {
    use DxfMTextXDataColumnRole as R;
    match selector {
        75 => Some((R::ColumnType, Wire::Int16)),
        79 => Some((R::ColumnAutoHeight, Wire::Int16)),
        76 => Some((R::ColumnCount, Wire::Int16)),
        78 => Some((R::ColumnFlowReversed, Wire::Int16)),
        48 => Some((R::ColumnWidth, Wire::Double)),
        49 => Some((R::ColumnGutter, Wire::Double)),
        50 => Some((R::ColumnHeightCount, Wire::Int16)),
        _ => None,
    }
}

fn exact_text(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    code: i16,
    text: &[u8],
) -> Result<bool, DxfError> {
    Ok(group.group_code().value() == code
        && document.raw_span_equals_exact(group.value_payload_span(), text)?)
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

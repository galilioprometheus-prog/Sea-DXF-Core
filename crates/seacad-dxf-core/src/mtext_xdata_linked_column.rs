//! Exact R2007-era MTEXT linked-column XDATA evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandleGroupClass,
    DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawHandleLookup, DxfRawHandleValue,
    DxfRawRecord, DxfSourceId, DxfTextSymbolKind, DxfTextSymbolNumericIssue,
    raw_integer::decode_raw_i16,
};

const APP_NAME: &[u8] = b"ACAD";
const BEGIN: &[u8] = b"ACAD_MTEXT_COLUMNS_BEGIN";
const END: &[u8] = b"ACAD_MTEXT_COLUMNS_END";
const COLUMN_COUNT_SELECTOR: i16 = 47;

/// One complete exact linked-column XDATA block and its handle slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextXDataLinkedColumnEntry {
    record: DxfRawRecord,
    begin: DxfRawGroup,
    selector_group: DxfRawGroup,
    count_group: DxfRawGroup,
    declared_column_count: Result<i16, DxfTextSymbolNumericIssue>,
    end: DxfRawGroup,
    handle_start: u32,
    handle_end: u32,
}

impl DxfMTextXDataLinkedColumnEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn begin(self) -> DxfRawGroup {
        self.begin
    }

    #[must_use]
    pub const fn selector_group(self) -> DxfRawGroup {
        self.selector_group
    }

    #[must_use]
    pub const fn count_group(self) -> DxfRawGroup {
        self.count_group
    }

    pub const fn declared_column_count(self) -> Result<i16, DxfTextSymbolNumericIssue> {
        self.declared_column_count
    }

    #[must_use]
    pub const fn end(self) -> DxfRawGroup {
        self.end
    }

    #[must_use]
    pub const fn linked_handle_count(self) -> u64 {
        (self.handle_end - self.handle_start) as u64
    }
}

/// Immutable exact linked-column handle evidence for R2007-era MTEXT.
#[derive(Debug)]
pub struct DxfMTextXDataLinkedColumnDirectory {
    source_id: DxfSourceId,
    entries: Box<[DxfMTextXDataLinkedColumnEntry]>,
    handles: Box<[DxfRawHandleValue]>,
}

impl DxfMTextXDataLinkedColumnDirectory {
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
        let mut handles = Vec::new();
        for text_record in text.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if text_record.kind() == DxfTextSymbolKind::MText {
                append_record(
                    document,
                    text_record.record(),
                    cancellation,
                    &mut entries,
                    &mut handles,
                )?;
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entries: entries.into_boxed_slice(),
            handles: handles.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfMTextXDataLinkedColumnEntry] {
        &self.entries
    }

    #[must_use]
    pub fn handles(&self) -> &[DxfRawHandleValue] {
        &self.handles
    }

    #[must_use]
    pub fn handles_for_entry(
        &self,
        entry: DxfMTextXDataLinkedColumnEntry,
    ) -> Option<&[DxfRawHandleValue]> {
        self.handles.get(
            usize::try_from(entry.handle_start).ok()?..usize::try_from(entry.handle_end).ok()?,
        )
    }

    #[must_use]
    pub fn entry_for_begin_occurrence(
        &self,
        occurrence: u64,
    ) -> Option<DxfMTextXDataLinkedColumnEntry> {
        self.entries
            .binary_search_by_key(&occurrence, |entry| entry.begin().occurrence())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_xdata_linked_column_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataLinkedColumnDirectory, DxfError> {
        DxfMTextXDataLinkedColumnDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_xdata_linked_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataLinkedColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_xdata_linked_column_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_xdata_linked_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXDataLinkedColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_xdata_linked_column_directory(cancellation)
    }
}

fn append_record(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    cancellation: &DxfCancellationToken,
    entries: &mut Vec<DxfMTextXDataLinkedColumnEntry>,
    handles: &mut Vec<DxfRawHandleValue>,
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
                handles,
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
    occurrence: u64,
    cancellation: &DxfCancellationToken,
    entries: &mut Vec<DxfMTextXDataLinkedColumnEntry>,
    handles: &mut Vec<DxfRawHandleValue>,
) -> Result<u64, DxfError> {
    let record_end = record.group_range().end();
    let Some(selector_group) = group_in_record(document, occurrence, record_end) else {
        return Ok(record_end);
    };
    if selector_group.group_code().value() != 1070
        || decode_raw_i16(document, selector_group, cancellation)? != Ok(COLUMN_COUNT_SELECTOR)
    {
        return Ok(occurrence.saturating_add(1));
    }

    let count_occurrence = occurrence.saturating_add(1);
    let Some(count_group) = group_in_record(document, count_occurrence, record_end) else {
        return Ok(record_end);
    };
    if count_group.group_code().value() != 1070 {
        return Ok(count_occurrence.saturating_add(1));
    }
    let issue = DxfTextSymbolNumericIssue::InvalidAsciiNumber;
    let declared_column_count = decode_raw_i16(document, count_group, cancellation)?.map_err(issue);

    let handle_start = handles.len();
    let mut next = count_occurrence.saturating_add(1);
    while next < record_end {
        ensure_not_cancelled(cancellation)?;
        let group = document.group(next).ok_or_else(invalid_internal_data)?;
        if exact_text(document, group, 1000, END)? {
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfMTextXDataLinkedColumnEntry {
                record,
                begin,
                selector_group,
                count_group,
                declared_column_count,
                end: group,
                handle_start: compact_len(handle_start)?,
                handle_end: compact_len(handles.len())?,
            });
            return Ok(next.saturating_add(1));
        }
        if group.group_code().value() == 1001 {
            handles.truncate(handle_start);
            return Ok(next);
        }
        if group.group_code().value() != 1005 {
            handles.truncate(handle_start);
            return Ok(next.saturating_add(1));
        }
        let DxfRawHandleLookup::Handle(handle) = document.raw_handle_at(next, cancellation)? else {
            return Err(invalid_internal_data());
        };
        if handle.class() != DxfHandleGroupClass::SoftPointer {
            return Err(invalid_internal_data());
        }
        handles.try_reserve(1).map_err(|_| out_of_memory())?;
        handles.push(handle);
        next = next.saturating_add(1);
    }
    handles.truncate(handle_start);
    Ok(record_end)
}

fn group_in_record(
    document: DxfRawDocumentView<'_>,
    occurrence: u64,
    record_end: u64,
) -> Option<DxfRawGroup> {
    (occurrence < record_end)
        .then(|| document.group(occurrence))
        .flatten()
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

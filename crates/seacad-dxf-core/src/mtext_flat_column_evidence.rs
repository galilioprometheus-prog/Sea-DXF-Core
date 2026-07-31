//! Direct MTEXT column-field evidence before any embedded-object boundary.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawGroup, DxfRawRecord, DxfSourceId, DxfTextSymbolDirectory,
    DxfTextSymbolKind, DxfTextSymbolValue, DxfTextSymbolValueData, DxfTextSymbolValueRole,
};

/// Role retained from Autodesk's direct MTEXT column group registry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextFlatColumnRole {
    ColumnType,
    ColumnCount,
    ColumnFlowReversed,
    ColumnAutoHeight,
    ColumnWidth,
    ColumnGutter,
    RotationOrColumnHeight,
}

/// One source-order direct column-related value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextFlatColumnValue {
    group: DxfRawGroup,
    role: DxfMTextFlatColumnRole,
    data: DxfTextSymbolValueData,
}

impl DxfMTextFlatColumnValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfMTextFlatColumnRole {
        self.role
    }

    #[must_use]
    pub const fn data(self) -> DxfTextSymbolValueData {
        self.data
    }
}

/// One MTEXT record with at least one unambiguous direct column field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextFlatColumnEntry {
    record: DxfRawRecord,
    marker: DxfRawGroup,
    value_start: u32,
    value_end: u32,
}

impl DxfMTextFlatColumnEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    /// First unambiguous direct column field in source order.
    #[must_use]
    pub const fn marker(self) -> DxfRawGroup {
        self.marker
    }

    #[must_use]
    pub const fn value_count(self) -> u64 {
        (self.value_end - self.value_start) as u64
    }
}

/// Immutable direct MTEXT column evidence retaining the underlying text index.
#[derive(Debug)]
pub struct DxfMTextFlatColumnDirectory {
    source_id: DxfSourceId,
    text: DxfTextSymbolDirectory,
    entries: Box<[DxfMTextFlatColumnEntry]>,
    values: Box<[DxfMTextFlatColumnValue]>,
}

impl DxfMTextFlatColumnDirectory {
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
        for record in text.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if record.kind() != DxfTextSymbolKind::MText {
                continue;
            }
            let record_values = text
                .values_for_raw_record(record.record().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let Some(marker) = record_values.iter().copied().find_map(definite_marker) else {
                continue;
            };
            let value_start = compact_len(values.len())?;
            for value in record_values.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let Some(role) = flat_role(value.role()) else {
                    continue;
                };
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfMTextFlatColumnValue {
                    group: value.group(),
                    role,
                    data: value.data(),
                });
            }
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfMTextFlatColumnEntry {
                record: record.record(),
                marker,
                value_start,
                value_end: compact_len(values.len())?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            text,
            entries: entries.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn text_directory(&self) -> &DxfTextSymbolDirectory {
        &self.text
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfMTextFlatColumnEntry] {
        &self.entries
    }

    #[must_use]
    pub fn values(&self) -> &[DxfMTextFlatColumnValue] {
        &self.values
    }

    #[must_use]
    pub fn values_for_entry(
        &self,
        entry: DxfMTextFlatColumnEntry,
    ) -> Option<&[DxfMTextFlatColumnValue]> {
        self.values
            .get(usize::try_from(entry.value_start).ok()?..usize::try_from(entry.value_end).ok()?)
    }

    #[must_use]
    pub fn entry_for_record_ordinal(&self, ordinal: u64) -> Option<DxfMTextFlatColumnEntry> {
        self.entries
            .binary_search_by_key(&ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_flat_column_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextFlatColumnDirectory, DxfError> {
        DxfMTextFlatColumnDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_flat_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextFlatColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_flat_column_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_flat_column_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextFlatColumnDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_flat_column_directory(cancellation)
    }
}

fn definite_marker(value: DxfTextSymbolValue) -> Option<DxfRawGroup> {
    (!matches!(value.role(), DxfTextSymbolValueRole::RotationOrColumnHeight)
        && flat_role(value.role()).is_some())
    .then(|| value.group())
}

const fn flat_role(role: DxfTextSymbolValueRole) -> Option<DxfMTextFlatColumnRole> {
    use DxfMTextFlatColumnRole as R;
    match role {
        DxfTextSymbolValueRole::ColumnType => Some(R::ColumnType),
        DxfTextSymbolValueRole::ColumnCount => Some(R::ColumnCount),
        DxfTextSymbolValueRole::ColumnFlowReversed => Some(R::ColumnFlowReversed),
        DxfTextSymbolValueRole::ColumnAutoHeight => Some(R::ColumnAutoHeight),
        DxfTextSymbolValueRole::ColumnWidth => Some(R::ColumnWidth),
        DxfTextSymbolValueRole::ColumnGutter => Some(R::ColumnGutter),
        DxfTextSymbolValueRole::RotationOrColumnHeight => Some(R::RotationOrColumnHeight),
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

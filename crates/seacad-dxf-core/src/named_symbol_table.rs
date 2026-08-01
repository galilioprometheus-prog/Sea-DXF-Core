//! Exact named records from completely closed reviewed symbol tables.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawRecord, DxfRawRecordDirectory,
    DxfRawRecordSectionKind, DxfRawValueProvenance, DxfSourceId,
};

/// Reviewed exact symbol-table family.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfNamedSymbolTableKind {
    DimStyle,
    Style,
    BlockRecord,
    Layer,
    Linetype,
}

impl DxfNamedSymbolTableKind {
    const fn marker(self) -> &'static [u8] {
        match self {
            Self::DimStyle => b"DIMSTYLE",
            Self::Style => b"STYLE",
            Self::BlockRecord => b"BLOCK_RECORD",
            Self::Layer => b"LAYER",
            Self::Linetype => b"LTYPE",
        }
    }
}

const TABLE_KINDS: [DxfNamedSymbolTableKind; 5] = [
    DxfNamedSymbolTableKind::DimStyle,
    DxfNamedSymbolTableKind::Style,
    DxfNamedSymbolTableKind::BlockRecord,
    DxfNamedSymbolTableKind::Layer,
    DxfNamedSymbolTableKind::Linetype,
];

/// One exact uniquely named record admitted from a matching closed table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfNamedSymbolTableEntry {
    record: DxfRawRecord,
    kind: DxfNamedSymbolTableKind,
    name: DxfRawValueProvenance,
}

impl DxfNamedSymbolTableEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfNamedSymbolTableKind {
        self.kind
    }

    #[must_use]
    pub const fn name(self) -> DxfRawValueProvenance {
        self.name
    }
}

/// Exact named records admitted from five reviewed symbol-table families.
#[derive(Debug)]
pub struct DxfNamedSymbolTableDirectory {
    source_id: DxfSourceId,
    raw_records: DxfRawRecordDirectory,
    application_groups: DxfApplicationGroupDirectory,
    entries: Box<[DxfNamedSymbolTableEntry]>,
}

impl DxfNamedSymbolTableDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        for observed in [raw_records.source_id(), application_groups.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        let mut pending = Vec::new();
        let mut active = None;
        let mut section = None;
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if record.section_kind() != DxfRawRecordSectionKind::Tables {
                continue;
            }
            if section != Some(record.structure_section_ordinal()) {
                pending.clear();
                active = None;
                section = Some(record.structure_section_ordinal());
            }
            let marker = marker(document, record)?;
            if marker_equals(document, marker, b"TABLE")? {
                pending.clear();
                active = table_kind(document, record, &application_groups)?;
            } else if marker_equals(document, marker, b"ENDTAB")? {
                if active.is_some() {
                    entries
                        .try_reserve(pending.len())
                        .map_err(|_| out_of_memory())?;
                    entries.extend_from_slice(&pending);
                }
                pending.clear();
                active = None;
            } else if let Some(kind) = active
                && marker_equals(document, marker, kind.marker())?
                && let Some(name) = unique_group_two(document, record, &application_groups)?
            {
                pending.try_reserve(1).map_err(|_| out_of_memory())?;
                pending.push(DxfNamedSymbolTableEntry { record, kind, name });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            raw_records,
            application_groups,
            entries: entries.into_boxed_slice(),
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
    pub fn entries(&self) -> &[DxfNamedSymbolTableEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_raw_ordinal(&self, raw_ordinal: u64) -> Option<DxfNamedSymbolTableEntry> {
        self.entries
            .binary_search_by_key(&raw_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn named_symbol_table_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfNamedSymbolTableDirectory, DxfError> {
        DxfNamedSymbolTableDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn named_symbol_table_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfNamedSymbolTableDirectory, DxfError> {
        DxfRawDocumentView::from(self).named_symbol_table_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn named_symbol_table_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfNamedSymbolTableDirectory, DxfError> {
        DxfRawDocumentView::from(self).named_symbol_table_directory(cancellation)
    }
}

fn marker(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<crate::ByteSpan, DxfError> {
    document
        .group(record.marker_occurrence())
        .map(crate::DxfRawGroup::value_payload_span)
        .ok_or_else(invalid_internal_data)
}

fn marker_equals(
    document: DxfRawDocumentView<'_>,
    marker: crate::ByteSpan,
    expected: &[u8],
) -> Result<bool, DxfError> {
    document.raw_span_equals_exact(marker, expected)
}

fn table_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    application_groups: &DxfApplicationGroupDirectory,
) -> Result<Option<DxfNamedSymbolTableKind>, DxfError> {
    let Some(name) = unique_group_two(document, record, application_groups)? else {
        return Ok(None);
    };
    for kind in TABLE_KINDS {
        if document.raw_span_equals_exact(name.value_span(), kind.marker())? {
            return Ok(Some(kind));
        }
    }
    Ok(None)
}

fn unique_group_two(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    application_groups: &DxfApplicationGroupDirectory,
) -> Result<Option<DxfRawValueProvenance>, DxfError> {
    let mut found = None;
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
        if application_groups
            .group_for_content_occurrence(occurrence)
            .is_some()
        {
            continue;
        }
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if group.group_code().value() != 2 {
            continue;
        }
        let value = DxfRawValueProvenance::new(occurrence, group.value_payload_span())
            .ok_or_else(invalid_internal_data)?;
        if found.replace(value).is_some() {
            return Ok(None);
        }
    }
    Ok(found)
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

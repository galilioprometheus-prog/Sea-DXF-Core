//! Exact named DIMSTYLE records from completely closed DIMSTYLE tables.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawRecord, DxfRawRecordDirectory,
    DxfRawRecordSectionKind, DxfRawValueProvenance, DxfSourceId,
};

/// One exact named DIMSTYLE record admitted from a closed DIMSTYLE table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleTableEntry {
    record: DxfRawRecord,
    name: DxfRawValueProvenance,
}

impl DxfDimStyleTableEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn name(self) -> DxfRawValueProvenance {
        self.name
    }
}

/// Immutable exact-name evidence from every complete DIMSTYLE table envelope.
#[derive(Debug)]
pub struct DxfDimStyleTableDirectory {
    source_id: DxfSourceId,
    raw_records: DxfRawRecordDirectory,
    application_groups: DxfApplicationGroupDirectory,
    entries: Box<[DxfDimStyleTableEntry]>,
}

impl DxfDimStyleTableDirectory {
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
        let mut active = false;
        let mut section = None;
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if record.section_kind() != DxfRawRecordSectionKind::Tables {
                continue;
            }
            if section != Some(record.structure_section_ordinal()) {
                pending.clear();
                active = false;
                section = Some(record.structure_section_ordinal());
            }
            match marker_kind(document, record)? {
                MarkerKind::Table => {
                    pending.clear();
                    active = table_is_dimstyle(document, record, &application_groups)?;
                }
                MarkerKind::Endtab => {
                    if active {
                        entries
                            .try_reserve(pending.len())
                            .map_err(|_| out_of_memory())?;
                        entries.extend_from_slice(&pending);
                    }
                    pending.clear();
                    active = false;
                }
                MarkerKind::Dimstyle if active => {
                    if let Some(name) = unique_group_two(document, record, &application_groups)? {
                        pending.try_reserve(1).map_err(|_| out_of_memory())?;
                        pending.push(DxfDimStyleTableEntry { record, name });
                    }
                }
                MarkerKind::Dimstyle | MarkerKind::Other => {}
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
    pub fn entries(&self) -> &[DxfDimStyleTableEntry] {
        &self.entries
    }
}

impl DxfRawDocumentView<'_> {
    pub fn dimstyle_table_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleTableDirectory, DxfError> {
        DxfDimStyleTableDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn dimstyle_table_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleTableDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_table_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn dimstyle_table_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleTableDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_table_directory(cancellation)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MarkerKind {
    Table,
    Endtab,
    Dimstyle,
    Other,
}

fn marker_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<MarkerKind, DxfError> {
    let span = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?
        .value_payload_span();
    for (expected, kind) in [
        (b"TABLE".as_slice(), MarkerKind::Table),
        (b"ENDTAB".as_slice(), MarkerKind::Endtab),
        (b"DIMSTYLE".as_slice(), MarkerKind::Dimstyle),
    ] {
        if document.raw_span_equals_exact(span, expected)? {
            return Ok(kind);
        }
    }
    Ok(MarkerKind::Other)
}

fn table_is_dimstyle(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    application_groups: &DxfApplicationGroupDirectory,
) -> Result<bool, DxfError> {
    let Some(name) = unique_group_two(document, record, application_groups)? else {
        return Ok(false);
    };
    document.raw_span_equals_exact(name.value_span(), b"DIMSTYLE")
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

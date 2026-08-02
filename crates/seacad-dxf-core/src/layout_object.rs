//! Subclass-aware names from completely closed `OBJECTS`/`LAYOUT` records.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawRecord, DxfRawRecordDirectory,
    DxfRawRecordSectionKind, DxfRawValueProvenance, DxfSourceId,
};

/// Cardinality of group 1 inside one exact `AcDbLayout` subclass.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLayoutObjectNameState {
    Missing,
    Unique(DxfRawValueProvenance),
    Duplicate {
        first: DxfRawValueProvenance,
        occurrence_count: u32,
    },
}

/// One exact `OBJECTS`/`LAYOUT` record and its reviewed name evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLayoutObjectEntry {
    record: DxfRawRecord,
    name: DxfLayoutObjectNameState,
}

impl DxfLayoutObjectEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn name(self) -> DxfLayoutObjectNameState {
        self.name
    }
}

/// Source-bound inventory of exact layout objects from closed `OBJECTS` sections.
#[derive(Debug)]
pub struct DxfLayoutObjectDirectory {
    source_id: DxfSourceId,
    raw_records: DxfRawRecordDirectory,
    application_groups: DxfApplicationGroupDirectory,
    entries: Box<[DxfLayoutObjectEntry]>,
}

impl DxfLayoutObjectDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        for observed in [raw_records.source_id(), application_groups.source_id()] {
            ensure_source(document.source_id(), observed)?;
        }
        let mut entries = Vec::new();
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if record.section_kind() != DxfRawRecordSectionKind::Objects
                || !record_marker_equals(document, record, b"LAYOUT")?
            {
                continue;
            }
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfLayoutObjectEntry {
                record,
                name: layout_name(document, record, &application_groups, cancellation)?,
            });
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
    pub fn entries(&self) -> &[DxfLayoutObjectEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_raw_ordinal(&self, raw_ordinal: u64) -> Option<DxfLayoutObjectEntry> {
        self.entries
            .binary_search_by_key(&raw_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn layout_object_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLayoutObjectDirectory, DxfError> {
        DxfLayoutObjectDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn layout_object_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLayoutObjectDirectory, DxfError> {
        DxfRawDocumentView::from(self).layout_object_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn layout_object_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLayoutObjectDirectory, DxfError> {
        DxfRawDocumentView::from(self).layout_object_directory(cancellation)
    }
}

fn layout_name(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    application_groups: &DxfApplicationGroupDirectory,
    cancellation: &DxfCancellationToken,
) -> Result<DxfLayoutObjectNameState, DxfError> {
    let mut inside_layout_subclass = false;
    let mut first = None;
    let mut count = 0_u32;
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
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
        if group.group_code().value() == 100 {
            inside_layout_subclass =
                document.raw_span_equals_exact(group.value_payload_span(), b"AcDbLayout")?;
            continue;
        }
        if !inside_layout_subclass || group.group_code().value() != 1 {
            continue;
        }
        let value = DxfRawValueProvenance::new(occurrence, group.value_payload_span())
            .ok_or_else(invalid_internal_data)?;
        count = count.checked_add(1).ok_or_else(invalid_internal_data)?;
        first.get_or_insert(value);
    }
    match (first, count) {
        (None, 0) => Ok(DxfLayoutObjectNameState::Missing),
        (Some(name), 1) => Ok(DxfLayoutObjectNameState::Unique(name)),
        (Some(first), occurrence_count) => Ok(DxfLayoutObjectNameState::Duplicate {
            first,
            occurrence_count,
        }),
        (None, _) => Err(invalid_internal_data()),
    }
}

fn record_marker_equals(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    expected: &[u8],
) -> Result<bool, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    document.raw_span_equals_exact(marker.value_payload_span(), expected)
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
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

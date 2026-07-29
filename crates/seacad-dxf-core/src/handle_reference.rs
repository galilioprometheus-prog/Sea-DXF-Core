//! Record-scoped pointer and owner handle occurrences without target resolution.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandleGroupClass,
    DxfIoOperation, DxfRawDocumentView, DxfRawHandleLookup, DxfRawHandleValue, DxfRawRecord,
    DxfSourceId, classify_dxf_handle_group_code,
};

/// One source-anchored pointer or owner handle occurrence inside a raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandleReferenceEntry {
    record: DxfRawRecord,
    value: DxfRawHandleValue,
}

impl DxfHandleReferenceEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn value(self) -> DxfRawHandleValue {
        self.value
    }

    #[must_use]
    pub const fn class(self) -> DxfHandleGroupClass {
        self.value.class()
    }
}

/// Immutable source-order directory of record-scoped pointer and owner handles.
///
/// Arbitrary handles and object-identity handles are deliberately excluded.
/// Entries retain lexical parse evidence but do not claim that a target exists.
#[derive(Debug)]
pub struct DxfHandleReferenceDirectory {
    source_id: DxfSourceId,
    record_count: u32,
    entries: Box<[DxfHandleReferenceEntry]>,
}

impl DxfHandleReferenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let records = document.raw_record_directory(cancellation)?;
        if records.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: records.source_id(),
            });
        }

        let record_count = compact_len(records.records().len())?;
        let mut entries = Vec::new();
        for record in records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            for occurrence in record.group_range().start()..record.group_range().end() {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                let Some(class) = classify_dxf_handle_group_code(group.group_code()) else {
                    continue;
                };
                if !is_reference_class(class) {
                    continue;
                }
                let value = match document.raw_handle_at(occurrence, cancellation)? {
                    DxfRawHandleLookup::Handle(value) if value.class() == class => value,
                    DxfRawHandleLookup::MissingOccurrence
                    | DxfRawHandleLookup::NotHandleGroup(_)
                    | DxfRawHandleLookup::Handle(_) => return Err(invalid_internal_data()),
                };
                entries.try_reserve(1).map_err(|_| out_of_memory())?;
                entries.push(DxfHandleReferenceEntry { record, value });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            record_count,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn record_count(&self) -> u64 {
        self.record_count as u64
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHandleReferenceEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHandleReferenceEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn references_for_record(&self, record_ordinal: u64) -> Option<&[DxfHandleReferenceEntry]> {
        if record_ordinal >= self.record_count() {
            return None;
        }
        let start = self
            .entries
            .partition_point(|entry| entry.record().ordinal() < record_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.record().ordinal() <= record_ordinal);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn reference_for_group(&self, occurrence: u64) -> Option<DxfHandleReferenceEntry> {
        let index = self
            .entries
            .partition_point(|entry| entry.value().group().occurrence() < occurrence);
        self.entries
            .get(index)
            .copied()
            .filter(|entry| entry.value().group().occurrence() == occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    /// Indexes pointer and owner handle occurrences without resolving targets.
    pub fn handle_reference_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleReferenceDirectory, DxfError> {
        DxfHandleReferenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn handle_reference_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleReferenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_reference_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn handle_reference_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleReferenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_reference_directory(cancellation)
    }
}

const fn is_reference_class(class: DxfHandleGroupClass) -> bool {
    matches!(
        class,
        DxfHandleGroupClass::SoftPointer
            | DxfHandleGroupClass::HardPointer
            | DxfHandleGroupClass::SoftOwner
            | DxfHandleGroupClass::HardOwner
    )
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

//! Exact named DIMSTYLE records from completely closed DIMSTYLE tables.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfNamedSymbolTableDirectory, DxfNamedSymbolTableKind,
    DxfRawDocumentView, DxfRawRecord, DxfRawRecordDirectory, DxfRawValueProvenance, DxfSourceId,
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
    named: DxfNamedSymbolTableDirectory,
    entries: Box<[DxfDimStyleTableEntry]>,
}

impl DxfDimStyleTableDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let named = document.named_symbol_table_directory(cancellation)?;
        if named.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: named.source_id(),
            });
        }
        let mut entries = Vec::new();
        for entry in named.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if entry.kind() != DxfNamedSymbolTableKind::DimStyle {
                continue;
            }
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfDimStyleTableEntry {
                record: entry.record(),
                name: entry.name(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            named,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn named_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.named
    }

    #[must_use]
    pub const fn raw_record_directory(&self) -> &DxfRawRecordDirectory {
        self.named.raw_record_directory()
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        self.named.application_group_directory()
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

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

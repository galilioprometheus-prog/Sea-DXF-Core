//! Exact ATTDEF record evidence owned by indexed BLOCK definitions.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockDefinitionDirectory,
    DxfBlockDefinitionEntry, DxfCancellationToken, DxfError, DxfIoOperation, DxfRawDocumentView,
    DxfRawRecord, DxfSourceId,
};

/// One exact ATTDEF member and its owning BLOCK-definition evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionEntry {
    owner: DxfBlockDefinitionEntry,
    record: DxfRawRecord,
    member_ordinal: u32,
    attribute_definition_ordinal: u32,
}

impl DxfBlockAttributeDefinitionEntry {
    #[must_use]
    pub const fn owner(self) -> DxfBlockDefinitionEntry {
        self.owner
    }

    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    /// Zero-based position of this ATTDEF among all member records of its BLOCK.
    #[must_use]
    pub const fn member_ordinal(self) -> u64 {
        self.member_ordinal as u64
    }

    /// Zero-based position of this ATTDEF among the ATTDEF members of its BLOCK.
    #[must_use]
    pub const fn attribute_definition_ordinal(self) -> u64 {
        self.attribute_definition_ordinal as u64
    }
}

/// Immutable exact-ATTDEF directory over complete BLOCKS sections.
#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionDirectory {
    source_id: DxfSourceId,
    definitions: DxfBlockDefinitionDirectory,
    entries: Box<[DxfBlockAttributeDefinitionEntry]>,
}

impl DxfBlockAttributeDefinitionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let definitions = document.block_definition_directory(cancellation)?;
        if definitions.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: definitions.source_id(),
            });
        }

        let mut entries = Vec::new();
        for owner in definitions.definitions().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let members = definitions
                .members_for_block_raw_ordinal(owner.block_record().ordinal())
                .ok_or_else(invalid_internal_data)?;
            let mut attribute_definition_ordinal = 0_u32;
            for (member_ordinal, record) in members.iter().copied().enumerate() {
                ensure_not_cancelled(cancellation)?;
                if is_exact_attdef(document, record)? {
                    entries.try_reserve(1).map_err(|_| out_of_memory())?;
                    entries.push(DxfBlockAttributeDefinitionEntry {
                        owner,
                        record,
                        member_ordinal: compact_len(member_ordinal)?,
                        attribute_definition_ordinal,
                    });
                    attribute_definition_ordinal = attribute_definition_ordinal
                        .checked_add(1)
                        .ok_or_else(invalid_internal_data)?;
                }
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            definitions,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn block_definition_directory(&self) -> &DxfBlockDefinitionDirectory {
        &self.definitions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfBlockAttributeDefinitionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_attdef_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfBlockAttributeDefinitionEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn entries_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> &[DxfBlockAttributeDefinitionEntry] {
        let start = self
            .entries
            .partition_point(|entry| entry.owner().block_record().ordinal() < raw_record_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.owner().block_record().ordinal() <= raw_record_ordinal);
        match self.entries.get(start..end) {
            Some(entries) => entries,
            None => &[],
        }
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_attribute_definition_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionDirectory, DxfError> {
        DxfBlockAttributeDefinitionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_directory(cancellation)
    }
}

fn is_exact_attdef(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<bool, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    let span = marker.value_payload_span();
    Ok(span.len() == b"ATTDEF".len() as u64 && document.raw_span_equals_exact(span, b"ATTDEF")?)
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

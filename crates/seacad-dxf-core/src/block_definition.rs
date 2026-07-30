//! Exact BLOCK/member/ENDBLK record-sequence evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawRecord, DxfRawRecordDirectory, DxfRawRecordSectionKind, DxfSourceId,
};

/// How one exact BLOCK record's definition sequence ends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockDefinitionState {
    /// Zero or more member records are followed by exact ENDBLK.
    Closed,
    /// Another exact BLOCK appears before exact ENDBLK.
    Interrupted,
    /// The containing BLOCKS section ends before exact ENDBLK.
    Unclosed,
}

/// Half-open range in the directory's retained member-record array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockMemberRecordRange {
    start: u32,
    end: u32,
}

impl DxfBlockMemberRecordRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// One exact BLOCK marker and its section-local definition evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockDefinitionEntry {
    block_record: DxfRawRecord,
    member_range: DxfBlockMemberRecordRange,
    boundary_record: Option<DxfRawRecord>,
    state: DxfBlockDefinitionState,
}

impl DxfBlockDefinitionEntry {
    #[must_use]
    pub const fn block_record(self) -> DxfRawRecord {
        self.block_record
    }

    #[must_use]
    pub const fn member_range(self) -> DxfBlockMemberRecordRange {
        self.member_range
    }

    /// Returns exact ENDBLK when closed or the nested BLOCK when interrupted.
    #[must_use]
    pub const fn boundary_record(self) -> Option<DxfRawRecord> {
        self.boundary_record
    }

    #[must_use]
    pub const fn state(self) -> DxfBlockDefinitionState {
        self.state
    }
}

/// Immutable BLOCK-definition directory over complete BLOCKS sections.
#[derive(Debug)]
pub struct DxfBlockDefinitionDirectory {
    source_id: DxfSourceId,
    raw_records: DxfRawRecordDirectory,
    definitions: Box<[DxfBlockDefinitionEntry]>,
    members: Box<[DxfRawRecord]>,
}

impl DxfBlockDefinitionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        if raw_records.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: raw_records.source_id(),
            });
        }

        let records = raw_records.records();
        let mut definitions = Vec::new();
        let mut members = Vec::new();
        let mut index = 0_usize;
        while let Some(record) = records.get(index).copied() {
            ensure_not_cancelled(cancellation)?;
            if record.section_kind() != DxfRawRecordSectionKind::Blocks
                || record_marker_kind(document, record)? != DxfBlockRecordMarkerKind::Block
            {
                index = index.checked_add(1).ok_or_else(invalid_internal_data)?;
                continue;
            }

            let member_start = compact_len(members.len())?;
            let mut cursor = index.checked_add(1).ok_or_else(invalid_internal_data)?;
            let (boundary_record, state) = loop {
                ensure_not_cancelled(cancellation)?;
                let Some(candidate) = records.get(cursor).copied() else {
                    break (None, DxfBlockDefinitionState::Unclosed);
                };
                if candidate.structure_section_ordinal() != record.structure_section_ordinal() {
                    break (None, DxfBlockDefinitionState::Unclosed);
                }
                match record_marker_kind(document, candidate)? {
                    DxfBlockRecordMarkerKind::Endblk => {
                        break (Some(candidate), DxfBlockDefinitionState::Closed);
                    }
                    DxfBlockRecordMarkerKind::Block => {
                        break (Some(candidate), DxfBlockDefinitionState::Interrupted);
                    }
                    DxfBlockRecordMarkerKind::Other => {
                        members.try_reserve(1).map_err(|_| out_of_memory())?;
                        members.push(candidate);
                        cursor = cursor.checked_add(1).ok_or_else(invalid_internal_data)?;
                    }
                }
            };
            let member_end = compact_len(members.len())?;
            definitions.try_reserve(1).map_err(|_| out_of_memory())?;
            definitions.push(DxfBlockDefinitionEntry {
                block_record: record,
                member_range: DxfBlockMemberRecordRange::new(member_start, member_end)?,
                boundary_record,
                state,
            });

            index = match state {
                DxfBlockDefinitionState::Closed => {
                    cursor.checked_add(1).ok_or_else(invalid_internal_data)?
                }
                DxfBlockDefinitionState::Interrupted | DxfBlockDefinitionState::Unclosed => cursor,
            };
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            raw_records,
            definitions: definitions.into_boxed_slice(),
            members: members.into_boxed_slice(),
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
    pub fn definitions(&self) -> &[DxfBlockDefinitionEntry] {
        &self.definitions
    }

    #[must_use]
    pub fn member_records(&self) -> &[DxfRawRecord] {
        &self.members
    }

    #[must_use]
    pub fn definition_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfBlockDefinitionEntry> {
        self.definitions
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.block_record().ordinal())
            .ok()
            .and_then(|index| self.definitions.get(index).copied())
    }

    #[must_use]
    pub fn members_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfRawRecord]> {
        let entry = self.definition_for_block_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.member_range().start()).ok()?;
        let end = usize::try_from(entry.member_range().end()).ok()?;
        self.members.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_definition_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockDefinitionDirectory, DxfError> {
        DxfBlockDefinitionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_definition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockDefinitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_definition_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_definition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockDefinitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_definition_directory(cancellation)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DxfBlockRecordMarkerKind {
    Block,
    Endblk,
    Other,
}

fn record_marker_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<DxfBlockRecordMarkerKind, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    let span = marker.value_payload_span();
    if span.len() == b"BLOCK".len() as u64 && document.raw_span_equals_exact(span, b"BLOCK")? {
        return Ok(DxfBlockRecordMarkerKind::Block);
    }
    if span.len() == b"ENDBLK".len() as u64 && document.raw_span_equals_exact(span, b"ENDBLK")? {
        return Ok(DxfBlockRecordMarkerKind::Endblk);
    }
    Ok(DxfBlockRecordMarkerKind::Other)
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

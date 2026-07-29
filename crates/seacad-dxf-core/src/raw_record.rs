//! Format-neutral raw record boundaries within documented record-bearing sections.

use std::io;

use crate::{
    DxfAsciiGroupRange, DxfAsciiRawDocument, DxfAsciiSectionClosure, DxfAsciiSectionKind,
    DxfAsciiSectionName, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

/// Documented section family whose content is organized by group-zero records.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfRawRecordSectionKind {
    Classes,
    Tables,
    Blocks,
    Entities,
    Objects,
}

impl DxfRawRecordSectionKind {
    fn from_section_name(name: DxfAsciiSectionName) -> Option<Self> {
        match name {
            DxfAsciiSectionName::Known(DxfAsciiSectionKind::Classes) => Some(Self::Classes),
            DxfAsciiSectionName::Known(DxfAsciiSectionKind::Tables) => Some(Self::Tables),
            DxfAsciiSectionName::Known(DxfAsciiSectionKind::Blocks) => Some(Self::Blocks),
            DxfAsciiSectionName::Known(DxfAsciiSectionKind::Entities) => Some(Self::Entities),
            DxfAsciiSectionName::Known(DxfAsciiSectionKind::Objects) => Some(Self::Objects),
            DxfAsciiSectionName::Known(
                DxfAsciiSectionKind::Header | DxfAsciiSectionKind::ThumbnailImage,
            )
            | DxfAsciiSectionName::Unknown
            | DxfAsciiSectionName::InvalidGroupCode(_)
            | DxfAsciiSectionName::Missing => None,
        }
    }
}

/// Whether one recognized record-bearing section was indexed completely.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfRawRecordSectionState {
    Indexed,
    Interrupted,
    Unclosed,
}

/// Half-open range of ordinals in a raw record directory.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfRawRecordRange {
    start: u32,
    end: u32,
}

impl DxfRawRecordRange {
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

/// One documented record-bearing section and its directory slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfRawRecordSection {
    structure_section_ordinal: u32,
    kind: DxfRawRecordSectionKind,
    source_group_range: DxfAsciiGroupRange,
    content_group_range: DxfAsciiGroupRange,
    record_range: DxfRawRecordRange,
    state: DxfRawRecordSectionState,
}

impl DxfRawRecordSection {
    #[must_use]
    pub const fn structure_section_ordinal(self) -> u64 {
        self.structure_section_ordinal as u64
    }

    #[must_use]
    pub const fn kind(self) -> DxfRawRecordSectionKind {
        self.kind
    }

    #[must_use]
    pub const fn source_group_range(self) -> DxfAsciiGroupRange {
        self.source_group_range
    }

    #[must_use]
    pub const fn content_group_range(self) -> DxfAsciiGroupRange {
        self.content_group_range
    }

    #[must_use]
    pub const fn record_range(self) -> DxfRawRecordRange {
        self.record_range
    }

    #[must_use]
    pub const fn state(self) -> DxfRawRecordSectionState {
        self.state
    }
}

/// One group-zero-delimited chunk inside a completely closed record-bearing section.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfRawRecord {
    ordinal: u32,
    section_record_ordinal: u32,
    structure_section_ordinal: u32,
    section_kind: DxfRawRecordSectionKind,
    group_range: DxfAsciiGroupRange,
}

impl DxfRawRecord {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn section_record_ordinal(self) -> u64 {
        self.section_record_ordinal as u64
    }

    #[must_use]
    pub const fn structure_section_ordinal(self) -> u64 {
        self.structure_section_ordinal as u64
    }

    #[must_use]
    pub const fn section_kind(self) -> DxfRawRecordSectionKind {
        self.section_kind
    }

    #[must_use]
    pub const fn group_range(self) -> DxfAsciiGroupRange {
        self.group_range
    }

    #[must_use]
    pub const fn marker_occurrence(self) -> u64 {
        self.group_range.start()
    }
}

/// Immutable directory of raw record chunks in completely closed known sections.
#[derive(Debug)]
pub struct DxfRawRecordDirectory {
    source_id: DxfSourceId,
    sections: Box<[DxfRawRecordSection]>,
    records: Box<[DxfRawRecord]>,
}

impl DxfRawRecordDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let structure = document.structure_index();
        if structure.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: structure.source_id(),
            });
        }

        let mut sections = Vec::new();
        let mut records = Vec::new();
        for (section_index, section) in structure.sections().iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            let Some(kind) = DxfRawRecordSectionKind::from_section_name(section.name()) else {
                continue;
            };
            let section_ordinal =
                u32::try_from(section_index).map_err(|_| invalid_internal_data())?;
            let record_start = compact_len(records.len())?;
            let state = match section.closure() {
                DxfAsciiSectionClosure::Closed => {
                    append_section_records(
                        structure,
                        section.content_range(),
                        section_ordinal,
                        kind,
                        cancellation,
                        &mut records,
                    )?;
                    DxfRawRecordSectionState::Indexed
                }
                DxfAsciiSectionClosure::Interrupted => DxfRawRecordSectionState::Interrupted,
                DxfAsciiSectionClosure::Unclosed => DxfRawRecordSectionState::Unclosed,
            };
            let record_end = compact_len(records.len())?;
            sections.try_reserve(1).map_err(|_| out_of_memory())?;
            sections.push(DxfRawRecordSection {
                structure_section_ordinal: section_ordinal,
                kind,
                source_group_range: section.group_range(),
                content_group_range: section.content_range(),
                record_range: DxfRawRecordRange::new(record_start, record_end)?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            sections: sections.into_boxed_slice(),
            records: records.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub fn sections(&self) -> &[DxfRawRecordSection] {
        &self.sections
    }

    #[must_use]
    pub fn records(&self) -> &[DxfRawRecord] {
        &self.records
    }

    #[must_use]
    pub fn record(&self, ordinal: u64) -> Option<DxfRawRecord> {
        let index = usize::try_from(ordinal).ok()?;
        self.records.get(index).copied()
    }

    #[must_use]
    pub fn record_for_group(&self, occurrence: u64) -> Option<DxfRawRecord> {
        let index = self
            .records
            .partition_point(|record| record.group_range().end() <= occurrence);
        self.records.get(index).copied().filter(|record| {
            occurrence >= record.group_range().start() && occurrence < record.group_range().end()
        })
    }

    #[must_use]
    pub fn section_by_structure_ordinal(
        &self,
        structure_section_ordinal: u64,
    ) -> Option<DxfRawRecordSection> {
        let structure_section_ordinal = u32::try_from(structure_section_ordinal).ok()?;
        self.sections
            .binary_search_by_key(&structure_section_ordinal, |section| {
                section.structure_section_ordinal
            })
            .ok()
            .and_then(|index| self.sections.get(index).copied())
    }

    #[must_use]
    pub fn records_for_structure_section(
        &self,
        structure_section_ordinal: u64,
    ) -> Option<&[DxfRawRecord]> {
        let section = self.section_by_structure_ordinal(structure_section_ordinal)?;
        let start = usize::try_from(section.record_range().start()).ok()?;
        let end = usize::try_from(section.record_range().end()).ok()?;
        self.records.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    /// Indexes group-zero-delimited raw records without interpreting record types.
    pub fn raw_record_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfRawRecordDirectory, DxfError> {
        DxfRawRecordDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn raw_record_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfRawRecordDirectory, DxfError> {
        DxfRawDocumentView::from(self).raw_record_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn raw_record_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfRawRecordDirectory, DxfError> {
        DxfRawDocumentView::from(self).raw_record_directory(cancellation)
    }
}

fn append_section_records(
    structure: &crate::DxfAsciiStructureIndex,
    content: DxfAsciiGroupRange,
    section_ordinal: u32,
    section_kind: DxfRawRecordSectionKind,
    cancellation: &DxfCancellationToken,
    records: &mut Vec<DxfRawRecord>,
) -> Result<(), DxfError> {
    let mut zero_index = lower_bound_zero_occurrence(structure, content.start());
    let mut section_record_ordinal = 0_u32;
    while let Some(start) = structure.zero_group_occurrence(zero_index) {
        ensure_not_cancelled(cancellation)?;
        if start >= content.end() {
            break;
        }
        let next = structure
            .zero_group_occurrence(zero_index.saturating_add(1))
            .unwrap_or(content.end())
            .min(content.end());
        let start = u32::try_from(start).map_err(|_| invalid_internal_data())?;
        let end = u32::try_from(next).map_err(|_| invalid_internal_data())?;
        let ordinal = compact_len(records.len())?;
        records.try_reserve(1).map_err(|_| out_of_memory())?;
        records.push(DxfRawRecord {
            ordinal,
            section_record_ordinal,
            structure_section_ordinal: section_ordinal,
            section_kind,
            group_range: DxfAsciiGroupRange::new(start, end)?,
        });
        section_record_ordinal = section_record_ordinal
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        zero_index = zero_index
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
    }
    Ok(())
}

fn lower_bound_zero_occurrence(structure: &crate::DxfAsciiStructureIndex, target: u64) -> u64 {
    let mut low = 0_u64;
    let mut high = structure.zero_group_count();
    while low < high {
        let midpoint = low + (high - low) / 2;
        if structure
            .zero_group_occurrence(midpoint)
            .is_some_and(|occurrence| occurrence < target)
        {
            low = midpoint.saturating_add(1);
        } else {
            high = midpoint;
        }
    }
    low
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

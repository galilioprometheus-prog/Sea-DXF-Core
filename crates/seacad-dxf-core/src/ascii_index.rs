//! Compact, source-anchored ASCII section and group-zero index.

use std::io;

use crate::{
    ByteSpan, DxfAsciiGroup, DxfDiagnostic, DxfDiagnosticCode, DxfError, DxfGroupCode,
    DxfIoOperation, DxfSourceId,
};

/// Half-open range of raw group occurrences.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfAsciiGroupRange {
    start: u32,
    end: u32,
}

impl DxfAsciiGroupRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_source_data())
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
}

/// Section names explicitly documented by Autodesk.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAsciiSectionKind {
    Header,
    Classes,
    Tables,
    Blocks,
    Entities,
    Objects,
    ThumbnailImage,
}

impl DxfAsciiSectionKind {
    fn from_bytes(value: &[u8]) -> Option<Self> {
        match value {
            b"HEADER" => Some(Self::Header),
            b"CLASSES" => Some(Self::Classes),
            b"TABLES" => Some(Self::Tables),
            b"BLOCKS" => Some(Self::Blocks),
            b"ENTITIES" => Some(Self::Entities),
            b"OBJECTS" => Some(Self::Objects),
            b"THUMBNAILIMAGE" => Some(Self::ThumbnailImage),
            _ => None,
        }
    }
}

/// Exact classification of the group immediately following `0/SECTION`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAsciiSectionName {
    Known(DxfAsciiSectionKind),
    Unknown,
    InvalidGroupCode(DxfGroupCode),
    Missing,
}

/// How one indexed section envelope ended.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAsciiSectionClosure {
    Closed,
    Interrupted,
    Unclosed,
}

/// Immutable provenance and group ranges for one section envelope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfAsciiSection {
    name_span: Option<ByteSpan>,
    name: DxfAsciiSectionName,
    group_range: DxfAsciiGroupRange,
    content_range: DxfAsciiGroupRange,
    closure: DxfAsciiSectionClosure,
}

impl DxfAsciiSection {
    #[must_use]
    pub const fn name_span(self) -> Option<ByteSpan> {
        self.name_span
    }

    #[must_use]
    pub const fn name(self) -> DxfAsciiSectionName {
        self.name
    }

    #[must_use]
    pub const fn group_range(self) -> DxfAsciiGroupRange {
        self.group_range
    }

    #[must_use]
    pub const fn content_range(self) -> DxfAsciiGroupRange {
        self.content_range
    }

    #[must_use]
    pub const fn closure(self) -> DxfAsciiSectionClosure {
        self.closure
    }
}

/// Eager one-pass index over section envelopes and every numeric group code 0.
#[derive(Debug)]
pub struct DxfAsciiStructureIndex {
    source_id: DxfSourceId,
    total_group_count: u32,
    inside_section_group_count: u32,
    sections: Box<[DxfAsciiSection]>,
    zero_group_occurrences: Box<[u32]>,
    diagnostics: Box<[DxfDiagnostic]>,
    diagnostics_truncated: bool,
}

impl DxfAsciiStructureIndex {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn total_group_count(&self) -> u64 {
        self.total_group_count as u64
    }

    #[must_use]
    pub const fn inside_section_group_count(&self) -> u64 {
        self.inside_section_group_count as u64
    }

    #[must_use]
    pub const fn outside_section_group_count(&self) -> u64 {
        (self.total_group_count - self.inside_section_group_count) as u64
    }

    #[must_use]
    pub fn sections(&self) -> &[DxfAsciiSection] {
        &self.sections
    }

    #[must_use]
    pub fn zero_group_count(&self) -> u64 {
        self.zero_group_occurrences.len() as u64
    }

    #[must_use]
    pub fn zero_group_occurrence(&self, index: u64) -> Option<u64> {
        let index = usize::try_from(index).ok()?;
        self.zero_group_occurrences
            .get(index)
            .map(|value| *value as u64)
    }

    #[must_use]
    pub fn zero_group_range(&self, index: u64) -> Option<DxfAsciiGroupRange> {
        let index = usize::try_from(index).ok()?;
        let start = *self.zero_group_occurrences.get(index)?;
        let end = self
            .zero_group_occurrences
            .get(index.saturating_add(1))
            .copied()
            .unwrap_or(self.total_group_count);
        DxfAsciiGroupRange::new(start, end).ok()
    }

    #[must_use]
    pub fn section_ordinal_for_group(&self, occurrence: u64) -> Option<u64> {
        let index = self
            .sections
            .partition_point(|section| section.group_range().end() <= occurrence);
        self.sections
            .get(index)
            .filter(|section| {
                occurrence >= section.group_range().start()
                    && occurrence < section.group_range().end()
            })
            .and_then(|_| u64::try_from(index).ok())
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[DxfDiagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub const fn diagnostics_were_truncated(&self) -> bool {
        self.diagnostics_truncated
    }
}

struct OpenSection {
    marker_occurrence: u32,
    marker_span: ByteSpan,
    name_span: Option<ByteSpan>,
    name: DxfAsciiSectionName,
    awaiting_name: bool,
    content_start: u32,
}

pub(crate) struct DxfAsciiStructureTracker {
    max_diagnostics: u64,
    inside_section_group_count: u32,
    open: Option<OpenSection>,
    sections: Vec<DxfAsciiSection>,
    zero_group_occurrences: Vec<u32>,
    diagnostics: Vec<DxfDiagnostic>,
    diagnostics_truncated: bool,
}

impl DxfAsciiStructureTracker {
    pub(crate) fn new(max_diagnostics: u64) -> Self {
        Self {
            max_diagnostics,
            inside_section_group_count: 0,
            open: None,
            sections: Vec::new(),
            zero_group_occurrences: Vec::new(),
            diagnostics: Vec::new(),
            diagnostics_truncated: false,
        }
    }

    pub(crate) fn observe(
        &mut self,
        group: DxfAsciiGroup<'_>,
        is_framed_eof: bool,
    ) -> Result<(), DxfError> {
        let occurrence = compact_occurrence(group.occurrence())?;
        let is_zero = group.group_code().value() == 0;
        if is_zero {
            self.zero_group_occurrences
                .try_reserve(1)
                .map_err(|_| out_of_memory())?;
            self.zero_group_occurrences.push(occurrence);
        }

        let is_section = is_zero && group.raw_value() == b"SECTION";
        let is_endsec = is_zero && group.raw_value() == b"ENDSEC";
        if is_section {
            self.begin_section(occurrence, group.value_line().content_span())?;
        } else if is_framed_eof {
            self.finish_before_eof(occurrence, group.value_line().content_span())?;
        } else {
            if self.open.is_some() {
                if is_endsec {
                    self.mark_name_missing()?;
                } else {
                    self.resolve_name(group, occurrence)?;
                }
                self.increment_inside_count()?;
            }
            if is_endsec {
                self.close_or_report_orphan(occurrence, group.value_line().content_span())?;
            }
        }
        Ok(())
    }

    pub(crate) fn finish(
        mut self,
        source_id: DxfSourceId,
        total_group_count: u64,
    ) -> Result<DxfAsciiStructureIndex, DxfError> {
        let total_group_count = compact_occurrence(total_group_count)?;
        if self.open.is_some() {
            self.mark_name_missing()?;
            let span = self.open.as_ref().map(|open| open.marker_span);
            self.record_diagnostic(DxfDiagnosticCode::SECTION_UNCLOSED, span)?;
            self.finish_open(
                DxfAsciiSectionClosure::Unclosed,
                total_group_count,
                total_group_count,
            )?;
        }
        if self.inside_section_group_count > total_group_count {
            return Err(invalid_source_data());
        }
        Ok(DxfAsciiStructureIndex {
            source_id,
            total_group_count,
            inside_section_group_count: self.inside_section_group_count,
            sections: self.sections.into_boxed_slice(),
            zero_group_occurrences: self.zero_group_occurrences.into_boxed_slice(),
            diagnostics: self.diagnostics.into_boxed_slice(),
            diagnostics_truncated: self.diagnostics_truncated,
        })
    }

    fn begin_section(&mut self, occurrence: u32, span: ByteSpan) -> Result<(), DxfError> {
        if self.open.is_some() {
            self.mark_name_missing()?;
            self.record_diagnostic(DxfDiagnosticCode::SECTION_INTERRUPTED, Some(span))?;
            self.finish_open(DxfAsciiSectionClosure::Interrupted, occurrence, occurrence)?;
        }
        let content_start = occurrence.checked_add(1).ok_or_else(invalid_source_data)?;
        self.open = Some(OpenSection {
            marker_occurrence: occurrence,
            marker_span: span,
            name_span: None,
            name: DxfAsciiSectionName::Missing,
            awaiting_name: true,
            content_start,
        });
        self.increment_inside_count()
    }

    fn finish_before_eof(&mut self, occurrence: u32, span: ByteSpan) -> Result<(), DxfError> {
        if self.open.is_some() {
            self.mark_name_missing()?;
            self.record_diagnostic(DxfDiagnosticCode::SECTION_UNCLOSED, Some(span))?;
            self.finish_open(DxfAsciiSectionClosure::Unclosed, occurrence, occurrence)?;
        }
        Ok(())
    }

    fn close_or_report_orphan(&mut self, occurrence: u32, span: ByteSpan) -> Result<(), DxfError> {
        if self.open.is_some() {
            let group_end = occurrence.checked_add(1).ok_or_else(invalid_source_data)?;
            self.finish_open(DxfAsciiSectionClosure::Closed, group_end, occurrence)
        } else {
            self.record_diagnostic(DxfDiagnosticCode::SECTION_END_ORPHAN, Some(span))
        }
    }

    fn resolve_name(&mut self, group: DxfAsciiGroup<'_>, occurrence: u32) -> Result<(), DxfError> {
        let invalid = group.group_code().value() != 2;
        if let Some(open) = self.open.as_mut()
            && open.awaiting_name
        {
            open.name_span = Some(group.value_line().content_span());
            open.name = if invalid {
                DxfAsciiSectionName::InvalidGroupCode(group.group_code())
            } else {
                DxfAsciiSectionKind::from_bytes(group.raw_value())
                    .map_or(DxfAsciiSectionName::Unknown, DxfAsciiSectionName::Known)
            };
            open.awaiting_name = false;
            open.content_start = occurrence.saturating_add(1);
            if invalid {
                self.record_diagnostic(
                    DxfDiagnosticCode::SECTION_NAME_INVALID,
                    Some(group.value_line().content_span()),
                )?;
            }
        }
        Ok(())
    }

    fn mark_name_missing(&mut self) -> Result<(), DxfError> {
        let span = match self.open.as_ref() {
            Some(open) if open.awaiting_name => Some(open.marker_span),
            _ => return Ok(()),
        };
        if let Some(open) = self.open.as_mut() {
            open.awaiting_name = false;
        }
        self.record_diagnostic(DxfDiagnosticCode::SECTION_NAME_INVALID, span)
    }

    fn finish_open(
        &mut self,
        closure: DxfAsciiSectionClosure,
        group_end: u32,
        content_end: u32,
    ) -> Result<(), DxfError> {
        let open = self.open.take().ok_or_else(invalid_source_data)?;
        self.sections.try_reserve(1).map_err(|_| out_of_memory())?;
        self.sections.push(DxfAsciiSection {
            name_span: open.name_span,
            name: open.name,
            group_range: DxfAsciiGroupRange::new(open.marker_occurrence, group_end)?,
            content_range: DxfAsciiGroupRange::new(open.content_start, content_end)?,
            closure,
        });
        Ok(())
    }

    fn increment_inside_count(&mut self) -> Result<(), DxfError> {
        self.inside_section_group_count = self
            .inside_section_group_count
            .checked_add(1)
            .ok_or_else(invalid_source_data)?;
        Ok(())
    }

    fn record_diagnostic(
        &mut self,
        code: DxfDiagnosticCode,
        span: Option<ByteSpan>,
    ) -> Result<(), DxfError> {
        let retained = u64::try_from(self.diagnostics.len()).map_err(|_| invalid_source_data())?;
        if retained < self.max_diagnostics {
            self.diagnostics
                .try_reserve(1)
                .map_err(|_| out_of_memory())?;
            self.diagnostics.push(DxfDiagnostic::new(code, span));
        } else if !self.diagnostics_truncated {
            if let Some(last) = self.diagnostics.last_mut() {
                *last = DxfDiagnostic::new(DxfDiagnosticCode::DIAGNOSTICS_TRUNCATED, span);
            }
            self.diagnostics_truncated = true;
        }
        Ok(())
    }
}

fn compact_occurrence(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_source_data())
}

fn invalid_source_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::{
        DxfAsciiSection, DxfAsciiSectionClosure, DxfAsciiSectionKind, DxfAsciiSectionName,
        DxfAsciiStructureIndex,
    };
    use crate::{
        DxfAsciiDocumentConformance, DxfAsciiRawDocument, DxfCancellationToken, DxfDiagnosticCode,
        DxfGroupCode, DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
    };

    #[test]
    fn known_and_unknown_sections_are_fully_accounted_and_source_anchored()
    -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nCLASSES\n0\nCLASS\n1\nX\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0.0\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nDICTIONARY\n0\nENDSEC\n0\nSECTION\n2\nTHUMBNAILIMAGE\n90\n0\n0\nENDSEC\n0\nSECTION\n2\nACDSDATA\n0\nACDSRECORD\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let index = document.structure_index();
        assert_send_sync::<DxfAsciiStructureIndex>();
        assert_eq!(index.source_id(), document.source_id());
        assert_eq!(index.total_group_count(), 39);
        assert_eq!(index.inside_section_group_count(), 38);
        assert_eq!(index.outside_section_group_count(), 1);
        assert_eq!(
            index.inside_section_group_count() + index.outside_section_group_count(),
            document.groups().len() as u64
        );
        assert_eq!(section_group_sum(index), index.inside_section_group_count());
        assert!(index.diagnostics().is_empty());
        assert!(!index.diagnostics_were_truncated());

        let expected = [
            DxfAsciiSectionKind::Header,
            DxfAsciiSectionKind::Classes,
            DxfAsciiSectionKind::Tables,
            DxfAsciiSectionKind::Blocks,
            DxfAsciiSectionKind::Entities,
            DxfAsciiSectionKind::Objects,
            DxfAsciiSectionKind::ThumbnailImage,
        ];
        assert_eq!(index.sections().len(), 8);
        for (section, kind) in index.sections().iter().zip(expected) {
            assert_eq!(section.name(), DxfAsciiSectionName::Known(kind));
            assert_eq!(section.closure(), DxfAsciiSectionClosure::Closed);
            assert!(section.name_span().is_some());
        }
        let unknown = required_section(index, 7)?;
        assert_eq!(unknown.name(), DxfAsciiSectionName::Unknown);
        let mut name = [0_u8; 8];
        document.read_span(
            unknown
                .name_span()
                .ok_or(io::Error::other("missing unknown name span"))?,
            &mut name,
        )?;
        assert_eq!(&name, b"ACDSDATA");

        assert_eq!(index.section_ordinal_for_group(0), Some(0));
        assert_eq!(index.section_ordinal_for_group(4), Some(0));
        assert_eq!(index.section_ordinal_for_group(5), Some(1));
        assert_eq!(index.section_ordinal_for_group(38), None);
        assert_eq!(index.zero_group_count(), 25);
        assert_eq!(index.zero_group_occurrence(0), Some(0));
        assert_eq!(index.zero_group_range(0).map(|range| range.end()), Some(4));
        assert_eq!(
            index
                .zero_group_range(index.zero_group_count() - 1)
                .map(|range| (range.start(), range.end())),
            Some((38, 39))
        );
        Ok(())
    }

    #[test]
    fn malformed_envelopes_are_non_overlapping_and_diagnostic() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nENDSEC\n0\nSECTION\n0\nSECTION\n2\nENTITIES\n0\nLINE\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nDICTIONARY\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let index = document.structure_index();
        assert_eq!(index.total_group_count(), 10);
        assert_eq!(index.inside_section_group_count(), 8);
        assert_eq!(index.outside_section_group_count(), 2);
        assert_eq!(index.sections().len(), 3);
        assert_eq!(section_group_sum(index), index.inside_section_group_count());

        let first = required_section(index, 0)?;
        assert_eq!(first.name(), DxfAsciiSectionName::Missing);
        assert_eq!(first.closure(), DxfAsciiSectionClosure::Interrupted);
        assert_eq!(
            (first.group_range().start(), first.group_range().end()),
            (1, 2)
        );
        let second = required_section(index, 1)?;
        assert_eq!(
            second.name(),
            DxfAsciiSectionName::Known(DxfAsciiSectionKind::Entities)
        );
        assert_eq!(second.closure(), DxfAsciiSectionClosure::Closed);
        assert_eq!(
            (second.content_range().start(), second.content_range().end()),
            (4, 5)
        );
        let third = required_section(index, 2)?;
        assert_eq!(third.closure(), DxfAsciiSectionClosure::Unclosed);
        assert_eq!(
            (third.group_range().start(), third.group_range().end()),
            (6, 9)
        );

        let codes: Vec<_> = index
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code())
            .collect();
        assert_eq!(
            codes,
            [
                DxfDiagnosticCode::SECTION_END_ORPHAN,
                DxfDiagnosticCode::SECTION_NAME_INVALID,
                DxfDiagnosticCode::SECTION_INTERRUPTED,
                DxfDiagnosticCode::SECTION_UNCLOSED,
            ]
        );
        Ok(())
    }

    #[test]
    fn invalid_name_group_is_not_guessed() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n9\nHEADER\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let section = required_section(document.structure_index(), 0)?;
        let code_9 = DxfGroupCode::new(9).ok_or(io::Error::other("group code"))?;
        assert_eq!(
            section.name(),
            DxfAsciiSectionName::InvalidGroupCode(code_9)
        );
        assert_eq!(
            (
                section.content_range().start(),
                section.content_range().end()
            ),
            (2, 2)
        );
        assert_eq!(
            document.structure_index().diagnostics()[0].code(),
            DxfDiagnosticCode::SECTION_NAME_INVALID
        );
        Ok(())
    }

    #[test]
    fn compatible_eof_does_not_normalize_section_semantics() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nsection\n0\nSECTION\n2\n entities \n0\n ENDSEC\n0\n EOF \n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::compatible())?;
        let index = document.structure_index();
        assert_eq!(
            document.conformance(),
            DxfAsciiDocumentConformance::Recovered
        );
        assert_eq!(index.sections().len(), 1);
        let section = required_section(index, 0)?;
        assert_eq!(section.name(), DxfAsciiSectionName::Unknown);
        assert_eq!(section.closure(), DxfAsciiSectionClosure::Unclosed);
        assert_eq!(
            (section.group_range().start(), section.group_range().end()),
            (1, 4)
        );
        assert_eq!(index.inside_section_group_count(), 3);
        assert_eq!(index.outside_section_group_count(), 2);
        assert_eq!(index.zero_group_count(), 4);
        assert_eq!(
            index.diagnostics()[0].code(),
            DxfDiagnosticCode::SECTION_UNCLOSED
        );
        Ok(())
    }

    #[test]
    fn hostile_diagnostics_are_capped_by_the_resource_profile() -> Result<(), Box<dyn Error>> {
        let mut text = String::new();
        for _ in 0..10_005 {
            text.push_str("0\nENDSEC\n");
        }
        text.push_str("0\nEOF\n");
        let source = DxfMemorySource::new(text.as_bytes(), DxfResourceProfile::Safe)?;
        let document = open(&source, DxfReadOptions::strict())?;
        let index = document.structure_index();
        assert_eq!(index.diagnostics().len(), 10_000);
        assert!(index.diagnostics_were_truncated());
        assert_eq!(
            index
                .diagnostics()
                .last()
                .ok_or(io::Error::other("missing diagnostic"))?
                .code(),
            DxfDiagnosticCode::DIAGNOSTICS_TRUNCATED
        );
        assert_eq!(index.zero_group_count(), 10_006);
        assert_eq!(index.inside_section_group_count(), 0);
        Ok(())
    }

    #[test]
    fn section_metadata_stays_compact() {
        assert!(std::mem::size_of::<DxfAsciiSection>() <= 72);
    }

    fn open<'a>(
        source: &'a DxfMemorySource<'_>,
        options: DxfReadOptions,
    ) -> Result<DxfAsciiRawDocument<'a>, crate::DxfError> {
        let cancellation = DxfCancellationToken::default();
        let mut observer = NoopDxfReadObserver;
        DxfAsciiRawDocument::open(source, options, &cancellation, &mut observer)
    }

    fn required_section(
        index: &DxfAsciiStructureIndex,
        ordinal: usize,
    ) -> Result<DxfAsciiSection, io::Error> {
        index
            .sections()
            .get(ordinal)
            .copied()
            .ok_or(io::Error::other("missing section"))
    }

    fn section_group_sum(index: &DxfAsciiStructureIndex) -> u64 {
        index
            .sections()
            .iter()
            .map(|section| section.group_range().end() - section.group_range().start())
            .sum()
    }

    fn assert_send_sync<T: Send + Sync>() {}
}

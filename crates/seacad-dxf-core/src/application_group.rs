//! Record-scoped application-control group evidence from DXF group code 102.

use std::io;

use crate::{
    ByteSpan, DxfAsciiGroupRange, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord, DxfSourceId,
};

/// Exact known or application-defined group kind introduced by a 102 marker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfApplicationGroupKind {
    AcadReactors,
    AcadXDictionary,
    Other,
}

/// Lexical role of one group-code 102 occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfApplicationControlKind {
    Start(DxfApplicationGroupKind),
    Close,
    Invalid,
}

/// Whether one application-control group closes within its raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfApplicationGroupState {
    Closed,
    Interrupted,
    Unclosed,
}

/// One source-anchored group-code 102 control occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfApplicationControlEntry {
    record: DxfRawRecord,
    group: DxfRawGroup,
    kind: DxfApplicationControlKind,
}

impl DxfApplicationControlEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn kind(self) -> DxfApplicationControlKind {
        self.kind
    }
}

/// One start marker and its bounded record-local application group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfApplicationGroupEntry {
    record: DxfRawRecord,
    kind: DxfApplicationGroupKind,
    start_control_ordinal: u32,
    end_control_ordinal: Option<u32>,
    source_group_range: DxfAsciiGroupRange,
    content_group_range: DxfAsciiGroupRange,
    state: DxfApplicationGroupState,
}

impl DxfApplicationGroupEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfApplicationGroupKind {
        self.kind
    }

    #[must_use]
    pub const fn start_control_ordinal(self) -> u64 {
        self.start_control_ordinal as u64
    }

    #[must_use]
    pub const fn end_control_ordinal(self) -> Option<u64> {
        match self.end_control_ordinal {
            Some(ordinal) => Some(ordinal as u64),
            None => None,
        }
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
    pub const fn state(self) -> DxfApplicationGroupState {
        self.state
    }
}

/// Immutable source-order evidence for record-local group-code 102 controls.
///
/// A new start interrupts any open group. A close without an open group remains
/// visible as a control entry but does not create a group. Open groups never
/// cross raw-record boundaries.
#[derive(Debug)]
pub struct DxfApplicationGroupDirectory {
    source_id: DxfSourceId,
    record_count: u32,
    controls: Box<[DxfApplicationControlEntry]>,
    groups: Box<[DxfApplicationGroupEntry]>,
}

impl DxfApplicationGroupDirectory {
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
        let mut controls = Vec::new();
        let mut groups = Vec::new();
        for record in records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let mut open_group_index = None;
            for occurrence in record.group_range().start()..record.group_range().end() {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                if group.group_code().value() != 102 {
                    continue;
                }
                let kind = classify_control(document, group, cancellation)?;
                controls.try_reserve(1).map_err(|_| out_of_memory())?;
                let control_ordinal = compact_len(controls.len())?;
                controls.push(DxfApplicationControlEntry {
                    record,
                    group,
                    kind,
                });

                match kind {
                    DxfApplicationControlKind::Start(group_kind) => {
                        if let Some(index) = open_group_index.take() {
                            close_group(
                                &mut groups,
                                index,
                                occurrence,
                                None,
                                DxfApplicationGroupState::Interrupted,
                            )?;
                        }
                        let content_start = compact_occurrence(
                            occurrence
                                .checked_add(1)
                                .ok_or_else(invalid_internal_data)?,
                        )?;
                        let record_end = compact_occurrence(record.group_range().end())?;
                        groups.try_reserve(1).map_err(|_| out_of_memory())?;
                        groups.push(DxfApplicationGroupEntry {
                            record,
                            kind: group_kind,
                            start_control_ordinal: control_ordinal,
                            end_control_ordinal: None,
                            source_group_range: DxfAsciiGroupRange::new(
                                compact_occurrence(occurrence)?,
                                record_end,
                            )?,
                            content_group_range: DxfAsciiGroupRange::new(
                                content_start,
                                record_end,
                            )?,
                            state: DxfApplicationGroupState::Unclosed,
                        });
                        open_group_index = Some(groups.len() - 1);
                    }
                    DxfApplicationControlKind::Close => {
                        if let Some(index) = open_group_index.take() {
                            close_group(
                                &mut groups,
                                index,
                                occurrence,
                                Some(control_ordinal),
                                DxfApplicationGroupState::Closed,
                            )?;
                        }
                    }
                    DxfApplicationControlKind::Invalid => {}
                }
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            record_count,
            controls: controls.into_boxed_slice(),
            groups: groups.into_boxed_slice(),
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
    pub fn controls(&self) -> &[DxfApplicationControlEntry] {
        &self.controls
    }

    #[must_use]
    pub fn groups(&self) -> &[DxfApplicationGroupEntry] {
        &self.groups
    }

    #[must_use]
    pub fn control(&self, ordinal: u64) -> Option<DxfApplicationControlEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.controls.get(index).copied()
    }

    #[must_use]
    pub fn group(&self, ordinal: u64) -> Option<DxfApplicationGroupEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.groups.get(index).copied()
    }

    #[must_use]
    pub fn controls_for_record(
        &self,
        record_ordinal: u64,
    ) -> Option<&[DxfApplicationControlEntry]> {
        if record_ordinal >= self.record_count() {
            return None;
        }
        let start = self
            .controls
            .partition_point(|entry| entry.record().ordinal() < record_ordinal);
        let end = self
            .controls
            .partition_point(|entry| entry.record().ordinal() <= record_ordinal);
        self.controls.get(start..end)
    }

    #[must_use]
    pub fn groups_for_record(&self, record_ordinal: u64) -> Option<&[DxfApplicationGroupEntry]> {
        if record_ordinal >= self.record_count() {
            return None;
        }
        let start = self
            .groups
            .partition_point(|entry| entry.record().ordinal() < record_ordinal);
        let end = self
            .groups
            .partition_point(|entry| entry.record().ordinal() <= record_ordinal);
        self.groups.get(start..end)
    }

    #[must_use]
    pub fn group_for_content_occurrence(
        &self,
        occurrence: u64,
    ) -> Option<DxfApplicationGroupEntry> {
        let index = self
            .groups
            .partition_point(|entry| entry.content_group_range().end() <= occurrence);
        self.groups.get(index).copied().filter(|entry| {
            occurrence >= entry.content_group_range().start()
                && occurrence < entry.content_group_range().end()
        })
    }
}

impl DxfRawDocumentView<'_> {
    /// Indexes exact record-local application-control groups from code 102.
    pub fn application_group_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfApplicationGroupDirectory, DxfError> {
        DxfApplicationGroupDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn application_group_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfApplicationGroupDirectory, DxfError> {
        DxfRawDocumentView::from(self).application_group_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn application_group_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfApplicationGroupDirectory, DxfError> {
        DxfRawDocumentView::from(self).application_group_directory(cancellation)
    }
}

fn classify_control(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<DxfApplicationControlKind, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let span = group.value_payload_span();
    if document.raw_span_equals_exact(span, b"{ACAD_REACTORS")? {
        return Ok(DxfApplicationControlKind::Start(
            DxfApplicationGroupKind::AcadReactors,
        ));
    }
    ensure_not_cancelled(cancellation)?;
    if document.raw_span_equals_exact(span, b"{ACAD_XDICTIONARY")? {
        return Ok(DxfApplicationControlKind::Start(
            DxfApplicationGroupKind::AcadXDictionary,
        ));
    }
    ensure_not_cancelled(cancellation)?;
    if document.raw_span_equals_exact(span, b"}")? {
        return Ok(DxfApplicationControlKind::Close);
    }
    ensure_not_cancelled(cancellation)?;
    if span.is_empty() {
        return Ok(DxfApplicationControlKind::Invalid);
    }
    let first_span =
        ByteSpan::from_start_and_len(span.start(), 1).ok_or_else(invalid_internal_data)?;
    let mut first = [0_u8; 1];
    document.read_span(first_span, &mut first)?;
    ensure_not_cancelled(cancellation)?;
    Ok(if first == *b"{" {
        DxfApplicationControlKind::Start(DxfApplicationGroupKind::Other)
    } else {
        DxfApplicationControlKind::Invalid
    })
}

fn close_group(
    groups: &mut [DxfApplicationGroupEntry],
    index: usize,
    boundary: u64,
    end_control_ordinal: Option<u32>,
    state: DxfApplicationGroupState,
) -> Result<(), DxfError> {
    let entry = groups.get_mut(index).ok_or_else(invalid_internal_data)?;
    let source_end = match end_control_ordinal {
        Some(_) => boundary.checked_add(1).ok_or_else(invalid_internal_data)?,
        None => boundary,
    };
    entry.source_group_range = DxfAsciiGroupRange::new(
        compact_occurrence(entry.source_group_range.start())?,
        compact_occurrence(source_end)?,
    )?;
    entry.content_group_range = DxfAsciiGroupRange::new(
        compact_occurrence(entry.content_group_range.start())?,
        compact_occurrence(boundary)?,
    )?;
    entry.end_control_ordinal = end_control_ordinal;
    entry.state = state;
    Ok(())
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn compact_occurrence(occurrence: u64) -> Result<u32, DxfError> {
    u32::try_from(occurrence).map_err(|_| invalid_internal_data())
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

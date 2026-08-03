//! Bounded structural validation for entity XDATA application lists.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityXDataApplication,
    DxfEntityXDataApplicationState, DxfEntityXDataDirectory, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawGroup, DxfSourceId, source_span::span_equals_bytes,
};

/// One structural failure in an entity XDATA application list.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataStructureIssueKind {
    ApplicationNameTooLong { byte_count: u64 },
    Interrupted,
    InvalidControlString { group: DxfRawGroup },
    UnexpectedListEnd { group: DxfRawGroup },
    UnclosedLists { open_count: u32 },
}

/// One application-bound structural issue in deterministic validation order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataStructureIssue {
    application: DxfEntityXDataApplication,
    kind: DxfEntityXDataStructureIssueKind,
}

impl DxfEntityXDataStructureIssue {
    #[must_use]
    pub const fn application(self) -> DxfEntityXDataApplication {
        self.application
    }

    #[must_use]
    pub const fn kind(self) -> DxfEntityXDataStructureIssueKind {
        self.kind
    }
}

/// Half-open range in the directory's issue array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataStructureIssueRange {
    start: u32,
    end: u32,
}

impl DxfEntityXDataStructureIssueRange {
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

/// Whether one XDATA application passes the reviewed structural checks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataStructureState {
    Valid,
    Invalid { issue_count: u32 },
}

/// One source application and its structural validation result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataStructureEntry {
    application: DxfEntityXDataApplication,
    state: DxfEntityXDataStructureState,
    issues: DxfEntityXDataStructureIssueRange,
}

impl DxfEntityXDataStructureEntry {
    #[must_use]
    pub const fn application(self) -> DxfEntityXDataApplication {
        self.application
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataStructureState {
        self.state
    }

    #[must_use]
    pub const fn issue_range(self) -> DxfEntityXDataStructureIssueRange {
        self.issues
    }
}

/// Source-order XDATA application structure with exact typed failures.
#[derive(Debug)]
pub struct DxfEntityXDataStructureDirectory {
    source_id: DxfSourceId,
    xdata: DxfEntityXDataDirectory,
    entries: Box<[DxfEntityXDataStructureEntry]>,
    issues: Box<[DxfEntityXDataStructureIssue]>,
}

impl DxfEntityXDataStructureDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let xdata = document.entity_xdata_directory(cancellation)?;
        ensure_source(document.source_id(), xdata.source_id())?;
        let mut entries = Vec::new();
        let mut issues = Vec::new();
        entries
            .try_reserve_exact(xdata.applications().len())
            .map_err(|_| out_of_memory())?;
        for application in xdata.applications().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(issues.len())?;
            validate_application(document, &xdata, application, cancellation, &mut issues)?;
            let end = compact_len(issues.len())?;
            let issue_count = end.checked_sub(start).ok_or_else(invalid_internal_data)?;
            let state = if issue_count == 0 {
                DxfEntityXDataStructureState::Valid
            } else {
                DxfEntityXDataStructureState::Invalid { issue_count }
            };
            entries.push(DxfEntityXDataStructureEntry {
                application,
                state,
                issues: DxfEntityXDataStructureIssueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            xdata,
            entries: entries.into_boxed_slice(),
            issues: issues.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn xdata_directory(&self) -> &DxfEntityXDataDirectory {
        &self.xdata
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataStructureEntry] {
        &self.entries
    }

    #[must_use]
    pub fn issues(&self) -> &[DxfEntityXDataStructureIssue] {
        &self.issues
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataStructureEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entry_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<DxfEntityXDataStructureEntry, DxfError> {
        ensure_source(self.source_id, application.entity().source_id())?;
        self.entry(application.ordinal())
            .filter(|entry| entry.application() == application)
            .ok_or_else(invalid_internal_data)
    }

    pub fn issues_for_entry(
        &self,
        entry: DxfEntityXDataStructureEntry,
    ) -> Result<&[DxfEntityXDataStructureIssue], DxfError> {
        let observed = self.entry_for_application(entry.application())?;
        if observed != entry {
            return Err(invalid_internal_data());
        }
        let range = entry.issue_range();
        self.issues
            .get(
                usize::try_from(range.start()).map_err(|_| invalid_internal_data())?
                    ..usize::try_from(range.end()).map_err(|_| invalid_internal_data())?,
            )
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_structure_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataStructureDirectory, DxfError> {
        DxfEntityXDataStructureDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_xdata_structure_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataStructureDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_structure_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_xdata_structure_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataStructureDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_structure_directory(cancellation)
    }
}

fn validate_application(
    document: DxfRawDocumentView<'_>,
    xdata: &DxfEntityXDataDirectory,
    application: DxfEntityXDataApplication,
    cancellation: &DxfCancellationToken,
    issues: &mut Vec<DxfEntityXDataStructureIssue>,
) -> Result<(), DxfError> {
    let name_bytes = application.application_name().value_payload_span().len();
    if name_bytes > 31 {
        push_issue(
            issues,
            application,
            DxfEntityXDataStructureIssueKind::ApplicationNameTooLong {
                byte_count: name_bytes,
            },
        )?;
    }
    let mut open_count = 0_u32;
    for occurrence in xdata
        .occurrences_for_application(application)?
        .iter()
        .copied()
        .skip(1)
    {
        ensure_not_cancelled(cancellation)?;
        let group = occurrence.group();
        if group.group_code().value() != 1002 {
            continue;
        }
        let span = group.value_payload_span();
        if span_equals_bytes(document, span, b"{", cancellation)? {
            open_count = open_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
        } else if span_equals_bytes(document, span, b"}", cancellation)? {
            if open_count == 0 {
                push_issue(
                    issues,
                    application,
                    DxfEntityXDataStructureIssueKind::UnexpectedListEnd { group },
                )?;
            } else {
                open_count -= 1;
            }
        } else {
            push_issue(
                issues,
                application,
                DxfEntityXDataStructureIssueKind::InvalidControlString { group },
            )?;
        }
    }
    if application.state() == DxfEntityXDataApplicationState::Interrupted {
        push_issue(
            issues,
            application,
            DxfEntityXDataStructureIssueKind::Interrupted,
        )?;
    }
    if open_count != 0 {
        push_issue(
            issues,
            application,
            DxfEntityXDataStructureIssueKind::UnclosedLists { open_count },
        )?;
    }
    Ok(())
}

fn push_issue(
    issues: &mut Vec<DxfEntityXDataStructureIssue>,
    application: DxfEntityXDataApplication,
    kind: DxfEntityXDataStructureIssueKind,
) -> Result<(), DxfError> {
    issues.try_reserve(1).map_err(|_| out_of_memory())?;
    issues.push(DxfEntityXDataStructureIssue { application, kind });
    Ok(())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
}

fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
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

//! Record-scoped object-identity candidates and parsed-handle lookup.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandle,
    DxfHandleGroupClass, DxfHandleParseIssue, DxfIoOperation, DxfRawDocumentView,
    DxfRawHandleLookup, DxfRawHandleValue, DxfRawRecord, DxfSourceId,
    classify_dxf_handle_group_code,
};

/// Half-open range of candidate ordinals in a handle-identity directory.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandleIdentityCandidateRange {
    start: u32,
    end: u32,
}

impl DxfHandleIdentityCandidateRange {
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

/// Record-local classification of group-code 5 or 105 identity candidates.
///
/// `Parsed` is lexical evidence only. It does not prove uniqueness, target
/// existence, record type, or whether a null value is a usable object identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleIdentityState {
    Absent,
    UniqueParsed(DxfHandle),
    UniqueInvalid(DxfHandleParseIssue),
    Multiple { candidate_count: u32 },
}

/// Identity evidence for one raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandleIdentityEntry {
    record: DxfRawRecord,
    candidate_range: DxfHandleIdentityCandidateRange,
    state: DxfHandleIdentityState,
}

impl DxfHandleIdentityEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn candidate_range(self) -> DxfHandleIdentityCandidateRange {
        self.candidate_range
    }

    #[must_use]
    pub const fn state(self) -> DxfHandleIdentityState {
        self.state
    }
}

/// One uniquely parsed record candidate retained in handle-sorted lookup order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandleIdentityMatch {
    handle: DxfHandle,
    record: DxfRawRecord,
    candidate: DxfRawHandleValue,
}

impl DxfHandleIdentityMatch {
    #[must_use]
    pub const fn handle(self) -> DxfHandle {
        self.handle
    }

    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn candidate(self) -> DxfRawHandleValue {
        self.candidate
    }
}

/// Exact result of looking up uniquely parsed record candidates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleIdentityLookup<'a> {
    Missing,
    Unique(DxfHandleIdentityMatch),
    Ambiguous(&'a [DxfHandleIdentityMatch]),
}

/// Immutable record identity-evidence directory with duplicate-preserving lookup.
///
/// Records with invalid or multiple identity candidates remain visible through
/// `entries` and `candidates_for_record`, but do not participate in handle lookup.
#[derive(Debug)]
pub struct DxfHandleIdentityDirectory {
    source_id: DxfSourceId,
    entries: Box<[DxfHandleIdentityEntry]>,
    candidates: Box<[DxfRawHandleValue]>,
    matches: Box<[DxfHandleIdentityMatch]>,
}

impl DxfHandleIdentityDirectory {
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

        let mut entries = Vec::new();
        let mut candidates = Vec::new();
        let mut matches = Vec::new();
        entries
            .try_reserve(records.records().len())
            .map_err(|_| out_of_memory())?;

        for record in records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let candidate_start = compact_len(candidates.len())?;
            for occurrence in record.group_range().start()..record.group_range().end() {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                if classify_dxf_handle_group_code(group.group_code())
                    != Some(DxfHandleGroupClass::ObjectIdentity)
                {
                    continue;
                }
                let candidate = match document.raw_handle_at(occurrence, cancellation)? {
                    DxfRawHandleLookup::Handle(value)
                        if value.class() == DxfHandleGroupClass::ObjectIdentity =>
                    {
                        value
                    }
                    DxfRawHandleLookup::MissingOccurrence
                    | DxfRawHandleLookup::NotHandleGroup(_)
                    | DxfRawHandleLookup::Handle(_) => return Err(invalid_internal_data()),
                };
                candidates.try_reserve(1).map_err(|_| out_of_memory())?;
                candidates.push(candidate);
            }
            let candidate_end = compact_len(candidates.len())?;
            let candidate_range =
                DxfHandleIdentityCandidateRange::new(candidate_start, candidate_end)?;
            let state = identity_state(&candidates, candidate_range)?;
            let entry = DxfHandleIdentityEntry {
                record,
                candidate_range,
                state,
            };
            entries.push(entry);

            if let DxfHandleIdentityState::UniqueParsed(handle) = state {
                let index =
                    usize::try_from(candidate_start).map_err(|_| invalid_internal_data())?;
                let candidate = candidates
                    .get(index)
                    .copied()
                    .ok_or_else(invalid_internal_data)?;
                matches.try_reserve(1).map_err(|_| out_of_memory())?;
                matches.push(DxfHandleIdentityMatch {
                    handle,
                    record,
                    candidate,
                });
            }
        }

        ensure_not_cancelled(cancellation)?;
        matches.sort_unstable_by_key(|identity| (identity.handle, identity.record.ordinal()));
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entries: entries.into_boxed_slice(),
            candidates: candidates.into_boxed_slice(),
            matches: matches.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHandleIdentityEntry] {
        &self.entries
    }

    #[must_use]
    pub fn candidates(&self) -> &[DxfRawHandleValue] {
        &self.candidates
    }

    #[must_use]
    pub fn matches(&self) -> &[DxfHandleIdentityMatch] {
        &self.matches
    }

    #[must_use]
    pub fn entry(&self, record_ordinal: u64) -> Option<DxfHandleIdentityEntry> {
        let index = usize::try_from(record_ordinal).ok()?;
        self.entries
            .get(index)
            .copied()
            .filter(|entry| entry.record().ordinal() == record_ordinal)
    }

    #[must_use]
    pub fn candidates_for_record(&self, record_ordinal: u64) -> Option<&[DxfRawHandleValue]> {
        let entry = self.entry(record_ordinal)?;
        let start = usize::try_from(entry.candidate_range().start()).ok()?;
        let end = usize::try_from(entry.candidate_range().end()).ok()?;
        self.candidates.get(start..end)
    }

    #[must_use]
    pub fn matches_for_handle(&self, handle: DxfHandle) -> &[DxfHandleIdentityMatch] {
        let start = self
            .matches
            .partition_point(|identity| identity.handle < handle);
        let end = self
            .matches
            .partition_point(|identity| identity.handle <= handle);
        self.matches.get(start..end).map_or(&[], |matches| matches)
    }

    #[must_use]
    pub fn lookup(&self, handle: DxfHandle) -> DxfHandleIdentityLookup<'_> {
        match self.matches_for_handle(handle) {
            [] => DxfHandleIdentityLookup::Missing,
            [identity] => DxfHandleIdentityLookup::Unique(*identity),
            identities => DxfHandleIdentityLookup::Ambiguous(identities),
        }
    }
}

impl DxfRawDocumentView<'_> {
    /// Indexes record-local group-code 5 and 105 identity evidence.
    pub fn handle_identity_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleIdentityDirectory, DxfError> {
        DxfHandleIdentityDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn handle_identity_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleIdentityDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_identity_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn handle_identity_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleIdentityDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_identity_directory(cancellation)
    }
}

fn identity_state(
    candidates: &[DxfRawHandleValue],
    range: DxfHandleIdentityCandidateRange,
) -> Result<DxfHandleIdentityState, DxfError> {
    let start = usize::try_from(range.start()).map_err(|_| invalid_internal_data())?;
    let end = usize::try_from(range.end()).map_err(|_| invalid_internal_data())?;
    match candidates
        .get(start..end)
        .ok_or_else(invalid_internal_data)?
    {
        [] => Ok(DxfHandleIdentityState::Absent),
        [candidate] => match candidate.parse_result() {
            Ok(handle) => Ok(DxfHandleIdentityState::UniqueParsed(handle)),
            Err(issue) => Ok(DxfHandleIdentityState::UniqueInvalid(issue)),
        },
        multiple => Ok(DxfHandleIdentityState::Multiple {
            candidate_count: compact_len(multiple.len())?,
        }),
    }
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

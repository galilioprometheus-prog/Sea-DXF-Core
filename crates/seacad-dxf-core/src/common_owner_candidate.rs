//! Per-record common-owner pointer candidate evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHandleIdentityMatch, DxfHandleResolutionState, DxfHandleRoleDirectory, DxfHandleRoleEntry,
    DxfHandleRoleEvidence, DxfIoOperation, DxfRawDocumentView, DxfRawRecord, DxfSourceId,
};

/// Half-open range of common-owner candidate ordinals.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCommonOwnerCandidateRange {
    start: u32,
    end: u32,
}

impl DxfCommonOwnerCandidateRange {
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

/// One source-order M7.4e common-owner pointer candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCommonOwnerCandidateEntry {
    role_ordinal: u32,
    role: DxfHandleRoleEntry,
}

impl DxfCommonOwnerCandidateEntry {
    #[must_use]
    pub const fn role_ordinal(self) -> u64 {
        self.role_ordinal as u64
    }

    #[must_use]
    pub const fn role(self) -> DxfHandleRoleEntry {
        self.role
    }

    #[must_use]
    pub const fn state(self) -> DxfHandleResolutionState {
        self.role.contextual().resolution().state()
    }
}

/// Cardinality of common-owner pointer candidates in one raw record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCommonOwnerCandidateState {
    NoCandidate,
    UniqueCandidate,
    MultipleCandidates { candidate_count: u32 },
}

/// One raw record and its source-order common-owner candidate range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCommonOwnerRecordEntry {
    record: DxfRawRecord,
    candidate_range: DxfCommonOwnerCandidateRange,
    state: DxfCommonOwnerCandidateState,
}

impl DxfCommonOwnerRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn candidate_range(self) -> DxfCommonOwnerCandidateRange {
        self.candidate_range
    }

    #[must_use]
    pub const fn state(self) -> DxfCommonOwnerCandidateState {
        self.state
    }
}

/// Immutable common-owner pointer candidates grouped by source record.
///
/// Every M7.4e `CommonOwnerPointerCandidate` remains visible regardless of its
/// lexical or target-resolution state. Cardinality describes candidate shape
/// only and is not one-owner conformance or an authoritative owner decision.
#[derive(Debug)]
pub struct DxfCommonOwnerCandidateDirectory {
    source_id: DxfSourceId,
    roles: DxfHandleRoleDirectory,
    entries: Box<[DxfCommonOwnerCandidateEntry]>,
    records: Box<[DxfCommonOwnerRecordEntry]>,
}

impl DxfCommonOwnerCandidateDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let roles = document.handle_role_directory(cancellation)?;
        if roles.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: roles.source_id(),
            });
        }

        let mut entries = Vec::new();
        entries
            .try_reserve(roles.entries().len())
            .map_err(|_| out_of_memory())?;
        for (role_index, role) in roles.entries().iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            if role.role() != DxfHandleRoleEvidence::CommonOwnerPointerCandidate {
                continue;
            }
            entries.push(DxfCommonOwnerCandidateEntry {
                role_ordinal: compact_len(role_index)?,
                role,
            });
        }
        ensure_not_cancelled(cancellation)?;

        let identity_entries = roles
            .contextual_directory()
            .resolution_directory()
            .identity_directory()
            .entries();
        let mut records = Vec::new();
        records
            .try_reserve(identity_entries.len())
            .map_err(|_| out_of_memory())?;
        let mut candidate_cursor = 0_usize;
        for identity in identity_entries.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(candidate_cursor)?;
            while entries.get(candidate_cursor).is_some_and(|entry| {
                entry
                    .role()
                    .contextual()
                    .resolution()
                    .reference()
                    .record()
                    .ordinal()
                    == identity.record().ordinal()
            }) {
                candidate_cursor = candidate_cursor
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
            }
            let end = compact_len(candidate_cursor)?;
            let candidate_range = DxfCommonOwnerCandidateRange::new(start, end)?;
            let state = match candidate_range.len() {
                0 => DxfCommonOwnerCandidateState::NoCandidate,
                1 => DxfCommonOwnerCandidateState::UniqueCandidate,
                candidate_count => DxfCommonOwnerCandidateState::MultipleCandidates {
                    candidate_count: u32::try_from(candidate_count)
                        .map_err(|_| invalid_internal_data())?,
                },
            };
            records.push(DxfCommonOwnerRecordEntry {
                record: identity.record(),
                candidate_range,
                state,
            });
        }
        if candidate_cursor != entries.len() {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;

        Ok(Self {
            source_id: document.source_id(),
            roles,
            entries: entries.into_boxed_slice(),
            records: records.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn role_directory(&self) -> &DxfHandleRoleDirectory {
        &self.roles
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfCommonOwnerCandidateEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, candidate_ordinal: u64) -> Option<DxfCommonOwnerCandidateEntry> {
        let index = usize::try_from(candidate_ordinal).ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn record_entries(&self) -> &[DxfCommonOwnerRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn record_entry(&self, record_ordinal: u64) -> Option<DxfCommonOwnerRecordEntry> {
        let index = usize::try_from(record_ordinal).ok()?;
        self.records
            .get(index)
            .copied()
            .filter(|entry| entry.record().ordinal() == record_ordinal)
    }

    #[must_use]
    pub fn candidates_for_record(
        &self,
        record_ordinal: u64,
    ) -> Option<&[DxfCommonOwnerCandidateEntry]> {
        let entry = self.record_entry(record_ordinal)?;
        let start = usize::try_from(entry.candidate_range().start()).ok()?;
        let end = usize::try_from(entry.candidate_range().end()).ok()?;
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn targets_for_candidate(
        &self,
        candidate_ordinal: u64,
    ) -> Option<&[DxfHandleIdentityMatch]> {
        let entry = self.entry(candidate_ordinal)?;
        self.roles
            .contextual_directory()
            .resolution_directory()
            .targets_for_reference(entry.role_ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    /// Indexes conservative common-owner pointer candidates by source record.
    pub fn common_owner_candidate_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCommonOwnerCandidateDirectory, DxfError> {
        DxfCommonOwnerCandidateDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn common_owner_candidate_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCommonOwnerCandidateDirectory, DxfError> {
        DxfRawDocumentView::from(self).common_owner_candidate_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn common_owner_candidate_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCommonOwnerCandidateDirectory, DxfError> {
        DxfRawDocumentView::from(self).common_owner_candidate_directory(cancellation)
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

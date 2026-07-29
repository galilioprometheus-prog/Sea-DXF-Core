//! Conservative comparison of bidirectional owner evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfCommonOwnerCandidateDirectory, DxfCommonOwnerCandidateEntry, DxfCommonOwnerCandidateState,
    DxfCommonOwnerRecordEntry, DxfError, DxfHandleIdentityMatch, DxfHandleResolutionState,
    DxfIncomingOwnershipState, DxfIoOperation, DxfOwnershipEvidenceDirectory,
    DxfOwnershipEvidenceEntry, DxfOwnershipTargetEntry, DxfRawDocumentView, DxfRawRecord,
    DxfResolvedOwnershipLink, DxfSourceId,
};

/// Comparison state for one raw record's two ownership-evidence directions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfOwnerEvidenceComparisonState {
    NotComparable,
    Matched,
    Conflicting,
}

/// One raw record and its conservative bidirectional owner-evidence comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfOwnerEvidenceComparisonEntry {
    record: DxfRawRecord,
    common_owner: DxfCommonOwnerRecordEntry,
    incoming_ownership: DxfOwnershipTargetEntry,
    candidate_ordinal: Option<u32>,
    incoming_link_ordinal: Option<u32>,
    state: DxfOwnerEvidenceComparisonState,
}

impl DxfOwnerEvidenceComparisonEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn common_owner(self) -> DxfCommonOwnerRecordEntry {
        self.common_owner
    }

    #[must_use]
    pub const fn incoming_ownership(self) -> DxfOwnershipTargetEntry {
        self.incoming_ownership
    }

    #[must_use]
    pub const fn candidate_ordinal(self) -> Option<u64> {
        match self.candidate_ordinal {
            Some(ordinal) => Some(ordinal as u64),
            None => None,
        }
    }

    #[must_use]
    pub const fn incoming_link_ordinal(self) -> Option<u64> {
        match self.incoming_link_ordinal {
            Some(ordinal) => Some(ordinal as u64),
            None => None,
        }
    }

    #[must_use]
    pub const fn state(self) -> DxfOwnerEvidenceComparisonState {
        self.state
    }
}

/// Immutable per-record comparison of common-owner and incoming ownership evidence.
///
/// Comparison occurs only for exactly one uniquely resolved common-owner
/// candidate and exactly one incoming ownership-class link. All other shapes
/// remain `NotComparable`; no authoritative owner is selected.
#[derive(Debug)]
pub struct DxfOwnerEvidenceComparisonDirectory {
    source_id: DxfSourceId,
    candidates: DxfCommonOwnerCandidateDirectory,
    ownership: DxfOwnershipEvidenceDirectory,
    entries: Box<[DxfOwnerEvidenceComparisonEntry]>,
}

impl DxfOwnerEvidenceComparisonDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let candidates = document.common_owner_candidate_directory(cancellation)?;
        let ownership = document.ownership_evidence_directory(cancellation)?;
        for observed in [candidates.source_id(), ownership.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }
        if candidates.record_entries().len() != ownership.target_entries().len() {
            return Err(invalid_internal_data());
        }

        let mut entries = Vec::new();
        entries
            .try_reserve(candidates.record_entries().len())
            .map_err(|_| out_of_memory())?;
        for (common_owner, incoming_ownership) in candidates
            .record_entries()
            .iter()
            .copied()
            .zip(ownership.target_entries().iter().copied())
        {
            ensure_not_cancelled(cancellation)?;
            if common_owner.record() != incoming_ownership.record() {
                return Err(invalid_internal_data());
            }
            let (state, candidate_ordinal, incoming_link_ordinal) =
                compare_record(&candidates, common_owner, &ownership, incoming_ownership)?;
            entries.push(DxfOwnerEvidenceComparisonEntry {
                record: common_owner.record(),
                common_owner,
                incoming_ownership,
                candidate_ordinal,
                incoming_link_ordinal,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;

        Ok(Self {
            source_id: document.source_id(),
            candidates,
            ownership,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn common_owner_candidate_directory(&self) -> &DxfCommonOwnerCandidateDirectory {
        &self.candidates
    }

    #[must_use]
    pub const fn ownership_evidence_directory(&self) -> &DxfOwnershipEvidenceDirectory {
        &self.ownership
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfOwnerEvidenceComparisonEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, record_ordinal: u64) -> Option<DxfOwnerEvidenceComparisonEntry> {
        let index = usize::try_from(record_ordinal).ok()?;
        self.entries
            .get(index)
            .copied()
            .filter(|entry| entry.record().ordinal() == record_ordinal)
    }

    #[must_use]
    pub fn candidate_for_entry(&self, record_ordinal: u64) -> Option<DxfCommonOwnerCandidateEntry> {
        let ordinal = self.entry(record_ordinal)?.candidate_ordinal()?;
        self.candidates.entry(ordinal)
    }

    #[must_use]
    pub fn candidate_target_for_entry(
        &self,
        record_ordinal: u64,
    ) -> Option<DxfHandleIdentityMatch> {
        let ordinal = self.entry(record_ordinal)?.candidate_ordinal()?;
        let [target] = self.candidates.targets_for_candidate(ordinal)? else {
            return None;
        };
        Some(*target)
    }

    #[must_use]
    pub fn incoming_link_for_entry(&self, record_ordinal: u64) -> Option<DxfResolvedOwnershipLink> {
        let ordinal = self.entry(record_ordinal)?.incoming_link_ordinal()?;
        let index = usize::try_from(ordinal).ok()?;
        self.ownership.resolved_links().get(index).copied()
    }

    #[must_use]
    pub fn incoming_source_for_entry(
        &self,
        record_ordinal: u64,
    ) -> Option<DxfOwnershipEvidenceEntry> {
        let link = self.incoming_link_for_entry(record_ordinal)?;
        self.ownership.entry(link.evidence_ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    /// Compares conservative owner evidence in both reference directions.
    pub fn owner_evidence_comparison_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfOwnerEvidenceComparisonDirectory, DxfError> {
        DxfOwnerEvidenceComparisonDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn owner_evidence_comparison_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfOwnerEvidenceComparisonDirectory, DxfError> {
        DxfRawDocumentView::from(self).owner_evidence_comparison_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn owner_evidence_comparison_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfOwnerEvidenceComparisonDirectory, DxfError> {
        DxfRawDocumentView::from(self).owner_evidence_comparison_directory(cancellation)
    }
}

fn compare_record(
    candidates: &DxfCommonOwnerCandidateDirectory,
    common_owner: DxfCommonOwnerRecordEntry,
    ownership: &DxfOwnershipEvidenceDirectory,
    incoming_ownership: DxfOwnershipTargetEntry,
) -> Result<(DxfOwnerEvidenceComparisonState, Option<u32>, Option<u32>), DxfError> {
    if common_owner.state() != DxfCommonOwnerCandidateState::UniqueCandidate
        || incoming_ownership.state() != DxfIncomingOwnershipState::UniqueIncomingLink
    {
        return Ok((DxfOwnerEvidenceComparisonState::NotComparable, None, None));
    }

    let candidate_ordinal = compact_ordinal(common_owner.candidate_range().start())?;
    let [candidate] = candidates
        .candidates_for_record(common_owner.record().ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    if candidate.state() != DxfHandleResolutionState::Unique {
        return Ok((DxfOwnerEvidenceComparisonState::NotComparable, None, None));
    }
    let [candidate_target] = candidates
        .targets_for_candidate(candidate_ordinal as u64)
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };

    let incoming_link_ordinal = compact_ordinal(incoming_ownership.incoming_range().start())?;
    let [incoming_link] = ownership
        .incoming_links_for_target_record(incoming_ownership.record().ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    let incoming_source = ownership
        .entry(incoming_link.evidence_ordinal())
        .ok_or_else(invalid_internal_data)?
        .resolution()
        .reference()
        .record();
    let state = if candidate_target.record().ordinal() == incoming_source.ordinal() {
        DxfOwnerEvidenceComparisonState::Matched
    } else {
        DxfOwnerEvidenceComparisonState::Conflicting
    };
    Ok((state, Some(candidate_ordinal), Some(incoming_link_ordinal)))
}

fn compact_ordinal(ordinal: u64) -> Result<u32, DxfError> {
    u32::try_from(ordinal).map_err(|_| invalid_internal_data())
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

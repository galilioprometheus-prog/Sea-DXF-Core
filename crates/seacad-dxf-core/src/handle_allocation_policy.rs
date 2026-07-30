//! Fail-closed object-handle allocation proposals.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandle,
    DxfHandleIdentityDirectory, DxfHandleIdentityState, DxfHandleParseIssue, DxfHandseedOccurrence,
    DxfHandseedState, DxfHandseedValue, DxfIoOperation, DxfRawDocumentView, DxfResource,
    DxfResourceProfile, DxfSourceId,
};

/// Eligibility of one document for monotonic object-handle allocation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleAllocationPolicyState {
    Ready,
    HandseedUnavailable {
        state: DxfHandseedState,
    },
    NullHandseed,
    IdentityInvalid {
        record_ordinal: u64,
        issue: DxfHandleParseIssue,
    },
    IdentityMultiple {
        record_ordinal: u64,
        candidate_count: u32,
    },
    NullIdentity {
        record_ordinal: u64,
    },
    DuplicateIdentity {
        handle: DxfHandle,
        record_count: u32,
    },
    HandseedNotAboveOccupied {
        handseed: DxfHandle,
        greatest_occupied: DxfHandle,
    },
}

/// Constant-space sequence of consecutive handles and its successor seed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandleAllocationProposal {
    source_id: DxfSourceId,
    first_handle: DxfHandle,
    handle_count: u64,
    next_handseed: DxfHandle,
}

impl DxfHandleAllocationProposal {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn first_handle(self) -> DxfHandle {
        self.first_handle
    }

    #[must_use]
    pub const fn handle_count(self) -> u64 {
        self.handle_count
    }

    #[must_use]
    pub const fn next_handseed(self) -> DxfHandle {
        self.next_handseed
    }

    #[must_use]
    pub fn handle_at(self, index: u64) -> Option<DxfHandle> {
        if index >= self.handle_count {
            return None;
        }
        self.first_handle
            .value()
            .checked_add(index)
            .map(DxfHandle::from_u64)
    }
}

/// Result of requesting a bounded allocation proposal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleAllocationOutcome {
    Unavailable {
        state: DxfHandleAllocationPolicyState,
    },
    Exhausted {
        handseed: DxfHandle,
        requested_count: u64,
    },
    Proposed(DxfHandleAllocationProposal),
}

/// Immutable evidence directory used to propose monotonic object handles.
#[derive(Debug)]
pub struct DxfHandleAllocationPolicyDirectory {
    source_id: DxfSourceId,
    handseed_state: DxfHandseedState,
    handseed_occurrence: Option<DxfHandseedOccurrence>,
    handseed: Option<DxfHandle>,
    greatest_occupied: Option<DxfHandle>,
    state: DxfHandleAllocationPolicyState,
    identities: DxfHandleIdentityDirectory,
}

impl DxfHandleAllocationPolicyDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let handseed_report = document.handseed_report();
        if handseed_report.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: handseed_report.source_id(),
            });
        }
        let identities = document.handle_identity_directory(cancellation)?;
        if identities.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: identities.source_id(),
            });
        }

        let handseed_state = handseed_report.state();
        let handseed_occurrence = handseed_report.primary_occurrence();
        let handseed = if handseed_state == DxfHandseedState::Parsed {
            match handseed_occurrence.map(DxfHandseedOccurrence::value) {
                Some(DxfHandseedValue::Parsed(handle)) => Some(handle),
                _ => None,
            }
        } else {
            None
        };
        let greatest_occupied = identities
            .matches()
            .last()
            .map(|identity| identity.handle());
        let state = classify_policy(handseed_state, handseed, greatest_occupied, &identities)?;
        ensure_not_cancelled(cancellation)?;

        Ok(Self {
            source_id: document.source_id(),
            handseed_state,
            handseed_occurrence,
            handseed,
            greatest_occupied,
            state,
            identities,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn state(&self) -> DxfHandleAllocationPolicyState {
        self.state
    }

    #[must_use]
    pub const fn handseed_state(&self) -> DxfHandseedState {
        self.handseed_state
    }

    #[must_use]
    pub const fn handseed_occurrence(&self) -> Option<DxfHandseedOccurrence> {
        self.handseed_occurrence
    }

    #[must_use]
    pub const fn handseed(&self) -> Option<DxfHandle> {
        self.handseed
    }

    #[must_use]
    pub const fn greatest_occupied(&self) -> Option<DxfHandle> {
        self.greatest_occupied
    }

    #[must_use]
    pub const fn identity_directory(&self) -> &DxfHandleIdentityDirectory {
        &self.identities
    }

    pub fn propose_allocation(
        &self,
        handle_count: u64,
        profile: DxfResourceProfile,
    ) -> Result<DxfHandleAllocationOutcome, DxfError> {
        let limit = profile.limits().max_records();
        if handle_count > limit {
            return Err(DxfError::resource_limit(
                DxfResource::Records,
                limit,
                handle_count,
            ));
        }
        if self.state != DxfHandleAllocationPolicyState::Ready {
            return Ok(DxfHandleAllocationOutcome::Unavailable { state: self.state });
        }
        let Some(handseed) = self.handseed else {
            return Ok(DxfHandleAllocationOutcome::Unavailable {
                state: DxfHandleAllocationPolicyState::HandseedUnavailable {
                    state: self.handseed_state,
                },
            });
        };
        let Some(next_value) = handseed.value().checked_add(handle_count) else {
            return Ok(DxfHandleAllocationOutcome::Exhausted {
                handseed,
                requested_count: handle_count,
            });
        };
        Ok(DxfHandleAllocationOutcome::Proposed(
            DxfHandleAllocationProposal {
                source_id: self.source_id,
                first_handle: handseed,
                handle_count,
                next_handseed: DxfHandle::from_u64(next_value),
            },
        ))
    }
}

impl DxfRawDocumentView<'_> {
    /// Builds fail-closed evidence for monotonic object-handle allocation.
    pub fn handle_allocation_policy_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleAllocationPolicyDirectory, DxfError> {
        DxfHandleAllocationPolicyDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn handle_allocation_policy_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleAllocationPolicyDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_allocation_policy_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn handle_allocation_policy_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleAllocationPolicyDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_allocation_policy_directory(cancellation)
    }
}

fn classify_policy(
    handseed_state: DxfHandseedState,
    handseed: Option<DxfHandle>,
    greatest_occupied: Option<DxfHandle>,
    identities: &DxfHandleIdentityDirectory,
) -> Result<DxfHandleAllocationPolicyState, DxfError> {
    if handseed_state != DxfHandseedState::Parsed {
        return Ok(DxfHandleAllocationPolicyState::HandseedUnavailable {
            state: handseed_state,
        });
    }
    let Some(handseed) = handseed else {
        return Ok(DxfHandleAllocationPolicyState::HandseedUnavailable {
            state: handseed_state,
        });
    };
    if handseed.is_null() {
        return Ok(DxfHandleAllocationPolicyState::NullHandseed);
    }

    for entry in identities.entries() {
        match entry.state() {
            DxfHandleIdentityState::Absent | DxfHandleIdentityState::UniqueParsed(_) => {}
            DxfHandleIdentityState::UniqueInvalid(issue) => {
                return Ok(DxfHandleAllocationPolicyState::IdentityInvalid {
                    record_ordinal: entry.record().ordinal(),
                    issue,
                });
            }
            DxfHandleIdentityState::Multiple { candidate_count } => {
                return Ok(DxfHandleAllocationPolicyState::IdentityMultiple {
                    record_ordinal: entry.record().ordinal(),
                    candidate_count,
                });
            }
        }
        if matches!(
            entry.state(),
            DxfHandleIdentityState::UniqueParsed(handle) if handle.is_null()
        ) {
            return Ok(DxfHandleAllocationPolicyState::NullIdentity {
                record_ordinal: entry.record().ordinal(),
            });
        }
    }

    let matches = identities.matches();
    let mut start = 0_usize;
    while let Some(first) = matches.get(start) {
        let end = matches.partition_point(|identity| identity.handle() <= first.handle());
        let record_count = end.saturating_sub(start);
        if record_count > 1 {
            return Ok(DxfHandleAllocationPolicyState::DuplicateIdentity {
                handle: first.handle(),
                record_count: u32::try_from(record_count).map_err(|_| invalid_internal_data())?,
            });
        }
        start = end;
    }

    if let Some(greatest_occupied) = greatest_occupied
        && handseed <= greatest_occupied
    {
        return Ok(DxfHandleAllocationPolicyState::HandseedNotAboveOccupied {
            handseed,
            greatest_occupied,
        });
    }
    Ok(DxfHandleAllocationPolicyState::Ready)
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

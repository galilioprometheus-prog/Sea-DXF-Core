//! Source-bound BLOCK_RECORD owner binding for entity insertion placements.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfCommonOwnerCandidateDirectory, DxfCommonOwnerCandidateState, DxfEntityPlacement,
    DxfEntityPlacementDirectory, DxfEntityPlacementState, DxfEntityPlacementTarget, DxfError,
    DxfHandle, DxfHandleIdentityLookup, DxfHandleIdentityMatch, DxfHandleResolutionState,
    DxfIoOperation, DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry,
    DxfNamedSymbolTableKind, DxfRawDocumentView, DxfSourceId,
};

/// A placement paired with one exact, uniquely identified BLOCK_RECORD owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityPlacementOwnerBinding {
    source_id: DxfSourceId,
    placement: DxfEntityPlacement,
    owner_handle: DxfHandle,
    block_record: DxfNamedSymbolTableEntry,
}

impl DxfEntityPlacementOwnerBinding {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn placement(self) -> DxfEntityPlacement {
        self.placement
    }

    #[must_use]
    pub const fn owner_handle(self) -> DxfHandle {
        self.owner_handle
    }

    #[must_use]
    pub const fn block_record_entry(self) -> DxfNamedSymbolTableEntry {
        self.block_record
    }
}

/// Typed reason that a placement and requested owner cannot be bound.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityPlacementOwnerIssue {
    PlacementUnavailable {
        target: DxfEntityPlacementTarget,
    },
    NullOwner,
    OwnerMissing {
        handle: DxfHandle,
    },
    OwnerAmbiguous {
        handle: DxfHandle,
        target_count: u32,
    },
    OwnerNotBlockRecord {
        target: DxfHandleIdentityMatch,
    },
    BlockOwnerCardinality {
        state: DxfCommonOwnerCandidateState,
    },
    BlockOwnerResolution {
        state: DxfHandleResolutionState,
    },
    BlockOwnerMismatch {
        declared: DxfHandleIdentityMatch,
        requested: DxfHandleIdentityMatch,
    },
}

/// Result of validating one placement-owner request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityPlacementOwnerOutcome {
    Bound(DxfEntityPlacementOwnerBinding),
    Rejected(DxfEntityPlacementOwnerIssue),
}

/// Immutable owner validator composed from placement, symbol, and handle evidence.
#[derive(Debug)]
pub struct DxfEntityPlacementOwnerDirectory {
    source_id: DxfSourceId,
    placements: DxfEntityPlacementDirectory,
    symbols: DxfNamedSymbolTableDirectory,
    common_owners: DxfCommonOwnerCandidateDirectory,
}

impl DxfEntityPlacementOwnerDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let placements = document.entity_placement_directory(cancellation)?;
        let symbols = document.named_symbol_table_directory(cancellation)?;
        let common_owners = document.common_owner_candidate_directory(cancellation)?;
        for observed in [
            placements.source_id(),
            symbols.source_id(),
            common_owners.source_id(),
        ] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            placements,
            symbols,
            common_owners,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn placement_directory(&self) -> &DxfEntityPlacementDirectory {
        &self.placements
    }

    #[must_use]
    pub const fn named_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.symbols
    }

    #[must_use]
    pub const fn common_owner_candidate_directory(&self) -> &DxfCommonOwnerCandidateDirectory {
        &self.common_owners
    }

    pub fn bind(
        &self,
        placement: DxfEntityPlacement,
        requested_owner: DxfHandle,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityPlacementOwnerOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        if placement.source_id() != self.source_id {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: placement.source_id(),
            });
        }
        if !self.placement_is_ready(placement) {
            return Ok(DxfEntityPlacementOwnerOutcome::Rejected(
                DxfEntityPlacementOwnerIssue::PlacementUnavailable {
                    target: placement.target(),
                },
            ));
        }
        let (requested, block_record) = match self.resolve_requested_owner(requested_owner)? {
            Ok(value) => value,
            Err(issue) => return Ok(DxfEntityPlacementOwnerOutcome::Rejected(issue)),
        };
        if let DxfEntityPlacementTarget::BlockDefinition { raw_record_ordinal } = placement.target()
            && let Err(issue) = self.validate_block_owner(raw_record_ordinal, requested)?
        {
            return Ok(DxfEntityPlacementOwnerOutcome::Rejected(issue));
        }
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityPlacementOwnerOutcome::Bound(
            DxfEntityPlacementOwnerBinding {
                source_id: self.source_id,
                placement,
                owner_handle: requested.handle(),
                block_record,
            },
        ))
    }

    fn placement_is_ready(&self, placement: DxfEntityPlacement) -> bool {
        self.placements
            .assessment_for_target(placement.target())
            .is_some_and(|assessment| {
                matches!(assessment.state(), DxfEntityPlacementState::Ready(ready) if ready == placement)
            })
    }

    fn resolve_requested_owner(
        &self,
        requested_owner: DxfHandle,
    ) -> Result<
        Result<(DxfHandleIdentityMatch, DxfNamedSymbolTableEntry), DxfEntityPlacementOwnerIssue>,
        DxfError,
    > {
        if requested_owner.is_null() {
            return Ok(Err(DxfEntityPlacementOwnerIssue::NullOwner));
        }
        let identities = self
            .common_owners
            .role_directory()
            .contextual_directory()
            .resolution_directory()
            .identity_directory();
        let target = match identities.lookup(requested_owner) {
            DxfHandleIdentityLookup::Missing => {
                return Ok(Err(DxfEntityPlacementOwnerIssue::OwnerMissing {
                    handle: requested_owner,
                }));
            }
            DxfHandleIdentityLookup::Unique(target) => target,
            DxfHandleIdentityLookup::Ambiguous(targets) => {
                return Ok(Err(DxfEntityPlacementOwnerIssue::OwnerAmbiguous {
                    handle: requested_owner,
                    target_count: compact_len(targets.len())?,
                }));
            }
        };
        let Some(block_record) = self
            .symbols
            .entry_for_raw_ordinal(target.record().ordinal())
            .filter(|entry| entry.kind() == DxfNamedSymbolTableKind::BlockRecord)
        else {
            return Ok(Err(DxfEntityPlacementOwnerIssue::OwnerNotBlockRecord {
                target,
            }));
        };
        Ok(Ok((target, block_record)))
    }

    fn validate_block_owner(
        &self,
        raw_record_ordinal: u64,
        requested: DxfHandleIdentityMatch,
    ) -> Result<Result<(), DxfEntityPlacementOwnerIssue>, DxfError> {
        let record = self
            .common_owners
            .record_entry(raw_record_ordinal)
            .ok_or_else(invalid_internal_data)?;
        if record.state() != DxfCommonOwnerCandidateState::UniqueCandidate {
            return Ok(Err(DxfEntityPlacementOwnerIssue::BlockOwnerCardinality {
                state: record.state(),
            }));
        }
        let candidate_ordinal = record.candidate_range().start();
        let candidate = self
            .common_owners
            .entry(candidate_ordinal)
            .ok_or_else(invalid_internal_data)?;
        if candidate.state() != DxfHandleResolutionState::Unique {
            return Ok(Err(DxfEntityPlacementOwnerIssue::BlockOwnerResolution {
                state: candidate.state(),
            }));
        }
        let [declared] = self
            .common_owners
            .targets_for_candidate(candidate_ordinal)
            .ok_or_else(invalid_internal_data)?
        else {
            return Err(invalid_internal_data());
        };
        if declared.record().ordinal() != requested.record().ordinal() {
            return Ok(Err(DxfEntityPlacementOwnerIssue::BlockOwnerMismatch {
                declared: *declared,
                requested,
            }));
        }
        Ok(Ok(()))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_placement_owner_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityPlacementOwnerDirectory, DxfError> {
        DxfEntityPlacementOwnerDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_placement_owner_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityPlacementOwnerDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_placement_owner_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_placement_owner_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityPlacementOwnerDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_placement_owner_directory(cancellation)
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
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

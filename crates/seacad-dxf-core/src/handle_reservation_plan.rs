//! Source-bound reservation of handles for records that have not been inserted yet.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandle,
    DxfHandleAllocationOutcome, DxfHandleAllocationPolicyDirectory, DxfHandleAllocationPolicyState,
    DxfHandleAllocationProposal, DxfIoOperation, DxfRawDocumentView, DxfResourceProfile,
    DxfSourceId, DxfTransactionPlan, handle::encode_dxf_handle_upper_hex,
};

/// Consecutive handle allocation paired with its exact `$HANDSEED` transaction.
#[derive(Debug)]
pub struct DxfHandleReservationPlan {
    allocation: DxfHandleAllocationProposal,
    transaction: DxfTransactionPlan,
}

impl DxfHandleReservationPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.allocation.source_id()
    }

    #[must_use]
    pub const fn allocation(&self) -> DxfHandleAllocationProposal {
        self.allocation
    }

    #[must_use]
    pub const fn transaction(&self) -> &DxfTransactionPlan {
        &self.transaction
    }

    #[must_use]
    pub fn into_transaction(self) -> DxfTransactionPlan {
        self.transaction
    }
}

/// Typed result of preparing handles for future inserted records.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfHandleReservationPlanOutcome {
    PolicyUnavailable {
        state: DxfHandleAllocationPolicyState,
    },
    Exhausted {
        handseed: DxfHandle,
        requested_count: u64,
    },
    Planned(DxfHandleReservationPlan),
}

impl DxfRawDocumentView<'_> {
    /// Plans a successor `$HANDSEED` for a bounded future-record handle range.
    pub fn plan_handle_reservation(
        self,
        policy: &DxfHandleAllocationPolicyDirectory,
        handle_count: u64,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleReservationPlanOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        if policy.source_id() != self.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id(),
                observed: policy.source_id(),
            });
        }

        let allocation = match policy.propose_allocation(handle_count, profile)? {
            DxfHandleAllocationOutcome::Unavailable { state } => {
                return Ok(DxfHandleReservationPlanOutcome::PolicyUnavailable { state });
            }
            DxfHandleAllocationOutcome::Exhausted {
                handseed,
                requested_count,
            } => {
                return Ok(DxfHandleReservationPlanOutcome::Exhausted {
                    handseed,
                    requested_count,
                });
            }
            DxfHandleAllocationOutcome::Proposed(allocation) => allocation,
        };

        let mut builder = self.transaction_plan_builder(profile)?;
        if handle_count != 0 {
            ensure_not_cancelled(cancellation)?;
            let handseed_span = policy
                .handseed_occurrence()
                .and_then(|occurrence| occurrence.value_span())
                .ok_or_else(invalid_internal_data)?;
            let mut encoded = [0_u8; DxfHandle::MAX_HEX_DIGITS];
            let encoded = encode_dxf_handle_upper_hex(allocation.next_handseed(), &mut encoded);
            builder.replace_raw_span(handseed_span, encoded, cancellation)?;
        }
        let transaction = builder.finish(cancellation)?;
        Ok(DxfHandleReservationPlanOutcome::Planned(
            DxfHandleReservationPlan {
                allocation,
                transaction,
            },
        ))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn plan_handle_reservation(
        &self,
        policy: &DxfHandleAllocationPolicyDirectory,
        handle_count: u64,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleReservationPlanOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_handle_reservation(
            policy,
            handle_count,
            profile,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn plan_handle_reservation(
        &self,
        policy: &DxfHandleAllocationPolicyDirectory,
        handle_count: u64,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleReservationPlanOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_handle_reservation(
            policy,
            handle_count,
            profile,
            cancellation,
        )
    }
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

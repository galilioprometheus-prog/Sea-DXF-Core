//! Atomic composition of independently planned source-bound transactions.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfResource, DxfResourceProfile, DxfTransactionPlan,
};

impl DxfRawDocumentView<'_> {
    /// Rebuilds several plans as one source-ordered transaction.
    ///
    /// Every input remains bound to this exact raw document. Cross-plan span
    /// conflicts fail typed, and no partial composition escapes.
    pub fn compose_transaction_plans(
        self,
        plans: &[&DxfTransactionPlan],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTransactionPlan, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let plan_count = u64::try_from(plans.len()).map_err(|_| invalid_internal_data())?;
        let plan_limit = profile.limits().max_records();
        if plan_count > plan_limit {
            return Err(DxfError::resource_limit(
                DxfResource::Records,
                plan_limit,
                plan_count,
            ));
        }
        for plan in plans {
            ensure_not_cancelled(cancellation)?;
            plan.validate_source_precondition(self)?;
        }

        let mut builder = self.transaction_plan_builder(profile)?;
        for plan in plans {
            for patch in plan.patches().iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let replacement = plan
                    .replacement_bytes_for_patch_ordinal(patch.ordinal())
                    .ok_or_else(invalid_internal_data)?;
                builder.replace_raw_span(patch.source_span(), replacement, cancellation)?;
            }
        }
        builder.finish(cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn compose_transaction_plans(
        &self,
        plans: &[&DxfTransactionPlan],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTransactionPlan, DxfError> {
        DxfRawDocumentView::from(self).compose_transaction_plans(plans, profile, cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn compose_transaction_plans(
        &self,
        plans: &[&DxfTransactionPlan],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTransactionPlan, DxfError> {
        DxfRawDocumentView::from(self).compose_transaction_plans(plans, profile, cancellation)
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

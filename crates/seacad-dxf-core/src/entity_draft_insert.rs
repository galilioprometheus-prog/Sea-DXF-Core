//! Atomic insertion transactions and semantic postconditions for typed drafts.

use crate::{
    DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityDraftRecordPlan, DxfEntityEditPlan, DxfError, DxfRawDocumentView, DxfResourceProfile,
    DxfSourceId,
};

impl DxfRawDocumentView<'_> {
    /// Inserts one encoded typed draft together with its reserved handle seed.
    ///
    /// The returned edit plan retains family semantic postconditions and can
    /// use the standard create-new write, strict-reparse, and inverse pipeline.
    pub fn plan_entity_draft_insert(
        self,
        record: DxfEntityDraftRecordPlan,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityEditPlan, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), record.source_id())?;
        record.transaction().validate_source_precondition(self)?;
        let (applicability, bytes, expectation) = record.into_parts();
        let identity = applicability.identity();
        let handle = identity.handle();
        let owner =
            (applicability.version() >= DxfAcadVersion::Ac1012).then_some(identity.owner_handle());
        let placement = identity.placement();

        let mut insertion = self.transaction_plan_builder(profile)?;
        insertion.replace_raw_span(placement.insertion_span(), &bytes, cancellation)?;
        let insertion = insertion.finish(cancellation)?;
        let transaction = self.compose_transaction_plans(
            &[applicability.transaction(), &insertion],
            profile,
            cancellation,
        )?;
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityEditPlan::new_point_insert(
            transaction,
            handle,
            owner,
            placement.target(),
            expectation,
        ))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn plan_entity_draft_insert(
        &self,
        record: DxfEntityDraftRecordPlan,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityEditPlan, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_draft_insert(record, profile, cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn plan_entity_draft_insert(
        &self,
        record: DxfEntityDraftRecordPlan,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityEditPlan, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_draft_insert(record, profile, cancellation)
    }
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

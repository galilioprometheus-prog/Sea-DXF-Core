//! Same-dialect staging transactions for complete entity XDATA handle replacement sets.

use std::io;

use crate::{
    DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityXDataHandleReplacementSetDirectory, DxfEntityXDataHandleReplacementSetEntry,
    DxfEntityXDataHandleReplacementSetState, DxfError, DxfIoOperation, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfResourceProfile, DxfSourceId, DxfTransactionPlan,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleReplacementTransactionIssue {
    UnknownSet,
    SetUnavailable(DxfEntityXDataHandleReplacementSetState),
    FormatMismatch {
        source: DxfRawDocumentFormat,
        destination: DxfRawDocumentFormat,
    },
    DialectMismatch {
        source: DxfAcadVersionState,
        destination: DxfAcadVersionState,
    },
}

#[derive(Debug)]
pub struct DxfEntityXDataHandleReplacementTransactionPlan {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    set: DxfEntityXDataHandleReplacementSetEntry,
    transaction: DxfTransactionPlan,
}

impl DxfEntityXDataHandleReplacementTransactionPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn set(&self) -> DxfEntityXDataHandleReplacementSetEntry {
        self.set
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

#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleReplacementTransactionOutcome {
    Unavailable(DxfEntityXDataHandleReplacementTransactionIssue),
    Planned(Box<DxfEntityXDataHandleReplacementTransactionPlan>),
}

impl DxfRawDocumentView<'_> {
    pub fn plan_entity_xdata_handle_replacement_set(
        self,
        sets: &DxfEntityXDataHandleReplacementSetDirectory,
        set: DxfEntityXDataHandleReplacementSetEntry,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleReplacementTransactionOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), sets.source_id())?;
        if sets.entry(set.ordinal()) != Some(set) {
            return Ok(
                DxfEntityXDataHandleReplacementTransactionOutcome::Unavailable(
                    DxfEntityXDataHandleReplacementTransactionIssue::UnknownSet,
                ),
            );
        }
        if !matches!(
            set.state(),
            DxfEntityXDataHandleReplacementSetState::Ready { .. }
        ) {
            return Ok(
                DxfEntityXDataHandleReplacementTransactionOutcome::Unavailable(
                    DxfEntityXDataHandleReplacementTransactionIssue::SetUnavailable(set.state()),
                ),
            );
        }
        let replacements = sets.replacement_directory();
        let destination_format = replacements.destination_format();
        if self.format() != destination_format {
            return Ok(
                DxfEntityXDataHandleReplacementTransactionOutcome::Unavailable(
                    DxfEntityXDataHandleReplacementTransactionIssue::FormatMismatch {
                        source: self.format(),
                        destination: destination_format,
                    },
                ),
            );
        }
        let source_dialect = self.acad_version_report().state();
        let destination_dialect = replacements.destination_version_state();
        if source_dialect != destination_dialect {
            return Ok(
                DxfEntityXDataHandleReplacementTransactionOutcome::Unavailable(
                    DxfEntityXDataHandleReplacementTransactionIssue::DialectMismatch {
                        source: source_dialect,
                        destination: destination_dialect,
                    },
                ),
            );
        }
        let members = sets
            .replacements_for_entry(set)
            .ok_or_else(invalid_internal_data)?;
        let mut builder = self.transaction_plan_builder(profile)?;
        for member in members.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let patch = sets
                .patch_for_replacement(set, member)
                .ok_or_else(invalid_internal_data)?;
            let bytes = replacements
                .replacement_bytes_for_patch(patch)
                .ok_or_else(invalid_internal_data)?;
            builder.replace_raw_span(patch.source_group().full_span(), bytes, cancellation)?;
        }
        let transaction = builder.finish(cancellation)?;
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityXDataHandleReplacementTransactionOutcome::Planned(
            Box::new(DxfEntityXDataHandleReplacementTransactionPlan {
                source_id: self.source_id(),
                destination_id: sets.destination_id(),
                set,
                transaction,
            }),
        ))
    }
}

macro_rules! document_handle_replacement_transaction {
    ($document:ty) => {
        impl $document {
            pub fn plan_entity_xdata_handle_replacement_set(
                &self,
                sets: &DxfEntityXDataHandleReplacementSetDirectory,
                set: DxfEntityXDataHandleReplacementSetEntry,
                profile: DxfResourceProfile,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataHandleReplacementTransactionOutcome, DxfError> {
                DxfRawDocumentView::from(self).plan_entity_xdata_handle_replacement_set(
                    sets,
                    set,
                    profile,
                    cancellation,
                )
            }
        }
    };
}

document_handle_replacement_transaction!(DxfAsciiRawDocument<'_>);
document_handle_replacement_transaction!(DxfBinaryRawDocument<'_>);

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

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

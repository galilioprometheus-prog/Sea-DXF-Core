//! POINT clone insertion, verification, and create-new writing with XDATA provenance.

use std::{io, path::Path};

use crate::{
    DxfAcadVersion, DxfCancellationToken, DxfEntityEditPlan, DxfEntityXDataDraftInsertPlan,
    DxfEntityXDataDraftVerificationIssue, DxfEntityXDataDraftVerificationJournal,
    DxfEntityXDataDraftVerificationOutcome, DxfEntityXDataDraftVerificationReceipt,
    DxfEntityXDataDraftWriteJournal, DxfEntityXDataDraftWriteOutcome, DxfError, DxfHandle,
    DxfIoOperation, DxfPointCloneDialectAdaptations, DxfPointCloneXDataDraftPlan,
    DxfRawDocumentView, DxfReadObserver, DxfResourceProfile, DxfSourceId, DxfTransactionPlan,
};

/// Compact immutable source semantic provenance retained through POINT clone
/// insertion, verification, and writing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPointCloneSourceEvidence {
    source_id: DxfSourceId,
    source_key: crate::DxfEntityKey,
    source_version: DxfAcadVersion,
    source_placement: crate::DxfEntityPlacementTarget,
    source_owner: Option<DxfHandle>,
    adaptations: DxfPointCloneDialectAdaptations,
}

impl DxfPointCloneSourceEvidence {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_key(self) -> crate::DxfEntityKey {
        self.source_key
    }

    #[must_use]
    pub const fn source_version(self) -> DxfAcadVersion {
        self.source_version
    }

    #[must_use]
    pub const fn source_placement(self) -> crate::DxfEntityPlacementTarget {
        self.source_placement
    }

    #[must_use]
    pub const fn source_owner(self) -> Option<DxfHandle> {
        self.source_owner
    }

    #[must_use]
    pub const fn dialect_adaptations(self) -> DxfPointCloneDialectAdaptations {
        self.adaptations
    }
}

/// Atomic destination insertion plan retaining source POINT provenance.
pub struct DxfPointCloneXDataInsertPlan {
    source: DxfPointCloneSourceEvidence,
    insert: DxfEntityXDataDraftInsertPlan,
}

impl DxfPointCloneXDataInsertPlan {
    #[must_use]
    pub const fn source_evidence(&self) -> DxfPointCloneSourceEvidence {
        self.source
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.insert.destination_id()
    }

    #[must_use]
    pub const fn destination_handle(&self) -> DxfHandle {
        self.insert.destination_handle()
    }

    #[must_use]
    pub fn expected_xdata_bytes(&self) -> &[u8] {
        self.insert.expected_xdata_bytes()
    }

    #[must_use]
    pub const fn edit_plan(&self) -> &DxfEntityEditPlan {
        self.insert.edit_plan()
    }

    #[must_use]
    pub const fn xdata_insert_plan(&self) -> &DxfEntityXDataDraftInsertPlan {
        &self.insert
    }

    #[must_use]
    pub fn into_xdata_insert_plan(self) -> DxfEntityXDataDraftInsertPlan {
        self.insert
    }

    pub fn verify_post_image(
        &self,
        destination: DxfRawDocumentView<'_>,
        post_image: DxfRawDocumentView<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPointCloneXDataVerificationOutcome, DxfError> {
        let outcome =
            self.insert
                .verify_post_image(destination, post_image, profile, cancellation)?;
        Ok(match outcome {
            DxfEntityXDataDraftVerificationOutcome::Unavailable(issue) => {
                DxfPointCloneXDataVerificationOutcome::Unavailable(issue)
            }
            DxfEntityXDataDraftVerificationOutcome::Verified(journal) => {
                DxfPointCloneXDataVerificationOutcome::Verified(Box::new(
                    DxfPointCloneXDataVerificationJournal {
                        source: self.source,
                        xdata: journal,
                    },
                ))
            }
        })
    }

    pub fn write_reparse_verify_and_journal_to_new_file(
        &self,
        destination_document: DxfRawDocumentView<'_>,
        destination: impl AsRef<Path>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfPointCloneXDataWriteOutcome, DxfError> {
        let outcome = self.insert.write_reparse_verify_and_journal_to_new_file(
            destination_document,
            destination,
            profile,
            cancellation,
            observer,
        )?;
        Ok(match outcome {
            DxfEntityXDataDraftWriteOutcome::Unavailable(issue) => {
                DxfPointCloneXDataWriteOutcome::Unavailable(issue)
            }
            DxfEntityXDataDraftWriteOutcome::Written(journal) => {
                DxfPointCloneXDataWriteOutcome::Written(Box::new(DxfPointCloneXDataWriteJournal {
                    source: self.source,
                    xdata: journal,
                }))
            }
        })
    }
}

impl std::fmt::Debug for DxfPointCloneXDataInsertPlan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DxfPointCloneXDataInsertPlan")
            .field("source", &self.source)
            .field("insert", &self.insert)
            .finish()
    }
}

/// Strict verification journal retaining source POINT provenance.
#[derive(Debug)]
pub struct DxfPointCloneXDataVerificationJournal {
    source: DxfPointCloneSourceEvidence,
    xdata: Box<DxfEntityXDataDraftVerificationJournal>,
}

impl DxfPointCloneXDataVerificationJournal {
    #[must_use]
    pub const fn source_evidence(&self) -> DxfPointCloneSourceEvidence {
        self.source
    }

    #[must_use]
    pub const fn receipt(&self) -> DxfEntityXDataDraftVerificationReceipt {
        self.xdata.receipt()
    }

    #[must_use]
    pub const fn inverse_plan(&self) -> &DxfTransactionPlan {
        self.xdata.inverse_plan()
    }

    #[must_use]
    pub const fn xdata_journal(&self) -> &DxfEntityXDataDraftVerificationJournal {
        &self.xdata
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DxfPointCloneXDataVerificationOutcome {
    Unavailable(DxfEntityXDataDraftVerificationIssue),
    Verified(Box<DxfPointCloneXDataVerificationJournal>),
}

/// Create-new write journal retaining source POINT provenance.
#[derive(Debug)]
pub struct DxfPointCloneXDataWriteJournal {
    source: DxfPointCloneSourceEvidence,
    xdata: Box<DxfEntityXDataDraftWriteJournal>,
}

impl DxfPointCloneXDataWriteJournal {
    #[must_use]
    pub const fn source_evidence(&self) -> DxfPointCloneSourceEvidence {
        self.source
    }

    #[must_use]
    pub const fn xdata_journal(&self) -> &DxfEntityXDataDraftWriteJournal {
        &self.xdata
    }

    #[must_use]
    pub const fn inverse_plan(&self) -> &DxfTransactionPlan {
        self.xdata.inverse_plan()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DxfPointCloneXDataWriteOutcome {
    Unavailable(DxfEntityXDataDraftVerificationIssue),
    Written(Box<DxfPointCloneXDataWriteJournal>),
}

impl DxfRawDocumentView<'_> {
    pub fn plan_point_clone_xdata_insert(
        self,
        draft: DxfPointCloneXDataDraftPlan,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPointCloneXDataInsertPlan, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), draft.destination_id())?;
        if draft.source_entity().key() != draft.source_key() {
            return Err(invalid_internal_data());
        }
        let source = DxfPointCloneSourceEvidence {
            source_id: draft.source_id(),
            source_key: draft.source_key(),
            source_version: draft.source_version(),
            source_placement: draft.source_placement(),
            source_owner: draft.source_owner(),
            adaptations: draft.dialect_adaptations(),
        };
        let insert = self.plan_entity_xdata_draft_insert(
            draft.into_xdata_draft_record(),
            profile,
            cancellation,
        )?;
        ensure_source(source.source_id(), insert.source_id())?;
        ensure_source(self.source_id(), insert.destination_id())?;
        if insert.source_entity().key() != source.source_key() {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;
        Ok(DxfPointCloneXDataInsertPlan { source, insert })
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

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

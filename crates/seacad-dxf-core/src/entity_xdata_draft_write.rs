//! Create-new writing, strict reparse, and XDATA-aware verification for entity drafts.

use std::path::Path;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityXDataDraftInsertPlan,
    DxfEntityXDataDraftVerificationIssue, DxfEntityXDataDraftVerificationOutcome,
    DxfEntityXDataDraftVerificationReceipt, DxfError, DxfFileSource, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadMode, DxfReadObserver, DxfReadOptions, DxfResourceProfile,
    DxfTransactionPlan, DxfTransactionWriteReceipt, NoopDxfReadObserver,
};

#[derive(Debug)]
pub struct DxfEntityXDataDraftWriteJournal {
    write_receipt: DxfTransactionWriteReceipt,
    verification_receipt: DxfEntityXDataDraftVerificationReceipt,
    inverse: DxfTransactionPlan,
}

impl DxfEntityXDataDraftWriteJournal {
    #[must_use]
    pub const fn write_receipt(&self) -> DxfTransactionWriteReceipt {
        self.write_receipt
    }

    #[must_use]
    pub const fn verification_receipt(&self) -> DxfEntityXDataDraftVerificationReceipt {
        self.verification_receipt
    }

    #[must_use]
    pub const fn inverse_plan(&self) -> &DxfTransactionPlan {
        &self.inverse
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        DxfTransactionWriteReceipt,
        DxfEntityXDataDraftVerificationReceipt,
        DxfTransactionPlan,
    ) {
        (self.write_receipt, self.verification_receipt, self.inverse)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityXDataDraftWriteOutcome {
    Unavailable(DxfEntityXDataDraftVerificationIssue),
    Written(Box<DxfEntityXDataDraftWriteJournal>),
}

impl DxfEntityXDataDraftInsertPlan {
    pub fn write_reparse_verify_and_journal_to_new_file(
        &self,
        destination_document: DxfRawDocumentView<'_>,
        destination: impl AsRef<Path>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfEntityXDataDraftWriteOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.destination_id(), destination_document.source_id())?;
        let destination = destination.as_ref();
        let write_receipt = self.edit_plan().transaction().write_to_new_file(
            destination_document,
            destination,
            cancellation,
            observer,
        )?;
        let outcome = self.reparse_written_destination(
            destination_document,
            destination,
            write_receipt,
            profile,
            cancellation,
        );
        match outcome {
            Ok(DxfEntityXDataDraftWriteOutcome::Written(journal)) => {
                Ok(DxfEntityXDataDraftWriteOutcome::Written(journal))
            }
            Ok(DxfEntityXDataDraftWriteOutcome::Unavailable(issue)) => {
                crate::transaction_write::remove_created_destination(destination)?;
                Ok(DxfEntityXDataDraftWriteOutcome::Unavailable(issue))
            }
            Err(primary) => match crate::transaction_write::remove_created_destination(destination)
            {
                Ok(()) => Err(primary),
                Err(cleanup) => Err(cleanup),
            },
        }
    }

    fn reparse_written_destination(
        &self,
        destination_document: DxfRawDocumentView<'_>,
        destination: &Path,
        write_receipt: DxfTransactionWriteReceipt,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataDraftWriteOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let output_source = DxfFileSource::open(destination, profile)?;
        let options = DxfReadOptions::new(DxfReadMode::Strict, profile);
        let mut observer = NoopDxfReadObserver;
        match self.edit_plan().transaction().format() {
            DxfRawDocumentFormat::Ascii => {
                let post_image = DxfAsciiRawDocument::open(
                    &output_source,
                    options,
                    cancellation,
                    &mut observer,
                )?;
                self.finish_written_verification(
                    destination_document,
                    DxfRawDocumentView::from(&post_image),
                    write_receipt,
                    profile,
                    cancellation,
                )
            }
            DxfRawDocumentFormat::Binary => {
                let post_image = DxfBinaryRawDocument::open(
                    &output_source,
                    options,
                    cancellation,
                    &mut observer,
                )?;
                self.finish_written_verification(
                    destination_document,
                    DxfRawDocumentView::from(&post_image),
                    write_receipt,
                    profile,
                    cancellation,
                )
            }
        }
    }

    fn finish_written_verification(
        &self,
        destination_document: DxfRawDocumentView<'_>,
        post_image: DxfRawDocumentView<'_>,
        write_receipt: DxfTransactionWriteReceipt,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataDraftWriteOutcome, DxfError> {
        let journal = match self.verify_post_image(
            destination_document,
            post_image,
            profile,
            cancellation,
        )? {
            DxfEntityXDataDraftVerificationOutcome::Unavailable(issue) => {
                return Ok(DxfEntityXDataDraftWriteOutcome::Unavailable(issue));
            }
            DxfEntityXDataDraftVerificationOutcome::Verified(journal) => journal,
        };
        let (verification_receipt, family) = journal.into_parts();
        let (_, inverse) = family.into_parts();
        validate_written_identities(write_receipt, verification_receipt)?;
        Ok(DxfEntityXDataDraftWriteOutcome::Written(Box::new(
            DxfEntityXDataDraftWriteJournal {
                write_receipt,
                verification_receipt,
                inverse,
            },
        )))
    }
}

fn validate_written_identities(
    write: DxfTransactionWriteReceipt,
    verification: DxfEntityXDataDraftVerificationReceipt,
) -> Result<(), DxfError> {
    ensure_source(write.source_id(), verification.destination_id())?;
    if write.output_id() == verification.post_image_id() {
        Ok(())
    } else {
        Err(DxfError::TransactionOutputIdentityMismatch {
            expected: write.output_id(),
            observed: verification.post_image_id(),
        })
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn ensure_source(
    expected: crate::DxfSourceId,
    observed: crate::DxfSourceId,
) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
}

//! Strict post-image verification and create-new writes for staged entity
//! XDATA handle replacements.

use std::{io, path::Path};

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityXDataHandleReplacementSetDirectory, DxfEntityXDataHandleReplacementTransactionPlan,
    DxfError, DxfFileSource, DxfHandle, DxfHandleParseIssue, DxfIoOperation, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfRawHandleLookup, DxfReadMode, DxfReadObserver, DxfReadOptions,
    DxfResourceProfile, DxfSourceId, DxfTransactionPlan, DxfTransactionWriteReceipt,
    NoopDxfReadObserver,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleReplacementVerificationIssue {
    EvidenceMismatch,
    MissingOccurrence {
        occurrence: u64,
    },
    NotHandleGroup {
        occurrence: u64,
        group_code: i16,
    },
    InvalidHandle {
        occurrence: u64,
        issue: DxfHandleParseIssue,
    },
    HandleMismatch {
        occurrence: u64,
        expected: DxfHandle,
        observed: DxfHandle,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataHandleReplacementVerificationReceipt {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    post_image_id: DxfSourceId,
    replacement_count: u32,
}

impl DxfEntityXDataHandleReplacementVerificationReceipt {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn post_image_id(self) -> DxfSourceId {
        self.post_image_id
    }

    #[must_use]
    pub const fn replacement_count(self) -> u64 {
        self.replacement_count as u64
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataHandleReplacementVerificationJournal {
    receipt: DxfEntityXDataHandleReplacementVerificationReceipt,
    inverse: DxfTransactionPlan,
}

impl DxfEntityXDataHandleReplacementVerificationJournal {
    #[must_use]
    pub const fn receipt(&self) -> DxfEntityXDataHandleReplacementVerificationReceipt {
        self.receipt
    }

    #[must_use]
    pub const fn inverse_plan(&self) -> &DxfTransactionPlan {
        &self.inverse
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        DxfEntityXDataHandleReplacementVerificationReceipt,
        DxfTransactionPlan,
    ) {
        (self.receipt, self.inverse)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleReplacementVerificationOutcome {
    Unavailable(DxfEntityXDataHandleReplacementVerificationIssue),
    Verified(DxfEntityXDataHandleReplacementVerificationJournal),
}

/// Create-new write and replacement-verification receipts with an inverse.
#[derive(Debug)]
pub struct DxfEntityXDataHandleReplacementWriteJournal {
    write_receipt: DxfTransactionWriteReceipt,
    destination_id: DxfSourceId,
    replacement_count: u32,
    inverse: DxfTransactionPlan,
}

impl DxfEntityXDataHandleReplacementWriteJournal {
    #[must_use]
    pub const fn write_receipt(&self) -> DxfTransactionWriteReceipt {
        self.write_receipt
    }

    #[must_use]
    pub const fn verification_receipt(&self) -> DxfEntityXDataHandleReplacementVerificationReceipt {
        DxfEntityXDataHandleReplacementVerificationReceipt {
            source_id: self.write_receipt.source_id(),
            destination_id: self.destination_id,
            post_image_id: self.write_receipt.output_id(),
            replacement_count: self.replacement_count,
        }
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
        DxfEntityXDataHandleReplacementVerificationReceipt,
        DxfTransactionPlan,
    ) {
        (
            self.write_receipt,
            self.verification_receipt(),
            self.inverse,
        )
    }
}

/// Result of create-new writing followed by strict replacement verification.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleReplacementWriteOutcome {
    Unavailable(DxfEntityXDataHandleReplacementVerificationIssue),
    Written(DxfEntityXDataHandleReplacementWriteJournal),
}

impl DxfEntityXDataHandleReplacementTransactionPlan {
    pub fn verify_post_image(
        &self,
        sets: &DxfEntityXDataHandleReplacementSetDirectory,
        source: DxfRawDocumentView<'_>,
        post_image: DxfRawDocumentView<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleReplacementVerificationOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        if !self.matches_set_evidence(sets) {
            return Ok(
                DxfEntityXDataHandleReplacementVerificationOutcome::Unavailable(
                    DxfEntityXDataHandleReplacementVerificationIssue::EvidenceMismatch,
                ),
            );
        }
        let inverse = self.transaction().materialize_inverse_plan(
            source,
            post_image,
            profile,
            cancellation,
        )?;
        let members = sets
            .replacements_for_entry(self.set())
            .ok_or_else(invalid_internal_data)?;
        for member in members.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let patch = sets
                .patch_for_replacement(self.set(), member)
                .ok_or_else(invalid_internal_data)?;
            let occurrence = patch.source_group().occurrence();
            let observed =
                match post_image.raw_handle_at(occurrence, cancellation)? {
                    DxfRawHandleLookup::MissingOccurrence => {
                        return Ok(DxfEntityXDataHandleReplacementVerificationOutcome::Unavailable(
                        DxfEntityXDataHandleReplacementVerificationIssue::MissingOccurrence {
                            occurrence,
                        },
                    ));
                    }
                    DxfRawHandleLookup::NotHandleGroup(group) => {
                        return Ok(
                            DxfEntityXDataHandleReplacementVerificationOutcome::Unavailable(
                                DxfEntityXDataHandleReplacementVerificationIssue::NotHandleGroup {
                                    occurrence,
                                    group_code: group.group_code().value(),
                                },
                            ),
                        );
                    }
                    DxfRawHandleLookup::Handle(value) => match value.parse_result() {
                        Ok(handle) => handle,
                        Err(issue) => {
                            return Ok(
                            DxfEntityXDataHandleReplacementVerificationOutcome::Unavailable(
                                DxfEntityXDataHandleReplacementVerificationIssue::InvalidHandle {
                                    occurrence,
                                    issue,
                                },
                            ),
                        );
                        }
                    },
                };
            if observed != patch.target() {
                return Ok(
                    DxfEntityXDataHandleReplacementVerificationOutcome::Unavailable(
                        DxfEntityXDataHandleReplacementVerificationIssue::HandleMismatch {
                            occurrence,
                            expected: patch.target(),
                            observed,
                        },
                    ),
                );
            }
        }
        let replacement_count =
            u32::try_from(members.len()).map_err(|_| invalid_internal_data())?;
        Ok(
            DxfEntityXDataHandleReplacementVerificationOutcome::Verified(
                DxfEntityXDataHandleReplacementVerificationJournal {
                    receipt: DxfEntityXDataHandleReplacementVerificationReceipt {
                        source_id: self.source_id(),
                        destination_id: self.destination_id(),
                        post_image_id: post_image.source_id(),
                        replacement_count,
                    },
                    inverse,
                },
            ),
        )
    }

    /// Writes to a new path, strictly reparses, verifies replacements, and journals.
    ///
    /// Any failure or unavailable verification after file creation removes the
    /// destination. An existing destination is never modified.
    pub fn write_reparse_verify_and_journal_to_new_file(
        &self,
        sets: &DxfEntityXDataHandleReplacementSetDirectory,
        source: DxfRawDocumentView<'_>,
        destination: impl AsRef<Path>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfEntityXDataHandleReplacementWriteOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        if !self.matches_set_evidence(sets) {
            return Ok(DxfEntityXDataHandleReplacementWriteOutcome::Unavailable(
                DxfEntityXDataHandleReplacementVerificationIssue::EvidenceMismatch,
            ));
        }
        let destination = destination.as_ref();
        let write_receipt =
            self.transaction()
                .write_to_new_file(source, destination, cancellation, observer)?;
        let outcome = self.reparse_and_verify_written_destination(
            sets,
            source,
            destination,
            write_receipt,
            profile,
            cancellation,
        );
        match outcome {
            Ok(DxfEntityXDataHandleReplacementWriteOutcome::Written(journal)) => Ok(
                DxfEntityXDataHandleReplacementWriteOutcome::Written(journal),
            ),
            Ok(DxfEntityXDataHandleReplacementWriteOutcome::Unavailable(issue)) => {
                crate::transaction_write::remove_created_destination(destination)?;
                Ok(DxfEntityXDataHandleReplacementWriteOutcome::Unavailable(
                    issue,
                ))
            }
            Err(primary) => {
                match crate::transaction_write::remove_created_destination(destination) {
                    Ok(()) => Err(primary),
                    Err(cleanup) => Err(cleanup),
                }
            }
        }
    }

    fn matches_set_evidence(&self, sets: &DxfEntityXDataHandleReplacementSetDirectory) -> bool {
        self.source_id() == sets.source_id()
            && self.destination_id() == sets.destination_id()
            && sets.entry(self.set().ordinal()) == Some(self.set())
    }

    fn reparse_and_verify_written_destination(
        &self,
        sets: &DxfEntityXDataHandleReplacementSetDirectory,
        source: DxfRawDocumentView<'_>,
        destination: &Path,
        write_receipt: DxfTransactionWriteReceipt,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleReplacementWriteOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let output_source = DxfFileSource::open(destination, profile)?;
        let options = DxfReadOptions::new(DxfReadMode::Strict, profile);
        let mut observer = NoopDxfReadObserver;
        match self.transaction().format() {
            DxfRawDocumentFormat::Ascii => {
                let post_image = DxfAsciiRawDocument::open(
                    &output_source,
                    options,
                    cancellation,
                    &mut observer,
                )?;
                self.finish_written_verification(
                    sets,
                    source,
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
                    sets,
                    source,
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
        sets: &DxfEntityXDataHandleReplacementSetDirectory,
        source: DxfRawDocumentView<'_>,
        post_image: DxfRawDocumentView<'_>,
        write_receipt: DxfTransactionWriteReceipt,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleReplacementWriteOutcome, DxfError> {
        let journal =
            match self.verify_post_image(sets, source, post_image, profile, cancellation)? {
                DxfEntityXDataHandleReplacementVerificationOutcome::Unavailable(issue) => {
                    return Ok(DxfEntityXDataHandleReplacementWriteOutcome::Unavailable(
                        issue,
                    ));
                }
                DxfEntityXDataHandleReplacementVerificationOutcome::Verified(journal) => journal,
            };
        let (verification_receipt, inverse) = journal.into_parts();
        validate_written_receipts(write_receipt, verification_receipt)?;
        Ok(DxfEntityXDataHandleReplacementWriteOutcome::Written(
            DxfEntityXDataHandleReplacementWriteJournal {
                write_receipt,
                destination_id: verification_receipt.destination_id,
                replacement_count: verification_receipt.replacement_count,
                inverse,
            },
        ))
    }
}

fn validate_written_receipts(
    write: DxfTransactionWriteReceipt,
    verification: DxfEntityXDataHandleReplacementVerificationReceipt,
) -> Result<(), DxfError> {
    if write.source_id() != verification.source_id() {
        return Err(DxfError::SourceIdentityMismatch {
            expected: write.source_id(),
            observed: verification.source_id(),
        });
    }
    if write.output_id() != verification.post_image_id() {
        return Err(DxfError::TransactionOutputIdentityMismatch {
            expected: write.output_id(),
            observed: verification.post_image_id(),
        });
    }
    if write.patch_count() != verification.replacement_count() {
        return Err(invalid_internal_data());
    }
    Ok(())
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

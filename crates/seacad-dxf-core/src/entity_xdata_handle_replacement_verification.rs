//! Strict post-image verification for staged entity XDATA handle replacements.

use std::io;

use crate::{
    DxfCancellationToken, DxfEntityXDataHandleReplacementSetDirectory,
    DxfEntityXDataHandleReplacementTransactionPlan, DxfError, DxfHandle, DxfHandleParseIssue,
    DxfIoOperation, DxfRawDocumentView, DxfRawHandleLookup, DxfResourceProfile, DxfSourceId,
    DxfTransactionPlan,
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
        if self.source_id() != sets.source_id()
            || self.destination_id() != sets.destination_id()
            || sets.entry(self.set().ordinal()) != Some(self.set())
        {
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

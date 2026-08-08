//! Strict post-image verification for inserted canonical entity XDATA payloads.

use std::io;

use crate::{
    ByteSpan, DxfCancellationToken, DxfEntityEditVerificationIssue,
    DxfEntityEditVerificationJournal, DxfEntityEditVerificationOutcome,
    DxfEntityXDataDraftInsertPlan, DxfError, DxfHandle, DxfHandleIdentityLookup, DxfIoOperation,
    DxfRawDocumentView, DxfResource, DxfResourceProfile, DxfSourceId, DxfTransactionPlan,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataDraftVerificationIssue {
    Family(DxfEntityEditVerificationIssue),
    MissingInsertedEntity {
        handle: DxfHandle,
    },
    AmbiguousInsertedEntity {
        handle: DxfHandle,
        candidate_count: u32,
    },
    InsertedEntityNotIndexed {
        handle: DxfHandle,
        raw_record_ordinal: u64,
    },
    PayloadShapeMismatch {
        expected_byte_count: u64,
        observed_byte_count: u64,
        application_count: u32,
        orphan_value_count: u32,
    },
    PayloadBytesMismatch {
        byte_count: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataDraftVerificationReceipt {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    post_image_id: DxfSourceId,
    destination_handle: DxfHandle,
    application_count: u32,
    xdata_byte_count: u64,
}

impl DxfEntityXDataDraftVerificationReceipt {
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
    pub const fn destination_handle(self) -> DxfHandle {
        self.destination_handle
    }

    #[must_use]
    pub const fn application_count(self) -> u64 {
        self.application_count as u64
    }

    #[must_use]
    pub const fn xdata_byte_count(self) -> u64 {
        self.xdata_byte_count
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataDraftVerificationJournal {
    receipt: DxfEntityXDataDraftVerificationReceipt,
    family: DxfEntityEditVerificationJournal,
}

impl DxfEntityXDataDraftVerificationJournal {
    #[must_use]
    pub const fn receipt(&self) -> DxfEntityXDataDraftVerificationReceipt {
        self.receipt
    }

    #[must_use]
    pub const fn family_journal(&self) -> &DxfEntityEditVerificationJournal {
        &self.family
    }

    #[must_use]
    pub const fn inverse_plan(&self) -> &DxfTransactionPlan {
        self.family.inverse_plan()
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        DxfEntityXDataDraftVerificationReceipt,
        DxfEntityEditVerificationJournal,
    ) {
        (self.receipt, self.family)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityXDataDraftVerificationOutcome {
    Unavailable(DxfEntityXDataDraftVerificationIssue),
    Verified(Box<DxfEntityXDataDraftVerificationJournal>),
}

impl DxfEntityXDataDraftInsertPlan {
    pub fn verify_post_image(
        &self,
        destination: DxfRawDocumentView<'_>,
        post_image: DxfRawDocumentView<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataDraftVerificationOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.destination_id(), destination.source_id())?;
        let family = match self.edit_plan().verify_post_image(
            destination,
            post_image,
            profile,
            cancellation,
        )? {
            DxfEntityEditVerificationOutcome::Unavailable(issue) => {
                return Ok(DxfEntityXDataDraftVerificationOutcome::Unavailable(
                    DxfEntityXDataDraftVerificationIssue::Family(issue),
                ));
            }
            DxfEntityEditVerificationOutcome::Verified(journal) => journal,
        };
        let application_count = match verify_payload(self, post_image, profile, cancellation)? {
            Ok(application_count) => application_count,
            Err(issue) => {
                return Ok(DxfEntityXDataDraftVerificationOutcome::Unavailable(issue));
            }
        };
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityXDataDraftVerificationOutcome::Verified(Box::new(
            DxfEntityXDataDraftVerificationJournal {
                receipt: DxfEntityXDataDraftVerificationReceipt {
                    source_id: self.source_id(),
                    destination_id: self.destination_id(),
                    post_image_id: post_image.source_id(),
                    destination_handle: self.destination_handle(),
                    application_count,
                    xdata_byte_count: compact_len(self.expected_xdata_bytes().len())?,
                },
                family,
            },
        )))
    }
}

fn verify_payload(
    plan: &DxfEntityXDataDraftInsertPlan,
    post_image: DxfRawDocumentView<'_>,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<u32, DxfEntityXDataDraftVerificationIssue>, DxfError> {
    let identities = post_image.handle_identity_directory(cancellation)?;
    let identity = match identities.lookup(plan.destination_handle()) {
        DxfHandleIdentityLookup::Missing => {
            return Ok(Err(
                DxfEntityXDataDraftVerificationIssue::MissingInsertedEntity {
                    handle: plan.destination_handle(),
                },
            ));
        }
        DxfHandleIdentityLookup::Ambiguous(candidates) => {
            return Ok(Err(
                DxfEntityXDataDraftVerificationIssue::AmbiguousInsertedEntity {
                    handle: plan.destination_handle(),
                    candidate_count: compact_count(candidates.len())?,
                },
            ));
        }
        DxfHandleIdentityLookup::Unique(identity) => identity,
    };
    let xdata = post_image.entity_xdata_directory(cancellation)?;
    let Some(entity) = xdata
        .entity_directory()
        .entity_for_raw_ordinal(identity.record().ordinal())
    else {
        return Ok(Err(
            DxfEntityXDataDraftVerificationIssue::InsertedEntityNotIndexed {
                handle: plan.destination_handle(),
                raw_record_ordinal: identity.record().ordinal(),
            },
        ));
    };
    let applications = xdata.applications_for_entity(entity)?;
    let occurrences = xdata.occurrences_for_entity(entity)?;
    let orphan_value_count = compact_count(
        occurrences
            .iter()
            .filter(|occurrence| {
                matches!(
                    occurrence.kind(),
                    crate::DxfEntityXDataOccurrenceKind::Orphan
                )
            })
            .count(),
    )?;
    let observed = read_occurrence_bytes(post_image, occurrences, profile)?;
    let expected = plan.expected_xdata_bytes();
    if orphan_value_count != 0 || observed.len() != expected.len() {
        return Ok(Err(
            DxfEntityXDataDraftVerificationIssue::PayloadShapeMismatch {
                expected_byte_count: compact_len(expected.len())?,
                observed_byte_count: compact_len(observed.len())?,
                application_count: compact_count(applications.len())?,
                orphan_value_count,
            },
        ));
    }
    if observed != expected {
        return Ok(Err(
            DxfEntityXDataDraftVerificationIssue::PayloadBytesMismatch {
                byte_count: compact_len(expected.len())?,
            },
        ));
    }
    Ok(Ok(compact_count(applications.len())?))
}

fn read_occurrence_bytes(
    post_image: DxfRawDocumentView<'_>,
    occurrences: &[crate::DxfEntityXDataOccurrence],
    profile: DxfResourceProfile,
) -> Result<Vec<u8>, DxfError> {
    let Some(first) = occurrences.first() else {
        return Ok(Vec::new());
    };
    let last = occurrences.last().ok_or_else(invalid_internal_data)?;
    let span = ByteSpan::new(
        first.group().full_span().start(),
        last.group().full_span().end(),
    )
    .ok_or_else(invalid_internal_data)?;
    let limit = profile.limits().max_value_bytes();
    if span.len() > limit {
        return Err(DxfError::resource_limit(
            DxfResource::ValueBytes,
            limit,
            span.len(),
        ));
    }
    let len = usize::try_from(span.len()).map_err(|_| invalid_internal_data())?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(len).map_err(|_| out_of_memory())?;
    bytes.resize(len, 0);
    post_image.read_span(span, &mut bytes)?;
    Ok(bytes)
}

fn compact_count(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

fn compact_len(value: usize) -> Result<u64, DxfError> {
    u64::try_from(value).map_err(|_| invalid_internal_data())
}

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

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

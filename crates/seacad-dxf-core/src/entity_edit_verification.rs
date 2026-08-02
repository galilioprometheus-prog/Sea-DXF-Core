//! Semantic postcondition verification for unified entity edit plans.

use std::{fmt, io, path::Path};

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfEntityEditValue,
    DxfEntityField, DxfEntityFieldCardState, DxfEntityFieldDefault, DxfEntityFieldSemantics,
    DxfEntityFieldValue, DxfError, DxfFileSource, DxfHandle, DxfIoOperation, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadMode, DxfReadObserver, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, DxfSourceId, DxfTransactionPlan, DxfTransactionWriteReceipt,
    NoopDxfReadObserver,
};

pub(crate) enum DxfEntityExpectedValue {
    Double(DxfDouble),
    ExactText(Box<[u8]>),
    Handle(DxfHandle),
    Int16(i16),
    Int32(i32),
}

pub(crate) enum DxfEntityExpectedField {
    Explicit(DxfEntityExpectedValue),
    Implicit,
}

impl DxfEntityExpectedField {
    pub(crate) fn explicit(value: DxfEntityEditValue<'_>) -> Result<Self, DxfError> {
        let value = match value {
            DxfEntityEditValue::Double(value) => DxfEntityExpectedValue::Double(value),
            DxfEntityEditValue::Handle(value) => DxfEntityExpectedValue::Handle(value),
            DxfEntityEditValue::Int16(value) => DxfEntityExpectedValue::Int16(value),
            DxfEntityEditValue::Int32(value) => DxfEntityExpectedValue::Int32(value),
            DxfEntityEditValue::ExactRawText(value) => {
                let mut owned = Vec::new();
                owned
                    .try_reserve_exact(value.len())
                    .map_err(|_| out_of_memory())?;
                owned.extend_from_slice(value);
                DxfEntityExpectedValue::ExactText(owned.into_boxed_slice())
            }
            DxfEntityEditValue::BinaryChunk(_) => return Err(invalid_internal_data()),
        };
        Ok(Self::Explicit(value))
    }
}

pub(crate) struct DxfEntityEditExpectation {
    raw_record_ordinal: u64,
    field: DxfEntityField,
    expected: DxfEntityExpectedField,
}

impl DxfEntityEditExpectation {
    pub(crate) const fn new(
        raw_record_ordinal: u64,
        field: DxfEntityField,
        expected: DxfEntityExpectedField,
    ) -> Self {
        Self {
            raw_record_ordinal,
            field,
            expected,
        }
    }
}

/// Expected semantic state retained without exposing edited payload bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityEditExpectedState {
    Explicit,
    Implicit,
}

/// Typed semantic postcondition failure for one edited field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityEditVerificationIssue {
    MissingEntity {
        raw_record_ordinal: u64,
    },
    MissingField {
        raw_record_ordinal: u64,
        field: DxfEntityField,
    },
    UnexpectedCardinality {
        raw_record_ordinal: u64,
        field: DxfEntityField,
        observed: DxfEntityFieldCardState,
    },
    UnexpectedSemanticShape {
        raw_record_ordinal: u64,
        field: DxfEntityField,
    },
    UnexpectedState {
        raw_record_ordinal: u64,
        field: DxfEntityField,
        expected: DxfEntityEditExpectedState,
        observed: DxfSemanticValueState,
    },
    ValueMismatch {
        raw_record_ordinal: u64,
        field: DxfEntityField,
    },
}

/// Identities and logical edit count proven by semantic verification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityEditVerificationReceipt {
    source_id: DxfSourceId,
    post_image_id: DxfSourceId,
    edit_count: u32,
}

impl DxfEntityEditVerificationReceipt {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn post_image_id(self) -> DxfSourceId {
        self.post_image_id
    }

    #[must_use]
    pub const fn edit_count(self) -> u64 {
        self.edit_count as u64
    }
}

/// Semantically verified post-image receipt paired with an executable inverse.
#[derive(Debug)]
pub struct DxfEntityEditVerificationJournal {
    receipt: DxfEntityEditVerificationReceipt,
    inverse: DxfTransactionPlan,
}

impl DxfEntityEditVerificationJournal {
    #[must_use]
    pub const fn receipt(&self) -> DxfEntityEditVerificationReceipt {
        self.receipt
    }

    #[must_use]
    pub const fn inverse_plan(&self) -> &DxfTransactionPlan {
        &self.inverse
    }

    #[must_use]
    pub fn into_parts(self) -> (DxfEntityEditVerificationReceipt, DxfTransactionPlan) {
        (self.receipt, self.inverse)
    }
}

/// Result of checking both semantic postconditions and exact transaction bytes.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityEditVerificationOutcome {
    Unavailable(DxfEntityEditVerificationIssue),
    Verified(DxfEntityEditVerificationJournal),
}

/// Create-new write and semantic receipts paired with an executable inverse.
#[derive(Debug)]
pub struct DxfEntityEditWriteJournal {
    write_receipt: DxfTransactionWriteReceipt,
    edit_count: u32,
    inverse: DxfTransactionPlan,
}

impl DxfEntityEditWriteJournal {
    #[must_use]
    pub const fn write_receipt(&self) -> DxfTransactionWriteReceipt {
        self.write_receipt
    }

    #[must_use]
    pub const fn verification_receipt(&self) -> DxfEntityEditVerificationReceipt {
        DxfEntityEditVerificationReceipt {
            source_id: self.write_receipt.source_id(),
            post_image_id: self.write_receipt.output_id(),
            edit_count: self.edit_count,
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
        DxfEntityEditVerificationReceipt,
        DxfTransactionPlan,
    ) {
        (
            self.write_receipt,
            self.verification_receipt(),
            self.inverse,
        )
    }
}

/// Result of create-new writing followed by strict semantic verification.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityEditWriteOutcome {
    Unavailable(DxfEntityEditVerificationIssue),
    Written(DxfEntityEditWriteJournal),
}

/// One immutable transaction plus the semantic states requested by its session.
pub struct DxfEntityEditPlan {
    transaction: DxfTransactionPlan,
    expectations: Box<[DxfEntityEditExpectation]>,
}

impl DxfEntityEditPlan {
    pub(crate) fn new(
        transaction: DxfTransactionPlan,
        expectations: Vec<DxfEntityEditExpectation>,
    ) -> Self {
        Self {
            transaction,
            expectations: expectations.into_boxed_slice(),
        }
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.transaction.source_id()
    }

    #[must_use]
    pub fn edit_count(&self) -> u64 {
        self.expectations.len() as u64
    }

    #[must_use]
    pub const fn transaction(&self) -> &DxfTransactionPlan {
        &self.transaction
    }

    #[must_use]
    pub fn into_transaction(self) -> DxfTransactionPlan {
        self.transaction
    }

    /// Composes raw supplemental work while retaining this plan's semantic
    /// field postconditions.
    ///
    /// This is the bridge for source-bound handle, owner, and placement work
    /// whose byte patches must commit with the entity-field transaction.
    pub fn compose_supplemental_transactions(
        mut self,
        source_document: DxfRawDocumentView<'_>,
        supplemental: &[&DxfTransactionPlan],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let plan_count = supplemental
            .len()
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        let mut plans = Vec::new();
        plans
            .try_reserve_exact(plan_count)
            .map_err(|_| out_of_memory())?;
        plans.push(&self.transaction);
        plans.extend_from_slice(supplemental);
        let transaction =
            source_document.compose_transaction_plans(&plans, profile, cancellation)?;
        self.transaction = transaction;
        Ok(self)
    }

    /// Verifies requested field semantics, exact post-image bytes, and inverse.
    pub fn verify_post_image(
        &self,
        source_document: DxfRawDocumentView<'_>,
        post_image: DxfRawDocumentView<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityEditVerificationOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        self.transaction
            .validate_source_precondition(source_document)?;
        validate_post_image_envelope(&self.transaction, post_image)?;
        let semantics = post_image.entity_field_semantic_directory(cancellation)?;
        for expectation in &self.expectations {
            if let Some(issue) = verify_expectation(post_image, &semantics, expectation)? {
                return Ok(DxfEntityEditVerificationOutcome::Unavailable(issue));
            }
            ensure_not_cancelled(cancellation)?;
        }
        let inverse = self.transaction.materialize_inverse_plan(
            source_document,
            post_image,
            profile,
            cancellation,
        )?;
        let edit_count =
            u32::try_from(self.expectations.len()).map_err(|_| invalid_internal_data())?;
        Ok(DxfEntityEditVerificationOutcome::Verified(
            DxfEntityEditVerificationJournal {
                receipt: DxfEntityEditVerificationReceipt {
                    source_id: source_document.source_id(),
                    post_image_id: post_image.source_id(),
                    edit_count,
                },
                inverse,
            },
        ))
    }

    /// Writes to a new path, strictly reparses, verifies semantics, and journals.
    ///
    /// Any failure or unavailable semantic postcondition after file creation
    /// removes the destination. An existing destination is never modified.
    pub fn write_reparse_verify_and_journal_to_new_file(
        &self,
        source_document: DxfRawDocumentView<'_>,
        destination: impl AsRef<Path>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfEntityEditWriteOutcome, DxfError> {
        let destination = destination.as_ref();
        let write_receipt = self.transaction.write_to_new_file(
            source_document,
            destination,
            cancellation,
            observer,
        )?;
        let outcome = self.reparse_and_verify_written_destination(
            source_document,
            destination,
            write_receipt,
            profile,
            cancellation,
        );
        match outcome {
            Ok(DxfEntityEditWriteOutcome::Written(journal)) => {
                Ok(DxfEntityEditWriteOutcome::Written(journal))
            }
            Ok(DxfEntityEditWriteOutcome::Unavailable(issue)) => {
                crate::transaction_write::remove_created_destination(destination)?;
                Ok(DxfEntityEditWriteOutcome::Unavailable(issue))
            }
            Err(primary) => {
                match crate::transaction_write::remove_created_destination(destination) {
                    Ok(()) => Err(primary),
                    Err(cleanup) => Err(cleanup),
                }
            }
        }
    }

    fn reparse_and_verify_written_destination(
        &self,
        source_document: DxfRawDocumentView<'_>,
        destination: &Path,
        write_receipt: DxfTransactionWriteReceipt,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityEditWriteOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let output_source = DxfFileSource::open(destination, profile)?;
        let options = DxfReadOptions::new(DxfReadMode::Strict, profile);
        let mut observer = NoopDxfReadObserver;
        match self.transaction.format() {
            DxfRawDocumentFormat::Ascii => {
                let post_image = DxfAsciiRawDocument::open(
                    &output_source,
                    options,
                    cancellation,
                    &mut observer,
                )?;
                self.finish_written_verification(
                    source_document,
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
                    source_document,
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
        source_document: DxfRawDocumentView<'_>,
        post_image: DxfRawDocumentView<'_>,
        write_receipt: DxfTransactionWriteReceipt,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityEditWriteOutcome, DxfError> {
        let journal =
            match self.verify_post_image(source_document, post_image, profile, cancellation)? {
                DxfEntityEditVerificationOutcome::Unavailable(issue) => {
                    return Ok(DxfEntityEditWriteOutcome::Unavailable(issue));
                }
                DxfEntityEditVerificationOutcome::Verified(journal) => journal,
            };
        let (verification_receipt, inverse) = journal.into_parts();
        validate_written_identities(write_receipt, verification_receipt)?;
        Ok(DxfEntityEditWriteOutcome::Written(
            DxfEntityEditWriteJournal {
                write_receipt,
                edit_count: verification_receipt.edit_count,
                inverse,
            },
        ))
    }
}

impl fmt::Debug for DxfEntityEditPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityEditPlan")
            .field("transaction", &self.transaction)
            .field("edit_count", &self.expectations.len())
            .finish()
    }
}

fn verify_expectation(
    document: DxfRawDocumentView<'_>,
    semantics: &crate::DxfEntityFieldSemanticDirectory,
    expectation: &DxfEntityEditExpectation,
) -> Result<Option<DxfEntityEditVerificationIssue>, DxfError> {
    let Some(entity) = semantics
        .evidence_directory()
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .find(|entity| entity.record().ordinal() == expectation.raw_record_ordinal)
    else {
        return Ok(Some(DxfEntityEditVerificationIssue::MissingEntity {
            raw_record_ordinal: expectation.raw_record_ordinal,
        }));
    };
    let Some(entry) = semantics.entry_for_field(entity, expectation.field)? else {
        return Ok(Some(DxfEntityEditVerificationIssue::MissingField {
            raw_record_ordinal: expectation.raw_record_ordinal,
            field: expectation.field,
        }));
    };
    let expected_card = match expectation.expected {
        DxfEntityExpectedField::Explicit(_) => DxfEntityFieldCardState::Unique,
        DxfEntityExpectedField::Implicit => DxfEntityFieldCardState::AbsentOptional,
    };
    if entry.card().state() != expected_card {
        return Ok(Some(
            DxfEntityEditVerificationIssue::UnexpectedCardinality {
                raw_record_ordinal: expectation.raw_record_ordinal,
                field: expectation.field,
                observed: entry.card().state(),
            },
        ));
    }
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Ok(Some(
            DxfEntityEditVerificationIssue::UnexpectedSemanticShape {
                raw_record_ordinal: expectation.raw_record_ordinal,
                field: expectation.field,
            },
        ));
    };
    match &expectation.expected {
        DxfEntityExpectedField::Explicit(expected) => {
            if value.state() != DxfSemanticValueState::Explicit {
                return Ok(Some(unexpected_state(
                    expectation,
                    DxfEntityEditExpectedState::Explicit,
                    value.state(),
                )));
            }
            if !explicit_value_matches(document, value.value().copied(), expected)? {
                return Ok(Some(DxfEntityEditVerificationIssue::ValueMismatch {
                    raw_record_ordinal: expectation.raw_record_ordinal,
                    field: expectation.field,
                }));
            }
        }
        DxfEntityExpectedField::Implicit => {
            let descriptor = expectation
                .field
                .descriptor()
                .ok_or_else(invalid_internal_data)?;
            let expected = match descriptor.default() {
                DxfEntityFieldDefault::None => DxfSemanticValueState::Absent,
                _ => DxfSemanticValueState::Defaulted,
            };
            if value.state() != expected {
                return Ok(Some(unexpected_state(
                    expectation,
                    DxfEntityEditExpectedState::Implicit,
                    value.state(),
                )));
            }
        }
    }
    Ok(None)
}

fn explicit_value_matches(
    document: DxfRawDocumentView<'_>,
    observed: Option<DxfEntityFieldValue>,
    expected: &DxfEntityExpectedValue,
) -> Result<bool, DxfError> {
    match (observed, expected) {
        (Some(DxfEntityFieldValue::Double(observed)), DxfEntityExpectedValue::Double(expected)) => {
            Ok(observed == *expected)
        }
        (Some(DxfEntityFieldValue::Handle(observed)), DxfEntityExpectedValue::Handle(expected)) => {
            Ok(observed == *expected)
        }
        (Some(DxfEntityFieldValue::Int16(observed)), DxfEntityExpectedValue::Int16(expected)) => {
            Ok(observed == *expected)
        }
        (Some(DxfEntityFieldValue::Int32(observed)), DxfEntityExpectedValue::Int32(expected)) => {
            Ok(observed == *expected)
        }
        (
            Some(DxfEntityFieldValue::ExactText(observed)),
            DxfEntityExpectedValue::ExactText(expected),
        ) => exact_text_matches(document, observed.value_span(), expected),
        _ => Ok(false),
    }
}

fn exact_text_matches(
    document: DxfRawDocumentView<'_>,
    span: crate::ByteSpan,
    expected: &[u8],
) -> Result<bool, DxfError> {
    if span.len() != expected.len() as u64 {
        return Ok(false);
    }
    let mut observed = Vec::new();
    observed
        .try_reserve_exact(expected.len())
        .map_err(|_| out_of_memory())?;
    observed.resize(expected.len(), 0);
    document.read_span(span, &mut observed)?;
    Ok(observed == expected)
}

fn validate_post_image_envelope(
    transaction: &DxfTransactionPlan,
    post_image: DxfRawDocumentView<'_>,
) -> Result<(), DxfError> {
    if post_image.source_len() != transaction.projected_len() {
        return Err(DxfError::TransactionPostImageLengthMismatch {
            expected: transaction.projected_len(),
            observed: post_image.source_len(),
        });
    }
    if post_image.format() != transaction.format() {
        return Err(DxfError::TransactionPostImageMismatch {
            span: crate::ByteSpan::new(0, 0).ok_or_else(invalid_internal_data)?,
        });
    }
    Ok(())
}

fn validate_written_identities(
    write: DxfTransactionWriteReceipt,
    verification: DxfEntityEditVerificationReceipt,
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
    Ok(())
}

const fn unexpected_state(
    expectation: &DxfEntityEditExpectation,
    expected: DxfEntityEditExpectedState,
    observed: DxfSemanticValueState,
) -> DxfEntityEditVerificationIssue {
    DxfEntityEditVerificationIssue::UnexpectedState {
        raw_record_ordinal: expectation.raw_record_ordinal,
        field: expectation.field,
        expected,
        observed,
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

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

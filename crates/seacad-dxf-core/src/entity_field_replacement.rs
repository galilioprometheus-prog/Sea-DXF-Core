//! Explicit replacement planning for one existing common-entity singleton.

use std::io;

use crate::{
    DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityClassification, DxfEntityEditValue, DxfEntityField, DxfEntityFieldCardState,
    DxfEntityFieldCardinality, DxfEntityFieldEvidenceDirectory, DxfEntityGroupEncodeIssue,
    DxfEntityGroupEncoder, DxfEntityKey, DxfEntityKnownClassification, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfResourceProfile, DxfSourceId, DxfTransactionPlan,
};

/// Typed reason why an explicit singleton replacement cannot be planned.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldReplacementIssue {
    DialectUnavailable {
        state: DxfAcadVersionState,
    },
    MissingEntity {
        raw_record_ordinal: u64,
    },
    WrongSection {
        classification: DxfEntityKnownClassification,
    },
    FieldAbsent {
        required: bool,
    },
    DuplicateSingleton {
        occurrence_count: u32,
    },
    SequenceOperationRequired {
        occurrence_count: u32,
    },
    Encoding(DxfEntityGroupEncodeIssue),
}

/// One source-bound single-group replacement and its immutable raw transaction.
#[derive(Debug)]
pub struct DxfEntityFieldReplacementPlan {
    key: DxfEntityKey,
    field: DxfEntityField,
    original_group_occurrence: u32,
    transaction: DxfTransactionPlan,
}

impl DxfEntityFieldReplacementPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.key.source_id()
    }

    #[must_use]
    pub const fn key(&self) -> DxfEntityKey {
        self.key
    }

    #[must_use]
    pub const fn field(&self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn original_group_occurrence(&self) -> u64 {
        self.original_group_occurrence as u64
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

/// Typed result of attempting one explicit common-field replacement.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityFieldReplacementOutcome {
    Unavailable(DxfEntityFieldReplacementIssue),
    Planned(DxfEntityFieldReplacementPlan),
}

impl DxfRawDocumentView<'_> {
    /// Plans replacement of one existing unique common-field singleton.
    pub fn plan_entity_field_replacement(
        self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldReplacementOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), evidence.source_id())?;
        ensure_source(self.source_id(), key.source_id())?;
        let version = match self.acad_version_report().state() {
            DxfAcadVersionState::Supported(version) => version,
            state => {
                return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                    DxfEntityFieldReplacementIssue::DialectUnavailable { state },
                ));
            }
        };
        let Some(entity) = evidence.entity_directory().entity_for_key(key)? else {
            return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                DxfEntityFieldReplacementIssue::MissingEntity {
                    raw_record_ordinal: key.raw_record_ordinal(),
                },
            ));
        };
        if let DxfEntityClassification::WrongSection(classification) = entity.classification() {
            return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                DxfEntityFieldReplacementIssue::WrongSection { classification },
            ));
        }
        let descriptor = field.descriptor().ok_or_else(invalid_internal_data)?;
        let card = evidence
            .card_for_field(entity, field)?
            .ok_or_else(invalid_internal_data)?;
        if descriptor.cardinality() == DxfEntityFieldCardinality::OptionalSequence {
            let occurrence_count = match card.state() {
                DxfEntityFieldCardState::AbsentOptional => 0,
                DxfEntityFieldCardState::Sequence { occurrence_count } => occurrence_count,
                _ => return Err(invalid_internal_data()),
            };
            return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                DxfEntityFieldReplacementIssue::SequenceOperationRequired { occurrence_count },
            ));
        }
        match card.state() {
            DxfEntityFieldCardState::AbsentRequired => {
                return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                    DxfEntityFieldReplacementIssue::FieldAbsent { required: true },
                ));
            }
            DxfEntityFieldCardState::AbsentOptional => {
                return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                    DxfEntityFieldReplacementIssue::FieldAbsent { required: false },
                ));
            }
            DxfEntityFieldCardState::Duplicate { occurrence_count } => {
                return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                    DxfEntityFieldReplacementIssue::DuplicateSingleton { occurrence_count },
                ));
            }
            DxfEntityFieldCardState::Sequence { .. } => return Err(invalid_internal_data()),
            DxfEntityFieldCardState::Unique => {}
        }
        let [member] = evidence.members_for_card(card)? else {
            return Err(invalid_internal_data());
        };
        let occurrence = evidence
            .occurrence_for_member(*member)
            .ok_or_else(invalid_internal_data)?;
        let encoded = match DxfEntityGroupEncoder::new(self.format(), version, profile).encode(
            *descriptor,
            value,
            cancellation,
        )? {
            Ok(encoded) => encoded,
            Err(issue) => {
                return Ok(DxfEntityFieldReplacementOutcome::Unavailable(
                    DxfEntityFieldReplacementIssue::Encoding(issue),
                ));
            }
        };
        let mut builder = self.transaction_plan_builder(profile)?;
        builder.replace_raw_span(
            occurrence.group().full_span(),
            encoded.bytes(),
            cancellation,
        )?;
        let transaction = builder.finish(cancellation)?;
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityFieldReplacementOutcome::Planned(
            DxfEntityFieldReplacementPlan {
                key,
                field,
                original_group_occurrence: u32::try_from(occurrence.group().occurrence())
                    .map_err(|_| invalid_internal_data())?,
                transaction,
            },
        ))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn plan_entity_field_replacement(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldReplacementOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_replacement(
            evidence,
            key,
            field,
            value,
            profile,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn plan_entity_field_replacement(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldReplacementOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_replacement(
            evidence,
            key,
            field,
            value,
            profile,
            cancellation,
        )
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
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

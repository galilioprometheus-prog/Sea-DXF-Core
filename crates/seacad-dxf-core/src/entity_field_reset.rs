//! Reset-to-default planning for one common-entity singleton.

use std::io;

use crate::{
    DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityClassification, DxfEntityField, DxfEntityFieldCardState, DxfEntityFieldCardinality,
    DxfEntityFieldEvidenceDirectory, DxfEntityFieldScope, DxfEntityKey,
    DxfEntityKnownClassification, DxfError, DxfIoOperation, DxfRawDocumentView, DxfResourceProfile,
    DxfSourceId, DxfTransactionPlan,
};

/// Typed reason why a common field cannot be reset by deleting one raw group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldResetIssue {
    DialectUnavailable {
        state: DxfAcadVersionState,
    },
    MissingEntity {
        raw_record_ordinal: u64,
    },
    WrongSection {
        classification: DxfEntityKnownClassification,
    },
    RequiredField,
    DuplicateSingleton {
        occurrence_count: u32,
    },
    SequenceOperationRequired {
        occurrence_count: u32,
    },
    NestedStructureOperationRequired {
        occurrence_count: u32,
    },
}

/// One source-bound deletion of an explicit optional singleton.
#[derive(Debug)]
pub struct DxfEntityFieldResetPlan {
    key: DxfEntityKey,
    field: DxfEntityField,
    original_group_occurrence: u32,
    transaction: DxfTransactionPlan,
}

impl DxfEntityFieldResetPlan {
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

/// Typed result of requesting one common-field reset to its implicit state.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityFieldResetOutcome {
    AlreadyImplicit {
        key: DxfEntityKey,
        field: DxfEntityField,
    },
    Unavailable(DxfEntityFieldResetIssue),
    Planned(DxfEntityFieldResetPlan),
}

impl DxfRawDocumentView<'_> {
    /// Plans deletion of one explicit unique optional common-field singleton.
    pub fn plan_entity_field_reset_to_default(
        self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldResetOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), evidence.source_id())?;
        ensure_source(self.source_id(), key.source_id())?;
        match self.acad_version_report().state() {
            DxfAcadVersionState::Supported(_) => {}
            state => {
                return Ok(DxfEntityFieldResetOutcome::Unavailable(
                    DxfEntityFieldResetIssue::DialectUnavailable { state },
                ));
            }
        }
        let Some(entity) = evidence.entity_directory().entity_for_key(key)? else {
            return Ok(DxfEntityFieldResetOutcome::Unavailable(
                DxfEntityFieldResetIssue::MissingEntity {
                    raw_record_ordinal: key.raw_record_ordinal(),
                },
            ));
        };
        if let DxfEntityClassification::WrongSection(classification) = entity.classification() {
            return Ok(DxfEntityFieldResetOutcome::Unavailable(
                DxfEntityFieldResetIssue::WrongSection { classification },
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
            return Ok(DxfEntityFieldResetOutcome::Unavailable(
                DxfEntityFieldResetIssue::SequenceOperationRequired { occurrence_count },
            ));
        }
        match card.state() {
            DxfEntityFieldCardState::AbsentOptional => {
                return Ok(DxfEntityFieldResetOutcome::AlreadyImplicit { key, field });
            }
            DxfEntityFieldCardState::AbsentRequired => {
                return Ok(DxfEntityFieldResetOutcome::Unavailable(
                    DxfEntityFieldResetIssue::RequiredField,
                ));
            }
            DxfEntityFieldCardState::Duplicate { occurrence_count } => {
                return Ok(DxfEntityFieldResetOutcome::Unavailable(
                    DxfEntityFieldResetIssue::DuplicateSingleton { occurrence_count },
                ));
            }
            DxfEntityFieldCardState::Sequence { .. } => return Err(invalid_internal_data()),
            DxfEntityFieldCardState::Unique => {}
        }
        if descriptor.cardinality() == DxfEntityFieldCardinality::RequiredSingleton {
            return Ok(DxfEntityFieldResetOutcome::Unavailable(
                DxfEntityFieldResetIssue::RequiredField,
            ));
        }
        if descriptor.scope() == DxfEntityFieldScope::ExtensionDictionaryApplicationGroup {
            return Ok(DxfEntityFieldResetOutcome::Unavailable(
                DxfEntityFieldResetIssue::NestedStructureOperationRequired {
                    occurrence_count: 1,
                },
            ));
        }
        let [member] = evidence.members_for_card(card)? else {
            return Err(invalid_internal_data());
        };
        let occurrence = evidence
            .occurrence_for_member(*member)
            .ok_or_else(invalid_internal_data)?;
        let mut builder = self.transaction_plan_builder(profile)?;
        builder.replace_raw_span(occurrence.group().full_span(), &[], cancellation)?;
        let transaction = builder.finish(cancellation)?;
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityFieldResetOutcome::Planned(
            DxfEntityFieldResetPlan {
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
    pub fn plan_entity_field_reset_to_default(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldResetOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_reset_to_default(
            evidence,
            key,
            field,
            profile,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn plan_entity_field_reset_to_default(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldResetOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_reset_to_default(
            evidence,
            key,
            field,
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

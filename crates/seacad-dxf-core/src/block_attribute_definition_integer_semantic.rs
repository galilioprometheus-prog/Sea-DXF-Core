//! Lazy typed integer semantics for classic ATTDEF records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockAttributeDefinitionCardDirectory, DxfBlockAttributeDefinitionValue,
    DxfBlockAttributeDefinitionValueCard, DxfBlockAttributeDefinitionValueCardState,
    DxfBlockAttributeDefinitionValueData, DxfBlockAttributeDefinitionValueEntry,
    DxfBlockAttributeDefinitionValueIssue, DxfBlockAttributeDefinitionValueRole,
    DxfCancellationToken, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "block.attribute_definition.integer";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionIntegerSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

pub type DxfBlockAttributeDefinitionSemanticInteger =
    DxfSemanticValue<i16, DxfBlockAttributeDefinitionIntegerSemanticIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionIntegerSemantics {
    record: DxfBlockAttributeDefinitionValueEntry,
    attribute_flags: DxfBlockAttributeDefinitionSemanticInteger,
    field_length: DxfBlockAttributeDefinitionSemanticInteger,
    text_generation_flags: DxfBlockAttributeDefinitionSemanticInteger,
    horizontal_justification: DxfBlockAttributeDefinitionSemanticInteger,
    vertical_justification: DxfBlockAttributeDefinitionSemanticInteger,
}

impl DxfBlockAttributeDefinitionIntegerSemantics {
    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.record
    }

    #[must_use]
    pub const fn attribute_flags(&self) -> &DxfBlockAttributeDefinitionSemanticInteger {
        &self.attribute_flags
    }

    #[must_use]
    pub const fn field_length(&self) -> &DxfBlockAttributeDefinitionSemanticInteger {
        &self.field_length
    }

    #[must_use]
    pub const fn text_generation_flags(&self) -> &DxfBlockAttributeDefinitionSemanticInteger {
        &self.text_generation_flags
    }

    #[must_use]
    pub const fn horizontal_justification(&self) -> &DxfBlockAttributeDefinitionSemanticInteger {
        &self.horizontal_justification
    }

    #[must_use]
    pub const fn vertical_justification(&self) -> &DxfBlockAttributeDefinitionSemanticInteger {
        &self.vertical_justification
    }

    #[must_use]
    pub fn attribute_flags_value(&self) -> Option<i16> {
        self.attribute_flags.value().copied()
    }

    #[must_use]
    pub fn field_length_value(&self) -> Option<i16> {
        self.field_length.value().copied()
    }

    #[must_use]
    pub fn text_generation_flags_value(&self) -> Option<i16> {
        self.text_generation_flags.value().copied()
    }

    #[must_use]
    pub fn horizontal_justification_value(&self) -> Option<i16> {
        self.horizontal_justification.value().copied()
    }

    #[must_use]
    pub fn vertical_justification_value(&self) -> Option<i16> {
        self.vertical_justification.value().copied()
    }

    #[must_use]
    pub fn is_invisible(&self) -> Option<bool> {
        flag(self.attribute_flags_value(), 1)
    }

    #[must_use]
    pub fn is_constant(&self) -> Option<bool> {
        flag(self.attribute_flags_value(), 2)
    }

    #[must_use]
    pub fn requires_verification(&self) -> Option<bool> {
        flag(self.attribute_flags_value(), 4)
    }

    #[must_use]
    pub fn is_preset(&self) -> Option<bool> {
        flag(self.attribute_flags_value(), 8)
    }

    #[must_use]
    pub fn is_backward(&self) -> Option<bool> {
        flag(self.text_generation_flags_value(), 2)
    }

    #[must_use]
    pub fn is_upside_down(&self) -> Option<bool> {
        flag(self.text_generation_flags_value(), 4)
    }
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionIntegerSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfBlockAttributeDefinitionCardDirectory,
}

impl DxfBlockAttributeDefinitionIntegerSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.block_attribute_definition_card_directory(cancellation)?;
        if cards.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: cards.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfBlockAttributeDefinitionCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBlockAttributeDefinitionValueEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionIntegerSemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        attribute_definition_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfBlockAttributeDefinitionValueEntry,
    ) -> Result<Option<DxfBlockAttributeDefinitionIntegerSemantics>, DxfError> {
        if self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.definition().record().ordinal())
            != Some(record)
        {
            return Ok(None);
        }
        attribute_definition_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_block_attribute_definition(
        &self,
        block_raw_ordinal: u64,
        attribute_definition_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionIntegerSemantics>, DxfError> {
        let Some(definition) = self
            .cards
            .evidence_directory()
            .definition_directory()
            .entries_for_block_raw_ordinal(block_raw_ordinal)
            .iter()
            .copied()
            .find(|entry| entry.attribute_definition_ordinal() == attribute_definition_ordinal)
        else {
            return Ok(None);
        };
        self.semantics_for_raw_record(definition.record().ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_attribute_definition_integer_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionIntegerSemanticDirectory, DxfError> {
        DxfBlockAttributeDefinitionIntegerSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_integer_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionIntegerSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_integer_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_integer_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionIntegerSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_integer_semantic_directory(cancellation)
    }
}

fn attribute_definition_semantics(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    record: DxfBlockAttributeDefinitionValueEntry,
) -> Result<DxfBlockAttributeDefinitionIntegerSemantics, DxfError> {
    use DxfBlockAttributeDefinitionValueRole::{
        AttributeFlags, FieldLength, HorizontalJustification, TextGenerationFlags,
        VerticalJustification,
    };
    Ok(DxfBlockAttributeDefinitionIntegerSemantics {
        record,
        attribute_flags: integer_semantic(
            cards,
            record,
            AttributeFlags,
            "attribute_flags",
            Absence::Required,
        )?,
        field_length: integer_semantic(
            cards,
            record,
            FieldLength,
            "field_length",
            Absence::DefaultZero,
        )?,
        text_generation_flags: integer_semantic(
            cards,
            record,
            TextGenerationFlags,
            "text_generation_flags",
            Absence::DefaultZero,
        )?,
        horizontal_justification: integer_semantic(
            cards,
            record,
            HorizontalJustification,
            "horizontal_justification",
            Absence::DefaultZero,
        )?,
        vertical_justification: integer_semantic(
            cards,
            record,
            VerticalJustification,
            "vertical_justification",
            Absence::DefaultZero,
        )?,
    })
}

fn integer_semantic(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    record: DxfBlockAttributeDefinitionValueEntry,
    role: DxfBlockAttributeDefinitionValueRole,
    field_id: &'static str,
    absence: Absence,
) -> Result<DxfBlockAttributeDefinitionSemanticInteger, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = cards
        .card_for_role(record.definition().record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfBlockAttributeDefinitionValueCardState::Absent => Ok(match absence {
            Absence::Required => DxfSemanticValue::invalid(
                DxfBlockAttributeDefinitionIntegerSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
            Absence::DefaultZero => DxfSemanticValue::defaulted(0, field),
        }),
        DxfBlockAttributeDefinitionValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfBlockAttributeDefinitionValueData::Int16(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(
                    DxfBlockAttributeDefinitionValueData::Text(_)
                    | DxfBlockAttributeDefinitionValueData::Double(_),
                ) => return Err(invalid_internal_data()),
                Err(DxfBlockAttributeDefinitionValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfBlockAttributeDefinitionIntegerSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfBlockAttributeDefinitionValueCardState::Multiple { occurrence_count } => {
            let first = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfBlockAttributeDefinitionIntegerSemanticIssue::MultipleValues {
                    occurrence_count,
                },
                field,
                Some(value_provenance(first)?),
            ))
        }
    }
}

fn unique_value(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    card: DxfBlockAttributeDefinitionValueCard,
) -> Result<DxfBlockAttributeDefinitionValue, DxfError> {
    let [member] = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    cards
        .value_for_member(*member)
        .ok_or_else(invalid_internal_data)
}

fn first_value(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    card: DxfBlockAttributeDefinitionValueCard,
) -> Result<DxfBlockAttributeDefinitionValue, DxfError> {
    let member = cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)?;
    cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(
    value: DxfBlockAttributeDefinitionValue,
) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn flag(value: Option<i16>, bit: i16) -> Option<bool> {
    Some(value? & bit != 0)
}

#[derive(Clone, Copy)]
enum Absence {
    Required,
    DefaultZero,
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

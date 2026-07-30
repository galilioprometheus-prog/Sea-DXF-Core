//! Lazy typed integer semantics for classic ATTRIB records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfInsertAttributeCardDirectory, DxfInsertAttributeValueCard,
    DxfInsertAttributeValueCardState, DxfInsertAttributeValueData, DxfInsertAttributeValueEntry,
    DxfInsertAttributeValueIssue, DxfInsertAttributeValueRole, DxfIoOperation, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "insert.attribute.integer";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeIntegerSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

pub type DxfInsertAttributeSemanticInteger =
    DxfSemanticValue<i16, DxfInsertAttributeIntegerSemanticIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeIntegerSemantics {
    record: DxfInsertAttributeValueEntry,
    attribute_flags: DxfInsertAttributeSemanticInteger,
    field_length: DxfInsertAttributeSemanticInteger,
    text_generation_flags: DxfInsertAttributeSemanticInteger,
    horizontal_justification: DxfInsertAttributeSemanticInteger,
    vertical_justification: DxfInsertAttributeSemanticInteger,
}

impl DxfInsertAttributeIntegerSemantics {
    #[must_use]
    pub const fn record(self) -> DxfInsertAttributeValueEntry {
        self.record
    }

    #[must_use]
    pub const fn attribute_flags(&self) -> &DxfInsertAttributeSemanticInteger {
        &self.attribute_flags
    }

    #[must_use]
    pub const fn field_length(&self) -> &DxfInsertAttributeSemanticInteger {
        &self.field_length
    }

    #[must_use]
    pub const fn text_generation_flags(&self) -> &DxfInsertAttributeSemanticInteger {
        &self.text_generation_flags
    }

    #[must_use]
    pub const fn horizontal_justification(&self) -> &DxfInsertAttributeSemanticInteger {
        &self.horizontal_justification
    }

    #[must_use]
    pub const fn vertical_justification(&self) -> &DxfInsertAttributeSemanticInteger {
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
pub struct DxfInsertAttributeIntegerSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfInsertAttributeCardDirectory,
}

impl DxfInsertAttributeIntegerSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.insert_attribute_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfInsertAttributeCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInsertAttributeValueEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributeIntegerSemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        attribute_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfInsertAttributeValueEntry,
    ) -> Result<Option<DxfInsertAttributeIntegerSemantics>, DxfError> {
        if self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal())
            != Some(record)
        {
            return Ok(None);
        }
        attribute_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_insert_sequence_attribute(
        &self,
        insert_raw_ordinal: u64,
        sequence_attribute_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributeIntegerSemantics>, DxfError> {
        let Some(records) = self
            .cards
            .evidence_directory()
            .sequence_directory()
            .attributes_for_insert_raw_ordinal(insert_raw_ordinal)
        else {
            return Ok(None);
        };
        let Some(record) = usize::try_from(sequence_attribute_ordinal)
            .ok()
            .and_then(|index| records.get(index))
            .copied()
        else {
            return Ok(None);
        };
        self.semantics_for_raw_record(record.ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_attribute_integer_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeIntegerSemanticDirectory, DxfError> {
        DxfInsertAttributeIntegerSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_integer_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeIntegerSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_integer_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_integer_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeIntegerSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_integer_semantic_directory(cancellation)
    }
}

fn attribute_semantics(
    cards: &DxfInsertAttributeCardDirectory,
    record: DxfInsertAttributeValueEntry,
) -> Result<DxfInsertAttributeIntegerSemantics, DxfError> {
    use DxfInsertAttributeValueRole::{
        AttributeFlags, FieldLength, HorizontalJustification, TextGenerationFlags,
        VerticalJustification,
    };
    Ok(DxfInsertAttributeIntegerSemantics {
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
    cards: &DxfInsertAttributeCardDirectory,
    record: DxfInsertAttributeValueEntry,
    role: DxfInsertAttributeValueRole,
    field_id: &'static str,
    absence: Absence,
) -> Result<DxfInsertAttributeSemanticInteger, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfInsertAttributeValueCardState::Absent => Ok(match absence {
            Absence::Required => DxfSemanticValue::invalid(
                DxfInsertAttributeIntegerSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
            Absence::DefaultZero => DxfSemanticValue::defaulted(0, field),
        }),
        DxfInsertAttributeValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfInsertAttributeValueData::Int16(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(
                    DxfInsertAttributeValueData::Text(_) | DxfInsertAttributeValueData::Double(_),
                ) => {
                    return Err(invalid_internal_data());
                }
                Err(DxfInsertAttributeValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfInsertAttributeIntegerSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfInsertAttributeValueCardState::Multiple { occurrence_count } => {
            let first = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfInsertAttributeIntegerSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(first)?),
            ))
        }
    }
}

fn unique_value(
    cards: &DxfInsertAttributeCardDirectory,
    card: DxfInsertAttributeValueCard,
) -> Result<crate::DxfInsertAttributeValue, DxfError> {
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
    cards: &DxfInsertAttributeCardDirectory,
    card: DxfInsertAttributeValueCard,
) -> Result<crate::DxfInsertAttributeValue, DxfError> {
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
    value: crate::DxfInsertAttributeValue,
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

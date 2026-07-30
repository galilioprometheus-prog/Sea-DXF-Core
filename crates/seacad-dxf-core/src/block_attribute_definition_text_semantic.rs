//! Lazy source-anchored text semantics for classic ATTDEF records.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockAttributeDefinitionCardDirectory,
    DxfBlockAttributeDefinitionTextValue, DxfBlockAttributeDefinitionValue,
    DxfBlockAttributeDefinitionValueCard, DxfBlockAttributeDefinitionValueCardState,
    DxfBlockAttributeDefinitionValueData, DxfBlockAttributeDefinitionValueEntry,
    DxfBlockAttributeDefinitionValueRole, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfSourceId,
};

const NAMESPACE: &str = "block.attribute_definition.text";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionTextSemanticIssue {
    MissingRequiredValue,
    MultipleValues { occurrence_count: u32 },
}

pub type DxfBlockAttributeDefinitionSemanticText = DxfSemanticValue<
    DxfBlockAttributeDefinitionTextValue,
    DxfBlockAttributeDefinitionTextSemanticIssue,
>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionTextStyleName {
    Source(DxfBlockAttributeDefinitionTextValue),
    Standard,
}

impl DxfBlockAttributeDefinitionTextStyleName {
    pub const STANDARD: &'static [u8] = b"STANDARD";

    #[must_use]
    pub const fn source(self) -> Option<DxfBlockAttributeDefinitionTextValue> {
        match self {
            Self::Source(source) => Some(source),
            Self::Standard => None,
        }
    }

    #[must_use]
    pub const fn is_standard_default(self) -> bool {
        matches!(self, Self::Standard)
    }
}

pub type DxfBlockAttributeDefinitionSemanticTextStyle = DxfSemanticValue<
    DxfBlockAttributeDefinitionTextStyleName,
    DxfBlockAttributeDefinitionTextSemanticIssue,
>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionTextSemantics {
    record: DxfBlockAttributeDefinitionValueEntry,
    default_value: DxfBlockAttributeDefinitionSemanticText,
    prompt: DxfBlockAttributeDefinitionSemanticText,
    attribute_tag: DxfBlockAttributeDefinitionSemanticText,
    text_style_name: DxfBlockAttributeDefinitionSemanticTextStyle,
}

impl DxfBlockAttributeDefinitionTextSemantics {
    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.record
    }

    #[must_use]
    pub const fn default_value(&self) -> &DxfBlockAttributeDefinitionSemanticText {
        &self.default_value
    }

    #[must_use]
    pub const fn prompt(&self) -> &DxfBlockAttributeDefinitionSemanticText {
        &self.prompt
    }

    #[must_use]
    pub const fn attribute_tag(&self) -> &DxfBlockAttributeDefinitionSemanticText {
        &self.attribute_tag
    }

    #[must_use]
    pub const fn text_style_name(&self) -> &DxfBlockAttributeDefinitionSemanticTextStyle {
        &self.text_style_name
    }
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionTextSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfBlockAttributeDefinitionCardDirectory,
}

impl DxfBlockAttributeDefinitionTextSemanticDirectory {
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
    ) -> Result<Option<DxfBlockAttributeDefinitionTextSemantics>, DxfError> {
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
    ) -> Result<Option<DxfBlockAttributeDefinitionTextSemantics>, DxfError> {
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
    ) -> Result<Option<DxfBlockAttributeDefinitionTextSemantics>, DxfError> {
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
    pub fn block_attribute_definition_text_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionTextSemanticDirectory, DxfError> {
        DxfBlockAttributeDefinitionTextSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_text_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionTextSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_text_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_text_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionTextSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_text_semantic_directory(cancellation)
    }
}

fn attribute_definition_semantics(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    record: DxfBlockAttributeDefinitionValueEntry,
) -> Result<DxfBlockAttributeDefinitionTextSemantics, DxfError> {
    Ok(DxfBlockAttributeDefinitionTextSemantics {
        record,
        default_value: required_text(
            cards,
            record,
            DxfBlockAttributeDefinitionValueRole::DefaultValue,
            "default_value",
        )?,
        prompt: required_text(
            cards,
            record,
            DxfBlockAttributeDefinitionValueRole::Prompt,
            "prompt",
        )?,
        attribute_tag: required_text(
            cards,
            record,
            DxfBlockAttributeDefinitionValueRole::AttributeTag,
            "attribute_tag",
        )?,
        text_style_name: text_style(cards, record)?,
    })
}

fn required_text(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    record: DxfBlockAttributeDefinitionValueEntry,
    role: DxfBlockAttributeDefinitionValueRole,
    field_id: &'static str,
) -> Result<DxfBlockAttributeDefinitionSemanticText, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfBlockAttributeDefinitionValueCardState::Absent => Ok(DxfSemanticValue::invalid(
            DxfBlockAttributeDefinitionTextSemanticIssue::MissingRequiredValue,
            field,
            None,
        )),
        DxfBlockAttributeDefinitionValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            match value.value() {
                Ok(DxfBlockAttributeDefinitionValueData::Text(text)) => {
                    Ok(DxfSemanticValue::explicit(text, field, raw))
                }
                Ok(
                    DxfBlockAttributeDefinitionValueData::Double(_)
                    | DxfBlockAttributeDefinitionValueData::Int16(_),
                )
                | Err(_) => Err(invalid_internal_data()),
            }
        }
        DxfBlockAttributeDefinitionValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn text_style(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    record: DxfBlockAttributeDefinitionValueEntry,
) -> Result<DxfBlockAttributeDefinitionSemanticTextStyle, DxfError> {
    let field = field(cards, "text_style_name");
    let card = card_for_role(
        cards,
        record,
        DxfBlockAttributeDefinitionValueRole::TextStyleName,
    )?;
    match card.state() {
        DxfBlockAttributeDefinitionValueCardState::Absent => Ok(DxfSemanticValue::defaulted(
            DxfBlockAttributeDefinitionTextStyleName::Standard,
            field,
        )),
        DxfBlockAttributeDefinitionValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            match value.value() {
                Ok(DxfBlockAttributeDefinitionValueData::Text(text)) => {
                    Ok(DxfSemanticValue::explicit(
                        DxfBlockAttributeDefinitionTextStyleName::Source(text),
                        field,
                        raw,
                    ))
                }
                Ok(
                    DxfBlockAttributeDefinitionValueData::Double(_)
                    | DxfBlockAttributeDefinitionValueData::Int16(_),
                )
                | Err(_) => Err(invalid_internal_data()),
            }
        }
        DxfBlockAttributeDefinitionValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn multiple_semantic<T>(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    card: DxfBlockAttributeDefinitionValueCard,
    field: DxfSemanticFieldProvenance,
    occurrence_count: u32,
) -> Result<DxfSemanticValue<T, DxfBlockAttributeDefinitionTextSemanticIssue>, DxfError> {
    let first = first_value(cards, card)?;
    Ok(DxfSemanticValue::invalid(
        DxfBlockAttributeDefinitionTextSemanticIssue::MultipleValues { occurrence_count },
        field,
        Some(value_provenance(first)?),
    ))
}

fn field(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    field_id: &'static str,
) -> DxfSemanticFieldProvenance {
    DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id)
}

fn card_for_role(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    record: DxfBlockAttributeDefinitionValueEntry,
    role: DxfBlockAttributeDefinitionValueRole,
) -> Result<DxfBlockAttributeDefinitionValueCard, DxfError> {
    cards
        .card_for_role(record.definition().record().ordinal(), role)
        .ok_or_else(invalid_internal_data)
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

//! Lazy source-anchored text semantics for classic ATTRIB records.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfInsertAttributeCardDirectory, DxfInsertAttributeTextValue, DxfInsertAttributeValueCard,
    DxfInsertAttributeValueCardState, DxfInsertAttributeValueData, DxfInsertAttributeValueEntry,
    DxfInsertAttributeValueRole, DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "insert.attribute.text";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeTextSemanticIssue {
    MissingRequiredValue,
    MultipleValues { occurrence_count: u32 },
}

pub type DxfInsertAttributeSemanticText =
    DxfSemanticValue<DxfInsertAttributeTextValue, DxfInsertAttributeTextSemanticIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeTextStyleName {
    Source(DxfInsertAttributeTextValue),
    Standard,
}

impl DxfInsertAttributeTextStyleName {
    pub const STANDARD: &'static [u8] = b"STANDARD";

    #[must_use]
    pub const fn source(self) -> Option<DxfInsertAttributeTextValue> {
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

pub type DxfInsertAttributeSemanticTextStyle =
    DxfSemanticValue<DxfInsertAttributeTextStyleName, DxfInsertAttributeTextSemanticIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeTextSemantics {
    record: DxfInsertAttributeValueEntry,
    text_value: DxfInsertAttributeSemanticText,
    attribute_tag: DxfInsertAttributeSemanticText,
    text_style_name: DxfInsertAttributeSemanticTextStyle,
}

impl DxfInsertAttributeTextSemantics {
    #[must_use]
    pub const fn record(self) -> DxfInsertAttributeValueEntry {
        self.record
    }

    #[must_use]
    pub const fn text_value(&self) -> &DxfInsertAttributeSemanticText {
        &self.text_value
    }

    #[must_use]
    pub const fn attribute_tag(&self) -> &DxfInsertAttributeSemanticText {
        &self.attribute_tag
    }

    #[must_use]
    pub const fn text_style_name(&self) -> &DxfInsertAttributeSemanticTextStyle {
        &self.text_style_name
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributeTextSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfInsertAttributeCardDirectory,
}

impl DxfInsertAttributeTextSemanticDirectory {
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
    ) -> Result<Option<DxfInsertAttributeTextSemantics>, DxfError> {
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
    ) -> Result<Option<DxfInsertAttributeTextSemantics>, DxfError> {
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
    ) -> Result<Option<DxfInsertAttributeTextSemantics>, DxfError> {
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
    pub fn insert_attribute_text_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeTextSemanticDirectory, DxfError> {
        DxfInsertAttributeTextSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_text_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeTextSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_text_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_text_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeTextSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_text_semantic_directory(cancellation)
    }
}

fn attribute_semantics(
    cards: &DxfInsertAttributeCardDirectory,
    record: DxfInsertAttributeValueEntry,
) -> Result<DxfInsertAttributeTextSemantics, DxfError> {
    Ok(DxfInsertAttributeTextSemantics {
        record,
        text_value: required_text(
            cards,
            record,
            DxfInsertAttributeValueRole::TextValue,
            "text_value",
        )?,
        attribute_tag: required_text(
            cards,
            record,
            DxfInsertAttributeValueRole::AttributeTag,
            "attribute_tag",
        )?,
        text_style_name: text_style(cards, record)?,
    })
}

fn required_text(
    cards: &DxfInsertAttributeCardDirectory,
    record: DxfInsertAttributeValueEntry,
    role: DxfInsertAttributeValueRole,
    field_id: &'static str,
) -> Result<DxfInsertAttributeSemanticText, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfInsertAttributeValueCardState::Absent => Ok(DxfSemanticValue::invalid(
            DxfInsertAttributeTextSemanticIssue::MissingRequiredValue,
            field,
            None,
        )),
        DxfInsertAttributeValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            match value.value() {
                Ok(DxfInsertAttributeValueData::Text(text)) => {
                    Ok(DxfSemanticValue::explicit(text, field, raw))
                }
                Ok(
                    DxfInsertAttributeValueData::Double(_) | DxfInsertAttributeValueData::Int16(_),
                )
                | Err(_) => Err(invalid_internal_data()),
            }
        }
        DxfInsertAttributeValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn text_style(
    cards: &DxfInsertAttributeCardDirectory,
    record: DxfInsertAttributeValueEntry,
) -> Result<DxfInsertAttributeSemanticTextStyle, DxfError> {
    let field = field(cards, "text_style_name");
    let card = card_for_role(cards, record, DxfInsertAttributeValueRole::TextStyleName)?;
    match card.state() {
        DxfInsertAttributeValueCardState::Absent => Ok(DxfSemanticValue::defaulted(
            DxfInsertAttributeTextStyleName::Standard,
            field,
        )),
        DxfInsertAttributeValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            match value.value() {
                Ok(DxfInsertAttributeValueData::Text(text)) => Ok(DxfSemanticValue::explicit(
                    DxfInsertAttributeTextStyleName::Source(text),
                    field,
                    raw,
                )),
                Ok(
                    DxfInsertAttributeValueData::Double(_) | DxfInsertAttributeValueData::Int16(_),
                )
                | Err(_) => Err(invalid_internal_data()),
            }
        }
        DxfInsertAttributeValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn multiple_semantic<T>(
    cards: &DxfInsertAttributeCardDirectory,
    card: DxfInsertAttributeValueCard,
    field: DxfSemanticFieldProvenance,
    occurrence_count: u32,
) -> Result<DxfSemanticValue<T, DxfInsertAttributeTextSemanticIssue>, DxfError> {
    let first = first_value(cards, card)?;
    Ok(DxfSemanticValue::invalid(
        DxfInsertAttributeTextSemanticIssue::MultipleValues { occurrence_count },
        field,
        Some(value_provenance(first)?),
    ))
}

fn field(
    cards: &DxfInsertAttributeCardDirectory,
    field_id: &'static str,
) -> DxfSemanticFieldProvenance {
    DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id)
}

fn card_for_role(
    cards: &DxfInsertAttributeCardDirectory,
    record: DxfInsertAttributeValueEntry,
    role: DxfInsertAttributeValueRole,
) -> Result<DxfInsertAttributeValueCard, DxfError> {
    cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)
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

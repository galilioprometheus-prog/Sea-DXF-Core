//! Scalar text/name selection over text-symbol cardinality cards.

use std::io;

use crate::{
    DxfError, DxfIoOperation, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfTextSymbolCardDirectory, DxfTextSymbolRecordEntry, DxfTextSymbolSemanticStyle,
    DxfTextSymbolSemanticText, DxfTextSymbolStyleName, DxfTextSymbolTextIssue, DxfTextSymbolValue,
    DxfTextSymbolValueCard, DxfTextSymbolValueCardState, DxfTextSymbolValueData,
    DxfTextSymbolValueRole,
};

pub(super) fn required_text(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field_id: &'static str,
) -> Result<DxfTextSymbolSemanticText, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), namespace, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfTextSymbolValueCardState::Absent => Ok(DxfSemanticValue::invalid(
            DxfTextSymbolTextIssue::MissingRequiredValue,
            field,
            None,
        )),
        DxfTextSymbolValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            ensure_text(value)?;
            Ok(DxfSemanticValue::explicit(
                value,
                field,
                value_provenance(value)?,
            ))
        }
        DxfTextSymbolValueCardState::Multiple { occurrence_count } => {
            multiple(cards, card, field, occurrence_count)
        }
    }
}

pub(super) fn style_name(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    namespace: &'static str,
) -> Result<DxfTextSymbolSemanticStyle, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), namespace, "style_name");
    let card = card_for_role(cards, record, DxfTextSymbolValueRole::StyleName)?;
    match card.state() {
        DxfTextSymbolValueCardState::Absent => Ok(DxfSemanticValue::defaulted(
            DxfTextSymbolStyleName::Standard,
            field,
        )),
        DxfTextSymbolValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            ensure_text(value)?;
            Ok(DxfSemanticValue::explicit(
                DxfTextSymbolStyleName::Source(value),
                field,
                value_provenance(value)?,
            ))
        }
        DxfTextSymbolValueCardState::Multiple { occurrence_count } => {
            multiple(cards, card, field, occurrence_count)
        }
    }
}

fn multiple<T>(
    cards: &DxfTextSymbolCardDirectory,
    card: DxfTextSymbolValueCard,
    field: DxfSemanticFieldProvenance,
    occurrence_count: u32,
) -> Result<DxfSemanticValue<T, DxfTextSymbolTextIssue>, DxfError> {
    let first = first_value(cards, card)?;
    Ok(DxfSemanticValue::invalid(
        DxfTextSymbolTextIssue::MultipleValues { occurrence_count },
        field,
        Some(value_provenance(first)?),
    ))
}

fn card_for_role(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
) -> Result<DxfTextSymbolValueCard, DxfError> {
    cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)
}

fn unique_value(
    cards: &DxfTextSymbolCardDirectory,
    card: DxfTextSymbolValueCard,
) -> Result<DxfTextSymbolValue, DxfError> {
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
    cards: &DxfTextSymbolCardDirectory,
    card: DxfTextSymbolValueCard,
) -> Result<DxfTextSymbolValue, DxfError> {
    cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .and_then(|member| cards.value_for_member(member))
        .ok_or_else(invalid_internal_data)
}

fn ensure_text(value: DxfTextSymbolValue) -> Result<(), DxfError> {
    if value.data() == DxfTextSymbolValueData::Text {
        Ok(())
    } else {
        Err(invalid_internal_data())
    }
}

fn value_provenance(value: DxfTextSymbolValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

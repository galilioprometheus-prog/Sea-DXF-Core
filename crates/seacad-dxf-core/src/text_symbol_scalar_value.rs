//! Shared scalar selection for TEXT and SHAPE numeric semantics.

use std::io;

use crate::{
    DxfDouble, DxfError, DxfIoOperation, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfTextSymbolCardDirectory, DxfTextSymbolNumericIssue,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue, DxfTextSymbolValue, DxfTextSymbolValueCard,
    DxfTextSymbolValueCardState, DxfTextSymbolValueData, DxfTextSymbolValueRole,
};

#[derive(Clone, Copy)]
pub(crate) enum DxfScalarRule<T> {
    Required,
    Defaulted(T),
    Optional,
}

pub(crate) fn semantic_double(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field_id: &'static str,
    rule: DxfScalarRule<DxfDouble>,
) -> Result<DxfSemanticValue<DxfDouble, DxfTextSymbolScalarIssue>, DxfError> {
    semantic_value(
        cards,
        record,
        role,
        namespace,
        field_id,
        rule,
        |data| match data {
            DxfTextSymbolValueData::Double(value) => Some(value),
            _ => None,
        },
    )
}

pub(crate) fn semantic_i16(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field_id: &'static str,
    rule: DxfScalarRule<i16>,
) -> Result<DxfSemanticValue<i16, DxfTextSymbolScalarIssue>, DxfError> {
    semantic_value(
        cards,
        record,
        role,
        namespace,
        field_id,
        rule,
        |data| match data {
            DxfTextSymbolValueData::Int16(value) => Some(value),
            _ => None,
        },
    )
}

pub(crate) fn semantic_i32(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field_id: &'static str,
    rule: DxfScalarRule<i32>,
) -> Result<DxfSemanticValue<i32, DxfTextSymbolScalarIssue>, DxfError> {
    semantic_value(
        cards,
        record,
        role,
        namespace,
        field_id,
        rule,
        |data| match data {
            DxfTextSymbolValueData::Int32(value) => Some(value),
            _ => None,
        },
    )
}

fn semantic_value<T: Copy>(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    role: DxfTextSymbolValueRole,
    namespace: &'static str,
    field_id: &'static str,
    rule: DxfScalarRule<T>,
    wire_value: impl FnOnce(DxfTextSymbolValueData) -> Option<Result<T, DxfTextSymbolNumericIssue>>,
) -> Result<DxfSemanticValue<T, DxfTextSymbolScalarIssue>, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), namespace, field_id);
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfTextSymbolValueCardState::Absent => Ok(match rule {
            DxfScalarRule::Required => DxfSemanticValue::invalid(
                DxfTextSymbolScalarIssue::MissingRequiredValue,
                field,
                None,
            ),
            DxfScalarRule::Defaulted(value) => DxfSemanticValue::defaulted(value, field),
            DxfScalarRule::Optional => DxfSemanticValue::absent(field),
        }),
        DxfTextSymbolValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            let result = wire_value(value.data()).ok_or_else(invalid_internal_data)?;
            Ok(match result {
                Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                Err(DxfTextSymbolNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfTextSymbolScalarIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfTextSymbolValueCardState::Multiple { occurrence_count } => {
            let primary = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfTextSymbolScalarIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(primary)?),
            ))
        }
    }
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

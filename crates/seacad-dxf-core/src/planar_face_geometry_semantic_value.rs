//! Shared scalar selection for planar-face semantics.

use std::io;

use crate::{
    DxfDouble, DxfError, DxfIoOperation, DxfPlanarFaceCardDirectory, DxfPlanarFaceNumber,
    DxfPlanarFaceNumericIssue, DxfPlanarFaceRecordEntry, DxfPlanarFaceValue,
    DxfPlanarFaceValueCard, DxfPlanarFaceValueCardState, DxfPlanarFaceValueRole,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    planar_face_geometry_semantic::{
        DxfPlanarFaceDoubleSemanticValue, DxfPlanarFaceInt16SemanticValue,
        DxfPlanarFaceSemanticIssue,
    },
};

pub(crate) fn defaulted_corner(
    source_id: DxfSourceId,
    namespace: &'static str,
    fields: [&'static str; 3],
    source: &[DxfPlanarFaceDoubleSemanticValue; 3],
) -> [DxfPlanarFaceDoubleSemanticValue; 3] {
    std::array::from_fn(|index| {
        let field = DxfSemanticFieldProvenance::new(source_id, namespace, fields[index]);
        match source[index].value().copied() {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfPlanarFaceSemanticIssue::UnavailableDefaultSource,
                field,
                None,
            ),
        }
    })
}

pub(crate) fn semantic_double(
    cards: &DxfPlanarFaceCardDirectory,
    record: DxfPlanarFaceRecordEntry,
    role: DxfPlanarFaceValueRole,
    namespace: &'static str,
    field_id: &'static str,
    default: Option<DxfDouble>,
) -> Result<DxfPlanarFaceDoubleSemanticValue, DxfError> {
    semantic_number(
        cards,
        record,
        role,
        namespace,
        field_id,
        default,
        |number| match number {
            DxfPlanarFaceNumber::Double(value) => Some(value),
            _ => None,
        },
    )
}

pub(crate) fn semantic_i16(
    cards: &DxfPlanarFaceCardDirectory,
    record: DxfPlanarFaceRecordEntry,
    role: DxfPlanarFaceValueRole,
    namespace: &'static str,
    field_id: &'static str,
    default: Option<i16>,
) -> Result<DxfPlanarFaceInt16SemanticValue, DxfError> {
    semantic_number(
        cards,
        record,
        role,
        namespace,
        field_id,
        default,
        |number| match number {
            DxfPlanarFaceNumber::Int16(value) => Some(value),
            _ => None,
        },
    )
}

fn semantic_number<T: Copy>(
    cards: &DxfPlanarFaceCardDirectory,
    record: DxfPlanarFaceRecordEntry,
    role: DxfPlanarFaceValueRole,
    namespace: &'static str,
    field_id: &'static str,
    default: Option<T>,
    decode: impl FnOnce(DxfPlanarFaceNumber) -> Option<T>,
) -> Result<DxfSemanticValue<T, DxfPlanarFaceSemanticIssue>, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), namespace, field_id);
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfPlanarFaceValueCardState::Absent => Ok(match default {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfPlanarFaceSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
        }),
        DxfPlanarFaceValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(number) => DxfSemanticValue::explicit(
                    decode(number).ok_or_else(invalid_internal_data)?,
                    field,
                    raw,
                ),
                Err(DxfPlanarFaceNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfPlanarFaceSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfPlanarFaceValueCardState::Multiple { occurrence_count } => {
            let primary = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfPlanarFaceSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(primary)?),
            ))
        }
    }
}

fn unique_value(
    cards: &DxfPlanarFaceCardDirectory,
    card: DxfPlanarFaceValueCard,
) -> Result<DxfPlanarFaceValue, DxfError> {
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
    cards: &DxfPlanarFaceCardDirectory,
    card: DxfPlanarFaceValueCard,
) -> Result<DxfPlanarFaceValue, DxfError> {
    cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .and_then(|member| cards.value_for_member(member))
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(value: DxfPlanarFaceValue) -> Result<DxfRawValueProvenance, DxfError> {
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

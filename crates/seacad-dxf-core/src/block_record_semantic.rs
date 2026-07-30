//! Lazy typed BLOCK record semantics over M10.1c cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockRecordCardDirectory,
    DxfBlockRecordCardMember, DxfBlockRecordTextValue, DxfBlockRecordValueCard,
    DxfBlockRecordValueCardState, DxfBlockRecordValueData, DxfBlockRecordValueEntry,
    DxfBlockRecordValueIssue, DxfBlockRecordValueRole, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "block.record";

/// Why one reviewed BLOCK field has no usable value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockRecordSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored state for one BLOCK text field.
pub type DxfBlockRecordSemanticText =
    DxfSemanticValue<DxfBlockRecordTextValue, DxfBlockRecordSemanticIssue>;
/// Source-anchored state for one BLOCK double field.
pub type DxfBlockRecordSemanticDouble = DxfSemanticValue<DxfDouble, DxfBlockRecordSemanticIssue>;
/// Source-anchored state for one BLOCK signed integer field.
pub type DxfBlockRecordSemanticInteger = DxfSemanticValue<i16, DxfBlockRecordSemanticIssue>;

/// Lazy reviewed semantics for one exact BLOCK record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordSemantics {
    record: DxfBlockRecordValueEntry,
    primary_name: DxfBlockRecordSemanticText,
    flags: DxfBlockRecordSemanticInteger,
    base_point: [DxfBlockRecordSemanticDouble; 3],
    secondary_name: DxfBlockRecordSemanticText,
    xref_path: DxfBlockRecordSemanticText,
    description: DxfBlockRecordSemanticText,
}

impl DxfBlockRecordSemantics {
    #[must_use]
    pub const fn record(self) -> DxfBlockRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn primary_name(&self) -> &DxfBlockRecordSemanticText {
        &self.primary_name
    }

    #[must_use]
    pub const fn flags(&self) -> &DxfBlockRecordSemanticInteger {
        &self.flags
    }

    #[must_use]
    pub const fn base_point(&self) -> &[DxfBlockRecordSemanticDouble; 3] {
        &self.base_point
    }

    #[must_use]
    pub const fn secondary_name(&self) -> &DxfBlockRecordSemanticText {
        &self.secondary_name
    }

    #[must_use]
    pub const fn xref_path(&self) -> &DxfBlockRecordSemanticText {
        &self.xref_path
    }

    #[must_use]
    pub const fn description(&self) -> &DxfBlockRecordSemanticText {
        &self.description
    }

    #[must_use]
    pub fn flags_value(&self) -> Option<i16> {
        self.flags.value().copied()
    }

    #[must_use]
    pub fn base_point_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.base_point)
    }

    #[must_use]
    pub fn is_anonymous(&self) -> Option<bool> {
        self.flag_bit(1)
    }

    #[must_use]
    pub fn has_non_constant_attribute_definitions(&self) -> Option<bool> {
        self.flag_bit(2)
    }

    #[must_use]
    pub fn is_external_reference(&self) -> Option<bool> {
        self.flag_bit(4)
    }

    #[must_use]
    pub fn is_external_reference_overlay(&self) -> Option<bool> {
        self.flag_bit(8)
    }

    #[must_use]
    pub fn is_externally_dependent(&self) -> Option<bool> {
        self.flag_bit(16)
    }

    #[must_use]
    pub fn is_resolved_external_reference_or_dependent(&self) -> Option<bool> {
        self.flag_bit(32)
    }

    #[must_use]
    pub fn is_referenced_external_reference(&self) -> Option<bool> {
        self.flag_bit(64)
    }

    fn flag_bit(&self, bit: i16) -> Option<bool> {
        Some(self.flags_value()? & bit != 0)
    }
}

/// Immutable lazy BLOCK semantics retaining all M10.1c evidence.
#[derive(Debug)]
pub struct DxfBlockRecordSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfBlockRecordCardDirectory,
}

impl DxfBlockRecordSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.block_record_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfBlockRecordCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBlockRecordValueEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn semantics_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfBlockRecordSemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_block_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        record_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfBlockRecordValueEntry,
    ) -> Result<Option<DxfBlockRecordSemantics>, DxfError> {
        let raw_ordinal = record.definition().block_record().ordinal();
        if self
            .cards
            .evidence_directory()
            .record_for_block_raw_ordinal(raw_ordinal)
            != Some(record)
        {
            return Ok(None);
        }
        record_semantics(&self.cards, record).map(Some)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_record_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordSemanticDirectory, DxfError> {
        DxfBlockRecordSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_record_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_record_semantic_directory(cancellation)
    }
}

fn record_semantics(
    cards: &DxfBlockRecordCardDirectory,
    record: DxfBlockRecordValueEntry,
) -> Result<DxfBlockRecordSemantics, DxfError> {
    Ok(DxfBlockRecordSemantics {
        record,
        primary_name: text_semantic(
            cards,
            record,
            DxfBlockRecordValueRole::PrimaryName,
            "primary_name",
            true,
        )?,
        flags: integer_semantic(cards, record, DxfBlockRecordValueRole::Flags, "flags")?,
        base_point: [
            double_semantic(
                cards,
                record,
                DxfBlockRecordValueRole::BasePointX,
                "base_point_x",
            )?,
            double_semantic(
                cards,
                record,
                DxfBlockRecordValueRole::BasePointY,
                "base_point_y",
            )?,
            double_semantic(
                cards,
                record,
                DxfBlockRecordValueRole::BasePointZ,
                "base_point_z",
            )?,
        ],
        secondary_name: text_semantic(
            cards,
            record,
            DxfBlockRecordValueRole::SecondaryName,
            "secondary_name",
            true,
        )?,
        xref_path: text_semantic(
            cards,
            record,
            DxfBlockRecordValueRole::XrefPath,
            "xref_path",
            false,
        )?,
        description: text_semantic(
            cards,
            record,
            DxfBlockRecordValueRole::Description,
            "description",
            false,
        )?,
    })
}

fn text_semantic(
    cards: &DxfBlockRecordCardDirectory,
    record: DxfBlockRecordValueEntry,
    role: DxfBlockRecordValueRole,
    field_id: &'static str,
    required: bool,
) -> Result<DxfBlockRecordSemanticText, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfBlockRecordValueCardState::Absent => Ok(if required {
            DxfSemanticValue::invalid(
                DxfBlockRecordSemanticIssue::MissingRequiredValue,
                field,
                None,
            )
        } else {
            DxfSemanticValue::absent(field)
        }),
        DxfBlockRecordValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            match value.value() {
                Ok(DxfBlockRecordValueData::Text(text)) => {
                    Ok(DxfSemanticValue::explicit(text, field, raw))
                }
                Ok(DxfBlockRecordValueData::Double(_) | DxfBlockRecordValueData::Int16(_))
                | Err(_) => Err(invalid_internal_data()),
            }
        }
        DxfBlockRecordValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn double_semantic(
    cards: &DxfBlockRecordCardDirectory,
    record: DxfBlockRecordValueEntry,
    role: DxfBlockRecordValueRole,
    field_id: &'static str,
) -> Result<DxfBlockRecordSemanticDouble, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfBlockRecordValueCardState::Absent => Ok(DxfSemanticValue::invalid(
            DxfBlockRecordSemanticIssue::MissingRequiredValue,
            field,
            None,
        )),
        DxfBlockRecordValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfBlockRecordValueData::Double(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfBlockRecordValueData::Text(_) | DxfBlockRecordValueData::Int16(_)) => {
                    return Err(invalid_internal_data());
                }
                Err(DxfBlockRecordValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfBlockRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfBlockRecordValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn integer_semantic(
    cards: &DxfBlockRecordCardDirectory,
    record: DxfBlockRecordValueEntry,
    role: DxfBlockRecordValueRole,
    field_id: &'static str,
) -> Result<DxfBlockRecordSemanticInteger, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfBlockRecordValueCardState::Absent => Ok(DxfSemanticValue::invalid(
            DxfBlockRecordSemanticIssue::MissingRequiredValue,
            field,
            None,
        )),
        DxfBlockRecordValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfBlockRecordValueData::Int16(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfBlockRecordValueData::Text(_) | DxfBlockRecordValueData::Double(_)) => {
                    return Err(invalid_internal_data());
                }
                Err(DxfBlockRecordValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfBlockRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfBlockRecordValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn multiple_semantic<T>(
    cards: &DxfBlockRecordCardDirectory,
    card: DxfBlockRecordValueCard,
    field: DxfSemanticFieldProvenance,
    occurrence_count: u32,
) -> Result<DxfSemanticValue<T, DxfBlockRecordSemanticIssue>, DxfError> {
    let value = first_value(cards, card)?;
    Ok(DxfSemanticValue::invalid(
        DxfBlockRecordSemanticIssue::MultipleValues { occurrence_count },
        field,
        Some(value_provenance(value)?),
    ))
}

fn field(
    cards: &DxfBlockRecordCardDirectory,
    field_id: &'static str,
) -> DxfSemanticFieldProvenance {
    DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id)
}

fn card_for_role(
    cards: &DxfBlockRecordCardDirectory,
    record: DxfBlockRecordValueEntry,
    role: DxfBlockRecordValueRole,
) -> Result<DxfBlockRecordValueCard, DxfError> {
    cards
        .card_for_role(record.definition().block_record().ordinal(), role)
        .ok_or_else(invalid_internal_data)
}

fn unique_value(
    cards: &DxfBlockRecordCardDirectory,
    card: DxfBlockRecordValueCard,
) -> Result<crate::DxfBlockRecordValue, DxfError> {
    let members = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
        return Err(invalid_internal_data());
    };
    value_for_member(cards, *member)
}

fn first_value(
    cards: &DxfBlockRecordCardDirectory,
    card: DxfBlockRecordValueCard,
) -> Result<crate::DxfBlockRecordValue, DxfError> {
    let member = cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)?;
    value_for_member(cards, member)
}

fn value_for_member(
    cards: &DxfBlockRecordCardDirectory,
    member: DxfBlockRecordCardMember,
) -> Result<crate::DxfBlockRecordValue, DxfError> {
    cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(value: crate::DxfBlockRecordValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn triple_value(values: &[DxfBlockRecordSemanticDouble; 3]) -> Option<[DxfDouble; 3]> {
    Some([
        values[0].value().copied()?,
        values[1].value().copied()?,
        values[2].value().copied()?,
    ])
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

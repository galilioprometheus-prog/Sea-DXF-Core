//! Lazy typed INSERT semantics over fixed cardinality cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfInsertRecordCardDirectory, DxfInsertRecordCardMember,
    DxfInsertRecordTextValue, DxfInsertRecordValue, DxfInsertRecordValueCard,
    DxfInsertRecordValueCardState, DxfInsertRecordValueData, DxfInsertRecordValueEntry,
    DxfInsertRecordValueIssue, DxfInsertRecordValueRole, DxfIoOperation, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "insert.record";
const ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const ONE: DxfDouble = DxfDouble::from_bits(1.0_f64.to_bits());

/// Why one reviewed INSERT field has no usable value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertRecordSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

pub type DxfInsertRecordSemanticText =
    DxfSemanticValue<DxfInsertRecordTextValue, DxfInsertRecordSemanticIssue>;
pub type DxfInsertRecordSemanticDouble = DxfSemanticValue<DxfDouble, DxfInsertRecordSemanticIssue>;
pub type DxfInsertRecordSemanticInteger = DxfSemanticValue<i16, DxfInsertRecordSemanticIssue>;

/// Reviewed typed semantics for one exact INSERT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertRecordSemantics {
    record: DxfInsertRecordValueEntry,
    block_name: DxfInsertRecordSemanticText,
    insertion_point: [DxfInsertRecordSemanticDouble; 3],
    scale_factors: [DxfInsertRecordSemanticDouble; 3],
    rotation_degrees: DxfInsertRecordSemanticDouble,
    array_counts: [DxfInsertRecordSemanticInteger; 2],
    array_spacing: [DxfInsertRecordSemanticDouble; 2],
    attributes_follow: DxfInsertRecordSemanticInteger,
    extrusion: [DxfInsertRecordSemanticDouble; 3],
}

impl DxfInsertRecordSemantics {
    #[must_use]
    pub const fn record(self) -> DxfInsertRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn block_name(&self) -> &DxfInsertRecordSemanticText {
        &self.block_name
    }

    #[must_use]
    pub const fn insertion_point(&self) -> &[DxfInsertRecordSemanticDouble; 3] {
        &self.insertion_point
    }

    #[must_use]
    pub const fn scale_factors(&self) -> &[DxfInsertRecordSemanticDouble; 3] {
        &self.scale_factors
    }

    #[must_use]
    pub const fn rotation_degrees(&self) -> &DxfInsertRecordSemanticDouble {
        &self.rotation_degrees
    }

    #[must_use]
    pub const fn array_counts(&self) -> &[DxfInsertRecordSemanticInteger; 2] {
        &self.array_counts
    }

    #[must_use]
    pub const fn array_spacing(&self) -> &[DxfInsertRecordSemanticDouble; 2] {
        &self.array_spacing
    }

    #[must_use]
    pub const fn attributes_follow(&self) -> &DxfInsertRecordSemanticInteger {
        &self.attributes_follow
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfInsertRecordSemanticDouble; 3] {
        &self.extrusion
    }

    #[must_use]
    pub fn insertion_point_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.insertion_point)
    }

    #[must_use]
    pub fn scale_factor_values(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.scale_factors)
    }

    #[must_use]
    pub fn rotation_degrees_value(&self) -> Option<DxfDouble> {
        self.rotation_degrees.value().copied()
    }

    #[must_use]
    pub fn array_count_values(&self) -> Option<[i16; 2]> {
        pair_value(&self.array_counts)
    }

    #[must_use]
    pub fn array_spacing_values(&self) -> Option<[DxfDouble; 2]> {
        pair_value(&self.array_spacing)
    }

    #[must_use]
    pub fn attributes_follow_value(&self) -> Option<i16> {
        self.attributes_follow.value().copied()
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.extrusion)
    }
}

/// Immutable lazy INSERT semantics retaining all M10.1h cards.
#[derive(Debug)]
pub struct DxfInsertRecordSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfInsertRecordCardDirectory,
}

impl DxfInsertRecordSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.insert_record_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfInsertRecordCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInsertRecordValueEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfInsertRecordSemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        insert_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfInsertRecordValueEntry,
    ) -> Result<Option<DxfInsertRecordSemantics>, DxfError> {
        if self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal())
            != Some(record)
        {
            return Ok(None);
        }
        insert_semantics(&self.cards, record).map(Some)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_record_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordSemanticDirectory, DxfError> {
        DxfInsertRecordSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_record_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_record_semantic_directory(cancellation)
    }
}

fn insert_semantics(
    cards: &DxfInsertRecordCardDirectory,
    record: DxfInsertRecordValueEntry,
) -> Result<DxfInsertRecordSemantics, DxfError> {
    use DxfInsertRecordValueRole::{
        AttributesFollow, BlockName, ColumnCount, ColumnSpacing, ExtrusionX, ExtrusionY,
        ExtrusionZ, InsertionPointX, InsertionPointY, InsertionPointZ, RotationAngle, RowCount,
        RowSpacing, ScaleFactorX, ScaleFactorY, ScaleFactorZ,
    };
    Ok(DxfInsertRecordSemantics {
        record,
        block_name: text_semantic(cards, record, BlockName, "block_name")?,
        insertion_point: [
            double_semantic(cards, record, InsertionPointX, "insertion_point_x", None)?,
            double_semantic(cards, record, InsertionPointY, "insertion_point_y", None)?,
            double_semantic(cards, record, InsertionPointZ, "insertion_point_z", None)?,
        ],
        scale_factors: [
            double_semantic(cards, record, ScaleFactorX, "scale_factor_x", Some(ONE))?,
            double_semantic(cards, record, ScaleFactorY, "scale_factor_y", Some(ONE))?,
            double_semantic(cards, record, ScaleFactorZ, "scale_factor_z", Some(ONE))?,
        ],
        rotation_degrees: double_semantic(
            cards,
            record,
            RotationAngle,
            "rotation_degrees",
            Some(ZERO),
        )?,
        array_counts: [
            integer_semantic(cards, record, ColumnCount, "column_count", 1)?,
            integer_semantic(cards, record, RowCount, "row_count", 1)?,
        ],
        array_spacing: [
            double_semantic(cards, record, ColumnSpacing, "column_spacing", Some(ZERO))?,
            double_semantic(cards, record, RowSpacing, "row_spacing", Some(ZERO))?,
        ],
        attributes_follow: integer_semantic(
            cards,
            record,
            AttributesFollow,
            "attributes_follow",
            0,
        )?,
        extrusion: [
            double_semantic(cards, record, ExtrusionX, "extrusion_x", Some(ZERO))?,
            double_semantic(cards, record, ExtrusionY, "extrusion_y", Some(ZERO))?,
            double_semantic(cards, record, ExtrusionZ, "extrusion_z", Some(ONE))?,
        ],
    })
}

fn text_semantic(
    cards: &DxfInsertRecordCardDirectory,
    record: DxfInsertRecordValueEntry,
    role: DxfInsertRecordValueRole,
    field_id: &'static str,
) -> Result<DxfInsertRecordSemanticText, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfInsertRecordValueCardState::Absent => Ok(DxfSemanticValue::invalid(
            DxfInsertRecordSemanticIssue::MissingRequiredValue,
            field,
            None,
        )),
        DxfInsertRecordValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            match value.value() {
                Ok(DxfInsertRecordValueData::Text(text)) => {
                    Ok(DxfSemanticValue::explicit(text, field, raw))
                }
                Ok(DxfInsertRecordValueData::Double(_) | DxfInsertRecordValueData::Int16(_))
                | Err(_) => Err(invalid_internal_data()),
            }
        }
        DxfInsertRecordValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn double_semantic(
    cards: &DxfInsertRecordCardDirectory,
    record: DxfInsertRecordValueEntry,
    role: DxfInsertRecordValueRole,
    field_id: &'static str,
    default: Option<DxfDouble>,
) -> Result<DxfInsertRecordSemanticDouble, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfInsertRecordValueCardState::Absent => Ok(match default {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfInsertRecordSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
        }),
        DxfInsertRecordValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfInsertRecordValueData::Double(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfInsertRecordValueData::Text(_) | DxfInsertRecordValueData::Int16(_)) => {
                    return Err(invalid_internal_data());
                }
                Err(DxfInsertRecordValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfInsertRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfInsertRecordValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn integer_semantic(
    cards: &DxfInsertRecordCardDirectory,
    record: DxfInsertRecordValueEntry,
    role: DxfInsertRecordValueRole,
    field_id: &'static str,
    default: i16,
) -> Result<DxfInsertRecordSemanticInteger, DxfError> {
    let field = field(cards, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfInsertRecordValueCardState::Absent => Ok(DxfSemanticValue::defaulted(default, field)),
        DxfInsertRecordValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfInsertRecordValueData::Int16(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfInsertRecordValueData::Text(_) | DxfInsertRecordValueData::Double(_)) => {
                    return Err(invalid_internal_data());
                }
                Err(DxfInsertRecordValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfInsertRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfInsertRecordValueCardState::Multiple { occurrence_count } => {
            multiple_semantic(cards, card, field, occurrence_count)
        }
    }
}

fn multiple_semantic<T>(
    cards: &DxfInsertRecordCardDirectory,
    card: DxfInsertRecordValueCard,
    field: DxfSemanticFieldProvenance,
    occurrence_count: u32,
) -> Result<DxfSemanticValue<T, DxfInsertRecordSemanticIssue>, DxfError> {
    let first = cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)?;
    let value = value_for_member(cards, first)?;
    Ok(DxfSemanticValue::invalid(
        DxfInsertRecordSemanticIssue::MultipleValues { occurrence_count },
        field,
        Some(value_provenance(value)?),
    ))
}

fn field(
    cards: &DxfInsertRecordCardDirectory,
    field_id: &'static str,
) -> DxfSemanticFieldProvenance {
    DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id)
}

fn card_for_role(
    cards: &DxfInsertRecordCardDirectory,
    record: DxfInsertRecordValueEntry,
    role: DxfInsertRecordValueRole,
) -> Result<DxfInsertRecordValueCard, DxfError> {
    cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)
}

fn unique_value(
    cards: &DxfInsertRecordCardDirectory,
    card: DxfInsertRecordValueCard,
) -> Result<DxfInsertRecordValue, DxfError> {
    let [member] = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    value_for_member(cards, *member)
}

fn value_for_member(
    cards: &DxfInsertRecordCardDirectory,
    member: DxfInsertRecordCardMember,
) -> Result<DxfInsertRecordValue, DxfError> {
    cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(value: DxfInsertRecordValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn triple_value<T: Copy, I>(values: &[DxfSemanticValue<T, I>; 3]) -> Option<[T; 3]> {
    Some([
        values[0].value().copied()?,
        values[1].value().copied()?,
        values[2].value().copied()?,
    ])
}

fn pair_value<T: Copy, I>(values: &[DxfSemanticValue<T, I>; 2]) -> Option<[T; 2]> {
    Some([values[0].value().copied()?, values[1].value().copied()?])
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

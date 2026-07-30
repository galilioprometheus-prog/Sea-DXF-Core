//! Lazy typed double semantics for classic ATTRIB records.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfInsertAttributeCardDirectory, DxfInsertAttributeValueCard,
    DxfInsertAttributeValueCardState, DxfInsertAttributeValueData, DxfInsertAttributeValueEntry,
    DxfInsertAttributeValueIssue, DxfInsertAttributeValueRole, DxfIoOperation, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "insert.attribute.double";
const ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const ONE: DxfDouble = DxfDouble::from_bits(1.0_f64.to_bits());

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeDoubleSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

pub type DxfInsertAttributeSemanticDouble =
    DxfSemanticValue<DxfDouble, DxfInsertAttributeDoubleSemanticIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeDoubleSemantics {
    record: DxfInsertAttributeValueEntry,
    thickness: DxfInsertAttributeSemanticDouble,
    text_start: [DxfInsertAttributeSemanticDouble; 3],
    text_height: DxfInsertAttributeSemanticDouble,
    rotation_degrees: DxfInsertAttributeSemanticDouble,
    relative_x_scale: DxfInsertAttributeSemanticDouble,
    oblique_degrees: DxfInsertAttributeSemanticDouble,
    alignment_point: [DxfInsertAttributeSemanticDouble; 3],
    extrusion: [DxfInsertAttributeSemanticDouble; 3],
}

impl DxfInsertAttributeDoubleSemantics {
    #[must_use]
    pub const fn record(self) -> DxfInsertAttributeValueEntry {
        self.record
    }

    #[must_use]
    pub const fn thickness(&self) -> &DxfInsertAttributeSemanticDouble {
        &self.thickness
    }

    #[must_use]
    pub const fn text_start(&self) -> &[DxfInsertAttributeSemanticDouble; 3] {
        &self.text_start
    }

    #[must_use]
    pub const fn text_height(&self) -> &DxfInsertAttributeSemanticDouble {
        &self.text_height
    }

    #[must_use]
    pub const fn rotation_degrees(&self) -> &DxfInsertAttributeSemanticDouble {
        &self.rotation_degrees
    }

    #[must_use]
    pub const fn relative_x_scale(&self) -> &DxfInsertAttributeSemanticDouble {
        &self.relative_x_scale
    }

    #[must_use]
    pub const fn oblique_degrees(&self) -> &DxfInsertAttributeSemanticDouble {
        &self.oblique_degrees
    }

    #[must_use]
    pub const fn alignment_point(&self) -> &[DxfInsertAttributeSemanticDouble; 3] {
        &self.alignment_point
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfInsertAttributeSemanticDouble; 3] {
        &self.extrusion
    }

    #[must_use]
    pub fn text_start_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.text_start)
    }

    #[must_use]
    pub fn text_height_value(&self) -> Option<DxfDouble> {
        self.text_height.value().copied()
    }

    #[must_use]
    pub fn alignment_point_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.alignment_point)
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.extrusion)
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributeDoubleSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfInsertAttributeCardDirectory,
}

impl DxfInsertAttributeDoubleSemanticDirectory {
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
    ) -> Result<Option<DxfInsertAttributeDoubleSemantics>, DxfError> {
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
    ) -> Result<Option<DxfInsertAttributeDoubleSemantics>, DxfError> {
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
    ) -> Result<Option<DxfInsertAttributeDoubleSemantics>, DxfError> {
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
    pub fn insert_attribute_double_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeDoubleSemanticDirectory, DxfError> {
        DxfInsertAttributeDoubleSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_double_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeDoubleSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_double_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_double_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeDoubleSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_double_semantic_directory(cancellation)
    }
}

fn attribute_semantics(
    cards: &DxfInsertAttributeCardDirectory,
    record: DxfInsertAttributeValueEntry,
) -> Result<DxfInsertAttributeDoubleSemantics, DxfError> {
    use DxfInsertAttributeValueRole::{
        AlignmentPointX, AlignmentPointY, AlignmentPointZ, ExtrusionX, ExtrusionY, ExtrusionZ,
        ObliqueAngle, RelativeXScale, RotationAngle, TextHeight, TextStartX, TextStartY,
        TextStartZ, Thickness,
    };
    let raw = record.record().ordinal();
    Ok(DxfInsertAttributeDoubleSemantics {
        record,
        thickness: double_semantic(cards, raw, Thickness, "thickness", Absence::Default(ZERO))?,
        text_start: [
            double_semantic(cards, raw, TextStartX, "text_start_x", Absence::Required)?,
            double_semantic(cards, raw, TextStartY, "text_start_y", Absence::Required)?,
            double_semantic(cards, raw, TextStartZ, "text_start_z", Absence::Required)?,
        ],
        text_height: double_semantic(cards, raw, TextHeight, "text_height", Absence::Required)?,
        rotation_degrees: double_semantic(
            cards,
            raw,
            RotationAngle,
            "rotation_degrees",
            Absence::Default(ZERO),
        )?,
        relative_x_scale: double_semantic(
            cards,
            raw,
            RelativeXScale,
            "relative_x_scale",
            Absence::Default(ONE),
        )?,
        oblique_degrees: double_semantic(
            cards,
            raw,
            ObliqueAngle,
            "oblique_degrees",
            Absence::Default(ZERO),
        )?,
        alignment_point: [
            double_semantic(
                cards,
                raw,
                AlignmentPointX,
                "alignment_point_x",
                Absence::Optional,
            )?,
            double_semantic(
                cards,
                raw,
                AlignmentPointY,
                "alignment_point_y",
                Absence::Optional,
            )?,
            double_semantic(
                cards,
                raw,
                AlignmentPointZ,
                "alignment_point_z",
                Absence::Optional,
            )?,
        ],
        extrusion: [
            double_semantic(
                cards,
                raw,
                ExtrusionX,
                "extrusion_x",
                Absence::Default(ZERO),
            )?,
            double_semantic(
                cards,
                raw,
                ExtrusionY,
                "extrusion_y",
                Absence::Default(ZERO),
            )?,
            double_semantic(cards, raw, ExtrusionZ, "extrusion_z", Absence::Default(ONE))?,
        ],
    })
}

fn double_semantic(
    cards: &DxfInsertAttributeCardDirectory,
    raw_record_ordinal: u64,
    role: DxfInsertAttributeValueRole,
    field_id: &'static str,
    absence: Absence,
) -> Result<DxfInsertAttributeSemanticDouble, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = cards
        .card_for_role(raw_record_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfInsertAttributeValueCardState::Absent => Ok(match absence {
            Absence::Required => DxfSemanticValue::invalid(
                DxfInsertAttributeDoubleSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
            Absence::Default(value) => DxfSemanticValue::defaulted(value, field),
            Absence::Optional => DxfSemanticValue::absent(field),
        }),
        DxfInsertAttributeValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfInsertAttributeValueData::Double(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(
                    DxfInsertAttributeValueData::Text(_) | DxfInsertAttributeValueData::Int16(_),
                ) => return Err(invalid_internal_data()),
                Err(DxfInsertAttributeValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfInsertAttributeDoubleSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfInsertAttributeValueCardState::Multiple { occurrence_count } => {
            let first = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfInsertAttributeDoubleSemanticIssue::MultipleValues { occurrence_count },
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
    let members = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
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

fn triple_value(values: &[DxfInsertAttributeSemanticDouble; 3]) -> Option<[DxfDouble; 3]> {
    Some([
        values[0].value().copied()?,
        values[1].value().copied()?,
        values[2].value().copied()?,
    ])
}

#[derive(Clone, Copy)]
enum Absence {
    Required,
    Default(DxfDouble),
    Optional,
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
        &std::io::Error::from(std::io::ErrorKind::InvalidData),
    )
}

//! Lazy typed double semantics for classic ATTDEF records.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockAttributeDefinitionCardDirectory, DxfBlockAttributeDefinitionValue,
    DxfBlockAttributeDefinitionValueCard, DxfBlockAttributeDefinitionValueCardState,
    DxfBlockAttributeDefinitionValueData, DxfBlockAttributeDefinitionValueEntry,
    DxfBlockAttributeDefinitionValueIssue, DxfBlockAttributeDefinitionValueRole,
    DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "block.attribute_definition.double";
const ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const ONE: DxfDouble = DxfDouble::from_bits(1.0_f64.to_bits());

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionDoubleSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

pub type DxfBlockAttributeDefinitionSemanticDouble =
    DxfSemanticValue<DxfDouble, DxfBlockAttributeDefinitionDoubleSemanticIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionDoubleSemantics {
    record: DxfBlockAttributeDefinitionValueEntry,
    thickness: DxfBlockAttributeDefinitionSemanticDouble,
    text_start: [DxfBlockAttributeDefinitionSemanticDouble; 3],
    text_height: DxfBlockAttributeDefinitionSemanticDouble,
    rotation_degrees: DxfBlockAttributeDefinitionSemanticDouble,
    relative_x_scale: DxfBlockAttributeDefinitionSemanticDouble,
    oblique_degrees: DxfBlockAttributeDefinitionSemanticDouble,
    alignment_point: [DxfBlockAttributeDefinitionSemanticDouble; 3],
    extrusion: [DxfBlockAttributeDefinitionSemanticDouble; 3],
}

impl DxfBlockAttributeDefinitionDoubleSemantics {
    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.record
    }

    #[must_use]
    pub const fn thickness(&self) -> &DxfBlockAttributeDefinitionSemanticDouble {
        &self.thickness
    }

    #[must_use]
    pub const fn text_start(&self) -> &[DxfBlockAttributeDefinitionSemanticDouble; 3] {
        &self.text_start
    }

    #[must_use]
    pub const fn text_height(&self) -> &DxfBlockAttributeDefinitionSemanticDouble {
        &self.text_height
    }

    #[must_use]
    pub const fn rotation_degrees(&self) -> &DxfBlockAttributeDefinitionSemanticDouble {
        &self.rotation_degrees
    }

    #[must_use]
    pub const fn relative_x_scale(&self) -> &DxfBlockAttributeDefinitionSemanticDouble {
        &self.relative_x_scale
    }

    #[must_use]
    pub const fn oblique_degrees(&self) -> &DxfBlockAttributeDefinitionSemanticDouble {
        &self.oblique_degrees
    }

    #[must_use]
    pub const fn alignment_point(&self) -> &[DxfBlockAttributeDefinitionSemanticDouble; 3] {
        &self.alignment_point
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfBlockAttributeDefinitionSemanticDouble; 3] {
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
pub struct DxfBlockAttributeDefinitionDoubleSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfBlockAttributeDefinitionCardDirectory,
}

impl DxfBlockAttributeDefinitionDoubleSemanticDirectory {
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
    ) -> Result<Option<DxfBlockAttributeDefinitionDoubleSemantics>, DxfError> {
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
    ) -> Result<Option<DxfBlockAttributeDefinitionDoubleSemantics>, DxfError> {
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
    ) -> Result<Option<DxfBlockAttributeDefinitionDoubleSemantics>, DxfError> {
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
    pub fn block_attribute_definition_double_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionDoubleSemanticDirectory, DxfError> {
        DxfBlockAttributeDefinitionDoubleSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_double_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionDoubleSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_double_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_double_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionDoubleSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_double_semantic_directory(cancellation)
    }
}

fn attribute_definition_semantics(
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    record: DxfBlockAttributeDefinitionValueEntry,
) -> Result<DxfBlockAttributeDefinitionDoubleSemantics, DxfError> {
    use DxfBlockAttributeDefinitionValueRole::{
        AlignmentPointX, AlignmentPointY, AlignmentPointZ, ExtrusionX, ExtrusionY, ExtrusionZ,
        ObliqueAngle, RelativeXScale, RotationAngle, TextHeight, TextStartX, TextStartY,
        TextStartZ, Thickness,
    };
    let raw = record.definition().record().ordinal();
    Ok(DxfBlockAttributeDefinitionDoubleSemantics {
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
    cards: &DxfBlockAttributeDefinitionCardDirectory,
    raw_record_ordinal: u64,
    role: DxfBlockAttributeDefinitionValueRole,
    field_id: &'static str,
    absence: Absence,
) -> Result<DxfBlockAttributeDefinitionSemanticDouble, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = cards
        .card_for_role(raw_record_ordinal, role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfBlockAttributeDefinitionValueCardState::Absent => Ok(match absence {
            Absence::Required => DxfSemanticValue::invalid(
                DxfBlockAttributeDefinitionDoubleSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
            Absence::Default(value) => DxfSemanticValue::defaulted(value, field),
            Absence::Optional => DxfSemanticValue::absent(field),
        }),
        DxfBlockAttributeDefinitionValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfBlockAttributeDefinitionValueData::Double(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(
                    DxfBlockAttributeDefinitionValueData::Text(_)
                    | DxfBlockAttributeDefinitionValueData::Int16(_),
                ) => return Err(invalid_internal_data()),
                Err(DxfBlockAttributeDefinitionValueIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfBlockAttributeDefinitionDoubleSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfBlockAttributeDefinitionValueCardState::Multiple { occurrence_count } => {
            let first = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfBlockAttributeDefinitionDoubleSemanticIssue::MultipleValues { occurrence_count },
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

fn triple_value(values: &[DxfBlockAttributeDefinitionSemanticDouble; 3]) -> Option<[DxfDouble; 3]> {
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

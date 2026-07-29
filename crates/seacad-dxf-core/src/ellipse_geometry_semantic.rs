//! Lazy typed ELLIPSE semantics over cardinality cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfEllipseGeometryCardDirectory, DxfEllipseGeometryNumericIssue,
    DxfEllipseGeometryRecordEntry, DxfEllipseGeometryValue, DxfEllipseGeometryValueCard,
    DxfEllipseGeometryValueCardState, DxfEllipseGeometryValueRole, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfSourceId,
};

const ELLIPSE_NAMESPACE: &str = "ellipse_geometry.ellipse";
const DEFAULT_EXTRUSION: [DxfDouble; 3] = [
    DxfDouble::from_bits(0.0_f64.to_bits()),
    DxfDouble::from_bits(0.0_f64.to_bits()),
    DxfDouble::from_bits(1.0_f64.to_bits()),
];

/// Why one reviewed ELLIPSE value has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEllipseGeometrySemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored semantic state for one ELLIPSE double value.
pub type DxfEllipseGeometrySemanticValue =
    DxfSemanticValue<DxfDouble, DxfEllipseGeometrySemanticIssue>;

/// Lazy reviewed semantics for one exact ELLIPSE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEllipseGeometrySemantics {
    record: DxfEllipseGeometryRecordEntry,
    center: [DxfEllipseGeometrySemanticValue; 3],
    major_axis_endpoint_relative_to_center: [DxfEllipseGeometrySemanticValue; 3],
    minor_to_major_axis_ratio: DxfEllipseGeometrySemanticValue,
    start_parameter: DxfEllipseGeometrySemanticValue,
    end_parameter: DxfEllipseGeometrySemanticValue,
    extrusion: [DxfEllipseGeometrySemanticValue; 3],
}

impl DxfEllipseGeometrySemantics {
    #[must_use]
    pub const fn record(self) -> DxfEllipseGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn center(&self) -> &[DxfEllipseGeometrySemanticValue; 3] {
        &self.center
    }

    #[must_use]
    pub const fn major_axis_endpoint_relative_to_center(
        &self,
    ) -> &[DxfEllipseGeometrySemanticValue; 3] {
        &self.major_axis_endpoint_relative_to_center
    }

    #[must_use]
    pub const fn minor_to_major_axis_ratio(&self) -> &DxfEllipseGeometrySemanticValue {
        &self.minor_to_major_axis_ratio
    }

    #[must_use]
    pub const fn start_parameter(&self) -> &DxfEllipseGeometrySemanticValue {
        &self.start_parameter
    }

    #[must_use]
    pub const fn end_parameter(&self) -> &DxfEllipseGeometrySemanticValue {
        &self.end_parameter
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfEllipseGeometrySemanticValue; 3] {
        &self.extrusion
    }

    #[must_use]
    pub fn center_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.center)
    }

    #[must_use]
    pub fn major_axis_endpoint_relative_to_center_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.major_axis_endpoint_relative_to_center)
    }

    #[must_use]
    pub fn minor_to_major_axis_ratio_value(&self) -> Option<DxfDouble> {
        self.minor_to_major_axis_ratio.value().copied()
    }

    #[must_use]
    pub fn start_parameter_value(&self) -> Option<DxfDouble> {
        self.start_parameter.value().copied()
    }

    #[must_use]
    pub fn end_parameter_value(&self) -> Option<DxfDouble> {
        self.end_parameter.value().copied()
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.extrusion)
    }
}

/// Immutable lazy ELLIPSE semantics with retained card and raw evidence.
#[derive(Debug)]
pub struct DxfEllipseGeometrySemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfEllipseGeometryCardDirectory,
}

impl DxfEllipseGeometrySemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.ellipse_geometry_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfEllipseGeometryCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfEllipseGeometryRecordEntry] {
        self.cards.evidence_directory().records()
    }

    #[must_use]
    pub fn record(&self, ordinal: u64) -> Option<DxfEllipseGeometryRecordEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.records().get(index).copied()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfEllipseGeometryRecordEntry,
    ) -> Result<Option<DxfEllipseGeometrySemantics>, DxfError> {
        let known = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal());
        if known != Some(record) {
            return Ok(None);
        }
        ellipse_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfEllipseGeometrySemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        self.semantics_for_record(record)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn ellipse_geometry_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEllipseGeometrySemanticDirectory, DxfError> {
        DxfEllipseGeometrySemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn ellipse_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEllipseGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).ellipse_geometry_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn ellipse_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEllipseGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).ellipse_geometry_semantic_directory(cancellation)
    }
}

fn ellipse_semantics(
    cards: &DxfEllipseGeometryCardDirectory,
    record: DxfEllipseGeometryRecordEntry,
) -> Result<DxfEllipseGeometrySemantics, DxfError> {
    Ok(DxfEllipseGeometrySemantics {
        record,
        center: required_triple(
            cards,
            record,
            ["center_x", "center_y", "center_z"],
            [
                DxfEllipseGeometryValueRole::WcsCenterX,
                DxfEllipseGeometryValueRole::WcsCenterY,
                DxfEllipseGeometryValueRole::WcsCenterZ,
            ],
        )?,
        major_axis_endpoint_relative_to_center: required_triple(
            cards,
            record,
            [
                "major_axis_endpoint_relative_x",
                "major_axis_endpoint_relative_y",
                "major_axis_endpoint_relative_z",
            ],
            [
                DxfEllipseGeometryValueRole::WcsMajorAxisEndpointX,
                DxfEllipseGeometryValueRole::WcsMajorAxisEndpointY,
                DxfEllipseGeometryValueRole::WcsMajorAxisEndpointZ,
            ],
        )?,
        minor_to_major_axis_ratio: semantic_value(
            cards,
            record,
            DxfEllipseGeometryValueRole::MinorToMajorAxisRatio,
            "minor_to_major_axis_ratio",
            None,
        )?,
        start_parameter: semantic_value(
            cards,
            record,
            DxfEllipseGeometryValueRole::StartParameter,
            "start_parameter",
            None,
        )?,
        end_parameter: semantic_value(
            cards,
            record,
            DxfEllipseGeometryValueRole::EndParameter,
            "end_parameter",
            None,
        )?,
        extrusion: extrusion_triple(cards, record)?,
    })
}

fn required_triple(
    cards: &DxfEllipseGeometryCardDirectory,
    record: DxfEllipseGeometryRecordEntry,
    field_ids: [&'static str; 3],
    roles: [DxfEllipseGeometryValueRole; 3],
) -> Result<[DxfEllipseGeometrySemanticValue; 3], DxfError> {
    Ok([
        semantic_value(cards, record, roles[0], field_ids[0], None)?,
        semantic_value(cards, record, roles[1], field_ids[1], None)?,
        semantic_value(cards, record, roles[2], field_ids[2], None)?,
    ])
}

fn extrusion_triple(
    cards: &DxfEllipseGeometryCardDirectory,
    record: DxfEllipseGeometryRecordEntry,
) -> Result<[DxfEllipseGeometrySemanticValue; 3], DxfError> {
    Ok([
        semantic_value(
            cards,
            record,
            DxfEllipseGeometryValueRole::ExtrusionX,
            "extrusion_x",
            Some(DEFAULT_EXTRUSION[0]),
        )?,
        semantic_value(
            cards,
            record,
            DxfEllipseGeometryValueRole::ExtrusionY,
            "extrusion_y",
            Some(DEFAULT_EXTRUSION[1]),
        )?,
        semantic_value(
            cards,
            record,
            DxfEllipseGeometryValueRole::ExtrusionZ,
            "extrusion_z",
            Some(DEFAULT_EXTRUSION[2]),
        )?,
    ])
}

fn semantic_value(
    cards: &DxfEllipseGeometryCardDirectory,
    record: DxfEllipseGeometryRecordEntry,
    role: DxfEllipseGeometryValueRole,
    field_id: &'static str,
    default: Option<DxfDouble>,
) -> Result<DxfEllipseGeometrySemanticValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), ELLIPSE_NAMESPACE, field_id);
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfEllipseGeometryValueCardState::Absent => Ok(match default {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfEllipseGeometrySemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
        }),
        DxfEllipseGeometryValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(number) => DxfSemanticValue::explicit(number, field, raw),
                Err(DxfEllipseGeometryNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfEllipseGeometrySemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfEllipseGeometryValueCardState::Multiple { occurrence_count } => {
            let primary = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfEllipseGeometrySemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(primary)?),
            ))
        }
    }
}

fn unique_value(
    cards: &DxfEllipseGeometryCardDirectory,
    card: DxfEllipseGeometryValueCard,
) -> Result<DxfEllipseGeometryValue, DxfError> {
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
    cards: &DxfEllipseGeometryCardDirectory,
    card: DxfEllipseGeometryValueCard,
) -> Result<DxfEllipseGeometryValue, DxfError> {
    let member = cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)?;
    cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(value: DxfEllipseGeometryValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn triple_value(values: &[DxfEllipseGeometrySemanticValue; 3]) -> Option<[DxfDouble; 3]> {
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

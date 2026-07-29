//! Lazy typed LWPOLYLINE record semantics over M9.1e cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfLightweightPolylineInteger,
    DxfLightweightPolylineIntegerIssue, DxfLightweightPolylineNumericIssue,
    DxfLightweightPolylineRecordCard, DxfLightweightPolylineRecordCardDirectory,
    DxfLightweightPolylineRecordCardEntry, DxfLightweightPolylineRecordCardState,
    DxfLightweightPolylineRecordRole, DxfLightweightPolylineValueRole, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "lightweight_polyline.record";
const DEFAULT_ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const DEFAULT_EXTRUSION: [DxfDouble; 3] = [
    DEFAULT_ZERO,
    DEFAULT_ZERO,
    DxfDouble::from_bits(1.0_f64.to_bits()),
];

/// Why one reviewed LWPOLYLINE record field has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineRecordSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored state for one LWPOLYLINE record double.
pub type DxfLightweightPolylineRecordSemanticDouble =
    DxfSemanticValue<DxfDouble, DxfLightweightPolylineRecordSemanticIssue>;

/// Source-anchored state for one exact signed LWPOLYLINE record integer.
pub type DxfLightweightPolylineRecordSemanticInteger =
    DxfSemanticValue<DxfLightweightPolylineInteger, DxfLightweightPolylineRecordSemanticIssue>;

/// Comparison between declared group-90 count and retained group-10 anchors.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineVertexCountComparison {
    NotComparable,
    Matched { count: u32 },
    Mismatched { declared: i32, observed: u32 },
}

/// Explicit constant/variable width occurrence shape without precedence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfLightweightPolylineWidthEvidenceState {
    NoExplicitWidth,
    ConstantOnly {
        constant_occurrence_count: u32,
    },
    VariableOnly {
        variable_occurrence_count: u32,
    },
    ConstantAndVariable {
        constant_occurrence_count: u32,
        variable_occurrence_count: u32,
    },
}

/// Lazy reviewed semantics for one exact LWPOLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLightweightPolylineRecordSemantics {
    record: DxfLightweightPolylineRecordCardEntry,
    vertex_count: DxfLightweightPolylineRecordSemanticInteger,
    observed_vertex_count: u32,
    vertex_count_comparison: DxfLightweightPolylineVertexCountComparison,
    flags: DxfLightweightPolylineRecordSemanticInteger,
    ocs_elevation: DxfLightweightPolylineRecordSemanticDouble,
    thickness: DxfLightweightPolylineRecordSemanticDouble,
    constant_width: DxfLightweightPolylineRecordSemanticDouble,
    extrusion: [DxfLightweightPolylineRecordSemanticDouble; 3],
    width_evidence: DxfLightweightPolylineWidthEvidenceState,
}

impl DxfLightweightPolylineRecordSemantics {
    #[must_use]
    pub const fn record(self) -> DxfLightweightPolylineRecordCardEntry {
        self.record
    }

    #[must_use]
    pub const fn vertex_count(&self) -> &DxfLightweightPolylineRecordSemanticInteger {
        &self.vertex_count
    }

    #[must_use]
    pub const fn observed_vertex_count(self) -> u32 {
        self.observed_vertex_count
    }

    #[must_use]
    pub const fn vertex_count_comparison(self) -> DxfLightweightPolylineVertexCountComparison {
        self.vertex_count_comparison
    }

    #[must_use]
    pub const fn flags(&self) -> &DxfLightweightPolylineRecordSemanticInteger {
        &self.flags
    }

    #[must_use]
    pub const fn ocs_elevation(&self) -> &DxfLightweightPolylineRecordSemanticDouble {
        &self.ocs_elevation
    }

    #[must_use]
    pub const fn thickness(&self) -> &DxfLightweightPolylineRecordSemanticDouble {
        &self.thickness
    }

    #[must_use]
    pub const fn constant_width(&self) -> &DxfLightweightPolylineRecordSemanticDouble {
        &self.constant_width
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfLightweightPolylineRecordSemanticDouble; 3] {
        &self.extrusion
    }

    #[must_use]
    pub const fn width_evidence(self) -> DxfLightweightPolylineWidthEvidenceState {
        self.width_evidence
    }

    #[must_use]
    pub fn vertex_count_value(&self) -> Option<i32> {
        match self.vertex_count.value()? {
            DxfLightweightPolylineInteger::I32(value) => Some(*value),
            DxfLightweightPolylineInteger::I16(_) => None,
        }
    }

    #[must_use]
    pub fn flags_value(&self) -> Option<i16> {
        match self.flags.value()? {
            DxfLightweightPolylineInteger::I16(value) => Some(*value),
            DxfLightweightPolylineInteger::I32(_) => None,
        }
    }

    #[must_use]
    pub fn is_closed(&self) -> Option<bool> {
        Some(self.flags_value()? & 1 != 0)
    }

    #[must_use]
    pub fn has_plinegen(&self) -> Option<bool> {
        Some(self.flags_value()? & 128 != 0)
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        Some([
            self.extrusion[0].value().copied()?,
            self.extrusion[1].value().copied()?,
            self.extrusion[2].value().copied()?,
        ])
    }
}

/// Immutable lazy LWPOLYLINE record semantics retaining all M9.1e evidence.
#[derive(Debug)]
pub struct DxfLightweightPolylineRecordSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfLightweightPolylineRecordCardDirectory,
}

impl DxfLightweightPolylineRecordSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.lightweight_polyline_record_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfLightweightPolylineRecordCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfLightweightPolylineRecordCardEntry] {
        self.cards.records()
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfLightweightPolylineRecordSemantics>, DxfError> {
        let Some(record) = self.cards.record_for_raw_ordinal(raw_record_ordinal) else {
            return Ok(None);
        };
        record_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfLightweightPolylineRecordCardEntry,
    ) -> Result<Option<DxfLightweightPolylineRecordSemantics>, DxfError> {
        if self.cards.record_for_raw_ordinal(record.record().ordinal()) != Some(record) {
            return Ok(None);
        }
        record_semantics(&self.cards, record).map(Some)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn lightweight_polyline_record_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineRecordSemanticDirectory, DxfError> {
        DxfLightweightPolylineRecordSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn lightweight_polyline_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_record_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn lightweight_polyline_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfLightweightPolylineRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).lightweight_polyline_record_semantic_directory(cancellation)
    }
}

fn record_semantics(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
) -> Result<DxfLightweightPolylineRecordSemantics, DxfError> {
    let vertex_count = integer_semantic(
        cards,
        record,
        DxfLightweightPolylineRecordRole::VertexCount,
        "vertex_count",
        None,
    )?;
    let observed_vertex_count = observed_vertex_count(cards, record)?;
    Ok(DxfLightweightPolylineRecordSemantics {
        record,
        vertex_count,
        observed_vertex_count,
        vertex_count_comparison: compare_vertex_count(&vertex_count, observed_vertex_count),
        flags: integer_semantic(
            cards,
            record,
            DxfLightweightPolylineRecordRole::Flags,
            "flags",
            Some(DxfLightweightPolylineInteger::I16(0)),
        )?,
        ocs_elevation: double_semantic(
            cards,
            record,
            DxfLightweightPolylineRecordRole::OcsElevation,
            "ocs_elevation",
            DEFAULT_ZERO,
        )?,
        thickness: double_semantic(
            cards,
            record,
            DxfLightweightPolylineRecordRole::Thickness,
            "thickness",
            DEFAULT_ZERO,
        )?,
        constant_width: double_semantic(
            cards,
            record,
            DxfLightweightPolylineRecordRole::ConstantWidth,
            "constant_width",
            DEFAULT_ZERO,
        )?,
        extrusion: [
            double_semantic(
                cards,
                record,
                DxfLightweightPolylineRecordRole::ExtrusionX,
                "extrusion_x",
                DEFAULT_EXTRUSION[0],
            )?,
            double_semantic(
                cards,
                record,
                DxfLightweightPolylineRecordRole::ExtrusionY,
                "extrusion_y",
                DEFAULT_EXTRUSION[1],
            )?,
            double_semantic(
                cards,
                record,
                DxfLightweightPolylineRecordRole::ExtrusionZ,
                "extrusion_z",
                DEFAULT_EXTRUSION[2],
            )?,
        ],
        width_evidence: width_evidence(cards, record)?,
    })
}

fn integer_semantic(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
    role: DxfLightweightPolylineRecordRole,
    field_id: &'static str,
    default: Option<DxfLightweightPolylineInteger>,
) -> Result<DxfLightweightPolylineRecordSemanticInteger, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfLightweightPolylineRecordCardState::Absent => Ok(match default {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfLightweightPolylineRecordSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
        }),
        DxfLightweightPolylineRecordCardState::Unique => {
            let value = unique_integer(cards, card)?;
            let raw = raw_provenance(
                value.group().occurrence(),
                value.group().value_payload_span(),
            )?;
            Ok(match value.value() {
                Ok(number) => DxfSemanticValue::explicit(number, field, raw),
                Err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfLightweightPolylineRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfLightweightPolylineRecordCardState::Multiple { occurrence_count } => {
            let first = first_member(cards, card)?;
            let value = cards
                .integer_value_for_member(first)
                .ok_or_else(invalid_internal_data)?;
            Ok(DxfSemanticValue::invalid(
                DxfLightweightPolylineRecordSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(raw_provenance(
                    value.group().occurrence(),
                    value.group().value_payload_span(),
                )?),
            ))
        }
    }
}

fn double_semantic(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
    role: DxfLightweightPolylineRecordRole,
    field_id: &'static str,
    default: DxfDouble,
) -> Result<DxfLightweightPolylineRecordSemanticDouble, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfLightweightPolylineRecordCardState::Absent => {
            Ok(DxfSemanticValue::defaulted(default, field))
        }
        DxfLightweightPolylineRecordCardState::Unique => {
            let member = first_member(cards, card)?;
            let value = cards
                .floating_value_for_member(member)
                .ok_or_else(invalid_internal_data)?;
            let raw = raw_provenance(
                value.group().occurrence(),
                value.group().value_payload_span(),
            )?;
            Ok(match value.value() {
                Ok(number) => DxfSemanticValue::explicit(number, field, raw),
                Err(DxfLightweightPolylineNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfLightweightPolylineRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfLightweightPolylineRecordCardState::Multiple { occurrence_count } => {
            let member = first_member(cards, card)?;
            let value = cards
                .floating_value_for_member(member)
                .ok_or_else(invalid_internal_data)?;
            Ok(DxfSemanticValue::invalid(
                DxfLightweightPolylineRecordSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(raw_provenance(
                    value.group().occurrence(),
                    value.group().value_payload_span(),
                )?),
            ))
        }
    }
}

fn card_for_role(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
    role: DxfLightweightPolylineRecordRole,
) -> Result<DxfLightweightPolylineRecordCard, DxfError> {
    cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)
}

fn first_member(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    card: DxfLightweightPolylineRecordCard,
) -> Result<crate::DxfLightweightPolylineRecordCardMember, DxfError> {
    cards
        .members_for_card(card)
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)
}

fn unique_integer(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    card: DxfLightweightPolylineRecordCard,
) -> Result<crate::DxfLightweightPolylineIntegerValue, DxfError> {
    let members = cards
        .members_for_card(card)
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
        return Err(invalid_internal_data());
    };
    cards
        .integer_value_for_member(*member)
        .ok_or_else(invalid_internal_data)
}

fn observed_vertex_count(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
) -> Result<u32, DxfError> {
    let values = cards
        .floating_evidence_directory()
        .values_for_raw_record(record.record().ordinal())
        .ok_or_else(invalid_internal_data)?;
    u32::try_from(
        values
            .iter()
            .filter(|value| value.role() == DxfLightweightPolylineValueRole::OcsVertexX)
            .count(),
    )
    .map_err(|_| invalid_internal_data())
}

fn compare_vertex_count(
    declared: &DxfLightweightPolylineRecordSemanticInteger,
    observed: u32,
) -> DxfLightweightPolylineVertexCountComparison {
    let Some(DxfLightweightPolylineInteger::I32(declared)) = declared.value().copied() else {
        return DxfLightweightPolylineVertexCountComparison::NotComparable;
    };
    if i64::from(declared) == i64::from(observed) {
        DxfLightweightPolylineVertexCountComparison::Matched { count: observed }
    } else {
        DxfLightweightPolylineVertexCountComparison::Mismatched { declared, observed }
    }
}

fn width_evidence(
    cards: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
) -> Result<DxfLightweightPolylineWidthEvidenceState, DxfError> {
    let constant = card_for_role(
        cards,
        record,
        DxfLightweightPolylineRecordRole::ConstantWidth,
    )?;
    let constant =
        u32::try_from(constant.member_range().len()).map_err(|_| invalid_internal_data())?;
    let values = cards
        .floating_evidence_directory()
        .values_for_raw_record(record.record().ordinal())
        .ok_or_else(invalid_internal_data)?;
    let variable = u32::try_from(
        values
            .iter()
            .filter(|value| {
                matches!(
                    value.role(),
                    DxfLightweightPolylineValueRole::StartWidth
                        | DxfLightweightPolylineValueRole::EndWidth
                )
            })
            .count(),
    )
    .map_err(|_| invalid_internal_data())?;
    Ok(match (constant, variable) {
        (0, 0) => DxfLightweightPolylineWidthEvidenceState::NoExplicitWidth,
        (constant_occurrence_count, 0) => DxfLightweightPolylineWidthEvidenceState::ConstantOnly {
            constant_occurrence_count,
        },
        (0, variable_occurrence_count) => DxfLightweightPolylineWidthEvidenceState::VariableOnly {
            variable_occurrence_count,
        },
        (constant_occurrence_count, variable_occurrence_count) => {
            DxfLightweightPolylineWidthEvidenceState::ConstantAndVariable {
                constant_occurrence_count,
                variable_occurrence_count,
            }
        }
    })
}

fn raw_provenance(
    occurrence: u64,
    span: crate::ByteSpan,
) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(occurrence, span).ok_or_else(invalid_internal_data)
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

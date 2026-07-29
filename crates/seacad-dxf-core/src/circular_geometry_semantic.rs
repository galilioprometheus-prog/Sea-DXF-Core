//! Lazy typed CIRCLE and ARC semantics over cardinality cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfCircularGeometryCardDirectory, DxfCircularGeometryKind, DxfCircularGeometryNumericIssue,
    DxfCircularGeometryRecordEntry, DxfCircularGeometryValue, DxfCircularGeometryValueCard,
    DxfCircularGeometryValueCardState, DxfCircularGeometryValueRole, DxfDouble, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId,
};

const CIRCLE_NAMESPACE: &str = "circular_geometry.circle";
const ARC_NAMESPACE: &str = "circular_geometry.arc";
const DEFAULT_EXTRUSION: [DxfDouble; 3] = [
    DxfDouble::from_bits(0.0_f64.to_bits()),
    DxfDouble::from_bits(0.0_f64.to_bits()),
    DxfDouble::from_bits(1.0_f64.to_bits()),
];

/// Why one reviewed circular-geometry value has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCircularGeometrySemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored semantic state for one CIRCLE or ARC double value.
pub type DxfCircularGeometrySemanticValue =
    DxfSemanticValue<DxfDouble, DxfCircularGeometrySemanticIssue>;

/// Lazy reviewed semantics for one exact CIRCLE or ARC record.
///
/// ARC angle accessors return `Some` even when their semantic state is invalid;
/// CIRCLE returns `None` because those roles are not applicable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCircularGeometrySemantics {
    record: DxfCircularGeometryRecordEntry,
    center: [DxfCircularGeometrySemanticValue; 3],
    radius: DxfCircularGeometrySemanticValue,
    start_angle_degrees: Option<DxfCircularGeometrySemanticValue>,
    end_angle_degrees: Option<DxfCircularGeometrySemanticValue>,
    extrusion: [DxfCircularGeometrySemanticValue; 3],
}

impl DxfCircularGeometrySemantics {
    #[must_use]
    pub const fn record(self) -> DxfCircularGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfCircularGeometryKind {
        self.record.kind()
    }

    #[must_use]
    pub const fn center(&self) -> &[DxfCircularGeometrySemanticValue; 3] {
        &self.center
    }

    #[must_use]
    pub const fn radius(&self) -> &DxfCircularGeometrySemanticValue {
        &self.radius
    }

    #[must_use]
    pub const fn start_angle_degrees(&self) -> Option<&DxfCircularGeometrySemanticValue> {
        self.start_angle_degrees.as_ref()
    }

    #[must_use]
    pub const fn end_angle_degrees(&self) -> Option<&DxfCircularGeometrySemanticValue> {
        self.end_angle_degrees.as_ref()
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfCircularGeometrySemanticValue; 3] {
        &self.extrusion
    }

    #[must_use]
    pub fn center_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.center)
    }

    #[must_use]
    pub fn radius_value(&self) -> Option<DxfDouble> {
        self.radius.value().copied()
    }

    #[must_use]
    pub fn start_angle_degrees_value(&self) -> Option<DxfDouble> {
        self.start_angle_degrees()?.value().copied()
    }

    #[must_use]
    pub fn end_angle_degrees_value(&self) -> Option<DxfDouble> {
        self.end_angle_degrees()?.value().copied()
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.extrusion)
    }
}

/// Immutable lazy CIRCLE/ARC semantics with retained card and raw evidence.
#[derive(Debug)]
pub struct DxfCircularGeometrySemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfCircularGeometryCardDirectory,
}

impl DxfCircularGeometrySemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.circular_geometry_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfCircularGeometryCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfCircularGeometryRecordEntry] {
        self.cards.evidence_directory().records()
    }

    #[must_use]
    pub fn record(&self, ordinal: u64) -> Option<DxfCircularGeometryRecordEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.records().get(index).copied()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfCircularGeometryRecordEntry,
    ) -> Result<Option<DxfCircularGeometrySemantics>, DxfError> {
        let known = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal());
        if known != Some(record) {
            return Ok(None);
        }
        circular_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfCircularGeometrySemantics>, DxfError> {
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
    pub fn circular_geometry_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCircularGeometrySemanticDirectory, DxfError> {
        DxfCircularGeometrySemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn circular_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCircularGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).circular_geometry_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn circular_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfCircularGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).circular_geometry_semantic_directory(cancellation)
    }
}

fn circular_semantics(
    cards: &DxfCircularGeometryCardDirectory,
    record: DxfCircularGeometryRecordEntry,
) -> Result<DxfCircularGeometrySemantics, DxfError> {
    let namespace = match record.kind() {
        DxfCircularGeometryKind::Circle => CIRCLE_NAMESPACE,
        DxfCircularGeometryKind::Arc => ARC_NAMESPACE,
    };
    let center = required_triple(cards, record, namespace)?;
    let radius = semantic_value(
        cards,
        record,
        DxfCircularGeometryValueRole::Radius,
        namespace,
        "radius",
        None,
    )?;
    let (start_angle_degrees, end_angle_degrees) = match record.kind() {
        DxfCircularGeometryKind::Circle => (None, None),
        DxfCircularGeometryKind::Arc => (
            Some(semantic_value(
                cards,
                record,
                DxfCircularGeometryValueRole::StartAngle,
                namespace,
                "start_angle_degrees",
                None,
            )?),
            Some(semantic_value(
                cards,
                record,
                DxfCircularGeometryValueRole::EndAngle,
                namespace,
                "end_angle_degrees",
                None,
            )?),
        ),
    };
    Ok(DxfCircularGeometrySemantics {
        record,
        center,
        radius,
        start_angle_degrees,
        end_angle_degrees,
        extrusion: extrusion_triple(cards, record, namespace)?,
    })
}

fn required_triple(
    cards: &DxfCircularGeometryCardDirectory,
    record: DxfCircularGeometryRecordEntry,
    namespace: &'static str,
) -> Result<[DxfCircularGeometrySemanticValue; 3], DxfError> {
    Ok([
        semantic_value(
            cards,
            record,
            DxfCircularGeometryValueRole::OcsCenterX,
            namespace,
            "center_x",
            None,
        )?,
        semantic_value(
            cards,
            record,
            DxfCircularGeometryValueRole::OcsCenterY,
            namespace,
            "center_y",
            None,
        )?,
        semantic_value(
            cards,
            record,
            DxfCircularGeometryValueRole::OcsCenterZ,
            namespace,
            "center_z",
            None,
        )?,
    ])
}

fn extrusion_triple(
    cards: &DxfCircularGeometryCardDirectory,
    record: DxfCircularGeometryRecordEntry,
    namespace: &'static str,
) -> Result<[DxfCircularGeometrySemanticValue; 3], DxfError> {
    Ok([
        semantic_value(
            cards,
            record,
            DxfCircularGeometryValueRole::ExtrusionX,
            namespace,
            "extrusion_x",
            Some(DEFAULT_EXTRUSION[0]),
        )?,
        semantic_value(
            cards,
            record,
            DxfCircularGeometryValueRole::ExtrusionY,
            namespace,
            "extrusion_y",
            Some(DEFAULT_EXTRUSION[1]),
        )?,
        semantic_value(
            cards,
            record,
            DxfCircularGeometryValueRole::ExtrusionZ,
            namespace,
            "extrusion_z",
            Some(DEFAULT_EXTRUSION[2]),
        )?,
    ])
}

fn semantic_value(
    cards: &DxfCircularGeometryCardDirectory,
    record: DxfCircularGeometryRecordEntry,
    role: DxfCircularGeometryValueRole,
    namespace: &'static str,
    field_id: &'static str,
    default: Option<DxfDouble>,
) -> Result<DxfCircularGeometrySemanticValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), namespace, field_id);
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfCircularGeometryValueCardState::Absent => Ok(match default {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfCircularGeometrySemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
        }),
        DxfCircularGeometryValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(number) => DxfSemanticValue::explicit(number, field, raw),
                Err(DxfCircularGeometryNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfCircularGeometrySemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfCircularGeometryValueCardState::Multiple { occurrence_count } => {
            let primary = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfCircularGeometrySemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(primary)?),
            ))
        }
    }
}

fn unique_value(
    cards: &DxfCircularGeometryCardDirectory,
    card: DxfCircularGeometryValueCard,
) -> Result<DxfCircularGeometryValue, DxfError> {
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
    cards: &DxfCircularGeometryCardDirectory,
    card: DxfCircularGeometryValueCard,
) -> Result<DxfCircularGeometryValue, DxfError> {
    let member = cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)?;
    cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(value: DxfCircularGeometryValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn triple_value(values: &[DxfCircularGeometrySemanticValue; 3]) -> Option<[DxfDouble; 3]> {
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
    io_error(io::ErrorKind::InvalidData)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

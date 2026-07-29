//! Lazy typed RAY and XLINE semantics over cardinality cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfInfiniteLineGeometryCardDirectory, DxfInfiniteLineGeometryKind,
    DxfInfiniteLineGeometryNumericIssue, DxfInfiniteLineGeometryRecordEntry,
    DxfInfiniteLineGeometryValue, DxfInfiniteLineGeometryValueCard,
    DxfInfiniteLineGeometryValueCardState, DxfInfiniteLineGeometryValueRole, DxfIoOperation,
    DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfSourceId,
};

const RAY_NAMESPACE: &str = "infinite_line_geometry.ray";
const XLINE_NAMESPACE: &str = "infinite_line_geometry.xline";

/// Why one reviewed RAY/XLINE value has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInfiniteLineGeometrySemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored semantic state for one RAY/XLINE double value.
pub type DxfInfiniteLineGeometrySemanticValue =
    DxfSemanticValue<DxfDouble, DxfInfiniteLineGeometrySemanticIssue>;

/// Lazy reviewed semantics for one exact RAY or XLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInfiniteLineGeometrySemantics {
    record: DxfInfiniteLineGeometryRecordEntry,
    start_or_first_point: [DxfInfiniteLineGeometrySemanticValue; 3],
    unit_direction: [DxfInfiniteLineGeometrySemanticValue; 3],
}

impl DxfInfiniteLineGeometrySemantics {
    #[must_use]
    pub const fn record(self) -> DxfInfiniteLineGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfInfiniteLineGeometryKind {
        self.record.kind()
    }

    #[must_use]
    pub const fn start_or_first_point(&self) -> &[DxfInfiniteLineGeometrySemanticValue; 3] {
        &self.start_or_first_point
    }

    #[must_use]
    pub const fn unit_direction(&self) -> &[DxfInfiniteLineGeometrySemanticValue; 3] {
        &self.unit_direction
    }

    #[must_use]
    pub fn start_or_first_point_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.start_or_first_point)
    }

    #[must_use]
    pub fn unit_direction_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.unit_direction)
    }
}

/// Immutable lazy RAY/XLINE semantics with retained card and raw evidence.
#[derive(Debug)]
pub struct DxfInfiniteLineGeometrySemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfInfiniteLineGeometryCardDirectory,
}

impl DxfInfiniteLineGeometrySemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.infinite_line_geometry_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfInfiniteLineGeometryCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInfiniteLineGeometryRecordEntry] {
        self.cards.evidence_directory().records()
    }

    #[must_use]
    pub fn record(&self, ordinal: u64) -> Option<DxfInfiniteLineGeometryRecordEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.records().get(index).copied()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfInfiniteLineGeometryRecordEntry,
    ) -> Result<Option<DxfInfiniteLineGeometrySemantics>, DxfError> {
        let known = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal());
        if known != Some(record) {
            return Ok(None);
        }
        infinite_line_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfInfiniteLineGeometrySemantics>, DxfError> {
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
    pub fn infinite_line_geometry_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometrySemanticDirectory, DxfError> {
        DxfInfiniteLineGeometrySemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn infinite_line_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).infinite_line_geometry_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn infinite_line_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInfiniteLineGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).infinite_line_geometry_semantic_directory(cancellation)
    }
}

fn infinite_line_semantics(
    cards: &DxfInfiniteLineGeometryCardDirectory,
    record: DxfInfiniteLineGeometryRecordEntry,
) -> Result<DxfInfiniteLineGeometrySemantics, DxfError> {
    let namespace = match record.kind() {
        DxfInfiniteLineGeometryKind::Ray => RAY_NAMESPACE,
        DxfInfiniteLineGeometryKind::Xline => XLINE_NAMESPACE,
    };
    Ok(DxfInfiniteLineGeometrySemantics {
        record,
        start_or_first_point: required_triple(
            cards,
            record,
            namespace,
            [
                "start_or_first_point_x",
                "start_or_first_point_y",
                "start_or_first_point_z",
            ],
            [
                DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointX,
                DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointY,
                DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointZ,
            ],
        )?,
        unit_direction: required_triple(
            cards,
            record,
            namespace,
            ["unit_direction_x", "unit_direction_y", "unit_direction_z"],
            [
                DxfInfiniteLineGeometryValueRole::WcsUnitDirectionX,
                DxfInfiniteLineGeometryValueRole::WcsUnitDirectionY,
                DxfInfiniteLineGeometryValueRole::WcsUnitDirectionZ,
            ],
        )?,
    })
}

fn required_triple(
    cards: &DxfInfiniteLineGeometryCardDirectory,
    record: DxfInfiniteLineGeometryRecordEntry,
    namespace: &'static str,
    field_ids: [&'static str; 3],
    roles: [DxfInfiniteLineGeometryValueRole; 3],
) -> Result<[DxfInfiniteLineGeometrySemanticValue; 3], DxfError> {
    Ok([
        semantic_value(cards, record, roles[0], namespace, field_ids[0])?,
        semantic_value(cards, record, roles[1], namespace, field_ids[1])?,
        semantic_value(cards, record, roles[2], namespace, field_ids[2])?,
    ])
}

fn semantic_value(
    cards: &DxfInfiniteLineGeometryCardDirectory,
    record: DxfInfiniteLineGeometryRecordEntry,
    role: DxfInfiniteLineGeometryValueRole,
    namespace: &'static str,
    field_id: &'static str,
) -> Result<DxfInfiniteLineGeometrySemanticValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), namespace, field_id);
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfInfiniteLineGeometryValueCardState::Absent => Ok(DxfSemanticValue::invalid(
            DxfInfiniteLineGeometrySemanticIssue::MissingRequiredValue,
            field,
            None,
        )),
        DxfInfiniteLineGeometryValueCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(number) => DxfSemanticValue::explicit(number, field, raw),
                Err(DxfInfiniteLineGeometryNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfInfiniteLineGeometrySemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfInfiniteLineGeometryValueCardState::Multiple { occurrence_count } => {
            let primary = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfInfiniteLineGeometrySemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(primary)?),
            ))
        }
    }
}

fn unique_value(
    cards: &DxfInfiniteLineGeometryCardDirectory,
    card: DxfInfiniteLineGeometryValueCard,
) -> Result<DxfInfiniteLineGeometryValue, DxfError> {
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
    cards: &DxfInfiniteLineGeometryCardDirectory,
    card: DxfInfiniteLineGeometryValueCard,
) -> Result<DxfInfiniteLineGeometryValue, DxfError> {
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
    value: DxfInfiniteLineGeometryValue,
) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn triple_value(values: &[DxfInfiniteLineGeometrySemanticValue; 3]) -> Option<[DxfDouble; 3]> {
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

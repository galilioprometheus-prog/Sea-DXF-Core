//! Typed POINT and LINE component semantics over cardinality cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBasicGeometryCardDirectory,
    DxfBasicGeometryComponentCard, DxfBasicGeometryComponentCardState,
    DxfBasicGeometryComponentRole, DxfBasicGeometryKind, DxfBasicGeometryNumericIssue,
    DxfBasicGeometryRecordEntry, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId,
};

const POINT_NAMESPACE: &str = "basic_geometry.point";
const LINE_NAMESPACE: &str = "basic_geometry.line";
const ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const DEFAULT_EXTRUSION: [DxfDouble; 3] = [ZERO, ZERO, DxfDouble::from_bits(1.0_f64.to_bits())];

/// Why one reviewed basic-geometry component has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBasicGeometrySemanticIssue {
    MissingRequiredComponent,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleComponents { occurrence_count: u32 },
}

/// Source-anchored semantic state for one POINT or LINE double component.
pub type DxfBasicGeometrySemanticValue = DxfSemanticValue<DxfDouble, DxfBasicGeometrySemanticIssue>;

/// Reviewed POINT component semantics without coordinate transformation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPointGeometrySemantics {
    record: DxfBasicGeometryRecordEntry,
    location: [DxfBasicGeometrySemanticValue; 3],
    thickness: DxfBasicGeometrySemanticValue,
    extrusion: [DxfBasicGeometrySemanticValue; 3],
    ucs_x_axis_angle: DxfBasicGeometrySemanticValue,
}

impl DxfPointGeometrySemantics {
    #[must_use]
    pub const fn record(self) -> DxfBasicGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn location(&self) -> &[DxfBasicGeometrySemanticValue; 3] {
        &self.location
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfBasicGeometrySemanticValue; 3] {
        &self.extrusion
    }

    #[must_use]
    pub const fn thickness(&self) -> &DxfBasicGeometrySemanticValue {
        &self.thickness
    }

    #[must_use]
    pub const fn ucs_x_axis_angle(&self) -> &DxfBasicGeometrySemanticValue {
        &self.ucs_x_axis_angle
    }

    #[must_use]
    pub fn location_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.location)
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.extrusion)
    }

    #[must_use]
    pub fn thickness_value(&self) -> Option<DxfDouble> {
        self.thickness.value().copied()
    }

    #[must_use]
    pub fn ucs_x_axis_angle_value(&self) -> Option<DxfDouble> {
        self.ucs_x_axis_angle.value().copied()
    }
}

/// Reviewed LINE component semantics without coordinate transformation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfLineGeometrySemantics {
    record: DxfBasicGeometryRecordEntry,
    start: [DxfBasicGeometrySemanticValue; 3],
    endpoint: [DxfBasicGeometrySemanticValue; 3],
    extrusion: [DxfBasicGeometrySemanticValue; 3],
}

impl DxfLineGeometrySemantics {
    #[must_use]
    pub const fn record(self) -> DxfBasicGeometryRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn start(&self) -> &[DxfBasicGeometrySemanticValue; 3] {
        &self.start
    }

    #[must_use]
    pub const fn endpoint(&self) -> &[DxfBasicGeometrySemanticValue; 3] {
        &self.endpoint
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfBasicGeometrySemanticValue; 3] {
        &self.extrusion
    }

    #[must_use]
    pub fn start_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.start)
    }

    #[must_use]
    pub fn endpoint_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.endpoint)
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.extrusion)
    }
}

/// Typed semantic family stored behind one compact directory entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBasicGeometrySemanticKind {
    Point,
    Line,
}

/// Compact source-order entry into POINT or LINE semantic storage.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBasicGeometrySemanticEntry {
    record: DxfBasicGeometryRecordEntry,
    kind: DxfBasicGeometrySemanticKind,
}

impl DxfBasicGeometrySemanticEntry {
    #[must_use]
    pub const fn kind(self) -> DxfBasicGeometrySemanticKind {
        self.kind
    }

    #[must_use]
    pub const fn record(self) -> DxfBasicGeometryRecordEntry {
        self.record
    }
}

/// Immutable typed POINT/LINE component semantics with retained raw evidence.
#[derive(Debug)]
pub struct DxfBasicGeometrySemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfBasicGeometryCardDirectory,
    entries: Box<[DxfBasicGeometrySemanticEntry]>,
}

impl DxfBasicGeometrySemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.basic_geometry_card_directory(cancellation)?;
        if cards.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: cards.source_id(),
            });
        }

        let mut entries = Vec::new();
        for record in cards.evidence_directory().records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            let kind = match record.kind() {
                DxfBasicGeometryKind::Point => DxfBasicGeometrySemanticKind::Point,
                DxfBasicGeometryKind::Line => DxfBasicGeometrySemanticKind::Line,
            };
            entries.push(DxfBasicGeometrySemanticEntry { record, kind });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfBasicGeometryCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfBasicGeometrySemanticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfBasicGeometrySemanticEntry> {
        let index = usize::try_from(ordinal).ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entry_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfBasicGeometrySemanticEntry> {
        let index = self
            .entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.record().record().ordinal()
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    pub fn point_for_entry(
        &self,
        entry: DxfBasicGeometrySemanticEntry,
    ) -> Result<Option<DxfPointGeometrySemantics>, DxfError> {
        if entry.kind() != DxfBasicGeometrySemanticKind::Point {
            return Ok(None);
        }
        point_semantics(&self.cards, entry.record()).map(Some)
    }

    pub fn line_for_entry(
        &self,
        entry: DxfBasicGeometrySemanticEntry,
    ) -> Result<Option<DxfLineGeometrySemantics>, DxfError> {
        if entry.kind() != DxfBasicGeometrySemanticKind::Line {
            return Ok(None);
        }
        line_semantics(&self.cards, entry.record()).map(Some)
    }

    pub fn point_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfPointGeometrySemantics>, DxfError> {
        let Some(entry) = self.entry_for_raw_record(raw_record_ordinal) else {
            return Ok(None);
        };
        self.point_for_entry(entry)
    }

    pub fn line_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfLineGeometrySemantics>, DxfError> {
        let Some(entry) = self.entry_for_raw_record(raw_record_ordinal) else {
            return Ok(None);
        };
        self.line_for_entry(entry)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn basic_geometry_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometrySemanticDirectory, DxfError> {
        DxfBasicGeometrySemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn basic_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).basic_geometry_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn basic_geometry_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometrySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).basic_geometry_semantic_directory(cancellation)
    }
}

fn point_semantics(
    cards: &DxfBasicGeometryCardDirectory,
    record: DxfBasicGeometryRecordEntry,
) -> Result<DxfPointGeometrySemantics, DxfError> {
    Ok(DxfPointGeometrySemantics {
        record,
        location: required_triple(
            cards,
            record,
            POINT_NAMESPACE,
            ["location_x", "location_y", "location_z"],
            [
                DxfBasicGeometryComponentRole::WcsLocationOrStartX,
                DxfBasicGeometryComponentRole::WcsLocationOrStartY,
                DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
            ],
        )?,
        thickness: semantic_component(
            cards,
            record,
            DxfBasicGeometryComponentRole::Thickness,
            POINT_NAMESPACE,
            "thickness",
            Some(ZERO),
        )?,
        extrusion: extrusion_triple(cards, record, POINT_NAMESPACE)?,
        ucs_x_axis_angle: semantic_component(
            cards,
            record,
            DxfBasicGeometryComponentRole::UcsXAxisAngle,
            POINT_NAMESPACE,
            "ucs_x_axis_angle",
            Some(ZERO),
        )?,
    })
}

fn line_semantics(
    cards: &DxfBasicGeometryCardDirectory,
    record: DxfBasicGeometryRecordEntry,
) -> Result<DxfLineGeometrySemantics, DxfError> {
    Ok(DxfLineGeometrySemantics {
        record,
        start: required_triple(
            cards,
            record,
            LINE_NAMESPACE,
            ["start_x", "start_y", "start_z"],
            [
                DxfBasicGeometryComponentRole::WcsLocationOrStartX,
                DxfBasicGeometryComponentRole::WcsLocationOrStartY,
                DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
            ],
        )?,
        endpoint: required_triple(
            cards,
            record,
            LINE_NAMESPACE,
            ["endpoint_x", "endpoint_y", "endpoint_z"],
            [
                DxfBasicGeometryComponentRole::WcsEndpointX,
                DxfBasicGeometryComponentRole::WcsEndpointY,
                DxfBasicGeometryComponentRole::WcsEndpointZ,
            ],
        )?,
        extrusion: extrusion_triple(cards, record, LINE_NAMESPACE)?,
    })
}

fn required_triple(
    cards: &DxfBasicGeometryCardDirectory,
    record: DxfBasicGeometryRecordEntry,
    namespace: &'static str,
    field_ids: [&'static str; 3],
    roles: [DxfBasicGeometryComponentRole; 3],
) -> Result<[DxfBasicGeometrySemanticValue; 3], DxfError> {
    Ok([
        semantic_component(cards, record, roles[0], namespace, field_ids[0], None)?,
        semantic_component(cards, record, roles[1], namespace, field_ids[1], None)?,
        semantic_component(cards, record, roles[2], namespace, field_ids[2], None)?,
    ])
}

fn extrusion_triple(
    cards: &DxfBasicGeometryCardDirectory,
    record: DxfBasicGeometryRecordEntry,
    namespace: &'static str,
) -> Result<[DxfBasicGeometrySemanticValue; 3], DxfError> {
    Ok([
        semantic_component(
            cards,
            record,
            DxfBasicGeometryComponentRole::ExtrusionX,
            namespace,
            "extrusion_x",
            Some(DEFAULT_EXTRUSION[0]),
        )?,
        semantic_component(
            cards,
            record,
            DxfBasicGeometryComponentRole::ExtrusionY,
            namespace,
            "extrusion_y",
            Some(DEFAULT_EXTRUSION[1]),
        )?,
        semantic_component(
            cards,
            record,
            DxfBasicGeometryComponentRole::ExtrusionZ,
            namespace,
            "extrusion_z",
            Some(DEFAULT_EXTRUSION[2]),
        )?,
    ])
}

fn semantic_component(
    cards: &DxfBasicGeometryCardDirectory,
    record: DxfBasicGeometryRecordEntry,
    role: DxfBasicGeometryComponentRole,
    namespace: &'static str,
    field_id: &'static str,
    default: Option<DxfDouble>,
) -> Result<DxfBasicGeometrySemanticValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), namespace, field_id);
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfBasicGeometryComponentCardState::Absent => Ok(match default {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfBasicGeometrySemanticIssue::MissingRequiredComponent,
                field,
                None,
            ),
        }),
        DxfBasicGeometryComponentCardState::Unique => {
            let component = unique_component(cards, card)?;
            let raw = component_provenance(component)?;
            Ok(match component.value() {
                Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                Err(DxfBasicGeometryNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfBasicGeometrySemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfBasicGeometryComponentCardState::Multiple { occurrence_count } => {
            let primary = first_component(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfBasicGeometrySemanticIssue::MultipleComponents { occurrence_count },
                field,
                Some(component_provenance(primary)?),
            ))
        }
    }
}

fn unique_component(
    cards: &DxfBasicGeometryCardDirectory,
    card: DxfBasicGeometryComponentCard,
) -> Result<crate::DxfBasicGeometryComponent, DxfError> {
    let members = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
        return Err(invalid_internal_data());
    };
    cards
        .component_for_member(*member)
        .ok_or_else(invalid_internal_data)
}

fn first_component(
    cards: &DxfBasicGeometryCardDirectory,
    card: DxfBasicGeometryComponentCard,
) -> Result<crate::DxfBasicGeometryComponent, DxfError> {
    let member = cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)?;
    cards
        .component_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn component_provenance(
    component: crate::DxfBasicGeometryComponent,
) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        component.group().occurrence(),
        component.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn triple_value(values: &[DxfBasicGeometrySemanticValue; 3]) -> Option<[DxfDouble; 3]> {
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

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

//! Lazy typed classic POLYLINE record semantics over M9.2f cards.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfPolylineRecordCardDirectory,
    DxfPolylineRecordCardMember, DxfPolylineRecordNumber, DxfPolylineRecordNumericIssue,
    DxfPolylineRecordValueCard, DxfPolylineRecordValueCardState, DxfPolylineRecordValueEntry,
    DxfPolylineRecordValueRole, DxfRawDocumentView, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "polyline.record";
const DEFAULT_ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const DEFAULT_EXTRUSION: [DxfDouble; 3] = [
    DEFAULT_ZERO,
    DEFAULT_ZERO,
    DxfDouble::from_bits(1.0_f64.to_bits()),
];

/// Why one reviewed classic POLYLINE record field has no usable value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineRecordSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored state for one classic POLYLINE record double.
pub type DxfPolylineRecordSemanticDouble =
    DxfSemanticValue<DxfDouble, DxfPolylineRecordSemanticIssue>;
/// Source-anchored state for one classic POLYLINE record signed integer.
pub type DxfPolylineRecordSemanticInteger = DxfSemanticValue<i16, DxfPolylineRecordSemanticIssue>;

/// Lazy reviewed semantics for one exact classic POLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineRecordSemantics {
    record: DxfPolylineRecordValueEntry,
    dummy_point: [DxfPolylineRecordSemanticDouble; 3],
    thickness: DxfPolylineRecordSemanticDouble,
    default_start_width: DxfPolylineRecordSemanticDouble,
    default_end_width: DxfPolylineRecordSemanticDouble,
    flags: DxfPolylineRecordSemanticInteger,
    mesh_m_vertex_count: DxfPolylineRecordSemanticInteger,
    mesh_n_vertex_count: DxfPolylineRecordSemanticInteger,
    smooth_surface_m_density: DxfPolylineRecordSemanticInteger,
    smooth_surface_n_density: DxfPolylineRecordSemanticInteger,
    smooth_surface_type: DxfPolylineRecordSemanticInteger,
    extrusion: [DxfPolylineRecordSemanticDouble; 3],
}

impl DxfPolylineRecordSemantics {
    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn dummy_point(&self) -> &[DxfPolylineRecordSemanticDouble; 3] {
        &self.dummy_point
    }

    #[must_use]
    pub const fn thickness(&self) -> &DxfPolylineRecordSemanticDouble {
        &self.thickness
    }

    #[must_use]
    pub const fn default_start_width(&self) -> &DxfPolylineRecordSemanticDouble {
        &self.default_start_width
    }

    #[must_use]
    pub const fn default_end_width(&self) -> &DxfPolylineRecordSemanticDouble {
        &self.default_end_width
    }

    #[must_use]
    pub const fn flags(&self) -> &DxfPolylineRecordSemanticInteger {
        &self.flags
    }

    #[must_use]
    pub const fn mesh_m_vertex_count(&self) -> &DxfPolylineRecordSemanticInteger {
        &self.mesh_m_vertex_count
    }

    #[must_use]
    pub const fn mesh_n_vertex_count(&self) -> &DxfPolylineRecordSemanticInteger {
        &self.mesh_n_vertex_count
    }

    #[must_use]
    pub const fn smooth_surface_m_density(&self) -> &DxfPolylineRecordSemanticInteger {
        &self.smooth_surface_m_density
    }

    #[must_use]
    pub const fn smooth_surface_n_density(&self) -> &DxfPolylineRecordSemanticInteger {
        &self.smooth_surface_n_density
    }

    #[must_use]
    pub const fn smooth_surface_type(&self) -> &DxfPolylineRecordSemanticInteger {
        &self.smooth_surface_type
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfPolylineRecordSemanticDouble; 3] {
        &self.extrusion
    }

    #[must_use]
    pub fn dummy_point_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.dummy_point)
    }

    #[must_use]
    pub fn flags_value(&self) -> Option<i16> {
        self.flags.value().copied()
    }

    #[must_use]
    pub fn is_closed_or_mesh_closed_m(&self) -> Option<bool> {
        self.flag_bit(1)
    }

    #[must_use]
    pub fn has_curve_fit_vertices(&self) -> Option<bool> {
        self.flag_bit(2)
    }

    #[must_use]
    pub fn has_spline_fit_vertices(&self) -> Option<bool> {
        self.flag_bit(4)
    }

    #[must_use]
    pub fn is_3d_polyline(&self) -> Option<bool> {
        self.flag_bit(8)
    }

    #[must_use]
    pub fn is_polygon_mesh(&self) -> Option<bool> {
        self.flag_bit(16)
    }

    #[must_use]
    pub fn is_mesh_closed_n(&self) -> Option<bool> {
        self.flag_bit(32)
    }

    #[must_use]
    pub fn is_polyface_mesh(&self) -> Option<bool> {
        self.flag_bit(64)
    }

    #[must_use]
    pub fn has_continuous_linetype_pattern(&self) -> Option<bool> {
        self.flag_bit(128)
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(&self.extrusion)
    }

    fn flag_bit(&self, bit: i16) -> Option<bool> {
        Some(self.flags_value()? & bit != 0)
    }
}

/// Immutable lazy classic POLYLINE record semantics retaining M9.2f evidence.
#[derive(Debug)]
pub struct DxfPolylineRecordSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfPolylineRecordCardDirectory,
}

impl DxfPolylineRecordSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.polyline_record_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfPolylineRecordCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfPolylineRecordValueEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn semantics_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfPolylineRecordSemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_polyline_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        record_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfPolylineRecordValueEntry,
    ) -> Result<Option<DxfPolylineRecordSemantics>, DxfError> {
        let raw_ordinal = record.sequence().polyline_record().ordinal();
        if self
            .cards
            .evidence_directory()
            .record_for_polyline_raw_ordinal(raw_ordinal)
            != Some(record)
        {
            return Ok(None);
        }
        record_semantics(&self.cards, record).map(Some)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_record_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineRecordSemanticDirectory, DxfError> {
        DxfPolylineRecordSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_record_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_record_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineRecordSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_record_semantic_directory(cancellation)
    }
}

fn record_semantics(
    cards: &DxfPolylineRecordCardDirectory,
    record: DxfPolylineRecordValueEntry,
) -> Result<DxfPolylineRecordSemantics, DxfError> {
    Ok(DxfPolylineRecordSemantics {
        record,
        dummy_point: [
            double_semantic(
                cards,
                record,
                DxfPolylineRecordValueRole::DummyX,
                "dummy_x",
                None,
            )?,
            double_semantic(
                cards,
                record,
                DxfPolylineRecordValueRole::DummyY,
                "dummy_y",
                None,
            )?,
            double_semantic(
                cards,
                record,
                DxfPolylineRecordValueRole::Elevation,
                "elevation",
                None,
            )?,
        ],
        thickness: defaulted_double(
            cards,
            record,
            DxfPolylineRecordValueRole::Thickness,
            "thickness",
        )?,
        default_start_width: defaulted_double(
            cards,
            record,
            DxfPolylineRecordValueRole::DefaultStartWidth,
            "default_start_width",
        )?,
        default_end_width: defaulted_double(
            cards,
            record,
            DxfPolylineRecordValueRole::DefaultEndWidth,
            "default_end_width",
        )?,
        flags: defaulted_integer(cards, record, DxfPolylineRecordValueRole::Flags, "flags")?,
        mesh_m_vertex_count: defaulted_integer(
            cards,
            record,
            DxfPolylineRecordValueRole::MeshMVertexCount,
            "mesh_m_vertex_count",
        )?,
        mesh_n_vertex_count: defaulted_integer(
            cards,
            record,
            DxfPolylineRecordValueRole::MeshNVertexCount,
            "mesh_n_vertex_count",
        )?,
        smooth_surface_m_density: defaulted_integer(
            cards,
            record,
            DxfPolylineRecordValueRole::SmoothSurfaceMDensity,
            "smooth_surface_m_density",
        )?,
        smooth_surface_n_density: defaulted_integer(
            cards,
            record,
            DxfPolylineRecordValueRole::SmoothSurfaceNDensity,
            "smooth_surface_n_density",
        )?,
        smooth_surface_type: defaulted_integer(
            cards,
            record,
            DxfPolylineRecordValueRole::SmoothSurfaceType,
            "smooth_surface_type",
        )?,
        extrusion: [
            double_semantic(
                cards,
                record,
                DxfPolylineRecordValueRole::ExtrusionX,
                "extrusion_x",
                Some(DEFAULT_EXTRUSION[0]),
            )?,
            double_semantic(
                cards,
                record,
                DxfPolylineRecordValueRole::ExtrusionY,
                "extrusion_y",
                Some(DEFAULT_EXTRUSION[1]),
            )?,
            double_semantic(
                cards,
                record,
                DxfPolylineRecordValueRole::ExtrusionZ,
                "extrusion_z",
                Some(DEFAULT_EXTRUSION[2]),
            )?,
        ],
    })
}

fn defaulted_double(
    cards: &DxfPolylineRecordCardDirectory,
    record: DxfPolylineRecordValueEntry,
    role: DxfPolylineRecordValueRole,
    field_id: &'static str,
) -> Result<DxfPolylineRecordSemanticDouble, DxfError> {
    double_semantic(cards, record, role, field_id, Some(DEFAULT_ZERO))
}

fn defaulted_integer(
    cards: &DxfPolylineRecordCardDirectory,
    record: DxfPolylineRecordValueEntry,
    role: DxfPolylineRecordValueRole,
    field_id: &'static str,
) -> Result<DxfPolylineRecordSemanticInteger, DxfError> {
    integer_semantic(cards, record, role, field_id, 0)
}

fn double_semantic(
    cards: &DxfPolylineRecordCardDirectory,
    record: DxfPolylineRecordValueEntry,
    role: DxfPolylineRecordValueRole,
    field_id: &'static str,
    default: Option<DxfDouble>,
) -> Result<DxfPolylineRecordSemanticDouble, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfPolylineRecordValueCardState::Absent => Ok(match default {
            Some(value) => DxfSemanticValue::defaulted(value, field),
            None => DxfSemanticValue::invalid(
                DxfPolylineRecordSemanticIssue::MissingRequiredValue,
                field,
                None,
            ),
        }),
        DxfPolylineRecordValueCardState::Unique => {
            let value = value_for_member(cards, unique_member(cards, card)?)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfPolylineRecordNumber::Double(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfPolylineRecordNumber::Int16(_)) => return Err(invalid_internal_data()),
                Err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfPolylineRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfPolylineRecordValueCardState::Multiple { occurrence_count } => {
            let value = value_for_member(cards, first_member(cards, card)?)?;
            Ok(DxfSemanticValue::invalid(
                DxfPolylineRecordSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(value)?),
            ))
        }
    }
}

fn integer_semantic(
    cards: &DxfPolylineRecordCardDirectory,
    record: DxfPolylineRecordValueEntry,
    role: DxfPolylineRecordValueRole,
    field_id: &'static str,
    default: i16,
) -> Result<DxfPolylineRecordSemanticInteger, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field_id);
    let card = card_for_role(cards, record, role)?;
    match card.state() {
        DxfPolylineRecordValueCardState::Absent => Ok(DxfSemanticValue::defaulted(default, field)),
        DxfPolylineRecordValueCardState::Unique => {
            let value = value_for_member(cards, unique_member(cards, card)?)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(DxfPolylineRecordNumber::Int16(number)) => {
                    DxfSemanticValue::explicit(number, field, raw)
                }
                Ok(DxfPolylineRecordNumber::Double(_)) => return Err(invalid_internal_data()),
                Err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber(issue)) => {
                    DxfSemanticValue::invalid(
                        DxfPolylineRecordSemanticIssue::InvalidAsciiNumber(issue),
                        field,
                        Some(raw),
                    )
                }
            })
        }
        DxfPolylineRecordValueCardState::Multiple { occurrence_count } => {
            let value = value_for_member(cards, first_member(cards, card)?)?;
            Ok(DxfSemanticValue::invalid(
                DxfPolylineRecordSemanticIssue::MultipleValues { occurrence_count },
                field,
                Some(value_provenance(value)?),
            ))
        }
    }
}

fn card_for_role(
    cards: &DxfPolylineRecordCardDirectory,
    record: DxfPolylineRecordValueEntry,
    role: DxfPolylineRecordValueRole,
) -> Result<DxfPolylineRecordValueCard, DxfError> {
    cards
        .card_for_role(record.sequence().polyline_record().ordinal(), role)
        .ok_or_else(invalid_internal_data)
}

fn unique_member(
    cards: &DxfPolylineRecordCardDirectory,
    card: DxfPolylineRecordValueCard,
) -> Result<DxfPolylineRecordCardMember, DxfError> {
    let members = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let [member] = members else {
        return Err(invalid_internal_data());
    };
    Ok(*member)
}

fn first_member(
    cards: &DxfPolylineRecordCardDirectory,
    card: DxfPolylineRecordValueCard,
) -> Result<DxfPolylineRecordCardMember, DxfError> {
    cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or_else(invalid_internal_data)
}

fn value_for_member(
    cards: &DxfPolylineRecordCardDirectory,
    member: DxfPolylineRecordCardMember,
) -> Result<crate::DxfPolylineRecordValue, DxfError> {
    cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(
    value: crate::DxfPolylineRecordValue,
) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn triple_value(values: &[DxfPolylineRecordSemanticDouble; 3]) -> Option<[DxfDouble; 3]> {
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

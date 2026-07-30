//! Lazy typed semantics over planar-face cardinality cards.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfPlanarFaceCardDirectory, DxfPlanarFaceKind, DxfPlanarFaceRecordEntry,
    DxfPlanarFaceValueCardState, DxfPlanarFaceValueRole, DxfRawDocumentView, DxfSemanticValue,
    DxfSourceId,
    planar_face_geometry_semantic_value::{defaulted_corner, semantic_double, semantic_i16},
};

const FACE3D_NAMESPACE: &str = "planar_face.face3d";
const SOLID_NAMESPACE: &str = "planar_face.solid";
const TRACE_NAMESPACE: &str = "planar_face.trace";
const ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const ONE: DxfDouble = DxfDouble::from_bits(1.0_f64.to_bits());
const CORNER_ROLES: [[DxfPlanarFaceValueRole; 3]; 4] = [
    [
        DxfPlanarFaceValueRole::FirstCornerX,
        DxfPlanarFaceValueRole::FirstCornerY,
        DxfPlanarFaceValueRole::FirstCornerZ,
    ],
    [
        DxfPlanarFaceValueRole::SecondCornerX,
        DxfPlanarFaceValueRole::SecondCornerY,
        DxfPlanarFaceValueRole::SecondCornerZ,
    ],
    [
        DxfPlanarFaceValueRole::ThirdCornerX,
        DxfPlanarFaceValueRole::ThirdCornerY,
        DxfPlanarFaceValueRole::ThirdCornerZ,
    ],
    [
        DxfPlanarFaceValueRole::FourthCornerX,
        DxfPlanarFaceValueRole::FourthCornerY,
        DxfPlanarFaceValueRole::FourthCornerZ,
    ],
];
const CORNER_FIELDS: [[&str; 3]; 4] = [
    ["first_corner_x", "first_corner_y", "first_corner_z"],
    ["second_corner_x", "second_corner_y", "second_corner_z"],
    ["third_corner_x", "third_corner_y", "third_corner_z"],
    ["fourth_corner_x", "fourth_corner_y", "fourth_corner_z"],
];

/// Why one reviewed planar-face value has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPlanarFaceSemanticIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
    UnavailableDefaultSource,
}

pub type DxfPlanarFaceDoubleSemanticValue = DxfSemanticValue<DxfDouble, DxfPlanarFaceSemanticIssue>;
pub type DxfPlanarFaceInt16SemanticValue = DxfSemanticValue<i16, DxfPlanarFaceSemanticIssue>;

/// Selected and defaulted public values for one planar-face record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceSemantics {
    record: DxfPlanarFaceRecordEntry,
    corners: [[DxfPlanarFaceDoubleSemanticValue; 3]; 4],
    thickness: Option<DxfPlanarFaceDoubleSemanticValue>,
    extrusion: Option<[DxfPlanarFaceDoubleSemanticValue; 3]>,
    invisible_edge_flags: Option<DxfPlanarFaceInt16SemanticValue>,
}

impl DxfPlanarFaceSemantics {
    #[must_use]
    pub const fn record(self) -> DxfPlanarFaceRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfPlanarFaceKind {
        self.record.kind()
    }

    #[must_use]
    pub const fn corners(&self) -> &[[DxfPlanarFaceDoubleSemanticValue; 3]; 4] {
        &self.corners
    }

    #[must_use]
    pub const fn thickness(&self) -> Option<&DxfPlanarFaceDoubleSemanticValue> {
        self.thickness.as_ref()
    }

    #[must_use]
    pub const fn extrusion(&self) -> Option<&[DxfPlanarFaceDoubleSemanticValue; 3]> {
        self.extrusion.as_ref()
    }

    #[must_use]
    pub const fn invisible_edge_flags(&self) -> Option<&DxfPlanarFaceInt16SemanticValue> {
        self.invisible_edge_flags.as_ref()
    }

    #[must_use]
    pub fn corner_value(&self, index: usize) -> Option<[DxfDouble; 3]> {
        triple_value(self.corners.get(index)?)
    }

    #[must_use]
    pub fn thickness_value(&self) -> Option<DxfDouble> {
        self.thickness()?.value().copied()
    }

    #[must_use]
    pub fn extrusion_value(&self) -> Option<[DxfDouble; 3]> {
        triple_value(self.extrusion()?)
    }

    #[must_use]
    pub fn invisible_edge_flags_value(&self) -> Option<i16> {
        self.invisible_edge_flags()?.value().copied()
    }
}

/// Immutable lazy typed semantics retaining cards and raw evidence.
#[derive(Debug)]
pub struct DxfPlanarFaceSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfPlanarFaceCardDirectory,
}

impl DxfPlanarFaceSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.planar_face_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfPlanarFaceCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfPlanarFaceRecordEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfPlanarFaceRecordEntry,
    ) -> Result<Option<DxfPlanarFaceSemantics>, DxfError> {
        let known = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal());
        if known != Some(record) {
            return Ok(None);
        }
        planar_face_semantics(&self.cards, record).map(Some)
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfPlanarFaceSemantics>, DxfError> {
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
    pub fn planar_face_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceSemanticDirectory, DxfError> {
        DxfPlanarFaceSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn planar_face_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn planar_face_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_semantic_directory(cancellation)
    }
}

fn planar_face_semantics(
    cards: &DxfPlanarFaceCardDirectory,
    record: DxfPlanarFaceRecordEntry,
) -> Result<DxfPlanarFaceSemantics, DxfError> {
    let namespace = namespace(record.kind());
    let first = required_corner(cards, record, namespace, 0)?;
    let second = required_corner(cards, record, namespace, 1)?;
    let third = required_corner(cards, record, namespace, 2)?;
    let fourth = fourth_corner(cards, record, namespace, &third)?;
    let (thickness, extrusion, invisible_edge_flags) = match record.kind() {
        DxfPlanarFaceKind::Face3d => (
            None,
            None,
            Some(semantic_i16(
                cards,
                record,
                DxfPlanarFaceValueRole::InvisibleEdgeFlags,
                namespace,
                "invisible_edge_flags",
                Some(0),
            )?),
        ),
        DxfPlanarFaceKind::Solid | DxfPlanarFaceKind::Trace => (
            Some(semantic_double(
                cards,
                record,
                DxfPlanarFaceValueRole::Thickness,
                namespace,
                "thickness",
                Some(ZERO),
            )?),
            Some([
                semantic_double(
                    cards,
                    record,
                    DxfPlanarFaceValueRole::ExtrusionX,
                    namespace,
                    "extrusion_x",
                    Some(ZERO),
                )?,
                semantic_double(
                    cards,
                    record,
                    DxfPlanarFaceValueRole::ExtrusionY,
                    namespace,
                    "extrusion_y",
                    Some(ZERO),
                )?,
                semantic_double(
                    cards,
                    record,
                    DxfPlanarFaceValueRole::ExtrusionZ,
                    namespace,
                    "extrusion_z",
                    Some(ONE),
                )?,
            ]),
            None,
        ),
    };
    Ok(DxfPlanarFaceSemantics {
        record,
        corners: [first, second, third, fourth],
        thickness,
        extrusion,
        invisible_edge_flags,
    })
}

fn required_corner(
    cards: &DxfPlanarFaceCardDirectory,
    record: DxfPlanarFaceRecordEntry,
    namespace: &'static str,
    index: usize,
) -> Result<[DxfPlanarFaceDoubleSemanticValue; 3], DxfError> {
    Ok([
        semantic_double(
            cards,
            record,
            CORNER_ROLES[index][0],
            namespace,
            CORNER_FIELDS[index][0],
            None,
        )?,
        semantic_double(
            cards,
            record,
            CORNER_ROLES[index][1],
            namespace,
            CORNER_FIELDS[index][1],
            None,
        )?,
        semantic_double(
            cards,
            record,
            CORNER_ROLES[index][2],
            namespace,
            CORNER_FIELDS[index][2],
            None,
        )?,
    ])
}

fn fourth_corner(
    cards: &DxfPlanarFaceCardDirectory,
    record: DxfPlanarFaceRecordEntry,
    namespace: &'static str,
    third: &[DxfPlanarFaceDoubleSemanticValue; 3],
) -> Result<[DxfPlanarFaceDoubleSemanticValue; 3], DxfError> {
    let roles = CORNER_ROLES[3];
    let all_absent = roles.iter().all(|role| {
        cards
            .card_for_role(record.record().ordinal(), *role)
            .map(|card| card.state())
            == Some(DxfPlanarFaceValueCardState::Absent)
    });
    if all_absent
        && matches!(
            record.kind(),
            DxfPlanarFaceKind::Face3d | DxfPlanarFaceKind::Solid
        )
    {
        return Ok(defaulted_corner(
            cards.source_id(),
            namespace,
            CORNER_FIELDS[3],
            third,
        ));
    }
    required_corner(cards, record, namespace, 3)
}

const fn namespace(kind: DxfPlanarFaceKind) -> &'static str {
    match kind {
        DxfPlanarFaceKind::Face3d => FACE3D_NAMESPACE,
        DxfPlanarFaceKind::Solid => SOLID_NAMESPACE,
        DxfPlanarFaceKind::Trace => TRACE_NAMESPACE,
    }
}

fn triple_value(values: &[DxfPlanarFaceDoubleSemanticValue; 3]) -> Option<[DxfDouble; 3]> {
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

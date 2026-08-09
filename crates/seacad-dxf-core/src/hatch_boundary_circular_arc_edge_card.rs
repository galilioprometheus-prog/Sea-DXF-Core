//! Per-role cardinality cards for HATCH boundary CircularArc edge fields.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchBoundaryEdgeCard, DxfHatchBoundaryEdgeCardDirectory, DxfHatchBoundaryEdgeCardState,
    DxfHatchBoundaryEdgeMember, DxfHatchBoundaryEdgeMemberRange, DxfHatchBoundaryEdgeType,
    DxfRawDocumentView,
};

pub const DXF_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_ROLES: [DxfHatchBoundaryCircularArcEdgeRole; 6] = [
    DxfHatchBoundaryCircularArcEdgeRole::CenterX,
    DxfHatchBoundaryCircularArcEdgeRole::CenterY,
    DxfHatchBoundaryCircularArcEdgeRole::Radius,
    DxfHatchBoundaryCircularArcEdgeRole::StartAngle,
    DxfHatchBoundaryCircularArcEdgeRole::EndAngle,
    DxfHatchBoundaryCircularArcEdgeRole::Counterclockwise,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryCircularArcEdgeRole {
    CenterX,
    CenterY,
    Radius,
    StartAngle,
    EndAngle,
    Counterclockwise,
}

impl DxfHatchBoundaryCircularArcEdgeRole {
    #[must_use]
    pub const fn group_code(self) -> i16 {
        match self {
            Self::CenterX => 10,
            Self::CenterY => 20,
            Self::Radius => 40,
            Self::StartAngle => 50,
            Self::EndAngle => 51,
            Self::Counterclockwise => 73,
        }
    }
}

pub type DxfHatchBoundaryCircularArcEdgeCardState = DxfHatchBoundaryEdgeCardState;
pub type DxfHatchBoundaryCircularArcEdgeMemberRange = DxfHatchBoundaryEdgeMemberRange;
pub type DxfHatchBoundaryCircularArcEdgeMember =
    DxfHatchBoundaryEdgeMember<DxfHatchBoundaryCircularArcEdgeRole>;
pub type DxfHatchBoundaryCircularArcEdgeCard =
    DxfHatchBoundaryEdgeCard<DxfHatchBoundaryCircularArcEdgeRole>;
pub type DxfHatchBoundaryCircularArcEdgeCardDirectory =
    DxfHatchBoundaryEdgeCardDirectory<DxfHatchBoundaryCircularArcEdgeRole>;

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_circular_arc_edge_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeCardDirectory, DxfError> {
        DxfHatchBoundaryCircularArcEdgeCardDirectory::from_document(
            self,
            cancellation,
            DxfHatchBoundaryEdgeType::CircularArc,
            DXF_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_ROLES,
            DxfHatchBoundaryCircularArcEdgeRole::group_code,
        )
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_circular_arc_edge_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_circular_arc_edge_card_directory(cancellation)
    }
}

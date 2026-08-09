//! Per-role cardinality cards for HATCH boundary EllipticArc edge fields.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchBoundaryEdgeCard, DxfHatchBoundaryEdgeCardDirectory, DxfHatchBoundaryEdgeCardState,
    DxfHatchBoundaryEdgeMember, DxfHatchBoundaryEdgeMemberRange, DxfHatchBoundaryEdgeType,
    DxfRawDocumentView,
};

pub const DXF_HATCH_BOUNDARY_ELLIPTIC_ARC_EDGE_ROLES: [DxfHatchBoundaryEllipticArcEdgeRole; 8] = [
    DxfHatchBoundaryEllipticArcEdgeRole::CenterX,
    DxfHatchBoundaryEllipticArcEdgeRole::CenterY,
    DxfHatchBoundaryEllipticArcEdgeRole::MajorAxisEndpointX,
    DxfHatchBoundaryEllipticArcEdgeRole::MajorAxisEndpointY,
    DxfHatchBoundaryEllipticArcEdgeRole::MinorToMajorRatio,
    DxfHatchBoundaryEllipticArcEdgeRole::StartAngle,
    DxfHatchBoundaryEllipticArcEdgeRole::EndAngle,
    DxfHatchBoundaryEllipticArcEdgeRole::Counterclockwise,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEllipticArcEdgeRole {
    CenterX,
    CenterY,
    MajorAxisEndpointX,
    MajorAxisEndpointY,
    MinorToMajorRatio,
    StartAngle,
    EndAngle,
    Counterclockwise,
}

impl DxfHatchBoundaryEllipticArcEdgeRole {
    #[must_use]
    pub const fn group_code(self) -> i16 {
        match self {
            Self::CenterX => 10,
            Self::CenterY => 20,
            Self::MajorAxisEndpointX => 11,
            Self::MajorAxisEndpointY => 21,
            Self::MinorToMajorRatio => 40,
            Self::StartAngle => 50,
            Self::EndAngle => 51,
            Self::Counterclockwise => 73,
        }
    }
}

pub type DxfHatchBoundaryEllipticArcEdgeCardState = DxfHatchBoundaryEdgeCardState;
pub type DxfHatchBoundaryEllipticArcEdgeMemberRange = DxfHatchBoundaryEdgeMemberRange;
pub type DxfHatchBoundaryEllipticArcEdgeMember =
    DxfHatchBoundaryEdgeMember<DxfHatchBoundaryEllipticArcEdgeRole>;
pub type DxfHatchBoundaryEllipticArcEdgeCard =
    DxfHatchBoundaryEdgeCard<DxfHatchBoundaryEllipticArcEdgeRole>;
pub type DxfHatchBoundaryEllipticArcEdgeCardDirectory =
    DxfHatchBoundaryEdgeCardDirectory<DxfHatchBoundaryEllipticArcEdgeRole>;

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeCardDirectory, DxfError> {
        DxfHatchBoundaryEllipticArcEdgeCardDirectory::from_document(
            self,
            cancellation,
            DxfHatchBoundaryEdgeType::EllipticArc,
            DXF_HATCH_BOUNDARY_ELLIPTIC_ARC_EDGE_ROLES,
            DxfHatchBoundaryEllipticArcEdgeRole::group_code,
        )
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_elliptic_arc_edge_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_elliptic_arc_edge_card_directory(cancellation)
    }
}

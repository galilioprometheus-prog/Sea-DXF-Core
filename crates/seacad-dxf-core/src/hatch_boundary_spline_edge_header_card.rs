//! Per-role cardinality cards for fixed HATCH boundary Spline-edge headers.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchBoundaryEdgeCard, DxfHatchBoundaryEdgeCardDirectory, DxfHatchBoundaryEdgeCardState,
    DxfHatchBoundaryEdgeMember, DxfHatchBoundaryEdgeMemberRange, DxfHatchBoundaryEdgeType,
    DxfRawDocumentView,
};

pub const DXF_HATCH_BOUNDARY_SPLINE_EDGE_HEADER_ROLES: [DxfHatchBoundarySplineEdgeHeaderRole; 5] = [
    DxfHatchBoundarySplineEdgeHeaderRole::Degree,
    DxfHatchBoundarySplineEdgeHeaderRole::Rational,
    DxfHatchBoundarySplineEdgeHeaderRole::Periodic,
    DxfHatchBoundarySplineEdgeHeaderRole::KnotCount,
    DxfHatchBoundarySplineEdgeHeaderRole::ControlPointCount,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgeHeaderRole {
    Degree,
    Rational,
    Periodic,
    KnotCount,
    ControlPointCount,
}

impl DxfHatchBoundarySplineEdgeHeaderRole {
    #[must_use]
    pub const fn group_code(self) -> i16 {
        match self {
            Self::Degree => 94,
            Self::Rational => 73,
            Self::Periodic => 74,
            Self::KnotCount => 95,
            Self::ControlPointCount => 96,
        }
    }
}

pub type DxfHatchBoundarySplineEdgeHeaderCardState = DxfHatchBoundaryEdgeCardState;
pub type DxfHatchBoundarySplineEdgeHeaderMemberRange = DxfHatchBoundaryEdgeMemberRange;
pub type DxfHatchBoundarySplineEdgeHeaderMember =
    DxfHatchBoundaryEdgeMember<DxfHatchBoundarySplineEdgeHeaderRole>;
pub type DxfHatchBoundarySplineEdgeHeaderCard =
    DxfHatchBoundaryEdgeCard<DxfHatchBoundarySplineEdgeHeaderRole>;
pub type DxfHatchBoundarySplineEdgeHeaderCardDirectory =
    DxfHatchBoundaryEdgeCardDirectory<DxfHatchBoundarySplineEdgeHeaderRole>;

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_spline_edge_header_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderCardDirectory, DxfError> {
        DxfHatchBoundarySplineEdgeHeaderCardDirectory::from_document(
            self,
            cancellation,
            DxfHatchBoundaryEdgeType::Spline,
            DXF_HATCH_BOUNDARY_SPLINE_EDGE_HEADER_ROLES,
            DxfHatchBoundarySplineEdgeHeaderRole::group_code,
        )
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_header_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderCardDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_header_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_header_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderCardDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_header_card_directory(cancellation)
    }
}

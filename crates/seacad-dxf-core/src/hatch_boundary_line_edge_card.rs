//! Per-role cardinality cards for HATCH boundary Line edge fields.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchBoundaryEdgeCard, DxfHatchBoundaryEdgeCardDirectory, DxfHatchBoundaryEdgeCardState,
    DxfHatchBoundaryEdgeMember, DxfHatchBoundaryEdgeMemberRange, DxfHatchBoundaryEdgeType,
    DxfRawDocumentView,
};

pub const DXF_HATCH_BOUNDARY_LINE_EDGE_ROLES: [DxfHatchBoundaryLineEdgeRole; 4] = [
    DxfHatchBoundaryLineEdgeRole::StartX,
    DxfHatchBoundaryLineEdgeRole::StartY,
    DxfHatchBoundaryLineEdgeRole::EndX,
    DxfHatchBoundaryLineEdgeRole::EndY,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeRole {
    StartX,
    StartY,
    EndX,
    EndY,
}

impl DxfHatchBoundaryLineEdgeRole {
    #[must_use]
    pub const fn group_code(self) -> i16 {
        match self {
            Self::StartX => 10,
            Self::StartY => 20,
            Self::EndX => 11,
            Self::EndY => 21,
        }
    }
}

pub type DxfHatchBoundaryLineEdgeCardState = DxfHatchBoundaryEdgeCardState;
pub type DxfHatchBoundaryLineEdgeMemberRange = DxfHatchBoundaryEdgeMemberRange;
pub type DxfHatchBoundaryLineEdgeMember = DxfHatchBoundaryEdgeMember<DxfHatchBoundaryLineEdgeRole>;
pub type DxfHatchBoundaryLineEdgeCard = DxfHatchBoundaryEdgeCard<DxfHatchBoundaryLineEdgeRole>;
pub type DxfHatchBoundaryLineEdgeCardDirectory =
    DxfHatchBoundaryEdgeCardDirectory<DxfHatchBoundaryLineEdgeRole>;

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_line_edge_card_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCardDirectory, DxfError> {
        DxfHatchBoundaryLineEdgeCardDirectory::from_document(
            self,
            cancellation,
            DxfHatchBoundaryEdgeType::Line,
            DXF_HATCH_BOUNDARY_LINE_EDGE_ROLES,
            DxfHatchBoundaryLineEdgeRole::group_code,
        )
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_line_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_card_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_line_edge_card_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCardDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_card_directory(cancellation)
    }
}

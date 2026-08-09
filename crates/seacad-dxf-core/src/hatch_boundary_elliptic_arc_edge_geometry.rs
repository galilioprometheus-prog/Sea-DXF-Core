//! Exact-value OCS geometry for HATCH boundary EllipticArc edges.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryEllipticArcEdgeDirection, DxfHatchBoundaryEllipticArcEdgeMajorAxisState,
    DxfHatchBoundaryEllipticArcEdgeSemanticDirectory, DxfHatchBoundaryEllipticArcEdgeSemanticEntry,
    DxfRawDocumentView, DxfSourceId,
    read_support::{compact_len, ensure_not_cancelled, ensure_source, out_of_memory},
};

/// Exact OCS EllipticArc values without angle normalization or derived points.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeOcsSegment {
    center: [DxfDouble; 2],
    major_axis_endpoint_relative_to_center: [DxfDouble; 2],
    minor_to_major_ratio: DxfDouble,
    start_angle_degrees: DxfDouble,
    end_angle_degrees: DxfDouble,
    direction: DxfHatchBoundaryEllipticArcEdgeDirection,
}

impl DxfHatchBoundaryEllipticArcEdgeOcsSegment {
    #[must_use]
    pub const fn center(self) -> [DxfDouble; 2] {
        self.center
    }

    #[must_use]
    pub const fn major_axis_endpoint_relative_to_center(self) -> [DxfDouble; 2] {
        self.major_axis_endpoint_relative_to_center
    }

    #[must_use]
    pub const fn minor_to_major_ratio(self) -> DxfDouble {
        self.minor_to_major_ratio
    }

    #[must_use]
    pub const fn start_angle_degrees(self) -> DxfDouble {
        self.start_angle_degrees
    }

    #[must_use]
    pub const fn end_angle_degrees(self) -> DxfDouble {
        self.end_angle_degrees
    }

    #[must_use]
    pub const fn direction(self) -> DxfHatchBoundaryEllipticArcEdgeDirection {
        self.direction
    }
}

/// EllipticArc scalar values that prevented publication of one OCS segment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeUnavailableValues {
    mask: u8,
}

impl DxfHatchBoundaryEllipticArcEdgeUnavailableValues {
    const CENTER_X: u8 = 1;
    const CENTER_Y: u8 = 2;
    const MAJOR_AXIS_X: u8 = 4;
    const MAJOR_AXIS_Y: u8 = 8;
    const RATIO: u8 = 16;
    const START_ANGLE: u8 = 32;
    const END_ANGLE: u8 = 64;
    const DIRECTION: u8 = 128;

    #[must_use]
    pub const fn center_x(self) -> bool {
        self.mask & Self::CENTER_X != 0
    }

    #[must_use]
    pub const fn center_y(self) -> bool {
        self.mask & Self::CENTER_Y != 0
    }

    #[must_use]
    pub const fn major_axis_x(self) -> bool {
        self.mask & Self::MAJOR_AXIS_X != 0
    }

    #[must_use]
    pub const fn major_axis_y(self) -> bool {
        self.mask & Self::MAJOR_AXIS_Y != 0
    }

    #[must_use]
    pub const fn ratio(self) -> bool {
        self.mask & Self::RATIO != 0
    }

    #[must_use]
    pub const fn start_angle(self) -> bool {
        self.mask & Self::START_ANGLE != 0
    }

    #[must_use]
    pub const fn end_angle(self) -> bool {
        self.mask & Self::END_ANGLE != 0
    }

    #[must_use]
    pub const fn direction(self) -> bool {
        self.mask & Self::DIRECTION != 0
    }

    #[must_use]
    pub const fn count(self) -> u32 {
        self.mask.count_ones()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEllipticArcEdgeGeometryIssue {
    ValuesUnavailable(DxfHatchBoundaryEllipticArcEdgeUnavailableValues),
    MajorAxisUnavailable(DxfHatchBoundaryEllipticArcEdgeMajorAxisState),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeGeometryEntry {
    ordinal: u32,
    semantics: DxfHatchBoundaryEllipticArcEdgeSemanticEntry,
    geometry: Result<
        DxfHatchBoundaryEllipticArcEdgeOcsSegment,
        DxfHatchBoundaryEllipticArcEdgeGeometryIssue,
    >,
}

impl DxfHatchBoundaryEllipticArcEdgeGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn semantics(self) -> DxfHatchBoundaryEllipticArcEdgeSemanticEntry {
        self.semantics
    }

    pub const fn geometry(
        self,
    ) -> Result<
        DxfHatchBoundaryEllipticArcEdgeOcsSegment,
        DxfHatchBoundaryEllipticArcEdgeGeometryIssue,
    > {
        self.geometry
    }
}

/// Exact OCS EllipticArc results retaining the complete semantic directory.
#[derive(Debug)]
pub struct DxfHatchBoundaryEllipticArcEdgeGeometryDirectory {
    source_id: DxfSourceId,
    semantics: DxfHatchBoundaryEllipticArcEdgeSemanticDirectory,
    entries: Box<[DxfHatchBoundaryEllipticArcEdgeGeometryEntry]>,
}

impl DxfHatchBoundaryEllipticArcEdgeGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics =
            document.hatch_boundary_elliptic_arc_edge_semantic_directory(cancellation)?;
        ensure_source(document.source_id(), semantics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(semantics.entries().len())
            .map_err(|_| out_of_memory())?;
        for semantic in semantics.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfHatchBoundaryEllipticArcEdgeGeometryEntry {
                ordinal: compact_len(entries.len())?,
                semantics: semantic,
                geometry: geometry(semantic),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryEllipticArcEdgeGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryEllipticArcEdgeGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    /// Resolve the HATCH subclass owning one locally indexed geometry entry.
    pub(crate) fn subclass_ordinal_for_entry(&self, entry_ordinal: u64) -> Option<u64> {
        let entry = self.entry(entry_ordinal)?;
        let path_ordinal = entry.semantics().numeric().path_ordinal();
        self.semantics
            .numeric_directory()
            .card_directory()
            .edge_type_directory()
            .edge_directory()
            .path(path_ordinal)
            .map(|path| path.subclass_ordinal())
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryEllipticArcEdgeGeometryEntry> {
        self.semantics.entry_for_edge(edge_ordinal)?;
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| {
                entry.semantics().numeric().edge_ordinal()
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchBoundaryEllipticArcEdgeGeometryEntry]> {
        self.semantics.entries_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.semantics().numeric().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.semantics().numeric().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeGeometryDirectory, DxfError> {
        DxfHatchBoundaryEllipticArcEdgeGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_geometry_directory(cancellation)
    }
}

fn geometry(
    entry: DxfHatchBoundaryEllipticArcEdgeSemanticEntry,
) -> Result<DxfHatchBoundaryEllipticArcEdgeOcsSegment, DxfHatchBoundaryEllipticArcEdgeGeometryIssue>
{
    let semantics = entry.semantics();
    let center_x = semantics.center_x().value().copied();
    let center_y = semantics.center_y().value().copied();
    let major_axis_x = semantics.major_axis_endpoint_x().value().copied();
    let major_axis_y = semantics.major_axis_endpoint_y().value().copied();
    let ratio = semantics.minor_to_major_ratio_value();
    let start_angle_degrees = semantics.start_angle_degrees_value();
    let end_angle_degrees = semantics.end_angle_degrees_value();
    let direction = semantics.direction_value();
    let mut mask = 0_u8;
    for (value, bit) in [
        (
            center_x.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::CENTER_X,
        ),
        (
            center_y.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::CENTER_Y,
        ),
        (
            major_axis_x.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::MAJOR_AXIS_X,
        ),
        (
            major_axis_y.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::MAJOR_AXIS_Y,
        ),
        (
            ratio.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::RATIO,
        ),
        (
            start_angle_degrees.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::START_ANGLE,
        ),
        (
            end_angle_degrees.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::END_ANGLE,
        ),
        (
            direction.is_some(),
            DxfHatchBoundaryEllipticArcEdgeUnavailableValues::DIRECTION,
        ),
    ] {
        if !value {
            mask |= bit;
        }
    }
    if mask != 0 {
        return Err(
            DxfHatchBoundaryEllipticArcEdgeGeometryIssue::ValuesUnavailable(
                DxfHatchBoundaryEllipticArcEdgeUnavailableValues { mask },
            ),
        );
    }
    if semantics.major_axis_state() != DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Usable {
        return Err(
            DxfHatchBoundaryEllipticArcEdgeGeometryIssue::MajorAxisUnavailable(
                semantics.major_axis_state(),
            ),
        );
    }
    match (
        center_x,
        center_y,
        major_axis_x,
        major_axis_y,
        ratio,
        start_angle_degrees,
        end_angle_degrees,
        direction,
    ) {
        (
            Some(center_x),
            Some(center_y),
            Some(major_axis_x),
            Some(major_axis_y),
            Some(ratio),
            Some(start_angle_degrees),
            Some(end_angle_degrees),
            Some(direction),
        ) => Ok(DxfHatchBoundaryEllipticArcEdgeOcsSegment {
            center: [center_x, center_y],
            major_axis_endpoint_relative_to_center: [major_axis_x, major_axis_y],
            minor_to_major_ratio: ratio,
            start_angle_degrees,
            end_angle_degrees,
            direction,
        }),
        _ => Err(
            DxfHatchBoundaryEllipticArcEdgeGeometryIssue::ValuesUnavailable(
                DxfHatchBoundaryEllipticArcEdgeUnavailableValues { mask },
            ),
        ),
    }
}

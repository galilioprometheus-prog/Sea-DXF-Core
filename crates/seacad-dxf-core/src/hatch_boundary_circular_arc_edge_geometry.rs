//! Exact-value OCS geometry for HATCH boundary CircularArc edges.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryCircularArcEdgeDirection, DxfHatchBoundaryCircularArcEdgeSemanticDirectory,
    DxfHatchBoundaryCircularArcEdgeSemanticEntry, DxfRawDocumentView, DxfSourceId,
    read_support::{compact_len, ensure_not_cancelled, ensure_source, out_of_memory},
};

/// Exact OCS CircularArc values without angle normalization or derived endpoints.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeOcsSegment {
    center: [DxfDouble; 2],
    radius: DxfDouble,
    start_angle_degrees: DxfDouble,
    end_angle_degrees: DxfDouble,
    direction: DxfHatchBoundaryCircularArcEdgeDirection,
}

impl DxfHatchBoundaryCircularArcEdgeOcsSegment {
    #[must_use]
    pub const fn center(self) -> [DxfDouble; 2] {
        self.center
    }

    #[must_use]
    pub const fn radius(self) -> DxfDouble {
        self.radius
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
    pub const fn direction(self) -> DxfHatchBoundaryCircularArcEdgeDirection {
        self.direction
    }
}

/// CircularArc values that prevented publication of one OCS segment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeUnavailableValues {
    mask: u8,
}

impl DxfHatchBoundaryCircularArcEdgeUnavailableValues {
    const CENTER_X: u8 = 1;
    const CENTER_Y: u8 = 2;
    const RADIUS: u8 = 4;
    const START_ANGLE: u8 = 8;
    const END_ANGLE: u8 = 16;
    const DIRECTION: u8 = 32;

    #[must_use]
    pub const fn center_x(self) -> bool {
        self.mask & Self::CENTER_X != 0
    }

    #[must_use]
    pub const fn center_y(self) -> bool {
        self.mask & Self::CENTER_Y != 0
    }

    #[must_use]
    pub const fn radius(self) -> bool {
        self.mask & Self::RADIUS != 0
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
pub enum DxfHatchBoundaryCircularArcEdgeGeometryIssue {
    ValuesUnavailable(DxfHatchBoundaryCircularArcEdgeUnavailableValues),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeGeometryEntry {
    ordinal: u32,
    semantics: DxfHatchBoundaryCircularArcEdgeSemanticEntry,
    geometry: Result<
        DxfHatchBoundaryCircularArcEdgeOcsSegment,
        DxfHatchBoundaryCircularArcEdgeGeometryIssue,
    >,
}

impl DxfHatchBoundaryCircularArcEdgeGeometryEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn semantics(self) -> DxfHatchBoundaryCircularArcEdgeSemanticEntry {
        self.semantics
    }

    pub const fn geometry(
        self,
    ) -> Result<
        DxfHatchBoundaryCircularArcEdgeOcsSegment,
        DxfHatchBoundaryCircularArcEdgeGeometryIssue,
    > {
        self.geometry
    }
}

/// Exact OCS CircularArc results retaining the complete M14.4ac directory.
#[derive(Debug)]
pub struct DxfHatchBoundaryCircularArcEdgeGeometryDirectory {
    source_id: DxfSourceId,
    semantics: DxfHatchBoundaryCircularArcEdgeSemanticDirectory,
    entries: Box<[DxfHatchBoundaryCircularArcEdgeGeometryEntry]>,
}

impl DxfHatchBoundaryCircularArcEdgeGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics =
            document.hatch_boundary_circular_arc_edge_semantic_directory(cancellation)?;
        ensure_source(document.source_id(), semantics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(semantics.entries().len())
            .map_err(|_| out_of_memory())?;
        for semantic in semantics.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfHatchBoundaryCircularArcEdgeGeometryEntry {
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
    pub const fn semantic_directory(&self) -> &DxfHatchBoundaryCircularArcEdgeSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryCircularArcEdgeGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryCircularArcEdgeGeometryEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryCircularArcEdgeGeometryEntry> {
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
    ) -> Option<&[DxfHatchBoundaryCircularArcEdgeGeometryEntry]> {
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
    pub fn hatch_boundary_circular_arc_edge_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeGeometryDirectory, DxfError> {
        DxfHatchBoundaryCircularArcEdgeGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_geometry_directory(cancellation)
    }
}

fn geometry(
    entry: DxfHatchBoundaryCircularArcEdgeSemanticEntry,
) -> Result<DxfHatchBoundaryCircularArcEdgeOcsSegment, DxfHatchBoundaryCircularArcEdgeGeometryIssue>
{
    let semantics = entry.semantics();
    let center_x = semantics.center_x().value().copied();
    let center_y = semantics.center_y().value().copied();
    let radius = semantics.radius_value();
    let start_angle_degrees = semantics.start_angle_degrees_value();
    let end_angle_degrees = semantics.end_angle_degrees_value();
    let direction = semantics.direction_value();
    let mut mask = 0_u8;
    if center_x.is_none() {
        mask |= DxfHatchBoundaryCircularArcEdgeUnavailableValues::CENTER_X;
    }
    if center_y.is_none() {
        mask |= DxfHatchBoundaryCircularArcEdgeUnavailableValues::CENTER_Y;
    }
    if radius.is_none() {
        mask |= DxfHatchBoundaryCircularArcEdgeUnavailableValues::RADIUS;
    }
    if start_angle_degrees.is_none() {
        mask |= DxfHatchBoundaryCircularArcEdgeUnavailableValues::START_ANGLE;
    }
    if end_angle_degrees.is_none() {
        mask |= DxfHatchBoundaryCircularArcEdgeUnavailableValues::END_ANGLE;
    }
    if direction.is_none() {
        mask |= DxfHatchBoundaryCircularArcEdgeUnavailableValues::DIRECTION;
    }
    match (
        center_x,
        center_y,
        radius,
        start_angle_degrees,
        end_angle_degrees,
        direction,
    ) {
        (
            Some(center_x),
            Some(center_y),
            Some(radius),
            Some(start_angle_degrees),
            Some(end_angle_degrees),
            Some(direction),
        ) => Ok(DxfHatchBoundaryCircularArcEdgeOcsSegment {
            center: [center_x, center_y],
            radius,
            start_angle_degrees,
            end_angle_degrees,
            direction,
        }),
        _ => Err(
            DxfHatchBoundaryCircularArcEdgeGeometryIssue::ValuesUnavailable(
                DxfHatchBoundaryCircularArcEdgeUnavailableValues { mask },
            ),
        ),
    }
}

//! Required OCS endpoint semantics for HATCH boundary Line edges.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryLineEdgeNumericDirectory, DxfHatchBoundaryLineEdgeNumericEntry,
    DxfHatchBoundaryLineEdgeNumericIssue, DxfHatchBoundaryLineEdgeNumericValue, DxfRawDocumentView,
    DxfSemanticValue, DxfSourceId,
    read_support::{
        compact_len, ensure_not_cancelled, ensure_source, invalid_internal_data, out_of_memory,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeCoordinateIssue {
    MissingRequiredValue,
    Numeric(DxfHatchBoundaryLineEdgeNumericIssue),
}

pub type DxfHatchBoundaryLineEdgeCoordinateValue =
    DxfSemanticValue<DxfDouble, DxfHatchBoundaryLineEdgeCoordinateIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeCoordinates {
    start_x: DxfHatchBoundaryLineEdgeCoordinateValue,
    start_y: DxfHatchBoundaryLineEdgeCoordinateValue,
    end_x: DxfHatchBoundaryLineEdgeCoordinateValue,
    end_y: DxfHatchBoundaryLineEdgeCoordinateValue,
}

impl DxfHatchBoundaryLineEdgeCoordinates {
    #[must_use]
    pub const fn start_x(&self) -> &DxfHatchBoundaryLineEdgeCoordinateValue {
        &self.start_x
    }

    #[must_use]
    pub const fn start_y(&self) -> &DxfHatchBoundaryLineEdgeCoordinateValue {
        &self.start_y
    }

    #[must_use]
    pub const fn end_x(&self) -> &DxfHatchBoundaryLineEdgeCoordinateValue {
        &self.end_x
    }

    #[must_use]
    pub const fn end_y(&self) -> &DxfHatchBoundaryLineEdgeCoordinateValue {
        &self.end_y
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeOcsPoint {
    x: DxfDouble,
    y: DxfDouble,
}

impl DxfHatchBoundaryLineEdgeOcsPoint {
    #[must_use]
    pub const fn x(self) -> DxfDouble {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> DxfDouble {
        self.y
    }

    #[must_use]
    pub const fn values(self) -> [DxfDouble; 2] {
        [self.x, self.y]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeOcsEndpoints {
    start: DxfHatchBoundaryLineEdgeOcsPoint,
    end: DxfHatchBoundaryLineEdgeOcsPoint,
}

impl DxfHatchBoundaryLineEdgeOcsEndpoints {
    #[must_use]
    pub const fn start(self) -> DxfHatchBoundaryLineEdgeOcsPoint {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> DxfHatchBoundaryLineEdgeOcsPoint {
        self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeUnavailableCoordinates {
    mask: u8,
}

impl DxfHatchBoundaryLineEdgeUnavailableCoordinates {
    const START_X: u8 = 1;
    const START_Y: u8 = 2;
    const END_X: u8 = 4;
    const END_Y: u8 = 8;

    #[must_use]
    pub const fn start_x(self) -> bool {
        self.mask & Self::START_X != 0
    }

    #[must_use]
    pub const fn start_y(self) -> bool {
        self.mask & Self::START_Y != 0
    }

    #[must_use]
    pub const fn end_x(self) -> bool {
        self.mask & Self::END_X != 0
    }

    #[must_use]
    pub const fn end_y(self) -> bool {
        self.mask & Self::END_Y != 0
    }

    #[must_use]
    pub const fn count(self) -> u32 {
        self.mask.count_ones()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryLineEdgeEndpointIssue {
    CoordinatesUnavailable(DxfHatchBoundaryLineEdgeUnavailableCoordinates),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryLineEdgeCoordinateEntry {
    ordinal: u32,
    numeric: DxfHatchBoundaryLineEdgeNumericEntry,
    coordinates: DxfHatchBoundaryLineEdgeCoordinates,
}

impl DxfHatchBoundaryLineEdgeCoordinateEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn numeric(self) -> DxfHatchBoundaryLineEdgeNumericEntry {
        self.numeric
    }

    #[must_use]
    pub const fn coordinates(&self) -> &DxfHatchBoundaryLineEdgeCoordinates {
        &self.coordinates
    }

    pub fn ocs_endpoints(
        self,
    ) -> Result<DxfHatchBoundaryLineEdgeOcsEndpoints, DxfHatchBoundaryLineEdgeEndpointIssue> {
        let start_x = self.coordinates.start_x().value().copied();
        let start_y = self.coordinates.start_y().value().copied();
        let end_x = self.coordinates.end_x().value().copied();
        let end_y = self.coordinates.end_y().value().copied();
        let mut mask = 0_u8;
        if start_x.is_none() {
            mask |= DxfHatchBoundaryLineEdgeUnavailableCoordinates::START_X;
        }
        if start_y.is_none() {
            mask |= DxfHatchBoundaryLineEdgeUnavailableCoordinates::START_Y;
        }
        if end_x.is_none() {
            mask |= DxfHatchBoundaryLineEdgeUnavailableCoordinates::END_X;
        }
        if end_y.is_none() {
            mask |= DxfHatchBoundaryLineEdgeUnavailableCoordinates::END_Y;
        }
        match (start_x, start_y, end_x, end_y) {
            (Some(start_x), Some(start_y), Some(end_x), Some(end_y)) => {
                Ok(DxfHatchBoundaryLineEdgeOcsEndpoints {
                    start: DxfHatchBoundaryLineEdgeOcsPoint {
                        x: start_x,
                        y: start_y,
                    },
                    end: DxfHatchBoundaryLineEdgeOcsPoint { x: end_x, y: end_y },
                })
            }
            _ => Err(
                DxfHatchBoundaryLineEdgeEndpointIssue::CoordinatesUnavailable(
                    DxfHatchBoundaryLineEdgeUnavailableCoordinates { mask },
                ),
            ),
        }
    }
}

/// Required OCS endpoint semantics retaining all M14.4w numeric evidence.
#[derive(Debug)]
pub struct DxfHatchBoundaryLineEdgeCoordinateDirectory {
    source_id: DxfSourceId,
    numerics: DxfHatchBoundaryLineEdgeNumericDirectory,
    entries: Box<[DxfHatchBoundaryLineEdgeCoordinateEntry]>,
}

impl DxfHatchBoundaryLineEdgeCoordinateDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let numerics = document.hatch_boundary_line_edge_numeric_directory(cancellation)?;
        ensure_source(document.source_id(), numerics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(numerics.entries().len())
            .map_err(|_| out_of_memory())?;
        for numeric in numerics.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let components = numeric.components();
            entries.push(DxfHatchBoundaryLineEdgeCoordinateEntry {
                ordinal: compact_len(entries.len())?,
                numeric,
                coordinates: DxfHatchBoundaryLineEdgeCoordinates {
                    start_x: required(components.start_x())?,
                    start_y: required(components.start_y())?,
                    end_x: required(components.end_x())?,
                    end_y: required(components.end_y())?,
                },
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            numerics,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn numeric_directory(&self) -> &DxfHatchBoundaryLineEdgeNumericDirectory {
        &self.numerics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryLineEdgeCoordinateEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryLineEdgeCoordinateEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryLineEdgeCoordinateEntry> {
        self.numerics.entry_for_edge(edge_ordinal)?;
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| entry.numeric().edge_ordinal())
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchBoundaryLineEdgeCoordinateEntry]> {
        self.numerics.entries_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.numeric().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.numeric().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_line_edge_coordinate_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCoordinateDirectory, DxfError> {
        DxfHatchBoundaryLineEdgeCoordinateDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_line_edge_coordinate_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCoordinateDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_coordinate_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_line_edge_coordinate_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryLineEdgeCoordinateDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_line_edge_coordinate_directory(cancellation)
    }
}

fn required(
    source: &DxfHatchBoundaryLineEdgeNumericValue,
) -> Result<DxfHatchBoundaryLineEdgeCoordinateValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => {
            DxfSemanticValue::explicit(value, field, raw)
        }
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundaryLineEdgeCoordinateIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundaryLineEdgeCoordinateIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

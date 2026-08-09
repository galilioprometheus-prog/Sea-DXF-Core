//! Required fields and reviewed domains for HATCH boundary CircularArc edges.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryCircularArcEdgeNumericDirectory,
    DxfHatchBoundaryCircularArcEdgeNumericDoubleValue, DxfHatchBoundaryCircularArcEdgeNumericEntry,
    DxfHatchBoundaryCircularArcEdgeNumericIntegerValue,
    DxfHatchBoundaryCircularArcEdgeNumericIssue, DxfRawDocumentView, DxfSemanticValue, DxfSourceId,
    read_support::{
        compact_len, ensure_not_cancelled, ensure_source, invalid_internal_data, out_of_memory,
    },
};

/// Reviewed meaning of the exact HATCH CircularArc group-73 flag.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryCircularArcEdgeDirection {
    Clockwise,
    Counterclockwise,
}

impl DxfHatchBoundaryCircularArcEdgeDirection {
    #[must_use]
    pub const fn flag(self) -> i16 {
        match self {
            Self::Clockwise => 0,
            Self::Counterclockwise => 1,
        }
    }

    #[must_use]
    pub const fn is_counterclockwise(self) -> bool {
        matches!(self, Self::Counterclockwise)
    }
}

/// Why one required CircularArc semantic value is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryCircularArcEdgeSemanticIssue {
    MissingRequiredValue,
    Numeric(DxfHatchBoundaryCircularArcEdgeNumericIssue),
    NonPositiveRadius { radius: DxfDouble },
    DirectionFlagOutOfDomain { value: i16 },
}

pub type DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue =
    DxfSemanticValue<DxfDouble, DxfHatchBoundaryCircularArcEdgeSemanticIssue>;

pub type DxfHatchBoundaryCircularArcEdgeSemanticDirectionValue = DxfSemanticValue<
    DxfHatchBoundaryCircularArcEdgeDirection,
    DxfHatchBoundaryCircularArcEdgeSemanticIssue,
>;

/// Required source-anchored fields for one CircularArc edge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeSemantics {
    center_x: DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue,
    center_y: DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue,
    radius: DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue,
    start_angle_degrees: DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue,
    end_angle_degrees: DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue,
    direction: DxfHatchBoundaryCircularArcEdgeSemanticDirectionValue,
}

impl DxfHatchBoundaryCircularArcEdgeSemantics {
    #[must_use]
    pub const fn center_x(&self) -> &DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue {
        &self.center_x
    }

    #[must_use]
    pub const fn center_y(&self) -> &DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue {
        &self.center_y
    }

    #[must_use]
    pub const fn radius(&self) -> &DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue {
        &self.radius
    }

    #[must_use]
    pub const fn start_angle_degrees(&self) -> &DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue {
        &self.start_angle_degrees
    }

    #[must_use]
    pub const fn end_angle_degrees(&self) -> &DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue {
        &self.end_angle_degrees
    }

    #[must_use]
    pub const fn direction(&self) -> &DxfHatchBoundaryCircularArcEdgeSemanticDirectionValue {
        &self.direction
    }

    #[must_use]
    pub fn ocs_center_value(&self) -> Option<[DxfDouble; 2]> {
        Some([
            self.center_x.value().copied()?,
            self.center_y.value().copied()?,
        ])
    }

    #[must_use]
    pub fn radius_value(&self) -> Option<DxfDouble> {
        self.radius.value().copied()
    }

    #[must_use]
    pub fn start_angle_degrees_value(&self) -> Option<DxfDouble> {
        self.start_angle_degrees.value().copied()
    }

    #[must_use]
    pub fn end_angle_degrees_value(&self) -> Option<DxfDouble> {
        self.end_angle_degrees.value().copied()
    }

    #[must_use]
    pub fn direction_value(&self) -> Option<DxfHatchBoundaryCircularArcEdgeDirection> {
        self.direction.value().copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryCircularArcEdgeSemanticEntry {
    ordinal: u32,
    numeric: DxfHatchBoundaryCircularArcEdgeNumericEntry,
    semantics: DxfHatchBoundaryCircularArcEdgeSemantics,
}

impl DxfHatchBoundaryCircularArcEdgeSemanticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn numeric(self) -> DxfHatchBoundaryCircularArcEdgeNumericEntry {
        self.numeric
    }

    #[must_use]
    pub const fn semantics(&self) -> &DxfHatchBoundaryCircularArcEdgeSemantics {
        &self.semantics
    }
}

/// Required and domain-checked semantics retaining the complete numeric evidence.
#[derive(Debug)]
pub struct DxfHatchBoundaryCircularArcEdgeSemanticDirectory {
    source_id: DxfSourceId,
    numerics: DxfHatchBoundaryCircularArcEdgeNumericDirectory,
    entries: Box<[DxfHatchBoundaryCircularArcEdgeSemanticEntry]>,
}

impl DxfHatchBoundaryCircularArcEdgeSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let numerics = document.hatch_boundary_circular_arc_edge_numeric_directory(cancellation)?;
        ensure_source(document.source_id(), numerics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(numerics.entries().len())
            .map_err(|_| out_of_memory())?;
        for numeric in numerics.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let components = numeric.components();
            entries.push(DxfHatchBoundaryCircularArcEdgeSemanticEntry {
                ordinal: compact_len(entries.len())?,
                numeric,
                semantics: DxfHatchBoundaryCircularArcEdgeSemantics {
                    center_x: required_double(components.center_x())?,
                    center_y: required_double(components.center_y())?,
                    radius: positive_radius(components.radius())?,
                    start_angle_degrees: required_double(components.start_angle())?,
                    end_angle_degrees: required_double(components.end_angle())?,
                    direction: required_direction(components.counterclockwise())?,
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
    pub const fn numeric_directory(&self) -> &DxfHatchBoundaryCircularArcEdgeNumericDirectory {
        &self.numerics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryCircularArcEdgeSemanticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryCircularArcEdgeSemanticEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryCircularArcEdgeSemanticEntry> {
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
    ) -> Option<&[DxfHatchBoundaryCircularArcEdgeSemanticEntry]> {
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
    pub fn hatch_boundary_circular_arc_edge_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeSemanticDirectory, DxfError> {
        DxfHatchBoundaryCircularArcEdgeSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_circular_arc_edge_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryCircularArcEdgeSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_circular_arc_edge_semantic_directory(cancellation)
    }
}

fn required_double(
    source: &DxfHatchBoundaryCircularArcEdgeNumericDoubleValue,
) -> Result<DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => {
            DxfSemanticValue::explicit(value, field, raw)
        }
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundaryCircularArcEdgeSemanticIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundaryCircularArcEdgeSemanticIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

fn positive_radius(
    source: &DxfHatchBoundaryCircularArcEdgeNumericDoubleValue,
) -> Result<DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue, DxfError> {
    Ok(match required_double(source)? {
        DxfSemanticValue::Explicit { value, field, raw } if value.to_f64() <= 0.0 => {
            DxfSemanticValue::invalid(
                DxfHatchBoundaryCircularArcEdgeSemanticIssue::NonPositiveRadius { radius: value },
                field,
                Some(raw),
            )
        }
        value => value,
    })
}

fn required_direction(
    source: &DxfHatchBoundaryCircularArcEdgeNumericIntegerValue,
) -> Result<DxfHatchBoundaryCircularArcEdgeSemanticDirectionValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => match value {
            0 => DxfSemanticValue::explicit(
                DxfHatchBoundaryCircularArcEdgeDirection::Clockwise,
                field,
                raw,
            ),
            1 => DxfSemanticValue::explicit(
                DxfHatchBoundaryCircularArcEdgeDirection::Counterclockwise,
                field,
                raw,
            ),
            value => DxfSemanticValue::invalid(
                DxfHatchBoundaryCircularArcEdgeSemanticIssue::DirectionFlagOutOfDomain { value },
                field,
                Some(raw),
            ),
        },
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundaryCircularArcEdgeSemanticIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundaryCircularArcEdgeSemanticIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

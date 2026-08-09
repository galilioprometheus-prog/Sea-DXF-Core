//! Required fields and reviewed domains for HATCH boundary EllipticArc edges.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryEllipticArcEdgeNumericDirectory,
    DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue, DxfHatchBoundaryEllipticArcEdgeNumericEntry,
    DxfHatchBoundaryEllipticArcEdgeNumericIntegerValue,
    DxfHatchBoundaryEllipticArcEdgeNumericIssue, DxfRawDocumentView, DxfSemanticValue, DxfSourceId,
    read_support::{
        compact_len, ensure_not_cancelled, ensure_source, invalid_internal_data, out_of_memory,
    },
};

const MIN_RADIUS_RATIO: f64 = 1.0e-6;
const MAX_RADIUS_RATIO: f64 = 1.0;
const MIN_MAJOR_AXIS_LENGTH_SQUARED: f64 = 1.0e-12;

/// Reviewed meaning of the exact HATCH EllipticArc group-73 flag.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEllipticArcEdgeDirection {
    Clockwise,
    Counterclockwise,
}

impl DxfHatchBoundaryEllipticArcEdgeDirection {
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

/// Why one required EllipticArc semantic value is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEllipticArcEdgeSemanticIssue {
    MissingRequiredValue,
    Numeric(DxfHatchBoundaryEllipticArcEdgeNumericIssue),
    MinorToMajorRatioOutOfDomain { ratio: DxfDouble },
    DirectionFlagOutOfDomain { value: i16 },
}

pub type DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue =
    DxfSemanticValue<DxfDouble, DxfHatchBoundaryEllipticArcEdgeSemanticIssue>;

pub type DxfHatchBoundaryEllipticArcEdgeSemanticDirectionValue = DxfSemanticValue<
    DxfHatchBoundaryEllipticArcEdgeDirection,
    DxfHatchBoundaryEllipticArcEdgeSemanticIssue,
>;

/// Exact unavailable-component mask for the required OCS major-axis vector.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeMajorAxisUnavailableComponents {
    mask: u8,
}

impl DxfHatchBoundaryEllipticArcEdgeMajorAxisUnavailableComponents {
    const X: u8 = 1;
    const Y: u8 = 2;

    #[must_use]
    pub const fn x(self) -> bool {
        self.mask & Self::X != 0
    }

    #[must_use]
    pub const fn y(self) -> bool {
        self.mask & Self::Y != 0
    }

    #[must_use]
    pub const fn count(self) -> u32 {
        self.mask.count_ones()
    }
}

/// Reviewed domain state of the two-component OCS major-axis vector.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEllipticArcEdgeMajorAxisState {
    Usable,
    ComponentsUnavailable(DxfHatchBoundaryEllipticArcEdgeMajorAxisUnavailableComponents),
    Degenerate,
    NonFiniteDerivedMagnitude,
}

/// Required source-anchored fields for one EllipticArc edge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeSemantics {
    center_x: DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    center_y: DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    major_axis_endpoint_x: DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    major_axis_endpoint_y: DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    minor_to_major_ratio: DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    start_angle_degrees: DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    end_angle_degrees: DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    direction: DxfHatchBoundaryEllipticArcEdgeSemanticDirectionValue,
    major_axis_state: DxfHatchBoundaryEllipticArcEdgeMajorAxisState,
}

impl DxfHatchBoundaryEllipticArcEdgeSemantics {
    #[must_use]
    pub const fn center_x(&self) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue {
        &self.center_x
    }

    #[must_use]
    pub const fn center_y(&self) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue {
        &self.center_y
    }

    #[must_use]
    pub const fn major_axis_endpoint_x(
        &self,
    ) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue {
        &self.major_axis_endpoint_x
    }

    #[must_use]
    pub const fn major_axis_endpoint_y(
        &self,
    ) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue {
        &self.major_axis_endpoint_y
    }

    #[must_use]
    pub const fn minor_to_major_ratio(
        &self,
    ) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue {
        &self.minor_to_major_ratio
    }

    #[must_use]
    pub const fn start_angle_degrees(&self) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue {
        &self.start_angle_degrees
    }

    #[must_use]
    pub const fn end_angle_degrees(&self) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue {
        &self.end_angle_degrees
    }

    #[must_use]
    pub const fn direction(&self) -> &DxfHatchBoundaryEllipticArcEdgeSemanticDirectionValue {
        &self.direction
    }

    #[must_use]
    pub const fn major_axis_state(&self) -> DxfHatchBoundaryEllipticArcEdgeMajorAxisState {
        self.major_axis_state
    }

    #[must_use]
    pub fn ocs_center_value(&self) -> Option<[DxfDouble; 2]> {
        Some([
            self.center_x.value().copied()?,
            self.center_y.value().copied()?,
        ])
    }

    #[must_use]
    pub fn ocs_major_axis_endpoint_value(&self) -> Option<[DxfDouble; 2]> {
        if self.major_axis_state != DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Usable {
            return None;
        }
        Some([
            self.major_axis_endpoint_x.value().copied()?,
            self.major_axis_endpoint_y.value().copied()?,
        ])
    }

    #[must_use]
    pub fn minor_to_major_ratio_value(&self) -> Option<DxfDouble> {
        self.minor_to_major_ratio.value().copied()
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
    pub fn direction_value(&self) -> Option<DxfHatchBoundaryEllipticArcEdgeDirection> {
        self.direction.value().copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEllipticArcEdgeSemanticEntry {
    ordinal: u32,
    numeric: DxfHatchBoundaryEllipticArcEdgeNumericEntry,
    semantics: DxfHatchBoundaryEllipticArcEdgeSemantics,
}

impl DxfHatchBoundaryEllipticArcEdgeSemanticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn numeric(self) -> DxfHatchBoundaryEllipticArcEdgeNumericEntry {
        self.numeric
    }

    #[must_use]
    pub const fn semantics(&self) -> &DxfHatchBoundaryEllipticArcEdgeSemantics {
        &self.semantics
    }
}

/// Required and domain-checked semantics retaining the complete numeric evidence.
#[derive(Debug)]
pub struct DxfHatchBoundaryEllipticArcEdgeSemanticDirectory {
    source_id: DxfSourceId,
    numerics: DxfHatchBoundaryEllipticArcEdgeNumericDirectory,
    entries: Box<[DxfHatchBoundaryEllipticArcEdgeSemanticEntry]>,
}

impl DxfHatchBoundaryEllipticArcEdgeSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let numerics = document.hatch_boundary_elliptic_arc_edge_numeric_directory(cancellation)?;
        ensure_source(document.source_id(), numerics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(numerics.entries().len())
            .map_err(|_| out_of_memory())?;
        for numeric in numerics.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let components = numeric.components();
            let major_axis_endpoint_x = required_double(components.major_axis_endpoint_x())?;
            let major_axis_endpoint_y = required_double(components.major_axis_endpoint_y())?;
            let major_axis_state = major_axis_state(&major_axis_endpoint_x, &major_axis_endpoint_y);
            entries.push(DxfHatchBoundaryEllipticArcEdgeSemanticEntry {
                ordinal: compact_len(entries.len())?,
                numeric,
                semantics: DxfHatchBoundaryEllipticArcEdgeSemantics {
                    center_x: required_double(components.center_x())?,
                    center_y: required_double(components.center_y())?,
                    major_axis_endpoint_x,
                    major_axis_endpoint_y,
                    minor_to_major_ratio: radius_ratio(components.minor_to_major_ratio())?,
                    start_angle_degrees: required_double(components.start_angle())?,
                    end_angle_degrees: required_double(components.end_angle())?,
                    direction: required_direction(components.counterclockwise())?,
                    major_axis_state,
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
    pub const fn numeric_directory(&self) -> &DxfHatchBoundaryEllipticArcEdgeNumericDirectory {
        &self.numerics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryEllipticArcEdgeSemanticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryEllipticArcEdgeSemanticEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryEllipticArcEdgeSemanticEntry> {
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
    ) -> Option<&[DxfHatchBoundaryEllipticArcEdgeSemanticEntry]> {
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
    pub fn hatch_boundary_elliptic_arc_edge_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeSemanticDirectory, DxfError> {
        DxfHatchBoundaryEllipticArcEdgeSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_elliptic_arc_edge_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEllipticArcEdgeSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_elliptic_arc_edge_semantic_directory(cancellation)
    }
}

fn required_double(
    source: &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
) -> Result<DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => {
            DxfSemanticValue::explicit(value, field, raw)
        }
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

fn radius_ratio(
    source: &DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
) -> Result<DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue, DxfError> {
    Ok(match required_double(source)? {
        DxfSemanticValue::Explicit { value, field, raw }
            if !(MIN_RADIUS_RATIO..=MAX_RADIUS_RATIO).contains(&value.to_f64()) =>
        {
            DxfSemanticValue::invalid(
                DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MinorToMajorRatioOutOfDomain {
                    ratio: value,
                },
                field,
                Some(raw),
            )
        }
        value => value,
    })
}

fn required_direction(
    source: &DxfHatchBoundaryEllipticArcEdgeNumericIntegerValue,
) -> Result<DxfHatchBoundaryEllipticArcEdgeSemanticDirectionValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => match value {
            0 => DxfSemanticValue::explicit(
                DxfHatchBoundaryEllipticArcEdgeDirection::Clockwise,
                field,
                raw,
            ),
            1 => DxfSemanticValue::explicit(
                DxfHatchBoundaryEllipticArcEdgeDirection::Counterclockwise,
                field,
                raw,
            ),
            value => DxfSemanticValue::invalid(
                DxfHatchBoundaryEllipticArcEdgeSemanticIssue::DirectionFlagOutOfDomain { value },
                field,
                Some(raw),
            ),
        },
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

fn major_axis_state(
    x: &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    y: &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
) -> DxfHatchBoundaryEllipticArcEdgeMajorAxisState {
    let (x, y) = match (x.value().copied(), y.value().copied()) {
        (Some(x), Some(y)) => (x.to_f64(), y.to_f64()),
        (x, y) => {
            let mut mask = 0_u8;
            if x.is_none() {
                mask |= DxfHatchBoundaryEllipticArcEdgeMajorAxisUnavailableComponents::X;
            }
            if y.is_none() {
                mask |= DxfHatchBoundaryEllipticArcEdgeMajorAxisUnavailableComponents::Y;
            }
            return DxfHatchBoundaryEllipticArcEdgeMajorAxisState::ComponentsUnavailable(
                DxfHatchBoundaryEllipticArcEdgeMajorAxisUnavailableComponents { mask },
            );
        }
    };
    let squared_length = x * x + y * y;
    if !squared_length.is_finite() {
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::NonFiniteDerivedMagnitude
    } else if squared_length <= MIN_MAJOR_AXIS_LENGTH_SQUARED {
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Degenerate
    } else {
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Usable
    }
}

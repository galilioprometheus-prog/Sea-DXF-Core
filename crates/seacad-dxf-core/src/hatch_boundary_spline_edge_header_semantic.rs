//! Required fields and reviewed Boolean domains for HATCH boundary Spline-edge headers.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHatchBoundarySplineEdgeHeaderNumericDirectory, DxfHatchBoundarySplineEdgeHeaderNumericEntry,
    DxfHatchBoundarySplineEdgeHeaderNumericI16Value,
    DxfHatchBoundarySplineEdgeHeaderNumericI32Value, DxfHatchBoundarySplineEdgeHeaderNumericIssue,
    DxfRawDocumentView, DxfSemanticValue, DxfSourceId,
    read_support::{
        compact_len, ensure_not_cancelled, ensure_source, invalid_internal_data, out_of_memory,
    },
};

/// Reviewed meaning of the exact HATCH Spline-edge group-73 value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgeHeaderRationality {
    NonRational,
    Rational,
}

impl DxfHatchBoundarySplineEdgeHeaderRationality {
    #[must_use]
    pub const fn flag(self) -> i16 {
        match self {
            Self::NonRational => 0,
            Self::Rational => 1,
        }
    }

    #[must_use]
    pub const fn is_rational(self) -> bool {
        matches!(self, Self::Rational)
    }
}

/// Reviewed meaning of the exact HATCH Spline-edge group-74 value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgeHeaderPeriodicity {
    NonPeriodic,
    Periodic,
}

impl DxfHatchBoundarySplineEdgeHeaderPeriodicity {
    #[must_use]
    pub const fn flag(self) -> i16 {
        match self {
            Self::NonPeriodic => 0,
            Self::Periodic => 1,
        }
    }

    #[must_use]
    pub const fn is_periodic(self) -> bool {
        matches!(self, Self::Periodic)
    }
}

/// Why one required Spline-edge header semantic is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundarySplineEdgeHeaderSemanticIssue {
    MissingRequiredValue,
    Numeric(DxfHatchBoundarySplineEdgeHeaderNumericIssue),
    RationalFlagOutOfDomain { value: i16 },
    PeriodicFlagOutOfDomain { value: i16 },
}

pub type DxfHatchBoundarySplineEdgeHeaderSemanticI32Value =
    DxfSemanticValue<i32, DxfHatchBoundarySplineEdgeHeaderSemanticIssue>;

pub type DxfHatchBoundarySplineEdgeHeaderSemanticRationalityValue = DxfSemanticValue<
    DxfHatchBoundarySplineEdgeHeaderRationality,
    DxfHatchBoundarySplineEdgeHeaderSemanticIssue,
>;

pub type DxfHatchBoundarySplineEdgeHeaderSemanticPeriodicityValue = DxfSemanticValue<
    DxfHatchBoundarySplineEdgeHeaderPeriodicity,
    DxfHatchBoundarySplineEdgeHeaderSemanticIssue,
>;

/// Required source-anchored fields for one HATCH boundary Spline edge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgeHeaderSemantics {
    degree: DxfHatchBoundarySplineEdgeHeaderSemanticI32Value,
    rationality: DxfHatchBoundarySplineEdgeHeaderSemanticRationalityValue,
    periodicity: DxfHatchBoundarySplineEdgeHeaderSemanticPeriodicityValue,
    knot_count: DxfHatchBoundarySplineEdgeHeaderSemanticI32Value,
    control_point_count: DxfHatchBoundarySplineEdgeHeaderSemanticI32Value,
}

impl DxfHatchBoundarySplineEdgeHeaderSemantics {
    #[must_use]
    pub const fn degree(&self) -> &DxfHatchBoundarySplineEdgeHeaderSemanticI32Value {
        &self.degree
    }

    #[must_use]
    pub const fn rationality(&self) -> &DxfHatchBoundarySplineEdgeHeaderSemanticRationalityValue {
        &self.rationality
    }

    #[must_use]
    pub const fn periodicity(&self) -> &DxfHatchBoundarySplineEdgeHeaderSemanticPeriodicityValue {
        &self.periodicity
    }

    #[must_use]
    pub const fn knot_count(&self) -> &DxfHatchBoundarySplineEdgeHeaderSemanticI32Value {
        &self.knot_count
    }

    #[must_use]
    pub const fn control_point_count(&self) -> &DxfHatchBoundarySplineEdgeHeaderSemanticI32Value {
        &self.control_point_count
    }

    #[must_use]
    pub fn degree_value(&self) -> Option<i32> {
        self.degree.value().copied()
    }

    #[must_use]
    pub fn rationality_value(&self) -> Option<DxfHatchBoundarySplineEdgeHeaderRationality> {
        self.rationality.value().copied()
    }

    #[must_use]
    pub fn periodicity_value(&self) -> Option<DxfHatchBoundarySplineEdgeHeaderPeriodicity> {
        self.periodicity.value().copied()
    }

    #[must_use]
    pub fn knot_count_value(&self) -> Option<i32> {
        self.knot_count.value().copied()
    }

    #[must_use]
    pub fn control_point_count_value(&self) -> Option<i32> {
        self.control_point_count.value().copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundarySplineEdgeHeaderSemanticEntry {
    ordinal: u32,
    numeric: DxfHatchBoundarySplineEdgeHeaderNumericEntry,
    semantics: DxfHatchBoundarySplineEdgeHeaderSemantics,
}

impl DxfHatchBoundarySplineEdgeHeaderSemanticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn numeric(self) -> DxfHatchBoundarySplineEdgeHeaderNumericEntry {
        self.numeric
    }

    #[must_use]
    pub const fn semantics(&self) -> &DxfHatchBoundarySplineEdgeHeaderSemantics {
        &self.semantics
    }
}

/// Required and Boolean-domain-checked semantics retaining all numeric evidence.
#[derive(Debug)]
pub struct DxfHatchBoundarySplineEdgeHeaderSemanticDirectory {
    source_id: DxfSourceId,
    numerics: DxfHatchBoundarySplineEdgeHeaderNumericDirectory,
    entries: Box<[DxfHatchBoundarySplineEdgeHeaderSemanticEntry]>,
}

impl DxfHatchBoundarySplineEdgeHeaderSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let numerics =
            document.hatch_boundary_spline_edge_header_numeric_directory(cancellation)?;
        ensure_source(document.source_id(), numerics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(numerics.entries().len())
            .map_err(|_| out_of_memory())?;
        for numeric in numerics.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let components = numeric.components();
            entries.push(DxfHatchBoundarySplineEdgeHeaderSemanticEntry {
                ordinal: compact_len(entries.len())?,
                numeric,
                semantics: DxfHatchBoundarySplineEdgeHeaderSemantics {
                    degree: required_i32(components.degree())?,
                    rationality: required_rationality(components.rational())?,
                    periodicity: required_periodicity(components.periodic())?,
                    knot_count: required_i32(components.knot_count())?,
                    control_point_count: required_i32(components.control_point_count())?,
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
    pub const fn numeric_directory(&self) -> &DxfHatchBoundarySplineEdgeHeaderNumericDirectory {
        &self.numerics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundarySplineEdgeHeaderSemanticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundarySplineEdgeHeaderSemanticEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundarySplineEdgeHeaderSemanticEntry> {
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
    ) -> Option<&[DxfHatchBoundarySplineEdgeHeaderSemanticEntry]> {
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
    pub fn hatch_boundary_spline_edge_header_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderSemanticDirectory, DxfError> {
        DxfHatchBoundarySplineEdgeHeaderSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_header_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_header_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_spline_edge_header_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundarySplineEdgeHeaderSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .hatch_boundary_spline_edge_header_semantic_directory(cancellation)
    }
}

fn required_i32(
    source: &DxfHatchBoundarySplineEdgeHeaderNumericI32Value,
) -> Result<DxfHatchBoundarySplineEdgeHeaderSemanticI32Value, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => {
            DxfSemanticValue::explicit(value, field, raw)
        }
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgeHeaderSemanticIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgeHeaderSemanticIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

fn required_rationality(
    source: &DxfHatchBoundarySplineEdgeHeaderNumericI16Value,
) -> Result<DxfHatchBoundarySplineEdgeHeaderSemanticRationalityValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => match value {
            0 => DxfSemanticValue::explicit(
                DxfHatchBoundarySplineEdgeHeaderRationality::NonRational,
                field,
                raw,
            ),
            1 => DxfSemanticValue::explicit(
                DxfHatchBoundarySplineEdgeHeaderRationality::Rational,
                field,
                raw,
            ),
            value => DxfSemanticValue::invalid(
                DxfHatchBoundarySplineEdgeHeaderSemanticIssue::RationalFlagOutOfDomain { value },
                field,
                Some(raw),
            ),
        },
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgeHeaderSemanticIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgeHeaderSemanticIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

fn required_periodicity(
    source: &DxfHatchBoundarySplineEdgeHeaderNumericI16Value,
) -> Result<DxfHatchBoundarySplineEdgeHeaderSemanticPeriodicityValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => match value {
            0 => DxfSemanticValue::explicit(
                DxfHatchBoundarySplineEdgeHeaderPeriodicity::NonPeriodic,
                field,
                raw,
            ),
            1 => DxfSemanticValue::explicit(
                DxfHatchBoundarySplineEdgeHeaderPeriodicity::Periodic,
                field,
                raw,
            ),
            value => DxfSemanticValue::invalid(
                DxfHatchBoundarySplineEdgeHeaderSemanticIssue::PeriodicFlagOutOfDomain { value },
                field,
                Some(raw),
            ),
        },
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgeHeaderSemanticIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchBoundarySplineEdgeHeaderSemanticIssue::MissingRequiredValue,
            field,
            None,
        ),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
}

//! Complete analytic NURBS readiness for SPLINE records.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId, DxfSplineAnalyticValueDirectory,
    DxfSplineAnalyticValueRange, DxfSplineAnalyticValueState, DxfSplineCountDirectory,
    DxfSplineCountDisposition, DxfSplineCountKind, DxfSplineCountState,
    DxfSplineDegreeControlRelation, DxfSplineDegreeState, DxfSplineFlags, DxfSplineFlagsSemantic,
    DxfSplineInvariantDisposition, DxfSplineKnotOrderState, DxfSplineLinearPlanarRelation,
    DxfSplineNurbsCountRelation, DxfSplinePlanarNormalRelation, DxfSplinePointKind,
    DxfSplineRecordEntry, DxfSplineTopologyDirectory, DxfSplineVectorKind,
    DxfSplineVectorSemanticState,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineAnalyticIssueKind {
    Values,
    Flags,
    LinearPlanar,
    Degree,
    KnotCount,
    ControlPointCount,
    FitPointCount,
    KnotOrder,
    DegreeControl,
    NurbsCount,
    KnotMultiplicity,
    ParameterDomain,
    OptionalVector,
    Normal,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticIssues {
    bits: u16,
}

impl DxfSplineAnalyticIssues {
    #[must_use]
    pub const fn bits(self) -> u16 {
        self.bits
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    #[must_use]
    pub const fn contains(self, kind: DxfSplineAnalyticIssueKind) -> bool {
        self.bits & issue_bit(kind) != 0
    }

    fn insert(&mut self, kind: DxfSplineAnalyticIssueKind) {
        self.bits |= issue_bit(kind);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticData {
    pub degree: u16,
    pub flags: DxfSplineFlags,
    pub parameter_start: DxfDouble,
    pub parameter_end: DxfDouble,
    pub knot_range: DxfSplineAnalyticValueRange,
    pub control_point_range: DxfSplineAnalyticValueRange,
    pub fit_point_range: DxfSplineAnalyticValueRange,
    pub start_tangent: DxfSplineVectorSemanticState,
    pub end_tangent: DxfSplineVectorSemanticState,
    pub normal: DxfSplineVectorSemanticState,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineAnalyticState {
    Available(DxfSplineAnalyticData),
    Unavailable(DxfSplineAnalyticIssues),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticEntry {
    pub ordinal: u32,
    pub record: DxfSplineRecordEntry,
    pub state: DxfSplineAnalyticState,
}

#[derive(Debug)]
pub struct DxfSplineAnalyticDirectory {
    source_id: DxfSourceId,
    values: DxfSplineAnalyticValueDirectory,
    topology: DxfSplineTopologyDirectory,
    counts: DxfSplineCountDirectory,
    entries: Box<[DxfSplineAnalyticEntry]>,
}

impl DxfSplineAnalyticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let values = document.spline_analytic_value_directory(cancellation)?;
        let topology = document.spline_topology_directory(cancellation)?;
        let counts = document.spline_count_directory(cancellation)?;
        for observed in [values.source_id(), topology.source_id(), counts.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }
        if values.entries().len() != topology.entries().len() {
            return Err(invalid_internal_data());
        }
        let mut entries = Vec::new();
        for topology_entry in topology.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw = topology_entry.record().record().ordinal();
            let value_entry = values
                .entry_for_raw_record(raw)
                .ok_or_else(invalid_internal_data)?;
            let state = analytic_state(&values, &topology, &counts, value_entry, topology_entry)?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfSplineAnalyticEntry {
                ordinal: compact_len(entries.len())?,
                record: topology_entry.record(),
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            values,
            topology,
            counts,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn value_directory(&self) -> &DxfSplineAnalyticValueDirectory {
        &self.values
    }

    #[must_use]
    pub const fn topology_directory(&self) -> &DxfSplineTopologyDirectory {
        &self.topology
    }

    #[must_use]
    pub const fn count_directory(&self) -> &DxfSplineCountDirectory {
        &self.counts
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfSplineAnalyticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfSplineAnalyticEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_raw_record(&self, raw: u64) -> Option<DxfSplineAnalyticEntry> {
        self.entries
            .binary_search_by_key(&raw, |entry| entry.record.record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_analytic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAnalyticDirectory, DxfError> {
        DxfSplineAnalyticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_analytic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAnalyticDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_analytic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_analytic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAnalyticDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_analytic_directory(cancellation)
    }
}

fn analytic_state(
    values: &DxfSplineAnalyticValueDirectory,
    topology: &DxfSplineTopologyDirectory,
    counts: &DxfSplineCountDirectory,
    value_entry: crate::DxfSplineAnalyticValueEntry,
    topology_entry: crate::DxfSplineTopologyEntry,
) -> Result<DxfSplineAnalyticState, DxfError> {
    let raw = topology_entry.record().record().ordinal();
    let relation = topology
        .relation_directory()
        .entry_for_raw_record(raw)
        .ok_or_else(invalid_internal_data)?;
    let mut issues = DxfSplineAnalyticIssues::default();
    let ranges = value_ranges(value_entry.state, &mut issues);
    let flags = match relation.flags() {
        DxfSplineFlagsSemantic::Explicit(flags) => Some(flags),
        _ => {
            issues.insert(DxfSplineAnalyticIssueKind::Flags);
            None
        }
    };
    if relation.linear_planar() == DxfSplineLinearPlanarRelation::LinearWithoutPlanar {
        issues.insert(DxfSplineAnalyticIssueKind::LinearPlanar);
    }
    let degree = match topology_entry.degree() {
        DxfSplineDegreeState::Explicit { degree, .. } => Some(degree),
        _ => {
            issues.insert(DxfSplineAnalyticIssueKind::Degree);
            None
        }
    };
    check_count(
        counts,
        raw,
        DxfSplineCountKind::Knot,
        false,
        DxfSplineAnalyticIssueKind::KnotCount,
        &mut issues,
    )?;
    check_count(
        counts,
        raw,
        DxfSplineCountKind::ControlPoint,
        false,
        DxfSplineAnalyticIssueKind::ControlPointCount,
        &mut issues,
    )?;
    let fit_absent = fit_anchor_count(values, raw)? == 0;
    check_count(
        counts,
        raw,
        DxfSplineCountKind::FitPoint,
        fit_absent,
        DxfSplineAnalyticIssueKind::FitPointCount,
        &mut issues,
    )?;
    if !matches!(
        topology_entry.knot_order(),
        DxfSplineKnotOrderState::Nondecreasing { .. }
    ) {
        issues.insert(DxfSplineAnalyticIssueKind::KnotOrder);
    }
    if !matches!(
        topology_entry.degree_control(),
        DxfSplineDegreeControlRelation::Compared {
            disposition: DxfSplineInvariantDisposition::Satisfied,
            ..
        }
    ) {
        issues.insert(DxfSplineAnalyticIssueKind::DegreeControl);
    }
    if !matches!(
        topology_entry.nurbs_count(),
        DxfSplineNurbsCountRelation::Compared {
            disposition: DxfSplineInvariantDisposition::Satisfied,
            ..
        }
    ) {
        issues.insert(DxfSplineAnalyticIssueKind::NurbsCount);
    }
    let vectors = vectors(topology, raw, &mut issues)?;
    check_normal(relation.planar_normal(), &mut issues);
    let parameter = match (ranges, degree) {
        (Some((knots, controls, _)), Some(degree)) => {
            knot_shape(values, knots, controls, degree, &mut issues)?
        }
        _ => None,
    };
    if !issues.is_empty() {
        return Ok(DxfSplineAnalyticState::Unavailable(issues));
    }
    let (knot_range, control_point_range, fit_point_range) =
        ranges.ok_or_else(invalid_internal_data)?;
    let (parameter_start, parameter_end) = parameter.ok_or_else(invalid_internal_data)?;
    Ok(DxfSplineAnalyticState::Available(DxfSplineAnalyticData {
        degree: degree.ok_or_else(invalid_internal_data)?,
        flags: flags.ok_or_else(invalid_internal_data)?,
        parameter_start,
        parameter_end,
        knot_range,
        control_point_range,
        fit_point_range,
        start_tangent: vectors[0],
        end_tangent: vectors[1],
        normal: vectors[2],
    }))
}

type ValueRanges = (
    DxfSplineAnalyticValueRange,
    DxfSplineAnalyticValueRange,
    DxfSplineAnalyticValueRange,
);

fn value_ranges(
    state: DxfSplineAnalyticValueState,
    issues: &mut DxfSplineAnalyticIssues,
) -> Option<ValueRanges> {
    match state {
        DxfSplineAnalyticValueState::Available {
            knot_range,
            control_point_range,
            fit_point_range,
        } => Some((knot_range, control_point_range, fit_point_range)),
        DxfSplineAnalyticValueState::Unavailable(_) => {
            issues.insert(DxfSplineAnalyticIssueKind::Values);
            None
        }
    }
}

fn check_count(
    counts: &DxfSplineCountDirectory,
    raw: u64,
    kind: DxfSplineCountKind,
    absent_allowed: bool,
    issue: DxfSplineAnalyticIssueKind,
    issues: &mut DxfSplineAnalyticIssues,
) -> Result<(), DxfError> {
    let state = counts
        .entry_for_kind(raw, kind)
        .ok_or_else(invalid_internal_data)?
        .state();
    let matched = matches!(
        state,
        DxfSplineCountState::Compared {
            disposition: DxfSplineCountDisposition::Matched,
            ..
        }
    );
    if !matched && !(absent_allowed && state == DxfSplineCountState::Absent) {
        issues.insert(issue);
    }
    Ok(())
}

fn fit_anchor_count(values: &DxfSplineAnalyticValueDirectory, raw: u64) -> Result<u64, DxfError> {
    values
        .semantic_directory()
        .auxiliary_directory()
        .point_directory()
        .entry_for_kind(raw, DxfSplinePointKind::FitPoint)
        .map(|entry| entry.component_counts().x())
        .ok_or_else(invalid_internal_data)
}

fn vectors(
    topology: &DxfSplineTopologyDirectory,
    raw: u64,
    issues: &mut DxfSplineAnalyticIssues,
) -> Result<[DxfSplineVectorSemanticState; 3], DxfError> {
    let semantics = topology.relation_directory().semantic_directory();
    let mut result = [DxfSplineVectorSemanticState::Absent; 3];
    for (index, kind) in [
        DxfSplineVectorKind::StartTangent,
        DxfSplineVectorKind::EndTangent,
        DxfSplineVectorKind::Normal,
    ]
    .into_iter()
    .enumerate()
    {
        result[index] = semantics
            .vector_for_kind(raw, kind)
            .ok_or_else(invalid_internal_data)?
            .state();
    }
    if result[..2]
        .iter()
        .any(|state| matches!(state, DxfSplineVectorSemanticState::Unavailable(_)))
    {
        issues.insert(DxfSplineAnalyticIssueKind::OptionalVector);
    }
    Ok(result)
}

fn check_normal(relation: DxfSplinePlanarNormalRelation, issues: &mut DxfSplineAnalyticIssues) {
    if !matches!(
        relation,
        DxfSplinePlanarNormalRelation::NonPlanarOmitted
            | DxfSplinePlanarNormalRelation::PlanarExplicit { is_zero: false }
    ) {
        issues.insert(DxfSplineAnalyticIssueKind::Normal);
    }
}

fn knot_shape(
    values: &DxfSplineAnalyticValueDirectory,
    knot_range: DxfSplineAnalyticValueRange,
    control_range: DxfSplineAnalyticValueRange,
    degree: u16,
    issues: &mut DxfSplineAnalyticIssues,
) -> Result<Option<(DxfDouble, DxfDouble)>, DxfError> {
    let knots = values.knots(knot_range).ok_or_else(invalid_internal_data)?;
    let maximum = usize::from(degree) + 1;
    let mut run = 0_usize;
    let mut previous = None;
    for knot in knots {
        let value = knot.value.to_f64();
        run = if previous == Some(value) { run + 1 } else { 1 };
        if run > maximum {
            issues.insert(DxfSplineAnalyticIssueKind::KnotMultiplicity);
        }
        previous = Some(value);
    }
    let start = knots.get(usize::from(degree)).map(|knot| knot.value);
    let control_count =
        usize::try_from(control_range.len()).map_err(|_| invalid_internal_data())?;
    let end = knots.get(control_count).map(|knot| knot.value);
    match (start, end) {
        (Some(start), Some(end)) if start.to_f64() < end.to_f64() => Ok(Some((start, end))),
        _ => {
            issues.insert(DxfSplineAnalyticIssueKind::ParameterDomain);
            Ok(None)
        }
    }
}

const fn issue_bit(kind: DxfSplineAnalyticIssueKind) -> u16 {
    1_u16 << kind as u16
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

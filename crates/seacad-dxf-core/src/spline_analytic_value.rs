//! Effective knot, weighted-control, and fit-point values for SPLINE.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId, DxfSplineAuxiliarySemanticDirectory,
    DxfSplineCardDirectory, DxfSplineCardMember, DxfSplineNumber, DxfSplineNumericIssue,
    DxfSplinePointKind, DxfSplinePointTuple, DxfSplineRecordEntry, DxfSplineValue,
    DxfSplineValueRole, DxfSplineWeightSemanticEntry, DxfSplineWeightSemanticState,
    DxfSplineWeightValueState,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineAnalyticValueIssueKind {
    Knot,
    ControlPoint,
    FitPoint,
    Weight,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticValueIssues {
    bits: u8,
}

impl DxfSplineAnalyticValueIssues {
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.bits
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    #[must_use]
    pub const fn contains(self, kind: DxfSplineAnalyticValueIssueKind) -> bool {
        self.bits & issue_bit(kind) != 0
    }

    fn insert(&mut self, kind: DxfSplineAnalyticValueIssueKind) {
        self.bits |= issue_bit(kind);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticValueRange {
    pub start: u32,
    pub end: u32,
}

impl DxfSplineAnalyticValueRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticKnot {
    pub evidence: DxfSplineValue,
    pub value: DxfDouble,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticPoint {
    pub tuple_ordinal: u32,
    pub x: DxfDouble,
    pub y: DxfDouble,
    pub z: DxfDouble,
    pub explicit_z: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticControlPoint {
    pub point: DxfSplineAnalyticPoint,
    pub weight: DxfDouble,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineAnalyticValueState {
    Available {
        knot_range: DxfSplineAnalyticValueRange,
        control_point_range: DxfSplineAnalyticValueRange,
        fit_point_range: DxfSplineAnalyticValueRange,
    },
    Unavailable(DxfSplineAnalyticValueIssues),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineAnalyticValueEntry {
    pub ordinal: u32,
    pub record: DxfSplineRecordEntry,
    pub state: DxfSplineAnalyticValueState,
}

#[derive(Debug)]
pub struct DxfSplineAnalyticValueDirectory {
    source_id: DxfSourceId,
    semantics: DxfSplineAuxiliarySemanticDirectory,
    entries: Box<[DxfSplineAnalyticValueEntry]>,
    knots: Box<[DxfSplineAnalyticKnot]>,
    control_points: Box<[DxfSplineAnalyticControlPoint]>,
    fit_points: Box<[DxfSplineAnalyticPoint]>,
}

impl DxfSplineAnalyticValueDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.spline_auxiliary_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }
        let mut entries = Vec::new();
        let mut knots = Vec::new();
        let mut control_points = Vec::new();
        let mut fit_points = Vec::new();
        for weight in semantics.weights().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = build_state(
                &semantics,
                weight,
                cancellation,
                &mut knots,
                &mut control_points,
                &mut fit_points,
            )?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfSplineAnalyticValueEntry {
                ordinal: compact_len(entries.len())?,
                record: weight.record(),
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            entries: entries.into_boxed_slice(),
            knots: knots.into_boxed_slice(),
            control_points: control_points.into_boxed_slice(),
            fit_points: fit_points.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfSplineAuxiliarySemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfSplineAnalyticValueEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfSplineAnalyticValueEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_raw_record(&self, raw: u64) -> Option<DxfSplineAnalyticValueEntry> {
        self.entries
            .binary_search_by_key(&raw, |entry| entry.record.record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn knots(&self, range: DxfSplineAnalyticValueRange) -> Option<&[DxfSplineAnalyticKnot]> {
        slice(&self.knots, range)
    }

    #[must_use]
    pub fn control_points(
        &self,
        range: DxfSplineAnalyticValueRange,
    ) -> Option<&[DxfSplineAnalyticControlPoint]> {
        slice(&self.control_points, range)
    }

    #[must_use]
    pub fn fit_points(
        &self,
        range: DxfSplineAnalyticValueRange,
    ) -> Option<&[DxfSplineAnalyticPoint]> {
        slice(&self.fit_points, range)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_analytic_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAnalyticValueDirectory, DxfError> {
        DxfSplineAnalyticValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_analytic_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAnalyticValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_analytic_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_analytic_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAnalyticValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_analytic_value_directory(cancellation)
    }
}

fn build_state(
    semantics: &DxfSplineAuxiliarySemanticDirectory,
    weight: DxfSplineWeightSemanticEntry,
    cancellation: &DxfCancellationToken,
    knots: &mut Vec<DxfSplineAnalyticKnot>,
    controls: &mut Vec<DxfSplineAnalyticControlPoint>,
    fits: &mut Vec<DxfSplineAnalyticPoint>,
) -> Result<DxfSplineAnalyticValueState, DxfError> {
    let raw = weight.record().record().ordinal();
    let points = semantics.auxiliary_directory().point_directory();
    let cards = points.card_directory();
    let mut issues = DxfSplineAnalyticValueIssues::default();
    let local_knots = collect_knots(cards, raw, cancellation, &mut issues)?;
    let control_tuples = points
        .tuples_for_raw_record(raw, DxfSplinePointKind::ControlPoint)
        .ok_or_else(invalid_internal_data)?;
    let fit_tuples = points
        .tuples_for_raw_record(raw, DxfSplinePointKind::FitPoint)
        .ok_or_else(invalid_internal_data)?;
    let local_points = collect_points(
        cards,
        control_tuples,
        cancellation,
        &mut issues,
        DxfSplineAnalyticValueIssueKind::ControlPoint,
    )?;
    let local_fits = collect_points(
        cards,
        fit_tuples,
        cancellation,
        &mut issues,
        DxfSplineAnalyticValueIssueKind::FitPoint,
    )?;
    let local_controls = collect_controls(
        semantics,
        weight,
        local_points,
        control_tuples.len(),
        &mut issues,
    )?;
    if !issues.is_empty() {
        return Ok(DxfSplineAnalyticValueState::Unavailable(issues));
    }
    append_values(
        local_knots,
        local_controls,
        local_fits,
        knots,
        controls,
        fits,
    )
}

fn collect_knots(
    cards: &DxfSplineCardDirectory,
    raw: u64,
    cancellation: &DxfCancellationToken,
    issues: &mut DxfSplineAnalyticValueIssues,
) -> Result<Vec<DxfSplineAnalyticKnot>, DxfError> {
    let card = cards
        .card_for_role(raw, DxfSplineValueRole::KnotValue)
        .ok_or_else(invalid_internal_data)?;
    let members = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let mut values = Vec::new();
    values
        .try_reserve(members.len())
        .map_err(|_| out_of_memory())?;
    for member in members.iter().copied() {
        ensure_not_cancelled(cancellation)?;
        let evidence = cards
            .value_for_member(member)
            .ok_or_else(invalid_internal_data)?;
        match evidence.value() {
            Ok(DxfSplineNumber::Double(value)) => {
                values.push(DxfSplineAnalyticKnot { evidence, value });
            }
            Err(DxfSplineNumericIssue::InvalidAsciiNumber(_)) => {
                issues.insert(DxfSplineAnalyticValueIssueKind::Knot);
            }
            Ok(_) => return Err(invalid_internal_data()),
        }
    }
    Ok(values)
}

fn collect_points(
    cards: &DxfSplineCardDirectory,
    tuples: &[DxfSplinePointTuple],
    cancellation: &DxfCancellationToken,
    issues: &mut DxfSplineAnalyticValueIssues,
    issue: DxfSplineAnalyticValueIssueKind,
) -> Result<Vec<DxfSplineAnalyticPoint>, DxfError> {
    let mut values = Vec::new();
    values
        .try_reserve(tuples.len())
        .map_err(|_| out_of_memory())?;
    for tuple in tuples.iter().copied() {
        ensure_not_cancelled(cancellation)?;
        let x = component(cards, tuple.x_member())?;
        let y = component(cards, tuple.y_member())?;
        let z_member = tuple.z_member();
        let z = component(cards, z_member)?;
        if z_member.is_some() && z.is_none() {
            issues.insert(issue);
            continue;
        }
        if let (Some(x), Some(y)) = (x, y) {
            values.push(DxfSplineAnalyticPoint {
                tuple_ordinal: u32::try_from(tuple.ordinal())
                    .map_err(|_| invalid_internal_data())?,
                x,
                y,
                z: z.unwrap_or_else(zero),
                explicit_z: z.is_some(),
            });
        } else {
            issues.insert(issue);
        }
    }
    Ok(values)
}

fn component(
    cards: &DxfSplineCardDirectory,
    member: Option<DxfSplineCardMember>,
) -> Result<Option<DxfDouble>, DxfError> {
    let Some(member) = member else {
        return Ok(None);
    };
    match cards
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)?
        .value()
    {
        Ok(DxfSplineNumber::Double(value)) => Ok(Some(value)),
        Err(DxfSplineNumericIssue::InvalidAsciiNumber(_)) => Ok(None),
        Ok(_) => Err(invalid_internal_data()),
    }
}

fn collect_controls(
    semantics: &DxfSplineAuxiliarySemanticDirectory,
    weight: DxfSplineWeightSemanticEntry,
    points: Vec<DxfSplineAnalyticPoint>,
    expected_count: usize,
    issues: &mut DxfSplineAnalyticValueIssues,
) -> Result<Vec<DxfSplineAnalyticControlPoint>, DxfError> {
    let weights = semantics
        .values_for_weight(weight.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let mut effective = Vec::new();
    effective
        .try_reserve(expected_count)
        .map_err(|_| out_of_memory())?;
    match weight.state() {
        DxfSplineWeightSemanticState::ImplicitUnit { .. } => {
            for _ in 0..expected_count {
                effective.push(unit());
            }
        }
        DxfSplineWeightSemanticState::ExplicitMatched { .. } if weights.len() == expected_count => {
            for weight in weights.iter().copied() {
                if let DxfSplineWeightValueState::Explicit { weight, .. } = weight {
                    effective.push(weight);
                } else {
                    issues.insert(DxfSplineAnalyticValueIssueKind::Weight);
                }
            }
        }
        _ => issues.insert(DxfSplineAnalyticValueIssueKind::Weight),
    }
    let mut values = Vec::new();
    if points.len() == expected_count
        && effective.len() == expected_count
        && !issues.contains(DxfSplineAnalyticValueIssueKind::Weight)
    {
        values
            .try_reserve(expected_count)
            .map_err(|_| out_of_memory())?;
        for (point, weight) in points.into_iter().zip(effective) {
            values.push(DxfSplineAnalyticControlPoint { point, weight });
        }
    }
    Ok(values)
}

fn append_values(
    local_knots: Vec<DxfSplineAnalyticKnot>,
    local_controls: Vec<DxfSplineAnalyticControlPoint>,
    local_fits: Vec<DxfSplineAnalyticPoint>,
    knots: &mut Vec<DxfSplineAnalyticKnot>,
    controls: &mut Vec<DxfSplineAnalyticControlPoint>,
    fits: &mut Vec<DxfSplineAnalyticPoint>,
) -> Result<DxfSplineAnalyticValueState, DxfError> {
    let knot_range = extend_range(knots, local_knots)?;
    let control_point_range = extend_range(controls, local_controls)?;
    let fit_point_range = extend_range(fits, local_fits)?;
    Ok(DxfSplineAnalyticValueState::Available {
        knot_range,
        control_point_range,
        fit_point_range,
    })
}

fn extend_range<T>(
    values: &mut Vec<T>,
    added: Vec<T>,
) -> Result<DxfSplineAnalyticValueRange, DxfError> {
    let start = compact_len(values.len())?;
    values
        .try_reserve(added.len())
        .map_err(|_| out_of_memory())?;
    values.extend(added);
    DxfSplineAnalyticValueRange::new(start, compact_len(values.len())?)
}

fn slice<T>(values: &[T], range: DxfSplineAnalyticValueRange) -> Option<&[T]> {
    values.get(usize::try_from(range.start).ok()?..usize::try_from(range.end).ok()?)
}

const fn issue_bit(kind: DxfSplineAnalyticValueIssueKind) -> u8 {
    1_u8 << kind as u8
}

fn zero() -> DxfDouble {
    DxfDouble::from_f64(0.0)
}

fn unit() -> DxfDouble {
    DxfDouble::from_f64(1.0)
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

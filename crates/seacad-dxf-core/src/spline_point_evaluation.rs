//! Bounded on-demand point evaluation for analytically ready SPLINE records.

use std::io;

use crate::{
    DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation, DxfSplineAnalyticDirectory,
    DxfSplineAnalyticIssues, DxfSplineAnalyticState, DxfSplineRecordEntry,
};

/// Hard CPU and scratch-space bound for quadratic-time De Boor evaluation.
pub const DXF_SPLINE_EVALUATION_MAX_DEGREE: u16 = 64;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineEvaluationInputKind {
    Knot,
    ControlPoint,
    Weight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplinePointEvaluationIssue {
    AnalyticUnavailable(DxfSplineAnalyticIssues),
    NonFiniteParameter,
    ParameterOutOfDomain {
        parameter_start: DxfDouble,
        parameter_end: DxfDouble,
    },
    DegreeLimitExceeded {
        degree: u16,
        maximum: u16,
    },
    NonFiniteInput {
        kind: DxfSplineEvaluationInputKind,
        ordinal: u32,
    },
    DegenerateKnotInterval,
    ArithmeticOverflow,
    NonPositiveHomogeneousWeight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineEvaluatedPoint {
    x: DxfDouble,
    y: DxfDouble,
    z: DxfDouble,
}

impl DxfSplineEvaluatedPoint {
    #[must_use]
    pub const fn x(self) -> DxfDouble {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> DxfDouble {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> DxfDouble {
        self.z
    }

    #[must_use]
    pub const fn components(self) -> [DxfDouble; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplinePointEvaluationState {
    Available(DxfSplineEvaluatedPoint),
    Unavailable(DxfSplinePointEvaluationIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplinePointEvaluation {
    record: DxfSplineRecordEntry,
    parameter: DxfDouble,
    state: DxfSplinePointEvaluationState,
}

impl DxfSplinePointEvaluation {
    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn parameter(self) -> DxfDouble {
        self.parameter
    }

    #[must_use]
    pub const fn state(self) -> DxfSplinePointEvaluationState {
        self.state
    }
}

impl DxfSplineAnalyticDirectory {
    /// Evaluates one source-bound record at one parameter without tessellation.
    pub fn evaluate_point_for_raw_record(
        &self,
        raw_record_ordinal: u64,
        parameter: DxfDouble,
        cancellation: &DxfCancellationToken,
    ) -> Result<Option<DxfSplinePointEvaluation>, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let Some(entry) = self.entry_for_raw_record(raw_record_ordinal) else {
            return Ok(None);
        };
        let state = match entry.state {
            DxfSplineAnalyticState::Available(data) => {
                evaluate_available(self, data, parameter, cancellation)?
            }
            DxfSplineAnalyticState::Unavailable(issues) => {
                DxfSplinePointEvaluationState::Unavailable(
                    DxfSplinePointEvaluationIssue::AnalyticUnavailable(issues),
                )
            }
        };
        ensure_not_cancelled(cancellation)?;
        Ok(Some(DxfSplinePointEvaluation {
            record: entry.record,
            parameter,
            state,
        }))
    }
}

fn evaluate_available(
    directory: &DxfSplineAnalyticDirectory,
    data: crate::DxfSplineAnalyticData,
    parameter: DxfDouble,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSplinePointEvaluationState, DxfError> {
    let value = parameter.to_f64();
    if !value.is_finite() {
        return Ok(unavailable(
            DxfSplinePointEvaluationIssue::NonFiniteParameter,
        ));
    }
    let start = data.parameter_start.to_f64();
    let end = data.parameter_end.to_f64();
    if value < start || value > end {
        return Ok(unavailable(
            DxfSplinePointEvaluationIssue::ParameterOutOfDomain {
                parameter_start: data.parameter_start,
                parameter_end: data.parameter_end,
            },
        ));
    }
    if data.degree > DXF_SPLINE_EVALUATION_MAX_DEGREE {
        return Ok(unavailable(
            DxfSplinePointEvaluationIssue::DegreeLimitExceeded {
                degree: data.degree,
                maximum: DXF_SPLINE_EVALUATION_MAX_DEGREE,
            },
        ));
    }

    let values = directory.value_directory();
    let knots = values
        .knots(data.knot_range)
        .ok_or_else(invalid_internal_data)?;
    let controls = values
        .control_points(data.control_point_range)
        .ok_or_else(invalid_internal_data)?;
    for (ordinal, knot) in knots.iter().enumerate() {
        if !knot.value.is_finite() {
            return Ok(unavailable(DxfSplinePointEvaluationIssue::NonFiniteInput {
                kind: DxfSplineEvaluationInputKind::Knot,
                ordinal: u32::try_from(ordinal).map_err(|_| invalid_internal_data())?,
            }));
        }
    }
    let degree = usize::from(data.degree);
    let Some(last_control) = controls.len().checked_sub(1) else {
        return Err(invalid_internal_data());
    };
    let span = find_span(knots, degree, last_control, value)?;
    let mut work = Vec::new();
    work.try_reserve_exact(degree + 1)
        .map_err(|_| out_of_memory())?;
    for local in 0..=degree {
        ensure_not_cancelled(cancellation)?;
        let control_index = span - degree + local;
        let control = controls
            .get(control_index)
            .ok_or_else(invalid_internal_data)?;
        let coordinates = [
            control.point.x.to_f64(),
            control.point.y.to_f64(),
            control.point.z.to_f64(),
        ];
        if coordinates.iter().any(|component| !component.is_finite()) {
            return Ok(unavailable(DxfSplinePointEvaluationIssue::NonFiniteInput {
                kind: DxfSplineEvaluationInputKind::ControlPoint,
                ordinal: control.point.tuple_ordinal,
            }));
        }
        let weight = control.weight.to_f64();
        if !weight.is_finite() {
            return Ok(unavailable(DxfSplinePointEvaluationIssue::NonFiniteInput {
                kind: DxfSplineEvaluationInputKind::Weight,
                ordinal: u32::try_from(control_index).map_err(|_| invalid_internal_data())?,
            }));
        }
        let homogeneous = [
            coordinates[0] * weight,
            coordinates[1] * weight,
            coordinates[2] * weight,
            weight,
        ];
        if homogeneous.iter().any(|component| !component.is_finite()) {
            return Ok(unavailable(
                DxfSplinePointEvaluationIssue::ArithmeticOverflow,
            ));
        }
        work.push(homogeneous);
    }

    for level in 1..=degree {
        ensure_not_cancelled(cancellation)?;
        for local in (level..=degree).rev() {
            let knot_index = span - degree + local;
            let left = finite_knot(knots, knot_index)?;
            let right = finite_knot(knots, knot_index + degree - level + 1)?;
            let denominator = right - left;
            if denominator == 0.0 {
                return Ok(unavailable(
                    DxfSplinePointEvaluationIssue::DegenerateKnotInterval,
                ));
            }
            let alpha = (value - left) / denominator;
            let previous = work[local - 1];
            let current = work[local];
            work[local] = std::array::from_fn(|component| {
                (1.0 - alpha) * previous[component] + alpha * current[component]
            });
            if work[local].iter().any(|component| !component.is_finite()) {
                return Ok(unavailable(
                    DxfSplinePointEvaluationIssue::ArithmeticOverflow,
                ));
            }
        }
    }

    let result = work.get(degree).ok_or_else(invalid_internal_data)?;
    if result[3] <= 0.0 {
        return Ok(unavailable(
            DxfSplinePointEvaluationIssue::NonPositiveHomogeneousWeight,
        ));
    }
    let point = [
        result[0] / result[3],
        result[1] / result[3],
        result[2] / result[3],
    ];
    if point.iter().any(|component| !component.is_finite()) {
        return Ok(unavailable(
            DxfSplinePointEvaluationIssue::ArithmeticOverflow,
        ));
    }
    Ok(DxfSplinePointEvaluationState::Available(
        DxfSplineEvaluatedPoint {
            x: canonical(point[0]),
            y: canonical(point[1]),
            z: canonical(point[2]),
        },
    ))
}

fn find_span(
    knots: &[crate::DxfSplineAnalyticKnot],
    degree: usize,
    last_control: usize,
    parameter: f64,
) -> Result<usize, DxfError> {
    let domain_end = finite_knot(knots, last_control + 1)?;
    if parameter == domain_end {
        return Ok(last_control);
    }
    let mut low = degree;
    let mut high = last_control + 1;
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if parameter < finite_knot(knots, middle)? {
            high = middle;
        } else {
            low = middle;
        }
    }
    Ok(low)
}

fn finite_knot(knots: &[crate::DxfSplineAnalyticKnot], index: usize) -> Result<f64, DxfError> {
    let value = knots
        .get(index)
        .ok_or_else(invalid_internal_data)?
        .value
        .to_f64();
    if value.is_finite() {
        Ok(value)
    } else {
        Err(invalid_internal_data())
    }
}

const fn unavailable(issue: DxfSplinePointEvaluationIssue) -> DxfSplinePointEvaluationState {
    DxfSplinePointEvaluationState::Unavailable(issue)
}

fn canonical(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
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

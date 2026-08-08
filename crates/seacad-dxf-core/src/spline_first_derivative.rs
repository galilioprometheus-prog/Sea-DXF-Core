//! Bounded rational first derivatives for analytically ready SPLINE records.

use std::io;

use crate::{
    DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation, DxfSplineAnalyticDirectory,
    DxfSplineAnalyticState, DxfSplineEvaluatedPoint, DxfSplinePointEvaluationIssue,
    DxfSplineRecordEntry,
    spline_point_evaluation::{
        DxfSplineEvaluationMath, ensure_not_cancelled, evaluate_homogeneous, homogeneous_to_point,
        prepare_evaluation,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineEvaluatedVector {
    x: DxfDouble,
    y: DxfDouble,
    z: DxfDouble,
}

impl DxfSplineEvaluatedVector {
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
pub struct DxfSplineEvaluatedDifferential {
    point: DxfSplineEvaluatedPoint,
    first_derivative: DxfSplineEvaluatedVector,
}

impl DxfSplineEvaluatedDifferential {
    #[must_use]
    pub const fn point(self) -> DxfSplineEvaluatedPoint {
        self.point
    }

    #[must_use]
    pub const fn first_derivative(self) -> DxfSplineEvaluatedVector {
        self.first_derivative
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineFirstDerivativeState {
    Available(DxfSplineEvaluatedDifferential),
    Unavailable(DxfSplinePointEvaluationIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineFirstDerivativeEvaluation {
    record: DxfSplineRecordEntry,
    parameter: DxfDouble,
    state: DxfSplineFirstDerivativeState,
}

impl DxfSplineFirstDerivativeEvaluation {
    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn parameter(self) -> DxfDouble {
        self.parameter
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineFirstDerivativeState {
        self.state
    }
}

impl DxfSplineAnalyticDirectory {
    /// Evaluates a source-bound point and its first Cartesian derivative.
    pub fn evaluate_first_derivative_for_raw_record(
        &self,
        raw_record_ordinal: u64,
        parameter: DxfDouble,
        cancellation: &DxfCancellationToken,
    ) -> Result<Option<DxfSplineFirstDerivativeEvaluation>, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let Some(entry) = self.entry_for_raw_record(raw_record_ordinal) else {
            return Ok(None);
        };
        let state = match entry.state {
            DxfSplineAnalyticState::Available(data) => {
                evaluate_available(self, data, parameter, cancellation)?
            }
            DxfSplineAnalyticState::Unavailable(issues) => {
                DxfSplineFirstDerivativeState::Unavailable(
                    DxfSplinePointEvaluationIssue::AnalyticUnavailable(issues),
                )
            }
        };
        ensure_not_cancelled(cancellation)?;
        Ok(Some(DxfSplineFirstDerivativeEvaluation {
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
) -> Result<DxfSplineFirstDerivativeState, DxfError> {
    let prepared = match prepare_evaluation(directory, data, parameter, cancellation)? {
        DxfSplineEvaluationMath::Available(prepared) => prepared,
        DxfSplineEvaluationMath::Unavailable(issue) => return Ok(unavailable(issue)),
    };
    if prepared.degree == 0 {
        return Err(invalid_internal_data());
    }

    let mut derivative_controls = Vec::new();
    derivative_controls
        .try_reserve_exact(prepared.degree)
        .map_err(|_| out_of_memory())?;
    for local in 0..prepared.degree {
        ensure_not_cancelled(cancellation)?;
        let control_index = prepared.span - prepared.degree + local;
        let left = knot(prepared.knots, control_index + 1)?;
        let right = knot(prepared.knots, control_index + prepared.degree + 1)?;
        let denominator = right - left;
        if denominator == 0.0 {
            return Ok(unavailable(
                DxfSplinePointEvaluationIssue::DegenerateKnotInterval,
            ));
        }
        let scale = prepared.degree as f64 / denominator;
        let derivative = std::array::from_fn(|component| {
            scale
                * (prepared.homogeneous[local + 1][component]
                    - prepared.homogeneous[local][component])
        });
        if derivative.iter().any(|component| !component.is_finite()) {
            return Ok(unavailable(
                DxfSplinePointEvaluationIssue::ArithmeticOverflow,
            ));
        }
        derivative_controls.push(derivative);
    }

    let homogeneous_derivative = match evaluate_homogeneous(
        prepared.knots,
        prepared.degree - 1,
        prepared.span - 1,
        1,
        prepared.parameter,
        &mut derivative_controls,
        cancellation,
    )? {
        DxfSplineEvaluationMath::Available(value) => value,
        DxfSplineEvaluationMath::Unavailable(issue) => return Ok(unavailable(issue)),
    };
    let mut point_controls = prepared.homogeneous;
    let homogeneous_point = match evaluate_homogeneous(
        prepared.knots,
        prepared.degree,
        prepared.span,
        0,
        prepared.parameter,
        &mut point_controls,
        cancellation,
    )? {
        DxfSplineEvaluationMath::Available(value) => value,
        DxfSplineEvaluationMath::Unavailable(issue) => return Ok(unavailable(issue)),
    };
    let point = match homogeneous_to_point(homogeneous_point) {
        DxfSplineEvaluationMath::Available(point) => point,
        DxfSplineEvaluationMath::Unavailable(issue) => return Ok(unavailable(issue)),
    };
    let point_components = point.components().map(DxfDouble::to_f64);
    let derivative: [f64; 3] = std::array::from_fn(|component| {
        (homogeneous_derivative[component]
            - point_components[component] * homogeneous_derivative[3])
            / homogeneous_point[3]
    });
    if derivative.iter().any(|component| !component.is_finite()) {
        return Ok(unavailable(
            DxfSplinePointEvaluationIssue::ArithmeticOverflow,
        ));
    }
    Ok(DxfSplineFirstDerivativeState::Available(
        DxfSplineEvaluatedDifferential {
            point,
            first_derivative: DxfSplineEvaluatedVector {
                x: canonical(derivative[0]),
                y: canonical(derivative[1]),
                z: canonical(derivative[2]),
            },
        },
    ))
}

fn knot(knots: &[crate::DxfSplineAnalyticKnot], index: usize) -> Result<f64, DxfError> {
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

const fn unavailable(issue: DxfSplinePointEvaluationIssue) -> DxfSplineFirstDerivativeState {
    DxfSplineFirstDerivativeState::Unavailable(issue)
}

fn canonical(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
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

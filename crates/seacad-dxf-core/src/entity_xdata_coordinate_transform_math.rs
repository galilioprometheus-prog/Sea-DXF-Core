//! Validated affine channels used by generic entity XDATA coordinate projection.

use crate::DxfDouble;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataCoordinateTransformInput {
    Translation,
    ScaleBasePoint,
    ScaleFactor,
    RotationBasePoint,
    RotationAxis,
    RotationDegrees,
    MirrorPoint,
    MirrorNormal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataCoordinateTransformIssue {
    NonFiniteInput(DxfEntityXDataCoordinateTransformInput),
    ZeroScaleFactor,
    ZeroLengthRotationAxis,
    ZeroLengthMirrorNormal,
    NonFiniteDerivedTransform,
}

/// Three coordinated affine channels matching XDATA groups 1011, 1012, and 1013.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataCoordinateTransform {
    position_rows: [[DxfDouble; 4]; 3],
    displacement_rows: [[DxfDouble; 3]; 3],
    direction_rows: [[DxfDouble; 3]; 3],
}

impl DxfEntityXDataCoordinateTransform {
    #[must_use]
    pub fn identity() -> Self {
        Self::from_linear_translation(identity3(), [0.0; 3], identity3(), identity3())
    }

    pub fn translation(
        delta: [DxfDouble; 3],
    ) -> Result<Self, DxfEntityXDataCoordinateTransformIssue> {
        let delta = finite_input(delta, DxfEntityXDataCoordinateTransformInput::Translation)?;
        Ok(Self::from_linear_translation(
            identity3(),
            delta,
            identity3(),
            identity3(),
        ))
    }

    pub fn uniform_scale(
        base: [DxfDouble; 3],
        factor: DxfDouble,
    ) -> Result<Self, DxfEntityXDataCoordinateTransformIssue> {
        let base = finite_input(base, DxfEntityXDataCoordinateTransformInput::ScaleBasePoint)?;
        let factor = finite_scalar(factor, DxfEntityXDataCoordinateTransformInput::ScaleFactor)?;
        if factor == 0.0 {
            return Err(DxfEntityXDataCoordinateTransformIssue::ZeroScaleFactor);
        }
        let linear = [[factor, 0.0, 0.0], [0.0, factor, 0.0], [0.0, 0.0, factor]];
        let translation = base_translation(linear, base)?;
        Ok(Self::from_linear_translation(
            linear,
            translation,
            linear,
            identity3(),
        ))
    }

    pub fn rotation(
        base: [DxfDouble; 3],
        axis: [DxfDouble; 3],
        degrees: DxfDouble,
    ) -> Result<Self, DxfEntityXDataCoordinateTransformIssue> {
        let base = finite_input(
            base,
            DxfEntityXDataCoordinateTransformInput::RotationBasePoint,
        )?;
        let axis = finite_input(axis, DxfEntityXDataCoordinateTransformInput::RotationAxis)?;
        let axis = normalize(axis)
            .ok_or(DxfEntityXDataCoordinateTransformIssue::ZeroLengthRotationAxis)?;
        let degrees = finite_scalar(
            degrees,
            DxfEntityXDataCoordinateTransformInput::RotationDegrees,
        )?;
        let linear = axis_rotation(axis, degrees.to_radians());
        ensure_finite_linear(linear)?;
        let translation = base_translation(linear, base)?;
        Ok(Self::from_linear_translation(
            linear,
            translation,
            linear,
            linear,
        ))
    }

    pub fn mirror(
        point: [DxfDouble; 3],
        normal: [DxfDouble; 3],
    ) -> Result<Self, DxfEntityXDataCoordinateTransformIssue> {
        let point = finite_input(point, DxfEntityXDataCoordinateTransformInput::MirrorPoint)?;
        let normal = finite_input(normal, DxfEntityXDataCoordinateTransformInput::MirrorNormal)?;
        let normal = normalize(normal)
            .ok_or(DxfEntityXDataCoordinateTransformIssue::ZeroLengthMirrorNormal)?;
        let mut linear = identity3();
        for row in 0..3 {
            for column in 0..3 {
                linear[row][column] -= 2.0 * normal[row] * normal[column];
            }
        }
        ensure_finite_linear(linear)?;
        let translation = base_translation(linear, point)?;
        Ok(Self::from_linear_translation(
            linear,
            translation,
            linear,
            linear,
        ))
    }

    /// Composes `self` followed by `next` while retaining the three role channels.
    pub fn then(self, next: Self) -> Result<Self, DxfEntityXDataCoordinateTransformIssue> {
        let position = compose_affine(next.position_f64(), self.position_f64())?;
        let displacement = compose_linear(next.displacement_f64(), self.displacement_f64())?;
        let direction = compose_linear(next.direction_f64(), self.direction_f64())?;
        Ok(Self {
            position_rows: position.map(|row| row.map(canonical_double)),
            displacement_rows: displacement.map(|row| row.map(canonical_double)),
            direction_rows: direction.map(|row| row.map(canonical_double)),
        })
    }

    #[must_use]
    pub const fn position_rows(self) -> [[DxfDouble; 4]; 3] {
        self.position_rows
    }

    #[must_use]
    pub const fn displacement_rows(self) -> [[DxfDouble; 3]; 3] {
        self.displacement_rows
    }

    #[must_use]
    pub const fn direction_rows(self) -> [[DxfDouble; 3]; 3] {
        self.direction_rows
    }

    pub(crate) fn position_f64(self) -> [[f64; 4]; 3] {
        self.position_rows.map(|row| row.map(DxfDouble::to_f64))
    }

    pub(crate) fn displacement_f64(self) -> [[f64; 3]; 3] {
        self.displacement_rows.map(|row| row.map(DxfDouble::to_f64))
    }

    pub(crate) fn direction_f64(self) -> [[f64; 3]; 3] {
        self.direction_rows.map(|row| row.map(DxfDouble::to_f64))
    }

    fn from_linear_translation(
        position: [[f64; 3]; 3],
        translation: [f64; 3],
        displacement: [[f64; 3]; 3],
        direction: [[f64; 3]; 3],
    ) -> Self {
        let position_rows = std::array::from_fn(|row| {
            [
                canonical_double(position[row][0]),
                canonical_double(position[row][1]),
                canonical_double(position[row][2]),
                canonical_double(translation[row]),
            ]
        });
        Self {
            position_rows,
            displacement_rows: displacement.map(|row| row.map(canonical_double)),
            direction_rows: direction.map(|row| row.map(canonical_double)),
        }
    }
}

fn finite_input(
    value: [DxfDouble; 3],
    input: DxfEntityXDataCoordinateTransformInput,
) -> Result<[f64; 3], DxfEntityXDataCoordinateTransformIssue> {
    let value = value.map(DxfDouble::to_f64);
    if finite3(value) {
        Ok(value)
    } else {
        Err(DxfEntityXDataCoordinateTransformIssue::NonFiniteInput(
            input,
        ))
    }
}

fn finite_scalar(
    value: DxfDouble,
    input: DxfEntityXDataCoordinateTransformInput,
) -> Result<f64, DxfEntityXDataCoordinateTransformIssue> {
    if value.is_finite() {
        Ok(value.to_f64())
    } else {
        Err(DxfEntityXDataCoordinateTransformIssue::NonFiniteInput(
            input,
        ))
    }
}

fn normalize(value: [f64; 3]) -> Option<[f64; 3]> {
    let scale = value
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return None;
    }
    let scaled = value.map(|value| value / scale);
    let length = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
    let normalized = scaled.map(|value| value / length);
    finite3(normalized).then_some(normalized)
}

fn axis_rotation(axis: [f64; 3], radians: f64) -> [[f64; 3]; 3] {
    let [x, y, z] = axis;
    let (sine, cosine) = radians.sin_cos();
    let remainder = 1.0 - cosine;
    [
        [
            cosine + x * x * remainder,
            x * y * remainder - z * sine,
            x * z * remainder + y * sine,
        ],
        [
            y * x * remainder + z * sine,
            cosine + y * y * remainder,
            y * z * remainder - x * sine,
        ],
        [
            z * x * remainder - y * sine,
            z * y * remainder + x * sine,
            cosine + z * z * remainder,
        ],
    ]
}

fn base_translation(
    linear: [[f64; 3]; 3],
    base: [f64; 3],
) -> Result<[f64; 3], DxfEntityXDataCoordinateTransformIssue> {
    let mapped = apply_linear(linear, base);
    let translation = std::array::from_fn(|index| base[index] - mapped[index]);
    if finite3(translation) {
        Ok(translation)
    } else {
        Err(DxfEntityXDataCoordinateTransformIssue::NonFiniteDerivedTransform)
    }
}

fn compose_affine(
    next: [[f64; 4]; 3],
    first: [[f64; 4]; 3],
) -> Result<[[f64; 4]; 3], DxfEntityXDataCoordinateTransformIssue> {
    let rows = std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            let linear = (0..3)
                .map(|index| next[row][index] * first[index][column])
                .sum::<f64>();
            if column == 3 {
                linear + next[row][3]
            } else {
                linear
            }
        })
    });
    ensure_finite_affine(rows)?;
    Ok(rows)
}

fn compose_linear(
    next: [[f64; 3]; 3],
    first: [[f64; 3]; 3],
) -> Result<[[f64; 3]; 3], DxfEntityXDataCoordinateTransformIssue> {
    let rows = std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            (0..3)
                .map(|index| next[row][index] * first[index][column])
                .sum()
        })
    });
    ensure_finite_linear(rows)?;
    Ok(rows)
}

pub(crate) fn apply_affine(rows: [[f64; 4]; 3], value: [f64; 3]) -> [f64; 3] {
    rows.map(|row| row[0] * value[0] + row[1] * value[1] + row[2] * value[2] + row[3])
}

pub(crate) fn apply_linear(rows: [[f64; 3]; 3], value: [f64; 3]) -> [f64; 3] {
    rows.map(|row| row[0] * value[0] + row[1] * value[1] + row[2] * value[2])
}

fn ensure_finite_affine(
    value: [[f64; 4]; 3],
) -> Result<(), DxfEntityXDataCoordinateTransformIssue> {
    finite_matrix(value)
        .then_some(())
        .ok_or(DxfEntityXDataCoordinateTransformIssue::NonFiniteDerivedTransform)
}

fn ensure_finite_linear(
    value: [[f64; 3]; 3],
) -> Result<(), DxfEntityXDataCoordinateTransformIssue> {
    finite_matrix(value)
        .then_some(())
        .ok_or(DxfEntityXDataCoordinateTransformIssue::NonFiniteDerivedTransform)
}

fn finite_matrix<const COLUMNS: usize>(value: [[f64; COLUMNS]; 3]) -> bool {
    value.iter().flatten().all(|value| value.is_finite())
}

const fn identity3() -> [[f64; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|value| value.is_finite())
}

pub(crate) fn canonical_double(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

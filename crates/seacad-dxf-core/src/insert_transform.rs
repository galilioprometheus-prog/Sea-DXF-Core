//! Fail-closed affine transforms for eligible INSERT targets.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertTargetEligibilityDirectory, DxfInsertTargetEligibilityEntry,
    DxfInsertTargetEligibilityState, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertTransformInput {
    InsertionPoint,
    ScaleFactors,
    RotationDegrees,
    Extrusion,
    BlockBasePoint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertTransformIssue {
    TargetNotEligible(DxfInsertTargetEligibilityState),
    InsertSemanticsUnavailable,
    BlockBasePointUnavailable,
    NonFiniteInput(DxfInsertTransformInput),
    ZeroLengthExtrusion,
    NonFiniteDerivedTransform,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertTransformApplicationIssue {
    NonFiniteInput,
    NonFiniteResult,
}

/// Finite row-major 3x4 affine matrix from BLOCK coordinates to WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAffineTransform {
    rows: [[DxfDouble; 4]; 3],
    normal: [DxfDouble; 3],
}

impl DxfInsertAffineTransform {
    #[must_use]
    pub const fn rows(self) -> [[DxfDouble; 4]; 3] {
        self.rows
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }

    pub fn transform_point(
        self,
        point: [DxfDouble; 3],
    ) -> Result<[DxfDouble; 3], DxfInsertTransformApplicationIssue> {
        let point = point.map(DxfDouble::to_f64);
        if !finite3(point) {
            return Err(DxfInsertTransformApplicationIssue::NonFiniteInput);
        }
        let rows = self.rows.map(|row| row.map(DxfDouble::to_f64));
        let transformed =
            rows.map(|row| row[0] * point[0] + row[1] * point[1] + row[2] * point[2] + row[3]);
        if finite3(transformed) {
            Ok(transformed.map(DxfDouble::from_f64))
        } else {
            Err(DxfInsertTransformApplicationIssue::NonFiniteResult)
        }
    }

    pub(crate) fn offset_wcs(
        self,
        offset: [f64; 3],
    ) -> Result<Self, DxfInsertTransformApplicationIssue> {
        if !finite3(offset) {
            return Err(DxfInsertTransformApplicationIssue::NonFiniteInput);
        }
        let mut rows = self.rows;
        for (row, component) in rows.iter_mut().zip(offset) {
            let translated = row[3].to_f64() + component;
            if !translated.is_finite() {
                return Err(DxfInsertTransformApplicationIssue::NonFiniteResult);
            }
            row[3] = canonical_double(translated);
        }
        Ok(Self {
            rows,
            normal: self.normal,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertTransformEntry {
    eligibility: DxfInsertTargetEligibilityEntry,
    transform: Result<DxfInsertAffineTransform, DxfInsertTransformIssue>,
}

impl DxfInsertTransformEntry {
    #[must_use]
    pub const fn eligibility(self) -> DxfInsertTargetEligibilityEntry {
        self.eligibility
    }

    pub const fn transform(self) -> Result<DxfInsertAffineTransform, DxfInsertTransformIssue> {
        self.transform
    }
}

#[derive(Debug)]
pub struct DxfInsertTransformDirectory {
    source_id: DxfSourceId,
    eligibility: DxfInsertTargetEligibilityDirectory,
    entries: Box<[DxfInsertTransformEntry]>,
}

impl DxfInsertTransformDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let eligibility = document.insert_target_eligibility_directory(cancellation)?;
        if eligibility.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: eligibility.source_id(),
            });
        }
        let mut entries = Vec::new();
        entries
            .try_reserve(eligibility.entries().len())
            .map_err(|_| out_of_memory())?;
        for reviewed in eligibility.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfInsertTransformEntry {
                eligibility: reviewed,
                transform: derive_transform(&eligibility, reviewed)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            eligibility,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn eligibility_directory(&self) -> &DxfInsertTargetEligibilityDirectory {
        &self.eligibility
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfInsertTransformEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_insert_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertTransformEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.eligibility().resolution().insert().record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_transform_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertTransformDirectory, DxfError> {
        DxfInsertTransformDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_transform_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertTransformDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_transform_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_transform_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertTransformDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_transform_directory(cancellation)
    }
}

fn derive_transform(
    eligibility: &DxfInsertTargetEligibilityDirectory,
    reviewed: DxfInsertTargetEligibilityEntry,
) -> Result<Result<DxfInsertAffineTransform, DxfInsertTransformIssue>, DxfError> {
    if reviewed.state() != DxfInsertTargetEligibilityState::Eligible {
        return Ok(Err(DxfInsertTransformIssue::TargetNotEligible(
            reviewed.state(),
        )));
    }
    let resolution = reviewed.resolution();
    let insert_raw = resolution.insert().record().ordinal();
    let insert_semantics = eligibility
        .resolution_directory()
        .insert_semantic_directory()
        .semantics_for_entry(resolution.insert())?;
    let [target] = eligibility
        .resolution_directory()
        .targets_for_insert_raw_ordinal(insert_raw)
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    let block_semantics = eligibility
        .resolution_directory()
        .block_name_index_directory()
        .consistency_directory()
        .semantic_directory()
        .semantics_for_entry(target.record())?;
    let Some(insert_semantics) = insert_semantics else {
        return Err(invalid_internal_data());
    };
    let Some(block_semantics) = block_semantics else {
        return Err(invalid_internal_data());
    };

    let Some(insertion) = insert_semantics.insertion_point_value() else {
        return Ok(Err(DxfInsertTransformIssue::InsertSemanticsUnavailable));
    };
    let Some(scale) = insert_semantics.scale_factor_values() else {
        return Ok(Err(DxfInsertTransformIssue::InsertSemanticsUnavailable));
    };
    let Some(rotation) = insert_semantics.rotation_degrees_value() else {
        return Ok(Err(DxfInsertTransformIssue::InsertSemanticsUnavailable));
    };
    let Some(extrusion) = insert_semantics.extrusion_value() else {
        return Ok(Err(DxfInsertTransformIssue::InsertSemanticsUnavailable));
    };
    let Some(base) = block_semantics.base_point_value() else {
        return Ok(Err(DxfInsertTransformIssue::BlockBasePointUnavailable));
    };
    Ok(derive_finite_transform(
        insertion, scale, rotation, extrusion, base,
    ))
}

fn derive_finite_transform(
    insertion: [DxfDouble; 3],
    scale: [DxfDouble; 3],
    rotation: DxfDouble,
    extrusion: [DxfDouble; 3],
    base: [DxfDouble; 3],
) -> Result<DxfInsertAffineTransform, DxfInsertTransformIssue> {
    let insertion = finite_input(insertion, DxfInsertTransformInput::InsertionPoint)?;
    let scale = finite_input(scale, DxfInsertTransformInput::ScaleFactors)?;
    let rotation = rotation.to_f64();
    if !rotation.is_finite() {
        return Err(DxfInsertTransformIssue::NonFiniteInput(
            DxfInsertTransformInput::RotationDegrees,
        ));
    }
    let extrusion = finite_input(extrusion, DxfInsertTransformInput::Extrusion)?;
    let base = finite_input(base, DxfInsertTransformInput::BlockBasePoint)?;
    let basis = OcsBasis::from_extrusion(extrusion)?;
    let axes = basis.rotated_xy(rotation.to_radians());
    let columns = [
        scale3(axes[0], scale[0]),
        scale3(axes[1], scale[1]),
        scale3(basis.z, scale[2]),
    ];
    let translation = subtract(basis.transform(insertion), linear_transform(columns, base));
    let rows = [
        [columns[0][0], columns[1][0], columns[2][0], translation[0]],
        [columns[0][1], columns[1][1], columns[2][1], translation[1]],
        [columns[0][2], columns[1][2], columns[2][2], translation[2]],
    ];
    if !rows.iter().flatten().all(|component| component.is_finite()) {
        return Err(DxfInsertTransformIssue::NonFiniteDerivedTransform);
    }
    Ok(DxfInsertAffineTransform {
        rows: rows.map(|row| row.map(canonical_double)),
        normal: basis.z.map(canonical_double),
    })
}

#[derive(Clone, Copy)]
pub(crate) struct OcsBasis {
    x: [f64; 3],
    y: [f64; 3],
    z: [f64; 3],
}

impl OcsBasis {
    pub(crate) fn from_extrusion(extrusion: [f64; 3]) -> Result<Self, DxfInsertTransformIssue> {
        let z = normalize(extrusion)?;
        let x = if z[0].abs() < 1.0 / 64.0 && z[1].abs() < 1.0 / 64.0 {
            normalize([z[2], 0.0, -z[0]])?
        } else {
            normalize([-z[1], z[0], 0.0])?
        };
        let y = normalize(cross(z, x))?;
        Ok(Self { x, y, z })
    }

    pub(crate) fn rotated_xy(self, rotation_radians: f64) -> [[f64; 3]; 2] {
        let (sine, cosine) = rotation_radians.sin_cos();
        [
            add(scale3(self.x, cosine), scale3(self.y, sine)),
            add(scale3(self.x, -sine), scale3(self.y, cosine)),
        ]
    }

    fn transform(self, point: [f64; 3]) -> [f64; 3] {
        linear_transform([self.x, self.y, self.z], point)
    }
}

fn finite_input(
    value: [DxfDouble; 3],
    input: DxfInsertTransformInput,
) -> Result<[f64; 3], DxfInsertTransformIssue> {
    let value = value.map(DxfDouble::to_f64);
    if finite3(value) {
        Ok(value)
    } else {
        Err(DxfInsertTransformIssue::NonFiniteInput(input))
    }
}

fn normalize(vector: [f64; 3]) -> Result<[f64; 3], DxfInsertTransformIssue> {
    let scale = vector
        .iter()
        .map(|component| component.abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Err(DxfInsertTransformIssue::ZeroLengthExtrusion);
    }
    let scaled = vector.map(|component| component / scale);
    let length = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
    let normalized = scaled.map(|component| component / length);
    if finite3(normalized) {
        Ok(normalized)
    } else {
        Err(DxfInsertTransformIssue::NonFiniteDerivedTransform)
    }
}

fn linear_transform(columns: [[f64; 3]; 3], value: [f64; 3]) -> [f64; 3] {
    [
        columns[0][0] * value[0] + columns[1][0] * value[1] + columns[2][0] * value[2],
        columns[0][1] * value[0] + columns[1][1] * value[1] + columns[2][1] * value[2],
        columns[0][2] * value[0] + columns[1][2] * value[1] + columns[2][2] * value[2],
    ]
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn scale3(value: [f64; 3], scale: f64) -> [f64; 3] {
    value.map(|component| component * scale)
}

fn add(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn subtract(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn canonical_double(value: f64) -> DxfDouble {
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
    DxfError::from_io(
        crate::DxfIoOperation::Read,
        &std::io::Error::from(std::io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        crate::DxfIoOperation::Read,
        &std::io::Error::from(std::io::ErrorKind::OutOfMemory),
    )
}

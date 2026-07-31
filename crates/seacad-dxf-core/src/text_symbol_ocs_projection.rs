//! Shared text-symbol extrusion normalization and finite OCS-point projection.

use crate::{
    DxfDouble, DxfInsertTransformIssue, DxfRawValueProvenance, DxfSemanticValue,
    DxfTextSymbolDoubleValue, DxfTextSymbolScalarIssue, insert_transform::OcsBasis,
};

#[derive(Clone, Copy)]
pub(crate) enum OcsProjectionComponent {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy)]
pub(crate) enum OcsPointProjectionIssue {
    NonFinitePoint,
    ExtrusionComponentAbsent {
        component: OcsProjectionComponent,
    },
    ExtrusionComponentInvalid {
        component: OcsProjectionComponent,
        issue: DxfTextSymbolScalarIssue,
    },
    NonFiniteExtrusion,
    ZeroLengthExtrusion,
    NonFiniteDerivedBasis,
    NonFiniteDerivedPoint,
}

pub(crate) type OcsPointProjectionFailure =
    (OcsPointProjectionIssue, Option<DxfRawValueProvenance>);

#[derive(Clone, Copy)]
pub(crate) struct OcsPointProjection {
    point: [DxfDouble; 3],
    normal: [DxfDouble; 3],
    raw: Option<DxfRawValueProvenance>,
}

#[derive(Clone, Copy)]
pub(crate) struct ExtrusionProjection {
    basis: OcsBasis,
    raw: Option<DxfRawValueProvenance>,
}

impl ExtrusionProjection {
    pub(crate) fn transform(self, point: [f64; 3]) -> [f64; 3] {
        self.basis.transform(point)
    }

    pub(crate) fn normal(self) -> [DxfDouble; 3] {
        self.basis.normal().map(canonical_double)
    }

    pub(crate) const fn raw(self) -> Option<DxfRawValueProvenance> {
        self.raw
    }
}

impl OcsPointProjection {
    pub(crate) const fn point(self) -> [DxfDouble; 3] {
        self.point
    }

    pub(crate) const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }

    pub(crate) const fn raw(self) -> Option<DxfRawValueProvenance> {
        self.raw
    }
}

pub(crate) fn project_ocs_point(
    point: [DxfDouble; 3],
    point_raw: Option<DxfRawValueProvenance>,
    extrusion_values: &[DxfTextSymbolDoubleValue; 3],
) -> Result<OcsPointProjection, OcsPointProjectionFailure> {
    let point = point.map(DxfDouble::to_f64);
    if !finite3(point) {
        return Err((OcsPointProjectionIssue::NonFinitePoint, point_raw));
    }
    let extrusion = project_extrusion(extrusion_values)?;
    let transformed = extrusion.transform(point);
    if !finite3(transformed) {
        return Err((
            OcsPointProjectionIssue::NonFiniteDerivedPoint,
            point_raw.or(extrusion.raw()),
        ));
    }
    Ok(OcsPointProjection {
        point: transformed.map(canonical_double),
        normal: extrusion.normal(),
        raw: point_raw.or(extrusion.raw()),
    })
}

pub(crate) fn project_extrusion(
    values: &[DxfTextSymbolDoubleValue; 3],
) -> Result<ExtrusionProjection, OcsPointProjectionFailure> {
    let extrusion = extrusion_vector(values)?;
    if !finite3(extrusion.0) {
        return Err((OcsPointProjectionIssue::NonFiniteExtrusion, extrusion.1));
    }
    let basis = OcsBasis::from_extrusion(extrusion.0)
        .map_err(|issue| (map_basis_issue(issue), extrusion.1))?;
    Ok(ExtrusionProjection {
        basis,
        raw: extrusion.1,
    })
}

fn extrusion_vector(
    values: &[DxfTextSymbolDoubleValue; 3],
) -> Result<([f64; 3], Option<DxfRawValueProvenance>), OcsPointProjectionFailure> {
    let x = extrusion_component(&values[0], OcsProjectionComponent::X)?;
    let y = extrusion_component(&values[1], OcsProjectionComponent::Y)?;
    let z = extrusion_component(&values[2], OcsProjectionComponent::Z)?;
    Ok((
        [x.0.to_f64(), y.0.to_f64(), z.0.to_f64()],
        x.1.or(y.1).or(z.1),
    ))
}

fn extrusion_component(
    value: &DxfTextSymbolDoubleValue,
    component: OcsProjectionComponent,
) -> Result<(DxfDouble, Option<DxfRawValueProvenance>), OcsPointProjectionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, raw, .. } => Ok((value, Some(raw))),
        DxfSemanticValue::Defaulted { value, .. } => Ok((value, None)),
        DxfSemanticValue::Absent { .. } => Err((
            OcsPointProjectionIssue::ExtrusionComponentAbsent { component },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            OcsPointProjectionIssue::ExtrusionComponentInvalid { component, issue },
            raw,
        )),
    }
}

fn map_basis_issue(issue: DxfInsertTransformIssue) -> OcsPointProjectionIssue {
    match issue {
        DxfInsertTransformIssue::ZeroLengthExtrusion => {
            OcsPointProjectionIssue::ZeroLengthExtrusion
        }
        _ => OcsPointProjectionIssue::NonFiniteDerivedBasis,
    }
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn canonical_double(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

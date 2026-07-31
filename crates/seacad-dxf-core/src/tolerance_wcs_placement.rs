//! Exact TOLERANCE WCS placement inputs plus normalized extrusion.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfMTextToleranceScalarDirectory, DxfRawDocumentView, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId, DxfTextSymbolDoubleValue,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue, DxfToleranceNumericSemantics,
    text_symbol_ocs_projection::{
        OcsPointProjectionIssue, OcsProjectionComponent, project_extrusion,
    },
};

const NAMESPACE: &str = "text_symbol.tolerance.wcs_placement";

/// One TOLERANCE WCS vector named by Autodesk's entity contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfToleranceWcsVector {
    Insertion,
    XAxis,
    Extrusion,
}

/// One Cartesian component of a TOLERANCE WCS vector.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfToleranceWcsComponent {
    X,
    Y,
    Z,
}

/// Why a complete TOLERANCE WCS placement cannot be constructed exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfToleranceWcsPlacementIssue {
    ComponentAbsent {
        vector: DxfToleranceWcsVector,
        component: DxfToleranceWcsComponent,
    },
    ComponentInvalid {
        vector: DxfToleranceWcsVector,
        component: DxfToleranceWcsComponent,
        issue: DxfTextSymbolScalarIssue,
    },
    NonFiniteInsertion,
    NonFiniteXAxis,
    ZeroLengthXAxis,
    NonFiniteExtrusion,
    ZeroLengthExtrusion,
    NonFiniteDerivedNormal,
}

/// Exact WCS insertion/X-axis inputs and normalized extrusion normal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfToleranceWcsPlacement {
    insertion: [DxfDouble; 3],
    x_axis: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfToleranceWcsPlacement {
    #[must_use]
    pub const fn insertion(self) -> [DxfDouble; 3] {
        self.insertion
    }

    #[must_use]
    pub const fn x_axis(self) -> [DxfDouble; 3] {
        self.x_axis
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }
}

pub type DxfToleranceWcsPlacementSemantic =
    DxfSemanticValue<DxfToleranceWcsPlacement, DxfToleranceWcsPlacementIssue>;

/// One TOLERANCE numeric projection plus its complete WCS placement.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfToleranceWcsPlacementSemantics {
    numeric: DxfToleranceNumericSemantics,
    placement: DxfToleranceWcsPlacementSemantic,
}

impl DxfToleranceWcsPlacementSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.numeric.record()
    }

    #[must_use]
    pub const fn numeric_semantics(self) -> DxfToleranceNumericSemantics {
        self.numeric
    }

    #[must_use]
    pub const fn placement(&self) -> &DxfToleranceWcsPlacementSemantic {
        &self.placement
    }
}

/// Lazy TOLERANCE WCS placement projection retaining all scalar evidence.
#[derive(Debug)]
pub struct DxfToleranceWcsPlacementDirectory {
    source_id: DxfSourceId,
    scalars: DxfMTextToleranceScalarDirectory,
}

impl DxfToleranceWcsPlacementDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let scalars = document.mtext_tolerance_scalar_directory(cancellation)?;
        if scalars.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: scalars.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            scalars,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn scalar_directory(&self) -> &DxfMTextToleranceScalarDirectory {
        &self.scalars
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.scalars.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfToleranceWcsPlacementSemantics>, DxfError> {
        let Some(numeric) = self.scalars.tolerance_semantics_for_record(record)? else {
            return Ok(None);
        };
        Ok(Some(DxfToleranceWcsPlacementSemantics {
            numeric,
            placement: project_placement(self.source_id, numeric),
        }))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfToleranceWcsPlacementSemantics>, DxfError> {
        let Some(record) = self
            .records()
            .iter()
            .copied()
            .find(|record| record.record().ordinal() == raw_record_ordinal)
        else {
            return Ok(None);
        };
        self.semantics_for_record(record)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn tolerance_wcs_placement_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfToleranceWcsPlacementDirectory, DxfError> {
        DxfToleranceWcsPlacementDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn tolerance_wcs_placement_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfToleranceWcsPlacementDirectory, DxfError> {
        DxfRawDocumentView::from(self).tolerance_wcs_placement_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn tolerance_wcs_placement_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfToleranceWcsPlacementDirectory, DxfError> {
        DxfRawDocumentView::from(self).tolerance_wcs_placement_directory(cancellation)
    }
}

fn project_placement(
    source_id: DxfSourceId,
    numeric: DxfToleranceNumericSemantics,
) -> DxfToleranceWcsPlacementSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "placement");
    let (insertion, insertion_raw) =
        match source_vector(numeric.insertion(), DxfToleranceWcsVector::Insertion) {
            Ok(value) => value,
            Err((issue, raw)) => return DxfSemanticValue::invalid(issue, field, raw),
        };
    if !finite3(insertion.map(DxfDouble::to_f64)) {
        return DxfSemanticValue::invalid(
            DxfToleranceWcsPlacementIssue::NonFiniteInsertion,
            field,
            insertion_raw,
        );
    }
    let (x_axis, x_axis_raw) = match source_vector(numeric.x_axis(), DxfToleranceWcsVector::XAxis) {
        Ok(value) => value,
        Err((issue, raw)) => return DxfSemanticValue::invalid(issue, field, raw),
    };
    let x = x_axis.map(DxfDouble::to_f64);
    if !finite3(x) {
        return DxfSemanticValue::invalid(
            DxfToleranceWcsPlacementIssue::NonFiniteXAxis,
            field,
            x_axis_raw,
        );
    }
    if x[0].hypot(x[1]).hypot(x[2]) == 0.0 {
        return DxfSemanticValue::invalid(
            DxfToleranceWcsPlacementIssue::ZeroLengthXAxis,
            field,
            x_axis_raw,
        );
    }
    let extrusion = match project_extrusion(numeric.extrusion()) {
        Ok(value) => value,
        Err((issue, raw)) => {
            return DxfSemanticValue::invalid(map_projection_issue(issue), field, raw);
        }
    };
    let value = DxfToleranceWcsPlacement {
        insertion,
        x_axis,
        normal: extrusion.normal(),
    };
    let raw = insertion_raw.or(x_axis_raw).or(extrusion.raw());
    match raw {
        Some(raw) => DxfSemanticValue::explicit(value, field, raw),
        None => DxfSemanticValue::defaulted(value, field),
    }
}

type VectorFailure = (DxfToleranceWcsPlacementIssue, Option<DxfRawValueProvenance>);

fn source_vector(
    values: &[DxfTextSymbolDoubleValue; 3],
    vector: DxfToleranceWcsVector,
) -> Result<([DxfDouble; 3], Option<DxfRawValueProvenance>), VectorFailure> {
    let x = source_component(&values[0], vector, DxfToleranceWcsComponent::X)?;
    let y = source_component(&values[1], vector, DxfToleranceWcsComponent::Y)?;
    let z = source_component(&values[2], vector, DxfToleranceWcsComponent::Z)?;
    Ok(([x.0, y.0, z.0], x.1.or(y.1).or(z.1)))
}

fn source_component(
    value: &DxfTextSymbolDoubleValue,
    vector: DxfToleranceWcsVector,
    component: DxfToleranceWcsComponent,
) -> Result<(DxfDouble, Option<DxfRawValueProvenance>), VectorFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, raw, .. } => Ok((value, Some(raw))),
        DxfSemanticValue::Defaulted { value, .. } => Ok((value, None)),
        DxfSemanticValue::Absent { .. } => Err((
            DxfToleranceWcsPlacementIssue::ComponentAbsent { vector, component },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            DxfToleranceWcsPlacementIssue::ComponentInvalid {
                vector,
                component,
                issue,
            },
            raw,
        )),
    }
}

fn map_projection_issue(issue: OcsPointProjectionIssue) -> DxfToleranceWcsPlacementIssue {
    match issue {
        OcsPointProjectionIssue::ExtrusionComponentAbsent { component } => {
            DxfToleranceWcsPlacementIssue::ComponentAbsent {
                vector: DxfToleranceWcsVector::Extrusion,
                component: map_component(component),
            }
        }
        OcsPointProjectionIssue::ExtrusionComponentInvalid { component, issue } => {
            DxfToleranceWcsPlacementIssue::ComponentInvalid {
                vector: DxfToleranceWcsVector::Extrusion,
                component: map_component(component),
                issue,
            }
        }
        OcsPointProjectionIssue::NonFiniteExtrusion => {
            DxfToleranceWcsPlacementIssue::NonFiniteExtrusion
        }
        OcsPointProjectionIssue::ZeroLengthExtrusion => {
            DxfToleranceWcsPlacementIssue::ZeroLengthExtrusion
        }
        OcsPointProjectionIssue::NonFiniteDerivedBasis => {
            DxfToleranceWcsPlacementIssue::NonFiniteDerivedNormal
        }
        OcsPointProjectionIssue::NonFinitePoint
        | OcsPointProjectionIssue::NonFiniteDerivedPoint => {
            DxfToleranceWcsPlacementIssue::NonFiniteDerivedNormal
        }
    }
}

const fn map_component(component: OcsProjectionComponent) -> DxfToleranceWcsComponent {
    match component {
        OcsProjectionComponent::X => DxfToleranceWcsComponent::X,
        OcsProjectionComponent::Y => DxfToleranceWcsComponent::Y,
        OcsProjectionComponent::Z => DxfToleranceWcsComponent::Z,
    }
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

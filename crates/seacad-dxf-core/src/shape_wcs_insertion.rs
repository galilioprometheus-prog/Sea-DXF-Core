//! Exact classic SHAPE WCS insertion points and normalized extrusion normals.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfShapeNumericSemantics, DxfSourceId, DxfTextShapeScalarDirectory, DxfTextSymbolDoubleValue,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue,
    text_symbol_ocs_projection::{
        OcsPointProjectionIssue, OcsProjectionComponent, project_extrusion,
    },
};

const NAMESPACE: &str = "text_symbol.shape.wcs_insertion";

/// Named component of a SHAPE insertion or extrusion vector.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfShapeInsertionComponent {
    X,
    Y,
    Z,
}

/// Why a SHAPE WCS insertion cannot be constructed exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfShapeWcsInsertionIssue {
    InsertionComponentAbsent {
        component: DxfShapeInsertionComponent,
    },
    InsertionComponentInvalid {
        component: DxfShapeInsertionComponent,
        issue: DxfTextSymbolScalarIssue,
    },
    NonFiniteInsertion,
    ExtrusionComponentAbsent {
        component: DxfShapeInsertionComponent,
    },
    ExtrusionComponentInvalid {
        component: DxfShapeInsertionComponent,
        issue: DxfTextSymbolScalarIssue,
    },
    NonFiniteExtrusion,
    ZeroLengthExtrusion,
    NonFiniteDerivedBasis,
}

/// Classic SHAPE WCS insertion point and normalized extrusion normal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfShapeWcsInsertion {
    point: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfShapeWcsInsertion {
    #[must_use]
    pub const fn point(self) -> [DxfDouble; 3] {
        self.point
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }
}

pub type DxfShapeWcsInsertionSemantic =
    DxfSemanticValue<DxfShapeWcsInsertion, DxfShapeWcsInsertionIssue>;

/// One SHAPE numeric projection plus its WCS insertion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfShapeWcsInsertionSemantics {
    numeric: DxfShapeNumericSemantics,
    insertion: DxfShapeWcsInsertionSemantic,
}

impl DxfShapeWcsInsertionSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.numeric.record()
    }

    #[must_use]
    pub const fn numeric_semantics(self) -> DxfShapeNumericSemantics {
        self.numeric
    }

    #[must_use]
    pub const fn wcs_insertion(&self) -> &DxfShapeWcsInsertionSemantic {
        &self.insertion
    }
}

/// Lazy SHAPE WCS projection retaining the complete scalar directory.
#[derive(Debug)]
pub struct DxfShapeWcsInsertionDirectory {
    source_id: DxfSourceId,
    scalars: DxfTextShapeScalarDirectory,
}

impl DxfShapeWcsInsertionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let scalars = document.text_shape_scalar_directory(cancellation)?;
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
    pub const fn scalar_directory(&self) -> &DxfTextShapeScalarDirectory {
        &self.scalars
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.scalars.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfShapeWcsInsertionSemantics>, DxfError> {
        let Some(numeric) = self.scalars.shape_semantics_for_record(record)? else {
            return Ok(None);
        };
        Ok(Some(DxfShapeWcsInsertionSemantics {
            numeric,
            insertion: project_insertion(self.source_id, numeric),
        }))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfShapeWcsInsertionSemantics>, DxfError> {
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
    pub fn shape_wcs_insertion_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfShapeWcsInsertionDirectory, DxfError> {
        DxfShapeWcsInsertionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn shape_wcs_insertion_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfShapeWcsInsertionDirectory, DxfError> {
        DxfRawDocumentView::from(self).shape_wcs_insertion_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn shape_wcs_insertion_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfShapeWcsInsertionDirectory, DxfError> {
        DxfRawDocumentView::from(self).shape_wcs_insertion_directory(cancellation)
    }
}

fn project_insertion(
    source_id: DxfSourceId,
    numeric: DxfShapeNumericSemantics,
) -> DxfShapeWcsInsertionSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "wcs_insertion");
    let point = match insertion_vector(numeric.insertion()) {
        Ok(point) => point,
        Err((issue, raw)) => return DxfSemanticValue::invalid(issue, field, raw),
    };
    if !point
        .0
        .iter()
        .all(|component| component.to_f64().is_finite())
    {
        return DxfSemanticValue::invalid(
            DxfShapeWcsInsertionIssue::NonFiniteInsertion,
            field,
            point.1,
        );
    }
    let extrusion = match project_extrusion(numeric.extrusion()) {
        Ok(projected) => projected,
        Err((issue, raw)) => {
            return DxfSemanticValue::invalid(map_projection_issue(issue), field, raw);
        }
    };
    let value = DxfShapeWcsInsertion {
        point: point.0,
        normal: extrusion.normal(),
    };
    match point.1.or(extrusion.raw()) {
        Some(raw) => DxfSemanticValue::explicit(value, field, raw),
        None => DxfSemanticValue::defaulted(value, field),
    }
}

type InsertionFailure = (DxfShapeWcsInsertionIssue, Option<DxfRawValueProvenance>);

fn insertion_vector(
    values: &[DxfTextSymbolDoubleValue; 3],
) -> Result<([DxfDouble; 3], Option<DxfRawValueProvenance>), InsertionFailure> {
    let x = insertion_component(&values[0], DxfShapeInsertionComponent::X)?;
    let y = insertion_component(&values[1], DxfShapeInsertionComponent::Y)?;
    let z = insertion_component(&values[2], DxfShapeInsertionComponent::Z)?;
    Ok(([x.0, y.0, z.0], x.1.or(y.1).or(z.1)))
}

fn insertion_component(
    value: &DxfTextSymbolDoubleValue,
    component: DxfShapeInsertionComponent,
) -> Result<(DxfDouble, Option<DxfRawValueProvenance>), InsertionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, raw, .. } => Ok((value, Some(raw))),
        DxfSemanticValue::Defaulted { value, .. } => Ok((value, None)),
        DxfSemanticValue::Absent { .. } => Err((
            DxfShapeWcsInsertionIssue::InsertionComponentAbsent { component },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            DxfShapeWcsInsertionIssue::InsertionComponentInvalid { component, issue },
            raw,
        )),
    }
}

fn map_projection_issue(issue: OcsPointProjectionIssue) -> DxfShapeWcsInsertionIssue {
    match issue {
        OcsPointProjectionIssue::NonFinitePoint => DxfShapeWcsInsertionIssue::NonFiniteInsertion,
        OcsPointProjectionIssue::ExtrusionComponentAbsent { component } => {
            DxfShapeWcsInsertionIssue::ExtrusionComponentAbsent {
                component: map_component(component),
            }
        }
        OcsPointProjectionIssue::ExtrusionComponentInvalid { component, issue } => {
            DxfShapeWcsInsertionIssue::ExtrusionComponentInvalid {
                component: map_component(component),
                issue,
            }
        }
        OcsPointProjectionIssue::NonFiniteExtrusion => {
            DxfShapeWcsInsertionIssue::NonFiniteExtrusion
        }
        OcsPointProjectionIssue::ZeroLengthExtrusion => {
            DxfShapeWcsInsertionIssue::ZeroLengthExtrusion
        }
        OcsPointProjectionIssue::NonFiniteDerivedBasis => {
            DxfShapeWcsInsertionIssue::NonFiniteDerivedBasis
        }
        OcsPointProjectionIssue::NonFiniteDerivedPoint => {
            DxfShapeWcsInsertionIssue::NonFiniteInsertion
        }
    }
}

const fn map_component(component: OcsProjectionComponent) -> DxfShapeInsertionComponent {
    match component {
        OcsProjectionComponent::X => DxfShapeInsertionComponent::X,
        OcsProjectionComponent::Y => DxfShapeInsertionComponent::Y,
        OcsProjectionComponent::Z => DxfShapeInsertionComponent::Z,
    }
}

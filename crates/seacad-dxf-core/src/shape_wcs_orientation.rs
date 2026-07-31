//! Classic SHAPE rotation projected onto its normalized WCS plane.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfRawDocumentView, DxfSemanticFieldProvenance, DxfSemanticValue, DxfShapeInsertionComponent,
    DxfShapeWcsInsertionDirectory, DxfShapeWcsInsertionSemantics, DxfSourceId,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue,
    text_symbol_ocs_projection::{
        OcsPointProjectionIssue, OcsProjectionComponent, project_extrusion,
    },
};

const NAMESPACE: &str = "text_symbol.shape.wcs_orientation";

/// Why a SHAPE WCS orientation cannot be constructed exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfShapeWcsOrientationIssue {
    RotationAbsent,
    RotationInvalid(DxfTextSymbolScalarIssue),
    NonFiniteRotation,
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
    NonFiniteDerivedAxes,
}

/// Rotated orthonormal axes of a classic SHAPE in WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfShapeWcsOrientation {
    x_axis: [DxfDouble; 3],
    y_axis: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfShapeWcsOrientation {
    #[must_use]
    pub const fn x_axis(self) -> [DxfDouble; 3] {
        self.x_axis
    }

    #[must_use]
    pub const fn y_axis(self) -> [DxfDouble; 3] {
        self.y_axis
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }
}

pub type DxfShapeWcsOrientationSemantic =
    DxfSemanticValue<DxfShapeWcsOrientation, DxfShapeWcsOrientationIssue>;

/// One SHAPE placement projection plus its WCS orientation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfShapeWcsOrientationSemantics {
    insertion: DxfShapeWcsInsertionSemantics,
    orientation: DxfShapeWcsOrientationSemantic,
}

impl DxfShapeWcsOrientationSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.insertion.record()
    }

    #[must_use]
    pub const fn insertion_semantics(self) -> DxfShapeWcsInsertionSemantics {
        self.insertion
    }

    #[must_use]
    pub const fn orientation(&self) -> &DxfShapeWcsOrientationSemantic {
        &self.orientation
    }
}

/// Lazy SHAPE orientation projection retaining the WCS insertion directory.
#[derive(Debug)]
pub struct DxfShapeWcsOrientationDirectory {
    source_id: DxfSourceId,
    insertions: DxfShapeWcsInsertionDirectory,
}

impl DxfShapeWcsOrientationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let insertions = document.shape_wcs_insertion_directory(cancellation)?;
        if insertions.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: insertions.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            insertions,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn insertion_directory(&self) -> &DxfShapeWcsInsertionDirectory {
        &self.insertions
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.insertions.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfShapeWcsOrientationSemantics>, DxfError> {
        let Some(insertion) = self.insertions.semantics_for_record(record)? else {
            return Ok(None);
        };
        Ok(Some(DxfShapeWcsOrientationSemantics {
            insertion,
            orientation: project_orientation(self.source_id, insertion),
        }))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfShapeWcsOrientationSemantics>, DxfError> {
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
    pub fn shape_wcs_orientation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfShapeWcsOrientationDirectory, DxfError> {
        DxfShapeWcsOrientationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn shape_wcs_orientation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfShapeWcsOrientationDirectory, DxfError> {
        DxfRawDocumentView::from(self).shape_wcs_orientation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn shape_wcs_orientation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfShapeWcsOrientationDirectory, DxfError> {
        DxfRawDocumentView::from(self).shape_wcs_orientation_directory(cancellation)
    }
}

fn project_orientation(
    source_id: DxfSourceId,
    insertion: DxfShapeWcsInsertionSemantics,
) -> DxfShapeWcsOrientationSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "rotated_axes");
    let numeric = insertion.numeric_semantics();
    let (rotation, rotation_raw) = match *numeric.rotation() {
        DxfSemanticValue::Explicit { value, raw, .. } => (value.to_f64(), Some(raw)),
        DxfSemanticValue::Defaulted { value, .. } => (value.to_f64(), None),
        DxfSemanticValue::Absent { .. } => {
            return DxfSemanticValue::invalid(
                DxfShapeWcsOrientationIssue::RotationAbsent,
                field,
                None,
            );
        }
        DxfSemanticValue::Invalid { issue, raw, .. } => {
            return DxfSemanticValue::invalid(
                DxfShapeWcsOrientationIssue::RotationInvalid(issue),
                field,
                raw,
            );
        }
    };
    if !rotation.is_finite() {
        return DxfSemanticValue::invalid(
            DxfShapeWcsOrientationIssue::NonFiniteRotation,
            field,
            rotation_raw,
        );
    }
    let extrusion = match project_extrusion(numeric.extrusion()) {
        Ok(extrusion) => extrusion,
        Err((issue, raw)) => {
            return DxfSemanticValue::invalid(map_projection_issue(issue), field, raw);
        }
    };
    let axes = extrusion.oriented_axes(rotation.to_radians(), false, false);
    if !axes
        .iter()
        .flatten()
        .all(|component| component.to_f64().is_finite())
    {
        return DxfSemanticValue::invalid(
            DxfShapeWcsOrientationIssue::NonFiniteDerivedAxes,
            field,
            rotation_raw.or(extrusion.raw()),
        );
    }
    let value = DxfShapeWcsOrientation {
        x_axis: axes[0],
        y_axis: axes[1],
        normal: extrusion.normal(),
    };
    match rotation_raw.or(extrusion.raw()) {
        Some(raw) => DxfSemanticValue::explicit(value, field, raw),
        None => DxfSemanticValue::defaulted(value, field),
    }
}

fn map_projection_issue(issue: OcsPointProjectionIssue) -> DxfShapeWcsOrientationIssue {
    match issue {
        OcsPointProjectionIssue::ExtrusionComponentAbsent { component } => {
            DxfShapeWcsOrientationIssue::ExtrusionComponentAbsent {
                component: map_component(component),
            }
        }
        OcsPointProjectionIssue::ExtrusionComponentInvalid { component, issue } => {
            DxfShapeWcsOrientationIssue::ExtrusionComponentInvalid {
                component: map_component(component),
                issue,
            }
        }
        OcsPointProjectionIssue::NonFiniteExtrusion => {
            DxfShapeWcsOrientationIssue::NonFiniteExtrusion
        }
        OcsPointProjectionIssue::ZeroLengthExtrusion => {
            DxfShapeWcsOrientationIssue::ZeroLengthExtrusion
        }
        OcsPointProjectionIssue::NonFiniteDerivedBasis => {
            DxfShapeWcsOrientationIssue::NonFiniteDerivedBasis
        }
        OcsPointProjectionIssue::NonFinitePoint
        | OcsPointProjectionIssue::NonFiniteDerivedPoint => {
            DxfShapeWcsOrientationIssue::NonFiniteDerivedAxes
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

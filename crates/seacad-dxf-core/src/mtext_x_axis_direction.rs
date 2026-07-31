//! Effective MTEXT WCS X-axis direction after source-order input selection.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfMTextOrientationDirectory, DxfMTextOrientationInput, DxfMTextOrientationIssue,
    DxfMTextOrientationSemantics, DxfRawDocumentView, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId, DxfTextSymbolDoubleValue,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue,
};

const NAMESPACE: &str = "text_symbol.mtext.x_axis_direction";

/// Named component of an MTEXT WCS X-axis direction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextXAxisComponent {
    X,
    Y,
    Z,
}

/// Why the effective MTEXT X-axis direction cannot be constructed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextXAxisDirectionIssue {
    Orientation(DxfMTextOrientationIssue),
    ComponentAbsent {
        component: DxfMTextXAxisComponent,
    },
    ComponentInvalid {
        component: DxfMTextXAxisComponent,
        issue: DxfTextSymbolScalarIssue,
    },
    ZeroLength,
    NonFiniteLength,
}

/// Effective WCS X-axis vector and its normalized direction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextXAxisDirection {
    input: DxfMTextOrientationInput,
    vector: [DxfDouble; 3],
    unit_vector: [DxfDouble; 3],
}

impl DxfMTextXAxisDirection {
    #[must_use]
    pub const fn input(self) -> DxfMTextOrientationInput {
        self.input
    }

    #[must_use]
    pub const fn vector(self) -> [DxfDouble; 3] {
        self.vector
    }

    #[must_use]
    pub const fn unit_vector(self) -> [DxfDouble; 3] {
        self.unit_vector
    }
}

pub type DxfMTextXAxisDirectionSemantic =
    DxfSemanticValue<DxfMTextXAxisDirection, DxfMTextXAxisDirectionIssue>;

/// One MTEXT orientation selection plus its effective WCS X-axis direction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextXAxisDirectionSemantics {
    orientation: DxfMTextOrientationSemantics,
    direction: DxfMTextXAxisDirectionSemantic,
}

impl DxfMTextXAxisDirectionSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.orientation.record()
    }

    #[must_use]
    pub const fn orientation_semantics(self) -> DxfMTextOrientationSemantics {
        self.orientation
    }

    #[must_use]
    pub const fn direction(&self) -> &DxfMTextXAxisDirectionSemantic {
        &self.direction
    }
}

/// Lazy MTEXT X-axis projection retaining the complete orientation directory.
#[derive(Debug)]
pub struct DxfMTextXAxisDirectionDirectory {
    source_id: DxfSourceId,
    orientations: DxfMTextOrientationDirectory,
}

impl DxfMTextXAxisDirectionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let orientations = document.mtext_orientation_directory(cancellation)?;
        if orientations.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: orientations.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            orientations,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn orientation_directory(&self) -> &DxfMTextOrientationDirectory {
        &self.orientations
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.orientations.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfMTextXAxisDirectionSemantics>, DxfError> {
        let Some(orientation) = self.orientations.semantics_for_record(record)? else {
            return Ok(None);
        };
        Ok(Some(DxfMTextXAxisDirectionSemantics {
            orientation,
            direction: project_direction(self.source_id, orientation),
        }))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfMTextXAxisDirectionSemantics>, DxfError> {
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
    pub fn mtext_x_axis_direction_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXAxisDirectionDirectory, DxfError> {
        DxfMTextXAxisDirectionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_x_axis_direction_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXAxisDirectionDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_x_axis_direction_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_x_axis_direction_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextXAxisDirectionDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_x_axis_direction_directory(cancellation)
    }
}

fn project_direction(
    source_id: DxfSourceId,
    orientation: DxfMTextOrientationSemantics,
) -> DxfMTextXAxisDirectionSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "effective_direction");
    match *orientation.effective_input() {
        DxfSemanticValue::Absent { .. } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, raw, .. } => {
            DxfSemanticValue::invalid(DxfMTextXAxisDirectionIssue::Orientation(issue), field, raw)
        }
        DxfSemanticValue::Explicit { value, raw, .. } => {
            project_selected(orientation, value, field, Some(raw))
        }
        DxfSemanticValue::Defaulted { value, .. } => {
            project_selected(orientation, value, field, None)
        }
    }
}

fn project_selected(
    orientation: DxfMTextOrientationSemantics,
    input: DxfMTextOrientationInput,
    field: DxfSemanticFieldProvenance,
    raw: Option<DxfRawValueProvenance>,
) -> DxfMTextXAxisDirectionSemantic {
    let projected = match input {
        DxfMTextOrientationInput::Rotation => rotation_direction(orientation),
        DxfMTextOrientationInput::XAxis => x_axis_direction(orientation),
    };
    match projected {
        Ok(direction) => match raw {
            Some(raw) => DxfSemanticValue::explicit(direction, field, raw),
            None => DxfSemanticValue::defaulted(direction, field),
        },
        Err((issue, issue_raw)) => DxfSemanticValue::invalid(issue, field, issue_raw.or(raw)),
    }
}

type ProjectionFailure = (DxfMTextXAxisDirectionIssue, Option<DxfRawValueProvenance>);

fn rotation_direction(
    orientation: DxfMTextOrientationSemantics,
) -> Result<DxfMTextXAxisDirection, ProjectionFailure> {
    let rotation = usable_rotation(orientation.rotation())?.to_f64();
    let vector = [
        canonical(rotation.cos()),
        canonical(rotation.sin()),
        canonical(0.0),
    ];
    Ok(DxfMTextXAxisDirection {
        input: DxfMTextOrientationInput::Rotation,
        vector,
        unit_vector: vector,
    })
}

fn x_axis_direction(
    orientation: DxfMTextOrientationSemantics,
) -> Result<DxfMTextXAxisDirection, ProjectionFailure> {
    let numeric = orientation.numeric_semantics();
    let components = [
        usable_component(&numeric.x_axis()[0], DxfMTextXAxisComponent::X)?,
        usable_component(&numeric.x_axis()[1], DxfMTextXAxisComponent::Y)?,
        usable_component(&numeric.x_axis()[2], DxfMTextXAxisComponent::Z)?,
    ];
    let vector = components.map(DxfDouble::to_f64);
    let length = vector[0].hypot(vector[1]).hypot(vector[2]);
    if length == 0.0 {
        return Err((DxfMTextXAxisDirectionIssue::ZeroLength, None));
    }
    if !length.is_finite() {
        return Err((DxfMTextXAxisDirectionIssue::NonFiniteLength, None));
    }
    let unit = vector.map(|component| component / length);
    Ok(DxfMTextXAxisDirection {
        input: DxfMTextOrientationInput::XAxis,
        vector: components,
        unit_vector: unit.map(canonical),
    })
}

fn usable_component(
    value: &DxfTextSymbolDoubleValue,
    component: DxfMTextXAxisComponent,
) -> Result<DxfDouble, ProjectionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, .. } | DxfSemanticValue::Defaulted { value, .. } => {
            Ok(value)
        }
        DxfSemanticValue::Absent { .. } => Err((
            DxfMTextXAxisDirectionIssue::ComponentAbsent { component },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            DxfMTextXAxisDirectionIssue::ComponentInvalid { component, issue },
            raw,
        )),
    }
}

fn usable_rotation(
    value: &DxfSemanticValue<DxfDouble, DxfMTextOrientationIssue>,
) -> Result<DxfDouble, ProjectionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, .. } | DxfSemanticValue::Defaulted { value, .. } => {
            Ok(value)
        }
        DxfSemanticValue::Absent { .. } => Err((
            DxfMTextXAxisDirectionIssue::Orientation(DxfMTextOrientationIssue::Scalar(
                DxfTextSymbolScalarIssue::MissingRequiredValue,
            )),
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => {
            Err((DxfMTextXAxisDirectionIssue::Orientation(issue), raw))
        }
    }
}

fn canonical(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

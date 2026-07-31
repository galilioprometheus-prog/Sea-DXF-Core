//! Classic TEXT OCS placement-anchor selection.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfSourceId, DxfTextHorizontalJustificationSemantic, DxfTextLayoutDirectory,
    DxfTextLayoutIssue, DxfTextLayoutSemantics, DxfTextSymbolDoubleValue, DxfTextSymbolRecordEntry,
    DxfTextSymbolScalarIssue, DxfTextVerticalJustificationSemantic,
};

const NAMESPACE: &str = "text_symbol.text.ocs_placement_anchor";

/// TEXT justification axis that controls placement-anchor selection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextJustificationAxis {
    Horizontal,
    Vertical,
}

/// Named component of a TEXT OCS placement point.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextPlacementComponent {
    X,
    Y,
    Z,
}

/// Which Autodesk TEXT point controls placement.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextOcsPlacementAnchorKind {
    FirstAlignment,
    SecondAlignment,
}

/// Selected TEXT placement point in the entity's object coordinate system.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextOcsPlacementAnchor {
    kind: DxfTextOcsPlacementAnchorKind,
    point: [DxfDouble; 3],
}

impl DxfTextOcsPlacementAnchor {
    #[must_use]
    pub const fn kind(self) -> DxfTextOcsPlacementAnchorKind {
        self.kind
    }

    #[must_use]
    pub const fn point(self) -> [DxfDouble; 3] {
        self.point
    }
}

/// Why a TEXT OCS placement anchor cannot be selected exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextOcsPlacementAnchorIssue {
    JustificationAbsent {
        axis: DxfTextJustificationAxis,
    },
    JustificationInvalid {
        axis: DxfTextJustificationAxis,
        issue: DxfTextLayoutIssue,
    },
    ComponentAbsent {
        anchor: DxfTextOcsPlacementAnchorKind,
        component: DxfTextPlacementComponent,
    },
    ComponentInvalid {
        anchor: DxfTextOcsPlacementAnchorKind,
        component: DxfTextPlacementComponent,
        issue: DxfTextSymbolScalarIssue,
    },
}

pub type DxfTextOcsPlacementAnchorSemantic =
    DxfSemanticValue<DxfTextOcsPlacementAnchor, DxfTextOcsPlacementAnchorIssue>;

/// One TEXT layout projection plus its selected OCS placement anchor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextOcsPlacementAnchorSemantics {
    layout: DxfTextLayoutSemantics,
    anchor: DxfTextOcsPlacementAnchorSemantic,
}

impl DxfTextOcsPlacementAnchorSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.layout.record()
    }

    #[must_use]
    pub const fn layout_semantics(self) -> DxfTextLayoutSemantics {
        self.layout
    }

    #[must_use]
    pub const fn anchor(&self) -> &DxfTextOcsPlacementAnchorSemantic {
        &self.anchor
    }
}

/// Lazy TEXT OCS anchor projection retaining the complete layout directory.
#[derive(Debug)]
pub struct DxfTextOcsPlacementAnchorDirectory {
    source_id: DxfSourceId,
    layouts: DxfTextLayoutDirectory,
}

impl DxfTextOcsPlacementAnchorDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let layouts = document.text_layout_directory(cancellation)?;
        if layouts.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: layouts.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            layouts,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn layout_directory(&self) -> &DxfTextLayoutDirectory {
        &self.layouts
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.layouts.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfTextOcsPlacementAnchorSemantics>, DxfError> {
        let Some(layout) = self.layouts.semantics_for_record(record)? else {
            return Ok(None);
        };
        Ok(Some(DxfTextOcsPlacementAnchorSemantics {
            layout,
            anchor: project_anchor(self.source_id, layout),
        }))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfTextOcsPlacementAnchorSemantics>, DxfError> {
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
    pub fn text_ocs_placement_anchor_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextOcsPlacementAnchorDirectory, DxfError> {
        DxfTextOcsPlacementAnchorDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_ocs_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextOcsPlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_ocs_placement_anchor_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_ocs_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextOcsPlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_ocs_placement_anchor_directory(cancellation)
    }
}

fn project_anchor(
    source_id: DxfSourceId,
    layout: DxfTextLayoutSemantics,
) -> DxfTextOcsPlacementAnchorSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "selected_anchor");
    let kind = match selected_kind(layout) {
        Ok(kind) => kind,
        Err((issue, raw)) => return DxfSemanticValue::invalid(issue, field, raw),
    };
    let numeric = layout.numeric_semantics();
    let components = match kind {
        DxfTextOcsPlacementAnchorKind::FirstAlignment => numeric.first_alignment(),
        DxfTextOcsPlacementAnchorKind::SecondAlignment => numeric.second_alignment(),
    };
    let projected = selected_point(components, kind);
    match projected {
        Ok((point, Some(raw))) => {
            DxfSemanticValue::explicit(DxfTextOcsPlacementAnchor { kind, point }, field, raw)
        }
        Ok((point, None)) => {
            DxfSemanticValue::defaulted(DxfTextOcsPlacementAnchor { kind, point }, field)
        }
        Err((issue, raw)) => DxfSemanticValue::invalid(issue, field, raw),
    }
}

type ProjectionFailure = (
    DxfTextOcsPlacementAnchorIssue,
    Option<DxfRawValueProvenance>,
);

fn selected_kind(
    layout: DxfTextLayoutSemantics,
) -> Result<DxfTextOcsPlacementAnchorKind, ProjectionFailure> {
    let horizontal = horizontal_code(layout.horizontal())?;
    let vertical = vertical_code(layout.vertical())?;
    if horizontal == 0 && vertical == 0 {
        Ok(DxfTextOcsPlacementAnchorKind::FirstAlignment)
    } else {
        Ok(DxfTextOcsPlacementAnchorKind::SecondAlignment)
    }
}

fn horizontal_code(
    value: &DxfTextHorizontalJustificationSemantic,
) -> Result<i16, ProjectionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, .. } | DxfSemanticValue::Defaulted { value, .. } => {
            Ok(value.code())
        }
        DxfSemanticValue::Absent { .. } => Err((
            DxfTextOcsPlacementAnchorIssue::JustificationAbsent {
                axis: DxfTextJustificationAxis::Horizontal,
            },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            DxfTextOcsPlacementAnchorIssue::JustificationInvalid {
                axis: DxfTextJustificationAxis::Horizontal,
                issue,
            },
            raw,
        )),
    }
}

fn vertical_code(value: &DxfTextVerticalJustificationSemantic) -> Result<i16, ProjectionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, .. } | DxfSemanticValue::Defaulted { value, .. } => {
            Ok(value.code())
        }
        DxfSemanticValue::Absent { .. } => Err((
            DxfTextOcsPlacementAnchorIssue::JustificationAbsent {
                axis: DxfTextJustificationAxis::Vertical,
            },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            DxfTextOcsPlacementAnchorIssue::JustificationInvalid {
                axis: DxfTextJustificationAxis::Vertical,
                issue,
            },
            raw,
        )),
    }
}

fn selected_point(
    components: &[DxfTextSymbolDoubleValue; 3],
    anchor: DxfTextOcsPlacementAnchorKind,
) -> Result<([DxfDouble; 3], Option<DxfRawValueProvenance>), ProjectionFailure> {
    let x = usable_component(&components[0], anchor, DxfTextPlacementComponent::X)?;
    let y = usable_component(&components[1], anchor, DxfTextPlacementComponent::Y)?;
    let z = usable_component(&components[2], anchor, DxfTextPlacementComponent::Z)?;
    Ok(([x.0, y.0, z.0], x.1.or(y.1).or(z.1)))
}

fn usable_component(
    value: &DxfTextSymbolDoubleValue,
    anchor: DxfTextOcsPlacementAnchorKind,
    component: DxfTextPlacementComponent,
) -> Result<(DxfDouble, Option<DxfRawValueProvenance>), ProjectionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, raw, .. } => Ok((value, Some(raw))),
        DxfSemanticValue::Defaulted { value, .. } => Ok((value, None)),
        DxfSemanticValue::Absent { .. } => Err((
            DxfTextOcsPlacementAnchorIssue::ComponentAbsent { anchor, component },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            DxfTextOcsPlacementAnchorIssue::ComponentInvalid {
                anchor,
                component,
                issue,
            },
            raw,
        )),
    }
}

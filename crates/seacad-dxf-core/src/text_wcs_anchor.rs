//! OCS-to-WCS projection for selected classic TEXT placement anchors.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfRawDocumentView, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    DxfTextOcsPlacementAnchorDirectory, DxfTextOcsPlacementAnchorIssue,
    DxfTextOcsPlacementAnchorKind, DxfTextOcsPlacementAnchorSemantics, DxfTextSymbolRecordEntry,
    DxfTextSymbolScalarIssue,
    text_symbol_ocs_projection::{
        OcsPointProjectionIssue, OcsProjectionComponent, project_ocs_point,
    },
};

const NAMESPACE: &str = "text_symbol.text.wcs_placement_anchor";

/// Named component of the TEXT extrusion vector.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextExtrusionComponent {
    X,
    Y,
    Z,
}

/// Why a TEXT WCS placement anchor cannot be constructed exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextWcsPlacementAnchorIssue {
    OcsAnchorAbsent,
    OcsAnchorInvalid(DxfTextOcsPlacementAnchorIssue),
    NonFiniteOcsAnchor,
    ExtrusionComponentAbsent {
        component: DxfTextExtrusionComponent,
    },
    ExtrusionComponentInvalid {
        component: DxfTextExtrusionComponent,
        issue: DxfTextSymbolScalarIssue,
    },
    NonFiniteExtrusion,
    ZeroLengthExtrusion,
    NonFiniteDerivedBasis,
    NonFiniteDerivedPoint,
}

/// Selected TEXT placement anchor transformed into WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextWcsPlacementAnchor {
    kind: DxfTextOcsPlacementAnchorKind,
    point: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfTextWcsPlacementAnchor {
    #[must_use]
    pub const fn kind(self) -> DxfTextOcsPlacementAnchorKind {
        self.kind
    }

    #[must_use]
    pub const fn point(self) -> [DxfDouble; 3] {
        self.point
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }
}

pub type DxfTextWcsPlacementAnchorSemantic =
    DxfSemanticValue<DxfTextWcsPlacementAnchor, DxfTextWcsPlacementAnchorIssue>;

/// One TEXT OCS anchor projection plus its WCS result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextWcsPlacementAnchorSemantics {
    ocs: DxfTextOcsPlacementAnchorSemantics,
    wcs: DxfTextWcsPlacementAnchorSemantic,
}

impl DxfTextWcsPlacementAnchorSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.ocs.record()
    }

    #[must_use]
    pub const fn ocs_semantics(self) -> DxfTextOcsPlacementAnchorSemantics {
        self.ocs
    }

    #[must_use]
    pub const fn wcs_anchor(&self) -> &DxfTextWcsPlacementAnchorSemantic {
        &self.wcs
    }
}

/// Lazy TEXT WCS projection retaining the complete OCS-anchor directory.
#[derive(Debug)]
pub struct DxfTextWcsPlacementAnchorDirectory {
    source_id: DxfSourceId,
    ocs: DxfTextOcsPlacementAnchorDirectory,
}

impl DxfTextWcsPlacementAnchorDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let ocs = document.text_ocs_placement_anchor_directory(cancellation)?;
        if ocs.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: ocs.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            ocs,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn ocs_directory(&self) -> &DxfTextOcsPlacementAnchorDirectory {
        &self.ocs
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.ocs.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfTextWcsPlacementAnchorSemantics>, DxfError> {
        let Some(ocs) = self.ocs.semantics_for_record(record)? else {
            return Ok(None);
        };
        Ok(Some(DxfTextWcsPlacementAnchorSemantics {
            ocs,
            wcs: project_wcs(self.source_id, ocs),
        }))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfTextWcsPlacementAnchorSemantics>, DxfError> {
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
    pub fn text_wcs_placement_anchor_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextWcsPlacementAnchorDirectory, DxfError> {
        DxfTextWcsPlacementAnchorDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_wcs_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextWcsPlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_wcs_placement_anchor_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_wcs_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextWcsPlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_wcs_placement_anchor_directory(cancellation)
    }
}

fn project_wcs(
    source_id: DxfSourceId,
    semantics: DxfTextOcsPlacementAnchorSemantics,
) -> DxfTextWcsPlacementAnchorSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "transformed_anchor");
    let (anchor, anchor_raw) = match *semantics.anchor() {
        DxfSemanticValue::Explicit { value, raw, .. } => (value, Some(raw)),
        DxfSemanticValue::Defaulted { value, .. } => (value, None),
        DxfSemanticValue::Absent { .. } => {
            return DxfSemanticValue::invalid(
                DxfTextWcsPlacementAnchorIssue::OcsAnchorAbsent,
                field,
                None,
            );
        }
        DxfSemanticValue::Invalid { issue, raw, .. } => {
            return DxfSemanticValue::invalid(
                DxfTextWcsPlacementAnchorIssue::OcsAnchorInvalid(issue),
                field,
                raw,
            );
        }
    };
    let numeric = semantics.layout_semantics().numeric_semantics();
    let projected = match project_ocs_point(anchor.point(), anchor_raw, numeric.extrusion()) {
        Ok(projected) => projected,
        Err((issue, raw)) => {
            return DxfSemanticValue::invalid(map_projection_issue(issue), field, raw);
        }
    };
    let value = DxfTextWcsPlacementAnchor {
        kind: anchor.kind(),
        point: projected.point(),
        normal: projected.normal(),
    };
    match projected.raw() {
        Some(raw) => DxfSemanticValue::explicit(value, field, raw),
        None => DxfSemanticValue::defaulted(value, field),
    }
}

fn map_projection_issue(issue: OcsPointProjectionIssue) -> DxfTextWcsPlacementAnchorIssue {
    match issue {
        OcsPointProjectionIssue::NonFinitePoint => {
            DxfTextWcsPlacementAnchorIssue::NonFiniteOcsAnchor
        }
        OcsPointProjectionIssue::ExtrusionComponentAbsent { component } => {
            DxfTextWcsPlacementAnchorIssue::ExtrusionComponentAbsent {
                component: map_component(component),
            }
        }
        OcsPointProjectionIssue::ExtrusionComponentInvalid { component, issue } => {
            DxfTextWcsPlacementAnchorIssue::ExtrusionComponentInvalid {
                component: map_component(component),
                issue,
            }
        }
        OcsPointProjectionIssue::NonFiniteExtrusion => {
            DxfTextWcsPlacementAnchorIssue::NonFiniteExtrusion
        }
        OcsPointProjectionIssue::ZeroLengthExtrusion => {
            DxfTextWcsPlacementAnchorIssue::ZeroLengthExtrusion
        }
        OcsPointProjectionIssue::NonFiniteDerivedBasis => {
            DxfTextWcsPlacementAnchorIssue::NonFiniteDerivedBasis
        }
        OcsPointProjectionIssue::NonFiniteDerivedPoint => {
            DxfTextWcsPlacementAnchorIssue::NonFiniteDerivedPoint
        }
    }
}

const fn map_component(component: OcsProjectionComponent) -> DxfTextExtrusionComponent {
    match component {
        OcsProjectionComponent::X => DxfTextExtrusionComponent::X,
        OcsProjectionComponent::Y => DxfTextExtrusionComponent::Y,
        OcsProjectionComponent::Z => DxfTextExtrusionComponent::Z,
    }
}

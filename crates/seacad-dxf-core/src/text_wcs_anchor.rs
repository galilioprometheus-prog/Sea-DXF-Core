//! OCS-to-WCS projection for selected classic TEXT placement anchors.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertTransformIssue, DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticFieldProvenance,
    DxfSemanticValue, DxfSourceId, DxfTextOcsPlacementAnchorDirectory,
    DxfTextOcsPlacementAnchorIssue, DxfTextOcsPlacementAnchorKind,
    DxfTextOcsPlacementAnchorSemantics, DxfTextSymbolDoubleValue, DxfTextSymbolRecordEntry,
    DxfTextSymbolScalarIssue, insert_transform::OcsBasis,
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
    let point = anchor.point().map(DxfDouble::to_f64);
    if !finite3(point) {
        return DxfSemanticValue::invalid(
            DxfTextWcsPlacementAnchorIssue::NonFiniteOcsAnchor,
            field,
            anchor_raw,
        );
    }
    let numeric = semantics.layout_semantics().numeric_semantics();
    let extrusion = match extrusion_vector(numeric.extrusion()) {
        Ok(extrusion) => extrusion,
        Err((issue, raw)) => return DxfSemanticValue::invalid(issue, field, raw),
    };
    if !finite3(extrusion.0) {
        return DxfSemanticValue::invalid(
            DxfTextWcsPlacementAnchorIssue::NonFiniteExtrusion,
            field,
            extrusion.1,
        );
    }
    let basis = match OcsBasis::from_extrusion(extrusion.0) {
        Ok(basis) => basis,
        Err(issue) => {
            return DxfSemanticValue::invalid(map_basis_issue(issue), field, extrusion.1);
        }
    };
    let transformed = basis.transform(point);
    if !finite3(transformed) {
        return DxfSemanticValue::invalid(
            DxfTextWcsPlacementAnchorIssue::NonFiniteDerivedPoint,
            field,
            anchor_raw.or(extrusion.1),
        );
    }
    let value = DxfTextWcsPlacementAnchor {
        kind: anchor.kind(),
        point: transformed.map(canonical_double),
        normal: basis.normal().map(canonical_double),
    };
    match anchor_raw.or(extrusion.1) {
        Some(raw) => DxfSemanticValue::explicit(value, field, raw),
        None => DxfSemanticValue::defaulted(value, field),
    }
}

type ProjectionFailure = (
    DxfTextWcsPlacementAnchorIssue,
    Option<DxfRawValueProvenance>,
);

fn extrusion_vector(
    values: &[DxfTextSymbolDoubleValue; 3],
) -> Result<([f64; 3], Option<DxfRawValueProvenance>), ProjectionFailure> {
    let x = extrusion_component(&values[0], DxfTextExtrusionComponent::X)?;
    let y = extrusion_component(&values[1], DxfTextExtrusionComponent::Y)?;
    let z = extrusion_component(&values[2], DxfTextExtrusionComponent::Z)?;
    Ok((
        [x.0.to_f64(), y.0.to_f64(), z.0.to_f64()],
        x.1.or(y.1).or(z.1),
    ))
}

fn extrusion_component(
    value: &DxfTextSymbolDoubleValue,
    component: DxfTextExtrusionComponent,
) -> Result<(DxfDouble, Option<DxfRawValueProvenance>), ProjectionFailure> {
    match *value {
        DxfSemanticValue::Explicit { value, raw, .. } => Ok((value, Some(raw))),
        DxfSemanticValue::Defaulted { value, .. } => Ok((value, None)),
        DxfSemanticValue::Absent { .. } => Err((
            DxfTextWcsPlacementAnchorIssue::ExtrusionComponentAbsent { component },
            None,
        )),
        DxfSemanticValue::Invalid { issue, raw, .. } => Err((
            DxfTextWcsPlacementAnchorIssue::ExtrusionComponentInvalid { component, issue },
            raw,
        )),
    }
}

fn map_basis_issue(issue: DxfInsertTransformIssue) -> DxfTextWcsPlacementAnchorIssue {
    match issue {
        DxfInsertTransformIssue::ZeroLengthExtrusion => {
            DxfTextWcsPlacementAnchorIssue::ZeroLengthExtrusion
        }
        _ => DxfTextWcsPlacementAnchorIssue::NonFiniteDerivedBasis,
    }
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn canonical_double(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

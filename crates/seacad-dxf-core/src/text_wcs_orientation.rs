//! Classic TEXT rotation and generation flags projected into WCS axes.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfRawDocumentView, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    DxfTextExtrusionComponent, DxfTextGenerationFlags, DxfTextLayoutIssue,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue, DxfTextWcsPlacementAnchorDirectory,
    DxfTextWcsPlacementAnchorSemantics,
    text_symbol_ocs_projection::{
        OcsPointProjectionIssue, OcsProjectionComponent, project_extrusion,
    },
};

const NAMESPACE: &str = "text_symbol.text.wcs_orientation";

/// Why a TEXT WCS orientation cannot be constructed exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextWcsOrientationIssue {
    RotationAbsent,
    RotationInvalid(DxfTextSymbolScalarIssue),
    NonFiniteRotation,
    GenerationFlagsAbsent,
    GenerationFlagsInvalid(DxfTextLayoutIssue),
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
    NonFiniteDerivedAxes,
}

/// Rotated and optionally mirrored glyph axes of a classic TEXT entity in WCS.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextWcsOrientation {
    x_axis: [DxfDouble; 3],
    y_axis: [DxfDouble; 3],
    normal: [DxfDouble; 3],
    generation_flags: DxfTextGenerationFlags,
}

impl DxfTextWcsOrientation {
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

    #[must_use]
    pub const fn generation_flags(self) -> DxfTextGenerationFlags {
        self.generation_flags
    }
}

pub type DxfTextWcsOrientationSemantic =
    DxfSemanticValue<DxfTextWcsOrientation, DxfTextWcsOrientationIssue>;

/// One TEXT placement projection plus its WCS glyph orientation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextWcsOrientationSemantics {
    placement: DxfTextWcsPlacementAnchorSemantics,
    orientation: DxfTextWcsOrientationSemantic,
}

impl DxfTextWcsOrientationSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.placement.record()
    }

    #[must_use]
    pub const fn placement_semantics(self) -> DxfTextWcsPlacementAnchorSemantics {
        self.placement
    }

    #[must_use]
    pub const fn orientation(&self) -> &DxfTextWcsOrientationSemantic {
        &self.orientation
    }
}

/// Lazy TEXT orientation projection retaining the WCS placement directory.
#[derive(Debug)]
pub struct DxfTextWcsOrientationDirectory {
    source_id: DxfSourceId,
    placements: DxfTextWcsPlacementAnchorDirectory,
}

impl DxfTextWcsOrientationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let placements = document.text_wcs_placement_anchor_directory(cancellation)?;
        if placements.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: placements.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            placements,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn placement_directory(&self) -> &DxfTextWcsPlacementAnchorDirectory {
        &self.placements
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.placements.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfTextWcsOrientationSemantics>, DxfError> {
        let Some(placement) = self.placements.semantics_for_record(record)? else {
            return Ok(None);
        };
        Ok(Some(DxfTextWcsOrientationSemantics {
            placement,
            orientation: project_orientation(self.source_id, placement),
        }))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfTextWcsOrientationSemantics>, DxfError> {
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
    pub fn text_wcs_orientation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextWcsOrientationDirectory, DxfError> {
        DxfTextWcsOrientationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_wcs_orientation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextWcsOrientationDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_wcs_orientation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_wcs_orientation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextWcsOrientationDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_wcs_orientation_directory(cancellation)
    }
}

fn project_orientation(
    source_id: DxfSourceId,
    placement: DxfTextWcsPlacementAnchorSemantics,
) -> DxfTextWcsOrientationSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "glyph_axes");
    let layout = placement.ocs_semantics().layout_semantics();
    let numeric = layout.numeric_semantics();
    let (rotation, rotation_raw) = match *numeric.rotation() {
        DxfSemanticValue::Explicit { value, raw, .. } => (value.to_f64(), Some(raw)),
        DxfSemanticValue::Defaulted { value, .. } => (value.to_f64(), None),
        DxfSemanticValue::Absent { .. } => {
            return DxfSemanticValue::invalid(
                DxfTextWcsOrientationIssue::RotationAbsent,
                field,
                None,
            );
        }
        DxfSemanticValue::Invalid { issue, raw, .. } => {
            return DxfSemanticValue::invalid(
                DxfTextWcsOrientationIssue::RotationInvalid(issue),
                field,
                raw,
            );
        }
    };
    if !rotation.is_finite() {
        return DxfSemanticValue::invalid(
            DxfTextWcsOrientationIssue::NonFiniteRotation,
            field,
            rotation_raw,
        );
    }
    let (flags, flags_raw) = match *layout.generation_flags() {
        DxfSemanticValue::Explicit { value, raw, .. } => (value, Some(raw)),
        DxfSemanticValue::Defaulted { value, .. } => (value, None),
        DxfSemanticValue::Absent { .. } => {
            return DxfSemanticValue::invalid(
                DxfTextWcsOrientationIssue::GenerationFlagsAbsent,
                field,
                None,
            );
        }
        DxfSemanticValue::Invalid { issue, raw, .. } => {
            return DxfSemanticValue::invalid(
                DxfTextWcsOrientationIssue::GenerationFlagsInvalid(issue),
                field,
                raw,
            );
        }
    };
    let extrusion = match project_extrusion(numeric.extrusion()) {
        Ok(extrusion) => extrusion,
        Err((issue, raw)) => {
            return DxfSemanticValue::invalid(map_projection_issue(issue), field, raw);
        }
    };
    let axes = extrusion.oriented_axes(
        rotation.to_radians(),
        flags.is_backward(),
        flags.is_upside_down(),
    );
    if !axes
        .iter()
        .flatten()
        .all(|component| component.to_f64().is_finite())
    {
        return DxfSemanticValue::invalid(
            DxfTextWcsOrientationIssue::NonFiniteDerivedAxes,
            field,
            rotation_raw.or(flags_raw).or(extrusion.raw()),
        );
    }
    let value = DxfTextWcsOrientation {
        x_axis: axes[0],
        y_axis: axes[1],
        normal: extrusion.normal(),
        generation_flags: flags,
    };
    match rotation_raw.or(flags_raw).or(extrusion.raw()) {
        Some(raw) => DxfSemanticValue::explicit(value, field, raw),
        None => DxfSemanticValue::defaulted(value, field),
    }
}

fn map_projection_issue(issue: OcsPointProjectionIssue) -> DxfTextWcsOrientationIssue {
    match issue {
        OcsPointProjectionIssue::ExtrusionComponentAbsent { component } => {
            DxfTextWcsOrientationIssue::ExtrusionComponentAbsent {
                component: map_component(component),
            }
        }
        OcsPointProjectionIssue::ExtrusionComponentInvalid { component, issue } => {
            DxfTextWcsOrientationIssue::ExtrusionComponentInvalid {
                component: map_component(component),
                issue,
            }
        }
        OcsPointProjectionIssue::NonFiniteExtrusion => {
            DxfTextWcsOrientationIssue::NonFiniteExtrusion
        }
        OcsPointProjectionIssue::ZeroLengthExtrusion => {
            DxfTextWcsOrientationIssue::ZeroLengthExtrusion
        }
        OcsPointProjectionIssue::NonFiniteDerivedBasis => {
            DxfTextWcsOrientationIssue::NonFiniteDerivedBasis
        }
        OcsPointProjectionIssue::NonFinitePoint
        | OcsPointProjectionIssue::NonFiniteDerivedPoint => {
            DxfTextWcsOrientationIssue::NonFiniteDerivedAxes
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

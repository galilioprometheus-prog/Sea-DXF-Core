//! OCS-to-WCS projection for selected classic ATTDEF placement anchors.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockAttributeDefinitionPlacementAnchor,
    DxfBlockAttributeDefinitionPlacementAnchorDirectory,
    DxfBlockAttributeDefinitionPlacementAnchorState, DxfBlockAttributeDefinitionValueEntry,
    DxfCancellationToken, DxfDouble, DxfError, DxfInsertTransformIssue, DxfRawDocumentView,
    DxfSourceId, insert_transform::OcsBasis,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionWcsAnchorIssue {
    PlacementUnavailable(DxfBlockAttributeDefinitionPlacementAnchorState),
    ExtrusionUnavailable,
    NonFinitePlacement,
    NonFiniteExtrusion,
    ZeroLengthExtrusion,
    NonFiniteDerivedBasis,
    NonFiniteDerivedPoint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionWcsAnchor {
    point: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfBlockAttributeDefinitionWcsAnchor {
    #[must_use]
    pub const fn point(self) -> [DxfDouble; 3] {
        self.point
    }

    #[must_use]
    pub const fn normal(self) -> [DxfDouble; 3] {
        self.normal
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionWcsAnchorEntry {
    placement: DxfBlockAttributeDefinitionPlacementAnchor,
    projection:
        Result<DxfBlockAttributeDefinitionWcsAnchor, DxfBlockAttributeDefinitionWcsAnchorIssue>,
}

impl DxfBlockAttributeDefinitionWcsAnchorEntry {
    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.placement.record()
    }

    #[must_use]
    pub const fn placement(self) -> DxfBlockAttributeDefinitionPlacementAnchor {
        self.placement
    }

    pub const fn projection(
        self,
    ) -> Result<DxfBlockAttributeDefinitionWcsAnchor, DxfBlockAttributeDefinitionWcsAnchorIssue>
    {
        self.projection
    }
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionWcsAnchorDirectory {
    source_id: DxfSourceId,
    placements: DxfBlockAttributeDefinitionPlacementAnchorDirectory,
}

impl DxfBlockAttributeDefinitionWcsAnchorDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let placements =
            document.block_attribute_definition_placement_anchor_directory(cancellation)?;
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
    pub const fn placement_directory(
        &self,
    ) -> &DxfBlockAttributeDefinitionPlacementAnchorDirectory {
        &self.placements
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBlockAttributeDefinitionValueEntry] {
        self.placements.records()
    }

    pub fn entry_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionWcsAnchorEntry>, DxfError> {
        Ok(self
            .placements
            .anchor_for_raw_record(raw_record_ordinal)?
            .map(wcs_entry))
    }

    pub fn entry_for_definition(
        &self,
        record: DxfBlockAttributeDefinitionValueEntry,
    ) -> Result<Option<DxfBlockAttributeDefinitionWcsAnchorEntry>, DxfError> {
        Ok(self.placements.anchor_for_entry(record)?.map(wcs_entry))
    }

    pub fn entry_for_block_attribute_definition(
        &self,
        block_raw_ordinal: u64,
        attribute_definition_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionWcsAnchorEntry>, DxfError> {
        Ok(self
            .placements
            .anchor_for_block_attribute_definition(block_raw_ordinal, attribute_definition_ordinal)?
            .map(wcs_entry))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_attribute_definition_wcs_anchor_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionWcsAnchorDirectory, DxfError> {
        DxfBlockAttributeDefinitionWcsAnchorDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_wcs_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionWcsAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_wcs_anchor_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_wcs_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionWcsAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_wcs_anchor_directory(cancellation)
    }
}

fn wcs_entry(
    placement: DxfBlockAttributeDefinitionPlacementAnchor,
) -> DxfBlockAttributeDefinitionWcsAnchorEntry {
    DxfBlockAttributeDefinitionWcsAnchorEntry {
        placement,
        projection: project_wcs(placement),
    }
}

fn project_wcs(
    placement: DxfBlockAttributeDefinitionPlacementAnchor,
) -> Result<DxfBlockAttributeDefinitionWcsAnchor, DxfBlockAttributeDefinitionWcsAnchorIssue> {
    let state = placement.state();
    let point = state
        .point()
        .ok_or(DxfBlockAttributeDefinitionWcsAnchorIssue::PlacementUnavailable(state))?
        .map(DxfDouble::to_f64);
    if !finite3(point) {
        return Err(DxfBlockAttributeDefinitionWcsAnchorIssue::NonFinitePlacement);
    }
    let extrusion = placement
        .double_semantics()
        .extrusion_value()
        .ok_or(DxfBlockAttributeDefinitionWcsAnchorIssue::ExtrusionUnavailable)?
        .map(DxfDouble::to_f64);
    if !finite3(extrusion) {
        return Err(DxfBlockAttributeDefinitionWcsAnchorIssue::NonFiniteExtrusion);
    }
    let basis = OcsBasis::from_extrusion(extrusion).map_err(map_basis_issue)?;
    let transformed = basis.transform(point);
    if !finite3(transformed) {
        return Err(DxfBlockAttributeDefinitionWcsAnchorIssue::NonFiniteDerivedPoint);
    }
    Ok(DxfBlockAttributeDefinitionWcsAnchor {
        point: transformed.map(canonical_double),
        normal: basis.normal().map(canonical_double),
    })
}

fn map_basis_issue(issue: DxfInsertTransformIssue) -> DxfBlockAttributeDefinitionWcsAnchorIssue {
    match issue {
        DxfInsertTransformIssue::ZeroLengthExtrusion => {
            DxfBlockAttributeDefinitionWcsAnchorIssue::ZeroLengthExtrusion
        }
        _ => DxfBlockAttributeDefinitionWcsAnchorIssue::NonFiniteDerivedBasis,
    }
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn canonical_double(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

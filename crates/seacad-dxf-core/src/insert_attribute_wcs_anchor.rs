//! OCS-to-WCS projection for selected classic ATTRIB placement anchors.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertAttributePlacementAnchor, DxfInsertAttributePlacementAnchorDirectory,
    DxfInsertAttributePlacementAnchorState, DxfInsertAttributeValueEntry, DxfInsertTransformIssue,
    DxfRawDocumentView, DxfSourceId, insert_transform::OcsBasis,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeWcsAnchorIssue {
    PlacementUnavailable(DxfInsertAttributePlacementAnchorState),
    ExtrusionUnavailable,
    NonFinitePlacement,
    NonFiniteExtrusion,
    ZeroLengthExtrusion,
    NonFiniteDerivedBasis,
    NonFiniteDerivedPoint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeWcsAnchor {
    point: [DxfDouble; 3],
    normal: [DxfDouble; 3],
}

impl DxfInsertAttributeWcsAnchor {
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
pub struct DxfInsertAttributeWcsAnchorEntry {
    placement: DxfInsertAttributePlacementAnchor,
    projection: Result<DxfInsertAttributeWcsAnchor, DxfInsertAttributeWcsAnchorIssue>,
}

impl DxfInsertAttributeWcsAnchorEntry {
    #[must_use]
    pub const fn record(self) -> DxfInsertAttributeValueEntry {
        self.placement.record()
    }

    #[must_use]
    pub const fn placement(self) -> DxfInsertAttributePlacementAnchor {
        self.placement
    }

    pub const fn projection(
        self,
    ) -> Result<DxfInsertAttributeWcsAnchor, DxfInsertAttributeWcsAnchorIssue> {
        self.projection
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributeWcsAnchorDirectory {
    source_id: DxfSourceId,
    placements: DxfInsertAttributePlacementAnchorDirectory,
}

impl DxfInsertAttributeWcsAnchorDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let placements = document.insert_attribute_placement_anchor_directory(cancellation)?;
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
    pub const fn placement_directory(&self) -> &DxfInsertAttributePlacementAnchorDirectory {
        &self.placements
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInsertAttributeValueEntry] {
        self.placements.records()
    }

    pub fn entry_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributeWcsAnchorEntry>, DxfError> {
        Ok(self
            .placements
            .anchor_for_raw_record(raw_record_ordinal)?
            .map(wcs_entry))
    }

    pub fn entry_for_attribute(
        &self,
        record: DxfInsertAttributeValueEntry,
    ) -> Result<Option<DxfInsertAttributeWcsAnchorEntry>, DxfError> {
        Ok(self.placements.anchor_for_entry(record)?.map(wcs_entry))
    }

    pub fn entry_for_insert_sequence_attribute(
        &self,
        insert_raw_ordinal: u64,
        sequence_attribute_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributeWcsAnchorEntry>, DxfError> {
        Ok(self
            .placements
            .anchor_for_insert_sequence_attribute(insert_raw_ordinal, sequence_attribute_ordinal)?
            .map(wcs_entry))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_attribute_wcs_anchor_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeWcsAnchorDirectory, DxfError> {
        DxfInsertAttributeWcsAnchorDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_wcs_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeWcsAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_wcs_anchor_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_wcs_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeWcsAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_wcs_anchor_directory(cancellation)
    }
}

fn wcs_entry(placement: DxfInsertAttributePlacementAnchor) -> DxfInsertAttributeWcsAnchorEntry {
    DxfInsertAttributeWcsAnchorEntry {
        placement,
        projection: project_wcs(placement),
    }
}

fn project_wcs(
    placement: DxfInsertAttributePlacementAnchor,
) -> Result<DxfInsertAttributeWcsAnchor, DxfInsertAttributeWcsAnchorIssue> {
    let state = placement.state();
    let point = state
        .point()
        .ok_or(DxfInsertAttributeWcsAnchorIssue::PlacementUnavailable(
            state,
        ))?
        .map(DxfDouble::to_f64);
    if !finite3(point) {
        return Err(DxfInsertAttributeWcsAnchorIssue::NonFinitePlacement);
    }
    let extrusion = placement
        .double_semantics()
        .extrusion_value()
        .ok_or(DxfInsertAttributeWcsAnchorIssue::ExtrusionUnavailable)?
        .map(DxfDouble::to_f64);
    if !finite3(extrusion) {
        return Err(DxfInsertAttributeWcsAnchorIssue::NonFiniteExtrusion);
    }
    let basis = OcsBasis::from_extrusion(extrusion).map_err(map_basis_issue)?;
    let transformed = basis.transform(point);
    if !finite3(transformed) {
        return Err(DxfInsertAttributeWcsAnchorIssue::NonFiniteDerivedPoint);
    }
    Ok(DxfInsertAttributeWcsAnchor {
        point: transformed.map(canonical_double),
        normal: basis.normal().map(canonical_double),
    })
}

fn map_basis_issue(issue: DxfInsertTransformIssue) -> DxfInsertAttributeWcsAnchorIssue {
    match issue {
        DxfInsertTransformIssue::ZeroLengthExtrusion => {
            DxfInsertAttributeWcsAnchorIssue::ZeroLengthExtrusion
        }
        _ => DxfInsertAttributeWcsAnchorIssue::NonFiniteDerivedBasis,
    }
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn canonical_double(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

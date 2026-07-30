//! Finite WCS geometry for 3DFACE, SOLID, and TRACE records.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertTransformIssue, DxfIoOperation, DxfPlanarFaceKind, DxfPlanarFaceRecordEntry,
    DxfPlanarFaceSemanticDirectory, DxfPlanarFaceSemantics, DxfRawDocumentView, DxfSourceId,
    insert_transform::OcsBasis,
};

/// Why one typed planar-face record cannot produce finite WCS geometry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPlanarFaceWcsGeometryIssue {
    CornerUnavailable { corner_index: u8 },
    NonFiniteCorner { corner_index: u8 },
    ThicknessUnavailable,
    NonFiniteThickness,
    ExtrusionUnavailable,
    NonFiniteExtrusion,
    ZeroLengthExtrusion,
    NonFiniteDerivedGeometry,
}

/// One reviewed planar face in perimeter order and WCS coordinates.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceWcsGeometry {
    kind: DxfPlanarFaceKind,
    corners: [[DxfDouble; 3]; 4],
    normal: Option<[DxfDouble; 3]>,
    thickness: Option<DxfDouble>,
}

impl DxfPlanarFaceWcsGeometry {
    #[must_use]
    pub const fn kind(self) -> DxfPlanarFaceKind {
        self.kind
    }

    #[must_use]
    pub const fn corners(self) -> [[DxfDouble; 3]; 4] {
        self.corners
    }

    #[must_use]
    pub const fn normal(self) -> Option<[DxfDouble; 3]> {
        self.normal
    }

    #[must_use]
    pub const fn thickness(self) -> Option<DxfDouble> {
        self.thickness
    }
}

/// Source-anchored geometry result for one planar-face record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPlanarFaceWcsGeometryEntry {
    record: DxfPlanarFaceRecordEntry,
    geometry: Result<DxfPlanarFaceWcsGeometry, DxfPlanarFaceWcsGeometryIssue>,
}

impl DxfPlanarFaceWcsGeometryEntry {
    #[must_use]
    pub const fn record(self) -> DxfPlanarFaceRecordEntry {
        self.record
    }

    pub const fn geometry(self) -> Result<DxfPlanarFaceWcsGeometry, DxfPlanarFaceWcsGeometryIssue> {
        self.geometry
    }
}

/// Immutable finite-WCS projection retaining all typed semantic evidence.
#[derive(Debug)]
pub struct DxfPlanarFaceWcsGeometryDirectory {
    source_id: DxfSourceId,
    semantics: DxfPlanarFaceSemanticDirectory,
    entries: Box<[DxfPlanarFaceWcsGeometryEntry]>,
}

impl DxfPlanarFaceWcsGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.planar_face_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }
        let mut entries = Vec::new();
        entries
            .try_reserve(semantics.records().len())
            .map_err(|_| out_of_memory())?;
        for record in semantics.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let typed = semantics
                .semantics_for_record(record)?
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfPlanarFaceWcsGeometryEntry {
                record,
                geometry: derive_geometry(&typed),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfPlanarFaceSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfPlanarFaceWcsGeometryEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfPlanarFaceWcsGeometryEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.record().record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn planar_face_wcs_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceWcsGeometryDirectory, DxfError> {
        DxfPlanarFaceWcsGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn planar_face_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_wcs_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn planar_face_wcs_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPlanarFaceWcsGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).planar_face_wcs_geometry_directory(cancellation)
    }
}

fn derive_geometry(
    semantics: &DxfPlanarFaceSemantics,
) -> Result<DxfPlanarFaceWcsGeometry, DxfPlanarFaceWcsGeometryIssue> {
    let corners = source_corners(semantics)?;
    if semantics.kind() == DxfPlanarFaceKind::Face3d {
        return Ok(DxfPlanarFaceWcsGeometry {
            kind: semantics.kind(),
            corners: corners.map(canonical3),
            normal: None,
            thickness: None,
        });
    }

    let thickness = semantics
        .thickness_value()
        .ok_or(DxfPlanarFaceWcsGeometryIssue::ThicknessUnavailable)?
        .to_f64();
    if !thickness.is_finite() {
        return Err(DxfPlanarFaceWcsGeometryIssue::NonFiniteThickness);
    }
    let extrusion = semantics
        .extrusion_value()
        .ok_or(DxfPlanarFaceWcsGeometryIssue::ExtrusionUnavailable)?
        .map(DxfDouble::to_f64);
    if !finite3(extrusion) {
        return Err(DxfPlanarFaceWcsGeometryIssue::NonFiniteExtrusion);
    }
    let basis = OcsBasis::from_extrusion(extrusion).map_err(map_basis_issue)?;
    let perimeter = [corners[0], corners[1], corners[3], corners[2]];
    let mut wcs = [[DxfDouble::from_f64(0.0); 3]; 4];
    for (target, point) in wcs.iter_mut().zip(perimeter) {
        let transformed = basis.transform(point.map(DxfDouble::to_f64));
        if !finite3(transformed) {
            return Err(DxfPlanarFaceWcsGeometryIssue::NonFiniteDerivedGeometry);
        }
        *target = canonical3(transformed.map(DxfDouble::from_f64));
    }
    Ok(DxfPlanarFaceWcsGeometry {
        kind: semantics.kind(),
        corners: wcs,
        normal: Some(canonical3(basis.normal().map(DxfDouble::from_f64))),
        thickness: Some(canonical(thickness)),
    })
}

fn source_corners(
    semantics: &DxfPlanarFaceSemantics,
) -> Result<[[DxfDouble; 3]; 4], DxfPlanarFaceWcsGeometryIssue> {
    let mut corners = [[DxfDouble::from_f64(0.0); 3]; 4];
    for (index, target) in corners.iter_mut().enumerate() {
        let corner = semantics.corner_value(index).ok_or(
            DxfPlanarFaceWcsGeometryIssue::CornerUnavailable {
                corner_index: index as u8,
            },
        )?;
        if !finite3(corner.map(DxfDouble::to_f64)) {
            return Err(DxfPlanarFaceWcsGeometryIssue::NonFiniteCorner {
                corner_index: index as u8,
            });
        }
        *target = corner;
    }
    Ok(corners)
}

fn map_basis_issue(issue: DxfInsertTransformIssue) -> DxfPlanarFaceWcsGeometryIssue {
    match issue {
        DxfInsertTransformIssue::ZeroLengthExtrusion => {
            DxfPlanarFaceWcsGeometryIssue::ZeroLengthExtrusion
        }
        _ => DxfPlanarFaceWcsGeometryIssue::NonFiniteDerivedGeometry,
    }
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn canonical3(value: [DxfDouble; 3]) -> [DxfDouble; 3] {
    value.map(|component| canonical(component.to_f64()))
}

fn canonical(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

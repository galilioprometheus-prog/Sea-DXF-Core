//! Fail-closed family evidence from classic POLYLINE and VERTEX flags.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfPolylineRecordSemanticDirectory, DxfPolylineRecordValueEntry,
    DxfPolylineVertexIntegerSemanticDirectory, DxfPolylineVertexValueEntry, DxfRawDocumentView,
    DxfSourceId,
};

/// One non-conflicting classic POLYLINE family selected by documented flags.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineFamily {
    TwoDimensional,
    ThreeDimensional,
    PolygonMesh,
    PolyfaceMesh,
}

/// One non-conflicting classic VERTEX family selected by documented flags.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexFamily {
    TwoDimensional,
    ThreeDimensional,
    PolygonMesh,
    PolyfaceCoordinate,
    PolyfaceFace,
}

/// Fail-closed family state retaining conflicting family-bit combinations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineFamilyState<T> {
    Unavailable,
    Classified(T),
    Conflicting { family_bits: i16 },
}

/// Conservative comparison of a classified VERTEX with its parent POLYLINE.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexFamilyComparison {
    NotComparable,
    Matched {
        polyline: DxfPolylineFamily,
        vertex: DxfPolylineVertexFamily,
    },
    Mismatched {
        polyline: DxfPolylineFamily,
        vertex: DxfPolylineVertexFamily,
    },
}

/// Family evidence for one classic POLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineFamilySemantics {
    record: DxfPolylineRecordValueEntry,
    family: DxfPolylineFamilyState<DxfPolylineFamily>,
}

impl DxfPolylineFamilySemantics {
    #[must_use]
    pub const fn record(self) -> DxfPolylineRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn family(self) -> DxfPolylineFamilyState<DxfPolylineFamily> {
        self.family
    }
}

/// Family evidence and parent comparison for one classic VERTEX record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexFamilySemantics {
    vertex: DxfPolylineVertexValueEntry,
    polyline_family: DxfPolylineFamilyState<DxfPolylineFamily>,
    vertex_family: DxfPolylineFamilyState<DxfPolylineVertexFamily>,
    comparison: DxfPolylineVertexFamilyComparison,
}

impl DxfPolylineVertexFamilySemantics {
    #[must_use]
    pub const fn vertex(self) -> DxfPolylineVertexValueEntry {
        self.vertex
    }

    #[must_use]
    pub const fn polyline_family(self) -> DxfPolylineFamilyState<DxfPolylineFamily> {
        self.polyline_family
    }

    #[must_use]
    pub const fn vertex_family(self) -> DxfPolylineFamilyState<DxfPolylineVertexFamily> {
        self.vertex_family
    }

    #[must_use]
    pub const fn comparison(self) -> DxfPolylineVertexFamilyComparison {
        self.comparison
    }
}

/// Lazy classic POLYLINE/VERTEX family evidence retaining both semantic graphs.
#[derive(Debug)]
pub struct DxfPolylineFamilySemanticDirectory {
    source_id: DxfSourceId,
    records: DxfPolylineRecordSemanticDirectory,
    vertices: DxfPolylineVertexIntegerSemanticDirectory,
}

impl DxfPolylineFamilySemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let records = document.polyline_record_semantic_directory(cancellation)?;
        let vertices = document.polyline_vertex_integer_semantic_directory(cancellation)?;
        if records.source_id() != document.source_id()
            || vertices.source_id() != document.source_id()
        {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: records.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            records,
            vertices,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn record_semantic_directory(&self) -> &DxfPolylineRecordSemanticDirectory {
        &self.records
    }

    #[must_use]
    pub const fn vertex_integer_semantic_directory(
        &self,
    ) -> &DxfPolylineVertexIntegerSemanticDirectory {
        &self.vertices
    }

    #[must_use]
    pub fn records(&self) -> &[DxfPolylineRecordValueEntry] {
        self.records.records()
    }

    #[must_use]
    pub fn vertices(&self) -> &[DxfPolylineVertexValueEntry] {
        self.vertices.vertices()
    }

    pub fn polyline_semantics_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfPolylineFamilySemantics>, DxfError> {
        let Some(semantics) = self
            .records
            .semantics_for_polyline_raw_ordinal(raw_record_ordinal)?
        else {
            return Ok(None);
        };
        Ok(Some(DxfPolylineFamilySemantics {
            record: semantics.record(),
            family: polyline_family(semantics.flags_value()),
        }))
    }

    pub fn vertex_semantics_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfPolylineVertexFamilySemantics>, DxfError> {
        let Some(vertex) = self
            .vertices
            .semantics_for_vertex_raw_ordinal(raw_record_ordinal)?
        else {
            return Ok(None);
        };
        let polyline = self
            .records
            .semantics_for_polyline_raw_ordinal(vertex.vertex().polyline_record().ordinal())?
            .ok_or_else(invalid_internal_data)?;
        let polyline_family = polyline_family(polyline.flags_value());
        let vertex_family = vertex_family(vertex.flags_value());
        Ok(Some(DxfPolylineVertexFamilySemantics {
            vertex: vertex.vertex(),
            polyline_family,
            vertex_family,
            comparison: compare_families(polyline_family, vertex_family),
        }))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_family_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineFamilySemanticDirectory, DxfError> {
        DxfPolylineFamilySemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_family_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineFamilySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_family_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_family_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineFamilySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_family_semantic_directory(cancellation)
    }
}

const fn polyline_family(flags: Option<i16>) -> DxfPolylineFamilyState<DxfPolylineFamily> {
    let Some(flags) = flags else {
        return DxfPolylineFamilyState::Unavailable;
    };
    let family_bits = flags & (8 | 16 | 64);
    match family_bits {
        0 => DxfPolylineFamilyState::Classified(DxfPolylineFamily::TwoDimensional),
        8 => DxfPolylineFamilyState::Classified(DxfPolylineFamily::ThreeDimensional),
        16 => DxfPolylineFamilyState::Classified(DxfPolylineFamily::PolygonMesh),
        64 => DxfPolylineFamilyState::Classified(DxfPolylineFamily::PolyfaceMesh),
        _ => DxfPolylineFamilyState::Conflicting { family_bits },
    }
}

const fn vertex_family(flags: Option<i16>) -> DxfPolylineFamilyState<DxfPolylineVertexFamily> {
    let Some(flags) = flags else {
        return DxfPolylineFamilyState::Unavailable;
    };
    let family_bits = flags & (32 | 64 | 128);
    match family_bits {
        0 => DxfPolylineFamilyState::Classified(DxfPolylineVertexFamily::TwoDimensional),
        32 => DxfPolylineFamilyState::Classified(DxfPolylineVertexFamily::ThreeDimensional),
        64 => DxfPolylineFamilyState::Classified(DxfPolylineVertexFamily::PolygonMesh),
        128 => DxfPolylineFamilyState::Classified(DxfPolylineVertexFamily::PolyfaceFace),
        192 => DxfPolylineFamilyState::Classified(DxfPolylineVertexFamily::PolyfaceCoordinate),
        _ => DxfPolylineFamilyState::Conflicting { family_bits },
    }
}

const fn compare_families(
    polyline: DxfPolylineFamilyState<DxfPolylineFamily>,
    vertex: DxfPolylineFamilyState<DxfPolylineVertexFamily>,
) -> DxfPolylineVertexFamilyComparison {
    let (DxfPolylineFamilyState::Classified(polyline), DxfPolylineFamilyState::Classified(vertex)) =
        (polyline, vertex)
    else {
        return DxfPolylineVertexFamilyComparison::NotComparable;
    };
    let matched = matches!(
        (polyline, vertex),
        (
            DxfPolylineFamily::TwoDimensional,
            DxfPolylineVertexFamily::TwoDimensional
        ) | (
            DxfPolylineFamily::ThreeDimensional,
            DxfPolylineVertexFamily::ThreeDimensional
        ) | (
            DxfPolylineFamily::PolygonMesh,
            DxfPolylineVertexFamily::PolygonMesh
        ) | (
            DxfPolylineFamily::PolyfaceMesh,
            DxfPolylineVertexFamily::PolyfaceCoordinate | DxfPolylineVertexFamily::PolyfaceFace
        )
    );
    if matched {
        DxfPolylineVertexFamilyComparison::Matched { polyline, vertex }
    } else {
        DxfPolylineVertexFamilyComparison::Mismatched { polyline, vertex }
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

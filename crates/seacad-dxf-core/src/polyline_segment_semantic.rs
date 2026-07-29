//! Lazy endpoint and local-field bindings for classic POLYLINE segments.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfPolylineFamily, DxfPolylineRecordSemantics, DxfPolylineSegmentDirectory,
    DxfPolylineSegmentEntry, DxfPolylineSegmentRecordState, DxfPolylineVertexSemanticDirectory,
    DxfPolylineVertexSemantics, DxfRawDocumentView, DxfSourceId,
};

/// Coordinate-system boundary implied by an eligible classic POLYLINE family.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineSegmentCoordinateSystem {
    Object,
    World,
}

/// Lazy semantics for one topology-proven classic POLYLINE segment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineSegmentSemantics {
    segment: DxfPolylineSegmentEntry,
    coordinate_system: DxfPolylineSegmentCoordinateSystem,
    parent: DxfPolylineRecordSemantics,
    start_vertex: DxfPolylineVertexSemantics,
    end_vertex: DxfPolylineVertexSemantics,
}

impl DxfPolylineSegmentSemantics {
    #[must_use]
    pub const fn segment(self) -> DxfPolylineSegmentEntry {
        self.segment
    }

    #[must_use]
    pub const fn coordinate_system(self) -> DxfPolylineSegmentCoordinateSystem {
        self.coordinate_system
    }

    #[must_use]
    pub const fn parent(&self) -> &DxfPolylineRecordSemantics {
        &self.parent
    }

    #[must_use]
    pub const fn start_vertex(&self) -> &DxfPolylineVertexSemantics {
        &self.start_vertex
    }

    #[must_use]
    pub const fn end_vertex(&self) -> &DxfPolylineVertexSemantics {
        &self.end_vertex
    }

    #[must_use]
    pub fn start_position(&self) -> Option<[DxfDouble; 3]> {
        self.start_vertex.position_value()
    }

    #[must_use]
    pub fn end_position(&self) -> Option<[DxfDouble; 3]> {
        self.end_vertex.position_value()
    }

    #[must_use]
    pub fn start_vertex_local_widths(&self) -> Option<[DxfDouble; 2]> {
        self.start_vertex.width_values()
    }

    #[must_use]
    pub fn parent_default_widths(&self) -> Option<[DxfDouble; 2]> {
        Some([
            self.parent.default_start_width().value().copied()?,
            self.parent.default_end_width().value().copied()?,
        ])
    }

    #[must_use]
    pub fn bulge(&self) -> Option<DxfDouble> {
        self.start_vertex.bulge_value()
    }

    #[must_use]
    pub fn curve_fit_tangent_direction(&self) -> Option<DxfDouble> {
        self.start_vertex.curve_fit_tangent_direction_value()
    }
}

/// Immutable lazy semantic bindings retaining topology and vertex evidence.
#[derive(Debug)]
pub struct DxfPolylineSegmentSemanticDirectory {
    source_id: DxfSourceId,
    segments: DxfPolylineSegmentDirectory,
    vertices: DxfPolylineVertexSemanticDirectory,
}

impl DxfPolylineSegmentSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let segments = document.polyline_segment_directory(cancellation)?;
        let vertices = document.polyline_vertex_semantic_directory(cancellation)?;
        if segments.source_id() != document.source_id()
            || vertices.source_id() != document.source_id()
        {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: segments.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            segments,
            vertices,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn segment_directory(&self) -> &DxfPolylineSegmentDirectory {
        &self.segments
    }

    #[must_use]
    pub const fn vertex_semantic_directory(&self) -> &DxfPolylineVertexSemanticDirectory {
        &self.vertices
    }

    pub fn semantics_for_segment(
        &self,
        segment_ordinal: u64,
    ) -> Result<Option<DxfPolylineSegmentSemantics>, DxfError> {
        let Some(segment) = self.segments.segment(segment_ordinal) else {
            return Ok(None);
        };
        let parent = self
            .segments
            .family_semantic_directory()
            .record_semantic_directory()
            .semantics_for_polyline_raw_ordinal(
                segment.record().sequence().polyline_record().ordinal(),
            )?
            .ok_or_else(invalid_internal_data)?;
        let start_vertex = self
            .vertices
            .semantics_for_vertex_raw_ordinal(segment.start_vertex().vertex_record().ordinal())?
            .ok_or_else(invalid_internal_data)?;
        let end_vertex = self
            .vertices
            .semantics_for_vertex_raw_ordinal(segment.end_vertex().vertex_record().ordinal())?
            .ok_or_else(invalid_internal_data)?;
        let record = self
            .segments
            .record_for_polyline_raw_ordinal(
                segment.record().sequence().polyline_record().ordinal(),
            )
            .ok_or_else(invalid_internal_data)?;
        let coordinate_system = match record.state() {
            DxfPolylineSegmentRecordState::Available {
                family: DxfPolylineFamily::TwoDimensional,
                ..
            } => DxfPolylineSegmentCoordinateSystem::Object,
            DxfPolylineSegmentRecordState::Available {
                family: DxfPolylineFamily::ThreeDimensional,
                ..
            } => DxfPolylineSegmentCoordinateSystem::World,
            _ => return Err(invalid_internal_data()),
        };
        Ok(Some(DxfPolylineSegmentSemantics {
            segment,
            coordinate_system,
            parent,
            start_vertex,
            end_vertex,
        }))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_segment_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentSemanticDirectory, DxfError> {
        DxfPolylineSegmentSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_segment_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_segment_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_semantic_directory(cancellation)
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

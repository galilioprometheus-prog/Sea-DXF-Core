//! Lazy effective-width precedence for classic 2D POLYLINE segments.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfPolylineRecordSemanticDouble, DxfPolylineSegmentCoordinateSystem, DxfPolylineSegmentEntry,
    DxfPolylineSegmentSemanticDirectory, DxfPolylineVertexSemanticDouble, DxfRawDocumentView,
    DxfSemanticValueState, DxfSourceId,
};

/// Source selected for one effective segment-width component.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineEffectiveWidthOrigin {
    Vertex,
    ParentDefault,
}

/// One usable effective width and its precedence origin.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineEffectiveWidth {
    value: DxfDouble,
    origin: DxfPolylineEffectiveWidthOrigin,
}

impl DxfPolylineEffectiveWidth {
    #[must_use]
    pub const fn value(self) -> DxfDouble {
        self.value
    }

    #[must_use]
    pub const fn origin(self) -> DxfPolylineEffectiveWidthOrigin {
        self.origin
    }
}

/// Why one effective width component cannot be selected.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineSegmentWidthIssue {
    UnsupportedForThreeDimensional,
    VertexValueUnavailable,
    ParentDefaultUnavailable,
}

/// Independently resolved effective start and end widths for one segment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineSegmentEffectiveWidths {
    segment: DxfPolylineSegmentEntry,
    start: Result<DxfPolylineEffectiveWidth, DxfPolylineSegmentWidthIssue>,
    end: Result<DxfPolylineEffectiveWidth, DxfPolylineSegmentWidthIssue>,
}

impl DxfPolylineSegmentEffectiveWidths {
    #[must_use]
    pub const fn segment(self) -> DxfPolylineSegmentEntry {
        self.segment
    }

    pub const fn start(self) -> Result<DxfPolylineEffectiveWidth, DxfPolylineSegmentWidthIssue> {
        self.start
    }

    pub const fn end(self) -> Result<DxfPolylineEffectiveWidth, DxfPolylineSegmentWidthIssue> {
        self.end
    }
}

/// Immutable lazy effective-width view retaining M9.2k semantic evidence.
#[derive(Debug)]
pub struct DxfPolylineSegmentWidthDirectory {
    source_id: DxfSourceId,
    semantics: DxfPolylineSegmentSemanticDirectory,
}

impl DxfPolylineSegmentWidthDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.polyline_segment_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn segment_semantic_directory(&self) -> &DxfPolylineSegmentSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn segments(&self) -> &[DxfPolylineSegmentEntry] {
        self.semantics.segment_directory().segments()
    }

    pub fn widths_for_segment(
        &self,
        segment_ordinal: u64,
    ) -> Result<Option<DxfPolylineSegmentEffectiveWidths>, DxfError> {
        let Some(semantics) = self.semantics.semantics_for_segment(segment_ordinal)? else {
            return Ok(None);
        };
        let (start, end) = match semantics.coordinate_system() {
            DxfPolylineSegmentCoordinateSystem::World => (
                Err(DxfPolylineSegmentWidthIssue::UnsupportedForThreeDimensional),
                Err(DxfPolylineSegmentWidthIssue::UnsupportedForThreeDimensional),
            ),
            DxfPolylineSegmentCoordinateSystem::Object => (
                select_width(
                    semantics.start_vertex().start_width(),
                    semantics.parent().default_start_width(),
                ),
                select_width(
                    semantics.start_vertex().end_width(),
                    semantics.parent().default_end_width(),
                ),
            ),
        };
        Ok(Some(DxfPolylineSegmentEffectiveWidths {
            segment: semantics.segment(),
            start,
            end,
        }))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_segment_width_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentWidthDirectory, DxfError> {
        DxfPolylineSegmentWidthDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_segment_width_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentWidthDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_width_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_segment_width_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSegmentWidthDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_segment_width_directory(cancellation)
    }
}

fn select_width(
    vertex: &DxfPolylineVertexSemanticDouble,
    parent: &DxfPolylineRecordSemanticDouble,
) -> Result<DxfPolylineEffectiveWidth, DxfPolylineSegmentWidthIssue> {
    match vertex.state() {
        DxfSemanticValueState::Explicit => vertex
            .value()
            .copied()
            .map(|value| DxfPolylineEffectiveWidth {
                value,
                origin: DxfPolylineEffectiveWidthOrigin::Vertex,
            })
            .ok_or(DxfPolylineSegmentWidthIssue::VertexValueUnavailable),
        DxfSemanticValueState::Defaulted => parent
            .value()
            .copied()
            .map(|value| DxfPolylineEffectiveWidth {
                value,
                origin: DxfPolylineEffectiveWidthOrigin::ParentDefault,
            })
            .ok_or(DxfPolylineSegmentWidthIssue::ParentDefaultUnavailable),
        DxfSemanticValueState::Absent | DxfSemanticValueState::Invalid => {
            Err(DxfPolylineSegmentWidthIssue::VertexValueUnavailable)
        }
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

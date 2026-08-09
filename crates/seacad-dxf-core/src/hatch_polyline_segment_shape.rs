//! Start-bulge shape classification for HATCH polyline segments.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchPolylineBulgeEntry, DxfHatchPolylineBulgeIssue, DxfHatchPolylineBulgeValue,
    DxfHatchPolylineSegmentDirectory, DxfHatchPolylineSegmentEntry, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineSegmentShape {
    Straight,
    Arc { bulge: DxfDouble },
    Indeterminate(DxfHatchPolylineBulgeIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineSegmentShapeEntry {
    ordinal: u32,
    segment: DxfHatchPolylineSegmentEntry,
    shape: DxfHatchPolylineSegmentShape,
}

impl DxfHatchPolylineSegmentShapeEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn segment(self) -> DxfHatchPolylineSegmentEntry {
        self.segment
    }

    #[must_use]
    pub const fn shape(self) -> DxfHatchPolylineSegmentShape {
        self.shape
    }
}

/// Shape classification retaining the complete M14.4o topology directory.
#[derive(Debug)]
pub struct DxfHatchPolylineSegmentShapeDirectory {
    source_id: DxfSourceId,
    segments: DxfHatchPolylineSegmentDirectory,
    entries: Box<[DxfHatchPolylineSegmentShapeEntry]>,
}

impl DxfHatchPolylineSegmentShapeDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let segments = document.hatch_polyline_segment_directory(cancellation)?;
        ensure_source(document.source_id(), segments.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(segments.segments().len())
            .map_err(|_| out_of_memory())?;
        for segment in segments.segments().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let endpoints = segments
                .endpoints_for_segment(segment.ordinal())
                .ok_or_else(invalid_internal_data)?;
            let start = endpoints.start();
            let bulge = start.bulge();
            entries.push(DxfHatchPolylineSegmentShapeEntry {
                ordinal: compact_len(entries.len())?,
                segment,
                shape: classify(bulge.bulge())?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            segments,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn segment_directory(&self) -> &DxfHatchPolylineSegmentDirectory {
        &self.segments
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineSegmentShapeEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineSegmentShapeEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchPolylineSegmentShapeEntry]> {
        self.segments.segments_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.segment().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.segment().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn start_bulge(&self, ordinal: u64) -> Option<DxfHatchPolylineBulgeEntry> {
        self.segments
            .endpoints_for_segment(ordinal)
            .map(|endpoints| endpoints.start().bulge())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_segment_shape_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentShapeDirectory, DxfError> {
        DxfHatchPolylineSegmentShapeDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_segment_shape_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentShapeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_segment_shape_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_segment_shape_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineSegmentShapeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_segment_shape_directory(cancellation)
    }
}

fn classify(bulge: &DxfHatchPolylineBulgeValue) -> Result<DxfHatchPolylineSegmentShape, DxfError> {
    if let Some(value) = bulge.value().copied() {
        return Ok(if value.to_f64() == 0.0 {
            DxfHatchPolylineSegmentShape::Straight
        } else {
            DxfHatchPolylineSegmentShape::Arc { bulge: value }
        });
    }
    bulge
        .invalid_issue()
        .copied()
        .map(DxfHatchPolylineSegmentShape::Indeterminate)
        .ok_or_else(invalid_internal_data)
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
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
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

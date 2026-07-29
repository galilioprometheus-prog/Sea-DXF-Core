//! Exact classic POLYLINE/VERTEX/SEQEND record-sequence evidence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfRawRecord, DxfRawRecordDirectory, DxfRawRecordSectionKind, DxfSourceId,
};

/// How one exact classic POLYLINE record's following vertex sequence ends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineSequenceState {
    /// Zero or more consecutive VERTEX records are followed by exact SEQEND.
    Closed,
    /// Another record in the same section appears before exact SEQEND.
    Interrupted,
    /// The containing section ends before exact SEQEND.
    Unclosed,
}

/// Half-open range in the directory's retained VERTEX-record array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexRecordRange {
    start: u32,
    end: u32,
}

impl DxfPolylineVertexRecordRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// One exact classic POLYLINE marker and its consecutive sequence evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineSequenceEntry {
    polyline_record: DxfRawRecord,
    vertex_range: DxfPolylineVertexRecordRange,
    boundary_record: Option<DxfRawRecord>,
    state: DxfPolylineSequenceState,
}

impl DxfPolylineSequenceEntry {
    #[must_use]
    pub const fn polyline_record(self) -> DxfRawRecord {
        self.polyline_record
    }

    #[must_use]
    pub const fn vertex_range(self) -> DxfPolylineVertexRecordRange {
        self.vertex_range
    }

    /// Returns exact SEQEND when closed or the first unexpected record when interrupted.
    #[must_use]
    pub const fn boundary_record(self) -> Option<DxfRawRecord> {
        self.boundary_record
    }

    #[must_use]
    pub const fn state(self) -> DxfPolylineSequenceState {
        self.state
    }
}

/// Immutable classic POLYLINE sequence directory over complete entity sections.
#[derive(Debug)]
pub struct DxfPolylineSequenceDirectory {
    source_id: DxfSourceId,
    raw_records: DxfRawRecordDirectory,
    sequences: Box<[DxfPolylineSequenceEntry]>,
    vertices: Box<[DxfRawRecord]>,
}

impl DxfPolylineSequenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        if raw_records.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: raw_records.source_id(),
            });
        }

        let records = raw_records.records();
        let mut sequences = Vec::new();
        let mut vertices = Vec::new();
        let mut index = 0_usize;
        while let Some(record) = records.get(index).copied() {
            ensure_not_cancelled(cancellation)?;
            if !is_entity_section(record.section_kind())
                || record_marker_kind(document, record)? != DxfPolylineRecordMarkerKind::Polyline
            {
                index = index.checked_add(1).ok_or_else(invalid_internal_data)?;
                continue;
            }

            let start = compact_len(vertices.len())?;
            let mut cursor = index.checked_add(1).ok_or_else(invalid_internal_data)?;
            while let Some(candidate) = records.get(cursor).copied() {
                ensure_not_cancelled(cancellation)?;
                if candidate.structure_section_ordinal() != record.structure_section_ordinal()
                    || record_marker_kind(document, candidate)?
                        != DxfPolylineRecordMarkerKind::Vertex
                {
                    break;
                }
                vertices.try_reserve(1).map_err(|_| out_of_memory())?;
                vertices.push(candidate);
                cursor = cursor.checked_add(1).ok_or_else(invalid_internal_data)?;
            }
            let end = compact_len(vertices.len())?;

            let (boundary_record, state) = match records.get(cursor).copied().filter(|candidate| {
                candidate.structure_section_ordinal() == record.structure_section_ordinal()
            }) {
                Some(boundary)
                    if record_marker_kind(document, boundary)?
                        == DxfPolylineRecordMarkerKind::Seqend =>
                {
                    (Some(boundary), DxfPolylineSequenceState::Closed)
                }
                Some(boundary) => (Some(boundary), DxfPolylineSequenceState::Interrupted),
                None => (None, DxfPolylineSequenceState::Unclosed),
            };
            sequences.try_reserve(1).map_err(|_| out_of_memory())?;
            sequences.push(DxfPolylineSequenceEntry {
                polyline_record: record,
                vertex_range: DxfPolylineVertexRecordRange::new(start, end)?,
                boundary_record,
                state,
            });

            index = match state {
                DxfPolylineSequenceState::Closed => {
                    cursor.checked_add(1).ok_or_else(invalid_internal_data)?
                }
                DxfPolylineSequenceState::Interrupted => cursor,
                DxfPolylineSequenceState::Unclosed => cursor,
            };
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            raw_records,
            sequences: sequences.into_boxed_slice(),
            vertices: vertices.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn raw_record_directory(&self) -> &DxfRawRecordDirectory {
        &self.raw_records
    }

    #[must_use]
    pub fn sequences(&self) -> &[DxfPolylineSequenceEntry] {
        &self.sequences
    }

    #[must_use]
    pub fn vertex_records(&self) -> &[DxfRawRecord] {
        &self.vertices
    }

    #[must_use]
    pub fn sequence_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfPolylineSequenceEntry> {
        self.sequences
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.polyline_record().ordinal()
            })
            .ok()
            .and_then(|index| self.sequences.get(index).copied())
    }

    #[must_use]
    pub fn vertices_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfRawRecord]> {
        let entry = self.sequence_for_polyline_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.vertex_range().start()).ok()?;
        let end = usize::try_from(entry.vertex_range().end()).ok()?;
        self.vertices.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_sequence_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSequenceDirectory, DxfError> {
        DxfPolylineSequenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_sequence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSequenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_sequence_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_sequence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineSequenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_sequence_directory(cancellation)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DxfPolylineRecordMarkerKind {
    Polyline,
    Vertex,
    Seqend,
    Other,
}

fn record_marker_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<DxfPolylineRecordMarkerKind, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    let span = marker.value_payload_span();
    if span.len() == b"POLYLINE".len() as u64
        && document.raw_span_equals_exact(span, b"POLYLINE")?
    {
        return Ok(DxfPolylineRecordMarkerKind::Polyline);
    }
    if span.len() == b"VERTEX".len() as u64 && document.raw_span_equals_exact(span, b"VERTEX")? {
        return Ok(DxfPolylineRecordMarkerKind::Vertex);
    }
    if span.len() == b"SEQEND".len() as u64 && document.raw_span_equals_exact(span, b"SEQEND")? {
        return Ok(DxfPolylineRecordMarkerKind::Seqend);
    }
    Ok(DxfPolylineRecordMarkerKind::Other)
}

const fn is_entity_section(kind: DxfRawRecordSectionKind) -> bool {
    matches!(
        kind,
        DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
    )
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
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

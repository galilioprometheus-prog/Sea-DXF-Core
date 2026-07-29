//! Source-anchored numeric evidence from classic POLYLINE VERTEX records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfPolylineSequenceDirectory, DxfRawDocumentView,
    DxfRawGroup, DxfRawRecord, DxfSourceId,
    raw_double::decode_raw_double,
    raw_integer::{decode_raw_i16, decode_raw_i32},
};

/// Documented numeric role retained from one classic VERTEX record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexValueRole {
    LocationX,
    LocationY,
    LocationZ,
    StartWidth,
    EndWidth,
    Bulge,
    CurveFitTangentDirection,
    Flags,
    PolyfaceVertexIndex1,
    PolyfaceVertexIndex2,
    PolyfaceVertexIndex3,
    PolyfaceVertexIndex4,
    Identifier,
}

/// Exact signed or binary64 VERTEX wire value without numeric-domain merging.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexNumber {
    Double(DxfDouble),
    Int16(i16),
    Int32(i32),
}

/// Lexical reason why one classic VERTEX number cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineVertexNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open numeric-value range owned by one classic VERTEX record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexValueRange {
    start: u32,
    end: u32,
}

impl DxfPolylineVertexValueRange {
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

/// One source-order classic VERTEX numeric group and its exact evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexValue {
    group: DxfRawGroup,
    role: DxfPolylineVertexValueRole,
    value: Result<DxfPolylineVertexNumber, DxfPolylineVertexNumericIssue>,
}

impl DxfPolylineVertexValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfPolylineVertexValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfPolylineVertexNumber, DxfPolylineVertexNumericIssue> {
        self.value
    }
}

/// One retained classic VERTEX record and its numeric-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineVertexValueEntry {
    polyline_record: DxfRawRecord,
    vertex_record: DxfRawRecord,
    sequence_vertex_ordinal: u32,
    value_range: DxfPolylineVertexValueRange,
}

impl DxfPolylineVertexValueEntry {
    #[must_use]
    pub const fn polyline_record(self) -> DxfRawRecord {
        self.polyline_record
    }

    #[must_use]
    pub const fn vertex_record(self) -> DxfRawRecord {
        self.vertex_record
    }

    #[must_use]
    pub const fn sequence_vertex_ordinal(self) -> u64 {
        self.sequence_vertex_ordinal as u64
    }

    #[must_use]
    pub const fn value_range(self) -> DxfPolylineVertexValueRange {
        self.value_range
    }
}

/// Immutable numeric evidence directory for exact classic VERTEX records.
#[derive(Debug)]
pub struct DxfPolylineVertexValueDirectory {
    source_id: DxfSourceId,
    sequences: DxfPolylineSequenceDirectory,
    vertices: Box<[DxfPolylineVertexValueEntry]>,
    values: Box<[DxfPolylineVertexValue]>,
}

impl DxfPolylineVertexValueDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let sequences = document.polyline_sequence_directory(cancellation)?;
        if sequences.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: sequences.source_id(),
            });
        }

        let mut vertices = Vec::new();
        let mut values = Vec::new();
        for sequence in sequences.sequences().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let polyline_record = sequence.polyline_record();
            let vertex_records = sequences
                .vertices_for_polyline_raw_ordinal(polyline_record.ordinal())
                .ok_or_else(invalid_internal_data)?;
            for (sequence_vertex_ordinal, vertex_record) in
                vertex_records.iter().copied().enumerate()
            {
                ensure_not_cancelled(cancellation)?;
                let start = compact_len(values.len())?;
                append_record_values(document, vertex_record, cancellation, &mut values)?;
                let end = compact_len(values.len())?;
                vertices.try_reserve(1).map_err(|_| out_of_memory())?;
                vertices.push(DxfPolylineVertexValueEntry {
                    polyline_record,
                    vertex_record,
                    sequence_vertex_ordinal: compact_len(sequence_vertex_ordinal)?,
                    value_range: DxfPolylineVertexValueRange::new(start, end)?,
                });
            }
        }
        if vertices.len() != sequences.vertex_records().len() {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            sequences,
            vertices: vertices.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn sequence_directory(&self) -> &DxfPolylineSequenceDirectory {
        &self.sequences
    }

    #[must_use]
    pub fn vertices(&self) -> &[DxfPolylineVertexValueEntry] {
        &self.vertices
    }

    #[must_use]
    pub fn values(&self) -> &[DxfPolylineVertexValue] {
        &self.values
    }

    #[must_use]
    pub fn vertex_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfPolylineVertexValueEntry> {
        self.vertices
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.vertex_record().ordinal())
            .ok()
            .and_then(|index| self.vertices.get(index).copied())
    }

    #[must_use]
    pub fn vertices_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfPolylineVertexValueEntry]> {
        let sequence = self
            .sequences
            .sequence_for_polyline_raw_ordinal(raw_record_ordinal)?;
        slice(
            &self.vertices,
            sequence.vertex_range().start(),
            sequence.vertex_range().end(),
        )
    }

    #[must_use]
    pub fn values_for_vertex_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfPolylineVertexValue]> {
        let entry = self.vertex_for_raw_ordinal(raw_record_ordinal)?;
        slice(
            &self.values,
            entry.value_range().start(),
            entry.value_range().end(),
        )
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfPolylineVertexValue> {
        let index = self
            .values
            .partition_point(|entry| entry.group().occurrence() < occurrence);
        self.values
            .get(index)
            .copied()
            .filter(|entry| entry.group().occurrence() == occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn polyline_vertex_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexValueDirectory, DxfError> {
        DxfPolylineVertexValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_vertex_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_vertex_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineVertexValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_vertex_value_directory(cancellation)
    }
}

fn append_record_values(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfPolylineVertexValue>,
) -> Result<(), DxfError> {
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
        ensure_not_cancelled(cancellation)?;
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        let Some(role) = value_role(group.group_code().value()) else {
            continue;
        };
        let value = decode_number(document, group, role, cancellation)?;
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(DxfPolylineVertexValue { group, role, value });
    }
    Ok(())
}

fn decode_number(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfPolylineVertexValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfPolylineVertexNumber, DxfPolylineVertexNumericIssue>, DxfError> {
    match wire_kind(role) {
        DxfPolylineVertexWireKind::Double => {
            decode_raw_double(document, group, cancellation).map(|value| {
                value
                    .map(DxfPolylineVertexNumber::Double)
                    .map_err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber)
            })
        }
        DxfPolylineVertexWireKind::Int16 => {
            decode_raw_i16(document, group, cancellation).map(|value| {
                value
                    .map(DxfPolylineVertexNumber::Int16)
                    .map_err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber)
            })
        }
        DxfPolylineVertexWireKind::Int32 => {
            decode_raw_i32(document, group, cancellation).map(|value| {
                value
                    .map(DxfPolylineVertexNumber::Int32)
                    .map_err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber)
            })
        }
    }
}

#[derive(Clone, Copy)]
enum DxfPolylineVertexWireKind {
    Double,
    Int16,
    Int32,
}

const fn wire_kind(role: DxfPolylineVertexValueRole) -> DxfPolylineVertexWireKind {
    match role {
        DxfPolylineVertexValueRole::LocationX
        | DxfPolylineVertexValueRole::LocationY
        | DxfPolylineVertexValueRole::LocationZ
        | DxfPolylineVertexValueRole::StartWidth
        | DxfPolylineVertexValueRole::EndWidth
        | DxfPolylineVertexValueRole::Bulge
        | DxfPolylineVertexValueRole::CurveFitTangentDirection => DxfPolylineVertexWireKind::Double,
        DxfPolylineVertexValueRole::Flags
        | DxfPolylineVertexValueRole::PolyfaceVertexIndex1
        | DxfPolylineVertexValueRole::PolyfaceVertexIndex2
        | DxfPolylineVertexValueRole::PolyfaceVertexIndex3
        | DxfPolylineVertexValueRole::PolyfaceVertexIndex4 => DxfPolylineVertexWireKind::Int16,
        DxfPolylineVertexValueRole::Identifier => DxfPolylineVertexWireKind::Int32,
    }
}

const fn value_role(group_code: i16) -> Option<DxfPolylineVertexValueRole> {
    use DxfPolylineVertexValueRole::{
        Bulge, CurveFitTangentDirection, EndWidth, Flags, Identifier, LocationX, LocationY,
        LocationZ, PolyfaceVertexIndex1, PolyfaceVertexIndex2, PolyfaceVertexIndex3,
        PolyfaceVertexIndex4, StartWidth,
    };
    match group_code {
        10 => Some(LocationX),
        20 => Some(LocationY),
        30 => Some(LocationZ),
        40 => Some(StartWidth),
        41 => Some(EndWidth),
        42 => Some(Bulge),
        50 => Some(CurveFitTangentDirection),
        70 => Some(Flags),
        71 => Some(PolyfaceVertexIndex1),
        72 => Some(PolyfaceVertexIndex2),
        73 => Some(PolyfaceVertexIndex3),
        74 => Some(PolyfaceVertexIndex4),
        91 => Some(Identifier),
        _ => None,
    }
}

fn slice<T>(values: &[T], start: u64, end: u64) -> Option<&[T]> {
    let start = usize::try_from(start).ok()?;
    let end = usize::try_from(end).ok()?;
    values.get(start..end)
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

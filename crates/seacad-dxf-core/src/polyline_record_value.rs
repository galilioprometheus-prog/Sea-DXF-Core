//! Source-anchored numeric evidence from classic POLYLINE records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfPolylineSequenceDirectory, DxfPolylineSequenceEntry,
    DxfRawDocumentView, DxfRawGroup, DxfSourceId, raw_double::decode_raw_double,
    raw_integer::decode_raw_i16,
};

/// Documented numeric role retained from one classic POLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineRecordValueRole {
    DummyX,
    DummyY,
    Elevation,
    Thickness,
    DefaultStartWidth,
    DefaultEndWidth,
    EntitiesFollow,
    Flags,
    MeshMVertexCount,
    MeshNVertexCount,
    SmoothSurfaceMDensity,
    SmoothSurfaceNDensity,
    SmoothSurfaceType,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
}

/// Exact signed or binary64 wire value without merging numeric domains.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineRecordNumber {
    Double(DxfDouble),
    Int16(i16),
}

/// Lexical reason why one classic POLYLINE number cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPolylineRecordNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open numeric-value range owned by one classic POLYLINE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineRecordValueRange {
    start: u32,
    end: u32,
}

impl DxfPolylineRecordValueRange {
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

/// One source-order classic POLYLINE numeric group and its exact evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineRecordValue {
    group: DxfRawGroup,
    role: DxfPolylineRecordValueRole,
    value: Result<DxfPolylineRecordNumber, DxfPolylineRecordNumericIssue>,
}

impl DxfPolylineRecordValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfPolylineRecordValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfPolylineRecordNumber, DxfPolylineRecordNumericIssue> {
        self.value
    }
}

/// One M9.2a sequence entry and its POLYLINE-record numeric-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPolylineRecordValueEntry {
    sequence: DxfPolylineSequenceEntry,
    value_range: DxfPolylineRecordValueRange,
}

impl DxfPolylineRecordValueEntry {
    #[must_use]
    pub const fn sequence(self) -> DxfPolylineSequenceEntry {
        self.sequence
    }

    #[must_use]
    pub const fn value_range(self) -> DxfPolylineRecordValueRange {
        self.value_range
    }
}

/// Immutable numeric evidence directory for exact classic POLYLINE records.
#[derive(Debug)]
pub struct DxfPolylineRecordValueDirectory {
    source_id: DxfSourceId,
    sequences: DxfPolylineSequenceDirectory,
    records: Box<[DxfPolylineRecordValueEntry]>,
    values: Box<[DxfPolylineRecordValue]>,
}

impl DxfPolylineRecordValueDirectory {
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

        let mut records = Vec::new();
        let mut values = Vec::new();
        for sequence in sequences.sequences().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let record = sequence.polyline_record();
            let start = compact_len(values.len())?;
            for occurrence in
                record.marker_occurrence().saturating_add(1)..record.group_range().end()
            {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                let Some(role) = value_role(group.group_code().value()) else {
                    continue;
                };
                let value = decode_number(document, group, role, cancellation)?;
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfPolylineRecordValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfPolylineRecordValueEntry {
                sequence,
                value_range: DxfPolylineRecordValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            sequences,
            records: records.into_boxed_slice(),
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
    pub fn records(&self) -> &[DxfPolylineRecordValueEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfPolylineRecordValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfPolylineRecordValueEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.sequence().polyline_record().ordinal()
            })
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_polyline_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfPolylineRecordValue]> {
        let entry = self.record_for_polyline_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfPolylineRecordValue> {
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
    pub fn polyline_record_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineRecordValueDirectory, DxfError> {
        DxfPolylineRecordValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn polyline_record_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineRecordValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_record_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn polyline_record_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfPolylineRecordValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).polyline_record_value_directory(cancellation)
    }
}

fn decode_number(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfPolylineRecordValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfPolylineRecordNumber, DxfPolylineRecordNumericIssue>, DxfError> {
    match wire_kind(role) {
        DxfPolylineRecordWireKind::Double => {
            decode_raw_double(document, group, cancellation).map(|value| {
                value
                    .map(DxfPolylineRecordNumber::Double)
                    .map_err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber)
            })
        }
        DxfPolylineRecordWireKind::Int16 => {
            decode_raw_i16(document, group, cancellation).map(|value| {
                value
                    .map(DxfPolylineRecordNumber::Int16)
                    .map_err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber)
            })
        }
    }
}

#[derive(Clone, Copy)]
enum DxfPolylineRecordWireKind {
    Double,
    Int16,
}

const fn wire_kind(role: DxfPolylineRecordValueRole) -> DxfPolylineRecordWireKind {
    match role {
        DxfPolylineRecordValueRole::DummyX
        | DxfPolylineRecordValueRole::DummyY
        | DxfPolylineRecordValueRole::Elevation
        | DxfPolylineRecordValueRole::Thickness
        | DxfPolylineRecordValueRole::DefaultStartWidth
        | DxfPolylineRecordValueRole::DefaultEndWidth
        | DxfPolylineRecordValueRole::ExtrusionX
        | DxfPolylineRecordValueRole::ExtrusionY
        | DxfPolylineRecordValueRole::ExtrusionZ => DxfPolylineRecordWireKind::Double,
        DxfPolylineRecordValueRole::EntitiesFollow
        | DxfPolylineRecordValueRole::Flags
        | DxfPolylineRecordValueRole::MeshMVertexCount
        | DxfPolylineRecordValueRole::MeshNVertexCount
        | DxfPolylineRecordValueRole::SmoothSurfaceMDensity
        | DxfPolylineRecordValueRole::SmoothSurfaceNDensity
        | DxfPolylineRecordValueRole::SmoothSurfaceType => DxfPolylineRecordWireKind::Int16,
    }
}

const fn value_role(group_code: i16) -> Option<DxfPolylineRecordValueRole> {
    use DxfPolylineRecordValueRole::{
        DefaultEndWidth, DefaultStartWidth, DummyX, DummyY, Elevation, EntitiesFollow, ExtrusionX,
        ExtrusionY, ExtrusionZ, Flags, MeshMVertexCount, MeshNVertexCount, SmoothSurfaceMDensity,
        SmoothSurfaceNDensity, SmoothSurfaceType, Thickness,
    };
    match group_code {
        10 => Some(DummyX),
        20 => Some(DummyY),
        30 => Some(Elevation),
        39 => Some(Thickness),
        40 => Some(DefaultStartWidth),
        41 => Some(DefaultEndWidth),
        66 => Some(EntitiesFollow),
        70 => Some(Flags),
        71 => Some(MeshMVertexCount),
        72 => Some(MeshNVertexCount),
        73 => Some(SmoothSurfaceMDensity),
        74 => Some(SmoothSurfaceNDensity),
        75 => Some(SmoothSurfaceType),
        210 => Some(ExtrusionX),
        220 => Some(ExtrusionY),
        230 => Some(ExtrusionZ),
        _ => None,
    }
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

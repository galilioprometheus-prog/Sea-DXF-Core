//! Typed header semantics for HATCH polyline boundary paths.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfFillMeshField, DxfHatchBoundaryPathFlagDirectory, DxfHatchBoundaryPathFlagIssue,
    DxfHatchBoundaryPathKind, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    raw_integer::{decode_raw_i16, decode_raw_i32},
};
use std::io;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineBoolean {
    group: DxfRawGroup,
    value: bool,
}
impl DxfHatchPolylineBoolean {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }
    #[must_use]
    pub const fn value(self) -> bool {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexCount {
    group: DxfRawGroup,
    value: u32,
}
impl DxfHatchPolylineVertexCount {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }
    #[must_use]
    pub const fn value(self) -> u32 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineHeader {
    has_bulge: DxfHatchPolylineBoolean,
    is_closed: DxfHatchPolylineBoolean,
    vertex_count: DxfHatchPolylineVertexCount,
}
impl DxfHatchPolylineHeader {
    #[must_use]
    pub const fn has_bulge(self) -> DxfHatchPolylineBoolean {
        self.has_bulge
    }
    #[must_use]
    pub const fn is_closed(self) -> DxfHatchPolylineBoolean {
        self.is_closed
    }
    #[must_use]
    pub const fn vertex_count(self) -> DxfHatchPolylineVertexCount {
        self.vertex_count
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineHeaderIssue {
    PathFlagsUnavailable(DxfHatchBoundaryPathFlagIssue),
    FieldAbsent {
        group_code: i16,
    },
    FieldMultiple {
        group_code: i16,
        occurrence_count: u32,
    },
    InvalidAsciiNumber {
        group: DxfRawGroup,
        issue: DxfAsciiNumericIssue,
    },
    ValueOutOfDomain {
        group: DxfRawGroup,
        value: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineHeaderState {
    NotPolyline,
    Explicit(DxfHatchPolylineHeader),
    Unavailable(DxfHatchPolylineHeaderIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineHeaderEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    raw_record_ordinal: u32,
    state: DxfHatchPolylineHeaderState,
}
impl DxfHatchPolylineHeaderEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }
    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }
    #[must_use]
    pub const fn raw_record_ordinal(self) -> u64 {
        self.raw_record_ordinal as u64
    }
    #[must_use]
    pub const fn state(self) -> DxfHatchPolylineHeaderState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfHatchPolylineHeaderDirectory {
    source_id: DxfSourceId,
    flags: DxfHatchBoundaryPathFlagDirectory,
    entries: Box<[DxfHatchPolylineHeaderEntry]>,
}
impl DxfHatchPolylineHeaderDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let flags = document.hatch_boundary_path_flag_directory(cancellation)?;
        ensure_source(document.source_id(), flags.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(flags.entries().len())
            .map_err(|_| out_of_memory())?;
        for flag_entry in flags.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = match flag_entry.state() {
                Err(issue) => DxfHatchPolylineHeaderState::Unavailable(
                    DxfHatchPolylineHeaderIssue::PathFlagsUnavailable(issue),
                ),
                Ok(value) if value.kind() != DxfHatchBoundaryPathKind::Polyline => {
                    DxfHatchPolylineHeaderState::NotPolyline
                }
                Ok(value) => match header(
                    document,
                    flags
                        .path_directory()
                        .payload_fields_for_path(value.path().ordinal())
                        .ok_or_else(invalid_internal_data)?,
                    cancellation,
                )? {
                    Ok(header) => DxfHatchPolylineHeaderState::Explicit(header),
                    Err(issue) => DxfHatchPolylineHeaderState::Unavailable(issue),
                },
            };
            entries.push(DxfHatchPolylineHeaderEntry {
                ordinal: compact_len(entries.len())?,
                subclass_ordinal: compact_u64(flag_entry.subclass_ordinal())?,
                raw_record_ordinal: compact_u64(flag_entry.raw_record_ordinal())?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            flags,
            entries: entries.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }
    #[must_use]
    pub const fn flag_directory(&self) -> &DxfHatchBoundaryPathFlagDirectory {
        &self.flags
    }
    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineHeaderEntry] {
        &self.entries
    }
    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineHeaderEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }
    #[must_use]
    pub fn entries_for_subclass(&self, ordinal: u64) -> &[DxfHatchPolylineHeaderEntry] {
        let s = self
            .entries
            .partition_point(|e| e.subclass_ordinal() < ordinal);
        let e = self
            .entries
            .partition_point(|e| e.subclass_ordinal() <= ordinal);
        self.entries.get(s..e).unwrap_or_default()
    }
    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchPolylineHeaderEntry] {
        let s = self
            .entries
            .partition_point(|e| e.raw_record_ordinal() < raw);
        let e = self
            .entries
            .partition_point(|e| e.raw_record_ordinal() <= raw);
        self.entries.get(s..e).unwrap_or_default()
    }
}
impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_header_directory(
        self,
        c: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineHeaderDirectory, DxfError> {
        DxfHatchPolylineHeaderDirectory::from_document(self, c)
    }
}
impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_header_directory(
        &self,
        c: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineHeaderDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_header_directory(c)
    }
}
impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_header_directory(
        &self,
        c: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineHeaderDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_header_directory(c)
    }
}

fn header(
    document: DxfRawDocumentView<'_>,
    fields: &[DxfFillMeshField],
    c: &DxfCancellationToken,
) -> Result<Result<DxfHatchPolylineHeader, DxfHatchPolylineHeaderIssue>, DxfError> {
    let b = match one(fields, 72)? {
        Ok(g) => match boolean(document, g, c)? {
            Ok(v) => v,
            Err(i) => return Ok(Err(i)),
        },
        Err(i) => return Ok(Err(i)),
    };
    let closed = match one(fields, 73)? {
        Ok(g) => match boolean(document, g, c)? {
            Ok(v) => v,
            Err(i) => return Ok(Err(i)),
        },
        Err(i) => return Ok(Err(i)),
    };
    let count = match one(fields, 93)? {
        Ok(g) => match count(document, g, c)? {
            Ok(v) => v,
            Err(i) => return Ok(Err(i)),
        },
        Err(i) => return Ok(Err(i)),
    };
    Ok(Ok(DxfHatchPolylineHeader {
        has_bulge: b,
        is_closed: closed,
        vertex_count: count,
    }))
}
fn one(
    fields: &[DxfFillMeshField],
    code: i16,
) -> Result<Result<DxfRawGroup, DxfHatchPolylineHeaderIssue>, DxfError> {
    let mut found = Vec::new();
    for f in fields {
        if f.group().group_code().value() == code {
            found.try_reserve(1).map_err(|_| out_of_memory())?;
            found.push(f.group())
        }
    }
    Ok(match found.as_slice() {
        [] => Err(DxfHatchPolylineHeaderIssue::FieldAbsent { group_code: code }),
        [g] => Ok(*g),
        v => Err(DxfHatchPolylineHeaderIssue::FieldMultiple {
            group_code: code,
            occurrence_count: compact_len(v.len())?,
        }),
    })
}
fn boolean(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    c: &DxfCancellationToken,
) -> Result<Result<DxfHatchPolylineBoolean, DxfHatchPolylineHeaderIssue>, DxfError> {
    Ok(match decode_raw_i16(document, group, c)? {
        Err(issue) => Err(DxfHatchPolylineHeaderIssue::InvalidAsciiNumber { group, issue }),
        Ok(v @ 0..=1) => Ok(DxfHatchPolylineBoolean {
            group,
            value: v == 1,
        }),
        Ok(v) => Err(DxfHatchPolylineHeaderIssue::ValueOutOfDomain {
            group,
            value: i32::from(v),
        }),
    })
}
fn count(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    c: &DxfCancellationToken,
) -> Result<Result<DxfHatchPolylineVertexCount, DxfHatchPolylineHeaderIssue>, DxfError> {
    Ok(match decode_raw_i32(document, group, c)? {
        Err(issue) => Err(DxfHatchPolylineHeaderIssue::InvalidAsciiNumber { group, issue }),
        Ok(v) if v >= 0 => Ok(DxfHatchPolylineVertexCount {
            group,
            value: u32::try_from(v).map_err(|_| invalid_internal_data())?,
        }),
        Ok(v) => Err(DxfHatchPolylineHeaderIssue::ValueOutOfDomain { group, value: v }),
    })
}
fn compact_len(v: usize) -> Result<u32, DxfError> {
    u32::try_from(v).map_err(|_| invalid_internal_data())
}
fn compact_u64(v: u64) -> Result<u32, DxfError> {
    u32::try_from(v).map_err(|_| invalid_internal_data())
}
fn ensure_source(e: DxfSourceId, o: DxfSourceId) -> Result<(), DxfError> {
    if e == o {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch {
            expected: e,
            observed: o,
        })
    }
}
fn ensure_not_cancelled(c: &DxfCancellationToken) -> Result<(), DxfError> {
    if c.is_cancelled() {
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
fn io_error(k: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(k))
}

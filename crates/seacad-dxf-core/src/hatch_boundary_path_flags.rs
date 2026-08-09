//! Typed group-92 flags and conservative HATCH boundary-path classification.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfHatchBoundaryPathDirectory, DxfHatchBoundaryPathEntry, DxfIoOperation,
    DxfRawDocumentView, DxfRawGroup, DxfSourceId, raw_integer::decode_raw_i32,
};

const EXTERNAL: u32 = 1;
const POLYLINE: u32 = 2;
const DERIVED: u32 = 4;
const TEXTBOX: u32 = 8;
const OUTERMOST: u32 = 16;
const KNOWN_MASK: u32 = EXTERNAL | POLYLINE | DERIVED | TEXTBOX | OUTERMOST;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryPathKind {
    Edges,
    Polyline,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPathFlags {
    raw: u32,
}

impl DxfHatchBoundaryPathFlags {
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.raw
    }
    #[must_use]
    pub const fn is_external(self) -> bool {
        self.raw & EXTERNAL != 0
    }
    #[must_use]
    pub const fn is_polyline(self) -> bool {
        self.raw & POLYLINE != 0
    }
    #[must_use]
    pub const fn is_derived(self) -> bool {
        self.raw & DERIVED != 0
    }
    #[must_use]
    pub const fn is_textbox(self) -> bool {
        self.raw & TEXTBOX != 0
    }
    #[must_use]
    pub const fn is_outermost(self) -> bool {
        self.raw & OUTERMOST != 0
    }
    #[must_use]
    pub const fn kind(self) -> DxfHatchBoundaryPathKind {
        if self.is_polyline() {
            DxfHatchBoundaryPathKind::Polyline
        } else {
            DxfHatchBoundaryPathKind::Edges
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPathFlagValue {
    path: DxfHatchBoundaryPathEntry,
    group: DxfRawGroup,
    flags: DxfHatchBoundaryPathFlags,
}

impl DxfHatchBoundaryPathFlagValue {
    #[must_use]
    pub const fn path(self) -> DxfHatchBoundaryPathEntry {
        self.path
    }
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }
    #[must_use]
    pub const fn flags(self) -> DxfHatchBoundaryPathFlags {
        self.flags
    }
    #[must_use]
    pub const fn kind(self) -> DxfHatchBoundaryPathKind {
        self.flags.kind()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryPathFlagIssue {
    InvalidAsciiNumber {
        group: DxfRawGroup,
        issue: DxfAsciiNumericIssue,
    },
    Negative {
        group: DxfRawGroup,
        value: i32,
    },
    UnsupportedBits {
        group: DxfRawGroup,
        value: u32,
        unsupported_bits: u32,
    },
}

pub type DxfHatchBoundaryPathFlagState =
    Result<DxfHatchBoundaryPathFlagValue, DxfHatchBoundaryPathFlagIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryPathFlagEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    raw_record_ordinal: u32,
    state: DxfHatchBoundaryPathFlagState,
}

impl DxfHatchBoundaryPathFlagEntry {
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
    pub const fn state(self) -> DxfHatchBoundaryPathFlagState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfHatchBoundaryPathFlagDirectory {
    source_id: DxfSourceId,
    paths: DxfHatchBoundaryPathDirectory,
    entries: Box<[DxfHatchBoundaryPathFlagEntry]>,
}

impl DxfHatchBoundaryPathFlagDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let paths = document.hatch_boundary_path_directory(cancellation)?;
        ensure_source(document.source_id(), paths.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(paths.paths().len())
            .map_err(|_| out_of_memory())?;
        for path in paths.paths().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let topology = paths
                .entry_for_subclass(path.subclass_ordinal())
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfHatchBoundaryPathFlagEntry {
                ordinal: compact_len(entries.len())?,
                subclass_ordinal: compact_u64(path.subclass_ordinal())?,
                raw_record_ordinal: compact_u64(topology.raw_record_ordinal())?,
                state: decode_flags(document, path, cancellation)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            paths,
            entries: entries.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }
    #[must_use]
    pub const fn path_directory(&self) -> &DxfHatchBoundaryPathDirectory {
        &self.paths
    }
    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryPathFlagEntry] {
        &self.entries
    }
    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryPathFlagEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }
    #[must_use]
    pub fn entry_for_path(&self, ordinal: u64) -> Option<DxfHatchBoundaryPathFlagEntry> {
        let entry = self.entry(ordinal)?;
        self.paths.path(ordinal)?;
        Some(entry)
    }
    #[must_use]
    pub fn entries_for_subclass(&self, ordinal: u64) -> &[DxfHatchBoundaryPathFlagEntry] {
        let start = self
            .entries
            .partition_point(|entry| entry.subclass_ordinal() < ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.subclass_ordinal() <= ordinal);
        self.entries.get(start..end).unwrap_or_default()
    }
    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchBoundaryPathFlagEntry] {
        let start = self
            .entries
            .partition_point(|entry| entry.raw_record_ordinal() < raw);
        let end = self
            .entries
            .partition_point(|entry| entry.raw_record_ordinal() <= raw);
        self.entries.get(start..end).unwrap_or_default()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_path_flag_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPathFlagDirectory, DxfError> {
        DxfHatchBoundaryPathFlagDirectory::from_document(self, cancellation)
    }
}
impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_path_flag_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPathFlagDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_path_flag_directory(cancellation)
    }
}
impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_path_flag_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryPathFlagDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_path_flag_directory(cancellation)
    }
}

fn decode_flags(
    document: DxfRawDocumentView<'_>,
    path: DxfHatchBoundaryPathEntry,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchBoundaryPathFlagState, DxfError> {
    let group = path.marker().group();
    let value = match decode_raw_i32(document, group, cancellation)? {
        Ok(value) if value >= 0 => u32::try_from(value).map_err(|_| invalid_internal_data())?,
        Ok(value) => {
            return Ok(Err(DxfHatchBoundaryPathFlagIssue::Negative {
                group,
                value,
            }));
        }
        Err(issue) => {
            return Ok(Err(DxfHatchBoundaryPathFlagIssue::InvalidAsciiNumber {
                group,
                issue,
            }));
        }
    };
    let unsupported_bits = value & !KNOWN_MASK;
    if unsupported_bits != 0 {
        return Ok(Err(DxfHatchBoundaryPathFlagIssue::UnsupportedBits {
            group,
            value,
            unsupported_bits,
        }));
    }
    Ok(Ok(DxfHatchBoundaryPathFlagValue {
        path,
        group,
        flags: DxfHatchBoundaryPathFlags { raw: value },
    }))
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}
fn compact_u64(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
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

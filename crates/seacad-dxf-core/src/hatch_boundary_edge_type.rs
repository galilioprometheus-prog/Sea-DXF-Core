//! Typed group-72 edge kinds for grouped HATCH boundary edges.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfFillMeshField, DxfHatchBoundaryEdgeDirectory, DxfHatchBoundaryEdgeEntry,
    DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId, raw_integer::decode_raw_i16,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEdgeType {
    Line,
    CircularArc,
    EllipticArc,
    Spline,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEdgeTypeIssue {
    InvalidAsciiNumber {
        group: DxfRawGroup,
        issue: DxfAsciiNumericIssue,
    },
    ValueOutOfDomain {
        group: DxfRawGroup,
        value: i16,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgeTypeEntry {
    ordinal: u32,
    edge: DxfHatchBoundaryEdgeEntry,
    edge_type: Result<DxfHatchBoundaryEdgeType, DxfHatchBoundaryEdgeTypeIssue>,
}

impl DxfHatchBoundaryEdgeTypeEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn edge(self) -> DxfHatchBoundaryEdgeEntry {
        self.edge
    }

    pub const fn edge_type(
        self,
    ) -> Result<DxfHatchBoundaryEdgeType, DxfHatchBoundaryEdgeTypeIssue> {
        self.edge_type
    }
}

/// Typed edge kinds retaining the complete M14.4t count/grouping directory.
#[derive(Debug)]
pub struct DxfHatchBoundaryEdgeTypeDirectory {
    source_id: DxfSourceId,
    edges: DxfHatchBoundaryEdgeDirectory,
    entries: Box<[DxfHatchBoundaryEdgeTypeEntry]>,
}

impl DxfHatchBoundaryEdgeTypeDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let edges = document.hatch_boundary_edge_directory(cancellation)?;
        ensure_source(document.source_id(), edges.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(edges.edges().len())
            .map_err(|_| out_of_memory())?;
        for edge in edges.edges().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfHatchBoundaryEdgeTypeEntry {
                ordinal: compact_len(entries.len())?,
                edge,
                edge_type: decode_edge_type(document, edge, cancellation)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            edges,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn edge_directory(&self) -> &DxfHatchBoundaryEdgeDirectory {
        &self.edges
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryEdgeTypeEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryEdgeTypeEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_path(&self, ordinal: u64) -> Option<&[DxfHatchBoundaryEdgeTypeEntry]> {
        self.edges.edges_for_path(ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.edge().path_ordinal() < ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.edge().path_ordinal() <= ordinal);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn payload_fields_for_entry(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let entry = self.entry(ordinal)?;
        self.edges.payload_fields_for_edge(entry.edge().ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_edge_type_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgeTypeDirectory, DxfError> {
        DxfHatchBoundaryEdgeTypeDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_edge_type_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgeTypeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_edge_type_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_edge_type_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgeTypeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_edge_type_directory(cancellation)
    }
}

fn decode_edge_type(
    document: DxfRawDocumentView<'_>,
    edge: DxfHatchBoundaryEdgeEntry,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfHatchBoundaryEdgeType, DxfHatchBoundaryEdgeTypeIssue>, DxfError> {
    let group = edge.marker().group();
    Ok(match decode_raw_i16(document, group, cancellation)? {
        Err(issue) => Err(DxfHatchBoundaryEdgeTypeIssue::InvalidAsciiNumber { group, issue }),
        Ok(1) => Ok(DxfHatchBoundaryEdgeType::Line),
        Ok(2) => Ok(DxfHatchBoundaryEdgeType::CircularArc),
        Ok(3) => Ok(DxfHatchBoundaryEdgeType::EllipticArc),
        Ok(4) => Ok(DxfHatchBoundaryEdgeType::Spline),
        Ok(value) => Err(DxfHatchBoundaryEdgeTypeIssue::ValueOutOfDomain { group, value }),
    })
}

fn compact_len(value: usize) -> Result<u32, DxfError> {
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

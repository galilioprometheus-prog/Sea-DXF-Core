//! Header-aware bulge defaults for HATCH polyline-boundary vertices.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchPolylineBoolean, DxfHatchPolylineHeaderState, DxfHatchPolylineVertexCardState,
    DxfHatchPolylineVertexNumericDirectory, DxfHatchPolylineVertexNumericEntry,
    DxfHatchPolylineVertexNumericIssue, DxfHatchPolylineVertexNumericValue, DxfIoOperation,
    DxfRawDocumentView, DxfSemanticValue, DxfSemanticValueState, DxfSourceId,
};

const DEFAULT_BULGE: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineBulgeIssue {
    Numeric(DxfHatchPolylineVertexNumericIssue),
    PresentWhenHeaderDisallows {
        state: DxfHatchPolylineVertexCardState,
    },
}

pub type DxfHatchPolylineBulgeValue = DxfSemanticValue<DxfDouble, DxfHatchPolylineBulgeIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineBulgeEntry {
    ordinal: u32,
    numeric: DxfHatchPolylineVertexNumericEntry,
    has_bulge: DxfHatchPolylineBoolean,
    bulge: DxfHatchPolylineBulgeValue,
}

impl DxfHatchPolylineBulgeEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn numeric(self) -> DxfHatchPolylineVertexNumericEntry {
        self.numeric
    }

    #[must_use]
    pub const fn has_bulge(self) -> DxfHatchPolylineBoolean {
        self.has_bulge
    }

    #[must_use]
    pub const fn bulge(&self) -> &DxfHatchPolylineBulgeValue {
        &self.bulge
    }

    #[must_use]
    pub fn bulge_value(&self) -> Option<DxfDouble> {
        self.bulge.value().copied()
    }
}

/// Effective bulge semantics with the complete M14.4l directory retained.
#[derive(Debug)]
pub struct DxfHatchPolylineBulgeDirectory {
    source_id: DxfSourceId,
    numerics: DxfHatchPolylineVertexNumericDirectory,
    entries: Box<[DxfHatchPolylineBulgeEntry]>,
}

impl DxfHatchPolylineBulgeDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let numerics = document.hatch_polyline_vertex_numeric_directory(cancellation)?;
        ensure_source(document.source_id(), numerics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(numerics.entries().len())
            .map_err(|_| out_of_memory())?;
        for numeric in numerics.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let path_ordinal = numeric.vertex().path_ordinal();
            let header_entry = numerics
                .vertex_directory()
                .header_directory()
                .entry(path_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let DxfHatchPolylineHeaderState::Explicit(header) = header_entry.state() else {
                return Err(invalid_internal_data());
            };
            let has_bulge = header.has_bulge();
            entries.push(DxfHatchPolylineBulgeEntry {
                ordinal: compact_len(entries.len())?,
                numeric,
                has_bulge,
                bulge: effective_bulge(numeric, has_bulge)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            numerics,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn numeric_directory(&self) -> &DxfHatchPolylineVertexNumericDirectory {
        &self.numerics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineBulgeEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineBulgeEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_path(&self, path_ordinal: u64) -> Option<&[DxfHatchPolylineBulgeEntry]> {
        self.numerics.entries_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.numeric().vertex().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.numeric().vertex().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_bulge_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineBulgeDirectory, DxfError> {
        DxfHatchPolylineBulgeDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_bulge_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineBulgeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_bulge_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_bulge_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineBulgeDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_bulge_directory(cancellation)
    }
}

fn effective_bulge(
    numeric: DxfHatchPolylineVertexNumericEntry,
    has_bulge: DxfHatchPolylineBoolean,
) -> Result<DxfHatchPolylineBulgeValue, DxfError> {
    let source = numeric.components().bulge();
    let field = source.field_provenance();
    if source.state() == DxfSemanticValueState::Absent {
        return Ok(DxfSemanticValue::defaulted(DEFAULT_BULGE, field));
    }
    if !has_bulge.value() {
        return Ok(DxfSemanticValue::invalid(
            DxfHatchPolylineBulgeIssue::PresentWhenHeaderDisallows {
                state: numeric.vertex().bulge_state(),
            },
            field,
            source.raw_provenance(),
        ));
    }
    copy_numeric(source)
}

fn copy_numeric(
    source: &DxfHatchPolylineVertexNumericValue,
) -> Result<DxfHatchPolylineBulgeValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => {
            DxfSemanticValue::explicit(value, field, raw)
        }
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(DxfHatchPolylineBulgeIssue::Numeric(issue), field, raw)
        }
        DxfSemanticValue::Absent { field } => DxfSemanticValue::defaulted(DEFAULT_BULGE, field),
        DxfSemanticValue::Defaulted { .. } => return Err(invalid_internal_data()),
    })
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

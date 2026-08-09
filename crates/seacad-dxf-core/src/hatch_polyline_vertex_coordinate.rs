//! Required OCS coordinate semantics for HATCH polyline-boundary vertices.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchPolylineBulgeDirectory, DxfHatchPolylineBulgeEntry, DxfHatchPolylineVertexNumericIssue,
    DxfHatchPolylineVertexNumericValue, DxfIoOperation, DxfRawDocumentView, DxfSemanticValue,
    DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineVertexCoordinateIssue {
    MissingRequiredValue,
    Numeric(DxfHatchPolylineVertexNumericIssue),
}

pub type DxfHatchPolylineVertexCoordinateValue =
    DxfSemanticValue<DxfDouble, DxfHatchPolylineVertexCoordinateIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexCoordinates {
    x: DxfHatchPolylineVertexCoordinateValue,
    y: DxfHatchPolylineVertexCoordinateValue,
}

impl DxfHatchPolylineVertexCoordinates {
    #[must_use]
    pub const fn x(&self) -> &DxfHatchPolylineVertexCoordinateValue {
        &self.x
    }

    #[must_use]
    pub const fn y(&self) -> &DxfHatchPolylineVertexCoordinateValue {
        &self.y
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexOcsPosition {
    x: DxfDouble,
    y: DxfDouble,
}

impl DxfHatchPolylineVertexOcsPosition {
    #[must_use]
    pub const fn x(self) -> DxfDouble {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> DxfDouble {
        self.y
    }

    #[must_use]
    pub const fn values(self) -> [DxfDouble; 2] {
        [self.x, self.y]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexUnavailableCoordinates {
    mask: u8,
}

impl DxfHatchPolylineVertexUnavailableCoordinates {
    const X: u8 = 1;
    const Y: u8 = 2;

    #[must_use]
    pub const fn x(self) -> bool {
        self.mask & Self::X != 0
    }

    #[must_use]
    pub const fn y(self) -> bool {
        self.mask & Self::Y != 0
    }

    #[must_use]
    pub const fn count(self) -> u32 {
        self.mask.count_ones()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchPolylineVertexPositionIssue {
    CoordinatesUnavailable(DxfHatchPolylineVertexUnavailableCoordinates),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchPolylineVertexCoordinateEntry {
    ordinal: u32,
    bulge: DxfHatchPolylineBulgeEntry,
    coordinates: DxfHatchPolylineVertexCoordinates,
}

impl DxfHatchPolylineVertexCoordinateEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn bulge(self) -> DxfHatchPolylineBulgeEntry {
        self.bulge
    }

    #[must_use]
    pub const fn coordinates(&self) -> &DxfHatchPolylineVertexCoordinates {
        &self.coordinates
    }

    pub fn ocs_position(
        self,
    ) -> Result<DxfHatchPolylineVertexOcsPosition, DxfHatchPolylineVertexPositionIssue> {
        let x = self.coordinates.x().value().copied();
        let y = self.coordinates.y().value().copied();
        let mut mask = 0_u8;
        if x.is_none() {
            mask |= DxfHatchPolylineVertexUnavailableCoordinates::X;
        }
        if y.is_none() {
            mask |= DxfHatchPolylineVertexUnavailableCoordinates::Y;
        }
        match (x, y) {
            (Some(x), Some(y)) => Ok(DxfHatchPolylineVertexOcsPosition { x, y }),
            _ => Err(DxfHatchPolylineVertexPositionIssue::CoordinatesUnavailable(
                DxfHatchPolylineVertexUnavailableCoordinates { mask },
            )),
        }
    }
}

/// Required coordinate semantics retaining all M14.4m bulge evidence.
#[derive(Debug)]
pub struct DxfHatchPolylineVertexCoordinateDirectory {
    source_id: DxfSourceId,
    bulges: DxfHatchPolylineBulgeDirectory,
    entries: Box<[DxfHatchPolylineVertexCoordinateEntry]>,
}

impl DxfHatchPolylineVertexCoordinateDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let bulges = document.hatch_polyline_bulge_directory(cancellation)?;
        ensure_source(document.source_id(), bulges.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(bulges.entries().len())
            .map_err(|_| out_of_memory())?;
        for bulge in bulges.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let numeric = bulge.numeric();
            let components = numeric.components();
            entries.push(DxfHatchPolylineVertexCoordinateEntry {
                ordinal: compact_len(entries.len())?,
                bulge,
                coordinates: DxfHatchPolylineVertexCoordinates {
                    x: required(components.x())?,
                    y: required(components.y())?,
                },
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            bulges,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn bulge_directory(&self) -> &DxfHatchPolylineBulgeDirectory {
        &self.bulges
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchPolylineVertexCoordinateEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchPolylineVertexCoordinateEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchPolylineVertexCoordinateEntry]> {
        self.bulges.entries_for_path(path_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry.bulge().numeric().vertex().path_ordinal() < path_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry.bulge().numeric().vertex().path_ordinal() <= path_ordinal
        });
        self.entries.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_polyline_vertex_coordinate_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexCoordinateDirectory, DxfError> {
        DxfHatchPolylineVertexCoordinateDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_polyline_vertex_coordinate_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexCoordinateDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_vertex_coordinate_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_polyline_vertex_coordinate_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchPolylineVertexCoordinateDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_polyline_vertex_coordinate_directory(cancellation)
    }
}

fn required(
    source: &DxfHatchPolylineVertexNumericValue,
) -> Result<DxfHatchPolylineVertexCoordinateValue, DxfError> {
    Ok(match *source {
        DxfSemanticValue::Explicit { value, field, raw } => {
            DxfSemanticValue::explicit(value, field, raw)
        }
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfHatchPolylineVertexCoordinateIssue::Numeric(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::invalid(
            DxfHatchPolylineVertexCoordinateIssue::MissingRequiredValue,
            field,
            None,
        ),
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

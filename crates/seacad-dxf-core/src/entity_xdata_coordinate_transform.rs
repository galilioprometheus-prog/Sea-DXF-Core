//! AutoCAD-compatible affine transformation roles for generic entity XDATA tuples.

use std::io;

use crate::entity_xdata_coordinate_transform_math::{
    DxfEntityXDataCoordinateTransform, apply_affine, apply_linear, canonical_double,
};
use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble,
    DxfEntityXDataPointComponents, DxfEntityXDataPointKind, DxfEntityXDataPointMember,
    DxfEntityXDataPointTuple, DxfEntityXDataPointTupleDirectory, DxfEntityXDataValue,
    DxfEntityXDataValueIssue, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataPointComponent {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataTransformedPointIssue {
    PartialTuple(DxfEntityXDataPointComponents),
    InvalidComponent {
        component: DxfEntityXDataPointComponent,
        issue: DxfEntityXDataValueIssue,
    },
    NonFiniteDerivedPoint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataTransformedPointState {
    Available {
        original: [DxfDouble; 3],
        transformed: [DxfDouble; 3],
    },
    Unavailable(DxfEntityXDataTransformedPointIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataTransformedPointEntry {
    tuple: DxfEntityXDataPointTuple,
    state: DxfEntityXDataTransformedPointState,
}

impl DxfEntityXDataTransformedPointEntry {
    #[must_use]
    pub const fn tuple(self) -> DxfEntityXDataPointTuple {
        self.tuple
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataTransformedPointState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataCoordinateTransformDirectory {
    source_id: DxfSourceId,
    tuples: DxfEntityXDataPointTupleDirectory,
    transform: DxfEntityXDataCoordinateTransform,
    entries: Box<[DxfEntityXDataTransformedPointEntry]>,
}

impl DxfEntityXDataCoordinateTransformDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let tuples = document.entity_xdata_point_tuple_directory(cancellation)?;
        ensure_source(document.source_id(), tuples.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(tuples.tuples().len())
            .map_err(|_| out_of_memory())?;
        for tuple in tuples.tuples().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfEntityXDataTransformedPointEntry {
                tuple,
                state: transform_tuple(&tuples, tuple, transform)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            tuples,
            transform,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn point_tuple_directory(&self) -> &DxfEntityXDataPointTupleDirectory {
        &self.tuples
    }

    #[must_use]
    pub const fn transform(&self) -> DxfEntityXDataCoordinateTransform {
        self.transform
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataTransformedPointEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataTransformedPointEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_tuple(
        &self,
        tuple: DxfEntityXDataPointTuple,
    ) -> Option<DxfEntityXDataTransformedPointEntry> {
        if self.tuples.tuple(tuple.ordinal()) != Some(tuple) {
            return None;
        }
        self.entry(tuple.ordinal())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_coordinate_transform_directory(
        self,
        transform: DxfEntityXDataCoordinateTransform,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataCoordinateTransformDirectory, DxfError> {
        DxfEntityXDataCoordinateTransformDirectory::from_document(self, transform, cancellation)
    }
}

macro_rules! document_transform_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_coordinate_transform_directory(
                &self,
                transform: DxfEntityXDataCoordinateTransform,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataCoordinateTransformDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_coordinate_transform_directory(transform, cancellation)
            }
        }
    };
}

document_transform_directory!(DxfAsciiRawDocument<'_>);
document_transform_directory!(DxfBinaryRawDocument<'_>);

fn transform_tuple(
    directory: &DxfEntityXDataPointTupleDirectory,
    tuple: DxfEntityXDataPointTuple,
    transform: DxfEntityXDataCoordinateTransform,
) -> Result<DxfEntityXDataTransformedPointState, DxfError> {
    if !tuple.components().is_complete() {
        return Ok(DxfEntityXDataTransformedPointState::Unavailable(
            DxfEntityXDataTransformedPointIssue::PartialTuple(tuple.components()),
        ));
    }
    let original = [
        component_value(directory, tuple, tuple.x(), DxfEntityXDataPointComponent::X)?,
        component_value(directory, tuple, tuple.y(), DxfEntityXDataPointComponent::Y)?,
        component_value(directory, tuple, tuple.z(), DxfEntityXDataPointComponent::Z)?,
    ];
    let original = match original {
        [Ok(x), Ok(y), Ok(z)] => [x, y, z],
        values => {
            let issue = values
                .into_iter()
                .find_map(Result::err)
                .ok_or_else(invalid_internal_data)?;
            return Ok(DxfEntityXDataTransformedPointState::Unavailable(issue));
        }
    };
    let value = original.map(DxfDouble::to_f64);
    let transformed = match tuple.kind() {
        DxfEntityXDataPointKind::Point => value,
        DxfEntityXDataPointKind::WorldPosition => apply_affine(transform.position_f64(), value),
        DxfEntityXDataPointKind::WorldDisplacement => {
            apply_linear(transform.displacement_f64(), value)
        }
        DxfEntityXDataPointKind::WorldDirection => apply_linear(transform.direction_f64(), value),
    };
    if !finite3(transformed) {
        return Ok(DxfEntityXDataTransformedPointState::Unavailable(
            DxfEntityXDataTransformedPointIssue::NonFiniteDerivedPoint,
        ));
    }
    Ok(DxfEntityXDataTransformedPointState::Available {
        original,
        transformed: transformed.map(canonical_double),
    })
}

fn component_value(
    directory: &DxfEntityXDataPointTupleDirectory,
    tuple: DxfEntityXDataPointTuple,
    member: Option<DxfEntityXDataPointMember>,
    component: DxfEntityXDataPointComponent,
) -> Result<Result<DxfDouble, DxfEntityXDataTransformedPointIssue>, DxfError> {
    let member = member.ok_or_else(invalid_internal_data)?;
    let entry = directory
        .entry_for_member(tuple, member)
        .ok_or_else(invalid_internal_data)?;
    match entry.value() {
        DxfEntityXDataValue::Double { value, .. } => Ok(Ok(value)),
        DxfEntityXDataValue::Invalid { issue, .. } => {
            Ok(Err(DxfEntityXDataTransformedPointIssue::InvalidComponent {
                component,
                issue,
            }))
        }
        _ => Err(invalid_internal_data()),
    }
}

fn finite3(value: [f64; 3]) -> bool {
    value.iter().all(|value| value.is_finite())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
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

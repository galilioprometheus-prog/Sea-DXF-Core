//! Exact HATCH elevation tuples over the reviewed boundary-header partition.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfFillMeshField, DxfHatchBoundaryPartitionDirectory,
    DxfHatchBoundaryPartitionIssue, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    raw_double::decode_raw_double,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchElevationComponentRole {
    X,
    Y,
    Z,
}

impl DxfHatchElevationComponentRole {
    const fn group_code(self) -> i16 {
        match self {
            Self::X => 10,
            Self::Y => 20,
            Self::Z => 30,
        }
    }

    const fn is_planar(self) -> bool {
        matches!(self, Self::X | Self::Y)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchElevationComponentValue {
    group: DxfRawGroup,
    value: DxfDouble,
}

impl DxfHatchElevationComponentValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn value(self) -> DxfDouble {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchElevationComponentIssue {
    Absent,
    Multiple {
        occurrence_count: u32,
    },
    InvalidAsciiNumber {
        group: DxfRawGroup,
        issue: DxfAsciiNumericIssue,
    },
    NonFiniteDouble {
        group: DxfRawGroup,
        value: DxfDouble,
    },
    PlanarComponentNonZero {
        group: DxfRawGroup,
        value: DxfDouble,
    },
}

pub type DxfHatchElevationComponent =
    Result<DxfHatchElevationComponentValue, DxfHatchElevationComponentIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchElevationComponents {
    x: DxfHatchElevationComponent,
    y: DxfHatchElevationComponent,
    z: DxfHatchElevationComponent,
}

impl DxfHatchElevationComponents {
    pub const fn x(self) -> DxfHatchElevationComponent {
        self.x
    }

    pub const fn y(self) -> DxfHatchElevationComponent {
        self.y
    }

    pub const fn z(self) -> DxfHatchElevationComponent {
        self.z
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchElevation {
    x: DxfHatchElevationComponentValue,
    y: DxfHatchElevationComponentValue,
    z: DxfHatchElevationComponentValue,
}

impl DxfHatchElevation {
    #[must_use]
    pub const fn x(self) -> DxfHatchElevationComponentValue {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> DxfHatchElevationComponentValue {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> DxfHatchElevationComponentValue {
        self.z
    }

    #[must_use]
    pub const fn values(self) -> [DxfDouble; 3] {
        [self.x.value(), self.y.value(), self.z.value()]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchElevationUnavailableComponents {
    mask: u8,
}

impl DxfHatchElevationUnavailableComponents {
    const X: u8 = 1;
    const Y: u8 = 2;
    const Z: u8 = 4;

    #[must_use]
    pub const fn x(self) -> bool {
        self.mask & Self::X != 0
    }

    #[must_use]
    pub const fn y(self) -> bool {
        self.mask & Self::Y != 0
    }

    #[must_use]
    pub const fn z(self) -> bool {
        self.mask & Self::Z != 0
    }

    #[must_use]
    pub const fn count(self) -> u32 {
        self.mask.count_ones()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchElevationIssue {
    PartitionUnavailable(DxfHatchBoundaryPartitionIssue),
    ComponentsUnavailable(DxfHatchElevationUnavailableComponents),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchElevationEntryState {
    PartitionUnavailable(DxfHatchBoundaryPartitionIssue),
    Components(DxfHatchElevationComponents),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchElevationEntry {
    ordinal: u32,
    subclass_ordinal: u32,
    raw_record_ordinal: u32,
    state: DxfHatchElevationEntryState,
}

impl DxfHatchElevationEntry {
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
    pub const fn state(self) -> DxfHatchElevationEntryState {
        self.state
    }

    pub fn elevation(self) -> Result<DxfHatchElevation, DxfHatchElevationIssue> {
        let components = match self.state {
            DxfHatchElevationEntryState::PartitionUnavailable(issue) => {
                return Err(DxfHatchElevationIssue::PartitionUnavailable(issue));
            }
            DxfHatchElevationEntryState::Components(components) => components,
        };
        let x = components.x();
        let y = components.y();
        let z = components.z();
        let mut mask = 0_u8;
        if x.is_err() {
            mask |= DxfHatchElevationUnavailableComponents::X;
        }
        if y.is_err() {
            mask |= DxfHatchElevationUnavailableComponents::Y;
        }
        if z.is_err() {
            mask |= DxfHatchElevationUnavailableComponents::Z;
        }
        match (x, y, z) {
            (Ok(x), Ok(y), Ok(z)) => Ok(DxfHatchElevation { x, y, z }),
            _ => Err(DxfHatchElevationIssue::ComponentsUnavailable(
                DxfHatchElevationUnavailableComponents { mask },
            )),
        }
    }
}

/// One exact elevation result per exact `AcDbHatch` subclass.
#[derive(Debug)]
pub struct DxfHatchElevationDirectory {
    source_id: DxfSourceId,
    partitions: DxfHatchBoundaryPartitionDirectory,
    entries: Box<[DxfHatchElevationEntry]>,
}

impl DxfHatchElevationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let partitions = document.hatch_boundary_partition_directory(cancellation)?;
        ensure_source(document.source_id(), partitions.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(partitions.entries().len())
            .map_err(|_| out_of_memory())?;
        for partition_entry in partitions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = match partition_entry.partition() {
                Ok(_) => {
                    let fields = partitions
                        .header_fields_for_subclass(partition_entry.subclass_ordinal())
                        .ok_or_else(invalid_internal_data)?;
                    DxfHatchElevationEntryState::Components(DxfHatchElevationComponents {
                        x: component(
                            document,
                            fields,
                            DxfHatchElevationComponentRole::X,
                            cancellation,
                        )?,
                        y: component(
                            document,
                            fields,
                            DxfHatchElevationComponentRole::Y,
                            cancellation,
                        )?,
                        z: component(
                            document,
                            fields,
                            DxfHatchElevationComponentRole::Z,
                            cancellation,
                        )?,
                    })
                }
                Err(issue) => DxfHatchElevationEntryState::PartitionUnavailable(issue),
            };
            entries.push(DxfHatchElevationEntry {
                ordinal: compact_len(entries.len())?,
                subclass_ordinal: compact_u64(partition_entry.subclass_ordinal())?,
                raw_record_ordinal: compact_u64(
                    partition_entry.subclass().entity().record().ordinal(),
                )?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            partitions,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn partition_directory(&self) -> &DxfHatchBoundaryPartitionDirectory {
        &self.partitions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchElevationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchElevationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_subclass(&self, ordinal: u64) -> Option<DxfHatchElevationEntry> {
        self.entries
            .binary_search_by_key(&ordinal, |entry| entry.subclass_ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchElevationEntry] {
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
    pub fn hatch_elevation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchElevationDirectory, DxfError> {
        DxfHatchElevationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_elevation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchElevationDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_elevation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_elevation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchElevationDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_elevation_directory(cancellation)
    }
}

fn component(
    document: DxfRawDocumentView<'_>,
    fields: &[DxfFillMeshField],
    role: DxfHatchElevationComponentRole,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHatchElevationComponent, DxfError> {
    let mut matches = Vec::new();
    for field in fields.iter().copied() {
        ensure_not_cancelled(cancellation)?;
        if field.group().group_code().value() == role.group_code() {
            matches.try_reserve(1).map_err(|_| out_of_memory())?;
            matches.push(field.group());
        }
    }
    let group = match matches.as_slice() {
        [] => return Ok(Err(DxfHatchElevationComponentIssue::Absent)),
        [group] => *group,
        values => {
            return Ok(Err(DxfHatchElevationComponentIssue::Multiple {
                occurrence_count: compact_len(values.len())?,
            }));
        }
    };
    let value = match decode_raw_double(document, group, cancellation)? {
        Ok(value) => value,
        Err(issue) => {
            return Ok(Err(DxfHatchElevationComponentIssue::InvalidAsciiNumber {
                group,
                issue,
            }));
        }
    };
    if !value.is_finite() {
        return Ok(Err(DxfHatchElevationComponentIssue::NonFiniteDouble {
            group,
            value,
        }));
    }
    if role.is_planar() && value.to_f64() != 0.0 {
        return Ok(Err(
            DxfHatchElevationComponentIssue::PlanarComponentNonZero { group, value },
        ));
    }
    Ok(Ok(DxfHatchElevationComponentValue { group, value }))
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

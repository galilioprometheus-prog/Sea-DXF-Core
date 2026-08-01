//! Derived axis and parameter relations for HELIX records.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHelixRecordEntry, DxfHelixScalarDirectory, DxfHelixScalarValue, DxfHelixValueRole,
    DxfHelixVectorDirectory, DxfHelixVectorKind, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

pub const DXF_HELIX_MAX_COMMAND_TURNS: f64 = 500.0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixAxisDerivationStage {
    RadialVector,
    AxisLength,
    BaseRadius,
    OrthogonalityResidual,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixAxisRelation {
    Unavailable,
    DerivedNonFinite {
        stage: DxfHelixAxisDerivationStage,
    },
    ZeroAxis {
        radial_vector: [DxfDouble; 3],
    },
    Compared {
        radial_vector: [DxfDouble; 3],
        normalized_axis: [DxfDouble; 3],
        axis_length: DxfDouble,
        derived_base_radius: DxfDouble,
        orthogonality_residual: DxfDouble,
        exactly_perpendicular: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixRadiusDomain {
    Unavailable,
    Negative { radius: DxfDouble },
    NonNegative { radius: DxfDouble },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixTurnsDomain {
    Unavailable,
    NonPositive { turns: DxfDouble },
    WithinCommandLimit { turns: DxfDouble },
    AboveCommandLimit { turns: DxfDouble },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixHeightRelation {
    Unavailable,
    DerivedNonFinite {
        turns: DxfDouble,
        turn_height: DxfDouble,
    },
    Compared {
        turns: DxfDouble,
        turn_height: DxfDouble,
        axial_height: DxfDouble,
        flat: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixRelationEntry {
    ordinal: u32,
    record: DxfHelixRecordEntry,
    axis: DxfHelixAxisRelation,
    radius: DxfHelixRadiusDomain,
    turns: DxfHelixTurnsDomain,
    height: DxfHelixHeightRelation,
}

impl DxfHelixRelationEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfHelixRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn axis(self) -> DxfHelixAxisRelation {
        self.axis
    }

    #[must_use]
    pub const fn radius(self) -> DxfHelixRadiusDomain {
        self.radius
    }

    #[must_use]
    pub const fn turns(self) -> DxfHelixTurnsDomain {
        self.turns
    }

    #[must_use]
    pub const fn height(self) -> DxfHelixHeightRelation {
        self.height
    }
}

#[derive(Debug)]
pub struct DxfHelixRelationDirectory {
    source_id: DxfSourceId,
    scalars: DxfHelixScalarDirectory,
    vectors: DxfHelixVectorDirectory,
    entries: Box<[DxfHelixRelationEntry]>,
}

impl DxfHelixRelationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let scalars = document.helix_scalar_directory(cancellation)?;
        let vectors = document.helix_vector_directory(cancellation)?;
        ensure_source(document.source_id(), scalars.source_id())?;
        ensure_source(document.source_id(), vectors.source_id())?;
        let records = scalars.card_directory().evidence_directory().records();
        let mut entries = Vec::new();
        entries
            .try_reserve(records.len())
            .map_err(|_| out_of_memory())?;
        for record in records.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw = record.entity().record().ordinal();
            entries.push(DxfHelixRelationEntry {
                ordinal: compact_len(entries.len())?,
                record,
                axis: axis_relation(&vectors, raw)?,
                radius: radius_domain(&scalars, raw),
                turns: turns_domain(&scalars, raw),
                height: height_relation(&scalars, raw),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            scalars,
            vectors,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn scalar_directory(&self) -> &DxfHelixScalarDirectory {
        &self.scalars
    }

    #[must_use]
    pub const fn vector_directory(&self) -> &DxfHelixVectorDirectory {
        &self.vectors
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHelixRelationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHelixRelationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_raw_record(&self, raw: u64) -> Option<DxfHelixRelationEntry> {
        self.entries
            .binary_search_by_key(&raw, |entry| entry.record().entity().record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn helix_relation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixRelationDirectory, DxfError> {
        DxfHelixRelationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn helix_relation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixRelationDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_relation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn helix_relation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixRelationDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_relation_directory(cancellation)
    }
}

fn axis_relation(
    vectors: &DxfHelixVectorDirectory,
    raw: u64,
) -> Result<DxfHelixAxisRelation, DxfError> {
    let Some(axis_base) = vector_value(vectors, raw, DxfHelixVectorKind::AxisBase)? else {
        return Ok(DxfHelixAxisRelation::Unavailable);
    };
    let Some(start) = vector_value(vectors, raw, DxfHelixVectorKind::StartPoint)? else {
        return Ok(DxfHelixAxisRelation::Unavailable);
    };
    let Some(axis) = vector_value(vectors, raw, DxfHelixVectorKind::AxisVector)? else {
        return Ok(DxfHelixAxisRelation::Unavailable);
    };
    let radial = [
        start[0].to_f64() - axis_base[0].to_f64(),
        start[1].to_f64() - axis_base[1].to_f64(),
        start[2].to_f64() - axis_base[2].to_f64(),
    ];
    let Some(radial_vector) = finite_vector(radial) else {
        return Ok(derived_axis_issue(
            DxfHelixAxisDerivationStage::RadialVector,
        ));
    };
    let axis = axis.map(DxfDouble::to_f64);
    let axis_length = axis[0].hypot(axis[1]).hypot(axis[2]);
    if !axis_length.is_finite() {
        return Ok(derived_axis_issue(DxfHelixAxisDerivationStage::AxisLength));
    }
    if axis_length == 0.0 {
        return Ok(DxfHelixAxisRelation::ZeroAxis { radial_vector });
    }
    let normalized = axis.map(|component| component / axis_length);
    let derived_base_radius = radial[0].hypot(radial[1]).hypot(radial[2]);
    if !derived_base_radius.is_finite() {
        return Ok(derived_axis_issue(DxfHelixAxisDerivationStage::BaseRadius));
    }
    let residual =
        radial[0] * normalized[0] + radial[1] * normalized[1] + radial[2] * normalized[2];
    if !residual.is_finite() {
        return Ok(derived_axis_issue(
            DxfHelixAxisDerivationStage::OrthogonalityResidual,
        ));
    }
    Ok(DxfHelixAxisRelation::Compared {
        radial_vector,
        normalized_axis: normalized.map(DxfDouble::from_f64),
        axis_length: DxfDouble::from_f64(axis_length),
        derived_base_radius: DxfDouble::from_f64(derived_base_radius),
        orthogonality_residual: DxfDouble::from_f64(residual),
        exactly_perpendicular: residual == 0.0,
    })
}

fn radius_domain(scalars: &DxfHelixScalarDirectory, raw: u64) -> DxfHelixRadiusDomain {
    let Some(radius) = scalar_double(scalars, raw, DxfHelixValueRole::Radius) else {
        return DxfHelixRadiusDomain::Unavailable;
    };
    if radius.to_f64() < 0.0 {
        DxfHelixRadiusDomain::Negative { radius }
    } else {
        DxfHelixRadiusDomain::NonNegative { radius }
    }
}

fn turns_domain(scalars: &DxfHelixScalarDirectory, raw: u64) -> DxfHelixTurnsDomain {
    let Some(turns) = scalar_double(scalars, raw, DxfHelixValueRole::Turns) else {
        return DxfHelixTurnsDomain::Unavailable;
    };
    if turns.to_f64() <= 0.0 {
        DxfHelixTurnsDomain::NonPositive { turns }
    } else if turns.to_f64() <= DXF_HELIX_MAX_COMMAND_TURNS {
        DxfHelixTurnsDomain::WithinCommandLimit { turns }
    } else {
        DxfHelixTurnsDomain::AboveCommandLimit { turns }
    }
}

fn height_relation(scalars: &DxfHelixScalarDirectory, raw: u64) -> DxfHelixHeightRelation {
    let Some(turns) = scalar_double(scalars, raw, DxfHelixValueRole::Turns) else {
        return DxfHelixHeightRelation::Unavailable;
    };
    let Some(turn_height) = scalar_double(scalars, raw, DxfHelixValueRole::TurnHeight) else {
        return DxfHelixHeightRelation::Unavailable;
    };
    let height = turns.to_f64() * turn_height.to_f64();
    if height.is_finite() {
        DxfHelixHeightRelation::Compared {
            turns,
            turn_height,
            axial_height: DxfDouble::from_f64(height),
            flat: height == 0.0,
        }
    } else {
        DxfHelixHeightRelation::DerivedNonFinite { turns, turn_height }
    }
}

fn vector_value(
    vectors: &DxfHelixVectorDirectory,
    raw: u64,
    kind: DxfHelixVectorKind,
) -> Result<Option<[DxfDouble; 3]>, DxfError> {
    Ok(vectors
        .vector_for_kind(raw, kind)
        .ok_or_else(invalid_internal_data)?
        .vector_value())
}

fn scalar_double(
    scalars: &DxfHelixScalarDirectory,
    raw: u64,
    role: DxfHelixValueRole,
) -> Option<DxfDouble> {
    let entry = scalars.entry_for_role(raw, role)?;
    let value = entry.semantic().value()?;
    match *value {
        DxfHelixScalarValue::Radius(value)
        | DxfHelixScalarValue::Turns(value)
        | DxfHelixScalarValue::TurnHeight(value) => Some(value),
        _ => None,
    }
}

fn finite_vector(value: [f64; 3]) -> Option<[DxfDouble; 3]> {
    value
        .iter()
        .all(|component| component.is_finite())
        .then(|| value.map(DxfDouble::from_f64))
}

const fn derived_axis_issue(stage: DxfHelixAxisDerivationStage) -> DxfHelixAxisRelation {
    DxfHelixAxisRelation::DerivedNonFinite { stage }
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

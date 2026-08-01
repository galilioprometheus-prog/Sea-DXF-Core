//! Typed relations among SPLINE flags, weights, and planar normals.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId, DxfSplineAuxiliarySemanticDirectory, DxfSplineFlagsSemantic,
    DxfSplineRecordEntry, DxfSplineVectorKind, DxfSplineVectorSemanticIssue,
    DxfSplineVectorSemanticState, DxfSplineWeightSemanticState, DxfSplineWeightValueState,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineLinearPlanarRelation {
    UnavailableFlags,
    NotLinear,
    LinearPlanar,
    LinearWithoutPlanar,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineRationalWeightRelation {
    UnavailableFlags,
    ImplicitUnit {
        rational: bool,
        control_point_count: u32,
    },
    ExplicitMatched {
        rational: bool,
        weight_count: u32,
        invalid_count: u32,
        non_positive_count: u32,
    },
    ExplicitCountMismatch {
        rational: bool,
        weight_count: u32,
        control_point_count: u32,
        invalid_count: u32,
        non_positive_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplinePlanarNormalRelation {
    UnavailableFlags,
    NonPlanarOmitted,
    NonPlanarUnexpected { state: DxfSplineVectorSemanticState },
    PlanarMissing,
    PlanarExplicit { is_zero: bool },
    PlanarUnavailable { issue: DxfSplineVectorSemanticIssue },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineRelationEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    flags: DxfSplineFlagsSemantic,
    linear_planar: DxfSplineLinearPlanarRelation,
    rational_weight: DxfSplineRationalWeightRelation,
    planar_normal: DxfSplinePlanarNormalRelation,
}

impl DxfSplineRelationEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn flags(self) -> DxfSplineFlagsSemantic {
        self.flags
    }

    #[must_use]
    pub const fn linear_planar(self) -> DxfSplineLinearPlanarRelation {
        self.linear_planar
    }

    #[must_use]
    pub const fn rational_weight(self) -> DxfSplineRationalWeightRelation {
        self.rational_weight
    }

    #[must_use]
    pub const fn planar_normal(self) -> DxfSplinePlanarNormalRelation {
        self.planar_normal
    }
}

#[derive(Debug)]
pub struct DxfSplineRelationDirectory {
    source_id: DxfSourceId,
    semantics: DxfSplineAuxiliarySemanticDirectory,
    entries: Box<[DxfSplineRelationEntry]>,
}

impl DxfSplineRelationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.spline_auxiliary_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }
        let cards = semantics
            .auxiliary_directory()
            .point_directory()
            .card_directory();
        let mut entries = Vec::new();
        for weight in semantics.weights().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw_ordinal = weight.record().record().ordinal();
            let flags = cards
                .flags_for_raw_record(raw_ordinal)?
                .ok_or_else(invalid_internal_data)?;
            let normal = semantics
                .vector_for_kind(raw_ordinal, DxfSplineVectorKind::Normal)
                .ok_or_else(invalid_internal_data)?
                .state();
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfSplineRelationEntry {
                ordinal: compact_len(entries.len())?,
                record: weight.record(),
                flags,
                linear_planar: linear_planar(flags),
                rational_weight: rational_weight(
                    &semantics,
                    weight.ordinal(),
                    weight.state(),
                    flags,
                )?,
                planar_normal: planar_normal(flags, normal),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfSplineAuxiliarySemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfSplineRelationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfSplineRelationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_raw_record(&self, raw_ordinal: u64) -> Option<DxfSplineRelationEntry> {
        self.entries
            .binary_search_by_key(&raw_ordinal, |entry| entry.record().record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_relation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineRelationDirectory, DxfError> {
        DxfSplineRelationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_relation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineRelationDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_relation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_relation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineRelationDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_relation_directory(cancellation)
    }
}

const fn linear_planar(flags: DxfSplineFlagsSemantic) -> DxfSplineLinearPlanarRelation {
    let DxfSplineFlagsSemantic::Explicit(flags) = flags else {
        return DxfSplineLinearPlanarRelation::UnavailableFlags;
    };
    match (flags.is_linear(), flags.is_planar()) {
        (false, _) => DxfSplineLinearPlanarRelation::NotLinear,
        (true, true) => DxfSplineLinearPlanarRelation::LinearPlanar,
        (true, false) => DxfSplineLinearPlanarRelation::LinearWithoutPlanar,
    }
}

fn rational_weight(
    semantics: &DxfSplineAuxiliarySemanticDirectory,
    weight_ordinal: u64,
    weight: DxfSplineWeightSemanticState,
    flags: DxfSplineFlagsSemantic,
) -> Result<DxfSplineRationalWeightRelation, DxfError> {
    let DxfSplineFlagsSemantic::Explicit(flags) = flags else {
        return Ok(DxfSplineRationalWeightRelation::UnavailableFlags);
    };
    let (invalid_count, non_positive_count) = weight_issue_counts(
        semantics
            .values_for_weight(weight_ordinal)
            .ok_or_else(invalid_internal_data)?,
    )?;
    Ok(match weight {
        DxfSplineWeightSemanticState::ImplicitUnit {
            control_point_count,
        } => DxfSplineRationalWeightRelation::ImplicitUnit {
            rational: flags.is_rational(),
            control_point_count,
        },
        DxfSplineWeightSemanticState::ExplicitMatched { weight_count } => {
            DxfSplineRationalWeightRelation::ExplicitMatched {
                rational: flags.is_rational(),
                weight_count,
                invalid_count,
                non_positive_count,
            }
        }
        DxfSplineWeightSemanticState::CountMismatch {
            weight_count,
            control_point_count,
        } => DxfSplineRationalWeightRelation::ExplicitCountMismatch {
            rational: flags.is_rational(),
            weight_count,
            control_point_count,
            invalid_count,
            non_positive_count,
        },
    })
}

fn weight_issue_counts(values: &[DxfSplineWeightValueState]) -> Result<(u32, u32), DxfError> {
    let invalid = values
        .iter()
        .filter(|value| matches!(value, DxfSplineWeightValueState::Invalid { .. }))
        .count();
    let non_positive = values
        .iter()
        .filter(|value| matches!(value, DxfSplineWeightValueState::NonPositive { .. }))
        .count();
    Ok((
        u32::try_from(invalid).map_err(|_| invalid_internal_data())?,
        u32::try_from(non_positive).map_err(|_| invalid_internal_data())?,
    ))
}

fn planar_normal(
    flags: DxfSplineFlagsSemantic,
    normal: DxfSplineVectorSemanticState,
) -> DxfSplinePlanarNormalRelation {
    let DxfSplineFlagsSemantic::Explicit(flags) = flags else {
        return DxfSplinePlanarNormalRelation::UnavailableFlags;
    };
    match (flags.is_planar(), normal) {
        (false, DxfSplineVectorSemanticState::Absent) => {
            DxfSplinePlanarNormalRelation::NonPlanarOmitted
        }
        (false, state) => DxfSplinePlanarNormalRelation::NonPlanarUnexpected { state },
        (true, DxfSplineVectorSemanticState::Absent) => {
            DxfSplinePlanarNormalRelation::PlanarMissing
        }
        (true, DxfSplineVectorSemanticState::Explicit(vector)) => {
            DxfSplinePlanarNormalRelation::PlanarExplicit {
                is_zero: vector.is_zero(),
            }
        }
        (true, DxfSplineVectorSemanticState::Unavailable(issue)) => {
            DxfSplinePlanarNormalRelation::PlanarUnavailable { issue }
        }
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

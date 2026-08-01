//! Effective SPLINE weight and vector values.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
    DxfSplineAuxiliaryDirectory, DxfSplineNumber, DxfSplineNumericIssue, DxfSplineRecordEntry,
    DxfSplineValue, DxfSplineVectorComponentState, DxfSplineVectorComponents, DxfSplineVectorEntry,
    DxfSplineVectorKind, DxfSplineVectorState, DxfSplineWeightState,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineWeightValueState {
    Explicit {
        evidence: DxfSplineValue,
        weight: DxfDouble,
    },
    Invalid {
        evidence: DxfSplineValue,
        issue: DxfAsciiNumericIssue,
    },
    NonPositive {
        evidence: DxfSplineValue,
        weight: DxfDouble,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineWeightValueRange {
    start: u32,
    end: u32,
}

impl DxfSplineWeightValueRange {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineWeightSemanticState {
    ImplicitUnit {
        control_point_count: u32,
    },
    ExplicitMatched {
        weight_count: u32,
    },
    CountMismatch {
        weight_count: u32,
        control_point_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineWeightSemanticEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    value_range: DxfSplineWeightValueRange,
    state: DxfSplineWeightSemanticState,
}

impl DxfSplineWeightSemanticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn value_range(self) -> DxfSplineWeightValueRange {
        self.value_range
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineWeightSemanticState {
        self.state
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineEffectiveVector {
    x: DxfDouble,
    y: DxfDouble,
    z: DxfDouble,
    explicit_components: DxfSplineVectorComponents,
}

impl DxfSplineEffectiveVector {
    #[must_use]
    pub const fn x(self) -> DxfDouble {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> DxfDouble {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> DxfDouble {
        self.z
    }

    #[must_use]
    pub const fn explicit_components(self) -> DxfSplineVectorComponents {
        self.explicit_components
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.x.to_f64() == 0.0 && self.y.to_f64() == 0.0 && self.z.to_f64() == 0.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineVectorSemanticIssue {
    duplicate_components: DxfSplineVectorComponents,
    invalid_components: DxfSplineVectorComponents,
    missing_x: bool,
}

impl DxfSplineVectorSemanticIssue {
    #[must_use]
    pub const fn duplicate_components(self) -> DxfSplineVectorComponents {
        self.duplicate_components
    }

    #[must_use]
    pub const fn invalid_components(self) -> DxfSplineVectorComponents {
        self.invalid_components
    }

    #[must_use]
    pub const fn is_missing_x(self) -> bool {
        self.missing_x
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineVectorSemanticState {
    Absent,
    Explicit(DxfSplineEffectiveVector),
    Unavailable(DxfSplineVectorSemanticIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineVectorSemanticEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    kind: DxfSplineVectorKind,
    state: DxfSplineVectorSemanticState,
}

impl DxfSplineVectorSemanticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfSplineVectorKind {
        self.kind
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineVectorSemanticState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfSplineAuxiliarySemanticDirectory {
    source_id: DxfSourceId,
    auxiliary: DxfSplineAuxiliaryDirectory,
    weights: Box<[DxfSplineWeightSemanticEntry]>,
    weight_values: Box<[DxfSplineWeightValueState]>,
    vectors: Box<[DxfSplineVectorSemanticEntry]>,
}

impl DxfSplineAuxiliarySemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let auxiliary = document.spline_auxiliary_directory(cancellation)?;
        if auxiliary.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: auxiliary.source_id(),
            });
        }
        let mut weights = Vec::new();
        let mut weight_values = Vec::new();
        for entry in auxiliary.weights().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            append_weight(&auxiliary, entry, &mut weights, &mut weight_values)?;
        }
        let mut vectors = Vec::new();
        for entry in auxiliary.vectors().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            vectors.try_reserve(1).map_err(|_| out_of_memory())?;
            vectors.push(DxfSplineVectorSemanticEntry {
                ordinal: compact_len(vectors.len())?,
                record: entry.record(),
                kind: entry.kind(),
                state: vector_state(&auxiliary, entry)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            auxiliary,
            weights: weights.into_boxed_slice(),
            weight_values: weight_values.into_boxed_slice(),
            vectors: vectors.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn auxiliary_directory(&self) -> &DxfSplineAuxiliaryDirectory {
        &self.auxiliary
    }

    #[must_use]
    pub fn weights(&self) -> &[DxfSplineWeightSemanticEntry] {
        &self.weights
    }

    #[must_use]
    pub fn weight_values(&self) -> &[DxfSplineWeightValueState] {
        &self.weight_values
    }

    #[must_use]
    pub fn vectors(&self) -> &[DxfSplineVectorSemanticEntry] {
        &self.vectors
    }

    #[must_use]
    pub fn values_for_weight(&self, ordinal: u64) -> Option<&[DxfSplineWeightValueState]> {
        let entry = self.weights.get(usize::try_from(ordinal).ok()?)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.weight_values.get(start..end)
    }

    #[must_use]
    pub fn vector_for_kind(
        &self,
        raw_ordinal: u64,
        kind: DxfSplineVectorKind,
    ) -> Option<DxfSplineVectorSemanticEntry> {
        self.vectors
            .iter()
            .copied()
            .find(|entry| entry.record().record().ordinal() == raw_ordinal && entry.kind() == kind)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_auxiliary_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAuxiliarySemanticDirectory, DxfError> {
        DxfSplineAuxiliarySemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_auxiliary_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAuxiliarySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_auxiliary_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_auxiliary_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineAuxiliarySemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_auxiliary_semantic_directory(cancellation)
    }
}

fn append_weight(
    auxiliary: &DxfSplineAuxiliaryDirectory,
    entry: crate::DxfSplineWeightEntry,
    weights: &mut Vec<DxfSplineWeightSemanticEntry>,
    values: &mut Vec<DxfSplineWeightValueState>,
) -> Result<(), DxfError> {
    let start = compact_len(values.len())?;
    for member in auxiliary
        .members_for_weight(entry.ordinal())
        .ok_or_else(invalid_internal_data)?
        .iter()
        .copied()
    {
        let evidence = auxiliary
            .value_for_member(member)
            .ok_or_else(invalid_internal_data)?;
        let state = match evidence.value() {
            Err(DxfSplineNumericIssue::InvalidAsciiNumber(issue)) => {
                DxfSplineWeightValueState::Invalid { evidence, issue }
            }
            Ok(DxfSplineNumber::Double(weight)) if weight.to_f64() <= 0.0 => {
                DxfSplineWeightValueState::NonPositive { evidence, weight }
            }
            Ok(DxfSplineNumber::Double(weight)) => {
                DxfSplineWeightValueState::Explicit { evidence, weight }
            }
            Ok(_) => return Err(invalid_internal_data()),
        };
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(state);
    }
    let end = compact_len(values.len())?;
    let state = match entry.state() {
        DxfSplineWeightState::ImplicitUnit {
            control_point_count,
        } => DxfSplineWeightSemanticState::ImplicitUnit {
            control_point_count,
        },
        DxfSplineWeightState::Explicit {
            weight_count,
            control_point_count: _,
            disposition: crate::DxfSplineWeightCountDisposition::Matched,
        } => DxfSplineWeightSemanticState::ExplicitMatched { weight_count },
        DxfSplineWeightState::Explicit {
            weight_count,
            control_point_count,
            disposition: crate::DxfSplineWeightCountDisposition::Mismatched,
        } => DxfSplineWeightSemanticState::CountMismatch {
            weight_count,
            control_point_count,
        },
    };
    weights.try_reserve(1).map_err(|_| out_of_memory())?;
    weights.push(DxfSplineWeightSemanticEntry {
        ordinal: compact_len(weights.len())?,
        record: entry.record(),
        value_range: DxfSplineWeightValueRange::new(start, end)?,
        state,
    });
    Ok(())
}

fn vector_state(
    auxiliary: &DxfSplineAuxiliaryDirectory,
    entry: DxfSplineVectorEntry,
) -> Result<DxfSplineVectorSemanticState, DxfError> {
    if entry.state() == DxfSplineVectorState::Absent {
        return Ok(DxfSplineVectorSemanticState::Absent);
    }
    let duplicate_components = match entry.state() {
        DxfSplineVectorState::Ambiguous {
            duplicate_components,
        } => duplicate_components,
        _ => empty_components(),
    };
    let mut invalid_bits = 0_u8;
    let x = scalar(auxiliary, entry.x(), 1, &mut invalid_bits)?;
    let y = scalar(auxiliary, entry.y(), 2, &mut invalid_bits)?;
    let z = scalar(auxiliary, entry.z(), 4, &mut invalid_bits)?;
    let invalid_components = components_from_bits(invalid_bits);
    let missing_x = matches!(entry.x(), DxfSplineVectorComponentState::Absent);
    if !duplicate_components.is_empty() || !invalid_components.is_empty() || missing_x {
        return Ok(DxfSplineVectorSemanticState::Unavailable(
            DxfSplineVectorSemanticIssue {
                duplicate_components,
                invalid_components,
                missing_x,
            },
        ));
    }
    let explicit_components = match entry.state() {
        DxfSplineVectorState::Present { components } => components,
        _ => return Err(invalid_internal_data()),
    };
    Ok(DxfSplineVectorSemanticState::Explicit(
        DxfSplineEffectiveVector {
            x: x.ok_or_else(invalid_internal_data)?,
            y: y.unwrap_or_else(zero),
            z: z.unwrap_or_else(zero),
            explicit_components,
        },
    ))
}

fn scalar(
    auxiliary: &DxfSplineAuxiliaryDirectory,
    state: DxfSplineVectorComponentState,
    bit: u8,
    invalid_bits: &mut u8,
) -> Result<Option<DxfDouble>, DxfError> {
    let DxfSplineVectorComponentState::Unique(member) = state else {
        return Ok(None);
    };
    match auxiliary
        .value_for_member(member)
        .ok_or_else(invalid_internal_data)?
        .value()
    {
        Ok(DxfSplineNumber::Double(value)) => Ok(Some(value)),
        Err(DxfSplineNumericIssue::InvalidAsciiNumber(_)) => {
            *invalid_bits |= bit;
            Ok(None)
        }
        Ok(_) => Err(invalid_internal_data()),
    }
}

fn components_from_bits(bits: u8) -> DxfSplineVectorComponents {
    DxfSplineVectorComponents::from_bits(bits)
}

fn empty_components() -> DxfSplineVectorComponents {
    DxfSplineVectorComponents::from_bits(0)
}

fn zero() -> DxfDouble {
    DxfDouble::from_f64(0.0)
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

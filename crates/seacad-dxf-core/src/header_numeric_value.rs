//! Typed numeric HEADER results and their semantic-state projections.

use std::num::NonZeroU64;

use crate::{
    DxfGroupCode, DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue,
    DxfSemanticValueState,
    ascii_numeric::DxfAsciiNumericIssue,
    header_scalar::{DxfDouble, DxfElapsedDays, DxfJulianDate},
};

/// Structural or lexical reason why a reviewed numeric HEADER field is invalid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHeaderNumericIssue {
    BooleanOutOfDomain {
        value: i16,
    },
    InvalidGroupCode(DxfGroupCode),
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MissingValue,
    MultipleValueGroups {
        group_count: NonZeroU64,
    },
    UnexpectedComponentCount {
        expected_count: NonZeroU64,
        observed_count: u64,
    },
    MultipleVariables {
        occurrence_count: NonZeroU64,
    },
}

/// One schema-selected numeric representation and its four-state provenance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHeaderNumericValue {
    Boolean(DxfSemanticValue<bool, DxfHeaderNumericIssue>),
    Double(DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>),
    Double2([DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; 2]),
    Double3([DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; 3]),
    ElapsedDays(DxfSemanticValue<DxfElapsedDays, DxfHeaderNumericIssue>),
    Int16(DxfSemanticValue<i16, DxfHeaderNumericIssue>),
    JulianDate(DxfSemanticValue<DxfJulianDate, DxfHeaderNumericIssue>),
}

impl DxfHeaderNumericValue {
    #[must_use]
    pub const fn state(&self) -> DxfSemanticValueState {
        match self {
            Self::Boolean(value) => value.state(),
            Self::Double(value) => value.state(),
            Self::Double2(values) => double2_state(values),
            Self::Double3(values) => double3_state(values),
            Self::ElapsedDays(value) => value.state(),
            Self::Int16(value) => value.state(),
            Self::JulianDate(value) => value.state(),
        }
    }

    #[must_use]
    pub const fn field_provenance(&self) -> DxfSemanticFieldProvenance {
        match self {
            Self::Boolean(value) => value.field_provenance(),
            Self::Double(value) => value.field_provenance(),
            Self::Double2(values) => values[0].field_provenance(),
            Self::Double3(values) => values[0].field_provenance(),
            Self::ElapsedDays(value) => value.field_provenance(),
            Self::Int16(value) => value.field_provenance(),
            Self::JulianDate(value) => value.field_provenance(),
        }
    }

    /// Returns scalar evidence or the first source-ordered tuple component.
    ///
    /// Use `as_double2` or `as_double3` to inspect every component span.
    #[must_use]
    pub const fn raw_provenance(&self) -> Option<DxfRawValueProvenance> {
        match self {
            Self::Boolean(value) => value.raw_provenance(),
            Self::Double(value) => value.raw_provenance(),
            Self::Double2(values) => first_raw_provenance(values),
            Self::Double3(values) => first_raw_provenance(values),
            Self::ElapsedDays(value) => value.raw_provenance(),
            Self::Int16(value) => value.raw_provenance(),
            Self::JulianDate(value) => value.raw_provenance(),
        }
    }

    #[must_use]
    pub const fn as_double(&self) -> Option<&DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>> {
        match self {
            Self::Double(value) => Some(value),
            Self::Boolean(_)
            | Self::Double2(_)
            | Self::Double3(_)
            | Self::ElapsedDays(_)
            | Self::Int16(_)
            | Self::JulianDate(_) => None,
        }
    }

    #[must_use]
    pub const fn as_double2(
        &self,
    ) -> Option<&[DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; 2]> {
        match self {
            Self::Double2(values) => Some(values),
            Self::Boolean(_)
            | Self::Double(_)
            | Self::Double3(_)
            | Self::ElapsedDays(_)
            | Self::Int16(_)
            | Self::JulianDate(_) => None,
        }
    }

    #[must_use]
    pub const fn as_double3(
        &self,
    ) -> Option<&[DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; 3]> {
        match self {
            Self::Double3(values) => Some(values),
            Self::Boolean(_)
            | Self::Double(_)
            | Self::Double2(_)
            | Self::ElapsedDays(_)
            | Self::Int16(_)
            | Self::JulianDate(_) => None,
        }
    }

    #[must_use]
    pub const fn as_elapsed_days(
        &self,
    ) -> Option<&DxfSemanticValue<DxfElapsedDays, DxfHeaderNumericIssue>> {
        match self {
            Self::ElapsedDays(value) => Some(value),
            Self::Boolean(_)
            | Self::Double(_)
            | Self::Double2(_)
            | Self::Double3(_)
            | Self::Int16(_)
            | Self::JulianDate(_) => None,
        }
    }

    #[must_use]
    pub const fn as_int16(&self) -> Option<&DxfSemanticValue<i16, DxfHeaderNumericIssue>> {
        match self {
            Self::Int16(value) => Some(value),
            Self::Boolean(_)
            | Self::Double(_)
            | Self::Double2(_)
            | Self::Double3(_)
            | Self::ElapsedDays(_)
            | Self::JulianDate(_) => None,
        }
    }

    #[must_use]
    pub const fn as_julian_date(
        &self,
    ) -> Option<&DxfSemanticValue<DxfJulianDate, DxfHeaderNumericIssue>> {
        match self {
            Self::JulianDate(value) => Some(value),
            Self::Boolean(_)
            | Self::Double(_)
            | Self::Double2(_)
            | Self::Double3(_)
            | Self::ElapsedDays(_)
            | Self::Int16(_) => None,
        }
    }

    #[must_use]
    pub const fn as_boolean(&self) -> Option<&DxfSemanticValue<bool, DxfHeaderNumericIssue>> {
        match self {
            Self::Boolean(value) => Some(value),
            Self::Double(_)
            | Self::Double2(_)
            | Self::Double3(_)
            | Self::ElapsedDays(_)
            | Self::Int16(_)
            | Self::JulianDate(_) => None,
        }
    }
}

const fn double2_state(
    values: &[DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; 2],
) -> DxfSemanticValueState {
    match (values[0].state(), values[1].state()) {
        (DxfSemanticValueState::Explicit, DxfSemanticValueState::Explicit) => {
            DxfSemanticValueState::Explicit
        }
        (DxfSemanticValueState::Defaulted, DxfSemanticValueState::Defaulted) => {
            DxfSemanticValueState::Defaulted
        }
        (DxfSemanticValueState::Absent, DxfSemanticValueState::Absent) => {
            DxfSemanticValueState::Absent
        }
        _ => DxfSemanticValueState::Invalid,
    }
}

const fn double3_state(
    values: &[DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; 3],
) -> DxfSemanticValueState {
    match (values[0].state(), values[1].state(), values[2].state()) {
        (
            DxfSemanticValueState::Explicit,
            DxfSemanticValueState::Explicit,
            DxfSemanticValueState::Explicit,
        ) => DxfSemanticValueState::Explicit,
        (
            DxfSemanticValueState::Defaulted,
            DxfSemanticValueState::Defaulted,
            DxfSemanticValueState::Defaulted,
        ) => DxfSemanticValueState::Defaulted,
        (
            DxfSemanticValueState::Absent,
            DxfSemanticValueState::Absent,
            DxfSemanticValueState::Absent,
        ) => DxfSemanticValueState::Absent,
        _ => DxfSemanticValueState::Invalid,
    }
}

const fn first_raw_provenance<const N: usize>(
    values: &[DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; N],
) -> Option<DxfRawValueProvenance> {
    let mut index = 0;
    while index < N {
        if let Some(raw) = values[index].raw_provenance() {
            return Some(raw);
        }
        index += 1;
    }
    None
}

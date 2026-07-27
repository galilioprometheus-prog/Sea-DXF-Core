//! Source-anchored numeric HEADER semantics over the shared raw document API.

use std::{fmt, io, num::NonZeroU64};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfGroupCode, DxfHeaderSchemaMatch, DxfHeaderVariable, DxfHeaderVariableLookupState,
    DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSemanticValueState, DxfSourceId,
    ascii_group::trim_horizontal_ascii,
    generated::header_schema::{
        DxfHeaderSchemaField, DxfSchemaStorageKind, HEADER_FIELDS, SCHEMA_VERSION,
    },
};

const HEADER_NAMESPACE: &str = "header";
const STACK_ASCII_NUMERIC_BYTES: usize = 128;
const SOURCE_READ_CHUNK_BYTES: usize = 8 * 1024;

/// Exact IEEE-754 binary64 payload without `f64` equality or hashing ambiguity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDouble(u64);

impl DxfDouble {
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub fn from_f64(value: f64) -> Self {
        Self(value.to_bits())
    }

    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn to_f64(self) -> f64 {
        f64::from_bits(self.0)
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.to_f64().is_finite()
    }
}

/// Finite day value split at the integer boundary without calendar conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDayParts {
    whole_days: i64,
    fractional_day: DxfDouble,
}

impl DxfDayParts {
    fn from_raw(raw: DxfDouble) -> Option<Self> {
        let value = raw.to_f64();
        let whole = value.trunc();
        const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;
        if !value.is_finite() || whole < i64::MIN as f64 || whole >= I64_MAX_EXCLUSIVE {
            return None;
        }
        Some(Self {
            whole_days: whole as i64,
            fractional_day: DxfDouble::from_f64(value - whole),
        })
    }

    #[must_use]
    pub const fn whole_days(self) -> i64 {
        self.whole_days
    }

    /// Fraction following Autodesk's truncation-at-zero representation.
    #[must_use]
    pub const fn fractional_day(self) -> DxfDouble {
        self.fractional_day
    }
}

/// Exact DXF Julian-date scalar; no timezone or calendar interpretation is inferred.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfJulianDate(DxfDouble);

impl DxfJulianDate {
    #[must_use]
    pub const fn from_raw(raw: DxfDouble) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> DxfDouble {
        self.0
    }

    /// Splits a finite, `i64`-bounded scalar into whole and fractional days.
    #[must_use]
    pub fn day_parts(self) -> Option<DxfDayParts> {
        DxfDayParts::from_raw(self.0)
    }
}

/// Exact DXF elapsed-days scalar; it is not an absolute date or timezone value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfElapsedDays(DxfDouble);

impl DxfElapsedDays {
    #[must_use]
    pub const fn from_raw(raw: DxfDouble) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> DxfDouble {
        self.0
    }

    /// Splits a finite, `i64`-bounded scalar into whole and fractional days.
    #[must_use]
    pub fn day_parts(self) -> Option<DxfDayParts> {
        DxfDayParts::from_raw(self.0)
    }
}

/// Exact failure while interpreting one ASCII numeric value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAsciiNumericIssue {
    Empty,
    InvalidSyntax { token_offset: u64 },
    OutOfRange,
}

/// Structural or lexical reason why a reviewed numeric HEADER field is invalid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHeaderNumericIssue {
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
            Self::Double2(_)
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
            Self::Double(_)
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
            Self::Double(_)
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
            Self::Double(_)
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
            Self::Double(_)
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
            Self::Double(_)
            | Self::Double2(_)
            | Self::Double3(_)
            | Self::ElapsedDays(_)
            | Self::Int16(_) => None,
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

/// Numeric HEADER field resolved from one generated schema ordinal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderNumericEntry {
    schema_ordinal: u32,
    schema_field_id: &'static str,
    dxf_name: &'static str,
    group_codes: &'static [i16],
    value: DxfHeaderNumericValue,
}

impl DxfHeaderNumericEntry {
    /// Append-only position in the complete HEADER schema.
    ///
    /// Persist `schema_field_id` as the canonical identity.
    #[must_use]
    pub const fn schema_ordinal(self) -> u64 {
        self.schema_ordinal as u64
    }

    #[must_use]
    pub const fn schema_field_id(self) -> &'static str {
        self.schema_field_id
    }

    #[must_use]
    pub const fn dxf_name(self) -> &'static str {
        self.dxf_name
    }

    #[must_use]
    pub const fn group_codes(self) -> &'static [i16] {
        self.group_codes
    }

    #[must_use]
    pub const fn value(&self) -> &DxfHeaderNumericValue {
        &self.value
    }
}

/// Lazy schema-ordered directory of every generated numeric HEADER field.
pub struct DxfHeaderNumericDirectory {
    source_id: DxfSourceId,
    entries: Box<[DxfHeaderNumericEntry]>,
}

impl DxfHeaderNumericDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let raw_directory = document.resolve_header_schema(cancellation)?;
        if raw_directory.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: raw_directory.source_id(),
            });
        }
        let numeric_count = HEADER_FIELDS
            .iter()
            .filter(|field| is_numeric_storage(field.storage))
            .count();
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(numeric_count)
            .map_err(|_| out_of_memory())?;
        for (ordinal, field) in HEADER_FIELDS.iter().enumerate() {
            ensure_not_cancelled(cancellation)?;
            let value = match field.storage {
                DxfSchemaStorageKind::Double => {
                    let matched = schema_match(&raw_directory, ordinal, field)?;
                    let spec = FieldSpec::from_schema(field);
                    Some(DxfHeaderNumericValue::Double(double_field(
                        document,
                        matched,
                        spec,
                        cancellation,
                    )?))
                }
                DxfSchemaStorageKind::Double2 => {
                    let matched = schema_match(&raw_directory, ordinal, field)?;
                    let spec = FieldSpec::from_schema(field);
                    Some(DxfHeaderNumericValue::Double2(double_tuple_field(
                        document,
                        matched,
                        spec,
                        cancellation,
                    )?))
                }
                DxfSchemaStorageKind::Double3 => {
                    let matched = schema_match(&raw_directory, ordinal, field)?;
                    let spec = FieldSpec::from_schema(field);
                    Some(DxfHeaderNumericValue::Double3(double_tuple_field(
                        document,
                        matched,
                        spec,
                        cancellation,
                    )?))
                }
                DxfSchemaStorageKind::ElapsedDays => {
                    let matched = schema_match(&raw_directory, ordinal, field)?;
                    let spec = FieldSpec::from_schema(field);
                    Some(DxfHeaderNumericValue::ElapsedDays(
                        double_field(document, matched, spec, cancellation)?
                            .map_value(DxfElapsedDays::from_raw),
                    ))
                }
                DxfSchemaStorageKind::Int16 => {
                    let matched = schema_match(&raw_directory, ordinal, field)?;
                    let spec = FieldSpec::from_schema(field);
                    Some(DxfHeaderNumericValue::Int16(int16_field(
                        document,
                        matched,
                        spec,
                        cancellation,
                    )?))
                }
                DxfSchemaStorageKind::JulianDate => {
                    let matched = schema_match(&raw_directory, ordinal, field)?;
                    let spec = FieldSpec::from_schema(field);
                    Some(DxfHeaderNumericValue::JulianDate(
                        double_field(document, matched, spec, cancellation)?
                            .map_value(DxfJulianDate::from_raw),
                    ))
                }
                DxfSchemaStorageKind::ExactText | DxfSchemaStorageKind::Handle => None,
            };
            if let Some(value) = value {
                entries.push(DxfHeaderNumericEntry {
                    schema_ordinal: u32::try_from(ordinal).map_err(|_| invalid_internal_data())?,
                    schema_field_id: field.id,
                    dxf_name: field.dxf_name,
                    group_codes: field.group_codes,
                    value,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> &'static str {
        SCHEMA_VERSION
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHeaderNumericEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, schema_field_id: &str) -> Option<&DxfHeaderNumericEntry> {
        self.entries
            .iter()
            .find(|entry| entry.schema_field_id == schema_field_id)
    }

    /// Finds a numeric field by its append-only complete-schema position.
    #[must_use]
    pub fn entry_at_schema_ordinal(&self, schema_ordinal: u64) -> Option<&DxfHeaderNumericEntry> {
        let schema_ordinal = u32::try_from(schema_ordinal).ok()?;
        self.entries
            .binary_search_by_key(&schema_ordinal, |entry| entry.schema_ordinal)
            .ok()
            .and_then(|index| self.entries.get(index))
    }
}

impl fmt::Debug for DxfHeaderNumericDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfHeaderNumericDirectory")
            .field("source_id", &self.source_id)
            .field("numeric_field_count", &self.entries.len())
            .finish()
    }
}

/// Fixed-size numeric projection for the first reviewed HEADER schema slice.
///
/// Values are source spelling or binary payload interpretations only. This
/// view does not apply defaults, enum meanings, unit conversion, or ranges.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderNumericView {
    source_id: DxfSourceId,
    acad_maintenance_version: DxfSemanticValue<i16, DxfHeaderNumericIssue>,
    angle_base: DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>,
    angle_direction: DxfSemanticValue<i16, DxfHeaderNumericIssue>,
    attribute_mode: DxfSemanticValue<i16, DxfHeaderNumericIssue>,
    angular_units: DxfSemanticValue<i16, DxfHeaderNumericIssue>,
    angular_precision: DxfSemanticValue<i16, DxfHeaderNumericIssue>,
}

impl DxfHeaderNumericView {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let directory = DxfHeaderNumericDirectory::from_document(document, cancellation)?;
        let view = Self {
            source_id: document.source_id(),
            acad_maintenance_version: required_int16(&directory, "acadmaintver")?,
            angle_base: required_double(&directory, "angbase")?,
            angle_direction: required_int16(&directory, "angdir")?,
            attribute_mode: required_int16(&directory, "attmode")?,
            angular_units: required_int16(&directory, "aunits")?,
            angular_precision: required_int16(&directory, "auprec")?,
        };
        ensure_not_cancelled(cancellation)?;
        Ok(view)
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn acad_maintenance_version(&self) -> &DxfSemanticValue<i16, DxfHeaderNumericIssue> {
        &self.acad_maintenance_version
    }

    #[must_use]
    pub const fn angle_base(&self) -> &DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue> {
        &self.angle_base
    }

    #[must_use]
    pub const fn angle_direction(&self) -> &DxfSemanticValue<i16, DxfHeaderNumericIssue> {
        &self.angle_direction
    }

    #[must_use]
    pub const fn attribute_mode(&self) -> &DxfSemanticValue<i16, DxfHeaderNumericIssue> {
        &self.attribute_mode
    }

    #[must_use]
    pub const fn angular_units(&self) -> &DxfSemanticValue<i16, DxfHeaderNumericIssue> {
        &self.angular_units
    }

    #[must_use]
    pub const fn angular_precision(&self) -> &DxfSemanticValue<i16, DxfHeaderNumericIssue> {
        &self.angular_precision
    }
}

impl DxfRawDocumentView<'_> {
    /// Lazily resolves every generated numeric HEADER field in schema order.
    pub fn header_numeric_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericDirectory, DxfError> {
        DxfHeaderNumericDirectory::from_document(self, cancellation)
    }

    /// Lazily resolves and reads the reviewed numeric HEADER schema slice.
    pub fn header_numeric_view(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericView, DxfError> {
        DxfHeaderNumericView::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn header_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self).header_numeric_directory(cancellation)
    }

    pub fn header_numeric_view(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericView, DxfError> {
        DxfRawDocumentView::from(self).header_numeric_view(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn header_numeric_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericDirectory, DxfError> {
        DxfRawDocumentView::from(self).header_numeric_directory(cancellation)
    }

    pub fn header_numeric_view(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericView, DxfError> {
        DxfRawDocumentView::from(self).header_numeric_view(cancellation)
    }
}

#[derive(Clone, Copy)]
struct FieldSpec {
    id: &'static str,
    group_codes: &'static [i16],
}

impl FieldSpec {
    const fn from_schema(field: &DxfHeaderSchemaField) -> Self {
        Self {
            id: field.id,
            group_codes: field.group_codes,
        }
    }

    fn single_group_code(self) -> Result<i16, DxfError> {
        match self.group_codes {
            [group_code] => Ok(*group_code),
            _ => Err(invalid_internal_data()),
        }
    }
}

fn is_numeric_storage(storage: DxfSchemaStorageKind) -> bool {
    matches!(
        storage,
        DxfSchemaStorageKind::Double
            | DxfSchemaStorageKind::Double2
            | DxfSchemaStorageKind::Double3
            | DxfSchemaStorageKind::ElapsedDays
            | DxfSchemaStorageKind::Int16
            | DxfSchemaStorageKind::JulianDate
    )
}

fn schema_match<'a>(
    directory: &'a crate::DxfHeaderSchemaDirectory,
    ordinal: usize,
    field: &DxfHeaderSchemaField,
) -> Result<&'a DxfHeaderSchemaMatch, DxfError> {
    let ordinal = u64::try_from(ordinal).map_err(|_| invalid_internal_data())?;
    let matched = directory
        .match_at(ordinal)
        .ok_or_else(invalid_internal_data)?;
    if matched.schema_ordinal() != ordinal
        || matched.schema_field_id() != field.id
        || matched.dxf_name() != field.dxf_name
    {
        return Err(invalid_internal_data());
    }
    Ok(matched)
}

fn required_int16(
    directory: &DxfHeaderNumericDirectory,
    schema_field_id: &str,
) -> Result<DxfSemanticValue<i16, DxfHeaderNumericIssue>, DxfError> {
    match directory
        .entry(schema_field_id)
        .map(DxfHeaderNumericEntry::value)
    {
        Some(DxfHeaderNumericValue::Int16(value)) => Ok(*value),
        Some(_) | None => Err(invalid_internal_data()),
    }
}

fn required_double(
    directory: &DxfHeaderNumericDirectory,
    schema_field_id: &str,
) -> Result<DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>, DxfError> {
    match directory
        .entry(schema_field_id)
        .map(DxfHeaderNumericEntry::value)
    {
        Some(DxfHeaderNumericValue::Double(value)) => Ok(*value),
        Some(_) | None => Err(invalid_internal_data()),
    }
}

fn double_tuple_field<const N: usize>(
    document: DxfRawDocumentView<'_>,
    matched: &DxfHeaderSchemaMatch,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<[DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>; N], DxfError> {
    if N == 0 || spec.group_codes.len() != N {
        return Err(invalid_internal_data());
    }
    ensure_not_cancelled(cancellation)?;
    let field = field_provenance(document.source_id(), spec);
    let mut values = [DxfSemanticValue::absent(field); N];
    match matched.state() {
        DxfHeaderVariableLookupState::Absent => return Ok(values),
        DxfHeaderVariableLookupState::Ambiguous => {
            let occurrence_count =
                NonZeroU64::new(matched.occurrence_count()).ok_or_else(invalid_internal_data)?;
            let raw = matched
                .conflicting()
                .or_else(|| matched.primary())
                .map(|variable| variable_provenance(document, variable))
                .transpose()?;
            values.fill(DxfSemanticValue::invalid(
                DxfHeaderNumericIssue::MultipleVariables { occurrence_count },
                field,
                raw,
            ));
            return Ok(values);
        }
        DxfHeaderVariableLookupState::Unique => {}
    }

    let variable = matched.primary().ok_or_else(invalid_internal_data)?;
    let range = variable.value_groups();
    let observed_count = range.len();
    let expected_count = u64::try_from(N).map_err(|_| invalid_internal_data())?;
    let expected_count = NonZeroU64::new(expected_count).ok_or_else(invalid_internal_data)?;
    if observed_count > expected_count.get() {
        let extra_occurrence = range
            .start()
            .checked_add(expected_count.get())
            .ok_or_else(invalid_internal_data)?;
        let extra = document
            .group(extra_occurrence)
            .ok_or_else(invalid_internal_data)?;
        values.fill(DxfSemanticValue::invalid(
            DxfHeaderNumericIssue::UnexpectedComponentCount {
                expected_count,
                observed_count,
            },
            field,
            Some(group_provenance(extra)?),
        ));
        return Ok(values);
    }

    let marker_raw = marker_provenance(variable)?;
    for index in 0..N {
        ensure_not_cancelled(cancellation)?;
        let slot = values.get_mut(index).ok_or_else(invalid_internal_data)?;
        let index_u64 = u64::try_from(index).map_err(|_| invalid_internal_data())?;
        if index_u64 >= observed_count {
            *slot = DxfSemanticValue::invalid(
                DxfHeaderNumericIssue::MissingValue,
                field,
                Some(marker_raw),
            );
            continue;
        }
        let occurrence = range
            .start()
            .checked_add(index_u64)
            .ok_or_else(invalid_internal_data)?;
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        let expected_group_code = spec
            .group_codes
            .get(index)
            .copied()
            .ok_or_else(invalid_internal_data)?;
        let raw = group_provenance(group)?;
        if group.group_code().value() != expected_group_code {
            *slot = DxfSemanticValue::invalid(
                DxfHeaderNumericIssue::InvalidGroupCode(group.group_code()),
                field,
                Some(raw),
            );
            continue;
        }
        *slot = match decode_double(document, group, cancellation)? {
            Ok(value) => DxfSemanticValue::explicit(value, field, raw),
            Err(issue) => DxfSemanticValue::invalid(issue, field, Some(raw)),
        };
    }
    ensure_not_cancelled(cancellation)?;
    Ok(values)
}

enum ResolvedGroup {
    Absent,
    Invalid {
        issue: DxfHeaderNumericIssue,
        raw: Option<DxfRawValueProvenance>,
    },
    Explicit(DxfRawGroup),
}

fn int16_field(
    document: DxfRawDocumentView<'_>,
    matched: &DxfHeaderSchemaMatch,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSemanticValue<i16, DxfHeaderNumericIssue>, DxfError> {
    let field = field_provenance(document.source_id(), spec);
    match resolve_group(document, matched, spec, cancellation)? {
        ResolvedGroup::Absent => Ok(DxfSemanticValue::absent(field)),
        ResolvedGroup::Invalid { issue, raw } => Ok(DxfSemanticValue::invalid(issue, field, raw)),
        ResolvedGroup::Explicit(group) => {
            let raw = group_provenance(group)?;
            match decode_i16(document, group, cancellation)? {
                Ok(value) => Ok(DxfSemanticValue::explicit(value, field, raw)),
                Err(issue) => Ok(DxfSemanticValue::invalid(issue, field, Some(raw))),
            }
        }
    }
}

fn double_field(
    document: DxfRawDocumentView<'_>,
    matched: &DxfHeaderSchemaMatch,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>, DxfError> {
    let field = field_provenance(document.source_id(), spec);
    match resolve_group(document, matched, spec, cancellation)? {
        ResolvedGroup::Absent => Ok(DxfSemanticValue::absent(field)),
        ResolvedGroup::Invalid { issue, raw } => Ok(DxfSemanticValue::invalid(issue, field, raw)),
        ResolvedGroup::Explicit(group) => {
            let raw = group_provenance(group)?;
            match decode_double(document, group, cancellation)? {
                Ok(value) => Ok(DxfSemanticValue::explicit(value, field, raw)),
                Err(issue) => Ok(DxfSemanticValue::invalid(issue, field, Some(raw))),
            }
        }
    }
}

fn resolve_group(
    document: DxfRawDocumentView<'_>,
    matched: &DxfHeaderSchemaMatch,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<ResolvedGroup, DxfError> {
    ensure_not_cancelled(cancellation)?;
    match matched.state() {
        DxfHeaderVariableLookupState::Absent => Ok(ResolvedGroup::Absent),
        DxfHeaderVariableLookupState::Ambiguous => {
            let occurrence_count =
                NonZeroU64::new(matched.occurrence_count()).ok_or_else(invalid_internal_data)?;
            let evidence = matched
                .conflicting()
                .or_else(|| matched.primary())
                .map(|variable| variable_provenance(document, variable))
                .transpose()?;
            Ok(ResolvedGroup::Invalid {
                issue: DxfHeaderNumericIssue::MultipleVariables { occurrence_count },
                raw: evidence,
            })
        }
        DxfHeaderVariableLookupState::Unique => {
            let variable = matched.primary().ok_or_else(invalid_internal_data)?;
            resolve_unique_variable(document, variable, spec.single_group_code()?)
        }
    }
}

fn resolve_unique_variable(
    document: DxfRawDocumentView<'_>,
    variable: DxfHeaderVariable,
    expected_group_code: i16,
) -> Result<ResolvedGroup, DxfError> {
    let range = variable.value_groups();
    match range.len() {
        0 => Ok(ResolvedGroup::Invalid {
            issue: DxfHeaderNumericIssue::MissingValue,
            raw: Some(marker_provenance(variable)?),
        }),
        1 => {
            let group = document
                .group(range.start())
                .ok_or_else(invalid_internal_data)?;
            if group.group_code().value() == expected_group_code {
                Ok(ResolvedGroup::Explicit(group))
            } else {
                Ok(ResolvedGroup::Invalid {
                    issue: DxfHeaderNumericIssue::InvalidGroupCode(group.group_code()),
                    raw: Some(group_provenance(group)?),
                })
            }
        }
        count => {
            let group_count = NonZeroU64::new(count).ok_or_else(invalid_internal_data)?;
            let conflict_occurrence = range
                .start()
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            let conflict = document
                .group(conflict_occurrence)
                .ok_or_else(invalid_internal_data)?;
            Ok(ResolvedGroup::Invalid {
                issue: DxfHeaderNumericIssue::MultipleValueGroups { group_count },
                raw: Some(group_provenance(conflict)?),
            })
        }
    }
}

fn decode_i16(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Result<i16, DxfHeaderNumericIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    match document.format() {
        DxfRawDocumentFormat::Ascii => read_ascii_numeric(
            document,
            group.value_payload_span(),
            parse_ascii_i16,
            cancellation,
        ),
        DxfRawDocumentFormat::Binary => {
            let mut bytes = [0_u8; 2];
            read_fixed_payload(document, group, &mut bytes, cancellation)?;
            Ok(Ok(i16::from_le_bytes(bytes)))
        }
    }
}

fn decode_double(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfDouble, DxfHeaderNumericIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    match document.format() {
        DxfRawDocumentFormat::Ascii => read_ascii_numeric(
            document,
            group.value_payload_span(),
            parse_ascii_double,
            cancellation,
        ),
        DxfRawDocumentFormat::Binary => {
            let mut bytes = [0_u8; 8];
            read_fixed_payload(document, group, &mut bytes, cancellation)?;
            Ok(Ok(DxfDouble::from_bits(u64::from_le_bytes(bytes))))
        }
    }
}

fn read_fixed_payload<const N: usize>(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    destination: &mut [u8; N],
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    if group.value_payload_span().len() != N as u64 {
        return Err(invalid_internal_data());
    }
    document.read_span(group.value_payload_span(), destination)?;
    ensure_not_cancelled(cancellation)
}

fn read_ascii_numeric<T>(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    parse: fn(&[u8]) -> Result<T, DxfAsciiNumericIssue>,
    cancellation: &DxfCancellationToken,
) -> Result<Result<T, DxfHeaderNumericIssue>, DxfError> {
    let length = usize::try_from(span.len()).map_err(|_| invalid_internal_data())?;
    if length <= STACK_ASCII_NUMERIC_BYTES {
        let mut bytes = [0_u8; STACK_ASCII_NUMERIC_BYTES];
        read_span_cancelled(document, span, &mut bytes[..length], cancellation)?;
        return parse_ascii_numeric(&bytes[..length], parse);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| out_of_memory())?;
    bytes.resize(length, 0);
    read_span_cancelled(document, span, &mut bytes, cancellation)?;
    parse_ascii_numeric(&bytes, parse)
}

fn read_span_cancelled(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    destination: &mut [u8],
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    if u64::try_from(destination.len()).map_err(|_| invalid_internal_data())? != span.len() {
        return Err(invalid_internal_data());
    }
    let mut consumed = 0_usize;
    while consumed < destination.len() {
        ensure_not_cancelled(cancellation)?;
        let chunk_len = (destination.len() - consumed).min(SOURCE_READ_CHUNK_BYTES);
        let start = span
            .start()
            .checked_add(u64::try_from(consumed).map_err(|_| invalid_internal_data())?)
            .ok_or_else(invalid_internal_data)?;
        let chunk_span = ByteSpan::from_start_and_len(start, chunk_len as u64)
            .ok_or_else(invalid_internal_data)?;
        let end = consumed
            .checked_add(chunk_len)
            .ok_or_else(invalid_internal_data)?;
        document.read_span(chunk_span, &mut destination[consumed..end])?;
        consumed = end;
    }
    ensure_not_cancelled(cancellation)
}

fn parse_ascii_numeric<T>(
    bytes: &[u8],
    parse: fn(&[u8]) -> Result<T, DxfAsciiNumericIssue>,
) -> Result<Result<T, DxfHeaderNumericIssue>, DxfError> {
    let token = trim_horizontal_ascii(bytes).ok_or_else(invalid_internal_data)?;
    Ok(parse(token).map_err(DxfHeaderNumericIssue::InvalidAsciiNumber))
}

fn parse_ascii_i16(token: &[u8]) -> Result<i16, DxfAsciiNumericIssue> {
    if token.is_empty() {
        return Err(DxfAsciiNumericIssue::Empty);
    }
    let (negative, digits, offset) = match token[0] {
        b'+' => (false, &token[1..], 1_usize),
        b'-' => (true, &token[1..], 1_usize),
        _ => (false, token, 0_usize),
    };
    if digits.is_empty() {
        return Err(invalid_syntax(offset));
    }
    let mut magnitude = 0_i32;
    for (index, byte) in digits.iter().copied().enumerate() {
        if !byte.is_ascii_digit() {
            return Err(invalid_syntax(offset + index));
        }
        magnitude = magnitude
            .checked_mul(10)
            .and_then(|value| value.checked_add(i32::from(byte - b'0')))
            .ok_or(DxfAsciiNumericIssue::OutOfRange)?;
        let limit = if negative { 32_768 } else { 32_767 };
        if magnitude > limit {
            return Err(DxfAsciiNumericIssue::OutOfRange);
        }
    }
    let signed = if negative { -magnitude } else { magnitude };
    i16::try_from(signed).map_err(|_| DxfAsciiNumericIssue::OutOfRange)
}

fn parse_ascii_double(token: &[u8]) -> Result<DxfDouble, DxfAsciiNumericIssue> {
    validate_double_syntax(token)?;
    let text = std::str::from_utf8(token).map_err(|_| invalid_syntax(0))?;
    let value = text
        .parse::<f64>()
        .map_err(|_| DxfAsciiNumericIssue::OutOfRange)?;
    if !value.is_finite() || (value == 0.0 && mantissa_has_nonzero_digit(token)) {
        return Err(DxfAsciiNumericIssue::OutOfRange);
    }
    Ok(DxfDouble::from_f64(value))
}

fn mantissa_has_nonzero_digit(token: &[u8]) -> bool {
    token
        .iter()
        .copied()
        .take_while(|byte| !matches!(byte, b'e' | b'E'))
        .any(|byte| matches!(byte, b'1'..=b'9'))
}

fn validate_double_syntax(token: &[u8]) -> Result<(), DxfAsciiNumericIssue> {
    if token.is_empty() {
        return Err(DxfAsciiNumericIssue::Empty);
    }
    let mut index = usize::from(matches!(token[0], b'+' | b'-'));
    let mut mantissa_digits = 0_usize;
    while matches!(token.get(index), Some(byte) if byte.is_ascii_digit()) {
        mantissa_digits += 1;
        index += 1;
    }
    if token.get(index) == Some(&b'.') {
        index += 1;
        while matches!(token.get(index), Some(byte) if byte.is_ascii_digit()) {
            mantissa_digits += 1;
            index += 1;
        }
    }
    if mantissa_digits == 0 {
        return Err(invalid_syntax(index));
    }
    if matches!(token.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(token.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while matches!(token.get(index), Some(byte) if byte.is_ascii_digit()) {
            index += 1;
        }
        if index == exponent_start {
            return Err(invalid_syntax(index));
        }
    }
    if index != token.len() {
        return Err(invalid_syntax(index));
    }
    Ok(())
}

fn field_provenance(source_id: DxfSourceId, spec: FieldSpec) -> DxfSemanticFieldProvenance {
    DxfSemanticFieldProvenance::new(source_id, HEADER_NAMESPACE, spec.id)
}

fn variable_provenance(
    document: DxfRawDocumentView<'_>,
    variable: DxfHeaderVariable,
) -> Result<DxfRawValueProvenance, DxfError> {
    if variable.value_groups().is_empty() {
        return marker_provenance(variable);
    }
    let group = document
        .group(variable.value_groups().start())
        .ok_or_else(invalid_internal_data)?;
    group_provenance(group)
}

fn marker_provenance(variable: DxfHeaderVariable) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(variable.marker_occurrence(), variable.name_span())
        .ok_or_else(invalid_internal_data)
}

fn group_provenance(group: DxfRawGroup) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
        .ok_or_else(invalid_internal_data)
}

fn invalid_syntax(offset: usize) -> DxfAsciiNumericIssue {
    match u64::try_from(offset) {
        Ok(token_offset) => DxfAsciiNumericIssue::InvalidSyntax { token_offset },
        Err(_) => DxfAsciiNumericIssue::OutOfRange,
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
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

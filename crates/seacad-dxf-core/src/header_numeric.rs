//! Source-anchored numeric HEADER semantics over the shared raw document API.

use std::{io, num::NonZeroU64};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfGroupCode, DxfHeaderSchemaDirectory, DxfHeaderVariable, DxfHeaderVariableLookupState,
    DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    ascii_group::trim_horizontal_ascii,
    generated::header_schema::{DxfSchemaStorageKind, HEADER_FIELDS},
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
    MultipleValueGroups { group_count: NonZeroU64 },
    MultipleVariables { occurrence_count: NonZeroU64 },
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
        let directory = document.resolve_header_schema(cancellation)?;
        if directory.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: directory.source_id(),
            });
        }
        let view = Self {
            source_id: document.source_id(),
            acad_maintenance_version: int16_field(
                document,
                &directory,
                FieldSpec::new(
                    "acadmaintver",
                    "$ACADMAINTVER",
                    70,
                    DxfSchemaStorageKind::Int16,
                ),
                cancellation,
            )?,
            angle_base: double_field(
                document,
                &directory,
                FieldSpec::new("angbase", "$ANGBASE", 50, DxfSchemaStorageKind::Double),
                cancellation,
            )?,
            angle_direction: int16_field(
                document,
                &directory,
                FieldSpec::new("angdir", "$ANGDIR", 70, DxfSchemaStorageKind::Int16),
                cancellation,
            )?,
            attribute_mode: int16_field(
                document,
                &directory,
                FieldSpec::new("attmode", "$ATTMODE", 70, DxfSchemaStorageKind::Int16),
                cancellation,
            )?,
            angular_units: int16_field(
                document,
                &directory,
                FieldSpec::new("aunits", "$AUNITS", 70, DxfSchemaStorageKind::Int16),
                cancellation,
            )?,
            angular_precision: int16_field(
                document,
                &directory,
                FieldSpec::new("auprec", "$AUPREC", 70, DxfSchemaStorageKind::Int16),
                cancellation,
            )?,
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
    /// Lazily resolves and reads the reviewed numeric HEADER schema slice.
    pub fn header_numeric_view(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericView, DxfError> {
        DxfHeaderNumericView::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn header_numeric_view(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderNumericView, DxfError> {
        DxfRawDocumentView::from(self).header_numeric_view(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
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
    dxf_name: &'static str,
    group_code: i16,
    storage: DxfSchemaStorageKind,
}

impl FieldSpec {
    const fn new(
        id: &'static str,
        dxf_name: &'static str,
        group_code: i16,
        storage: DxfSchemaStorageKind,
    ) -> Self {
        Self {
            id,
            dxf_name,
            group_code,
            storage,
        }
    }
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
    directory: &DxfHeaderSchemaDirectory,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSemanticValue<i16, DxfHeaderNumericIssue>, DxfError> {
    let field = field_provenance(document.source_id(), spec)?;
    match resolve_group(document, directory, spec, cancellation)? {
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
    directory: &DxfHeaderSchemaDirectory,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>, DxfError> {
    let field = field_provenance(document.source_id(), spec)?;
    match resolve_group(document, directory, spec, cancellation)? {
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
    directory: &DxfHeaderSchemaDirectory,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<ResolvedGroup, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let matched = directory.entry(spec.id).ok_or_else(invalid_internal_data)?;
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
            resolve_unique_variable(document, variable, spec.group_code)
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

fn field_provenance(
    source_id: DxfSourceId,
    spec: FieldSpec,
) -> Result<DxfSemanticFieldProvenance, DxfError> {
    let field = HEADER_FIELDS
        .iter()
        .find(|field| {
            field.id == spec.id
                && field.dxf_name == spec.dxf_name
                && field.group_codes == [spec.group_code]
                && field.storage == spec.storage
        })
        .ok_or_else(invalid_internal_data)?;
    Ok(DxfSemanticFieldProvenance::new(
        source_id,
        HEADER_NAMESPACE,
        field.id,
    ))
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

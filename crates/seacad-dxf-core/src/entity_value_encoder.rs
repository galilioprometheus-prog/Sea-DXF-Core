//! Canonical ASCII/Binary encoding for typed common-entity field values.

use std::{fmt, fmt::Write as _, io};

use crate::{
    DxfAcadVersion, DxfBinaryGroupCodeEncoding, DxfCancellationToken, DxfDouble,
    DxfEntityFieldDescriptor, DxfEntityFieldWireType, DxfError, DxfHandle, DxfIoOperation,
    DxfRawDocumentFormat, DxfResource, DxfResourceProfile,
};

pub const DXF_ENTITY_BINARY_CHUNK_MAX_BYTES: u64 = 128;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityEditValueKind {
    BinaryChunk,
    Double,
    ExactRawText,
    Handle,
    Int16,
    Int32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityEditValue<'a> {
    BinaryChunk(&'a [u8]),
    Double(DxfDouble),
    ExactRawText(&'a [u8]),
    Handle(DxfHandle),
    Int16(i16),
    Int32(i32),
}

impl DxfEntityEditValue<'_> {
    #[must_use]
    pub const fn kind(self) -> DxfEntityEditValueKind {
        match self {
            Self::BinaryChunk(_) => DxfEntityEditValueKind::BinaryChunk,
            Self::Double(_) => DxfEntityEditValueKind::Double,
            Self::ExactRawText(_) => DxfEntityEditValueKind::ExactRawText,
            Self::Handle(_) => DxfEntityEditValueKind::Handle,
            Self::Int16(_) => DxfEntityEditValueKind::Int16,
            Self::Int32(_) => DxfEntityEditValueKind::Int32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityGroupEncodeIssue {
    WireTypeMismatch {
        expected: DxfEntityFieldWireType,
        observed: DxfEntityEditValueKind,
    },
    NonFiniteDouble(DxfDouble),
    ForbiddenTextByte {
        offset: u64,
        byte: u8,
    },
    BinaryChunkTooLong {
        limit: u64,
        observed: u64,
    },
    GroupCodeUnavailableInDialect {
        group_code: i16,
        version: DxfAcadVersion,
    },
}

/// One complete encoded group. `Debug` intentionally omits payload bytes.
pub struct DxfEncodedEntityGroup {
    descriptor: DxfEntityFieldDescriptor,
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    bytes: Box<[u8]>,
}

impl DxfEncodedEntityGroup {
    #[must_use]
    pub const fn descriptor(&self) -> DxfEntityFieldDescriptor {
        self.descriptor
    }

    #[must_use]
    pub const fn format(&self) -> DxfRawDocumentFormat {
        self.format
    }

    #[must_use]
    pub const fn version(&self) -> DxfAcadVersion {
        self.version
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for DxfEncodedEntityGroup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEncodedEntityGroup")
            .field("field", &self.descriptor.field())
            .field("group_code", &self.descriptor.group_code())
            .field("format", &self.format)
            .field("version", &self.version)
            .field("byte_count", &self.bytes.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityGroupEncoder {
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    profile: DxfResourceProfile,
}

impl DxfEntityGroupEncoder {
    #[must_use]
    pub const fn new(
        format: DxfRawDocumentFormat,
        version: DxfAcadVersion,
        profile: DxfResourceProfile,
    ) -> Self {
        Self {
            format,
            version,
            profile,
        }
    }

    pub fn encode(
        self,
        descriptor: DxfEntityFieldDescriptor,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEncodedEntityGroup, DxfEntityGroupEncodeIssue>, DxfError> {
        ensure_not_cancelled(cancellation)?;
        if let Err(issue) = validate_value(descriptor, value) {
            return Ok(Err(issue));
        }
        let encoded = match self.format {
            DxfRawDocumentFormat::Ascii => self.encode_ascii(descriptor, value, cancellation),
            DxfRawDocumentFormat::Binary => self.encode_binary(descriptor, value, cancellation),
        }?;
        ensure_not_cancelled(cancellation)?;
        Ok(encoded.map(|bytes| DxfEncodedEntityGroup {
            descriptor,
            format: self.format,
            version: self.version,
            bytes: bytes.into_boxed_slice(),
        }))
    }

    fn encode_ascii(
        self,
        descriptor: DxfEntityFieldDescriptor,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<Vec<u8>, DxfEntityGroupEncodeIssue>, DxfError> {
        let mut fixed = FixedAscii::default();
        let payload = match value {
            DxfEntityEditValue::Double(value) => {
                write!(&mut fixed, "{}", value.to_f64()).map_err(|_| invalid_internal_data())?;
                Payload::Bytes(fixed.bytes())
            }
            DxfEntityEditValue::Int16(value) => {
                write!(&mut fixed, "{value}").map_err(|_| invalid_internal_data())?;
                Payload::Bytes(fixed.bytes())
            }
            DxfEntityEditValue::Int32(value) => {
                write!(&mut fixed, "{value}").map_err(|_| invalid_internal_data())?;
                Payload::Bytes(fixed.bytes())
            }
            DxfEntityEditValue::Handle(value) => {
                write!(&mut fixed, "{:X}", value.value()).map_err(|_| invalid_internal_data())?;
                Payload::Bytes(fixed.bytes())
            }
            DxfEntityEditValue::ExactRawText(value) => Payload::Bytes(value),
            DxfEntityEditValue::BinaryChunk(value) => Payload::UpperHex(value),
        };
        let payload_len = payload.encoded_len()?;
        enforce_value_limit(self.profile, payload_len)?;
        let mut code = [0_u8; 6];
        let code = encode_i16(descriptor.group_code(), &mut code);
        let capacity = checked_sum(&[code.len(), 1, payload_len, 1])?;
        let mut bytes = reserve_bytes(capacity)?;
        bytes.extend_from_slice(code);
        bytes.push(b'\n');
        payload.append_to(&mut bytes);
        bytes.push(b'\n');
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(bytes))
    }

    fn encode_binary(
        self,
        descriptor: DxfEntityFieldDescriptor,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<Vec<u8>, DxfEntityGroupEncodeIssue>, DxfError> {
        let group_code = descriptor.group_code();
        let mut code = [0_u8; 3];
        let code_len = match binary_group_code(self.version, group_code, &mut code) {
            Ok(len) => len,
            Err(issue) => return Ok(Err(issue)),
        };
        let payload_len = binary_payload_len(value)?;
        enforce_value_limit(self.profile, payload_len)?;
        let capacity = checked_sum(&[code_len, payload_len])?;
        let mut bytes = reserve_bytes(capacity)?;
        bytes.extend_from_slice(&code[..code_len]);
        append_binary_value(&mut bytes, value)?;
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(bytes))
    }
}

enum Payload<'a> {
    Bytes(&'a [u8]),
    UpperHex(&'a [u8]),
}

impl Payload<'_> {
    fn encoded_len(&self) -> Result<usize, DxfError> {
        match self {
            Self::Bytes(bytes) => Ok(bytes.len()),
            Self::UpperHex(bytes) => bytes.len().checked_mul(2).ok_or_else(offset_overflow),
        }
    }

    fn append_to(&self, destination: &mut Vec<u8>) {
        match self {
            Self::Bytes(bytes) => destination.extend_from_slice(bytes),
            Self::UpperHex(bytes) => {
                for byte in *bytes {
                    destination.push(hex_digit(byte >> 4));
                    destination.push(hex_digit(byte & 0x0f));
                }
            }
        }
    }
}

struct FixedAscii {
    bytes: [u8; 64],
    len: usize,
}

impl Default for FixedAscii {
    fn default() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
        }
    }
}

impl FixedAscii {
    fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl fmt::Write for FixedAscii {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn validate_value(
    descriptor: DxfEntityFieldDescriptor,
    value: DxfEntityEditValue<'_>,
) -> Result<(), DxfEntityGroupEncodeIssue> {
    let matches = matches!(
        (descriptor.wire_type(), value),
        (
            DxfEntityFieldWireType::BinaryChunk,
            DxfEntityEditValue::BinaryChunk(_)
        ) | (
            DxfEntityFieldWireType::Double,
            DxfEntityEditValue::Double(_)
        ) | (
            DxfEntityFieldWireType::ExactText,
            DxfEntityEditValue::ExactRawText(_)
        ) | (
            DxfEntityFieldWireType::Handle,
            DxfEntityEditValue::Handle(_)
        ) | (DxfEntityFieldWireType::Int16, DxfEntityEditValue::Int16(_))
            | (DxfEntityFieldWireType::Int32, DxfEntityEditValue::Int32(_))
    );
    if !matches {
        return Err(DxfEntityGroupEncodeIssue::WireTypeMismatch {
            expected: descriptor.wire_type(),
            observed: value.kind(),
        });
    }
    match value {
        DxfEntityEditValue::Double(value) if !value.is_finite() => {
            Err(DxfEntityGroupEncodeIssue::NonFiniteDouble(value))
        }
        DxfEntityEditValue::ExactRawText(bytes) => validate_text(bytes),
        DxfEntityEditValue::BinaryChunk(bytes)
            if bytes.len() as u64 > DXF_ENTITY_BINARY_CHUNK_MAX_BYTES =>
        {
            Err(DxfEntityGroupEncodeIssue::BinaryChunkTooLong {
                limit: DXF_ENTITY_BINARY_CHUNK_MAX_BYTES,
                observed: bytes.len() as u64,
            })
        }
        _ => Ok(()),
    }
}

fn validate_text(bytes: &[u8]) -> Result<(), DxfEntityGroupEncodeIssue> {
    for (offset, byte) in bytes.iter().copied().enumerate() {
        if matches!(byte, 0 | b'\r' | b'\n') {
            return Err(DxfEntityGroupEncodeIssue::ForbiddenTextByte {
                offset: offset as u64,
                byte,
            });
        }
    }
    Ok(())
}

fn binary_group_code(
    version: DxfAcadVersion,
    group_code: i16,
    destination: &mut [u8; 3],
) -> Result<usize, DxfEntityGroupEncodeIssue> {
    match version.binary_group_code_encoding() {
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape
            if (0..=254).contains(&group_code) =>
        {
            destination[0] = group_code as u8;
            Ok(1)
        }
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape
            if (1000..=1071).contains(&group_code) =>
        {
            destination[0] = u8::MAX;
            destination[1..].copy_from_slice(&group_code.to_le_bytes());
            Ok(3)
        }
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape => {
            Err(DxfEntityGroupEncodeIssue::GroupCodeUnavailableInDialect {
                group_code,
                version,
            })
        }
        DxfBinaryGroupCodeEncoding::TwoByteLittleEndian => {
            destination[..2].copy_from_slice(&group_code.to_le_bytes());
            Ok(2)
        }
    }
}

fn binary_payload_len(value: DxfEntityEditValue<'_>) -> Result<usize, DxfError> {
    match value {
        DxfEntityEditValue::Double(_) => Ok(8),
        DxfEntityEditValue::Int16(_) => Ok(2),
        DxfEntityEditValue::Int32(_) => Ok(4),
        DxfEntityEditValue::Handle(handle) => Ok(handle_hex_len(handle) + 1),
        DxfEntityEditValue::ExactRawText(bytes) | DxfEntityEditValue::BinaryChunk(bytes) => {
            bytes.len().checked_add(1).ok_or_else(offset_overflow)
        }
    }
}

fn append_binary_value(
    destination: &mut Vec<u8>,
    value: DxfEntityEditValue<'_>,
) -> Result<(), DxfError> {
    match value {
        DxfEntityEditValue::Double(value) => {
            destination.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        DxfEntityEditValue::Int16(value) => destination.extend_from_slice(&value.to_le_bytes()),
        DxfEntityEditValue::Int32(value) => destination.extend_from_slice(&value.to_le_bytes()),
        DxfEntityEditValue::ExactRawText(bytes) => {
            destination.extend_from_slice(bytes);
            destination.push(0);
        }
        DxfEntityEditValue::BinaryChunk(bytes) => {
            let len = u8::try_from(bytes.len()).map_err(|_| invalid_internal_data())?;
            destination.push(len);
            destination.extend_from_slice(bytes);
        }
        DxfEntityEditValue::Handle(handle) => {
            let mut encoded = FixedAscii::default();
            write!(&mut encoded, "{:X}", handle.value()).map_err(|_| invalid_internal_data())?;
            destination.extend_from_slice(encoded.bytes());
            destination.push(0);
        }
    }
    Ok(())
}

fn handle_hex_len(handle: DxfHandle) -> usize {
    let mut value = handle.value();
    let mut len = 1_usize;
    while value >= 16 {
        len += 1;
        value >>= 4;
    }
    len
}

fn encode_i16(value: i16, destination: &mut [u8; 6]) -> &[u8] {
    let negative = value < 0;
    let mut magnitude = i32::from(value).unsigned_abs();
    let mut start = destination.len();
    loop {
        start -= 1;
        destination[start] = b'0' + (magnitude % 10) as u8;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if negative {
        start -= 1;
        destination[start] = b'-';
    }
    &destination[start..]
}

const fn hex_digit(value: u8) -> u8 {
    if value < 10 {
        b'0' + value
    } else {
        b'A' + (value - 10)
    }
}

fn checked_sum(values: &[usize]) -> Result<usize, DxfError> {
    values.iter().try_fold(0_usize, |total, value| {
        total.checked_add(*value).ok_or_else(offset_overflow)
    })
}

fn reserve_bytes(capacity: usize) -> Result<Vec<u8>, DxfError> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| out_of_memory())?;
    Ok(bytes)
}

fn enforce_value_limit(profile: DxfResourceProfile, observed: usize) -> Result<(), DxfError> {
    let observed = u64::try_from(observed).map_err(|_| offset_overflow())?;
    let limit = profile.limits().max_value_bytes();
    if observed > limit {
        Err(DxfError::resource_limit(
            DxfResource::ValueBytes,
            limit,
            observed,
        ))
    } else {
        Ok(())
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn offset_overflow() -> DxfError {
    DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Write, &io::Error::from(kind))
}

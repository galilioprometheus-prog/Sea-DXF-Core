use std::{fmt, io};

use crate::{
    binary_wire::DxfBinaryGroupCodeEncoding,
    diagnostic::ByteSpan,
    dialect::{DxfAcadVersion, DxfAcadVersionState},
    source_id::DxfSourceId,
};

/// Stable machine-readable code for a fatal DXF operation error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfErrorCode(&'static str);

impl DxfErrorCode {
    pub const IO: Self = Self("DXF-E0001");
    pub const CANCELLED: Self = Self("DXF-E0002");
    pub const SOURCE_LIMIT_EXCEEDED: Self = Self("DXF-E0101");
    pub const RECORD_LIMIT_EXCEEDED: Self = Self("DXF-E0102");
    pub const VALUE_LIMIT_EXCEEDED: Self = Self("DXF-E0103");
    pub const OFFSET_OVERFLOW: Self = Self("DXF-E0105");
    pub const INVALID_ASCII_GROUP_CODE: Self = Self("DXF-E0201");
    pub const MISSING_ASCII_GROUP_VALUE: Self = Self("DXF-E0202");
    pub const MISSING_ASCII_EOF: Self = Self("DXF-E0203");
    pub const TRAILING_ASCII_DATA: Self = Self("DXF-E0204");
    pub const INVALID_BINARY_SENTINEL: Self = Self("DXF-E0210");
    pub const TRUNCATED_BINARY_GROUP_CODE: Self = Self("DXF-E0211");
    pub const INVALID_BINARY_GROUP_CODE: Self = Self("DXF-E0212");
    pub const UNSUPPORTED_BINARY_GROUP_CODE: Self = Self("DXF-E0213");
    pub const TRUNCATED_BINARY_VALUE: Self = Self("DXF-E0214");
    pub const UNTERMINATED_BINARY_STRING: Self = Self("DXF-E0215");
    pub const INVALID_BINARY_OPENING: Self = Self("DXF-E0216");
    pub const BINARY_ACADVER_UNAVAILABLE: Self = Self("DXF-E0217");
    pub const BINARY_ENCODING_DIALECT_MISMATCH: Self = Self("DXF-E0218");
    pub const SOURCE_IDENTITY_MISMATCH: Self = Self("DXF-E0301");
    pub const VERBATIM_OUTPUT_LENGTH_MISMATCH: Self = Self("DXF-E0302");
    pub const VERBATIM_OUTPUT_IDENTITY_MISMATCH: Self = Self("DXF-E0303");

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for DxfErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfIoOperation {
    Create,
    Flush,
    Metadata,
    Open,
    Read,
    Remove,
    Seek,
    Sync,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfResource {
    SourceBytes,
    Records,
    ValueBytes,
}

/// Fatal failure with a stable code and no source path disclosure.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfError {
    Io {
        operation: DxfIoOperation,
        kind: io::ErrorKind,
        raw_os_error: Option<i32>,
    },
    Cancelled,
    ResourceLimitExceeded {
        resource: DxfResource,
        limit: u64,
        observed: u64,
    },
    OffsetOverflow {
        offset: u64,
        requested: u64,
    },
    InvalidAsciiGroupCode {
        span: ByteSpan,
    },
    MissingAsciiGroupValue {
        group_code_span: ByteSpan,
    },
    MissingAsciiEof {
        at_offset: u64,
    },
    TrailingAsciiData {
        span: ByteSpan,
    },
    InvalidBinarySentinel {
        span: ByteSpan,
    },
    TruncatedBinaryGroupCode {
        span: ByteSpan,
        expected_bytes: u8,
    },
    InvalidBinaryGroupCode {
        span: ByteSpan,
    },
    UnsupportedBinaryGroupCode {
        group_code: i16,
        span: ByteSpan,
    },
    TruncatedBinaryValue {
        group_code: i16,
        span: ByteSpan,
        expected_value_bytes: u64,
    },
    UnterminatedBinaryString {
        group_code: i16,
        span: ByteSpan,
    },
    InvalidBinaryOpening {
        span: ByteSpan,
    },
    BinaryAcadVersionUnavailable {
        state: DxfAcadVersionState,
        span: Option<ByteSpan>,
    },
    BinaryEncodingDialectMismatch {
        encoding: DxfBinaryGroupCodeEncoding,
        declared_version: DxfAcadVersion,
        span: ByteSpan,
    },
    SourceIdentityMismatch {
        expected: DxfSourceId,
        observed: DxfSourceId,
    },
    VerbatimOutputLengthMismatch {
        expected: u64,
        observed: u64,
    },
    VerbatimOutputIdentityMismatch {
        expected: DxfSourceId,
        observed: DxfSourceId,
    },
}

impl DxfError {
    #[must_use]
    pub fn from_io(operation: DxfIoOperation, error: &io::Error) -> Self {
        Self::Io {
            operation,
            kind: error.kind(),
            raw_os_error: error.raw_os_error(),
        }
    }

    #[must_use]
    pub const fn resource_limit(resource: DxfResource, limit: u64, observed: u64) -> Self {
        Self::ResourceLimitExceeded {
            resource,
            limit,
            observed,
        }
    }

    #[must_use]
    pub const fn code(&self) -> DxfErrorCode {
        match self {
            Self::Io { .. } => DxfErrorCode::IO,
            Self::Cancelled => DxfErrorCode::CANCELLED,
            Self::ResourceLimitExceeded { resource, .. } => match resource {
                DxfResource::SourceBytes => DxfErrorCode::SOURCE_LIMIT_EXCEEDED,
                DxfResource::Records => DxfErrorCode::RECORD_LIMIT_EXCEEDED,
                DxfResource::ValueBytes => DxfErrorCode::VALUE_LIMIT_EXCEEDED,
            },
            Self::OffsetOverflow { .. } => DxfErrorCode::OFFSET_OVERFLOW,
            Self::InvalidAsciiGroupCode { .. } => DxfErrorCode::INVALID_ASCII_GROUP_CODE,
            Self::MissingAsciiGroupValue { .. } => DxfErrorCode::MISSING_ASCII_GROUP_VALUE,
            Self::MissingAsciiEof { .. } => DxfErrorCode::MISSING_ASCII_EOF,
            Self::TrailingAsciiData { .. } => DxfErrorCode::TRAILING_ASCII_DATA,
            Self::InvalidBinarySentinel { .. } => DxfErrorCode::INVALID_BINARY_SENTINEL,
            Self::TruncatedBinaryGroupCode { .. } => DxfErrorCode::TRUNCATED_BINARY_GROUP_CODE,
            Self::InvalidBinaryGroupCode { .. } => DxfErrorCode::INVALID_BINARY_GROUP_CODE,
            Self::UnsupportedBinaryGroupCode { .. } => DxfErrorCode::UNSUPPORTED_BINARY_GROUP_CODE,
            Self::TruncatedBinaryValue { .. } => DxfErrorCode::TRUNCATED_BINARY_VALUE,
            Self::UnterminatedBinaryString { .. } => DxfErrorCode::UNTERMINATED_BINARY_STRING,
            Self::InvalidBinaryOpening { .. } => DxfErrorCode::INVALID_BINARY_OPENING,
            Self::BinaryAcadVersionUnavailable { .. } => DxfErrorCode::BINARY_ACADVER_UNAVAILABLE,
            Self::BinaryEncodingDialectMismatch { .. } => {
                DxfErrorCode::BINARY_ENCODING_DIALECT_MISMATCH
            }
            Self::SourceIdentityMismatch { .. } => DxfErrorCode::SOURCE_IDENTITY_MISMATCH,
            Self::VerbatimOutputLengthMismatch { .. } => {
                DxfErrorCode::VERBATIM_OUTPUT_LENGTH_MISMATCH
            }
            Self::VerbatimOutputIdentityMismatch { .. } => {
                DxfErrorCode::VERBATIM_OUTPUT_IDENTITY_MISMATCH
            }
        }
    }
}

impl fmt::Display for DxfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                kind,
                raw_os_error,
            } => write!(
                formatter,
                "{}: {operation:?} failed ({kind:?}, OS code {raw_os_error:?})",
                self.code()
            ),
            Self::Cancelled => write!(formatter, "{}: operation cancelled", self.code()),
            Self::ResourceLimitExceeded {
                resource,
                limit,
                observed,
            } => write!(
                formatter,
                "{}: {resource:?} limit {limit} exceeded by observed value {observed}",
                self.code()
            ),
            Self::OffsetOverflow { offset, requested } => write!(
                formatter,
                "{}: byte offset {offset} plus request {requested} overflows u64",
                self.code()
            ),
            Self::InvalidAsciiGroupCode { span } => write!(
                formatter,
                "{}: invalid ASCII group code at byte span [{}, {})",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::MissingAsciiGroupValue { group_code_span } => write!(
                formatter,
                "{}: ASCII group at byte span [{}, {}) has no value line",
                self.code(),
                group_code_span.start(),
                group_code_span.end()
            ),
            Self::MissingAsciiEof { at_offset } => write!(
                formatter,
                "{}: ASCII document has no terminal 0/EOF marker at byte offset {at_offset}",
                self.code()
            ),
            Self::TrailingAsciiData { span } => write!(
                formatter,
                "{}: strict ASCII document has trailing data at byte span [{}, {})",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::InvalidBinarySentinel { span } => write!(
                formatter,
                "{}: invalid Binary DXF sentinel at byte span [{}, {})",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::TruncatedBinaryGroupCode {
                span,
                expected_bytes,
            } => write!(
                formatter,
                "{}: Binary DXF group code at byte span [{}, {}) requires {expected_bytes} bytes",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::InvalidBinaryGroupCode { span } => write!(
                formatter,
                "{}: invalid Binary DXF group code at byte span [{}, {})",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::UnsupportedBinaryGroupCode { group_code, span } => write!(
                formatter,
                "{}: Binary DXF group code {group_code} at byte span [{}, {}) has no documented wire family",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::TruncatedBinaryValue {
                group_code,
                span,
                expected_value_bytes,
            } => write!(
                formatter,
                "{}: Binary DXF group {group_code} value at byte span [{}, {}) requires {expected_value_bytes} wire bytes",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::UnterminatedBinaryString { group_code, span } => write!(
                formatter,
                "{}: Binary DXF group {group_code} string at byte span [{}, {}) has no NUL terminator",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::InvalidBinaryOpening { span } => write!(
                formatter,
                "{}: Binary DXF has no canonical opening 0/SECTION record at byte span [{}, {})",
                self.code(),
                span.start(),
                span.end()
            ),
            Self::BinaryAcadVersionUnavailable { state, span } => write!(
                formatter,
                "{}: Binary DXF requires one exact supported HEADER $ACADVER (state {state:?}, span {span:?})",
                self.code()
            ),
            Self::BinaryEncodingDialectMismatch {
                encoding,
                declared_version,
                span,
            } => write!(
                formatter,
                "{}: Binary DXF encoding {encoding:?} disagrees with declared {} at byte span [{}, {})",
                self.code(),
                declared_version.code(),
                span.start(),
                span.end()
            ),
            Self::SourceIdentityMismatch { expected, observed } => write!(
                formatter,
                "{}: source identity changed from {expected} to {observed}",
                self.code()
            ),
            Self::VerbatimOutputLengthMismatch { expected, observed } => write!(
                formatter,
                "{}: verbatim output length {observed} differs from expected {expected}",
                self.code()
            ),
            Self::VerbatimOutputIdentityMismatch { expected, observed } => write!(
                formatter,
                "{}: verbatim output identity {observed} differs from expected {expected}",
                self.code()
            ),
        }
    }
}

impl std::error::Error for DxfError {}

#[cfg(test)]
mod tests {
    use std::io;

    use super::{ByteSpan, DxfError, DxfErrorCode, DxfIoOperation, DxfResource, DxfSourceId};
    use crate::{DxfAcadVersion, DxfAcadVersionState, DxfBinaryGroupCodeEncoding};

    #[test]
    fn every_fatal_variant_has_the_stable_code() -> Result<(), io::Error> {
        let expected_id = DxfSourceId::from_sha256([1_u8; DxfSourceId::BYTE_LEN]);
        let observed_id = DxfSourceId::from_sha256([2_u8; DxfSourceId::BYTE_LEN]);
        let cases = [
            (
                DxfError::from_io(DxfIoOperation::Read, &io::Error::from(io::ErrorKind::Other)),
                DxfErrorCode::IO,
                "DXF-E0001",
            ),
            (DxfError::Cancelled, DxfErrorCode::CANCELLED, "DXF-E0002"),
            (
                DxfError::resource_limit(DxfResource::SourceBytes, 1, 2),
                DxfErrorCode::SOURCE_LIMIT_EXCEEDED,
                "DXF-E0101",
            ),
            (
                DxfError::resource_limit(DxfResource::Records, 1, 2),
                DxfErrorCode::RECORD_LIMIT_EXCEEDED,
                "DXF-E0102",
            ),
            (
                DxfError::resource_limit(DxfResource::ValueBytes, 1, 2),
                DxfErrorCode::VALUE_LIMIT_EXCEEDED,
                "DXF-E0103",
            ),
            (
                DxfError::OffsetOverflow {
                    offset: u64::MAX,
                    requested: 1,
                },
                DxfErrorCode::OFFSET_OVERFLOW,
                "DXF-E0105",
            ),
            (
                DxfError::InvalidAsciiGroupCode {
                    span: ByteSpan::new(2, 5).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::INVALID_ASCII_GROUP_CODE,
                "DXF-E0201",
            ),
            (
                DxfError::MissingAsciiGroupValue {
                    group_code_span: ByteSpan::new(7, 10)
                        .ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::MISSING_ASCII_GROUP_VALUE,
                "DXF-E0202",
            ),
            (
                DxfError::MissingAsciiEof { at_offset: 12 },
                DxfErrorCode::MISSING_ASCII_EOF,
                "DXF-E0203",
            ),
            (
                DxfError::TrailingAsciiData {
                    span: ByteSpan::new(12, 18).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::TRAILING_ASCII_DATA,
                "DXF-E0204",
            ),
            (
                DxfError::InvalidBinarySentinel {
                    span: ByteSpan::new(0, 22).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::INVALID_BINARY_SENTINEL,
                "DXF-E0210",
            ),
            (
                DxfError::TruncatedBinaryGroupCode {
                    span: ByteSpan::new(20, 21).ok_or(io::Error::other("invalid test span"))?,
                    expected_bytes: 2,
                },
                DxfErrorCode::TRUNCATED_BINARY_GROUP_CODE,
                "DXF-E0211",
            ),
            (
                DxfError::InvalidBinaryGroupCode {
                    span: ByteSpan::new(22, 24).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::INVALID_BINARY_GROUP_CODE,
                "DXF-E0212",
            ),
            (
                DxfError::UnsupportedBinaryGroupCode {
                    group_code: 80,
                    span: ByteSpan::new(24, 26).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::UNSUPPORTED_BINARY_GROUP_CODE,
                "DXF-E0213",
            ),
            (
                DxfError::TruncatedBinaryValue {
                    group_code: 10,
                    span: ByteSpan::new(26, 30).ok_or(io::Error::other("invalid test span"))?,
                    expected_value_bytes: 8,
                },
                DxfErrorCode::TRUNCATED_BINARY_VALUE,
                "DXF-E0214",
            ),
            (
                DxfError::UnterminatedBinaryString {
                    group_code: 1,
                    span: ByteSpan::new(30, 35).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::UNTERMINATED_BINARY_STRING,
                "DXF-E0215",
            ),
            (
                DxfError::InvalidBinaryOpening {
                    span: ByteSpan::new(22, 31).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::INVALID_BINARY_OPENING,
                "DXF-E0216",
            ),
            (
                DxfError::BinaryAcadVersionUnavailable {
                    state: DxfAcadVersionState::Absent,
                    span: None,
                },
                DxfErrorCode::BINARY_ACADVER_UNAVAILABLE,
                "DXF-E0217",
            ),
            (
                DxfError::BinaryEncodingDialectMismatch {
                    encoding: DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape,
                    declared_version: DxfAcadVersion::Ac1015,
                    span: ByteSpan::new(40, 46).ok_or(io::Error::other("invalid test span"))?,
                },
                DxfErrorCode::BINARY_ENCODING_DIALECT_MISMATCH,
                "DXF-E0218",
            ),
            (
                DxfError::SourceIdentityMismatch {
                    expected: expected_id,
                    observed: observed_id,
                },
                DxfErrorCode::SOURCE_IDENTITY_MISMATCH,
                "DXF-E0301",
            ),
            (
                DxfError::VerbatimOutputLengthMismatch {
                    expected: 12,
                    observed: 13,
                },
                DxfErrorCode::VERBATIM_OUTPUT_LENGTH_MISMATCH,
                "DXF-E0302",
            ),
            (
                DxfError::VerbatimOutputIdentityMismatch {
                    expected: expected_id,
                    observed: observed_id,
                },
                DxfErrorCode::VERBATIM_OUTPUT_IDENTITY_MISMATCH,
                "DXF-E0303",
            ),
        ];

        for (error, code, text) in cases {
            assert_eq!(error.code(), code);
            assert_eq!(error.code().as_str(), text);
            assert!(error.to_string().starts_with(text));
        }
        Ok(())
    }

    #[test]
    fn io_error_display_does_not_include_a_path() {
        let error = DxfError::from_io(
            DxfIoOperation::Open,
            &io::Error::new(io::ErrorKind::PermissionDenied, "D:/private/secret.dxf"),
        );
        assert!(!error.to_string().contains("secret.dxf"));
        assert!(error.to_string().contains("PermissionDenied"));
    }
}

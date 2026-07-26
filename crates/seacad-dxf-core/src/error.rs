use std::{fmt, io};

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
    Metadata,
    Open,
    Read,
    Seek,
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
        }
    }
}

impl std::error::Error for DxfError {}

#[cfg(test)]
mod tests {
    use std::io;

    use super::{DxfError, DxfErrorCode, DxfIoOperation, DxfResource};

    #[test]
    fn every_fatal_variant_has_the_stable_code() {
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
        ];

        for (error, code, text) in cases {
            assert_eq!(error.code(), code);
            assert_eq!(error.code().as_str(), text);
            assert!(error.to_string().starts_with(text));
        }
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

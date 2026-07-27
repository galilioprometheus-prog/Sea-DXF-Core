use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
    sync::Mutex,
};

use crate::{DxfError, DxfIoOperation, DxfResource, DxfResourceLimits, DxfResourceProfile};

/// Stable-content, stable-length, thread-safe random access to source bytes.
///
/// Implementors and callers must prevent byte changes for the lifetime of any
/// document borrowing the source. External mutation of an opened file violates
/// this snapshot precondition.
pub trait DxfByteSource: Send + Sync {
    /// Source length captured when this source was created.
    fn len(&self) -> u64;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads at most destination.len() bytes without changing a public cursor.
    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError>;

    /// Fills the destination or reports a path-redacted unexpected-EOF error.
    fn read_exact_at(&self, offset: u64, destination: &mut [u8]) -> Result<(), DxfError> {
        validate_request_range(offset, destination.len())?;
        let mut filled = 0_usize;

        while filled < destination.len() {
            let delta = usize_to_u64(filled)?;
            let read_offset = offset.checked_add(delta).ok_or(DxfError::OffsetOverflow {
                offset,
                requested: delta,
            })?;
            let remaining = destination.len() - filled;
            let read = self.read_at(read_offset, &mut destination[filled..])?;
            if read == 0 {
                return Err(io_error(DxfIoOperation::Read, io::ErrorKind::UnexpectedEof));
            }
            if read > remaining {
                return Err(io_error(DxfIoOperation::Read, io::ErrorKind::InvalidData));
            }
            filled += read;
        }

        Ok(())
    }
}

/// Zero-copy view over caller-owned immutable bytes.
pub struct DxfMemorySource<'a> {
    bytes: &'a [u8],
    len: u64,
}

impl<'a> DxfMemorySource<'a> {
    pub fn new(bytes: &'a [u8], profile: DxfResourceProfile) -> Result<Self, DxfError> {
        Self::with_limits(bytes, profile.limits())
    }

    fn with_limits(bytes: &'a [u8], limits: DxfResourceLimits) -> Result<Self, DxfError> {
        let len = usize_to_u64(bytes.len())?;
        enforce_source_limit(len, limits)?;
        Ok(Self { bytes, len })
    }
}

impl DxfByteSource for DxfMemorySource<'_> {
    fn len(&self) -> u64 {
        self.len
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        validate_request_range(offset, destination.len())?;
        if destination.is_empty() || offset >= self.len {
            return Ok(0);
        }

        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: 0,
        })?;
        let count = (self.bytes.len() - start).min(destination.len());
        destination[..count].copy_from_slice(&self.bytes[start..start + count]);
        Ok(count)
    }
}

/// File-backed source that never maps or loads the complete file.
pub struct DxfFileSource {
    file: Mutex<File>,
    len: u64,
}

impl DxfFileSource {
    pub fn open(path: impl AsRef<Path>, profile: DxfResourceProfile) -> Result<Self, DxfError> {
        let file =
            File::open(path).map_err(|error| DxfError::from_io(DxfIoOperation::Open, &error))?;
        Self::from_file(file, profile)
    }

    pub fn from_file(file: File, profile: DxfResourceProfile) -> Result<Self, DxfError> {
        Self::with_limits(file, profile.limits())
    }

    fn with_limits(file: File, limits: DxfResourceLimits) -> Result<Self, DxfError> {
        let metadata = file
            .metadata()
            .map_err(|error| DxfError::from_io(DxfIoOperation::Metadata, &error))?;
        let len = metadata.len();
        enforce_source_limit(len, limits)?;
        Ok(Self {
            file: Mutex::new(file),
            len,
        })
    }
}

impl DxfByteSource for DxfFileSource {
    fn len(&self) -> u64 {
        self.len
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        let requested = validate_request_range(offset, destination.len())?;
        if destination.is_empty() || offset >= self.len {
            return Ok(0);
        }

        let allowed_u64 = (self.len - offset).min(requested);
        let allowed = usize::try_from(allowed_u64).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: allowed_u64,
        })?;
        let mut file = match self.file.lock() {
            Ok(file) => file,
            Err(poisoned) => poisoned.into_inner(),
        };
        file.seek(SeekFrom::Start(offset))
            .map_err(|error| DxfError::from_io(DxfIoOperation::Seek, &error))?;
        file.read(&mut destination[..allowed])
            .map_err(|error| DxfError::from_io(DxfIoOperation::Read, &error))
    }
}

fn enforce_source_limit(len: u64, limits: DxfResourceLimits) -> Result<(), DxfError> {
    let limit = limits.max_source_bytes();
    if len > limit {
        Err(DxfError::resource_limit(
            DxfResource::SourceBytes,
            limit,
            len,
        ))
    } else {
        Ok(())
    }
}

fn validate_request_range(offset: u64, destination_len: usize) -> Result<u64, DxfError> {
    let requested = usize_to_u64(destination_len)?;
    offset
        .checked_add(requested)
        .ok_or(DxfError::OffsetOverflow { offset, requested })?;
    Ok(requested)
}

fn usize_to_u64(value: usize) -> Result<u64, DxfError> {
    u64::try_from(value).map_err(|_| DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    })
}

fn io_error(operation: DxfIoOperation, kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(operation, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        io,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{DxfByteSource, DxfFileSource, DxfMemorySource};
    use crate::{
        DxfError, DxfErrorCode, DxfIoOperation, DxfResource, DxfResourceLimits, DxfResourceProfile,
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TestFile {
        path: PathBuf,
    }

    impl TestFile {
        fn new(bytes: &[u8]) -> io::Result<Self> {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("seacad-source-{}-{id}.dxf", std::process::id()));
            fs::write(&path, bytes)?;
            Ok(Self { path })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    struct OneByteSource {
        bytes: &'static [u8],
    }

    impl DxfByteSource for OneByteSource {
        fn len(&self) -> u64 {
            self.bytes.len() as u64
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            if destination.is_empty() || offset >= self.len() {
                return Ok(0);
            }

            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            destination[0] = self.bytes[start];
            Ok(1)
        }
    }

    #[test]
    fn exact_read_retries_partial_reads() -> Result<(), DxfError> {
        let source = OneByteSource { bytes: b"abcd" };
        let mut destination = [0_u8; 4];
        source.read_exact_at(0, &mut destination)?;
        assert_eq!(&destination, b"abcd");
        Ok(())
    }

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn memory_source_is_bounded_and_reads_partial_ranges() -> Result<(), DxfError> {
        let tiny = DxfResourceLimits::test_with_max_source_bytes(4);
        let rejected = DxfMemorySource::with_limits(b"12345", tiny);
        assert!(matches!(
            rejected,
            Err(DxfError::ResourceLimitExceeded {
                resource: DxfResource::SourceBytes,
                limit: 4,
                observed: 5
            })
        ));

        let source = DxfMemorySource::with_limits(b"abcd", tiny)?;
        let mut destination = [0xff; 4];
        assert_eq!(source.len(), 4);
        assert_eq!(source.read_at(1, &mut destination)?, 3);
        assert_eq!(&destination[..3], b"bcd");
        assert_eq!(source.read_at(4, &mut destination)?, 0);
        let empty = DxfMemorySource::new(b"", DxfResourceProfile::Safe)?;
        assert!(empty.is_empty());
        Ok(())
    }

    #[test]
    fn exact_reads_report_eof_and_ranges_reject_overflow() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(b"abcd", DxfResourceProfile::Safe)?;
        let mut exact = [0_u8; 3];
        source.read_exact_at(1, &mut exact)?;
        assert_eq!(&exact, b"bcd");

        let mut too_long = [0_u8; 5];
        assert!(matches!(
            source.read_exact_at(0, &mut too_long),
            Err(DxfError::Io {
                operation: DxfIoOperation::Read,
                kind: io::ErrorKind::UnexpectedEof,
                ..
            })
        ));
        let mut overflow = [0_u8; 2];
        assert!(matches!(
            source.read_at(u64::MAX, &mut overflow),
            Err(DxfError::OffsetOverflow { .. })
        ));
        Ok(())
    }

    #[test]
    fn file_source_is_random_access_and_enforces_snapshot_limit() -> Result<(), Box<dyn Error>> {
        assert_send_sync::<DxfFileSource>();
        assert_send_sync::<DxfMemorySource<'static>>();

        let fixture = TestFile::new(b"0123456789")?;
        let source = DxfFileSource::open(fixture.path(), DxfResourceProfile::Safe)?;
        let mut first = [0_u8; 4];
        let mut second = [0_u8; 3];
        assert_eq!(source.len(), 10);
        assert_eq!(source.read_at(4, &mut first)?, 4);
        assert_eq!(&first, b"4567");
        assert_eq!(source.read_at(1, &mut second)?, 3);
        assert_eq!(&second, b"123");

        let file = File::open(fixture.path())?;
        let tiny = DxfResourceLimits::test_with_max_source_bytes(9);
        assert!(matches!(
            DxfFileSource::with_limits(file, tiny),
            Err(DxfError::ResourceLimitExceeded {
                resource: DxfResource::SourceBytes,
                limit: 9,
                observed: 10
            })
        ));
        Ok(())
    }

    #[test]
    fn failed_open_does_not_disclose_the_requested_path() -> Result<(), Box<dyn Error>> {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let missing = std::env::temp_dir().join(format!(
            "seacad-private-missing-{}-{id}.dxf",
            std::process::id()
        ));
        let error = match DxfFileSource::open(&missing, DxfResourceProfile::Safe) {
            Ok(_) => return Err(io::Error::other("missing source unexpectedly opened").into()),
            Err(error) => error,
        };
        assert_eq!(error.code(), DxfErrorCode::IO);
        let missing_text = missing.to_string_lossy();
        assert!(!error.to_string().contains(missing_text.as_ref()));
        Ok(())
    }
}

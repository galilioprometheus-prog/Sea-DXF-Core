use std::io;

use sha2::{Digest, Sha256};

use crate::{
    DxfByteSource, DxfCancellationToken, DxfError, DxfIoOperation, DxfReadControl, DxfReadObserver,
    DxfReadProgress, DxfResource, DxfResourceProfile, DxfSourceId,
};

const SOURCE_SCAN_CHUNK_BYTES: usize = 64 * 1024;
const SOURCE_SCAN_CHUNK_BYTES_U64: u64 = SOURCE_SCAN_CHUNK_BYTES as u64;

/// Evidence produced after every byte position in the accepted length is read.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSourceScanReceipt {
    source_id: DxfSourceId,
    source_bytes: u64,
}

impl DxfSourceScanReceipt {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_bytes(self) -> u64 {
        self.source_bytes
    }
}

/// Streams the complete bounded source into SHA-256 without retaining its bytes.
///
/// Progress begins at zero and advances after each successful source read.
/// Cancellation requested by either channel is honored before further work.
pub fn scan_dxf_source(
    source: &dyn DxfByteSource,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
    observer: &mut dyn DxfReadObserver,
) -> Result<DxfSourceScanReceipt, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let total_bytes = source.len();
    enforce_scan_limit(total_bytes, profile)?;
    report_progress(observer, cancellation, 0, total_bytes)?;

    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; SOURCE_SCAN_CHUNK_BYTES];
    let mut processed_bytes = 0_u64;

    while processed_bytes < total_bytes {
        ensure_not_cancelled(cancellation)?;
        let remaining = total_bytes - processed_bytes;
        let requested_u64 = remaining.min(SOURCE_SCAN_CHUNK_BYTES_U64);
        let requested = usize::try_from(requested_u64).map_err(|_| invalid_source_data())?;
        let read = source.read_at(processed_bytes, &mut buffer[..requested])?;
        ensure_not_cancelled(cancellation)?;

        if read == 0 {
            return Err(source_io_error(io::ErrorKind::UnexpectedEof));
        }
        if read > requested {
            return Err(invalid_source_data());
        }

        hasher.update(&buffer[..read]);
        let read_u64 = u64::try_from(read).map_err(|_| invalid_source_data())?;
        processed_bytes =
            processed_bytes
                .checked_add(read_u64)
                .ok_or(DxfError::OffsetOverflow {
                    offset: processed_bytes,
                    requested: read_u64,
                })?;
        report_progress(observer, cancellation, processed_bytes, total_bytes)?;
    }

    let digest = hasher.finalize();
    let mut hash = [0_u8; DxfSourceId::BYTE_LEN];
    hash.copy_from_slice(&digest);
    Ok(DxfSourceScanReceipt {
        source_id: DxfSourceId::from_sha256(hash),
        source_bytes: processed_bytes,
    })
}

fn enforce_scan_limit(total_bytes: u64, profile: DxfResourceProfile) -> Result<(), DxfError> {
    let limit = profile.limits().max_source_bytes();
    if total_bytes > limit {
        Err(DxfError::resource_limit(
            DxfResource::SourceBytes,
            limit,
            total_bytes,
        ))
    } else {
        Ok(())
    }
}

fn report_progress(
    observer: &mut dyn DxfReadObserver,
    cancellation: &DxfCancellationToken,
    processed_bytes: u64,
    total_bytes: u64,
) -> Result<(), DxfError> {
    ensure_not_cancelled(cancellation)?;
    let progress = match DxfReadProgress::new(processed_bytes, total_bytes) {
        Some(progress) => progress,
        None => return Err(invalid_source_data()),
    };
    if observer.on_progress(progress) == DxfReadControl::Cancel {
        return Err(DxfError::Cancelled);
    }
    ensure_not_cancelled(cancellation)
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_source_data() -> DxfError {
    source_io_error(io::ErrorKind::InvalidData)
}

fn source_io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs, io,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::scan_dxf_source;
    use crate::{
        DxfByteSource, DxfCancellationToken, DxfError, DxfFileSource, DxfIoOperation,
        DxfMemorySource, DxfReadControl, DxfResource, DxfResourceProfile, NoopDxfReadObserver,
    };

    static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

    struct TestFile {
        path: PathBuf,
    }

    impl TestFile {
        fn new(bytes: &[u8]) -> io::Result<Self> {
            let id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("seacad-scan-{}-{id}.dxf", std::process::id()));
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

    struct ChunkedSource {
        bytes: &'static [u8],
        declared_len: u64,
        max_read: usize,
        read_calls: AtomicU64,
    }

    impl ChunkedSource {
        fn new(bytes: &'static [u8], declared_len: u64, max_read: usize) -> Self {
            Self {
                bytes,
                declared_len,
                max_read,
                read_calls: AtomicU64::new(0),
            }
        }

        fn read_calls(&self) -> u64 {
            self.read_calls.load(Ordering::Relaxed)
        }
    }

    impl DxfByteSource for ChunkedSource {
        fn len(&self) -> u64 {
            self.declared_len
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            self.read_calls.fetch_add(1, Ordering::Relaxed);
            if destination.is_empty() {
                return Ok(0);
            }
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            if start >= self.bytes.len() {
                return Ok(0);
            }
            let count = (self.bytes.len() - start)
                .min(destination.len())
                .min(self.max_read);
            destination[..count].copy_from_slice(&self.bytes[start..start + count]);
            Ok(count)
        }
    }

    struct OverReportingSource;

    impl DxfByteSource for OverReportingSource {
        fn len(&self) -> u64 {
            1
        }

        fn read_at(&self, _offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            Ok(destination.len().saturating_add(1))
        }
    }

    struct FailingSource;

    impl DxfByteSource for FailingSource {
        fn len(&self) -> u64 {
            1
        }

        fn read_at(&self, _offset: u64, _destination: &mut [u8]) -> Result<usize, DxfError> {
            Err(DxfError::from_io(
                DxfIoOperation::Read,
                &io::Error::from(io::ErrorKind::PermissionDenied),
            ))
        }
    }

    #[test]
    fn known_sha256_vectors_and_receipt_are_stable() -> Result<(), DxfError> {
        let token = DxfCancellationToken::default();
        let source = DxfMemorySource::new(b"abc", DxfResourceProfile::Safe)?;
        let mut events = Vec::new();
        let mut observer = |progress: crate::DxfReadProgress| {
            events.push((progress.processed_bytes(), progress.total_bytes()));
            DxfReadControl::Continue
        };
        let receipt = scan_dxf_source(&source, DxfResourceProfile::Safe, &token, &mut observer)?;

        assert_eq!(receipt.source_bytes(), 3);
        assert_eq!(
            receipt.source_id().to_string(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(events, [(0, 3), (3, 3)]);

        let empty = DxfMemorySource::new(b"", DxfResourceProfile::Safe)?;
        let mut noop = NoopDxfReadObserver;
        let empty_receipt = scan_dxf_source(&empty, DxfResourceProfile::Safe, &token, &mut noop)?;
        assert_eq!(empty_receipt.source_bytes(), 0);
        assert_eq!(
            empty_receipt.source_id().to_string(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        Ok(())
    }

    #[test]
    fn file_and_memory_sources_have_the_same_identity() -> Result<(), Box<dyn Error>> {
        let bytes = b"file-backed DXF bytes";
        let fixture = TestFile::new(bytes)?;
        let file_source = DxfFileSource::open(fixture.path(), DxfResourceProfile::Safe)?;
        let memory_source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut file_observer = NoopDxfReadObserver;
        let mut memory_observer = NoopDxfReadObserver;

        let file_receipt = scan_dxf_source(
            &file_source,
            DxfResourceProfile::Safe,
            &token,
            &mut file_observer,
        )?;
        let memory_receipt = scan_dxf_source(
            &memory_source,
            DxfResourceProfile::Safe,
            &token,
            &mut memory_observer,
        )?;
        assert_eq!(file_receipt, memory_receipt);
        Ok(())
    }

    #[test]
    fn scan_crosses_internal_chunks_without_hash_drift() -> Result<(), DxfError> {
        let bytes = vec![b'a'; 1_000_000];
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut events = Vec::new();
        let mut observer = |progress: crate::DxfReadProgress| {
            events.push(progress.processed_bytes());
            DxfReadControl::Continue
        };

        let receipt = scan_dxf_source(&source, DxfResourceProfile::Safe, &token, &mut observer)?;
        assert_eq!(receipt.source_bytes(), 1_000_000);
        assert_eq!(
            receipt.source_id().to_string(),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
        assert_eq!(events.first().copied(), Some(0));
        assert_eq!(events.last().copied(), Some(1_000_000));
        assert_eq!(events.len(), 17);
        assert!(events.windows(2).all(|pair| pair[0] < pair[1]));
        Ok(())
    }

    #[test]
    fn partial_reads_produce_monotonic_progress() -> Result<(), DxfError> {
        let source = ChunkedSource::new(b"abc", 3, 1);
        let token = DxfCancellationToken::default();
        let mut events = Vec::new();
        let mut observer = |progress: crate::DxfReadProgress| {
            events.push(progress.processed_bytes());
            DxfReadControl::Continue
        };

        let receipt = scan_dxf_source(&source, DxfResourceProfile::Safe, &token, &mut observer)?;
        assert_eq!(receipt.source_bytes(), 3);
        assert_eq!(events, [0, 1, 2, 3]);
        assert_eq!(source.read_calls(), 3);
        Ok(())
    }

    #[test]
    fn observer_and_token_cancellation_stop_before_further_reads() {
        let source = ChunkedSource::new(b"abcd", 4, 1);
        let token = DxfCancellationToken::default();
        let mut observer = |progress: crate::DxfReadProgress| {
            if progress.processed_bytes() == 2 {
                DxfReadControl::Cancel
            } else {
                DxfReadControl::Continue
            }
        };
        let result = scan_dxf_source(&source, DxfResourceProfile::Safe, &token, &mut observer);
        assert!(matches!(result, Err(DxfError::Cancelled)));
        assert_eq!(source.read_calls(), 2);

        let token_cancelled_source = ChunkedSource::new(b"abcd", 4, 1);
        let token_cancelled = DxfCancellationToken::default();
        let observer_token = token_cancelled.clone();
        let mut observer = move |progress: crate::DxfReadProgress| {
            if progress.processed_bytes() == 2 {
                observer_token.cancel();
            }
            DxfReadControl::Continue
        };
        let result = scan_dxf_source(
            &token_cancelled_source,
            DxfResourceProfile::Safe,
            &token_cancelled,
            &mut observer,
        );
        assert!(matches!(result, Err(DxfError::Cancelled)));
        assert_eq!(token_cancelled_source.read_calls(), 2);

        let pre_cancelled_source = ChunkedSource::new(b"abcd", 4, 1);
        let pre_cancelled_token = DxfCancellationToken::default();
        pre_cancelled_token.cancel();
        let mut noop = NoopDxfReadObserver;
        let result = scan_dxf_source(
            &pre_cancelled_source,
            DxfResourceProfile::Safe,
            &pre_cancelled_token,
            &mut noop,
        );
        assert!(matches!(result, Err(DxfError::Cancelled)));
        assert_eq!(pre_cancelled_source.read_calls(), 0);
    }

    #[test]
    fn scan_limit_and_premature_eof_fail_closed() {
        let oversized_len = DxfResourceProfile::Safe.limits().max_source_bytes() + 1;
        let oversized = ChunkedSource::new(b"", oversized_len, 1);
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let result = scan_dxf_source(&oversized, DxfResourceProfile::Safe, &token, &mut noop);
        assert!(matches!(
            result,
            Err(DxfError::ResourceLimitExceeded {
                resource: DxfResource::SourceBytes,
                ..
            })
        ));
        assert_eq!(oversized.read_calls(), 0);

        let truncated = ChunkedSource::new(b"ab", 4, 2);
        let result = scan_dxf_source(&truncated, DxfResourceProfile::Safe, &token, &mut noop);
        assert!(matches!(
            result,
            Err(DxfError::Io {
                operation: DxfIoOperation::Read,
                kind: io::ErrorKind::UnexpectedEof,
                ..
            })
        ));
    }

    #[test]
    fn source_contract_and_io_failures_are_preserved() {
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let over_reported = scan_dxf_source(
            &OverReportingSource,
            DxfResourceProfile::Safe,
            &token,
            &mut noop,
        );
        assert!(matches!(
            over_reported,
            Err(DxfError::Io {
                operation: DxfIoOperation::Read,
                kind: io::ErrorKind::InvalidData,
                ..
            })
        ));

        let failed = scan_dxf_source(&FailingSource, DxfResourceProfile::Safe, &token, &mut noop);
        assert!(matches!(
            failed,
            Err(DxfError::Io {
                operation: DxfIoOperation::Read,
                kind: io::ErrorKind::PermissionDenied,
                ..
            })
        ));
    }
}

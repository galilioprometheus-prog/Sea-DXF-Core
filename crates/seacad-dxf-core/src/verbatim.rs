use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfCancellationToken, DxfError, DxfIoOperation, DxfReadControl,
    DxfReadObserver, DxfReadProgress, DxfSourceId,
};

const VERBATIM_CHUNK_BYTES: usize = 64 * 1024;
const VERBATIM_CHUNK_BYTES_U64: u64 = VERBATIM_CHUNK_BYTES as u64;

/// Evidence that a newly created file was copied and verified byte-identical.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfVerbatimWriteReceipt {
    source_id: DxfSourceId,
    output_id: DxfSourceId,
    bytes_written: u64,
}

impl DxfVerbatimWriteReceipt {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn output_id(self) -> DxfSourceId {
        self.output_id
    }

    #[must_use]
    pub const fn bytes_written(self) -> u64 {
        self.bytes_written
    }
}

impl DxfAsciiRawDocument<'_> {
    /// Copies the exact source bytes to a path that must not already exist.
    ///
    /// Progress spans two equal byte ranges: source copy, then output
    /// verification. Any failure after creation attempts to remove the
    /// incomplete destination. A cleanup failure replaces the primary error.
    pub fn write_verbatim_to_new_file(
        &self,
        destination: impl AsRef<Path>,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfVerbatimWriteReceipt, DxfError> {
        let destination = destination.as_ref();
        let source_len = self.source_len();
        let total_work = source_len.checked_mul(2).ok_or(DxfError::OffsetOverflow {
            offset: source_len,
            requested: source_len,
        })?;
        notify_progress(observer, cancellation, 0, total_work)?;

        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|error| DxfError::from_io(DxfIoOperation::Create, &error))?;

        let copy_result = self
            .copy_source(&mut output, cancellation, observer, total_work)
            .and_then(|source_id| {
                output
                    .flush()
                    .map_err(|error| DxfError::from_io(DxfIoOperation::Flush, &error))?;
                output
                    .sync_all()
                    .map_err(|error| DxfError::from_io(DxfIoOperation::Sync, &error))?;
                Ok(source_id)
            });
        drop(output);

        let observed_source_id = match copy_result {
            Ok(source_id) => source_id,
            Err(error) => return Err(remove_incomplete(destination, error)),
        };
        if observed_source_id != self.source_id() {
            let error = DxfError::SourceIdentityMismatch {
                expected: self.source_id(),
                observed: observed_source_id,
            };
            return Err(remove_incomplete(destination, error));
        }

        let output_id =
            match verify_output(destination, source_len, cancellation, observer, total_work) {
                Ok(output_id) => output_id,
                Err(error) => return Err(remove_incomplete(destination, error)),
            };
        if output_id != self.source_id() {
            let error = DxfError::VerbatimOutputIdentityMismatch {
                expected: self.source_id(),
                observed: output_id,
            };
            return Err(remove_incomplete(destination, error));
        }

        Ok(DxfVerbatimWriteReceipt {
            source_id: self.source_id(),
            output_id,
            bytes_written: source_len,
        })
    }

    fn copy_source(
        &self,
        output: &mut File,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
        total_work: u64,
    ) -> Result<DxfSourceId, DxfError> {
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; VERBATIM_CHUNK_BYTES];
        let mut copied = 0_u64;

        while copied < self.source_len() {
            ensure_not_cancelled(cancellation)?;
            let remaining = self.source_len() - copied;
            let chunk_u64 = remaining.min(VERBATIM_CHUNK_BYTES_U64);
            let chunk = usize::try_from(chunk_u64).map_err(|_| invalid_source_data())?;
            let span = ByteSpan::from_start_and_len(copied, chunk_u64).ok_or(
                DxfError::OffsetOverflow {
                    offset: copied,
                    requested: chunk_u64,
                },
            )?;
            let destination = buffer.get_mut(..chunk).ok_or_else(invalid_source_data)?;
            self.read_span(span, destination)?;
            ensure_not_cancelled(cancellation)?;
            output
                .write_all(destination)
                .map_err(|error| DxfError::from_io(DxfIoOperation::Write, &error))?;
            hasher.update(destination);
            copied = copied
                .checked_add(chunk_u64)
                .ok_or(DxfError::OffsetOverflow {
                    offset: copied,
                    requested: chunk_u64,
                })?;
            notify_progress(observer, cancellation, copied, total_work)?;
        }

        Ok(finalize_source_id(hasher))
    }
}

fn verify_output(
    destination: &Path,
    expected_len: u64,
    cancellation: &DxfCancellationToken,
    observer: &mut dyn DxfReadObserver,
    total_work: u64,
) -> Result<DxfSourceId, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let mut output =
        File::open(destination).map_err(|error| DxfError::from_io(DxfIoOperation::Open, &error))?;
    let observed_len = output
        .metadata()
        .map_err(|error| DxfError::from_io(DxfIoOperation::Metadata, &error))?
        .len();
    if observed_len != expected_len {
        return Err(DxfError::VerbatimOutputLengthMismatch {
            expected: expected_len,
            observed: observed_len,
        });
    }

    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; VERBATIM_CHUNK_BYTES];
    let mut verified = 0_u64;
    while verified < expected_len {
        ensure_not_cancelled(cancellation)?;
        let remaining = expected_len - verified;
        let requested_u64 = remaining.min(VERBATIM_CHUNK_BYTES_U64);
        let requested = usize::try_from(requested_u64).map_err(|_| invalid_source_data())?;
        let destination = buffer
            .get_mut(..requested)
            .ok_or_else(invalid_source_data)?;
        let read = output
            .read(destination)
            .map_err(|error| DxfError::from_io(DxfIoOperation::Read, &error))?;
        ensure_not_cancelled(cancellation)?;
        if read == 0 {
            return Err(source_io_error(io::ErrorKind::UnexpectedEof));
        }
        let payload = destination.get(..read).ok_or_else(invalid_source_data)?;
        hasher.update(payload);
        let read_u64 = u64::try_from(read).map_err(|_| invalid_source_data())?;
        verified = verified
            .checked_add(read_u64)
            .ok_or(DxfError::OffsetOverflow {
                offset: verified,
                requested: read_u64,
            })?;
        let work = expected_len
            .checked_add(verified)
            .ok_or(DxfError::OffsetOverflow {
                offset: expected_len,
                requested: verified,
            })?;
        notify_progress(observer, cancellation, work, total_work)?;
    }

    let mut extra = [0_u8; 1];
    if output
        .read(&mut extra)
        .map_err(|error| DxfError::from_io(DxfIoOperation::Read, &error))?
        != 0
    {
        let observed = output
            .metadata()
            .map_err(|error| DxfError::from_io(DxfIoOperation::Metadata, &error))?
            .len()
            .max(expected_len.saturating_add(1));
        return Err(DxfError::VerbatimOutputLengthMismatch {
            expected: expected_len,
            observed,
        });
    }
    Ok(finalize_source_id(hasher))
}

fn notify_progress(
    observer: &mut dyn DxfReadObserver,
    cancellation: &DxfCancellationToken,
    processed: u64,
    total: u64,
) -> Result<(), DxfError> {
    ensure_not_cancelled(cancellation)?;
    let progress = DxfReadProgress::new(processed, total).ok_or_else(invalid_source_data)?;
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

fn remove_incomplete(destination: &Path, primary: DxfError) -> DxfError {
    match fs::remove_file(destination) {
        Ok(()) => primary,
        Err(error) if error.kind() == io::ErrorKind::NotFound => primary,
        Err(error) => DxfError::from_io(DxfIoOperation::Remove, &error),
    }
}

fn finalize_source_id(hasher: Sha256) -> DxfSourceId {
    let digest = hasher.finalize();
    let mut bytes = [0_u8; DxfSourceId::BYTE_LEN];
    bytes.copy_from_slice(&digest);
    DxfSourceId::from_sha256(bytes)
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
        sync::{
            RwLock,
            atomic::{AtomicU64, Ordering},
        },
    };

    use sha2::{Digest, Sha256};

    use super::{DxfVerbatimWriteReceipt, finalize_source_id, verify_output};
    use crate::{
        DxfAsciiRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfErrorCode,
        DxfFileSource, DxfIoOperation, DxfMemorySource, DxfReadControl, DxfReadOptions,
        DxfResourceProfile, NoopDxfReadObserver,
    };

    static NEXT_TEMP_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);
    const STRICT_BYTES: &[u8] = b"0\r\nSECTION\n2\rENTITIES\r\n0\nENDSEC\r0\nEOF";

    #[test]
    fn strict_verbatim_write_is_byte_identical_and_verified() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let output = directory.path().join("strict-copy.dxf");
        let source = DxfMemorySource::new(STRICT_BYTES, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop)?;
        assert_eq!(
            document.source_id().to_string(),
            "d01aad6e2f1f3ecad12fd450747c531ebf0e3d44d70a66e9d993f1931213d068"
        );
        let mut events = Vec::new();
        let mut observer = |progress: crate::DxfReadProgress| {
            events.push((progress.processed_bytes(), progress.total_bytes()));
            DxfReadControl::Continue
        };

        let receipt = document.write_verbatim_to_new_file(&output, &token, &mut observer)?;
        assert_receipt(receipt, document.source_id(), STRICT_BYTES.len() as u64);
        assert_eq!(fs::read(&output)?, STRICT_BYTES);
        assert_eq!(
            events,
            [
                (0, (STRICT_BYTES.len() * 2) as u64),
                (STRICT_BYTES.len() as u64, (STRICT_BYTES.len() * 2) as u64),
                (
                    (STRICT_BYTES.len() * 2) as u64,
                    (STRICT_BYTES.len() * 2) as u64
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn compatible_recoveries_are_copied_without_normalization() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let cases: [(&str, &[u8], &str); 3] = [
            (
                "bom-and-padded-eof.dxf",
                b"\xef\xbb\xbf0\n EOF \n",
                "0d62665023c48c0b14c2300a0548cd5a1fd8ae756fceb8150df3390838a34771",
            ),
            (
                "missing-eof.dxf",
                b"0\nSECTION\n0\nENDSEC\n",
                "09fd60cb2c2df5b80b6c206cb16ba3f81c515c5182116ef8b2dc615bc92ff787",
            ),
            (
                "tail-after-eof.dxf",
                b"0\nEOF\n999\nafter\n\x1a",
                "d68c017bc2a740319ade1a905d41e3e9e10ba10efadc6b7ee41fb39f1f75f02d",
            ),
        ];
        let token = DxfCancellationToken::default();

        for (name, bytes, expected_hash) in cases {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            let mut noop = NoopDxfReadObserver;
            let document = DxfAsciiRawDocument::open(
                &source,
                DxfReadOptions::compatible(),
                &token,
                &mut noop,
            )?;
            assert_eq!(document.source_id().to_string(), expected_hash);
            let output = directory.path().join(name);
            let receipt = document.write_verbatim_to_new_file(&output, &token, &mut noop)?;
            assert_receipt(receipt, document.source_id(), bytes.len() as u64);
            assert_eq!(fs::read(output)?, bytes);
        }
        Ok(())
    }

    #[test]
    fn large_source_crosses_copy_and_verification_chunks() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let output = directory.path().join("large.dxf");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"999\n");
        bytes.resize(super::VERBATIM_CHUNK_BYTES * 2 + 7, b'x');
        bytes.extend_from_slice(b"\n0\nEOF\n");
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop)?;
        let mut events = Vec::new();
        let mut observer = |progress: crate::DxfReadProgress| {
            events.push(progress.processed_bytes());
            DxfReadControl::Continue
        };

        let receipt = document.write_verbatim_to_new_file(&output, &token, &mut observer)?;
        assert_receipt(receipt, document.source_id(), bytes.len() as u64);
        assert_eq!(fs::read(output)?, bytes);
        assert!(events.len() > 5);
        assert_eq!(events.first().copied(), Some(0));
        assert_eq!(events.last().copied(), Some((bytes.len() * 2) as u64));
        assert!(events.windows(2).all(|pair| pair[0] < pair[1]));
        Ok(())
    }

    #[test]
    fn file_backed_source_writes_to_a_distinct_new_file() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let input = directory.path().join("input.dxf");
        let output = directory.path().join("output.dxf");
        fs::write(&input, STRICT_BYTES)?;
        let source = DxfFileSource::open(&input, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop)?;

        document.write_verbatim_to_new_file(&output, &token, &mut noop)?;
        assert_eq!(fs::read(&input)?, STRICT_BYTES);
        assert_eq!(fs::read(&output)?, STRICT_BYTES);
        Ok(())
    }

    #[test]
    fn existing_destination_is_never_modified() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let output = directory.path().join("existing.dxf");
        fs::write(&output, b"keep me")?;
        let source = DxfMemorySource::new(STRICT_BYTES, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop)?;

        let error = document
            .write_verbatim_to_new_file(&output, &token, &mut noop)
            .err()
            .ok_or(io::Error::other("existing destination was overwritten"))?;
        assert!(matches!(
            error,
            DxfError::Io {
                operation: DxfIoOperation::Create,
                kind: io::ErrorKind::AlreadyExists,
                ..
            }
        ));
        assert_eq!(fs::read(output)?, b"keep me");
        Ok(())
    }

    #[test]
    fn changed_source_fails_precondition_and_removes_output() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let output = directory.path().join("changed-source.dxf");
        let source = MutableSource::new(STRICT_BYTES);
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::strict(), &token, &mut noop)?;
        source.replace_same_len(b"0\r\nSECTION\n2\rENTITIES\r\n0\nENDSEC\r0\nEOG")?;

        let error = document
            .write_verbatim_to_new_file(&output, &token, &mut noop)
            .err()
            .ok_or(io::Error::other("changed source was accepted"))?;
        assert_eq!(error.code(), DxfErrorCode::SOURCE_IDENTITY_MISMATCH);
        assert!(!output.exists());
        Ok(())
    }

    #[test]
    fn observer_cancellation_removes_partial_output() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let output = directory.path().join("cancelled.dxf");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"0\nEOF\n");
        bytes.resize(super::VERBATIM_CHUNK_BYTES * 2 + 7, b'x');
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::compatible(), &token, &mut noop)?;
        let mut observer = |progress: crate::DxfReadProgress| {
            if progress.processed_bytes() == 0 {
                DxfReadControl::Continue
            } else {
                DxfReadControl::Cancel
            }
        };

        let result = document.write_verbatim_to_new_file(&output, &token, &mut observer);
        assert!(matches!(result, Err(DxfError::Cancelled)));
        assert!(!output.exists());
        Ok(())
    }

    #[test]
    fn empty_recovered_document_writes_an_empty_verified_file() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let output = directory.path().join("empty.dxf");
        let source = DxfMemorySource::new(b"", DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;
        let document =
            DxfAsciiRawDocument::open(&source, DxfReadOptions::compatible(), &token, &mut noop)?;

        let receipt = document.write_verbatim_to_new_file(&output, &token, &mut noop)?;
        assert_receipt(receipt, document.source_id(), 0);
        assert_eq!(fs::metadata(output)?.len(), 0);
        Ok(())
    }

    #[test]
    fn output_verifier_detects_length_and_identity_differences() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let output = directory.path().join("verify.dxf");
        fs::write(&output, b"abd")?;
        let token = DxfCancellationToken::default();
        let mut noop = NoopDxfReadObserver;

        let length_error = verify_output(&output, 4, &token, &mut noop, 8)
            .err()
            .ok_or(io::Error::other("length mismatch was accepted"))?;
        assert_eq!(
            length_error.code(),
            DxfErrorCode::VERBATIM_OUTPUT_LENGTH_MISMATCH
        );

        let observed = verify_output(&output, 3, &token, &mut noop, 6)?;
        let mut expected_hasher = Sha256::new();
        expected_hasher.update(b"abc");
        let expected = finalize_source_id(expected_hasher);
        assert_ne!(observed, expected);
        Ok(())
    }

    fn assert_receipt(receipt: DxfVerbatimWriteReceipt, expected: crate::DxfSourceId, bytes: u64) {
        assert_eq!(receipt.source_id(), expected);
        assert_eq!(receipt.output_id(), expected);
        assert_eq!(receipt.bytes_written(), bytes);
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> io::Result<Self> {
            for _ in 0..100 {
                let id = NEXT_TEMP_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir()
                    .join(format!("seacad-verbatim-{}-{id}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => return Ok(Self { path }),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error),
                }
            }
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "unable to reserve temporary test directory",
            ))
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    struct MutableSource {
        bytes: RwLock<Vec<u8>>,
        len: u64,
    }

    impl MutableSource {
        fn new(bytes: &[u8]) -> Self {
            Self {
                bytes: RwLock::new(bytes.to_vec()),
                len: bytes.len() as u64,
            }
        }

        fn replace_same_len(&self, replacement: &[u8]) -> Result<(), io::Error> {
            if replacement.len() as u64 != self.len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "replacement length differs",
                ));
            }
            let mut bytes = match self.bytes.write() {
                Ok(bytes) => bytes,
                Err(poisoned) => poisoned.into_inner(),
            };
            bytes.copy_from_slice(replacement);
            Ok(())
        }
    }

    impl DxfByteSource for MutableSource {
        fn len(&self) -> u64 {
            self.len
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            let bytes = match self.bytes.read() {
                Ok(bytes) => bytes,
                Err(poisoned) => poisoned.into_inner(),
            };
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            if start >= bytes.len() || destination.is_empty() {
                return Ok(0);
            }
            let count = (bytes.len() - start).min(destination.len());
            destination[..count].copy_from_slice(&bytes[start..start + count]);
            Ok(count)
        }
    }
}

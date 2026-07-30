//! Canonical ASCII framing writer over immutable ASCII raw groups.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{
    ByteSpan, DxfAsciiDocumentConformance, DxfAsciiRawDocument, DxfCancellationToken, DxfError,
    DxfFileSource, DxfIoOperation, DxfReadControl, DxfReadObserver, DxfReadOptions,
    DxfReadProgress, DxfResource, DxfResourceProfile, DxfSourceId, NoopDxfReadObserver,
};

const CANONICAL_CHUNK_BYTES: usize = 64 * 1024;
const CANONICAL_CHUNK_BYTES_U64: u64 = CANONICAL_CHUNK_BYTES as u64;
const CANONICAL_EOF_GROUP: &[u8] = b"0\nEOF\n";

/// How canonical ASCII writing handled the source EOF envelope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfCanonicalAsciiEnvelopeAction {
    PreservedStrict,
    CanonicalizedRecovered,
    AppendedMissingEof,
}

/// Verified receipt for one strictly reparsed canonical ASCII output.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfCanonicalAsciiWriteReceipt {
    source_id: DxfSourceId,
    output_id: DxfSourceId,
    bytes_written: u64,
    groups_written: u64,
    envelope_action: DxfCanonicalAsciiEnvelopeAction,
}

impl DxfCanonicalAsciiWriteReceipt {
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

    #[must_use]
    pub const fn groups_written(self) -> u64 {
        self.groups_written
    }

    #[must_use]
    pub const fn envelope_action(self) -> DxfCanonicalAsciiEnvelopeAction {
        self.envelope_action
    }
}

impl DxfAsciiRawDocument<'_> {
    /// Writes minimal decimal group codes, LF framing, and exact value payloads.
    ///
    /// Accepted EOF recoveries become one strict terminal `0`/`EOF` group.
    pub fn write_canonical_ascii_to_new_file(
        &self,
        destination: impl AsRef<Path>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfCanonicalAsciiWriteReceipt, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let projection = canonical_projection(self, profile)?;
        let total_work = self.source_len().checked_add(projection.output_len).ok_or(
            DxfError::OffsetOverflow {
                offset: self.source_len(),
                requested: projection.output_len,
            },
        )?;
        notify_progress(observer, cancellation, 0, total_work)?;

        let destination = destination.as_ref();
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|error| DxfError::from_io(DxfIoOperation::Create, &error))?;
        let stream_result = stream_canonical_ascii(
            self,
            &mut output,
            projection,
            cancellation,
            observer,
            total_work,
        )
        .and_then(|stream| {
            output
                .flush()
                .map_err(|error| DxfError::from_io(DxfIoOperation::Flush, &error))?;
            output
                .sync_all()
                .map_err(|error| DxfError::from_io(DxfIoOperation::Sync, &error))?;
            Ok(stream)
        });
        drop(output);

        let stream = match stream_result {
            Ok(stream) => stream,
            Err(error) => return Err(remove_incomplete(destination, error)),
        };
        if stream.source_id != self.source_id() {
            return Err(remove_incomplete(
                destination,
                DxfError::SourceIdentityMismatch {
                    expected: self.source_id(),
                    observed: stream.source_id,
                },
            ));
        }
        if stream.bytes_written != projection.output_len {
            return Err(remove_incomplete(
                destination,
                DxfError::CanonicalOutputLengthMismatch {
                    expected: projection.output_len,
                    observed: stream.bytes_written,
                },
            ));
        }

        let verified_id = match verify_output(
            destination,
            projection.output_len,
            self.source_len(),
            cancellation,
            observer,
            total_work,
        ) {
            Ok(identity) => identity,
            Err(error) => return Err(remove_incomplete(destination, error)),
        };
        if verified_id != stream.output_id {
            return Err(remove_incomplete(
                destination,
                DxfError::CanonicalOutputIdentityMismatch {
                    expected: stream.output_id,
                    observed: verified_id,
                },
            ));
        }
        if let Err(error) = strict_reparse_output(destination, profile, cancellation, verified_id) {
            return Err(remove_incomplete(destination, error));
        }

        Ok(DxfCanonicalAsciiWriteReceipt {
            source_id: stream.source_id,
            output_id: verified_id,
            bytes_written: stream.bytes_written,
            groups_written: projection.output_group_count,
            envelope_action: projection.envelope_action,
        })
    }
}

#[derive(Clone, Copy)]
struct CanonicalProjection {
    output_len: u64,
    output_group_count: u64,
    eof_occurrence: Option<u64>,
    envelope_action: DxfCanonicalAsciiEnvelopeAction,
}

fn canonical_projection(
    document: &DxfAsciiRawDocument<'_>,
    profile: DxfResourceProfile,
) -> Result<CanonicalProjection, DxfError> {
    let limits = profile.limits();
    enforce_limit(
        DxfResource::SourceBytes,
        limits.max_source_bytes(),
        document.source_len(),
    )?;
    let input_group_count = u64::try_from(document.groups().len()).map_err(|_| invalid_data())?;
    let appended_eof = document.eof_occurrence().is_none();
    let output_group_count = input_group_count
        .checked_add(u64::from(appended_eof))
        .ok_or_else(invalid_data)?;
    enforce_limit(
        DxfResource::Records,
        limits.max_records(),
        output_group_count,
    )?;

    let mut output_len = 0_u64;
    for group in document.groups() {
        let value_len = if document.eof_occurrence() == Some(group.occurrence()) {
            3
        } else {
            group.value_content_span().len()
        };
        enforce_limit(DxfResource::ValueBytes, limits.max_value_bytes(), value_len)?;
        let group_len = decimal_i16_len(group.group_code().value())
            .checked_add(1)
            .and_then(|len| len.checked_add(value_len))
            .and_then(|len| len.checked_add(1))
            .ok_or_else(invalid_data)?;
        output_len = output_len
            .checked_add(group_len)
            .ok_or(DxfError::OffsetOverflow {
                offset: output_len,
                requested: group_len,
            })?;
    }
    if appended_eof {
        output_len = output_len
            .checked_add(CANONICAL_EOF_GROUP.len() as u64)
            .ok_or(DxfError::OffsetOverflow {
                offset: output_len,
                requested: CANONICAL_EOF_GROUP.len() as u64,
            })?;
    }
    enforce_limit(
        DxfResource::SourceBytes,
        limits.max_source_bytes(),
        output_len,
    )?;
    let envelope_action = if appended_eof {
        DxfCanonicalAsciiEnvelopeAction::AppendedMissingEof
    } else if document.conformance() == DxfAsciiDocumentConformance::Strict {
        DxfCanonicalAsciiEnvelopeAction::PreservedStrict
    } else {
        DxfCanonicalAsciiEnvelopeAction::CanonicalizedRecovered
    };
    Ok(CanonicalProjection {
        output_len,
        output_group_count,
        eof_occurrence: document.eof_occurrence(),
        envelope_action,
    })
}

struct CanonicalStreamReceipt {
    source_id: DxfSourceId,
    output_id: DxfSourceId,
    bytes_written: u64,
}

struct CanonicalStream<'a, 'observer> {
    document: &'a DxfAsciiRawDocument<'a>,
    output: &'observer mut File,
    cancellation: &'observer DxfCancellationToken,
    observer: &'observer mut dyn DxfReadObserver,
    total_work: u64,
    buffer: [u8; CANONICAL_CHUNK_BYTES],
    source_hasher: Sha256,
    output_hasher: Sha256,
    source_processed: u64,
    output_written: u64,
}

impl CanonicalStream<'_, '_> {
    fn source_range(&mut self, span: ByteSpan, emit: bool) -> Result<(), DxfError> {
        let mut offset = span.start();
        while offset < span.end() {
            ensure_not_cancelled(self.cancellation)?;
            let len_u64 = (span.end() - offset).min(CANONICAL_CHUNK_BYTES_U64);
            let len = usize::try_from(len_u64).map_err(|_| invalid_data())?;
            let chunk_span =
                ByteSpan::from_start_and_len(offset, len_u64).ok_or(DxfError::OffsetOverflow {
                    offset,
                    requested: len_u64,
                })?;
            let chunk = self.buffer.get_mut(..len).ok_or_else(invalid_data)?;
            self.document.read_span(chunk_span, chunk)?;
            self.source_hasher.update(&*chunk);
            if emit {
                self.output
                    .write_all(chunk)
                    .map_err(|error| DxfError::from_io(DxfIoOperation::Write, &error))?;
                self.output_hasher.update(&*chunk);
                self.add_output(len_u64)?;
            }
            offset = offset
                .checked_add(len_u64)
                .ok_or(DxfError::OffsetOverflow {
                    offset,
                    requested: len_u64,
                })?;
            self.source_processed =
                self.source_processed
                    .checked_add(len_u64)
                    .ok_or(DxfError::OffsetOverflow {
                        offset: self.source_processed,
                        requested: len_u64,
                    })?;
            notify_progress(
                self.observer,
                self.cancellation,
                self.source_processed,
                self.total_work,
            )?;
        }
        Ok(())
    }

    fn literal(&mut self, bytes: &[u8]) -> Result<(), DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        self.output
            .write_all(bytes)
            .map_err(|error| DxfError::from_io(DxfIoOperation::Write, &error))?;
        self.output_hasher.update(bytes);
        self.add_output(u64::try_from(bytes.len()).map_err(|_| invalid_data())?)
    }

    fn add_output(&mut self, len: u64) -> Result<(), DxfError> {
        self.output_written =
            self.output_written
                .checked_add(len)
                .ok_or(DxfError::OffsetOverflow {
                    offset: self.output_written,
                    requested: len,
                })?;
        Ok(())
    }
}

fn stream_canonical_ascii(
    document: &DxfAsciiRawDocument<'_>,
    output: &mut File,
    projection: CanonicalProjection,
    cancellation: &DxfCancellationToken,
    observer: &mut dyn DxfReadObserver,
    total_work: u64,
) -> Result<CanonicalStreamReceipt, DxfError> {
    let mut stream = CanonicalStream {
        document,
        output,
        cancellation,
        observer,
        total_work,
        buffer: [0_u8; CANONICAL_CHUNK_BYTES],
        source_hasher: Sha256::new(),
        output_hasher: Sha256::new(),
        source_processed: 0,
        output_written: 0,
    };
    let mut source_cursor = 0_u64;
    let mut code_buffer = [0_u8; 6];
    for group in document.groups() {
        let code = encode_i16(group.group_code().value(), &mut code_buffer);
        stream.literal(code)?;
        stream.literal(b"\n")?;
        let value_span = group.value_content_span();
        stream.source_range(
            ByteSpan::new(source_cursor, value_span.start()).ok_or_else(invalid_data)?,
            false,
        )?;
        if projection.eof_occurrence == Some(group.occurrence()) {
            stream.source_range(value_span, false)?;
            stream.literal(b"EOF")?;
        } else {
            stream.source_range(value_span, true)?;
        }
        stream.literal(b"\n")?;
        stream.source_range(
            ByteSpan::new(value_span.end(), group.full_span().end()).ok_or_else(invalid_data)?,
            false,
        )?;
        source_cursor = group.full_span().end();
    }
    stream.source_range(
        ByteSpan::new(source_cursor, document.source_len()).ok_or_else(invalid_data)?,
        false,
    )?;
    if projection.eof_occurrence.is_none() {
        stream.literal(CANONICAL_EOF_GROUP)?;
    }
    if stream.source_processed != document.source_len() {
        return Err(invalid_data());
    }
    Ok(CanonicalStreamReceipt {
        source_id: finalize_id(stream.source_hasher),
        output_id: finalize_id(stream.output_hasher),
        bytes_written: stream.output_written,
    })
}

fn strict_reparse_output(
    destination: &Path,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
    expected_id: DxfSourceId,
) -> Result<(), DxfError> {
    let source = DxfFileSource::open(destination, profile)?;
    let options = DxfReadOptions::new(crate::DxfReadMode::Strict, profile);
    let mut observer = NoopDxfReadObserver;
    let document = DxfAsciiRawDocument::open(&source, options, cancellation, &mut observer)?;
    if document.source_id() == expected_id {
        Ok(())
    } else {
        Err(DxfError::CanonicalOutputIdentityMismatch {
            expected: expected_id,
            observed: document.source_id(),
        })
    }
}

fn verify_output(
    destination: &Path,
    expected_len: u64,
    source_work: u64,
    cancellation: &DxfCancellationToken,
    observer: &mut dyn DxfReadObserver,
    total_work: u64,
) -> Result<DxfSourceId, DxfError> {
    let mut output =
        File::open(destination).map_err(|error| DxfError::from_io(DxfIoOperation::Open, &error))?;
    let observed_len = output
        .metadata()
        .map_err(|error| DxfError::from_io(DxfIoOperation::Metadata, &error))?
        .len();
    if observed_len != expected_len {
        return Err(DxfError::CanonicalOutputLengthMismatch {
            expected: expected_len,
            observed: observed_len,
        });
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; CANONICAL_CHUNK_BYTES];
    let mut verified = 0_u64;
    while verified < expected_len {
        ensure_not_cancelled(cancellation)?;
        let requested_u64 = (expected_len - verified).min(CANONICAL_CHUNK_BYTES_U64);
        let requested = usize::try_from(requested_u64).map_err(|_| invalid_data())?;
        let destination = buffer.get_mut(..requested).ok_or_else(invalid_data)?;
        let read = output
            .read(destination)
            .map_err(|error| DxfError::from_io(DxfIoOperation::Read, &error))?;
        if read == 0 {
            return Err(unexpected_eof());
        }
        let payload = destination.get(..read).ok_or_else(invalid_data)?;
        hasher.update(payload);
        let read_u64 = u64::try_from(read).map_err(|_| invalid_data())?;
        verified = verified
            .checked_add(read_u64)
            .ok_or(DxfError::OffsetOverflow {
                offset: verified,
                requested: read_u64,
            })?;
        notify_progress(
            observer,
            cancellation,
            source_work
                .checked_add(verified)
                .ok_or(DxfError::OffsetOverflow {
                    offset: source_work,
                    requested: verified,
                })?,
            total_work,
        )?;
    }
    Ok(finalize_id(hasher))
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

fn decimal_i16_len(value: i16) -> u64 {
    let mut magnitude = i32::from(value).unsigned_abs();
    let mut len = u64::from(value < 0);
    loop {
        len += 1;
        magnitude /= 10;
        if magnitude == 0 {
            return len;
        }
    }
}

fn enforce_limit(resource: DxfResource, limit: u64, observed: u64) -> Result<(), DxfError> {
    if observed > limit {
        Err(DxfError::resource_limit(resource, limit, observed))
    } else {
        Ok(())
    }
}

fn notify_progress(
    observer: &mut dyn DxfReadObserver,
    cancellation: &DxfCancellationToken,
    processed: u64,
    total: u64,
) -> Result<(), DxfError> {
    ensure_not_cancelled(cancellation)?;
    let progress = DxfReadProgress::new(processed, total).ok_or_else(invalid_data)?;
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

fn finalize_id(hasher: Sha256) -> DxfSourceId {
    let digest = hasher.finalize();
    let mut bytes = [0_u8; DxfSourceId::BYTE_LEN];
    bytes.copy_from_slice(&digest);
    DxfSourceId::from_sha256(bytes)
}

fn invalid_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn unexpected_eof() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::UnexpectedEof),
    )
}

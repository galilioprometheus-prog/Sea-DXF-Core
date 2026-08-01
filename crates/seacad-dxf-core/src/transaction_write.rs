//! Verified create-new streaming application of immutable transaction plans.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfFileSource, DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadControl,
    DxfReadObserver, DxfReadOptions, DxfReadProgress, DxfResourceProfile, DxfSourceId,
    DxfTransactionPlan, NoopDxfReadObserver,
};

const WRITE_CHUNK_BYTES: usize = 64 * 1024;
const WRITE_CHUNK_BYTES_U64: u64 = WRITE_CHUNK_BYTES as u64;

/// Verified identities and counts for one newly created transaction output.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTransactionWriteReceipt {
    source_id: DxfSourceId,
    output_id: DxfSourceId,
    bytes_written: u64,
    patch_count: u64,
}

impl DxfTransactionWriteReceipt {
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
    pub const fn patch_count(self) -> u64 {
        self.patch_count
    }
}

/// Verified create-new write receipt paired with an executable inverse plan.
#[derive(Debug)]
pub struct DxfTransactionWriteJournal {
    receipt: DxfTransactionWriteReceipt,
    inverse: DxfTransactionPlan,
}

impl DxfTransactionWriteJournal {
    #[must_use]
    pub const fn receipt(&self) -> DxfTransactionWriteReceipt {
        self.receipt
    }

    #[must_use]
    pub const fn inverse_plan(&self) -> &DxfTransactionPlan {
        &self.inverse
    }

    #[must_use]
    pub fn into_parts(self) -> (DxfTransactionWriteReceipt, DxfTransactionPlan) {
        (self.receipt, self.inverse)
    }
}

impl DxfTransactionPlan {
    /// Streams this plan to a path that must not already exist, then verifies it.
    ///
    /// Any failure after creation attempts to remove the incomplete output.
    pub fn write_to_new_file(
        &self,
        document: DxfRawDocumentView<'_>,
        destination: impl AsRef<Path>,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfTransactionWriteReceipt, DxfError> {
        self.validate_source_precondition(document)?;
        ensure_not_cancelled(cancellation)?;
        let total_work = self.source_len().checked_add(self.projected_len()).ok_or(
            DxfError::OffsetOverflow {
                offset: self.source_len(),
                requested: self.projected_len(),
            },
        )?;
        notify_progress(observer, cancellation, 0, total_work)?;

        let destination = destination.as_ref();
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|error| DxfError::from_io(DxfIoOperation::Create, &error))?;

        let stream_result = stream_plan(
            self,
            document,
            &mut output,
            cancellation,
            observer,
            total_work,
        )
        .and_then(|receipt| {
            output
                .flush()
                .map_err(|error| DxfError::from_io(DxfIoOperation::Flush, &error))?;
            output
                .sync_all()
                .map_err(|error| DxfError::from_io(DxfIoOperation::Sync, &error))?;
            Ok(receipt)
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
        if stream.bytes_written != self.projected_len() {
            return Err(remove_incomplete(
                destination,
                DxfError::TransactionOutputLengthMismatch {
                    expected: self.projected_len(),
                    observed: stream.bytes_written,
                },
            ));
        }

        let verified_id = match verify_output(
            destination,
            self.projected_len(),
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
                DxfError::TransactionOutputIdentityMismatch {
                    expected: stream.output_id,
                    observed: verified_id,
                },
            ));
        }

        Ok(DxfTransactionWriteReceipt {
            source_id: stream.source_id,
            output_id: verified_id,
            bytes_written: stream.bytes_written,
            patch_count: self.patches().len() as u64,
        })
    }

    /// Writes, strictly reparses, and returns an executable inverse journal.
    ///
    /// Reparse and inverse failures remove the newly created output.
    pub fn write_reparse_and_journal_to_new_file(
        &self,
        document: DxfRawDocumentView<'_>,
        destination: impl AsRef<Path>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
        observer: &mut dyn DxfReadObserver,
    ) -> Result<DxfTransactionWriteJournal, DxfError> {
        let destination = destination.as_ref();
        let receipt = self.write_to_new_file(document, destination, cancellation, observer)?;
        let journal = reparse_and_materialize_inverse(
            self,
            document,
            destination,
            receipt,
            profile,
            cancellation,
        );
        match journal {
            Ok(journal) => Ok(journal),
            Err(error) => Err(remove_incomplete(destination, error)),
        }
    }
}

fn reparse_and_materialize_inverse(
    plan: &DxfTransactionPlan,
    source_document: DxfRawDocumentView<'_>,
    destination: &Path,
    receipt: DxfTransactionWriteReceipt,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<DxfTransactionWriteJournal, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let output_source = DxfFileSource::open(destination, profile)?;
    let options = DxfReadOptions::new(crate::DxfReadMode::Strict, profile);
    let mut observer = NoopDxfReadObserver;
    let inverse = match plan.format() {
        DxfRawDocumentFormat::Ascii => {
            let post_image =
                DxfAsciiRawDocument::open(&output_source, options, cancellation, &mut observer)?;
            validate_reparsed_identity(receipt, post_image.source_id())?;
            plan.materialize_inverse_plan(
                source_document,
                DxfRawDocumentView::from(&post_image),
                profile,
                cancellation,
            )?
        }
        DxfRawDocumentFormat::Binary => {
            let post_image =
                DxfBinaryRawDocument::open(&output_source, options, cancellation, &mut observer)?;
            validate_reparsed_identity(receipt, post_image.source_id())?;
            plan.materialize_inverse_plan(
                source_document,
                DxfRawDocumentView::from(&post_image),
                profile,
                cancellation,
            )?
        }
    };
    Ok(DxfTransactionWriteJournal { receipt, inverse })
}

fn validate_reparsed_identity(
    receipt: DxfTransactionWriteReceipt,
    reparsed_id: DxfSourceId,
) -> Result<(), DxfError> {
    if reparsed_id == receipt.output_id() {
        Ok(())
    } else {
        Err(DxfError::TransactionOutputIdentityMismatch {
            expected: receipt.output_id(),
            observed: reparsed_id,
        })
    }
}

struct StreamReceipt {
    source_id: DxfSourceId,
    output_id: DxfSourceId,
    bytes_written: u64,
}

struct PlanStream<'a, 'observer> {
    document: DxfRawDocumentView<'a>,
    output: &'observer mut File,
    cancellation: &'observer DxfCancellationToken,
    observer: &'observer mut dyn DxfReadObserver,
    total_work: u64,
    buffer: [u8; WRITE_CHUNK_BYTES],
    source_hasher: Sha256,
    output_hasher: Sha256,
    source_processed: u64,
    output_written: u64,
}

impl PlanStream<'_, '_> {
    fn source_range(&mut self, span: ByteSpan, emit: bool) -> Result<(), DxfError> {
        let mut offset = span.start();
        while offset < span.end() {
            ensure_not_cancelled(self.cancellation)?;
            let len_u64 = (span.end() - offset).min(WRITE_CHUNK_BYTES_U64);
            let len = usize::try_from(len_u64).map_err(|_| invalid_data())?;
            let chunk_span =
                ByteSpan::from_start_and_len(offset, len_u64).ok_or(DxfError::OffsetOverflow {
                    offset,
                    requested: len_u64,
                })?;
            let chunk = self.buffer.get_mut(..len).ok_or_else(invalid_data)?;
            self.document.read_span(chunk_span, chunk)?;
            ensure_not_cancelled(self.cancellation)?;
            self.source_hasher.update(&*chunk);
            if emit {
                self.output
                    .write_all(chunk)
                    .map_err(|error| DxfError::from_io(DxfIoOperation::Write, &error))?;
                self.output_hasher.update(&*chunk);
                self.output_written =
                    self.output_written
                        .checked_add(len_u64)
                        .ok_or(DxfError::OffsetOverflow {
                            offset: self.output_written,
                            requested: len_u64,
                        })?;
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

    fn replacement(&mut self, bytes: &[u8]) -> Result<(), DxfError> {
        ensure_not_cancelled(self.cancellation)?;
        self.output
            .write_all(bytes)
            .map_err(|error| DxfError::from_io(DxfIoOperation::Write, &error))?;
        self.output_hasher.update(bytes);
        let len = u64::try_from(bytes.len()).map_err(|_| invalid_data())?;
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

fn stream_plan(
    plan: &DxfTransactionPlan,
    document: DxfRawDocumentView<'_>,
    output: &mut File,
    cancellation: &DxfCancellationToken,
    observer: &mut dyn DxfReadObserver,
    total_work: u64,
) -> Result<StreamReceipt, DxfError> {
    let mut stream = PlanStream {
        document,
        output,
        cancellation,
        observer,
        total_work,
        buffer: [0_u8; WRITE_CHUNK_BYTES],
        source_hasher: Sha256::new(),
        output_hasher: Sha256::new(),
        source_processed: 0,
        output_written: 0,
    };
    let mut cursor = 0_u64;
    for patch in plan.patches().iter().copied() {
        let unchanged =
            ByteSpan::new(cursor, patch.source_span().start()).ok_or_else(invalid_data)?;
        stream.source_range(unchanged, true)?;
        stream.source_range(patch.source_span(), false)?;
        let replacement = plan
            .replacement_bytes_for_patch_ordinal(patch.ordinal())
            .ok_or_else(invalid_data)?;
        stream.replacement(replacement)?;
        cursor = patch.source_span().end();
    }
    let tail = ByteSpan::new(cursor, plan.source_len()).ok_or_else(invalid_data)?;
    stream.source_range(tail, true)?;
    if stream.source_processed != plan.source_len() {
        return Err(invalid_data());
    }
    Ok(StreamReceipt {
        source_id: finalize_id(stream.source_hasher),
        output_id: finalize_id(stream.output_hasher),
        bytes_written: stream.output_written,
    })
}

fn verify_output(
    destination: &Path,
    expected_len: u64,
    source_work: u64,
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
        return Err(DxfError::TransactionOutputLengthMismatch {
            expected: expected_len,
            observed: observed_len,
        });
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; WRITE_CHUNK_BYTES];
    let mut verified = 0_u64;
    while verified < expected_len {
        ensure_not_cancelled(cancellation)?;
        let requested_u64 = (expected_len - verified).min(WRITE_CHUNK_BYTES_U64);
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
        let progress = source_work
            .checked_add(verified)
            .ok_or(DxfError::OffsetOverflow {
                offset: source_work,
                requested: verified,
            })?;
        notify_progress(observer, cancellation, progress, total_work)?;
    }
    Ok(finalize_id(hasher))
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

pub(crate) fn remove_created_destination(destination: &Path) -> Result<(), DxfError> {
    match fs::remove_file(destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(DxfError::from_io(DxfIoOperation::Remove, &error)),
    }
}

fn remove_incomplete(destination: &Path, primary: DxfError) -> DxfError {
    match remove_created_destination(destination) {
        Ok(()) => primary,
        Err(error) => error,
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

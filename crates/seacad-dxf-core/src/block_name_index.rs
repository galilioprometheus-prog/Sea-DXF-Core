//! Duplicate-preserving exact BLOCK-name lookup.

use std::io;

use sha2::{Digest, Sha256};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockNameConsistencyDirectory,
    DxfBlockNameConsistencyState, DxfBlockRecordTextValue, DxfBlockRecordValueEntry,
    DxfCancellationToken, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

const NAME_IO_CHUNK_BYTES: usize = 4 * 1024;
type NameDigest = [u8; 32];

/// One BLOCK admitted to exact-name lookup.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockNameIndexMatch {
    record: DxfBlockRecordValueEntry,
    name: DxfBlockRecordTextValue,
}

impl DxfBlockNameIndexMatch {
    #[must_use]
    pub const fn record(self) -> DxfBlockRecordValueEntry {
        self.record
    }

    #[must_use]
    pub const fn name(self) -> DxfBlockRecordTextValue {
        self.name
    }
}

/// Exact lookup outcome over matched primary/secondary BLOCK names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockNameIndexLookup<'a> {
    Missing,
    Unique(DxfBlockNameIndexMatch),
    Ambiguous(&'a [DxfBlockNameIndexMatch]),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DxfBlockNameIndexKey {
    digest: NameDigest,
    exact_group_ordinal: u32,
}

#[derive(Clone, Copy)]
struct PendingMatch {
    key: DxfBlockNameIndexKey,
    indexed: DxfBlockNameIndexMatch,
}

#[derive(Clone, Copy)]
struct ExactGroupRepresentative {
    digest: NameDigest,
    span: ByteSpan,
    ordinal: u32,
}

/// Immutable exact-name index retaining all M10.1e consistency evidence.
///
/// Only records whose primary and secondary names match participate in lookup.
/// Conflicting and unusable names remain visible through
/// `consistency_directory`.
#[derive(Debug)]
pub struct DxfBlockNameIndexDirectory {
    source_id: DxfSourceId,
    consistency: DxfBlockNameConsistencyDirectory,
    keys: Box<[DxfBlockNameIndexKey]>,
    matches: Box<[DxfBlockNameIndexMatch]>,
}

impl DxfBlockNameIndexDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let consistency = document.block_name_consistency_directory(cancellation)?;
        if consistency.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: consistency.source_id(),
            });
        }

        let mut pending = Vec::new();
        let mut representatives = Vec::new();
        for entry in consistency.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if entry.state() != DxfBlockNameConsistencyState::Matched {
                continue;
            }
            let semantics = consistency
                .semantic_directory()
                .semantics_for_entry(entry.record())?
                .ok_or_else(invalid_internal_data)?;
            let name = semantics
                .primary_name()
                .value()
                .copied()
                .ok_or_else(invalid_internal_data)?;
            if name.source_id() != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed: name.source_id(),
                });
            }

            let digest = digest_span(document, name.value_span(), cancellation)?;
            let exact_group_ordinal = exact_group_ordinal(
                document,
                &mut representatives,
                digest,
                name.value_span(),
                cancellation,
            )?;
            pending.try_reserve(1).map_err(|_| out_of_memory())?;
            pending.push(PendingMatch {
                key: DxfBlockNameIndexKey {
                    digest,
                    exact_group_ordinal,
                },
                indexed: DxfBlockNameIndexMatch {
                    record: entry.record(),
                    name,
                },
            });
        }

        ensure_not_cancelled(cancellation)?;
        pending.sort_unstable_by_key(|candidate| {
            (
                candidate.key,
                candidate
                    .indexed
                    .record()
                    .definition()
                    .block_record()
                    .ordinal(),
            )
        });
        let mut keys = Vec::new();
        let mut matches = Vec::new();
        keys.try_reserve(pending.len())
            .map_err(|_| out_of_memory())?;
        matches
            .try_reserve(pending.len())
            .map_err(|_| out_of_memory())?;
        for candidate in pending {
            keys.push(candidate.key);
            matches.push(candidate.indexed);
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            consistency,
            keys: keys.into_boxed_slice(),
            matches: matches.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn consistency_directory(&self) -> &DxfBlockNameConsistencyDirectory {
        &self.consistency
    }

    /// Returns every lookup-admitted BLOCK in digest/group/record order.
    #[must_use]
    pub fn matches(&self) -> &[DxfBlockNameIndexMatch] {
        &self.matches
    }

    pub fn matches_for_exact_name<'a>(
        &'a self,
        document: DxfRawDocumentView<'_>,
        exact_name: &[u8],
        cancellation: &DxfCancellationToken,
    ) -> Result<&'a [DxfBlockNameIndexMatch], DxfError> {
        self.ensure_document(document)?;
        ensure_not_cancelled(cancellation)?;
        let digest = digest_bytes(exact_name);
        let start = self.keys.partition_point(|key| key.digest < digest);
        let end = self.keys.partition_point(|key| key.digest <= digest);
        let mut cursor = start;
        while cursor < end {
            ensure_not_cancelled(cancellation)?;
            let key = *self.keys.get(cursor).ok_or_else(invalid_internal_data)?;
            let relative_end = self.keys[cursor..end].partition_point(|candidate| {
                candidate.exact_group_ordinal == key.exact_group_ordinal
            });
            let group_end = cursor
                .checked_add(relative_end)
                .ok_or_else(invalid_internal_data)?;
            let representative = self
                .matches
                .get(cursor)
                .copied()
                .ok_or_else(invalid_internal_data)?;
            if span_equals_bytes(
                document,
                representative.name().value_span(),
                exact_name,
                cancellation,
            )? {
                return self
                    .matches
                    .get(cursor..group_end)
                    .ok_or_else(invalid_internal_data);
            }
            if group_end <= cursor {
                return Err(invalid_internal_data());
            }
            cursor = group_end;
        }
        ensure_not_cancelled(cancellation)?;
        Ok(self.matches.get(0..0).map_or(&[], |matches| matches))
    }

    pub fn lookup<'a>(
        &'a self,
        document: DxfRawDocumentView<'_>,
        exact_name: &[u8],
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockNameIndexLookup<'a>, DxfError> {
        Ok(
            match self.matches_for_exact_name(document, exact_name, cancellation)? {
                [] => DxfBlockNameIndexLookup::Missing,
                [indexed] => DxfBlockNameIndexLookup::Unique(*indexed),
                matches => DxfBlockNameIndexLookup::Ambiguous(matches),
            },
        )
    }

    fn ensure_document(&self, document: DxfRawDocumentView<'_>) -> Result<(), DxfError> {
        if document.source_id() == self.source_id {
            Ok(())
        } else {
            Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: document.source_id(),
            })
        }
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_name_index_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockNameIndexDirectory, DxfError> {
        DxfBlockNameIndexDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_name_index_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockNameIndexDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_name_index_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_name_index_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockNameIndexDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_name_index_directory(cancellation)
    }
}

fn exact_group_ordinal(
    document: DxfRawDocumentView<'_>,
    representatives: &mut Vec<ExactGroupRepresentative>,
    digest: NameDigest,
    span: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<u32, DxfError> {
    for representative in representatives.iter().copied() {
        if representative.digest == digest
            && raw_spans_equal(document, representative.span, span, cancellation)?
        {
            return Ok(representative.ordinal);
        }
    }
    let ordinal = compact_len(representatives.len())?;
    representatives
        .try_reserve(1)
        .map_err(|_| out_of_memory())?;
    representatives.push(ExactGroupRepresentative {
        digest,
        span,
        ordinal,
    });
    Ok(ordinal)
}

fn digest_span(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<NameDigest, DxfError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; NAME_IO_CHUNK_BYTES];
    let mut consumed = 0_u64;
    while consumed < span.len() {
        ensure_not_cancelled(cancellation)?;
        let chunk_len_u64 = (span.len() - consumed).min(NAME_IO_CHUNK_BYTES as u64);
        let chunk_len = usize::try_from(chunk_len_u64).map_err(|_| invalid_internal_data())?;
        let chunk = chunk_span(span, consumed, chunk_len_u64)?;
        document.read_span(chunk, &mut buffer[..chunk_len])?;
        hasher.update(&buffer[..chunk_len]);
        consumed = consumed
            .checked_add(chunk_len_u64)
            .ok_or_else(invalid_internal_data)?;
    }
    ensure_not_cancelled(cancellation)?;
    Ok(hasher.finalize().into())
}

fn digest_bytes(bytes: &[u8]) -> NameDigest {
    Sha256::digest(bytes).into()
}

fn raw_spans_equal(
    document: DxfRawDocumentView<'_>,
    left: ByteSpan,
    right: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<bool, DxfError> {
    if left.len() != right.len() {
        return Ok(false);
    }
    let mut left_bytes = [0_u8; NAME_IO_CHUNK_BYTES];
    let mut right_bytes = [0_u8; NAME_IO_CHUNK_BYTES];
    let mut consumed = 0_u64;
    while consumed < left.len() {
        ensure_not_cancelled(cancellation)?;
        let chunk_len_u64 = (left.len() - consumed).min(NAME_IO_CHUNK_BYTES as u64);
        let chunk_len = usize::try_from(chunk_len_u64).map_err(|_| invalid_internal_data())?;
        document.read_span(
            chunk_span(left, consumed, chunk_len_u64)?,
            &mut left_bytes[..chunk_len],
        )?;
        document.read_span(
            chunk_span(right, consumed, chunk_len_u64)?,
            &mut right_bytes[..chunk_len],
        )?;
        if left_bytes[..chunk_len] != right_bytes[..chunk_len] {
            return Ok(false);
        }
        consumed = consumed
            .checked_add(chunk_len_u64)
            .ok_or_else(invalid_internal_data)?;
    }
    ensure_not_cancelled(cancellation)?;
    Ok(true)
}

fn span_equals_bytes(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    expected: &[u8],
    cancellation: &DxfCancellationToken,
) -> Result<bool, DxfError> {
    if span.len() != expected.len() as u64 {
        return Ok(false);
    }
    let mut buffer = [0_u8; NAME_IO_CHUNK_BYTES];
    let mut consumed = 0_usize;
    while consumed < expected.len() {
        ensure_not_cancelled(cancellation)?;
        let chunk_len = (expected.len() - consumed).min(NAME_IO_CHUNK_BYTES);
        let chunk_len_u64 = u64::try_from(chunk_len).map_err(|_| invalid_internal_data())?;
        document.read_span(
            chunk_span(span, consumed as u64, chunk_len_u64)?,
            &mut buffer[..chunk_len],
        )?;
        let end = consumed
            .checked_add(chunk_len)
            .ok_or_else(invalid_internal_data)?;
        if buffer[..chunk_len] != expected[consumed..end] {
            return Ok(false);
        }
        consumed = end;
    }
    ensure_not_cancelled(cancellation)?;
    Ok(true)
}

fn chunk_span(base: ByteSpan, offset: u64, len: u64) -> Result<ByteSpan, DxfError> {
    let start = base
        .start()
        .checked_add(offset)
        .ok_or_else(invalid_internal_data)?;
    let span = ByteSpan::from_start_and_len(start, len).ok_or_else(invalid_internal_data)?;
    if span.end() > base.end() {
        return Err(invalid_internal_data());
    }
    Ok(span)
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

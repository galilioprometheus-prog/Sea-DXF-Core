//! Shared bounded comparison and hashing for authoritative source spans.

use std::io;

use sha2::{Digest, Sha256};

use crate::{ByteSpan, DxfCancellationToken, DxfError, DxfIoOperation, DxfRawDocumentView};

const IO_CHUNK_BYTES: usize = 4 * 1024;

pub(crate) fn sha256_span(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<[u8; 32], DxfError> {
    let mut hasher = Sha256::new();
    visit_span(document, span, cancellation, |bytes| {
        hasher.update(bytes);
        Ok(true)
    })?;
    Ok(hasher.finalize().into())
}

pub(crate) fn spans_equal(
    document: DxfRawDocumentView<'_>,
    left: ByteSpan,
    right: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<bool, DxfError> {
    spans_equal_across_documents(document, left, document, right, cancellation)
}

pub(crate) fn spans_equal_across_documents(
    left_document: DxfRawDocumentView<'_>,
    left: ByteSpan,
    right_document: DxfRawDocumentView<'_>,
    right: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<bool, DxfError> {
    if left.len() != right.len() {
        return Ok(false);
    }
    let mut right_bytes = [0_u8; IO_CHUNK_BYTES];
    let mut consumed = 0_u64;
    visit_span(left_document, left, cancellation, |left_bytes| {
        let chunk_len = left_bytes.len();
        let chunk_len_u64 = u64::try_from(chunk_len).map_err(|_| invalid_internal_data())?;
        right_document.read_span(
            chunk_span(right, consumed, chunk_len_u64)?,
            &mut right_bytes[..chunk_len],
        )?;
        consumed = consumed
            .checked_add(chunk_len_u64)
            .ok_or_else(invalid_internal_data)?;
        Ok(left_bytes == &right_bytes[..chunk_len])
    })
}

pub(crate) fn span_equals_bytes(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    expected: &[u8],
    cancellation: &DxfCancellationToken,
) -> Result<bool, DxfError> {
    if span.len() != expected.len() as u64 {
        return Ok(false);
    }
    let mut consumed = 0_usize;
    visit_span(document, span, cancellation, |actual| {
        let end = consumed
            .checked_add(actual.len())
            .ok_or_else(invalid_internal_data)?;
        let equal = actual == &expected[consumed..end];
        consumed = end;
        Ok(equal)
    })
}

fn visit_span(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    cancellation: &DxfCancellationToken,
    mut visitor: impl FnMut(&[u8]) -> Result<bool, DxfError>,
) -> Result<bool, DxfError> {
    let mut buffer = [0_u8; IO_CHUNK_BYTES];
    let mut consumed = 0_u64;
    while consumed < span.len() {
        ensure_not_cancelled(cancellation)?;
        let chunk_len_u64 = (span.len() - consumed).min(IO_CHUNK_BYTES as u64);
        let chunk_len = usize::try_from(chunk_len_u64).map_err(|_| invalid_internal_data())?;
        document.read_span(
            chunk_span(span, consumed, chunk_len_u64)?,
            &mut buffer[..chunk_len],
        )?;
        if !visitor(&buffer[..chunk_len])? {
            return Ok(false);
        }
        consumed = consumed
            .checked_add(chunk_len_u64)
            .ok_or_else(invalid_internal_data)?;
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

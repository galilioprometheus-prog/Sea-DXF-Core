//! Streaming post-image verification and executable inverse-plan materialization.

use std::io;

use crate::{
    ByteSpan, DxfCancellationToken, DxfError, DxfIoOperation, DxfRawDocumentView,
    DxfResourceProfile, DxfTransactionPlan, DxfTransactionPlanBuilder,
};

const VERIFY_CHUNK_BYTES: usize = 4 * 1024;

impl DxfTransactionPlan {
    pub fn materialize_inverse_plan(
        &self,
        source_document: DxfRawDocumentView<'_>,
        post_image_document: DxfRawDocumentView<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTransactionPlan, DxfError> {
        ensure_not_cancelled(cancellation)?;
        self.validate_source_precondition(source_document)?;
        if post_image_document.source_len() != self.projected_len() {
            return Err(DxfError::TransactionPostImageLengthMismatch {
                expected: self.projected_len(),
                observed: post_image_document.source_len(),
            });
        }
        if post_image_document.format() != self.format() {
            return Err(DxfError::TransactionPostImageMismatch {
                span: ByteSpan::new(0, 0).ok_or_else(invalid_internal_data)?,
            });
        }
        verify_post_image(self, source_document, post_image_document, cancellation)?;

        let mut inverse_builder = post_image_document.transaction_plan_builder(profile)?;
        append_inverse_operations(self, &mut inverse_builder, cancellation)?;
        let inverse = inverse_builder.finish(cancellation)?;
        if inverse.projected_len() != self.source_len() {
            return Err(invalid_internal_data());
        }
        Ok(inverse)
    }
}

fn verify_post_image(
    plan: &DxfTransactionPlan,
    source: DxfRawDocumentView<'_>,
    post_image: DxfRawDocumentView<'_>,
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    let mut source_cursor = 0_u64;
    let mut post_cursor = 0_u64;
    for patch in plan.patches().iter().copied() {
        ensure_not_cancelled(cancellation)?;
        let unchanged_len = patch
            .source_span()
            .start()
            .checked_sub(source_cursor)
            .ok_or_else(invalid_internal_data)?;
        compare_document_ranges(
            source,
            source_cursor,
            post_image,
            post_cursor,
            unchanged_len,
            cancellation,
        )?;
        post_cursor = post_cursor
            .checked_add(unchanged_len)
            .ok_or(DxfError::OffsetOverflow {
                offset: post_cursor,
                requested: unchanged_len,
            })?;
        let replacement = plan
            .replacement_bytes_for_patch_ordinal(patch.ordinal())
            .ok_or_else(invalid_internal_data)?;
        compare_bytes_to_document(replacement, post_image, post_cursor, cancellation)?;
        post_cursor = post_cursor.checked_add(bytes_len_u64(replacement)?).ok_or(
            DxfError::OffsetOverflow {
                offset: post_cursor,
                requested: bytes_len_u64(replacement)?,
            },
        )?;
        source_cursor = patch.source_span().end();
    }
    let trailing_len = plan
        .source_len()
        .checked_sub(source_cursor)
        .ok_or_else(invalid_internal_data)?;
    compare_document_ranges(
        source,
        source_cursor,
        post_image,
        post_cursor,
        trailing_len,
        cancellation,
    )?;
    post_cursor = post_cursor
        .checked_add(trailing_len)
        .ok_or(DxfError::OffsetOverflow {
            offset: post_cursor,
            requested: trailing_len,
        })?;
    if post_cursor != plan.projected_len() {
        return Err(invalid_internal_data());
    }
    ensure_not_cancelled(cancellation)
}

fn append_inverse_operations(
    plan: &DxfTransactionPlan,
    builder: &mut DxfTransactionPlanBuilder<'_>,
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    let mut source_cursor = 0_u64;
    let mut post_cursor = 0_u64;
    let mut insertion_offset = None;
    let mut insertion_fragments: Vec<&[u8]> = Vec::new();
    for patch in plan.patches().iter().copied() {
        ensure_not_cancelled(cancellation)?;
        let unchanged_len = patch
            .source_span()
            .start()
            .checked_sub(source_cursor)
            .ok_or_else(invalid_internal_data)?;
        post_cursor = post_cursor
            .checked_add(unchanged_len)
            .ok_or(DxfError::OffsetOverflow {
                offset: post_cursor,
                requested: unchanged_len,
            })?;
        let replacement = plan
            .replacement_bytes_for_patch_ordinal(patch.ordinal())
            .ok_or_else(invalid_internal_data)?;
        let original = plan
            .inverse_bytes_for_patch_ordinal(patch.ordinal())
            .ok_or_else(invalid_internal_data)?;
        let replacement_len = bytes_len_u64(replacement)?;
        let inverse_source_span = ByteSpan::from_start_and_len(post_cursor, replacement_len)
            .ok_or(DxfError::OffsetOverflow {
                offset: post_cursor,
                requested: replacement_len,
            })?;
        if inverse_source_span.is_empty() {
            if insertion_offset != Some(post_cursor) {
                flush_insertion(
                    builder,
                    &mut insertion_offset,
                    &mut insertion_fragments,
                    cancellation,
                )?;
                insertion_offset = Some(post_cursor);
            }
            insertion_fragments
                .try_reserve(1)
                .map_err(|_| out_of_memory())?;
            insertion_fragments.push(original);
        } else {
            flush_insertion(
                builder,
                &mut insertion_offset,
                &mut insertion_fragments,
                cancellation,
            )?;
            builder.replace_raw_span_fragments(inverse_source_span, &[original], cancellation)?;
        }
        post_cursor = post_cursor
            .checked_add(replacement_len)
            .ok_or(DxfError::OffsetOverflow {
                offset: post_cursor,
                requested: replacement_len,
            })?;
        source_cursor = patch.source_span().end();
    }
    flush_insertion(
        builder,
        &mut insertion_offset,
        &mut insertion_fragments,
        cancellation,
    )
}

fn flush_insertion(
    builder: &mut DxfTransactionPlanBuilder<'_>,
    insertion_offset: &mut Option<u64>,
    fragments: &mut Vec<&[u8]>,
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    if let Some(offset) = insertion_offset.take() {
        let source_span = ByteSpan::new(offset, offset).ok_or_else(invalid_internal_data)?;
        builder.replace_raw_span_fragments(source_span, fragments, cancellation)?;
    }
    fragments.clear();
    Ok(())
}

fn compare_document_ranges(
    expected_document: DxfRawDocumentView<'_>,
    expected_start: u64,
    observed_document: DxfRawDocumentView<'_>,
    observed_start: u64,
    len: u64,
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    let mut expected_buffer = [0_u8; VERIFY_CHUNK_BYTES];
    let mut observed_buffer = [0_u8; VERIFY_CHUNK_BYTES];
    let mut compared = 0_u64;
    while compared < len {
        ensure_not_cancelled(cancellation)?;
        let chunk_len_u64 = (len - compared).min(VERIFY_CHUNK_BYTES as u64);
        let chunk_len = usize::try_from(chunk_len_u64).map_err(|_| invalid_internal_data())?;
        let expected_offset =
            expected_start
                .checked_add(compared)
                .ok_or(DxfError::OffsetOverflow {
                    offset: expected_start,
                    requested: compared,
                })?;
        let observed_offset =
            observed_start
                .checked_add(compared)
                .ok_or(DxfError::OffsetOverflow {
                    offset: observed_start,
                    requested: compared,
                })?;
        expected_document.read_span(
            span(expected_offset, chunk_len_u64)?,
            &mut expected_buffer[..chunk_len],
        )?;
        observed_document.read_span(
            span(observed_offset, chunk_len_u64)?,
            &mut observed_buffer[..chunk_len],
        )?;
        if expected_buffer[..chunk_len] != observed_buffer[..chunk_len] {
            return Err(mismatch(
                observed_offset,
                first_difference(&expected_buffer[..chunk_len], &observed_buffer[..chunk_len])?,
            )?);
        }
        compared = compared
            .checked_add(chunk_len_u64)
            .ok_or_else(invalid_internal_data)?;
    }
    Ok(())
}

fn compare_bytes_to_document(
    expected: &[u8],
    observed_document: DxfRawDocumentView<'_>,
    observed_start: u64,
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    let mut observed_buffer = [0_u8; VERIFY_CHUNK_BYTES];
    let mut compared = 0_usize;
    while compared < expected.len() {
        ensure_not_cancelled(cancellation)?;
        let chunk_len = (expected.len() - compared).min(VERIFY_CHUNK_BYTES);
        let compared_u64 = u64::try_from(compared).map_err(|_| invalid_internal_data())?;
        let observed_offset =
            observed_start
                .checked_add(compared_u64)
                .ok_or(DxfError::OffsetOverflow {
                    offset: observed_start,
                    requested: compared_u64,
                })?;
        observed_document.read_span(
            span(
                observed_offset,
                u64::try_from(chunk_len).map_err(|_| invalid_internal_data())?,
            )?,
            &mut observed_buffer[..chunk_len],
        )?;
        if expected[compared..compared + chunk_len] != observed_buffer[..chunk_len] {
            return Err(mismatch(
                observed_offset,
                first_difference(
                    &expected[compared..compared + chunk_len],
                    &observed_buffer[..chunk_len],
                )?,
            )?);
        }
        compared = compared
            .checked_add(chunk_len)
            .ok_or_else(invalid_internal_data)?;
    }
    Ok(())
}

fn first_difference(expected: &[u8], observed: &[u8]) -> Result<u64, DxfError> {
    expected
        .iter()
        .zip(observed)
        .position(|(expected, observed)| expected != observed)
        .and_then(|offset| u64::try_from(offset).ok())
        .ok_or_else(invalid_internal_data)
}

fn mismatch(chunk_start: u64, relative_offset: u64) -> Result<DxfError, DxfError> {
    let offset = chunk_start
        .checked_add(relative_offset)
        .ok_or(DxfError::OffsetOverflow {
            offset: chunk_start,
            requested: relative_offset,
        })?;
    let mismatch_span =
        ByteSpan::from_start_and_len(offset, 1).ok_or(DxfError::OffsetOverflow {
            offset,
            requested: 1,
        })?;
    Ok(DxfError::TransactionPostImageMismatch {
        span: mismatch_span,
    })
}

fn span(start: u64, len: u64) -> Result<ByteSpan, DxfError> {
    ByteSpan::from_start_and_len(start, len).ok_or(DxfError::OffsetOverflow {
        offset: start,
        requested: len,
    })
}

fn bytes_len_u64(bytes: &[u8]) -> Result<u64, DxfError> {
    u64::try_from(bytes.len()).map_err(|_| invalid_internal_data())
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

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

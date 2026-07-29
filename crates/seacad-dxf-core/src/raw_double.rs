//! Shared format-neutral decoding for one raw DXF binary64 group.

use std::io;

use crate::{
    ByteSpan, DxfAsciiNumericIssue, DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup, ascii_group::trim_horizontal_ascii,
    ascii_numeric::parse_f64 as parse_ascii_f64,
};

const STACK_ASCII_NUMERIC_BYTES: usize = 128;
const SOURCE_READ_CHUNK_BYTES: usize = 8 * 1024;

pub(crate) fn decode_raw_double(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfDouble, DxfAsciiNumericIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    match document.format() {
        DxfRawDocumentFormat::Ascii => {
            let span = group.value_payload_span();
            let length = usize::try_from(span.len()).map_err(|_| invalid_internal_data())?;
            if length <= STACK_ASCII_NUMERIC_BYTES {
                let mut bytes = [0_u8; STACK_ASCII_NUMERIC_BYTES];
                read_span_cancelled(document, span, &mut bytes[..length], cancellation)?;
                return parse_ascii_double(&bytes[..length]);
            }

            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(length)
                .map_err(|_| out_of_memory())?;
            bytes.resize(length, 0);
            read_span_cancelled(document, span, &mut bytes, cancellation)?;
            parse_ascii_double(&bytes)
        }
        DxfRawDocumentFormat::Binary => {
            if group.value_payload_span().len() != 8 {
                return Err(invalid_internal_data());
            }
            let mut bytes = [0_u8; 8];
            document.read_span(group.value_payload_span(), &mut bytes)?;
            ensure_not_cancelled(cancellation)?;
            Ok(Ok(DxfDouble::from_bits(u64::from_le_bytes(bytes))))
        }
    }
}

fn parse_ascii_double(bytes: &[u8]) -> Result<Result<DxfDouble, DxfAsciiNumericIssue>, DxfError> {
    let token = trim_horizontal_ascii(bytes).ok_or_else(invalid_internal_data)?;
    Ok(parse_ascii_f64(token).map(DxfDouble::from_f64))
}

fn read_span_cancelled(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    destination: &mut [u8],
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    if destination.len() as u64 != span.len() {
        return Err(invalid_internal_data());
    }
    let mut consumed = 0_usize;
    while consumed < destination.len() {
        ensure_not_cancelled(cancellation)?;
        let chunk_len = (destination.len() - consumed).min(SOURCE_READ_CHUNK_BYTES);
        let start = span
            .start()
            .checked_add(consumed as u64)
            .ok_or_else(invalid_internal_data)?;
        let chunk_span = ByteSpan::from_start_and_len(start, chunk_len as u64)
            .ok_or_else(invalid_internal_data)?;
        let end = consumed
            .checked_add(chunk_len)
            .ok_or_else(invalid_internal_data)?;
        document.read_span(chunk_span, &mut destination[consumed..end])?;
        consumed = end;
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

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

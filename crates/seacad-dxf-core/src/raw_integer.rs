//! Shared format-neutral decoding for raw signed integer groups.

use crate::{
    DxfAsciiNumericIssue, DxfCancellationToken, DxfError, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfRawGroup,
    ascii_numeric::{parse_i16 as parse_ascii_i16, parse_i32 as parse_ascii_i32},
    raw_double::{decode_raw_ascii_numeric, read_raw_fixed_payload},
};

pub(crate) fn decode_raw_i16(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Result<i16, DxfAsciiNumericIssue>, DxfError> {
    match document.format() {
        DxfRawDocumentFormat::Ascii => decode_raw_ascii_numeric(
            document,
            group.value_payload_span(),
            cancellation,
            parse_ascii_i16,
        ),
        DxfRawDocumentFormat::Binary => read_raw_fixed_payload(document, group, cancellation)
            .map(|bytes| Ok(i16::from_le_bytes(bytes))),
    }
}

pub(crate) fn decode_raw_i32(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Result<i32, DxfAsciiNumericIssue>, DxfError> {
    match document.format() {
        DxfRawDocumentFormat::Ascii => decode_raw_ascii_numeric(
            document,
            group.value_payload_span(),
            cancellation,
            parse_ascii_i32,
        ),
        DxfRawDocumentFormat::Binary => read_raw_fixed_payload(document, group, cancellation)
            .map(|bytes| Ok(i32::from_le_bytes(bytes))),
    }
}

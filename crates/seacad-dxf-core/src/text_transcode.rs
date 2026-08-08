//! Dual-document, replacement-free, round-trip-verified text transcoding.

use std::io;

use crate::{
    ByteSpan, DxfCancellationToken, DxfError, DxfIoOperation, DxfRawDocumentView, DxfResource,
    DxfResourceProfile, DxfSourceId, DxfTextDecodeStatus, DxfTextEncodeStatus,
    DxfTextEncodingResolution,
};

/// Typed reason why one exact source value cannot be transcoded.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextTranscodeIssue {
    SourceEncodingUnavailable {
        resolution: DxfTextEncodingResolution,
    },
    DestinationEncodingUnavailable {
        resolution: DxfTextEncodingResolution,
    },
    SourceDecode {
        status: DxfTextDecodeStatus,
    },
    DestinationEncode {
        status: DxfTextEncodeStatus,
    },
    DestinationRoundTripDecode {
        status: DxfTextDecodeStatus,
    },
    DestinationRoundTripMismatch {
        utf8_byte_count: u64,
        round_trip_byte_count: u64,
    },
}

/// Compact provenance for one replacement-free, round-trip-verified result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextTranscodeReceipt {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    source_span: ByteSpan,
    source_encoding: DxfTextEncodingResolution,
    destination_encoding: DxfTextEncodingResolution,
    source_byte_count: u64,
    utf8_byte_count: u64,
    encoded_byte_count: u64,
}

impl DxfTextTranscodeReceipt {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn source_span(self) -> ByteSpan {
        self.source_span
    }

    #[must_use]
    pub const fn source_encoding(self) -> DxfTextEncodingResolution {
        self.source_encoding
    }

    #[must_use]
    pub const fn destination_encoding(self) -> DxfTextEncodingResolution {
        self.destination_encoding
    }

    #[must_use]
    pub const fn source_byte_count(self) -> u64 {
        self.source_byte_count
    }

    #[must_use]
    pub const fn utf8_byte_count(self) -> u64 {
        self.utf8_byte_count
    }

    #[must_use]
    pub const fn encoded_byte_count(self) -> u64 {
        self.encoded_byte_count
    }
}

/// One exact source span transcoded for an independently parsed destination.
pub struct DxfTextTranscodePlan {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    source_span: ByteSpan,
    source_encoding: DxfTextEncodingResolution,
    destination_encoding: DxfTextEncodingResolution,
    source_byte_count: u64,
    utf8_byte_count: u64,
    encoded_byte_count: u64,
    encoded: Box<[u8]>,
}

impl DxfTextTranscodePlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn source_span(&self) -> ByteSpan {
        self.source_span
    }

    #[must_use]
    pub const fn source_encoding(&self) -> DxfTextEncodingResolution {
        self.source_encoding
    }

    #[must_use]
    pub const fn destination_encoding(&self) -> DxfTextEncodingResolution {
        self.destination_encoding
    }

    #[must_use]
    pub const fn source_byte_count(&self) -> u64 {
        self.source_byte_count
    }

    #[must_use]
    pub const fn utf8_byte_count(&self) -> u64 {
        self.utf8_byte_count
    }

    #[must_use]
    pub const fn encoded_byte_count(&self) -> u64 {
        self.encoded_byte_count
    }

    #[must_use]
    pub const fn receipt(&self) -> DxfTextTranscodeReceipt {
        DxfTextTranscodeReceipt {
            source_id: self.source_id,
            destination_id: self.destination_id,
            source_span: self.source_span,
            source_encoding: self.source_encoding,
            destination_encoding: self.destination_encoding,
            source_byte_count: self.source_byte_count,
            utf8_byte_count: self.utf8_byte_count,
            encoded_byte_count: self.encoded_byte_count,
        }
    }

    #[must_use]
    pub fn encoded_bytes(&self) -> &[u8] {
        &self.encoded
    }
}

impl std::fmt::Debug for DxfTextTranscodePlan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DxfTextTranscodePlan")
            .field("source_id", &self.source_id)
            .field("destination_id", &self.destination_id)
            .field("source_span", &self.source_span)
            .field("source_encoding", &self.source_encoding)
            .field("destination_encoding", &self.destination_encoding)
            .field("source_byte_count", &self.source_byte_count)
            .field("utf8_byte_count", &self.utf8_byte_count)
            .field("encoded_byte_count", &self.encoded_byte_count)
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    /// Transcodes one exact source span into the destination document's
    /// reviewed text storage without replacement.
    pub fn transcode_text_span_to(
        self,
        source_span: ByteSpan,
        destination: DxfRawDocumentView<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfTextTranscodePlan, DxfTextTranscodeIssue>, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let limit = profile.limits().max_value_bytes();
        let source_len = bounded_len(source_span.len(), limit)?;
        let mut source_bytes = zeroed(source_len)?;
        self.read_span(source_span, &mut source_bytes)?;
        ensure_not_cancelled(cancellation)?;

        let source_encoding = self.text_encoding_report().resolution();
        let Some(source_decoder) = source_encoding.decoder() else {
            return Ok(Err(DxfTextTranscodeIssue::SourceEncodingUnavailable {
                resolution: source_encoding,
            }));
        };
        let decoded_capacity = decode_capacity(source_len, limit)?;
        let mut decoded = zeroed(decoded_capacity)?;
        let decoded_result =
            source_decoder.decode_complete_to_utf8_without_replacement(&source_bytes, &mut decoded);
        match decoded_result.status() {
            DxfTextDecodeStatus::Complete => decoded.truncate(decoded_result.written()),
            DxfTextDecodeStatus::OutputFull => return Err(value_limit(limit)),
            status @ DxfTextDecodeStatus::Malformed { .. } => {
                return Ok(Err(DxfTextTranscodeIssue::SourceDecode { status }));
            }
        }
        let decoded_text = std::str::from_utf8(&decoded).map_err(|_| invalid_internal_data())?;

        let destination_encoding = destination.text_encoding_report().resolution();
        let Some(destination_encoder) = destination_encoding.encoder() else {
            return Ok(Err(DxfTextTranscodeIssue::DestinationEncodingUnavailable {
                resolution: destination_encoding,
            }));
        };
        let mut encoded = zeroed(decoded.len())?;
        let encoded_result = destination_encoder
            .encode_complete_from_utf8_without_replacement(decoded_text, &mut encoded);
        match encoded_result.status() {
            DxfTextEncodeStatus::Complete => encoded.truncate(encoded_result.written()),
            DxfTextEncodeStatus::OutputFull => return Err(value_limit(limit)),
            status
            @ (DxfTextEncodeStatus::Unmappable { .. } | DxfTextEncodeStatus::Unavailable) => {
                return Ok(Err(DxfTextTranscodeIssue::DestinationEncode { status }));
            }
        }
        ensure_not_cancelled(cancellation)?;

        let destination_decoder = destination_encoding
            .decoder()
            .ok_or_else(invalid_internal_data)?;
        // `encoding_rs` may need one scalar of terminal slack even when the
        // final UTF-8 output is exactly `decoded.len()`. This bounded scratch
        // overhead does not increase the accepted value-byte count.
        let round_trip_capacity = decoded
            .len()
            .checked_add(4)
            .ok_or_else(invalid_internal_data)?;
        let mut round_trip = zeroed(round_trip_capacity)?;
        let round_trip_result = destination_decoder
            .decode_complete_to_utf8_without_replacement(&encoded, &mut round_trip);
        match round_trip_result.status() {
            DxfTextDecodeStatus::Complete => round_trip.truncate(round_trip_result.written()),
            status => {
                return Ok(Err(DxfTextTranscodeIssue::DestinationRoundTripDecode {
                    status,
                }));
            }
        }
        if round_trip != decoded {
            return Ok(Err(DxfTextTranscodeIssue::DestinationRoundTripMismatch {
                utf8_byte_count: compact_len(decoded.len())?,
                round_trip_byte_count: compact_len(round_trip.len())?,
            }));
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfTextTranscodePlan {
            source_id: self.source_id(),
            destination_id: destination.source_id(),
            source_span,
            source_encoding,
            destination_encoding,
            source_byte_count: source_span.len(),
            utf8_byte_count: compact_len(decoded.len())?,
            encoded_byte_count: compact_len(encoded.len())?,
            encoded: encoded.into_boxed_slice(),
        }))
    }
}

fn decode_capacity(source_len: usize, limit: u64) -> Result<usize, DxfError> {
    let expanded = source_len
        .checked_mul(4)
        .ok_or_else(invalid_internal_data)?;
    let limit = usize::try_from(limit).map_err(|_| invalid_internal_data())?;
    Ok(expanded.max(4).min(limit))
}

fn bounded_len(len: u64, limit: u64) -> Result<usize, DxfError> {
    if len > limit {
        return Err(DxfError::resource_limit(
            DxfResource::ValueBytes,
            limit,
            len,
        ));
    }
    usize::try_from(len).map_err(|_| invalid_internal_data())
}

fn zeroed(len: usize) -> Result<Vec<u8>, DxfError> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(len).map_err(|_| out_of_memory())?;
    bytes.resize(len, 0);
    Ok(bytes)
}

fn compact_len(len: usize) -> Result<u64, DxfError> {
    u64::try_from(len).map_err(|_| invalid_internal_data())
}

fn value_limit(limit: u64) -> DxfError {
    DxfError::resource_limit(DxfResource::ValueBytes, limit, limit.saturating_add(1))
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

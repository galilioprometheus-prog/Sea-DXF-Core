//! Source-anchored, allocation-free decoding of raw ASCII group values.

use std::io;

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfError, DxfIoOperation, DxfSourceId, DxfTextDecodeResult,
    DxfTextDecodeStatus, DxfTextEncodingResolution, text_decoder::DxfTextDecoderSession,
};

const SOURCE_CHUNK_BYTES: usize = 4 * 1024;

/// Provenance and terminal state for one raw group-value decode request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextValueDecodeReceipt {
    source_id: DxfSourceId,
    group_occurrence: u64,
    value_span: ByteSpan,
    encoding: DxfTextEncodingResolution,
    decode_result: Option<DxfTextDecodeResult>,
}

impl DxfTextValueDecodeReceipt {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn group_occurrence(self) -> u64 {
        self.group_occurrence
    }

    #[must_use]
    pub const fn value_span(self) -> ByteSpan {
        self.value_span
    }

    #[must_use]
    pub const fn encoding(self) -> DxfTextEncodingResolution {
        self.encoding
    }

    /// Returns `None` only when the document encoding is unavailable.
    #[must_use]
    pub const fn decode_result(self) -> Option<DxfTextDecodeResult> {
        self.decode_result
    }
}

impl DxfAsciiRawDocument<'_> {
    /// Decodes one group value selected by this document's occurrence index.
    ///
    /// The destination is caller-owned. Only its prefix through
    /// `decode_result().written()` is defined output. On `OutputFull`, retry
    /// the same occurrence from the beginning with a larger destination.
    pub fn decode_group_value_to_utf8_without_replacement(
        &self,
        group_occurrence: u64,
        destination: &mut [u8],
    ) -> Result<DxfTextValueDecodeReceipt, DxfError> {
        let index = usize::try_from(group_occurrence).map_err(|_| invalid_source_data())?;
        let group = self
            .groups()
            .get(index)
            .copied()
            .ok_or_else(invalid_source_data)?;
        if group.occurrence() != group_occurrence
            || self.text_encoding_report().source_id() != self.source_id()
        {
            return Err(invalid_source_data());
        }

        let value_span = group.value_content_span();
        let encoding = self.text_encoding_report().resolution();
        let Some(decoder) = encoding.decoder() else {
            return Ok(DxfTextValueDecodeReceipt {
                source_id: self.source_id(),
                group_occurrence,
                value_span,
                encoding,
                decode_result: None,
            });
        };

        let decode_result = if value_span.is_empty() {
            DxfTextDecodeResult::from_parts(DxfTextDecodeStatus::Complete, 0, 0)
        } else if destination.is_empty() {
            DxfTextDecodeResult::from_parts(DxfTextDecodeStatus::OutputFull, 0, 0)
        } else {
            decode_span(self, value_span, decoder, destination)?
        };
        Ok(DxfTextValueDecodeReceipt {
            source_id: self.source_id(),
            group_occurrence,
            value_span,
            encoding,
            decode_result: Some(decode_result),
        })
    }
}

fn decode_span(
    document: &DxfAsciiRawDocument<'_>,
    span: ByteSpan,
    decoder: crate::DxfTextDecoder,
    destination: &mut [u8],
) -> Result<DxfTextDecodeResult, DxfError> {
    let mut session = DxfTextDecoderSession::new(decoder);
    let mut source_buffer = [0_u8; SOURCE_CHUNK_BYTES];
    let mut short_output = [0_u8; 4];
    let mut offset = span.start();
    let mut total_read = 0_usize;
    let mut total_written = 0_usize;

    while offset < span.end() {
        let remaining = span.end() - offset;
        let chunk_len_u64 = remaining.min(SOURCE_CHUNK_BYTES as u64);
        let chunk_len = usize::try_from(chunk_len_u64).map_err(|_| invalid_source_data())?;
        let next_offset = offset
            .checked_add(chunk_len_u64)
            .ok_or_else(invalid_source_data)?;
        let chunk_span = ByteSpan::new(offset, next_offset).ok_or_else(invalid_source_data)?;
        let source_chunk = source_buffer
            .get_mut(..chunk_len)
            .ok_or_else(invalid_source_data)?;
        document.read_span(chunk_span, source_chunk)?;

        let last = next_offset == span.end();
        let mut chunk_read = 0_usize;
        loop {
            let input = source_chunk
                .get(chunk_read..)
                .ok_or_else(invalid_source_data)?;
            let output = destination
                .get_mut(total_written..)
                .ok_or_else(invalid_source_data)?;
            let result = if output.len() < 4 {
                short_output.fill(0);
                let result = session.decode(input, &mut short_output, last);
                total_read = total_read
                    .checked_add(result.read())
                    .ok_or_else(invalid_source_data)?;
                chunk_read = chunk_read
                    .checked_add(result.read())
                    .ok_or_else(invalid_source_data)?;
                if result.written() > output.len() {
                    let produced = short_output
                        .get(..result.written())
                        .ok_or_else(invalid_source_data)?;
                    let copy_len = fitting_utf8_prefix(produced, output.len())?;
                    let prefix = produced.get(..copy_len).ok_or_else(invalid_source_data)?;
                    let target = output.get_mut(..copy_len).ok_or_else(invalid_source_data)?;
                    target.copy_from_slice(prefix);
                    total_written = total_written
                        .checked_add(copy_len)
                        .ok_or_else(invalid_source_data)?;
                    return Ok(DxfTextDecodeResult::from_parts(
                        DxfTextDecodeStatus::OutputFull,
                        total_read,
                        total_written,
                    ));
                }
                let produced = short_output
                    .get(..result.written())
                    .ok_or_else(invalid_source_data)?;
                let target = output
                    .get_mut(..result.written())
                    .ok_or_else(invalid_source_data)?;
                target.copy_from_slice(produced);
                result
            } else {
                let result = session.decode(input, output, last);
                total_read = total_read
                    .checked_add(result.read())
                    .ok_or_else(invalid_source_data)?;
                chunk_read = chunk_read
                    .checked_add(result.read())
                    .ok_or_else(invalid_source_data)?;
                result
            };
            total_written = total_written
                .checked_add(result.written())
                .ok_or_else(invalid_source_data)?;
            if chunk_read > chunk_len {
                return Err(invalid_source_data());
            }

            match result.status() {
                DxfTextDecodeStatus::Complete if chunk_read == chunk_len => {
                    offset = next_offset;
                    if last {
                        return Ok(DxfTextDecodeResult::from_parts(
                            DxfTextDecodeStatus::Complete,
                            total_read,
                            total_written,
                        ));
                    }
                    break;
                }
                DxfTextDecodeStatus::Complete => return Err(invalid_source_data()),
                DxfTextDecodeStatus::OutputFull
                    if total_written < destination.len()
                        && (result.read() != 0 || result.written() != 0) => {}
                DxfTextDecodeStatus::OutputFull => {
                    return Ok(DxfTextDecodeResult::from_parts(
                        DxfTextDecodeStatus::OutputFull,
                        total_read,
                        total_written,
                    ));
                }
                status @ DxfTextDecodeStatus::Malformed { .. } => {
                    return Ok(DxfTextDecodeResult::from_parts(
                        status,
                        total_read,
                        total_written,
                    ));
                }
            }
        }
    }
    Err(invalid_source_data())
}

fn fitting_utf8_prefix(bytes: &[u8], limit: usize) -> Result<usize, DxfError> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid_source_data())?;
    let mut end = bytes.len().min(limit);
    while !text.is_char_boundary(end) {
        end = end.checked_sub(1).ok_or_else(invalid_source_data)?;
    }
    Ok(end)
}

fn invalid_source_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io, sync::Mutex};

    use super::DxfTextValueDecodeReceipt;
    use crate::{
        DxfAcadVersion, DxfAsciiRawDocument, DxfByteSource, DxfCancellationToken,
        DxfDiagnosticCode, DxfError, DxfGroupCode, DxfLegacyCodePage, DxfMemorySource,
        DxfReadOptions, DxfResourceProfile, DxfTextDecodeStatus, DxfTextEncodingResolution,
        NoopDxfReadObserver,
    };

    #[test]
    fn modern_utf8_crosses_the_internal_source_chunk_with_exact_provenance()
    -> Result<(), Box<dyn Error>> {
        let mut value = vec![b'a'; super::SOURCE_CHUNK_BYTES - 1];
        value.extend_from_slice("é".as_bytes());
        let bytes = fixture("AC1021", None, &value);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open(&source)?;
        let occurrence = last_group_one(&document)?;
        let mut destination = vec![0_u8; value.len()];
        let receipt = document
            .decode_group_value_to_utf8_without_replacement(occurrence, &mut destination)?;
        let result = decoded(receipt)?;

        assert_eq!(receipt.source_id(), document.source_id());
        assert_eq!(receipt.group_occurrence(), occurrence);
        assert_eq!(
            receipt.encoding(),
            DxfTextEncodingResolution::Utf8(DxfAcadVersion::Ac1021)
        );
        assert_eq!(
            receipt.value_span(),
            document.groups()[occurrence as usize].value_content_span()
        );
        assert_eq!(result.status(), DxfTextDecodeStatus::Complete);
        assert_eq!(result.read(), value.len());
        assert_eq!(result.written(), value.len());
        assert_eq!(destination, value);
        Ok(())
    }

    #[test]
    fn known_legacy_page_is_selected_from_original_parse_bytes() -> Result<(), Box<dyn Error>> {
        let bytes = fixture("AC1018", Some("ANSI_932"), b"\x82\xA0");
        let source = RecordingSource::new(&bytes);
        let document = open(&source)?;
        let occurrence = last_group_one(&document)?;
        let declaration_span = document
            .text_encoding_report()
            .primary_occurrence()
            .and_then(|item| item.value_span())
            .ok_or(io::Error::other("missing declaration span"))?;
        let occurrence_index = usize::try_from(occurrence)?;
        let value_span = document
            .groups()
            .get(occurrence_index)
            .ok_or(io::Error::other("missing value group"))?
            .value_content_span();
        source.clear_reads();
        let mut destination = [0_u8; 3];
        let receipt = document
            .decode_group_value_to_utf8_without_replacement(occurrence, &mut destination)?;
        let result = decoded(receipt)?;
        assert_eq!(
            receipt.encoding(),
            DxfTextEncodingResolution::Legacy {
                version: DxfAcadVersion::Ac1018,
                code_page: DxfLegacyCodePage::Windows932,
            }
        );
        assert_eq!(&destination[..result.written()], "あ".as_bytes());
        let reads = source.reads();
        assert!(!reads.is_empty());
        for (offset, count) in reads {
            let end = offset
                .checked_add(u64::try_from(count)?)
                .ok_or(io::Error::other("read overflow"))?;
            assert!(offset >= value_span.start() && end <= value_span.end());
            assert!(end <= declaration_span.start() || offset >= declaration_span.end());
        }
        Ok(())
    }

    #[test]
    fn unavailable_encoding_is_explicit_and_never_touches_output() -> Result<(), Box<dyn Error>> {
        let cases = [
            (fixture("AC1018", Some("ANSI_1361"), b"text"), true),
            (fixture("AC1018", None, b"text"), false),
        ];
        for (bytes, unsupported) in cases {
            let source = RecordingSource::new(&bytes);
            let document = open(&source)?;
            let occurrence = last_group_one(&document)?;
            source.clear_reads();
            let mut destination = [0xA5_u8; 8];
            let receipt = document
                .decode_group_value_to_utf8_without_replacement(occurrence, &mut destination)?;
            assert_eq!(receipt.decode_result(), None);
            assert_eq!(destination, [0xA5; 8]);
            assert!(source.reads().is_empty());
            assert_eq!(
                matches!(
                    receipt.encoding(),
                    DxfTextEncodingResolution::UnsupportedLegacy { .. }
                ),
                unsupported
            );
            if unsupported {
                assert!(
                    document
                        .text_encoding_report()
                        .diagnostics()
                        .iter()
                        .any(|item| item.code() == DxfDiagnosticCode::CODEPAGE_UNSUPPORTED)
                );
            } else {
                assert_eq!(receipt.encoding(), DxfTextEncodingResolution::Indeterminate);
            }
        }
        Ok(())
    }

    #[test]
    fn output_full_retries_from_start_and_malformed_utf8_is_not_replaced()
    -> Result<(), Box<dyn Error>> {
        let bytes = fixture("AC1021", None, "Biển".as_bytes());
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open(&source)?;
        let occurrence = last_group_one(&document)?;
        let mut small = [0_u8; 4];
        let partial = decoded(
            document.decode_group_value_to_utf8_without_replacement(occurrence, &mut small)?,
        )?;
        assert_eq!(partial.status(), DxfTextDecodeStatus::OutputFull);

        let mut full = [0_u8; 16];
        let complete = decoded(
            document.decode_group_value_to_utf8_without_replacement(occurrence, &mut full)?,
        )?;
        assert_eq!(complete.status(), DxfTextDecodeStatus::Complete);
        assert_eq!(&full[..complete.written()], "Biển".as_bytes());

        let malformed_bytes = fixture("AC1021", None, b"\xC3(");
        let malformed_source = DxfMemorySource::new(&malformed_bytes, DxfResourceProfile::Safe)?;
        let malformed_document = open(&malformed_source)?;
        let malformed_occurrence = last_group_one(&malformed_document)?;
        full.fill(0);
        let malformed = decoded(
            malformed_document
                .decode_group_value_to_utf8_without_replacement(malformed_occurrence, &mut full)?,
        )?;
        assert!(matches!(
            malformed.status(),
            DxfTextDecodeStatus::Malformed { .. }
        ));
        assert!(
            !full[..malformed.written()]
                .windows(3)
                .any(|item| item == b"\xEF\xBF\xBD")
        );
        Ok(())
    }

    #[test]
    fn invalid_occurrence_fails_without_touching_output() -> Result<(), Box<dyn Error>> {
        let bytes = fixture("AC1021", None, b"text");
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open(&source)?;
        let mut destination = [0x5A_u8; 8];
        let result =
            document.decode_group_value_to_utf8_without_replacement(u64::MAX, &mut destination);
        assert!(matches!(result, Err(DxfError::Io { .. })));
        assert_eq!(destination, [0x5A; 8]);
        Ok(())
    }

    #[test]
    fn destination_capacity_bounds_source_io() -> Result<(), Box<dyn Error>> {
        let value = vec![b'a'; super::SOURCE_CHUNK_BYTES * 2];
        let bytes = fixture("AC1021", None, &value);
        let source = RecordingSource::new(&bytes);
        let document = open(&source)?;
        let occurrence = last_group_one(&document)?;

        source.clear_reads();
        let mut empty = [];
        let tiny_receipt =
            document.decode_group_value_to_utf8_without_replacement(occurrence, &mut empty)?;
        assert_eq!(
            decoded(tiny_receipt)?.status(),
            DxfTextDecodeStatus::OutputFull
        );
        assert!(source.reads().is_empty());

        source.clear_reads();
        let mut bounded = [0_u8; 4];
        let bounded_receipt =
            document.decode_group_value_to_utf8_without_replacement(occurrence, &mut bounded)?;
        assert_eq!(
            decoded(bounded_receipt)?.status(),
            DxfTextDecodeStatus::OutputFull
        );
        let total_read: usize = source.reads().iter().map(|(_, count)| count).sum();
        assert!(total_read > 0);
        assert!(total_read <= super::SOURCE_CHUNK_BYTES);
        Ok(())
    }

    fn decoded(
        receipt: DxfTextValueDecodeReceipt,
    ) -> Result<crate::DxfTextDecodeResult, io::Error> {
        receipt
            .decode_result()
            .ok_or(io::Error::other("decoder unavailable"))
    }

    fn last_group_one(document: &DxfAsciiRawDocument<'_>) -> Result<u64, io::Error> {
        let code = DxfGroupCode::new(1).ok_or(io::Error::other("group code"))?;
        document
            .groups()
            .iter()
            .rev()
            .find(|group| group.group_code() == code)
            .map(|group| group.occurrence())
            .ok_or(io::Error::other("missing group 1"))
    }

    fn fixture(version: &str, codepage: Option<&str>, value: &[u8]) -> Vec<u8> {
        let mut bytes = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n").into_bytes();
        if let Some(token) = codepage {
            bytes.extend_from_slice(format!("9\n$DWGCODEPAGE\n3\n{token}\n").as_bytes());
        }
        bytes.extend_from_slice(b"0\nENDSEC\n0\nSECTION\n2\nENTITIES\n1\n");
        bytes.extend_from_slice(value);
        bytes.extend_from_slice(b"\n0\nENDSEC\n0\nEOF\n");
        bytes
    }

    fn open<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
        let cancellation = DxfCancellationToken::default();
        let mut observer = NoopDxfReadObserver;
        DxfAsciiRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &cancellation,
            &mut observer,
        )
    }

    struct RecordingSource<'a> {
        bytes: &'a [u8],
        reads: Mutex<Vec<(u64, usize)>>,
    }

    impl<'a> RecordingSource<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self {
                bytes,
                reads: Mutex::new(Vec::new()),
            }
        }

        fn clear_reads(&self) {
            match self.reads.lock() {
                Ok(mut reads) => reads.clear(),
                Err(poisoned) => poisoned.into_inner().clear(),
            }
        }

        fn reads(&self) -> Vec<(u64, usize)> {
            match self.reads.lock() {
                Ok(reads) => reads.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            }
        }
    }

    impl DxfByteSource for RecordingSource<'_> {
        fn len(&self) -> u64 {
            self.bytes.len() as u64
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            if destination.is_empty() || offset >= self.len() {
                return Ok(0);
            }
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            let count = (self.bytes.len() - start).min(destination.len());
            destination[..count].copy_from_slice(&self.bytes[start..start + count]);
            match self.reads.lock() {
                Ok(mut reads) => reads.push((offset, count)),
                Err(poisoned) => poisoned.into_inner().push((offset, count)),
            }
            Ok(count)
        }
    }
}

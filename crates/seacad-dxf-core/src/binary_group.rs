use std::{fmt, io};

use crate::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfBinaryGroupCodeEncoding, DxfBinaryValueFamily, DxfByteSource,
    DxfCancellationToken, DxfError, DxfGroupCode, DxfIoOperation, DxfPhysicalFormat,
    DxfReadOptions, DxfResource, decode_binary_group_code, probe_dxf_physical_format,
};

const BUFFER_BYTES: usize = 8 * 1024;

/// Borrowed lossless Binary DXF group/value pair. Fields remain valid until
/// the cursor is advanced again.
#[derive(Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub struct DxfBinaryGroup<'a> {
    pub occurrence: u64,
    pub group_code: DxfGroupCode,
    pub value_family: DxfBinaryValueFamily,
    pub raw_group_code: &'a [u8],
    /// Includes the string NUL or binary-chunk length byte.
    pub raw_value: &'a [u8],
    pub group_code_span: ByteSpan,
    pub value_span: ByteSpan,
    pub payload_span: ByteSpan,
    pub full_span: ByteSpan,
}

impl fmt::Debug for DxfBinaryGroup<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfBinaryGroup")
            .field("occurrence", &self.occurrence)
            .field("group_code", &self.group_code)
            .field("value_family", &self.value_family)
            .field("raw_group_code_bytes", &self.raw_group_code.len())
            .field("raw_value_bytes", &self.raw_value.len())
            .field("full_span", &self.full_span)
            .finish()
    }
}

/// Bounded buffered cursor over Binary DXF groups after the exact sentinel.
pub struct DxfBinaryGroupCursor<'a> {
    source: &'a dyn DxfByteSource,
    encoding: DxfBinaryGroupCodeEncoding,
    source_len: u64,
    source_offset: u64,
    consumed_offset: u64,
    max_records: u64,
    max_value_bytes: u64,
    next_occurrence: u64,
    buffer: [u8; BUFFER_BYTES],
    buffer_pos: usize,
    buffer_len: usize,
    raw_group_code: [u8; 3],
    raw_group_code_len: usize,
    raw_value: Vec<u8>,
}

impl<'a> DxfBinaryGroupCursor<'a> {
    pub fn new(
        source: &'a dyn DxfByteSource,
        encoding: DxfBinaryGroupCodeEncoding,
        options: DxfReadOptions,
    ) -> Result<Self, DxfError> {
        let limits = options.resource_profile().limits();
        Self::with_limits(
            source,
            encoding,
            options,
            limits.max_records(),
            limits.max_value_bytes(),
        )
    }

    fn with_limits(
        source: &'a dyn DxfByteSource,
        encoding: DxfBinaryGroupCodeEncoding,
        options: DxfReadOptions,
        max_records: u64,
        max_value_bytes: u64,
    ) -> Result<Self, DxfError> {
        if probe_dxf_physical_format(source, options.resource_profile())?
            != DxfPhysicalFormat::Binary
        {
            return Err(DxfError::InvalidBinarySentinel {
                span: span(0, source.len().min(DXF_BINARY_SENTINEL.len() as u64))?,
            });
        }
        let start = DXF_BINARY_SENTINEL.len() as u64;
        Ok(Self {
            source,
            encoding,
            source_len: source.len(),
            source_offset: start,
            consumed_offset: start,
            max_records,
            max_value_bytes,
            next_occurrence: 0,
            buffer: [0; BUFFER_BYTES],
            buffer_pos: 0,
            buffer_len: 0,
            raw_group_code: [0; 3],
            raw_group_code_len: 0,
            raw_value: Vec::new(),
        })
    }

    #[must_use]
    pub const fn source_len(&self) -> u64 {
        self.source_len
    }

    #[must_use]
    pub const fn consumed_bytes(&self) -> u64 {
        self.consumed_offset
    }

    #[must_use]
    pub const fn records_framed(&self) -> u64 {
        self.next_occurrence
    }

    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.consumed_offset == self.source_len
    }

    /// Returns the next pair. Discard the cursor after any error.
    pub fn next_group(
        &mut self,
        cancellation: &DxfCancellationToken,
    ) -> Result<Option<DxfBinaryGroup<'_>>, DxfError> {
        if self.is_complete() {
            return Ok(None);
        }
        ensure_not_cancelled(cancellation)?;
        let observed = checked_add(self.next_occurrence, 1)?;
        if observed > self.max_records {
            return Err(DxfError::resource_limit(
                DxfResource::Records,
                self.max_records,
                observed,
            ));
        }

        self.raw_group_code_len = 0;
        self.raw_value.clear();
        let group_start = self.consumed_offset;
        let group_code = self.read_group_code(group_start, cancellation)?;
        let value_start = self.consumed_offset;
        let code_span = span(group_start, value_start)?;
        let family = DxfBinaryValueFamily::from_group_code(group_code).ok_or(
            DxfError::UnsupportedBinaryGroupCode {
                group_code: group_code.value(),
                span: code_span,
            },
        )?;
        let payload_start = if family == DxfBinaryValueFamily::BinaryChunk {
            checked_add(value_start, 1)?
        } else {
            value_start
        };
        self.read_value(group_code, family, value_start, cancellation)?;
        let value_end = self.consumed_offset;
        let payload_end = if family == DxfBinaryValueFamily::NullTerminatedString {
            value_end.checked_sub(1).ok_or_else(invalid_data)?
        } else {
            value_end
        };
        self.next_occurrence = observed;

        Ok(Some(DxfBinaryGroup {
            occurrence: observed - 1,
            group_code,
            value_family: family,
            raw_group_code: &self.raw_group_code[..self.raw_group_code_len],
            raw_value: &self.raw_value,
            group_code_span: code_span,
            value_span: span(value_start, value_end)?,
            payload_span: span(payload_start, payload_end)?,
            full_span: span(group_start, value_end)?,
        }))
    }

    fn read_group_code(
        &mut self,
        start: u64,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfGroupCode, DxfError> {
        let initial = usize::from(self.encoding.minimum_group_code_bytes());
        self.read_code_bytes(initial, start, cancellation)?;
        if self.encoding == DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape
            && self.raw_group_code[0] == u8::MAX
        {
            self.read_code_bytes(3, start, cancellation)?;
        }
        let header = decode_binary_group_code(
            &self.raw_group_code[..self.raw_group_code_len],
            start,
            self.encoding,
        )?;
        Ok(header.group_code)
    }

    fn read_code_bytes(
        &mut self,
        required: usize,
        start: u64,
        cancellation: &DxfCancellationToken,
    ) -> Result<(), DxfError> {
        while self.raw_group_code_len < required {
            let Some(byte) = self.read_byte(cancellation)? else {
                return Err(DxfError::TruncatedBinaryGroupCode {
                    span: span(start, self.consumed_offset)?,
                    expected_bytes: required as u8,
                });
            };
            self.raw_group_code[self.raw_group_code_len] = byte;
            self.raw_group_code_len += 1;
        }
        Ok(())
    }

    fn read_value(
        &mut self,
        group_code: DxfGroupCode,
        family: DxfBinaryValueFamily,
        start: u64,
        cancellation: &DxfCancellationToken,
    ) -> Result<(), DxfError> {
        if let Some(width) = family.fixed_payload_bytes() {
            self.enforce_value_limit(u64::from(width))?;
            return self.read_exact_value(group_code, start, usize::from(width), 0, cancellation);
        }
        if family == DxfBinaryValueFamily::BinaryChunk {
            let Some(length) = self.read_byte(cancellation)? else {
                return Err(self.truncated_value(group_code, start, 1)?);
            };
            self.push_value(length)?;
            self.enforce_value_limit(u64::from(length))?;
            return self.read_exact_value(group_code, start, usize::from(length), 1, cancellation);
        }
        self.read_string(group_code, start, cancellation)
    }

    fn read_exact_value(
        &mut self,
        group_code: DxfGroupCode,
        start: u64,
        payload_bytes: usize,
        prefix_bytes: usize,
        cancellation: &DxfCancellationToken,
    ) -> Result<(), DxfError> {
        let target = payload_bytes + prefix_bytes;
        while self.raw_value.len() < target {
            if !self.ensure_buffer(cancellation)? {
                return Err(self.truncated_value(group_code, start, target as u64)?);
            }
            let remaining = target - self.raw_value.len();
            self.append_buffered(remaining.min(self.buffer_len - self.buffer_pos))?;
        }
        Ok(())
    }

    fn read_string(
        &mut self,
        group_code: DxfGroupCode,
        start: u64,
        cancellation: &DxfCancellationToken,
    ) -> Result<(), DxfError> {
        loop {
            if !self.ensure_buffer(cancellation)? {
                return Err(DxfError::UnterminatedBinaryString {
                    group_code: group_code.value(),
                    span: span(start, self.consumed_offset)?,
                });
            }
            let available = self.buffer_len - self.buffer_pos;
            let delimiter = self.buffer[self.buffer_pos..self.buffer_len]
                .iter()
                .position(|byte| *byte == 0);
            let count = delimiter.unwrap_or(available);
            self.enforce_value_limit(checked_add(
                usize_u64(self.raw_value.len())?,
                usize_u64(count)?,
            )?)?;
            self.append_buffered(count)?;
            if delimiter.is_some() {
                let terminator = self.read_byte(cancellation)?.ok_or_else(invalid_data)?;
                self.push_value(terminator)?;
                return Ok(());
            }
        }
    }

    fn truncated_value(
        &self,
        group_code: DxfGroupCode,
        start: u64,
        expected_value_bytes: u64,
    ) -> Result<DxfError, DxfError> {
        Ok(DxfError::TruncatedBinaryValue {
            group_code: group_code.value(),
            span: span(start, self.consumed_offset)?,
            expected_value_bytes,
        })
    }

    fn enforce_value_limit(&self, observed: u64) -> Result<(), DxfError> {
        if observed > self.max_value_bytes {
            Err(DxfError::resource_limit(
                DxfResource::ValueBytes,
                self.max_value_bytes,
                observed,
            ))
        } else {
            Ok(())
        }
    }

    fn ensure_buffer(&mut self, cancellation: &DxfCancellationToken) -> Result<bool, DxfError> {
        if self.buffer_pos < self.buffer_len {
            return Ok(true);
        }
        if self.consumed_offset == self.source_len {
            return Ok(false);
        }
        ensure_not_cancelled(cancellation)?;
        let remaining = self.source_len - self.source_offset;
        let requested =
            usize::try_from(remaining.min(BUFFER_BYTES as u64)).map_err(|_| invalid_data())?;
        if requested == 0 {
            return Err(source_error(io::ErrorKind::UnexpectedEof));
        }
        let read = self
            .source
            .read_at(self.source_offset, &mut self.buffer[..requested])?;
        ensure_not_cancelled(cancellation)?;
        if read == 0 {
            return Err(source_error(io::ErrorKind::UnexpectedEof));
        }
        if read > requested {
            return Err(invalid_data());
        }
        self.source_offset = checked_add(self.source_offset, usize_u64(read)?)?;
        self.buffer_pos = 0;
        self.buffer_len = read;
        Ok(true)
    }

    fn read_byte(&mut self, cancellation: &DxfCancellationToken) -> Result<Option<u8>, DxfError> {
        if !self.ensure_buffer(cancellation)? {
            return Ok(None);
        }
        let byte = self
            .buffer
            .get(self.buffer_pos)
            .copied()
            .ok_or_else(invalid_data)?;
        self.buffer_pos += 1;
        self.consumed_offset = checked_add(self.consumed_offset, 1)?;
        Ok(Some(byte))
    }

    fn append_buffered(&mut self, count: usize) -> Result<(), DxfError> {
        let end = self
            .buffer_pos
            .checked_add(count)
            .ok_or_else(invalid_data)?;
        let bytes = self
            .buffer
            .get(self.buffer_pos..end)
            .ok_or_else(invalid_data)?;
        self.raw_value
            .try_reserve(count)
            .map_err(|_| source_error(io::ErrorKind::OutOfMemory))?;
        self.raw_value.extend_from_slice(bytes);
        self.buffer_pos = end;
        self.consumed_offset = checked_add(self.consumed_offset, usize_u64(count)?)?;
        Ok(())
    }

    fn push_value(&mut self, byte: u8) -> Result<(), DxfError> {
        self.raw_value
            .try_reserve(1)
            .map_err(|_| source_error(io::ErrorKind::OutOfMemory))?;
        self.raw_value.push(byte);
        Ok(())
    }
}

fn span(start: u64, end: u64) -> Result<ByteSpan, DxfError> {
    ByteSpan::new(start, end).ok_or_else(invalid_data)
}

fn checked_add(value: u64, increment: u64) -> Result<u64, DxfError> {
    value
        .checked_add(increment)
        .ok_or(DxfError::OffsetOverflow {
            offset: value,
            requested: increment,
        })
}

fn usize_u64(value: usize) -> Result<u64, DxfError> {
    u64::try_from(value).map_err(|_| DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    })
}

fn ensure_not_cancelled(token: &DxfCancellationToken) -> Result<(), DxfError> {
    if token.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_data() -> DxfError {
    source_error(io::ErrorKind::InvalidData)
}

fn source_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        io,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{DxfBinaryGroupCursor, span};
    use crate::{
        ByteSpan, DXF_BINARY_SENTINEL, DxfBinaryGroup, DxfBinaryGroupCodeEncoding,
        DxfBinaryValueFamily, DxfByteSource, DxfCancellationToken, DxfError, DxfErrorCode,
        DxfIoOperation, DxfMemorySource, DxfReadOptions, DxfResource, DxfResourceProfile,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct OwnedGroup {
        occurrence: u64,
        code: i16,
        family: DxfBinaryValueFamily,
        raw_code: Vec<u8>,
        raw_value: Vec<u8>,
        code_span: ByteSpan,
        value_span: ByteSpan,
        payload_span: ByteSpan,
        full_span: ByteSpan,
    }

    fn own(group: DxfBinaryGroup<'_>) -> OwnedGroup {
        OwnedGroup {
            occurrence: group.occurrence,
            code: group.group_code.value(),
            family: group.value_family,
            raw_code: group.raw_group_code.to_vec(),
            raw_value: group.raw_value.to_vec(),
            code_span: group.group_code_span,
            value_span: group.value_span,
            payload_span: group.payload_span,
            full_span: group.full_span,
        }
    }

    fn fixture(payload: &[u8]) -> Vec<u8> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        bytes.extend_from_slice(payload);
        bytes
    }

    fn pair_two(target: &mut Vec<u8>, code: i16, value: &[u8]) {
        target.extend_from_slice(&code.to_le_bytes());
        target.extend_from_slice(value);
    }

    fn pair_one(target: &mut Vec<u8>, code: i16, value: &[u8]) {
        if code < 255 {
            target.push(code as u8);
        } else {
            target.push(u8::MAX);
            target.extend_from_slice(&code.to_le_bytes());
        }
        target.extend_from_slice(value);
    }

    fn collect(
        bytes: &[u8],
        encoding: DxfBinaryGroupCodeEncoding,
    ) -> Result<Vec<OwnedGroup>, DxfError> {
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let mut cursor = DxfBinaryGroupCursor::new(&source, encoding, DxfReadOptions::strict())?;
        let token = DxfCancellationToken::default();
        let mut groups = Vec::new();
        while let Some(group) = cursor.next_group(&token)? {
            groups.push(own(group));
        }
        assert!(cursor.is_complete());
        assert_eq!(cursor.consumed_bytes(), bytes.len() as u64);
        assert_eq!(cursor.records_framed(), groups.len() as u64);
        Ok(groups)
    }

    #[test]
    fn two_byte_cursor_frames_every_value_family_and_exact_spans() -> Result<(), Box<dyn Error>> {
        let mut payload = Vec::new();
        let cases: [(i16, &[u8]); 10] = [
            (0, b"SECTION\0"),
            (10, b"\x01\x02\x03\x04\x05\x06\x07\x08"),
            (60, b"\x09\x0a"),
            (90, b"\x0b\x0c\x0d\x0e"),
            (160, b"\x0f\x10\x11\x12\x13\x14\x15\x16"),
            (290, b"\x01"),
            (310, b"\x03\xaa\xbb\xcc"),
            (999, b"comment\0"),
            (1004, b"\x02\xdd\xee"),
            (1071, b"\x17\x18\x19\x1a"),
        ];
        for (code, value) in cases {
            pair_two(&mut payload, code, value);
        }
        let bytes = fixture(&payload);
        let groups = collect(&bytes, DxfBinaryGroupCodeEncoding::TwoByteLittleEndian)?;
        assert_eq!(groups.len(), cases.len());
        assert_eq!(groups[0].code_span, span(22, 24)?);
        assert_eq!(groups[0].value_span, span(24, 32)?);
        assert_eq!(groups[0].payload_span, span(24, 31)?);
        assert_eq!(
            groups[6].payload_span.start(),
            groups[6].value_span.start() + 1
        );
        assert_eq!(groups[6].payload_span.end(), groups[6].value_span.end());

        let reconstructed: Vec<u8> = groups
            .iter()
            .flat_map(|group| group.raw_code.iter().chain(group.raw_value.iter()).copied())
            .collect();
        assert_eq!(reconstructed, payload);
        Ok(())
    }

    #[test]
    fn pre_r13_cursor_preserves_one_byte_and_escaped_codes() -> Result<(), Box<dyn Error>> {
        let mut payload = Vec::new();
        pair_one(&mut payload, 0, b"SECTION\0");
        pair_one(&mut payload, 10, &[0; 8]);
        pair_one(&mut payload, 1004, b"\x02\x11\x22");
        pair_one(&mut payload, 1071, b"\x01\x00\x00\x00");
        let groups = collect(
            &fixture(&payload),
            DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape,
        )?;
        assert_eq!(groups[0].raw_code, [0]);
        assert_eq!(groups[1].raw_code, [10]);
        assert_eq!(groups[2].raw_code, [0xff, 0xec, 0x03]);
        assert_eq!(groups[3].raw_code, [0xff, 0x2f, 0x04]);
        Ok(())
    }

    #[test]
    fn framing_does_not_guess_boolean_or_xdata_chunk_semantics() -> Result<(), Box<dyn Error>> {
        let mut payload = Vec::new();
        pair_two(&mut payload, 290, b"\xff");
        let mut chunk = vec![u8::MAX];
        chunk.extend(0_u8..u8::MAX);
        pair_two(&mut payload, 1004, &chunk);
        pair_two(&mut payload, 999, b"preserved\0");
        let groups = collect(
            &fixture(&payload),
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
        )?;
        assert_eq!(groups[0].raw_value, [u8::MAX]);
        assert_eq!(groups[1].raw_value.len(), 256);
        assert_eq!(groups[1].payload_span.len(), 255);
        assert_eq!(groups[2].raw_value, b"preserved\0");
        Ok(())
    }

    #[test]
    fn invalid_entry_and_group_headers_are_typed() -> Result<(), Box<dyn Error>> {
        for bytes in [
            b"".as_slice(),
            b"AutoCAD Binary".as_slice(),
            b"ASCII DXF".as_slice(),
        ] {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            assert!(matches!(
                DxfBinaryGroupCursor::new(
                    &source,
                    DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
                    DxfReadOptions::strict()
                ),
                Err(DxfError::InvalidBinarySentinel { .. })
            ));
        }

        let token = DxfCancellationToken::default();
        for (tail, expected) in [
            (b"\0".as_slice(), DxfErrorCode::TRUNCATED_BINARY_GROUP_CODE),
            (
                &(-6_i16).to_le_bytes(),
                DxfErrorCode::INVALID_BINARY_GROUP_CODE,
            ),
            (
                &80_i16.to_le_bytes(),
                DxfErrorCode::UNSUPPORTED_BINARY_GROUP_CODE,
            ),
        ] {
            let bytes = fixture(tail);
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let mut cursor = DxfBinaryGroupCursor::new(
                &source,
                DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
                DxfReadOptions::strict(),
            )?;
            let error = cursor
                .next_group(&token)
                .err()
                .ok_or(io::Error::other("bad header was accepted"))?;
            assert_eq!(error.code(), expected);
        }
        Ok(())
    }

    #[test]
    fn every_truncated_value_shape_and_unterminated_string_is_typed() -> Result<(), Box<dyn Error>>
    {
        let cases: [(i16, &[u8], DxfErrorCode, u64); 4] = [
            (
                10,
                b"\x01\x02\x03\x04",
                DxfErrorCode::TRUNCATED_BINARY_VALUE,
                8,
            ),
            (310, b"", DxfErrorCode::TRUNCATED_BINARY_VALUE, 1),
            (
                310,
                b"\x03\x01\x02",
                DxfErrorCode::TRUNCATED_BINARY_VALUE,
                4,
            ),
            (1, b"text", DxfErrorCode::UNTERMINATED_BINARY_STRING, 0),
        ];
        for (code, value, expected_code, expected_bytes) in cases {
            let mut payload = Vec::new();
            pair_two(&mut payload, code, value);
            let bytes = fixture(&payload);
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let mut cursor = DxfBinaryGroupCursor::new(
                &source,
                DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
                DxfReadOptions::strict(),
            )?;
            let error = cursor
                .next_group(&DxfCancellationToken::default())
                .err()
                .ok_or(io::Error::other("truncated value was accepted"))?;
            assert_eq!(error.code(), expected_code);
            if expected_code == DxfErrorCode::TRUNCATED_BINARY_VALUE {
                assert!(matches!(
                    error,
                    DxfError::TruncatedBinaryValue { expected_value_bytes: observed, .. }
                        if observed == expected_bytes
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn record_and_value_limits_fail_before_excess_consumption() -> Result<(), Box<dyn Error>> {
        let mut payload = Vec::new();
        pair_two(&mut payload, 1, b"a\0");
        pair_two(&mut payload, 1, b"b\0");
        let bytes = fixture(&payload);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let options = DxfReadOptions::strict();
        let token = DxfCancellationToken::default();
        let mut cursor = DxfBinaryGroupCursor::with_limits(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            options,
            1,
            2,
        )?;
        assert!(cursor.next_group(&token)?.is_some());
        let before_second = cursor.consumed_bytes();
        assert!(matches!(
            cursor.next_group(&token),
            Err(DxfError::ResourceLimitExceeded {
                resource: DxfResource::Records,
                limit: 1,
                observed: 2,
            })
        ));
        assert_eq!(cursor.consumed_bytes(), before_second);

        for (code, value) in [(1, b"abc\0".as_slice()), (10, &[0; 8]), (310, b"\x03abc")] {
            let mut payload = Vec::new();
            pair_two(&mut payload, code, value);
            let bytes = fixture(&payload);
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let mut cursor = DxfBinaryGroupCursor::with_limits(
                &source,
                DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
                options,
                1,
                2,
            )?;
            assert!(matches!(
                cursor.next_group(&token),
                Err(DxfError::ResourceLimitExceeded {
                    resource: DxfResource::ValueBytes,
                    limit: 2,
                    ..
                })
            ));
        }
        Ok(())
    }

    #[test]
    fn string_exactly_at_limit_can_consume_its_unaccounted_nul() -> Result<(), Box<dyn Error>> {
        let mut payload = Vec::new();
        pair_two(&mut payload, 1, b"ab\0");
        let bytes = fixture(&payload);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let mut cursor = DxfBinaryGroupCursor::with_limits(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::strict(),
            1,
            2,
        )?;
        let group = cursor
            .next_group(&DxfCancellationToken::default())?
            .ok_or(io::Error::other("missing group"))?;
        assert_eq!(group.raw_value, b"ab\0");
        assert_eq!(group.payload_span.len(), 2);
        Ok(())
    }

    #[test]
    fn string_crosses_read_windows_without_losing_or_repeating_bytes() -> Result<(), Box<dyn Error>>
    {
        let payload_len = super::BUFFER_BYTES + 17;
        let mut value = vec![b'x'; payload_len];
        value.push(0);
        let mut payload = Vec::new();
        pair_two(&mut payload, 1, &value);
        let bytes = fixture(&payload);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let mut cursor = DxfBinaryGroupCursor::with_limits(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::strict(),
            1,
            payload_len as u64,
        )?;
        let group = cursor
            .next_group(&DxfCancellationToken::default())?
            .ok_or(io::Error::other("missing group"))?;
        assert_eq!(group.raw_value, value);
        assert_eq!(group.payload_span.len(), payload_len as u64);
        assert!(cursor.is_complete());
        Ok(())
    }

    struct ChunkedSource {
        bytes: Vec<u8>,
        declared_len: u64,
        max_read: usize,
        cancel: Option<DxfCancellationToken>,
        reads: AtomicU64,
    }

    impl DxfByteSource for ChunkedSource {
        fn len(&self) -> u64 {
            self.declared_len
        }

        fn read_at(&self, offset: u64, output: &mut [u8]) -> Result<usize, DxfError> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            if start >= self.bytes.len() || output.is_empty() {
                return Ok(0);
            }
            let count = (self.bytes.len() - start)
                .min(output.len())
                .min(self.max_read);
            output[..count].copy_from_slice(&self.bytes[start..start + count]);
            if offset >= DXF_BINARY_SENTINEL.len() as u64
                && let Some(token) = &self.cancel
            {
                token.cancel();
            }
            Ok(count)
        }
    }

    #[test]
    fn buffering_partial_reads_cancellation_and_hostile_length_fail_closed()
    -> Result<(), Box<dyn Error>> {
        let mut payload = Vec::new();
        for _ in 0..100 {
            pair_two(&mut payload, 290, b"\0");
        }
        let bytes = fixture(&payload);
        let source = ChunkedSource {
            declared_len: bytes.len() as u64,
            bytes: bytes.clone(),
            max_read: usize::MAX,
            cancel: None,
            reads: AtomicU64::new(0),
        };
        let mut cursor = DxfBinaryGroupCursor::new(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::strict(),
        )?;
        let token = DxfCancellationToken::default();
        while cursor.next_group(&token)?.is_some() {}
        assert_eq!(cursor.records_framed(), 100);
        assert_eq!(source.reads.load(Ordering::Relaxed), 2);

        let source = ChunkedSource {
            declared_len: bytes.len() as u64,
            bytes: bytes.clone(),
            max_read: 1,
            cancel: None,
            reads: AtomicU64::new(0),
        };
        let mut cursor = DxfBinaryGroupCursor::new(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::strict(),
        )?;
        while cursor.next_group(&token)?.is_some() {}
        assert_eq!(cursor.records_framed(), 100);

        let cancel = DxfCancellationToken::default();
        let source = ChunkedSource {
            declared_len: bytes.len() as u64,
            bytes: bytes.clone(),
            max_read: usize::MAX,
            cancel: Some(cancel.clone()),
            reads: AtomicU64::new(0),
        };
        let mut cursor = DxfBinaryGroupCursor::new(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::strict(),
        )?;
        assert!(matches!(
            cursor.next_group(&cancel),
            Err(DxfError::Cancelled)
        ));
        assert_eq!(cursor.consumed_bytes(), 22);

        let source = ChunkedSource {
            declared_len: bytes.len() as u64 + 1,
            bytes,
            max_read: usize::MAX,
            cancel: None,
            reads: AtomicU64::new(0),
        };
        let mut cursor = DxfBinaryGroupCursor::new(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::strict(),
        )?;
        let error = loop {
            match cursor.next_group(&token) {
                Ok(Some(_)) => {}
                Err(error) => break error,
                Ok(None) => return Err(io::Error::other("hostile length was accepted").into()),
            }
        };
        assert!(matches!(
            error,
            DxfError::Io {
                operation: DxfIoOperation::Read,
                kind: io::ErrorKind::UnexpectedEof,
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn sentinel_only_is_empty_and_debug_never_discloses_raw_value() -> Result<(), Box<dyn Error>> {
        let source = DxfMemorySource::new(&DXF_BINARY_SENTINEL, DxfResourceProfile::Safe)?;
        let mut cursor = DxfBinaryGroupCursor::new(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::compatible(),
        )?;
        assert_eq!(cursor.source_len(), 22);
        assert!(cursor.is_complete());
        assert!(
            cursor
                .next_group(&DxfCancellationToken::default())?
                .is_none()
        );

        let bytes = fixture(b"\x01\x00secret\0");
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let mut cursor = DxfBinaryGroupCursor::new(
            &source,
            DxfBinaryGroupCodeEncoding::TwoByteLittleEndian,
            DxfReadOptions::strict(),
        )?;
        let group = cursor
            .next_group(&DxfCancellationToken::default())?
            .ok_or(io::Error::other("missing group"))?;
        let debug = format!("{group:?}");
        assert!(debug.contains("raw_value_bytes"));
        assert!(!debug.contains("secret"));
        Ok(())
    }
}

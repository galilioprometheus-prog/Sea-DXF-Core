use std::{fmt, io};

use crate::{
    ByteSpan, DxfByteSource, DxfCancellationToken, DxfError, DxfIoOperation, DxfResource,
    DxfResourceProfile,
};

const ASCII_READ_BUFFER_BYTES: usize = 8 * 1024;
const ASCII_READ_BUFFER_BYTES_U64: u64 = ASCII_READ_BUFFER_BYTES as u64;

/// Exact byte sequence that terminated one physical ASCII line.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAsciiLineEnding {
    CrLf,
    Lf,
    Cr,
    /// Final physical line reached source EOF without a terminator.
    None,
}

impl DxfAsciiLineEnding {
    #[must_use]
    pub const fn byte_len(self) -> u64 {
        match self {
            Self::CrLf => 2,
            Self::Lf | Self::Cr => 1,
            Self::None => 0,
        }
    }

    #[must_use]
    pub const fn is_terminated(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Byte-exact location metadata for one physical ASCII line.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfAsciiLineMetadata {
    line_index: u64,
    content_span: ByteSpan,
    terminator_span: ByteSpan,
    full_span: ByteSpan,
    ending: DxfAsciiLineEnding,
}

impl DxfAsciiLineMetadata {
    #[must_use]
    pub const fn line_index(self) -> u64 {
        self.line_index
    }

    #[must_use]
    pub const fn content_span(self) -> ByteSpan {
        self.content_span
    }

    #[must_use]
    pub const fn terminator_span(self) -> ByteSpan {
        self.terminator_span
    }

    #[must_use]
    pub const fn full_span(self) -> ByteSpan {
        self.full_span
    }

    #[must_use]
    pub const fn ending(self) -> DxfAsciiLineEnding {
        self.ending
    }
}

/// Borrowed, byte-exact view of one physical line.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct DxfAsciiPhysicalLine<'a> {
    line_index: u64,
    bytes: &'a [u8],
    content_span: ByteSpan,
    terminator_span: ByteSpan,
    full_span: ByteSpan,
    ending: DxfAsciiLineEnding,
}

impl fmt::Debug for DxfAsciiPhysicalLine<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfAsciiPhysicalLine")
            .field("line_index", &self.line_index)
            .field("content_bytes", &self.bytes.len())
            .field("content_span", &self.content_span)
            .field("terminator_span", &self.terminator_span)
            .field("full_span", &self.full_span)
            .field("ending", &self.ending)
            .finish()
    }
}

impl<'a> DxfAsciiPhysicalLine<'a> {
    /// Zero-based physical line occurrence.
    #[must_use]
    pub const fn line_index(self) -> u64 {
        self.line_index
    }

    /// Content bytes excluding the physical line terminator.
    #[must_use]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub const fn content_span(self) -> ByteSpan {
        self.content_span
    }

    #[must_use]
    pub const fn terminator_span(self) -> ByteSpan {
        self.terminator_span
    }

    #[must_use]
    pub const fn full_span(self) -> ByteSpan {
        self.full_span
    }

    #[must_use]
    pub const fn ending(self) -> DxfAsciiLineEnding {
        self.ending
    }

    #[must_use]
    pub const fn metadata(self) -> DxfAsciiLineMetadata {
        DxfAsciiLineMetadata {
            line_index: self.line_index,
            content_span: self.content_span,
            terminator_span: self.terminator_span,
            full_span: self.full_span,
            ending: self.ending,
        }
    }
}

/// Buffered cursor that frames physical lines without decoding or normalizing.
pub struct DxfAsciiLineCursor<'a> {
    source: &'a dyn DxfByteSource,
    source_len: u64,
    max_value_bytes: u64,
    source_offset: u64,
    consumed_offset: u64,
    next_line_index: u64,
    read_buffer: [u8; ASCII_READ_BUFFER_BYTES],
    buffer_pos: usize,
    buffer_len: usize,
    line: Vec<u8>,
}

impl<'a> DxfAsciiLineCursor<'a> {
    pub fn new(
        source: &'a dyn DxfByteSource,
        profile: DxfResourceProfile,
    ) -> Result<Self, DxfError> {
        let limits = profile.limits();
        let source_len = source.len();
        let source_limit = limits.max_source_bytes();
        if source_len > source_limit {
            return Err(DxfError::resource_limit(
                DxfResource::SourceBytes,
                source_limit,
                source_len,
            ));
        }

        Ok(Self {
            source,
            source_len,
            max_value_bytes: limits.max_value_bytes(),
            source_offset: 0,
            consumed_offset: 0,
            next_line_index: 0,
            read_buffer: [0_u8; ASCII_READ_BUFFER_BYTES],
            buffer_pos: 0,
            buffer_len: 0,
            line: Vec::new(),
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
    pub const fn is_complete(&self) -> bool {
        self.consumed_offset == self.source_len
    }

    /// Returns the next physical line. Discard the cursor after any error.
    pub fn next_line(
        &mut self,
        cancellation: &DxfCancellationToken,
    ) -> Result<Option<DxfAsciiPhysicalLine<'_>>, DxfError> {
        if self.consumed_offset == self.source_len {
            return Ok(None);
        }
        ensure_not_cancelled(cancellation)?;
        self.line.clear();
        let line_start = self.consumed_offset;

        loop {
            ensure_not_cancelled(cancellation)?;
            if self.consumed_offset == self.source_len {
                return self.finish_line(
                    line_start,
                    self.consumed_offset,
                    DxfAsciiLineEnding::None,
                );
            }
            self.ensure_buffer(cancellation)?;

            let (delimiter, available) =
                match self.read_buffer.get(self.buffer_pos..self.buffer_len) {
                    Some(bytes) => (
                        bytes.iter().position(|byte| matches!(byte, b'\r' | b'\n')),
                        bytes.len(),
                    ),
                    None => return Err(invalid_source_data()),
                };
            let Some(delimiter_index) = delimiter else {
                self.append_buffered(available)?;
                continue;
            };

            self.append_buffered(delimiter_index)?;
            let content_end = self.consumed_offset;
            let delimiter_byte = self.consume_buffered_byte()?;
            let ending = if delimiter_byte == b'\n' {
                DxfAsciiLineEnding::Lf
            } else if self.peek_byte(cancellation)? == Some(b'\n') {
                self.consume_buffered_byte()?;
                DxfAsciiLineEnding::CrLf
            } else {
                DxfAsciiLineEnding::Cr
            };
            return self.finish_line(line_start, content_end, ending);
        }
    }

    fn ensure_buffer(&mut self, cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
        if self.buffer_pos < self.buffer_len {
            return Ok(());
        }
        if self.source_offset >= self.source_len {
            return Err(source_io_error(io::ErrorKind::UnexpectedEof));
        }

        ensure_not_cancelled(cancellation)?;
        let remaining = self.source_len - self.source_offset;
        let requested_u64 = remaining.min(ASCII_READ_BUFFER_BYTES_U64);
        let requested = usize::try_from(requested_u64).map_err(|_| invalid_source_data())?;
        let destination = match self.read_buffer.get_mut(..requested) {
            Some(destination) => destination,
            None => return Err(invalid_source_data()),
        };
        let read = self.source.read_at(self.source_offset, destination)?;
        ensure_not_cancelled(cancellation)?;
        if read == 0 {
            return Err(source_io_error(io::ErrorKind::UnexpectedEof));
        }
        if read > requested {
            return Err(invalid_source_data());
        }

        let read_u64 = usize_to_u64(read)?;
        let next_source_offset =
            self.source_offset
                .checked_add(read_u64)
                .ok_or(DxfError::OffsetOverflow {
                    offset: self.source_offset,
                    requested: read_u64,
                })?;
        if next_source_offset > self.source_len {
            return Err(invalid_source_data());
        }
        self.source_offset = next_source_offset;
        self.buffer_pos = 0;
        self.buffer_len = read;
        Ok(())
    }

    fn peek_byte(&mut self, cancellation: &DxfCancellationToken) -> Result<Option<u8>, DxfError> {
        if self.consumed_offset == self.source_len {
            return Ok(None);
        }
        self.ensure_buffer(cancellation)?;
        Ok(self.read_buffer.get(self.buffer_pos).copied())
    }

    fn append_buffered(&mut self, count: usize) -> Result<(), DxfError> {
        let available = match self.buffer_len.checked_sub(self.buffer_pos) {
            Some(available) => available,
            None => return Err(invalid_source_data()),
        };
        if count > available {
            return Err(invalid_source_data());
        }

        let count_u64 = usize_to_u64(count)?;
        let current_len = usize_to_u64(self.line.len())?;
        let observed = current_len
            .checked_add(count_u64)
            .ok_or(DxfError::OffsetOverflow {
                offset: current_len,
                requested: count_u64,
            })?;
        if observed > self.max_value_bytes {
            return Err(DxfError::resource_limit(
                DxfResource::ValueBytes,
                self.max_value_bytes,
                observed,
            ));
        }

        let next_consumed = self.checked_consumed_after(count_u64)?;
        self.line
            .try_reserve(count)
            .map_err(|_| source_io_error(io::ErrorKind::OutOfMemory))?;
        let start = self.buffer_pos;
        let end = match start.checked_add(count) {
            Some(end) => end,
            None => return Err(invalid_source_data()),
        };
        let bytes = match self.read_buffer.get(start..end) {
            Some(bytes) => bytes,
            None => return Err(invalid_source_data()),
        };
        self.line.extend_from_slice(bytes);
        self.buffer_pos = end;
        self.consumed_offset = next_consumed;
        Ok(())
    }

    fn consume_buffered_byte(&mut self) -> Result<u8, DxfError> {
        let byte = match self.read_buffer.get(self.buffer_pos).copied() {
            Some(byte) => byte,
            None => return Err(invalid_source_data()),
        };
        let next_consumed = self.checked_consumed_after(1)?;
        self.buffer_pos = match self.buffer_pos.checked_add(1) {
            Some(position) => position,
            None => return Err(invalid_source_data()),
        };
        self.consumed_offset = next_consumed;
        Ok(byte)
    }

    fn checked_consumed_after(&self, requested: u64) -> Result<u64, DxfError> {
        let next = self
            .consumed_offset
            .checked_add(requested)
            .ok_or(DxfError::OffsetOverflow {
                offset: self.consumed_offset,
                requested,
            })?;
        if next > self.source_len {
            Err(invalid_source_data())
        } else {
            Ok(next)
        }
    }

    fn finish_line(
        &mut self,
        line_start: u64,
        content_end: u64,
        ending: DxfAsciiLineEnding,
    ) -> Result<Option<DxfAsciiPhysicalLine<'_>>, DxfError> {
        let full_end = self.consumed_offset;
        let content_span = make_span(line_start, content_end)?;
        let terminator_span = make_span(content_end, full_end)?;
        let full_span = make_span(line_start, full_end)?;
        let line_index = self.next_line_index;
        self.next_line_index =
            self.next_line_index
                .checked_add(1)
                .ok_or(DxfError::OffsetOverflow {
                    offset: self.next_line_index,
                    requested: 1,
                })?;
        Ok(Some(DxfAsciiPhysicalLine {
            line_index,
            bytes: &self.line,
            content_span,
            terminator_span,
            full_span,
            ending,
        }))
    }
}

fn make_span(start: u64, end: u64) -> Result<ByteSpan, DxfError> {
    match ByteSpan::new(start, end) {
        Some(span) => Ok(span),
        None => Err(invalid_source_data()),
    }
}

fn usize_to_u64(value: usize) -> Result<u64, DxfError> {
    u64::try_from(value).map_err(|_| DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    })
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_source_data() -> DxfError {
    source_io_error(io::ErrorKind::InvalidData)
}

fn source_io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        io,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{DxfAsciiLineCursor, DxfAsciiLineEnding};
    use crate::{
        DxfByteSource, DxfCancellationToken, DxfError, DxfIoOperation, DxfMemorySource,
        DxfResource, DxfResourceProfile,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct OwnedLine {
        index: u64,
        bytes: Vec<u8>,
        content: (u64, u64),
        terminator: (u64, u64),
        full: (u64, u64),
        ending: DxfAsciiLineEnding,
    }

    fn next_owned(
        cursor: &mut DxfAsciiLineCursor<'_>,
        token: &DxfCancellationToken,
    ) -> Result<Option<OwnedLine>, DxfError> {
        let Some(line) = cursor.next_line(token)? else {
            return Ok(None);
        };
        Ok(Some(OwnedLine {
            index: line.line_index(),
            bytes: line.bytes().to_vec(),
            content: (line.content_span().start(), line.content_span().end()),
            terminator: (line.terminator_span().start(), line.terminator_span().end()),
            full: (line.full_span().start(), line.full_span().end()),
            ending: line.ending(),
        }))
    }

    struct ChunkedSource {
        bytes: &'static [u8],
        declared_len: u64,
        max_read: usize,
        cancellation: Option<DxfCancellationToken>,
        read_calls: AtomicU64,
    }

    impl ChunkedSource {
        fn new(bytes: &'static [u8], declared_len: u64, max_read: usize) -> Self {
            Self {
                bytes,
                declared_len,
                max_read,
                cancellation: None,
                read_calls: AtomicU64::new(0),
            }
        }

        fn cancelling(bytes: &'static [u8], cancellation: DxfCancellationToken) -> Self {
            Self {
                bytes,
                declared_len: bytes.len() as u64,
                max_read: 1,
                cancellation: Some(cancellation),
                read_calls: AtomicU64::new(0),
            }
        }

        fn read_calls(&self) -> u64 {
            self.read_calls.load(Ordering::Relaxed)
        }
    }

    impl DxfByteSource for ChunkedSource {
        fn len(&self) -> u64 {
            self.declared_len
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            self.read_calls.fetch_add(1, Ordering::Relaxed);
            if let Some(cancellation) = &self.cancellation {
                cancellation.cancel();
            }
            if destination.is_empty() {
                return Ok(0);
            }
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: 0,
            })?;
            if start >= self.bytes.len() {
                return Ok(0);
            }
            let count = (self.bytes.len() - start)
                .min(destination.len())
                .min(self.max_read);
            destination[..count].copy_from_slice(&self.bytes[start..start + count]);
            Ok(count)
        }
    }

    #[test]
    fn every_line_ending_and_span_is_preserved() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(b"a\r\nb\nc\rd", DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut observed = Vec::new();
        while let Some(line) = next_owned(&mut cursor, &token)? {
            observed.push(line);
        }

        assert_eq!(
            observed,
            [
                OwnedLine {
                    index: 0,
                    bytes: b"a".to_vec(),
                    content: (0, 1),
                    terminator: (1, 3),
                    full: (0, 3),
                    ending: DxfAsciiLineEnding::CrLf,
                },
                OwnedLine {
                    index: 1,
                    bytes: b"b".to_vec(),
                    content: (3, 4),
                    terminator: (4, 5),
                    full: (3, 5),
                    ending: DxfAsciiLineEnding::Lf,
                },
                OwnedLine {
                    index: 2,
                    bytes: b"c".to_vec(),
                    content: (5, 6),
                    terminator: (6, 7),
                    full: (5, 7),
                    ending: DxfAsciiLineEnding::Cr,
                },
                OwnedLine {
                    index: 3,
                    bytes: b"d".to_vec(),
                    content: (7, 8),
                    terminator: (8, 8),
                    full: (7, 8),
                    ending: DxfAsciiLineEnding::None,
                },
            ]
        );
        assert_eq!(cursor.consumed_bytes(), 8);
        assert_eq!(cursor.source_len(), 8);
        assert!(cursor.is_complete());
        Ok(())
    }

    #[test]
    fn empty_lines_do_not_create_a_phantom_line_at_eof() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(b"\r\n\n\r", DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut endings = Vec::new();
        while let Some(line) = cursor.next_line(&token)? {
            assert!(line.bytes().is_empty());
            endings.push(line.ending());
        }
        assert_eq!(
            endings,
            [
                DxfAsciiLineEnding::CrLf,
                DxfAsciiLineEnding::Lf,
                DxfAsciiLineEnding::Cr,
            ]
        );
        assert_eq!(cursor.next_line(&token)?, None);
        Ok(())
    }

    #[test]
    fn bom_whitespace_and_nul_remain_raw_physical_bytes() -> Result<(), DxfError> {
        let bytes = b"\xef\xbb\xbf  0 \0\r\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let Some(line) = cursor.next_line(&token)? else {
            return Err(DxfError::from_io(
                DxfIoOperation::Read,
                &io::Error::from(io::ErrorKind::UnexpectedEof),
            ));
        };
        assert_eq!(line.bytes(), &bytes[..bytes.len() - 2]);
        assert!(format!("{line:?}").contains("content_bytes"));
        assert!(!format!("{line:?}").contains("bytes: ["));
        Ok(())
    }

    #[test]
    fn one_byte_reads_handle_crlf_across_read_boundaries() -> Result<(), DxfError> {
        let source = ChunkedSource::new(b"a\r\nb", 4, 1);
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let first = next_owned(&mut cursor, &token)?;
        let second = next_owned(&mut cursor, &token)?;
        assert_eq!(
            first.map(|line| line.ending),
            Some(DxfAsciiLineEnding::CrLf)
        );
        assert_eq!(second.map(|line| line.bytes), Some(b"b".to_vec()));
        assert_eq!(source.read_calls(), 4);
        Ok(())
    }

    #[test]
    fn buffered_cursor_does_not_reread_short_lines() -> Result<(), DxfError> {
        let source = ChunkedSource::new(b"a\nb\nc\n", 6, 1_024);
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let mut line_count = 0_u64;

        while cursor.next_line(&token)?.is_some() {
            line_count += 1;
        }

        assert_eq!(line_count, 3);
        assert_eq!(source.read_calls(), 1);
        Ok(())
    }

    #[test]
    fn value_limit_fails_before_appending_the_excess_byte() -> Result<(), Box<dyn Error>> {
        let limit = DxfResourceProfile::Safe.limits().max_value_bytes();
        let limit_usize = usize::try_from(limit)?;
        let mut bytes = vec![b'x'; limit_usize + 1];
        bytes.push(b'\n');
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        let error = match cursor.next_line(&token) {
            Err(error) => error,
            Ok(_) => return Err(io::Error::other("oversized line unexpectedly succeeded").into()),
        };
        assert!(matches!(
            error,
            DxfError::ResourceLimitExceeded {
                resource: DxfResource::ValueBytes,
                limit: observed_limit,
                observed,
            } if observed_limit == limit && observed == limit + 1
        ));
        assert_eq!(cursor.consumed_bytes(), limit);
        Ok(())
    }

    #[test]
    fn cancellation_is_checked_before_and_after_source_reads() -> Result<(), DxfError> {
        let source = DxfMemorySource::new(b"a\n", DxfResourceProfile::Safe)?;
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        let pre_cancelled = DxfCancellationToken::default();
        pre_cancelled.cancel();
        assert!(matches!(
            cursor.next_line(&pre_cancelled),
            Err(DxfError::Cancelled)
        ));
        assert_eq!(cursor.consumed_bytes(), 0);

        let token = DxfCancellationToken::default();
        let source = ChunkedSource::cancelling(b"a\n", token.clone());
        let mut cursor = DxfAsciiLineCursor::new(&source, DxfResourceProfile::Safe)?;
        assert!(matches!(cursor.next_line(&token), Err(DxfError::Cancelled)));
        assert_eq!(cursor.consumed_bytes(), 0);
        assert_eq!(source.read_calls(), 1);
        Ok(())
    }

    #[test]
    fn source_limit_and_premature_eof_fail_closed() -> Result<(), DxfError> {
        let oversized = ChunkedSource::new(
            b"",
            DxfResourceProfile::Safe.limits().max_source_bytes() + 1,
            1,
        );
        assert!(matches!(
            DxfAsciiLineCursor::new(&oversized, DxfResourceProfile::Safe),
            Err(DxfError::ResourceLimitExceeded {
                resource: DxfResource::SourceBytes,
                ..
            })
        ));
        assert_eq!(oversized.read_calls(), 0);

        let truncated = ChunkedSource::new(b"ab", 4, 2);
        let mut cursor = DxfAsciiLineCursor::new(&truncated, DxfResourceProfile::Safe)?;
        let token = DxfCancellationToken::default();
        assert!(matches!(
            cursor.next_line(&token),
            Err(DxfError::Io {
                operation: DxfIoOperation::Read,
                kind: io::ErrorKind::UnexpectedEof,
                ..
            })
        ));
        assert_eq!(truncated.read_calls(), 2);
        Ok(())
    }

    #[test]
    fn line_ending_lengths_are_explicit() {
        assert_eq!(DxfAsciiLineEnding::CrLf.byte_len(), 2);
        assert_eq!(DxfAsciiLineEnding::Lf.byte_len(), 1);
        assert_eq!(DxfAsciiLineEnding::Cr.byte_len(), 1);
        assert_eq!(DxfAsciiLineEnding::None.byte_len(), 0);
        assert!(DxfAsciiLineEnding::CrLf.is_terminated());
        assert!(!DxfAsciiLineEnding::None.is_terminated());
    }
}

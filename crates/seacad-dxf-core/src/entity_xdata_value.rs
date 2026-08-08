//! Typed, source-anchored projections for documented entity XDATA values.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfEntityRef, DxfEntityXDataApplication, DxfEntityXDataDirectory,
    DxfEntityXDataOccurrence, DxfError, DxfHandle, DxfHandleParseIssue, DxfIoOperation,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfRawValueProvenance, DxfSourceId,
    raw_double::decode_raw_double,
    raw_handle::parse_raw_group_handle,
    raw_integer::{decode_raw_i16, decode_raw_i32},
    source_span::span_equals_bytes,
};

/// Maximum documented byte length of one group-1000 XDATA string.
pub const DXF_XDATA_STRING_MAX_BYTES: u64 = 255;
/// Maximum documented decoded byte length of one group-1004 XDATA chunk.
pub const DXF_XDATA_BINARY_CHUNK_MAX_BYTES: u64 = 127;
const SCAN_CHUNK_BYTES: usize = 4 * 1024;

/// Exact text role carried by one XDATA string group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataTextKind {
    ApplicationName,
    String,
    LayerName,
}

/// Exact group-1002 list control.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataControl {
    OpenList,
    CloseList,
}

/// Documented semantic role of one XDATA binary64 component.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataDoubleRole {
    PointX,
    WorldPositionX,
    WorldDisplacementX,
    WorldDirectionX,
    PointY,
    WorldPositionY,
    WorldDisplacementY,
    WorldDirectionY,
    PointZ,
    WorldPositionZ,
    WorldDisplacementZ,
    WorldDirectionZ,
    Real,
    Distance,
    ScaleFactor,
}

/// Exact typed value or source-anchored invalidity for one XDATA occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataValue {
    ExactText {
        kind: DxfEntityXDataTextKind,
        raw: DxfRawValueProvenance,
    },
    Control {
        control: DxfEntityXDataControl,
        raw: DxfRawValueProvenance,
    },
    BinaryChunk {
        decoded_bytes: u8,
        raw: DxfRawValueProvenance,
    },
    Handle {
        value: DxfHandle,
        raw: DxfRawValueProvenance,
    },
    Double {
        role: DxfEntityXDataDoubleRole,
        value: DxfDouble,
        raw: DxfRawValueProvenance,
    },
    Int16 {
        value: i16,
        raw: DxfRawValueProvenance,
    },
    Int32 {
        value: i32,
        raw: DxfRawValueProvenance,
    },
    Invalid {
        issue: DxfEntityXDataValueIssue,
        raw: DxfRawValueProvenance,
    },
}

/// Why one retained XDATA occurrence cannot be used as its documented value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataValueIssue {
    StringTooLong { observed_bytes: u64 },
    InvalidControlString,
    BinaryChunkTooLong { observed_bytes: u64 },
    OddBinaryChunkHexLength { encoded_bytes: u64 },
    InvalidBinaryChunkHexDigit { byte_offset: u64 },
    InvalidHandle(DxfHandleParseIssue),
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
    UnsupportedGroupCode { group_code: i16 },
}

/// One typed result retaining its generic XDATA occurrence and stable ordinal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataTypedEntry {
    ordinal: u32,
    occurrence: DxfEntityXDataOccurrence,
    value: DxfEntityXDataValue,
}

impl DxfEntityXDataTypedEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn occurrence(self) -> DxfEntityXDataOccurrence {
        self.occurrence
    }

    #[must_use]
    pub const fn value(self) -> DxfEntityXDataValue {
        self.value
    }
}

/// Typed projection of every retained application, value, and orphan occurrence.
#[derive(Debug)]
pub struct DxfEntityXDataTypedDirectory {
    source_id: DxfSourceId,
    xdata: DxfEntityXDataDirectory,
    entries: Box<[DxfEntityXDataTypedEntry]>,
}

impl DxfEntityXDataTypedDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let xdata = document.entity_xdata_directory(cancellation)?;
        ensure_source(document.source_id(), xdata.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(xdata.occurrences().len())
            .map_err(|_| out_of_memory())?;
        for occurrence in xdata.occurrences().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfEntityXDataTypedEntry {
                ordinal: compact_len(entries.len())?,
                occurrence,
                value: project_value(document, occurrence, cancellation)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            xdata,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn xdata_directory(&self) -> &DxfEntityXDataDirectory {
        &self.xdata
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataTypedEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataTypedEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_group(&self, group_occurrence: u64) -> Option<DxfEntityXDataTypedEntry> {
        self.entries
            .binary_search_by_key(&group_occurrence, |entry| {
                entry.occurrence().group().occurrence()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    pub fn entries_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<&[DxfEntityXDataTypedEntry], DxfError> {
        ensure_source(self.source_id, application.entity().source_id())?;
        if self.xdata.application(application.ordinal()) != Some(application) {
            return Err(invalid_internal_data());
        }
        let range = application.occurrence_range();
        self.entries
            .get(
                usize::try_from(range.start()).map_err(|_| invalid_internal_data())?
                    ..usize::try_from(range.end()).map_err(|_| invalid_internal_data())?,
            )
            .ok_or_else(invalid_internal_data)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataTypedEntry], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let ordinal = entity.record().ordinal();
        let start = self
            .entries
            .partition_point(|entry| entry.occurrence().entity().record().ordinal() < ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.occurrence().entity().record().ordinal() <= ordinal);
        let entries = self
            .entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)?;
        if entries
            .iter()
            .all(|entry| entry.occurrence().entity() == entity)
        {
            Ok(entries)
        } else {
            Err(invalid_internal_data())
        }
    }

    /// Decodes one validated group-1004 chunk into an exact caller-owned buffer.
    pub fn read_binary_chunk(
        &self,
        document: DxfRawDocumentView<'_>,
        entry: DxfEntityXDataTypedEntry,
        destination: &mut [u8],
        cancellation: &DxfCancellationToken,
    ) -> Result<(), DxfError> {
        ensure_source(self.source_id, document.source_id())?;
        if self.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        let DxfEntityXDataValue::BinaryChunk { decoded_bytes, raw } = entry.value() else {
            return Err(invalid_internal_data());
        };
        if destination.len() != usize::from(decoded_bytes) {
            return Err(invalid_internal_data());
        }
        match document.format() {
            DxfRawDocumentFormat::Binary => {
                document.read_span(raw.value_span(), destination)?;
                ensure_not_cancelled(cancellation)
            }
            DxfRawDocumentFormat::Ascii => {
                decode_hex_chunk(document, raw, destination, cancellation)
            }
        }
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_typed_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataTypedDirectory, DxfError> {
        DxfEntityXDataTypedDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_xdata_typed_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataTypedDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_typed_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_xdata_typed_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataTypedDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_typed_directory(cancellation)
    }
}

fn project_value(
    document: DxfRawDocumentView<'_>,
    occurrence: DxfEntityXDataOccurrence,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityXDataValue, DxfError> {
    let group = occurrence.group();
    let raw = DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
        .ok_or_else(invalid_internal_data)?;
    let code = group.group_code().value();
    Ok(match code {
        1001 => DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::ApplicationName,
            raw,
        },
        1000 if raw.value_span().len() <= DXF_XDATA_STRING_MAX_BYTES => {
            DxfEntityXDataValue::ExactText {
                kind: DxfEntityXDataTextKind::String,
                raw,
            }
        }
        1000 => invalid(
            raw,
            DxfEntityXDataValueIssue::StringTooLong {
                observed_bytes: raw.value_span().len(),
            },
        ),
        1002 if span_equals_bytes(document, raw.value_span(), b"{", cancellation)? => {
            DxfEntityXDataValue::Control {
                control: DxfEntityXDataControl::OpenList,
                raw,
            }
        }
        1002 if span_equals_bytes(document, raw.value_span(), b"}", cancellation)? => {
            DxfEntityXDataValue::Control {
                control: DxfEntityXDataControl::CloseList,
                raw,
            }
        }
        1002 => invalid(raw, DxfEntityXDataValueIssue::InvalidControlString),
        1003 => DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::LayerName,
            raw,
        },
        1004 => project_binary_chunk(document, raw, cancellation)?,
        1005 => match parse_raw_group_handle(document, group, cancellation)? {
            Ok(value) => DxfEntityXDataValue::Handle { value, raw },
            Err(issue) => invalid(raw, DxfEntityXDataValueIssue::InvalidHandle(issue)),
        },
        1010..=1013 | 1020..=1023 | 1030..=1033 | 1040..=1042 => {
            let role = double_role(code).ok_or_else(invalid_internal_data)?;
            match decode_raw_double(document, group, cancellation)? {
                Ok(value) if value.is_finite() => DxfEntityXDataValue::Double { role, value, raw },
                Ok(value) => invalid(raw, DxfEntityXDataValueIssue::NonFiniteDouble(value)),
                Err(issue) => invalid(raw, DxfEntityXDataValueIssue::InvalidAsciiNumber(issue)),
            }
        }
        1070 => match decode_raw_i16(document, group, cancellation)? {
            Ok(value) => DxfEntityXDataValue::Int16 { value, raw },
            Err(issue) => invalid(raw, DxfEntityXDataValueIssue::InvalidAsciiNumber(issue)),
        },
        1071 => match decode_raw_i32(document, group, cancellation)? {
            Ok(value) => DxfEntityXDataValue::Int32 { value, raw },
            Err(issue) => invalid(raw, DxfEntityXDataValueIssue::InvalidAsciiNumber(issue)),
        },
        group_code => invalid(
            raw,
            DxfEntityXDataValueIssue::UnsupportedGroupCode { group_code },
        ),
    })
}

fn project_binary_chunk(
    document: DxfRawDocumentView<'_>,
    raw: DxfRawValueProvenance,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityXDataValue, DxfError> {
    let bytes = match document.format() {
        DxfRawDocumentFormat::Binary => raw.value_span().len(),
        DxfRawDocumentFormat::Ascii => match validate_hex_chunk(document, raw, cancellation)? {
            Ok(bytes) => bytes,
            Err(issue) => return Ok(invalid(raw, issue)),
        },
    };
    if bytes > DXF_XDATA_BINARY_CHUNK_MAX_BYTES {
        return Ok(invalid(
            raw,
            DxfEntityXDataValueIssue::BinaryChunkTooLong {
                observed_bytes: bytes,
            },
        ));
    }
    Ok(DxfEntityXDataValue::BinaryChunk {
        decoded_bytes: u8::try_from(bytes).map_err(|_| invalid_internal_data())?,
        raw,
    })
}

fn validate_hex_chunk(
    document: DxfRawDocumentView<'_>,
    raw: DxfRawValueProvenance,
    cancellation: &DxfCancellationToken,
) -> Result<Result<u64, DxfEntityXDataValueIssue>, DxfError> {
    let span = raw.value_span();
    if !span.len().is_multiple_of(2) {
        return Ok(Err(DxfEntityXDataValueIssue::OddBinaryChunkHexLength {
            encoded_bytes: span.len(),
        }));
    }
    let mut buffer = [0_u8; SCAN_CHUNK_BYTES];
    let mut scanned = 0_u64;
    while scanned < span.len() {
        ensure_not_cancelled(cancellation)?;
        let take = usize::try_from((span.len() - scanned).min(SCAN_CHUNK_BYTES as u64))
            .map_err(|_| invalid_internal_data())?;
        let part = crate::ByteSpan::from_start_and_len(span.start() + scanned, take as u64)
            .ok_or_else(invalid_internal_data)?;
        document.read_span(part, &mut buffer[..take])?;
        if let Some(offset) = buffer[..take]
            .iter()
            .position(|byte| hex_nibble(*byte).is_none())
        {
            return Ok(Err(DxfEntityXDataValueIssue::InvalidBinaryChunkHexDigit {
                byte_offset: scanned + offset as u64,
            }));
        }
        scanned += take as u64;
    }
    Ok(Ok(span.len() / 2))
}

fn decode_hex_chunk(
    document: DxfRawDocumentView<'_>,
    raw: DxfRawValueProvenance,
    destination: &mut [u8],
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    let mut pair = [0_u8; 2];
    for (index, output) in destination.iter_mut().enumerate() {
        ensure_not_cancelled(cancellation)?;
        let offset = u64::try_from(index)
            .ok()
            .and_then(|value| value.checked_mul(2))
            .ok_or_else(invalid_internal_data)?;
        let start = raw
            .value_span()
            .start()
            .checked_add(offset)
            .ok_or_else(invalid_internal_data)?;
        let span =
            crate::ByteSpan::from_start_and_len(start, 2).ok_or_else(invalid_internal_data)?;
        document.read_span(span, &mut pair)?;
        let high = hex_nibble(pair[0]).ok_or_else(invalid_internal_data)?;
        let low = hex_nibble(pair[1]).ok_or_else(invalid_internal_data)?;
        *output = (high << 4) | low;
    }
    ensure_not_cancelled(cancellation)
}

const fn double_role(code: i16) -> Option<DxfEntityXDataDoubleRole> {
    use DxfEntityXDataDoubleRole::*;
    match code {
        1010 => Some(PointX),
        1011 => Some(WorldPositionX),
        1012 => Some(WorldDisplacementX),
        1013 => Some(WorldDirectionX),
        1020 => Some(PointY),
        1021 => Some(WorldPositionY),
        1022 => Some(WorldDisplacementY),
        1023 => Some(WorldDirectionY),
        1030 => Some(PointZ),
        1031 => Some(WorldPositionZ),
        1032 => Some(WorldDisplacementZ),
        1033 => Some(WorldDirectionZ),
        1040 => Some(Real),
        1041 => Some(Distance),
        1042 => Some(ScaleFactor),
        _ => None,
    }
}

const fn invalid(
    raw: DxfRawValueProvenance,
    issue: DxfEntityXDataValueIssue,
) -> DxfEntityXDataValue {
    DxfEntityXDataValue::Invalid { issue, raw }
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
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

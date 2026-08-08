//! Canonical destination-byte groups for logical entity XDATA values.

use std::{fmt, io};

use crate::{
    DXF_XDATA_BINARY_CHUNK_MAX_BYTES, DxfAcadVersion, DxfAcadVersionState, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfEntityEditValue, DxfEntityFieldWireType,
    DxfEntityGroupEncodeIssue, DxfEntityGroupEncoder, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataHandleRemap, DxfEntityXDataLogicalDestinationDirectory,
    DxfEntityXDataLogicalDestinationEntry, DxfEntityXDataLogicalDestinationIssue,
    DxfEntityXDataLogicalDestinationState, DxfEntityXDataLogicalDestinationValue,
    DxfEntityXDataTextKind, DxfEntityXDataTypedDirectory, DxfEntityXDataTypedEntry,
    DxfEntityXDataValue, DxfError, DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfRawValueProvenance, DxfResource, DxfResourceProfile, DxfSourceId, DxfTextEncodingResolution,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataDestinationEncodeIssue {
    Group(DxfEntityGroupEncodeIssue),
    TextTranscodingRequired {
        source: DxfTextEncodingResolution,
        destination: DxfTextEncodingResolution,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataEncodedDestinationState {
    LogicalUnavailable(DxfEntityXDataLogicalDestinationIssue),
    DialectUnavailable { state: DxfAcadVersionState },
    EncodingUnavailable(DxfEntityXDataDestinationEncodeIssue),
    Ready { encoded_byte_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataEncodedDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    logical_ordinal: u32,
    byte_start: u32,
    byte_end: u32,
    state: DxfEntityXDataEncodedDestinationState,
}

impl DxfEntityXDataEncodedDestinationEntry {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn logical_ordinal(self) -> u64 {
        self.logical_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataEncodedDestinationState {
        self.state
    }
}

pub struct DxfEntityXDataEncodedDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    destination_format: DxfRawDocumentFormat,
    destination_version_state: DxfAcadVersionState,
    logical: DxfEntityXDataLogicalDestinationDirectory,
    entries: Box<[DxfEntityXDataEncodedDestinationEntry]>,
    bytes: Box<[u8]>,
}

impl DxfEntityXDataEncodedDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let logical = source.entity_xdata_logical_destination_directory(
            destination,
            transform,
            mappings,
            cancellation,
        )?;
        ensure_source(source.source_id(), logical.source_id())?;
        ensure_source(destination.source_id(), logical.destination_id())?;
        let destination_version_state = destination.acad_version_report().state();
        let destination_format = destination.format();
        let mut entries = Vec::new();
        let mut bytes = Vec::new();
        entries
            .try_reserve_exact(logical.entries().len())
            .map_err(|_| out_of_memory())?;
        for logical_entry in logical.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(bytes.len())?;
            let state = encode_entry(
                source,
                destination,
                &logical,
                logical_entry,
                destination_version_state,
                profile,
                cancellation,
                &mut bytes,
            )?;
            entries.push(DxfEntityXDataEncodedDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                logical_ordinal: compact_u64(logical_entry.ordinal())?,
                byte_start: start,
                byte_end: compact_len(bytes.len())?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            destination_format,
            destination_version_state,
            logical,
            entries: entries.into_boxed_slice(),
            bytes: bytes.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.destination_id
    }

    #[must_use]
    pub const fn destination_format(&self) -> DxfRawDocumentFormat {
        self.destination_format
    }

    #[must_use]
    pub const fn destination_version_state(&self) -> DxfAcadVersionState {
        self.destination_version_state
    }

    #[must_use]
    pub const fn logical_destination_directory(
        &self,
    ) -> &DxfEntityXDataLogicalDestinationDirectory {
        &self.logical
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataEncodedDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataEncodedDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn logical_for_entry(
        &self,
        entry: DxfEntityXDataEncodedDestinationEntry,
    ) -> Option<DxfEntityXDataLogicalDestinationEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.logical.entry(entry.logical_ordinal())
    }

    #[must_use]
    pub fn encoded_bytes_for_entry(
        &self,
        entry: DxfEntityXDataEncodedDestinationEntry,
    ) -> Option<&[u8]> {
        if self.entry(entry.ordinal()) != Some(entry)
            || !matches!(
                entry.state(),
                DxfEntityXDataEncodedDestinationState::Ready { .. }
            )
        {
            return None;
        }
        self.bytes
            .get(usize::try_from(entry.byte_start).ok()?..usize::try_from(entry.byte_end).ok()?)
    }

    pub(crate) fn encoded_bytes_for_ready_range(
        &self,
        entries: &[DxfEntityXDataEncodedDestinationEntry],
    ) -> Option<&[u8]> {
        let first = entries.first().copied()?;
        let last = entries.last().copied()?;
        let mut previous_end = first.byte_start;
        for entry in entries.iter().copied() {
            if self.entry(entry.ordinal()) != Some(entry)
                || entry.byte_start != previous_end
                || !matches!(
                    entry.state(),
                    DxfEntityXDataEncodedDestinationState::Ready { .. }
                )
            {
                return None;
            }
            previous_end = entry.byte_end;
        }
        self.bytes
            .get(usize::try_from(first.byte_start).ok()?..usize::try_from(last.byte_end).ok()?)
    }
}

impl fmt::Debug for DxfEntityXDataEncodedDestinationDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityXDataEncodedDestinationDirectory")
            .field("source_id", &self.source_id)
            .field("destination_id", &self.destination_id)
            .field("destination_format", &self.destination_format)
            .field("destination_version_state", &self.destination_version_state)
            .field("logical", &self.logical)
            .field("entries", &self.entries)
            .field("encoded_byte_count", &self.bytes.len())
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_encoded_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataEncodedDestinationDirectory, DxfError> {
        DxfEntityXDataEncodedDestinationDirectory::from_documents(
            self,
            destination,
            transform,
            mappings,
            profile,
            cancellation,
        )
    }
}

macro_rules! document_encoded_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_encoded_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                transform: DxfEntityXDataCoordinateTransform,
                mappings: &[DxfEntityXDataHandleRemap],
                profile: DxfResourceProfile,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataEncodedDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_encoded_destination_directory(
                    destination,
                    transform,
                    mappings,
                    profile,
                    cancellation,
                )
            }
        }
    };
}

document_encoded_destination_directory!(DxfAsciiRawDocument<'_>);
document_encoded_destination_directory!(DxfBinaryRawDocument<'_>);

#[allow(clippy::too_many_arguments)]
fn encode_entry(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    logical: &DxfEntityXDataLogicalDestinationDirectory,
    entry: DxfEntityXDataLogicalDestinationEntry,
    version_state: DxfAcadVersionState,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
    destination_bytes: &mut Vec<u8>,
) -> Result<DxfEntityXDataEncodedDestinationState, DxfError> {
    let DxfEntityXDataLogicalDestinationState::Available(value) = entry.state() else {
        let DxfEntityXDataLogicalDestinationState::Unavailable(issue) = entry.state() else {
            return Err(invalid_internal_data());
        };
        return Ok(DxfEntityXDataEncodedDestinationState::LogicalUnavailable(
            issue,
        ));
    };
    let DxfAcadVersionState::Supported(version) = version_state else {
        return Ok(DxfEntityXDataEncodedDestinationState::DialectUnavailable {
            state: version_state,
        });
    };
    let typed = logical
        .typed_for_entry(entry)
        .ok_or_else(invalid_internal_data)?;
    let encoded = encode_value(
        source,
        destination,
        logical.payload_destination_directory().typed_directory(),
        typed,
        value,
        version,
        profile,
        cancellation,
    )?;
    let encoded = match encoded {
        Ok(encoded) => encoded,
        Err(issue) => {
            return Ok(DxfEntityXDataEncodedDestinationState::EncodingUnavailable(
                issue,
            ));
        }
    };
    let count = compact_len(encoded.len())?;
    destination_bytes
        .try_reserve(encoded.len())
        .map_err(|_| out_of_memory())?;
    destination_bytes.extend_from_slice(&encoded);
    Ok(DxfEntityXDataEncodedDestinationState::Ready {
        encoded_byte_count: count,
    })
}

#[allow(clippy::too_many_arguments)]
fn encode_value(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    typed_directory: &DxfEntityXDataTypedDirectory,
    typed: DxfEntityXDataTypedEntry,
    logical: DxfEntityXDataLogicalDestinationValue,
    version: DxfAcadVersion,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<Vec<u8>, DxfEntityXDataDestinationEncodeIssue>, DxfError> {
    let encoder = DxfEntityGroupEncoder::new(destination.format(), version, profile);
    let code = typed.occurrence().group().group_code().value();
    match logical {
        DxfEntityXDataLogicalDestinationValue::Application { target }
        | DxfEntityXDataLogicalDestinationValue::Layer { target } => {
            let bytes = read_raw(destination, target.name(), profile, cancellation)?;
            encode_raw(
                encoder,
                code,
                DxfEntityFieldWireType::ExactText,
                DxfEntityEditValue::ExactRawText(&bytes),
                cancellation,
            )
        }
        DxfEntityXDataLogicalDestinationValue::Handle { target } => encode_raw(
            encoder,
            code,
            DxfEntityFieldWireType::Handle,
            DxfEntityEditValue::Handle(target),
            cancellation,
        ),
        DxfEntityXDataLogicalDestinationValue::TransformedDouble { value, .. } => encode_raw(
            encoder,
            code,
            DxfEntityFieldWireType::Double,
            DxfEntityEditValue::Double(value),
            cancellation,
        ),
        DxfEntityXDataLogicalDestinationValue::SourceExact(value) => encode_source_exact(
            source,
            destination,
            encoder,
            code,
            typed_directory,
            typed,
            value,
            profile,
            cancellation,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_source_exact(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    encoder: DxfEntityGroupEncoder,
    code: i16,
    typed_directory: &DxfEntityXDataTypedDirectory,
    typed: DxfEntityXDataTypedEntry,
    value: DxfEntityXDataValue,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<Vec<u8>, DxfEntityXDataDestinationEncodeIssue>, DxfError> {
    match value {
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::String,
            raw,
        } => {
            let bytes = read_raw(source, raw, profile, cancellation)?;
            if !portable_text(source, destination, &bytes) {
                return Ok(Err(
                    DxfEntityXDataDestinationEncodeIssue::TextTranscodingRequired {
                        source: source.text_encoding_report().resolution(),
                        destination: destination.text_encoding_report().resolution(),
                    },
                ));
            }
            encode_raw(
                encoder,
                code,
                DxfEntityFieldWireType::ExactText,
                DxfEntityEditValue::ExactRawText(&bytes),
                cancellation,
            )
        }
        DxfEntityXDataValue::Control { raw, .. } => {
            let bytes = read_raw(source, raw, profile, cancellation)?;
            encode_raw(
                encoder,
                code,
                DxfEntityFieldWireType::ExactText,
                DxfEntityEditValue::ExactRawText(&bytes),
                cancellation,
            )
        }
        DxfEntityXDataValue::BinaryChunk { decoded_bytes, .. } => {
            let mut storage = [0_u8; DXF_XDATA_BINARY_CHUNK_MAX_BYTES as usize];
            let bytes = storage
                .get_mut(..usize::from(decoded_bytes))
                .ok_or_else(invalid_internal_data)?;
            typed_directory.read_binary_chunk(source, typed, bytes, cancellation)?;
            encode_raw(
                encoder,
                code,
                DxfEntityFieldWireType::BinaryChunk,
                DxfEntityEditValue::BinaryChunk(bytes),
                cancellation,
            )
        }
        DxfEntityXDataValue::Double { value, .. } => encode_raw(
            encoder,
            code,
            DxfEntityFieldWireType::Double,
            DxfEntityEditValue::Double(value),
            cancellation,
        ),
        DxfEntityXDataValue::Int16 { value, .. } => encode_raw(
            encoder,
            code,
            DxfEntityFieldWireType::Int16,
            DxfEntityEditValue::Int16(value),
            cancellation,
        ),
        DxfEntityXDataValue::Int32 { value, .. } => encode_raw(
            encoder,
            code,
            DxfEntityFieldWireType::Int32,
            DxfEntityEditValue::Int32(value),
            cancellation,
        ),
        DxfEntityXDataValue::ExactText { .. }
        | DxfEntityXDataValue::Handle { .. }
        | DxfEntityXDataValue::Invalid { .. } => Err(invalid_internal_data()),
    }
}

fn encode_raw(
    encoder: DxfEntityGroupEncoder,
    code: i16,
    wire_type: DxfEntityFieldWireType,
    value: DxfEntityEditValue<'_>,
    cancellation: &DxfCancellationToken,
) -> Result<Result<Vec<u8>, DxfEntityXDataDestinationEncodeIssue>, DxfError> {
    Ok(encoder
        .encode_raw(code, wire_type, value, cancellation)?
        .map_err(DxfEntityXDataDestinationEncodeIssue::Group))
}

fn portable_text(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    bytes: &[u8],
) -> bool {
    bytes.is_ascii()
        || source.text_encoding_report().resolution().decoder()
            == destination.text_encoding_report().resolution().decoder()
            && source
                .text_encoding_report()
                .resolution()
                .decoder()
                .is_some()
}

fn read_raw(
    document: DxfRawDocumentView<'_>,
    raw: DxfRawValueProvenance,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Vec<u8>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let observed = raw.value_span().len();
    let limit = profile.limits().max_value_bytes();
    if observed > limit {
        return Err(DxfError::resource_limit(
            DxfResource::ValueBytes,
            limit,
            observed,
        ));
    }
    let len = usize::try_from(observed).map_err(|_| offset_overflow())?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(len).map_err(|_| out_of_memory())?;
    bytes.resize(len, 0);
    document.read_span(raw.value_span(), &mut bytes)?;
    ensure_not_cancelled(cancellation)?;
    Ok(bytes)
}

fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

fn compact_u64(value: u64) -> Result<u32, DxfError> {
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

fn offset_overflow() -> DxfError {
    DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Write, &io::Error::from(kind))
}

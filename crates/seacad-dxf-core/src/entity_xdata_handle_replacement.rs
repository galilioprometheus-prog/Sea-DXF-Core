//! Exact destination-dialect replacement groups for generic XDATA group-1005 handles.

use std::{fmt, io};

use crate::{
    DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityEditValue, DxfEntityFieldWireType, DxfEntityGroupEncodeIssue, DxfEntityGroupEncoder,
    DxfEntityXDataHandleDestinationDirectory, DxfEntityXDataHandleDestinationEntry,
    DxfEntityXDataHandleDestinationState, DxfEntityXDataHandleRemap, DxfError, DxfHandle,
    DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView, DxfResourceProfile, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataHandleReplacementState {
    DestinationUnavailable(DxfEntityXDataHandleDestinationState),
    DialectUnavailable {
        state: DxfAcadVersionState,
    },
    EncodingUnavailable(DxfEntityGroupEncodeIssue),
    Ready {
        target: DxfHandle,
        encoded_byte_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataHandleReplacementEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    destination_ordinal: u32,
    byte_start: u32,
    byte_end: u32,
    state: DxfEntityXDataHandleReplacementState,
}

impl DxfEntityXDataHandleReplacementEntry {
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
    pub const fn destination_ordinal(self) -> u64 {
        self.destination_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataHandleReplacementState {
        self.state
    }
}

pub struct DxfEntityXDataHandleReplacementDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    destination_format: DxfRawDocumentFormat,
    destination_version_state: DxfAcadVersionState,
    destinations: DxfEntityXDataHandleDestinationDirectory,
    entries: Box<[DxfEntityXDataHandleReplacementEntry]>,
    replacement_bytes: Box<[u8]>,
}

impl DxfEntityXDataHandleReplacementDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let destinations = source.entity_xdata_handle_destination_directory(
            destination,
            mappings,
            cancellation,
        )?;
        ensure_source(source.source_id(), destinations.source_id())?;
        ensure_source(destination.source_id(), destinations.destination_id())?;
        let destination_version_state = destination.acad_version_report().state();
        let destination_format = destination.format();
        let mut entries = Vec::new();
        let mut replacement_bytes = Vec::new();
        entries
            .try_reserve_exact(destinations.entries().len())
            .map_err(|_| out_of_memory())?;

        for destination_entry in destinations.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let byte_start = compact_len(replacement_bytes.len())?;
            let state = encode_replacement(
                destination_entry,
                destination_format,
                destination_version_state,
                profile,
                cancellation,
                &mut replacement_bytes,
            )?;
            let byte_end = compact_len(replacement_bytes.len())?;
            entries.push(DxfEntityXDataHandleReplacementEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                destination_ordinal: compact_u64(destination_entry.ordinal())?,
                byte_start,
                byte_end,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            destination_format,
            destination_version_state,
            destinations,
            entries: entries.into_boxed_slice(),
            replacement_bytes: replacement_bytes.into_boxed_slice(),
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
    pub const fn destination_directory(&self) -> &DxfEntityXDataHandleDestinationDirectory {
        &self.destinations
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataHandleReplacementEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataHandleReplacementEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn destination_for_entry(
        &self,
        entry: DxfEntityXDataHandleReplacementEntry,
    ) -> Option<DxfEntityXDataHandleDestinationEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.destinations.entry(entry.destination_ordinal())
    }

    #[must_use]
    pub fn replacement_bytes_for_entry(
        &self,
        entry: DxfEntityXDataHandleReplacementEntry,
    ) -> Option<&[u8]> {
        if self.entry(entry.ordinal()) != Some(entry)
            || !matches!(
                entry.state(),
                DxfEntityXDataHandleReplacementState::Ready { .. }
            )
        {
            return None;
        }
        self.replacement_bytes
            .get(usize::try_from(entry.byte_start).ok()?..usize::try_from(entry.byte_end).ok()?)
    }
}

impl fmt::Debug for DxfEntityXDataHandleReplacementDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityXDataHandleReplacementDirectory")
            .field("source_id", &self.source_id)
            .field("destination_id", &self.destination_id)
            .field("destination_format", &self.destination_format)
            .field("destination_version_state", &self.destination_version_state)
            .field("destinations", &self.destinations)
            .field("entries", &self.entries)
            .field("replacement_byte_count", &self.replacement_bytes.len())
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_handle_replacement_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataHandleReplacementDirectory, DxfError> {
        DxfEntityXDataHandleReplacementDirectory::from_documents(
            self,
            destination,
            mappings,
            profile,
            cancellation,
        )
    }
}

macro_rules! document_handle_replacement_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_handle_replacement_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                mappings: &[DxfEntityXDataHandleRemap],
                profile: DxfResourceProfile,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataHandleReplacementDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_handle_replacement_directory(
                    destination,
                    mappings,
                    profile,
                    cancellation,
                )
            }
        }
    };
}

document_handle_replacement_directory!(DxfAsciiRawDocument<'_>);
document_handle_replacement_directory!(DxfBinaryRawDocument<'_>);

fn encode_replacement(
    entry: DxfEntityXDataHandleDestinationEntry,
    format: DxfRawDocumentFormat,
    version_state: DxfAcadVersionState,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
    bytes: &mut Vec<u8>,
) -> Result<DxfEntityXDataHandleReplacementState, DxfError> {
    let DxfEntityXDataHandleDestinationState::Unique { target } = entry.state() else {
        return Ok(DxfEntityXDataHandleReplacementState::DestinationUnavailable(entry.state()));
    };
    let DxfAcadVersionState::Supported(version) = version_state else {
        return Ok(DxfEntityXDataHandleReplacementState::DialectUnavailable {
            state: version_state,
        });
    };
    let encoded = DxfEntityGroupEncoder::new(format, version, profile).encode_raw(
        1005,
        DxfEntityFieldWireType::Handle,
        DxfEntityEditValue::Handle(target),
        cancellation,
    )?;
    let encoded = match encoded {
        Ok(encoded) => encoded,
        Err(issue) => {
            return Ok(DxfEntityXDataHandleReplacementState::EncodingUnavailable(
                issue,
            ));
        }
    };
    let encoded_byte_count = compact_len(encoded.len())?;
    bytes
        .try_reserve(encoded.len())
        .map_err(|_| out_of_memory())?;
    bytes.extend_from_slice(&encoded);
    Ok(DxfEntityXDataHandleReplacementState::Ready {
        target,
        encoded_byte_count,
    })
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

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

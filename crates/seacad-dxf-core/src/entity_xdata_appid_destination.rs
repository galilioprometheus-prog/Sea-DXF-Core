//! Destination-document APPID validation for generic entity XDATA applications.
//!
//! A unique result proves only byte-exact registration in one completely closed destination
//! APPID table. It does not validate APPID syntax, XDATA structure or capacity, application
//! semantics, or whether an XDATA payload is otherwise ready to insert or clone.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataAppIdResolutionDirectory, DxfEntityXDataAppIdResolutionEntry,
    DxfEntityXDataAppIdResolutionState, DxfEntityXDataApplication, DxfError, DxfIoOperation,
    DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind,
    DxfRawDocumentView, DxfSourceId, named_symbol_destination::NamedSymbolDestinationIndex,
};

/// Exact source-resolution precedence followed by destination APPID lookup.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataAppIdDestinationState {
    /// The source application's group-1001 name has no exact source APPID record.
    SourceMissing,
    /// The source application's group-1001 name has multiple exact source APPID records.
    SourceAmbiguous { target_count: u32 },
    /// A source-unique APPID name has no exact destination APPID record.
    DestinationMissing,
    /// A source-unique APPID name has exactly one exact destination APPID record.
    DestinationUnique { target: DxfNamedSymbolTableEntry },
    /// A source-unique APPID name has multiple exact destination APPID records.
    DestinationAmbiguous { target_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataAppIdDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    source_resolution_ordinal: u32,
    state: DxfEntityXDataAppIdDestinationState,
}

impl DxfEntityXDataAppIdDestinationEntry {
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
    pub const fn source_resolution_ordinal(self) -> u64 {
        self.source_resolution_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataAppIdDestinationState {
        self.state
    }
}

/// Exact source APPID states composed with a separately opened destination.
///
/// The directory owns both the source-resolution evidence and the destination symbol-table
/// evidence. Digest equality only narrows candidates; authoritative name spans are compared
/// byte-for-byte across the two documents before a destination state is published.
#[derive(Debug)]
pub struct DxfEntityXDataAppIdDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    source_resolutions: DxfEntityXDataAppIdResolutionDirectory,
    destination_symbols: DxfNamedSymbolTableDirectory,
    entries: Box<[DxfEntityXDataAppIdDestinationEntry]>,
}

impl DxfEntityXDataAppIdDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source_resolutions = source.entity_xdata_appid_resolution_directory(cancellation)?;
        let destination_symbols = destination.named_symbol_table_directory(cancellation)?;
        ensure_source(source.source_id(), source_resolutions.source_id())?;
        ensure_source(destination.source_id(), destination_symbols.source_id())?;
        let index = NamedSymbolDestinationIndex::from_directory(
            destination,
            &destination_symbols,
            DxfNamedSymbolTableKind::AppId,
            cancellation,
        )?;

        let mut entries = Vec::new();
        entries
            .try_reserve_exact(source_resolutions.entries().len())
            .map_err(|_| out_of_memory())?;
        for resolution in source_resolutions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = destination_state(source, destination, resolution, &index, cancellation)?;
            entries.push(DxfEntityXDataAppIdDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                source_resolution_ordinal: compact_u64(resolution.application().ordinal())?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            source_resolutions,
            destination_symbols,
            entries: entries.into_boxed_slice(),
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
    pub const fn source_resolution_directory(&self) -> &DxfEntityXDataAppIdResolutionDirectory {
        &self.source_resolutions
    }

    #[must_use]
    pub const fn destination_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.destination_symbols
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataAppIdDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataAppIdDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn source_resolution_for_entry(
        &self,
        entry: DxfEntityXDataAppIdDestinationEntry,
    ) -> Option<DxfEntityXDataAppIdResolutionEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.source_resolutions
            .entry(entry.source_resolution_ordinal())
    }

    #[must_use]
    pub fn destination_target_for_entry(
        &self,
        entry: DxfEntityXDataAppIdDestinationEntry,
    ) -> Option<DxfNamedSymbolTableEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        let DxfEntityXDataAppIdDestinationState::DestinationUnique { target } = entry.state()
        else {
            return None;
        };
        self.destination_symbols
            .entry_for_raw_ordinal(target.record().ordinal())
            .filter(|observed| {
                *observed == target && observed.kind() == DxfNamedSymbolTableKind::AppId
            })
    }

    pub fn entry_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<DxfEntityXDataAppIdDestinationEntry, DxfError> {
        let resolution = self.source_resolutions.entry_for_application(application)?;
        self.entry(application.ordinal())
            .filter(|entry| self.source_resolution_for_entry(*entry) == Some(resolution))
            .ok_or_else(invalid_internal_data)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataAppIdDestinationEntry], DxfError> {
        let resolutions = self.source_resolutions.entries_for_entity(entity)?;
        let Some(first) = resolutions.first() else {
            return self.entries.get(0..0).ok_or_else(invalid_internal_data);
        };
        let start =
            usize::try_from(first.application().ordinal()).map_err(|_| invalid_internal_data())?;
        let end = start
            .checked_add(resolutions.len())
            .ok_or_else(invalid_internal_data)?;
        self.entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_appid_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataAppIdDestinationDirectory, DxfError> {
        DxfEntityXDataAppIdDestinationDirectory::from_documents(self, destination, cancellation)
    }
}

macro_rules! document_appid_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_appid_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataAppIdDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_appid_destination_directory(destination, cancellation)
            }
        }
    };
}

document_appid_destination_directory!(DxfAsciiRawDocument<'_>);
document_appid_destination_directory!(DxfBinaryRawDocument<'_>);

fn destination_state(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    resolution: DxfEntityXDataAppIdResolutionEntry,
    index: &NamedSymbolDestinationIndex,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityXDataAppIdDestinationState, DxfError> {
    match resolution.state() {
        DxfEntityXDataAppIdResolutionState::Missing => {
            Ok(DxfEntityXDataAppIdDestinationState::SourceMissing)
        }
        DxfEntityXDataAppIdResolutionState::Ambiguous { target_count } => {
            Ok(DxfEntityXDataAppIdDestinationState::SourceAmbiguous { target_count })
        }
        DxfEntityXDataAppIdResolutionState::Unique { .. } => {
            let span = resolution
                .application()
                .application_name()
                .value_payload_span();
            let (target, target_count) =
                index.exact_matches(source, span, destination, cancellation)?;
            match (target, target_count) {
                (None, 0) => Ok(DxfEntityXDataAppIdDestinationState::DestinationMissing),
                (Some(target), 1) => {
                    Ok(DxfEntityXDataAppIdDestinationState::DestinationUnique { target })
                }
                (Some(_), target_count) => {
                    Ok(DxfEntityXDataAppIdDestinationState::DestinationAmbiguous { target_count })
                }
                (None, _) => Err(invalid_internal_data()),
            }
        }
    }
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

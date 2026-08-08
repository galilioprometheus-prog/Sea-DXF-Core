//! Destination-document LAYER validation for generic entity XDATA group 1003.
//!
//! A unique result proves only byte-exact registration in one completely closed destination
//! LAYER table. It does not validate layer-name syntax, XDATA structure or capacity, application
//! semantics, or whether an XDATA payload is otherwise ready to insert or clone.

use std::io;

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplication, DxfEntityXDataLayerResolutionDirectory,
    DxfEntityXDataLayerResolutionEntry, DxfEntityXDataLayerResolutionState, DxfEntityXDataTextKind,
    DxfEntityXDataValue, DxfError, DxfIoOperation, DxfNamedSymbolTableDirectory,
    DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind, DxfRawDocumentView, DxfSourceId,
    named_symbol_destination::NamedSymbolDestinationIndex,
};

/// Exact source-resolution precedence followed by destination LAYER lookup.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataLayerDestinationState {
    /// The group-1003 name has no exact source LAYER record.
    SourceMissing,
    /// The group-1003 name has multiple exact source LAYER records.
    SourceAmbiguous { target_count: u32 },
    /// A source-unique layer name has no exact destination LAYER record.
    DestinationMissing,
    /// A source-unique layer name has exactly one exact destination LAYER record.
    DestinationUnique { target: DxfNamedSymbolTableEntry },
    /// A source-unique layer name has multiple exact destination LAYER records.
    DestinationAmbiguous { target_count: u32 },
}

/// One source group-1003 resolution composed with destination LAYER evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataLayerDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    source_resolution_ordinal: u32,
    state: DxfEntityXDataLayerDestinationState,
}

impl DxfEntityXDataLayerDestinationEntry {
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
    pub const fn state(self) -> DxfEntityXDataLayerDestinationState {
        self.state
    }
}

/// Exact source LAYER states composed with a separately opened destination.
///
/// The directory owns both source-resolution evidence and destination symbol-table evidence.
/// Digest equality only narrows candidates; authoritative name spans are compared byte-for-byte
/// across the two documents before a destination state is published.
#[derive(Debug)]
pub struct DxfEntityXDataLayerDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    source_resolutions: DxfEntityXDataLayerResolutionDirectory,
    destination_symbols: DxfNamedSymbolTableDirectory,
    entries: Box<[DxfEntityXDataLayerDestinationEntry]>,
}

impl DxfEntityXDataLayerDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source_resolutions = source.entity_xdata_layer_resolution_directory(cancellation)?;
        let destination_symbols = destination.named_symbol_table_directory(cancellation)?;
        ensure_source(source.source_id(), source_resolutions.source_id())?;
        ensure_source(destination.source_id(), destination_symbols.source_id())?;
        let index = NamedSymbolDestinationIndex::from_directory(
            destination,
            &destination_symbols,
            DxfNamedSymbolTableKind::Layer,
            cancellation,
        )?;

        let mut entries = Vec::new();
        entries
            .try_reserve_exact(source_resolutions.entries().len())
            .map_err(|_| out_of_memory())?;
        for resolution in source_resolutions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = destination_state(
                source,
                destination,
                &source_resolutions,
                resolution,
                &index,
                cancellation,
            )?;
            entries.push(DxfEntityXDataLayerDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                source_resolution_ordinal: compact_u64(resolution.ordinal())?,
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
    pub const fn source_resolution_directory(&self) -> &DxfEntityXDataLayerResolutionDirectory {
        &self.source_resolutions
    }

    #[must_use]
    pub const fn destination_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.destination_symbols
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataLayerDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataLayerDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn source_resolution_for_entry(
        &self,
        entry: DxfEntityXDataLayerDestinationEntry,
    ) -> Option<DxfEntityXDataLayerResolutionEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.source_resolutions
            .entry(entry.source_resolution_ordinal())
    }

    #[must_use]
    pub fn destination_target_for_entry(
        &self,
        entry: DxfEntityXDataLayerDestinationEntry,
    ) -> Option<DxfNamedSymbolTableEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        let DxfEntityXDataLayerDestinationState::DestinationUnique { target } = entry.state()
        else {
            return None;
        };
        self.destination_symbols
            .entry_for_raw_ordinal(target.record().ordinal())
            .filter(|observed| {
                *observed == target && observed.kind() == DxfNamedSymbolTableKind::Layer
            })
    }

    #[must_use]
    pub fn entry_for_source_resolution(
        &self,
        resolution: DxfEntityXDataLayerResolutionEntry,
    ) -> Option<DxfEntityXDataLayerDestinationEntry> {
        if self.source_resolutions.entry(resolution.ordinal()) != Some(resolution) {
            return None;
        }
        self.entry(resolution.ordinal())
            .filter(|entry| self.source_resolution_for_entry(*entry) == Some(resolution))
    }

    pub fn entries_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<&[DxfEntityXDataLayerDestinationEntry], DxfError> {
        let resolutions = self
            .source_resolutions
            .entries_for_application(application)?;
        self.entries_for_resolutions(resolutions)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataLayerDestinationEntry], DxfError> {
        let resolutions = self.source_resolutions.entries_for_entity(entity)?;
        self.entries_for_resolutions(resolutions)
    }

    fn entries_for_resolutions(
        &self,
        resolutions: &[DxfEntityXDataLayerResolutionEntry],
    ) -> Result<&[DxfEntityXDataLayerDestinationEntry], DxfError> {
        let Some(first) = resolutions.first() else {
            return self.entries.get(0..0).ok_or_else(invalid_internal_data);
        };
        let start = usize::try_from(first.ordinal()).map_err(|_| invalid_internal_data())?;
        let end = start
            .checked_add(resolutions.len())
            .ok_or_else(invalid_internal_data)?;
        let entries = self
            .entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)?;
        if entries
            .first()
            .and_then(|entry| self.source_resolution_for_entry(*entry))
            != Some(*first)
        {
            return Err(invalid_internal_data());
        }
        Ok(entries)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_layer_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataLayerDestinationDirectory, DxfError> {
        DxfEntityXDataLayerDestinationDirectory::from_documents(self, destination, cancellation)
    }
}

macro_rules! document_layer_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_layer_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataLayerDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_layer_destination_directory(destination, cancellation)
            }
        }
    };
}

document_layer_destination_directory!(DxfAsciiRawDocument<'_>);
document_layer_destination_directory!(DxfBinaryRawDocument<'_>);

fn destination_state(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    source_resolutions: &DxfEntityXDataLayerResolutionDirectory,
    resolution: DxfEntityXDataLayerResolutionEntry,
    index: &NamedSymbolDestinationIndex,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityXDataLayerDestinationState, DxfError> {
    match resolution.state() {
        DxfEntityXDataLayerResolutionState::Missing => {
            Ok(DxfEntityXDataLayerDestinationState::SourceMissing)
        }
        DxfEntityXDataLayerResolutionState::Ambiguous { target_count } => {
            Ok(DxfEntityXDataLayerDestinationState::SourceAmbiguous { target_count })
        }
        DxfEntityXDataLayerResolutionState::Unique { .. } => {
            let typed = source_resolutions
                .typed_entry(resolution)
                .ok_or_else(invalid_internal_data)?;
            let span = layer_span(typed.value()).ok_or_else(invalid_internal_data)?;
            let (target, target_count) =
                index.exact_matches(source, span, destination, cancellation)?;
            match (target, target_count) {
                (None, 0) => Ok(DxfEntityXDataLayerDestinationState::DestinationMissing),
                (Some(target), 1) => {
                    Ok(DxfEntityXDataLayerDestinationState::DestinationUnique { target })
                }
                (Some(_), target_count) => {
                    Ok(DxfEntityXDataLayerDestinationState::DestinationAmbiguous { target_count })
                }
                (None, _) => Err(invalid_internal_data()),
            }
        }
    }
}

fn layer_span(value: DxfEntityXDataValue) -> Option<ByteSpan> {
    match value {
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::LayerName,
            raw,
        } => Some(raw.value_span()),
        _ => None,
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

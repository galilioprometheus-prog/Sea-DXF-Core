//! Per-application generic XDATA destination readiness for symbols and list structure.
//!
//! This composes exact destination APPID/LAYER evidence with source structural validation. It
//! does not validate capacity, transformed coordinates, mapped handles, payload semantics, or
//! encoding.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplication, DxfEntityXDataLayerDestinationEntry,
    DxfEntityXDataStructureDirectory, DxfEntityXDataStructureEntry, DxfEntityXDataStructureIssue,
    DxfEntityXDataStructureState, DxfEntityXDataSymbolDestinationDirectory,
    DxfEntityXDataSymbolDestinationEntry, DxfEntityXDataSymbolDestinationIssue,
    DxfEntityXDataSymbolDestinationState, DxfError, DxfIoOperation, DxfNamedSymbolTableEntry,
    DxfRawDocumentView, DxfSourceId,
};

/// Exact symbol and structure readiness for one source XDATA application.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataApplicationDestinationState {
    Ready {
        destination_layer_occurrence_count: u32,
    },
    Unavailable {
        symbol_issue_count: u32,
        structure_issue_count: u32,
    },
}

/// One source application composed with exact destination symbols and source structure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataApplicationDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    application_ordinal: u32,
    state: DxfEntityXDataApplicationDestinationState,
}

impl DxfEntityXDataApplicationDestinationEntry {
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
    pub const fn application_ordinal(self) -> u64 {
        self.application_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataApplicationDestinationState {
        self.state
    }
}

/// Source-order application readiness for exact destination symbols and valid list structure.
#[derive(Debug)]
pub struct DxfEntityXDataApplicationDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    symbols: DxfEntityXDataSymbolDestinationDirectory,
    structure: DxfEntityXDataStructureDirectory,
    entries: Box<[DxfEntityXDataApplicationDestinationEntry]>,
}

impl DxfEntityXDataApplicationDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let symbols =
            source.entity_xdata_symbol_destination_directory(destination, cancellation)?;
        let structure = source.entity_xdata_structure_directory(cancellation)?;
        for observed in [symbols.source_id(), structure.source_id()] {
            ensure_source(source.source_id(), observed)?;
        }
        ensure_source(destination.source_id(), symbols.destination_id())?;

        let applications = symbols
            .appid_destination_directory()
            .source_resolution_directory()
            .xdata_directory()
            .applications();
        if structure.entries().len() != applications.len() {
            return Err(invalid_internal_data());
        }
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(applications.len())
            .map_err(|_| out_of_memory())?;
        for application in applications.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let symbol = symbols.entry_for_application(application)?;
            let structural = structure.entry_for_application(application)?;
            entries.push(DxfEntityXDataApplicationDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                application_ordinal: compact_u64(application.ordinal())?,
                state: readiness_state(symbol, structural),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            symbols,
            structure,
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
    pub const fn symbol_destination_directory(&self) -> &DxfEntityXDataSymbolDestinationDirectory {
        &self.symbols
    }

    #[must_use]
    pub const fn structure_directory(&self) -> &DxfEntityXDataStructureDirectory {
        &self.structure
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataApplicationDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataApplicationDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entry_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<DxfEntityXDataApplicationDestinationEntry, DxfError> {
        let symbol = self.symbols.entry_for_application(application)?;
        let structural = self.structure.entry_for_application(application)?;
        self.entry(application.ordinal())
            .filter(|entry| {
                self.application_for_entry(*entry) == Some(application)
                    && self.symbol_entry_for_entry(*entry) == Some(symbol)
                    && self.structure_entry_for_entry(*entry) == Some(structural)
            })
            .ok_or_else(invalid_internal_data)
    }

    #[must_use]
    pub fn application_for_entry(
        &self,
        entry: DxfEntityXDataApplicationDestinationEntry,
    ) -> Option<DxfEntityXDataApplication> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.symbols
            .appid_destination_directory()
            .source_resolution_directory()
            .xdata_directory()
            .application(entry.application_ordinal())
    }

    #[must_use]
    pub fn symbol_entry_for_entry(
        &self,
        entry: DxfEntityXDataApplicationDestinationEntry,
    ) -> Option<DxfEntityXDataSymbolDestinationEntry> {
        let application = self.application_for_entry(entry)?;
        self.symbols.entry_for_application(application).ok()
    }

    #[must_use]
    pub fn structure_entry_for_entry(
        &self,
        entry: DxfEntityXDataApplicationDestinationEntry,
    ) -> Option<DxfEntityXDataStructureEntry> {
        let application = self.application_for_entry(entry)?;
        self.structure.entry_for_application(application).ok()
    }

    pub fn symbol_issues_for_entry(
        &self,
        entry: DxfEntityXDataApplicationDestinationEntry,
    ) -> Result<&[DxfEntityXDataSymbolDestinationIssue], DxfError> {
        let symbol = self
            .symbol_entry_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.symbols.issues_for_entry(symbol)
    }

    pub fn structure_issues_for_entry(
        &self,
        entry: DxfEntityXDataApplicationDestinationEntry,
    ) -> Result<&[DxfEntityXDataStructureIssue], DxfError> {
        let structural = self
            .structure_entry_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.structure.issues_for_entry(structural)
    }

    pub fn layer_entries_for_entry(
        &self,
        entry: DxfEntityXDataApplicationDestinationEntry,
    ) -> Result<&[DxfEntityXDataLayerDestinationEntry], DxfError> {
        let symbol = self
            .symbol_entry_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.symbols.layer_entries_for_entry(symbol)
    }

    #[must_use]
    pub fn destination_appid_target_for_entry(
        &self,
        entry: DxfEntityXDataApplicationDestinationEntry,
    ) -> Option<DxfNamedSymbolTableEntry> {
        let DxfEntityXDataApplicationDestinationState::Ready { .. } = entry.state() else {
            return None;
        };
        let symbol = self.symbol_entry_for_entry(entry)?;
        self.symbols.destination_appid_target_for_entry(symbol)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataApplicationDestinationEntry], DxfError> {
        let symbols = self.symbols.entries_for_entity(entity)?;
        let Some(first) = symbols.first() else {
            return self.entries.get(0..0).ok_or_else(invalid_internal_data);
        };
        let start =
            usize::try_from(first.application_ordinal()).map_err(|_| invalid_internal_data())?;
        let end = start
            .checked_add(symbols.len())
            .ok_or_else(invalid_internal_data)?;
        self.entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_application_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataApplicationDestinationDirectory, DxfError> {
        DxfEntityXDataApplicationDestinationDirectory::from_documents(
            self,
            destination,
            cancellation,
        )
    }
}

macro_rules! document_application_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_application_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataApplicationDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_application_destination_directory(destination, cancellation)
            }
        }
    };
}

document_application_destination_directory!(DxfAsciiRawDocument<'_>);
document_application_destination_directory!(DxfBinaryRawDocument<'_>);

fn readiness_state(
    symbol: DxfEntityXDataSymbolDestinationEntry,
    structure: DxfEntityXDataStructureEntry,
) -> DxfEntityXDataApplicationDestinationState {
    let (symbol_issue_count, destination_layer_occurrence_count) = match symbol.state() {
        DxfEntityXDataSymbolDestinationState::Ready {
            destination_layer_occurrence_count,
        } => (0, destination_layer_occurrence_count),
        DxfEntityXDataSymbolDestinationState::Unavailable { issue_count } => (issue_count, 0),
    };
    let structure_issue_count = match structure.state() {
        DxfEntityXDataStructureState::Valid => 0,
        DxfEntityXDataStructureState::Invalid { issue_count } => issue_count,
    };
    if symbol_issue_count == 0 && structure_issue_count == 0 {
        DxfEntityXDataApplicationDestinationState::Ready {
            destination_layer_occurrence_count,
        }
    } else {
        DxfEntityXDataApplicationDestinationState::Unavailable {
            symbol_issue_count,
            structure_issue_count,
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

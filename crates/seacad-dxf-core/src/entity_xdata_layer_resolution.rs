//! Collision-safe exact LAYER resolution for entity XDATA group 1003.

use std::io;

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplication, DxfEntityXDataTextKind, DxfEntityXDataTypedDirectory,
    DxfEntityXDataTypedEntry, DxfEntityXDataValue, DxfError, DxfIoOperation,
    DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind,
    DxfRawDocumentView, DxfSourceId,
    source_span::{sha256_span, spans_equal},
};

type NameDigest = [u8; 32];

/// Exact group-1003 lookup result against completely closed LAYER tables.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataLayerResolutionState {
    Missing,
    Unique { target: DxfNamedSymbolTableEntry },
    Ambiguous { target_count: u32 },
}

/// One source group-1003 entry and its exact LAYER lookup state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataLayerResolutionEntry {
    ordinal: u32,
    source_id: DxfSourceId,
    source_ordinal: u32,
    entity_ordinal: u32,
    state: DxfEntityXDataLayerResolutionState,
}

impl DxfEntityXDataLayerResolutionEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_entry_ordinal(self) -> u64 {
        self.source_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataLayerResolutionState {
        self.state
    }
}

#[derive(Clone, Copy)]
struct LayerIndexEntry {
    digest: NameDigest,
    target: DxfNamedSymbolTableEntry,
}

/// Immutable exact layer resolutions for all retained group-1003 occurrences.
#[derive(Debug)]
pub struct DxfEntityXDataLayerResolutionDirectory {
    source_id: DxfSourceId,
    typed: DxfEntityXDataTypedDirectory,
    named: DxfNamedSymbolTableDirectory,
    entries: Box<[DxfEntityXDataLayerResolutionEntry]>,
}

impl DxfEntityXDataLayerResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let typed = document.entity_xdata_typed_directory(cancellation)?;
        let named = document.named_symbol_table_directory(cancellation)?;
        ensure_source(document.source_id(), typed.source_id())?;
        ensure_source(document.source_id(), named.source_id())?;
        let index = build_index(document, &named, cancellation)?;
        let layer_count = typed
            .entries()
            .iter()
            .filter(|entry| layer_span(**entry).is_some())
            .count();
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(layer_count)
            .map_err(|_| out_of_memory())?;
        for source in typed.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let Some(span) = layer_span(source) else {
                continue;
            };
            let (target, target_count) = exact_matches(document, &index, span, cancellation)?;
            let state = match (target, target_count) {
                (None, 0) => DxfEntityXDataLayerResolutionState::Missing,
                (Some(target), 1) => DxfEntityXDataLayerResolutionState::Unique { target },
                (Some(_), target_count) => {
                    DxfEntityXDataLayerResolutionState::Ambiguous { target_count }
                }
                (None, _) => return Err(invalid_internal_data()),
            };
            entries.push(DxfEntityXDataLayerResolutionEntry {
                ordinal: compact_len(entries.len())?,
                source_id: document.source_id(),
                source_ordinal: compact_u64(source.ordinal())?,
                entity_ordinal: compact_u64(source.occurrence().entity().record().ordinal())?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            typed,
            named,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn typed_directory(&self) -> &DxfEntityXDataTypedDirectory {
        &self.typed
    }

    #[must_use]
    pub const fn named_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.named
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataLayerResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataLayerResolutionEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_source(
        &self,
        source: DxfEntityXDataTypedEntry,
    ) -> Option<DxfEntityXDataLayerResolutionEntry> {
        if self.typed.entry(source.ordinal()) != Some(source) {
            return None;
        }
        self.entries
            .binary_search_by_key(&source.ordinal(), |entry| entry.source_entry_ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn typed_entry(
        &self,
        entry: DxfEntityXDataLayerResolutionEntry,
    ) -> Option<DxfEntityXDataTypedEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.typed.entry(entry.source_entry_ordinal())
    }

    pub fn entries_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<&[DxfEntityXDataLayerResolutionEntry], DxfError> {
        ensure_source(self.source_id, application.entity().source_id())?;
        if self
            .typed
            .xdata_directory()
            .application(application.ordinal())
            != Some(application)
        {
            return Err(invalid_internal_data());
        }
        let range = application.occurrence_range();
        let start = self
            .entries
            .partition_point(|entry| entry.source_entry_ordinal() < range.start());
        let end = self
            .entries
            .partition_point(|entry| entry.source_entry_ordinal() < range.end());
        self.entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataLayerResolutionEntry], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let ordinal = entity.record().ordinal();
        let start = self
            .entries
            .partition_point(|entry| u64::from(entry.entity_ordinal) < ordinal);
        let end = self
            .entries
            .partition_point(|entry| u64::from(entry.entity_ordinal) <= ordinal);
        self.entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_layer_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataLayerResolutionDirectory, DxfError> {
        DxfEntityXDataLayerResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_xdata_layer_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataLayerResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_layer_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_xdata_layer_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataLayerResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_layer_resolution_directory(cancellation)
    }
}

fn layer_span(entry: DxfEntityXDataTypedEntry) -> Option<ByteSpan> {
    match entry.value() {
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::LayerName,
            raw,
        } => Some(raw.value_span()),
        _ => None,
    }
}

fn build_index(
    document: DxfRawDocumentView<'_>,
    named: &DxfNamedSymbolTableDirectory,
    cancellation: &DxfCancellationToken,
) -> Result<Box<[LayerIndexEntry]>, DxfError> {
    let count = named
        .entries()
        .iter()
        .filter(|entry| entry.kind() == DxfNamedSymbolTableKind::Layer)
        .count();
    let mut index = Vec::new();
    index
        .try_reserve_exact(count)
        .map_err(|_| out_of_memory())?;
    for target in named
        .entries()
        .iter()
        .copied()
        .filter(|entry| entry.kind() == DxfNamedSymbolTableKind::Layer)
    {
        ensure_not_cancelled(cancellation)?;
        index.push(LayerIndexEntry {
            digest: sha256_span(document, target.name().value_span(), cancellation)?,
            target,
        });
    }
    index.sort_unstable_by_key(|entry| (entry.digest, entry.target.record().ordinal()));
    Ok(index.into_boxed_slice())
}

fn exact_matches(
    document: DxfRawDocumentView<'_>,
    index: &[LayerIndexEntry],
    span: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<(Option<DxfNamedSymbolTableEntry>, u32), DxfError> {
    let digest = sha256_span(document, span, cancellation)?;
    let start = index.partition_point(|entry| entry.digest < digest);
    let end = index.partition_point(|entry| entry.digest <= digest);
    let mut first = None;
    let mut count = 0_u32;
    for candidate in index.get(start..end).ok_or_else(invalid_internal_data)? {
        ensure_not_cancelled(cancellation)?;
        if spans_equal(
            document,
            span,
            candidate.target.name().value_span(),
            cancellation,
        )? {
            count = count.checked_add(1).ok_or_else(invalid_internal_data)?;
            first.get_or_insert(candidate.target);
        }
    }
    Ok((first, count))
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

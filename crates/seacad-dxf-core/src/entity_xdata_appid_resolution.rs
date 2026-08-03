//! Collision-safe exact APPID resolution for entity XDATA applications.

use std::io;

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplication, DxfEntityXDataDirectory, DxfError, DxfIoOperation,
    DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind,
    DxfRawDocumentView, DxfSourceId,
    source_span::{sha256_span, spans_equal},
};

type NameDigest = [u8; 32];

/// Result of resolving one group-1001 application name against exact APPID names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataAppIdResolutionState {
    Missing,
    Unique { target: DxfNamedSymbolTableEntry },
    Ambiguous { target_count: u32 },
}

/// One source-anchored XDATA application and its exact APPID lookup result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataAppIdResolutionEntry {
    application: DxfEntityXDataApplication,
    state: DxfEntityXDataAppIdResolutionState,
}

impl DxfEntityXDataAppIdResolutionEntry {
    #[must_use]
    pub const fn application(self) -> DxfEntityXDataApplication {
        self.application
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataAppIdResolutionState {
        self.state
    }
}

#[derive(Clone, Copy)]
struct AppIdIndexEntry {
    digest: NameDigest,
    target: DxfNamedSymbolTableEntry,
}

/// Immutable source-order exact APPID resolutions for indexed entity XDATA.
#[derive(Debug)]
pub struct DxfEntityXDataAppIdResolutionDirectory {
    source_id: DxfSourceId,
    xdata: DxfEntityXDataDirectory,
    named_symbols: DxfNamedSymbolTableDirectory,
    entries: Box<[DxfEntityXDataAppIdResolutionEntry]>,
}

impl DxfEntityXDataAppIdResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let xdata = document.entity_xdata_directory(cancellation)?;
        let named_symbols = document.named_symbol_table_directory(cancellation)?;
        ensure_source(document.source_id(), xdata.source_id())?;
        ensure_source(document.source_id(), named_symbols.source_id())?;

        let index = build_index(document, &named_symbols, cancellation)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(xdata.applications().len())
            .map_err(|_| out_of_memory())?;
        for application in xdata.applications().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let span = application.application_name().value_payload_span();
            let (target, target_count) = exact_matches(document, &index, span, cancellation)?;
            let state = match (target, target_count) {
                (None, 0) => DxfEntityXDataAppIdResolutionState::Missing,
                (Some(target), 1) => DxfEntityXDataAppIdResolutionState::Unique { target },
                (Some(_), target_count) => {
                    DxfEntityXDataAppIdResolutionState::Ambiguous { target_count }
                }
                (None, _) => return Err(invalid_internal_data()),
            };
            entries.push(DxfEntityXDataAppIdResolutionEntry { application, state });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            xdata,
            named_symbols,
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
    pub const fn named_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.named_symbols
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataAppIdResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataAppIdResolutionEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entry_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<DxfEntityXDataAppIdResolutionEntry, DxfError> {
        ensure_source(self.source_id, application.entity().source_id())?;
        self.entry(application.ordinal())
            .filter(|entry| entry.application() == application)
            .ok_or_else(invalid_internal_data)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataAppIdResolutionEntry], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let ordinal = entity.record().ordinal();
        let start = self
            .entries
            .partition_point(|entry| entry.application().entity().record().ordinal() < ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.application().entity().record().ordinal() <= ordinal);
        self.entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    /// Resolves group-1001 names against exact entries in closed APPID tables.
    pub fn entity_xdata_appid_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataAppIdResolutionDirectory, DxfError> {
        DxfEntityXDataAppIdResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_xdata_appid_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataAppIdResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_appid_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_xdata_appid_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataAppIdResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_appid_resolution_directory(cancellation)
    }
}

fn build_index(
    document: DxfRawDocumentView<'_>,
    named_symbols: &DxfNamedSymbolTableDirectory,
    cancellation: &DxfCancellationToken,
) -> Result<Box<[AppIdIndexEntry]>, DxfError> {
    let appid_count = named_symbols
        .entries()
        .iter()
        .filter(|entry| entry.kind() == DxfNamedSymbolTableKind::AppId)
        .count();
    let mut index = Vec::new();
    index
        .try_reserve_exact(appid_count)
        .map_err(|_| out_of_memory())?;
    for target in named_symbols
        .entries()
        .iter()
        .copied()
        .filter(|entry| entry.kind() == DxfNamedSymbolTableKind::AppId)
    {
        ensure_not_cancelled(cancellation)?;
        index.push(AppIdIndexEntry {
            digest: sha256_span(document, target.name().value_span(), cancellation)?,
            target,
        });
    }
    index.sort_unstable_by_key(|entry| (entry.digest, entry.target.record().ordinal()));
    Ok(index.into_boxed_slice())
}

fn exact_matches(
    document: DxfRawDocumentView<'_>,
    index: &[AppIdIndexEntry],
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

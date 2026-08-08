//! Per-application destination readiness for exact generic XDATA symbol names.
//!
//! This composes APPID registration and group-1003 LAYER evidence only. It does not validate
//! structure, capacity, transformed coordinates, mapped handles, payload semantics, or encoding.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataAppIdDestinationDirectory, DxfEntityXDataAppIdDestinationEntry,
    DxfEntityXDataAppIdDestinationState, DxfEntityXDataApplication,
    DxfEntityXDataLayerDestinationDirectory, DxfEntityXDataLayerDestinationEntry,
    DxfEntityXDataLayerDestinationState, DxfError, DxfIoOperation, DxfNamedSymbolTableEntry,
    DxfRawDocumentView, DxfSourceId,
};

/// One exact symbol blocker for an XDATA application destination.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataSymbolDestinationIssueKind {
    AppId(DxfEntityXDataAppIdDestinationState),
    Layer {
        source_resolution_ordinal: u32,
        state: DxfEntityXDataLayerDestinationState,
    },
}

/// One source-application-bound destination symbol issue.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataSymbolDestinationIssue {
    application_ordinal: u32,
    kind: DxfEntityXDataSymbolDestinationIssueKind,
}

impl DxfEntityXDataSymbolDestinationIssue {
    #[must_use]
    pub const fn application_ordinal(self) -> u64 {
        self.application_ordinal as u64
    }

    #[must_use]
    pub const fn kind(self) -> DxfEntityXDataSymbolDestinationIssueKind {
        self.kind
    }
}

/// Half-open range in the directory's destination symbol issue array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataSymbolDestinationIssueRange {
    start: u32,
    end: u32,
}

impl DxfEntityXDataSymbolDestinationIssueRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Whether all exact APPID and application-bound LAYER destination symbols are unique.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataSymbolDestinationState {
    Ready {
        destination_layer_occurrence_count: u32,
    },
    Unavailable {
        issue_count: u32,
    },
}

/// One source application and its exact destination symbol readiness.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataSymbolDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    application_ordinal: u32,
    state: DxfEntityXDataSymbolDestinationState,
    issues: DxfEntityXDataSymbolDestinationIssueRange,
}

impl DxfEntityXDataSymbolDestinationEntry {
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
    pub const fn state(self) -> DxfEntityXDataSymbolDestinationState {
        self.state
    }

    #[must_use]
    pub const fn issue_range(self) -> DxfEntityXDataSymbolDestinationIssueRange {
        self.issues
    }
}

/// Exact APPID and application-bound LAYER destination readiness in source order.
#[derive(Debug)]
pub struct DxfEntityXDataSymbolDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    appids: DxfEntityXDataAppIdDestinationDirectory,
    layers: DxfEntityXDataLayerDestinationDirectory,
    entries: Box<[DxfEntityXDataSymbolDestinationEntry]>,
    issues: Box<[DxfEntityXDataSymbolDestinationIssue]>,
}

impl DxfEntityXDataSymbolDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let appids = source.entity_xdata_appid_destination_directory(destination, cancellation)?;
        let layers = source.entity_xdata_layer_destination_directory(destination, cancellation)?;
        for observed in [appids.source_id(), layers.source_id()] {
            ensure_source(source.source_id(), observed)?;
        }
        for observed in [appids.destination_id(), layers.destination_id()] {
            ensure_source(destination.source_id(), observed)?;
        }

        let applications = appids
            .source_resolution_directory()
            .xdata_directory()
            .applications();
        let mut entries = Vec::new();
        let mut issues = Vec::new();
        entries
            .try_reserve_exact(applications.len())
            .map_err(|_| out_of_memory())?;
        issues
            .try_reserve_exact(applications.len().saturating_add(layers.entries().len()))
            .map_err(|_| out_of_memory())?;
        for application in applications.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let issue_start = compact_len(issues.len())?;
            let appid = appids.entry_for_application(application)?;
            let destination_appid = collect_appid_issue(application, appid, &mut issues)?;
            let application_layers = layers.entries_for_application(application)?;
            let destination_layer_occurrence_count =
                collect_layer_issues(application, application_layers, &mut issues)?;
            let issue_end = compact_len(issues.len())?;
            let issue_count = issue_end
                .checked_sub(issue_start)
                .ok_or_else(invalid_internal_data)?;
            let state = if issue_count == 0 {
                let target = destination_appid.ok_or_else(invalid_internal_data)?;
                if appids.destination_target_for_entry(appid) != Some(target) {
                    return Err(invalid_internal_data());
                }
                DxfEntityXDataSymbolDestinationState::Ready {
                    destination_layer_occurrence_count,
                }
            } else {
                DxfEntityXDataSymbolDestinationState::Unavailable { issue_count }
            };
            entries.push(DxfEntityXDataSymbolDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                application_ordinal: compact_u64(application.ordinal())?,
                state,
                issues: DxfEntityXDataSymbolDestinationIssueRange::new(issue_start, issue_end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            appids,
            layers,
            entries: entries.into_boxed_slice(),
            issues: issues.into_boxed_slice(),
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
    pub const fn appid_destination_directory(&self) -> &DxfEntityXDataAppIdDestinationDirectory {
        &self.appids
    }

    #[must_use]
    pub const fn layer_destination_directory(&self) -> &DxfEntityXDataLayerDestinationDirectory {
        &self.layers
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataSymbolDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn issues(&self) -> &[DxfEntityXDataSymbolDestinationIssue] {
        &self.issues
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataSymbolDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entry_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<DxfEntityXDataSymbolDestinationEntry, DxfError> {
        let appid = self.appids.entry_for_application(application)?;
        self.entry(application.ordinal())
            .filter(|entry| {
                self.application_for_entry(*entry) == Some(application)
                    && self.appids.source_resolution_for_entry(appid).is_some()
            })
            .ok_or_else(invalid_internal_data)
    }

    #[must_use]
    pub fn application_for_entry(
        &self,
        entry: DxfEntityXDataSymbolDestinationEntry,
    ) -> Option<DxfEntityXDataApplication> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.appids
            .source_resolution_directory()
            .xdata_directory()
            .application(entry.application_ordinal())
    }

    pub fn issues_for_entry(
        &self,
        entry: DxfEntityXDataSymbolDestinationEntry,
    ) -> Result<&[DxfEntityXDataSymbolDestinationIssue], DxfError> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        let range = entry.issue_range();
        self.issues
            .get(
                usize::try_from(range.start()).map_err(|_| invalid_internal_data())?
                    ..usize::try_from(range.end()).map_err(|_| invalid_internal_data())?,
            )
            .ok_or_else(invalid_internal_data)
    }

    pub fn layer_entries_for_entry(
        &self,
        entry: DxfEntityXDataSymbolDestinationEntry,
    ) -> Result<&[DxfEntityXDataLayerDestinationEntry], DxfError> {
        let application = self
            .application_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.layers.entries_for_application(application)
    }

    #[must_use]
    pub fn destination_appid_target_for_entry(
        &self,
        entry: DxfEntityXDataSymbolDestinationEntry,
    ) -> Option<DxfNamedSymbolTableEntry> {
        let DxfEntityXDataSymbolDestinationState::Ready { .. } = entry.state() else {
            return None;
        };
        let application = self.application_for_entry(entry)?;
        let appid = self.appids.entry_for_application(application).ok()?;
        self.appids.destination_target_for_entry(appid)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataSymbolDestinationEntry], DxfError> {
        let appids = self
            .appids
            .source_resolution_directory()
            .entries_for_entity(entity)?;
        let Some(first) = appids.first() else {
            return self.entries.get(0..0).ok_or_else(invalid_internal_data);
        };
        let start =
            usize::try_from(first.application().ordinal()).map_err(|_| invalid_internal_data())?;
        let end = start
            .checked_add(appids.len())
            .ok_or_else(invalid_internal_data)?;
        self.entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_symbol_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataSymbolDestinationDirectory, DxfError> {
        DxfEntityXDataSymbolDestinationDirectory::from_documents(self, destination, cancellation)
    }
}

macro_rules! document_symbol_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_symbol_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataSymbolDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_symbol_destination_directory(destination, cancellation)
            }
        }
    };
}

document_symbol_destination_directory!(DxfAsciiRawDocument<'_>);
document_symbol_destination_directory!(DxfBinaryRawDocument<'_>);

fn collect_appid_issue(
    application: DxfEntityXDataApplication,
    appid: DxfEntityXDataAppIdDestinationEntry,
    issues: &mut Vec<DxfEntityXDataSymbolDestinationIssue>,
) -> Result<Option<DxfNamedSymbolTableEntry>, DxfError> {
    match appid.state() {
        DxfEntityXDataAppIdDestinationState::DestinationUnique { target } => Ok(Some(target)),
        state => {
            push_issue(
                issues,
                application,
                DxfEntityXDataSymbolDestinationIssueKind::AppId(state),
            )?;
            Ok(None)
        }
    }
}

fn collect_layer_issues(
    application: DxfEntityXDataApplication,
    layers: &[DxfEntityXDataLayerDestinationEntry],
    issues: &mut Vec<DxfEntityXDataSymbolDestinationIssue>,
) -> Result<u32, DxfError> {
    let mut target_count = 0_u32;
    for layer in layers.iter().copied() {
        match layer.state() {
            DxfEntityXDataLayerDestinationState::DestinationUnique { .. } => {
                target_count = target_count
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
            }
            state => push_issue(
                issues,
                application,
                DxfEntityXDataSymbolDestinationIssueKind::Layer {
                    source_resolution_ordinal: compact_u64(layer.source_resolution_ordinal())?,
                    state,
                },
            )?,
        }
    }
    Ok(target_count)
}

fn push_issue(
    issues: &mut Vec<DxfEntityXDataSymbolDestinationIssue>,
    application: DxfEntityXDataApplication,
    kind: DxfEntityXDataSymbolDestinationIssueKind,
) -> Result<(), DxfError> {
    issues.try_reserve(1).map_err(|_| out_of_memory())?;
    issues.push(DxfEntityXDataSymbolDestinationIssue {
        application_ordinal: compact_u64(application.ordinal())?,
        kind,
    });
    Ok(())
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

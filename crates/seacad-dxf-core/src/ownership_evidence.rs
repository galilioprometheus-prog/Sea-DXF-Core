//! Incoming ownership-class evidence over document-local handle resolution.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandleGroupClass,
    DxfHandleIdentityMatch, DxfHandleResolutionDirectory, DxfHandleResolutionEntry,
    DxfHandleResolutionState, DxfIoOperation, DxfRawDocumentView, DxfRawRecord, DxfSourceId,
};

/// Half-open range of resolved-link ordinals in an ownership-evidence directory.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfOwnershipLinkRange {
    start: u32,
    end: u32,
}

impl DxfOwnershipLinkRange {
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

/// One source-order soft-owner or hard-owner handle occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfOwnershipEvidenceEntry {
    resolution_ordinal: u32,
    resolution: DxfHandleResolutionEntry,
}

impl DxfOwnershipEvidenceEntry {
    #[must_use]
    pub const fn resolution_ordinal(self) -> u64 {
        self.resolution_ordinal as u64
    }

    #[must_use]
    pub const fn resolution(self) -> DxfHandleResolutionEntry {
        self.resolution
    }

    #[must_use]
    pub const fn class(self) -> DxfHandleGroupClass {
        self.resolution.reference().class()
    }

    #[must_use]
    pub const fn state(self) -> DxfHandleResolutionState {
        self.resolution.state()
    }
}

/// One uniquely resolved ownership-class occurrence, indexed by target record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfResolvedOwnershipLink {
    evidence_ordinal: u32,
    target: DxfHandleIdentityMatch,
}

impl DxfResolvedOwnershipLink {
    #[must_use]
    pub const fn evidence_ordinal(self) -> u64 {
        self.evidence_ordinal as u64
    }

    #[must_use]
    pub const fn target(self) -> DxfHandleIdentityMatch {
        self.target
    }
}

/// Cardinality of uniquely resolved incoming ownership-class evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfIncomingOwnershipState {
    NoIncomingLink,
    UniqueIncomingLink,
    MultipleIncomingLinks { link_count: u32 },
}

/// One raw target record and its incoming ownership-class link range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfOwnershipTargetEntry {
    record: DxfRawRecord,
    incoming_range: DxfOwnershipLinkRange,
    state: DxfIncomingOwnershipState,
}

impl DxfOwnershipTargetEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn incoming_range(self) -> DxfOwnershipLinkRange {
        self.incoming_range
    }

    #[must_use]
    pub const fn state(self) -> DxfIncomingOwnershipState {
        self.state
    }
}

/// Immutable ownership-class evidence with incoming links grouped by target.
///
/// Only soft-owner and hard-owner occurrences participate. Invalid, null,
/// missing, and ambiguous occurrences remain visible in `entries`; only unique
/// resolutions enter the incoming-target index. This is evidence aggregation,
/// not a complete or validated document ownership graph.
#[derive(Debug)]
pub struct DxfOwnershipEvidenceDirectory {
    source_id: DxfSourceId,
    resolutions: DxfHandleResolutionDirectory,
    entries: Box<[DxfOwnershipEvidenceEntry]>,
    resolved_links: Box<[DxfResolvedOwnershipLink]>,
    targets: Box<[DxfOwnershipTargetEntry]>,
}

impl DxfOwnershipEvidenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let resolutions = document.handle_resolution_directory(cancellation)?;
        if resolutions.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: resolutions.source_id(),
            });
        }

        let mut entries = Vec::new();
        let mut resolved_links = Vec::new();
        for (resolution_index, resolution) in resolutions.entries().iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            if !is_ownership_class(resolution.reference().class()) {
                continue;
            }
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            let evidence_ordinal = compact_len(entries.len())?;
            let resolution_ordinal = compact_len(resolution_index)?;
            entries.push(DxfOwnershipEvidenceEntry {
                resolution_ordinal,
                resolution,
            });

            if resolution.state() == DxfHandleResolutionState::Unique {
                let targets = resolutions
                    .targets_for_reference(resolution_ordinal as u64)
                    .ok_or_else(invalid_internal_data)?;
                let [target] = targets else {
                    return Err(invalid_internal_data());
                };
                resolved_links.try_reserve(1).map_err(|_| out_of_memory())?;
                resolved_links.push(DxfResolvedOwnershipLink {
                    evidence_ordinal,
                    target: *target,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        resolved_links.sort_unstable_by_key(|link| {
            (link.target().record().ordinal(), link.evidence_ordinal())
        });
        ensure_not_cancelled(cancellation)?;

        let identity_entries = resolutions.identity_directory().entries();
        let mut targets = Vec::new();
        targets
            .try_reserve(identity_entries.len())
            .map_err(|_| out_of_memory())?;
        let mut link_cursor = 0_usize;
        for identity in identity_entries.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(link_cursor)?;
            while resolved_links
                .get(link_cursor)
                .is_some_and(|link| link.target().record().ordinal() == identity.record().ordinal())
            {
                link_cursor = link_cursor
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
            }
            let end = compact_len(link_cursor)?;
            let incoming_range = DxfOwnershipLinkRange::new(start, end)?;
            let state = match incoming_range.len() {
                0 => DxfIncomingOwnershipState::NoIncomingLink,
                1 => DxfIncomingOwnershipState::UniqueIncomingLink,
                link_count => DxfIncomingOwnershipState::MultipleIncomingLinks {
                    link_count: u32::try_from(link_count).map_err(|_| invalid_internal_data())?,
                },
            };
            targets.push(DxfOwnershipTargetEntry {
                record: identity.record(),
                incoming_range,
                state,
            });
        }
        if link_cursor != resolved_links.len() {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;

        Ok(Self {
            source_id: document.source_id(),
            resolutions,
            entries: entries.into_boxed_slice(),
            resolved_links: resolved_links.into_boxed_slice(),
            targets: targets.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn resolution_directory(&self) -> &DxfHandleResolutionDirectory {
        &self.resolutions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfOwnershipEvidenceEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, evidence_ordinal: u64) -> Option<DxfOwnershipEvidenceEntry> {
        let index = usize::try_from(evidence_ordinal).ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn evidence_for_source_record(
        &self,
        record_ordinal: u64,
    ) -> Option<&[DxfOwnershipEvidenceEntry]> {
        self.resolutions
            .identity_directory()
            .entry(record_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry.resolution().reference().record().ordinal() < record_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry.resolution().reference().record().ordinal() <= record_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn resolved_links(&self) -> &[DxfResolvedOwnershipLink] {
        &self.resolved_links
    }

    #[must_use]
    pub fn target_entries(&self) -> &[DxfOwnershipTargetEntry] {
        &self.targets
    }

    #[must_use]
    pub fn target_entry(&self, record_ordinal: u64) -> Option<DxfOwnershipTargetEntry> {
        let index = usize::try_from(record_ordinal).ok()?;
        self.targets
            .get(index)
            .copied()
            .filter(|entry| entry.record().ordinal() == record_ordinal)
    }

    #[must_use]
    pub fn incoming_links_for_target_record(
        &self,
        record_ordinal: u64,
    ) -> Option<&[DxfResolvedOwnershipLink]> {
        let entry = self.target_entry(record_ordinal)?;
        let start = usize::try_from(entry.incoming_range().start()).ok()?;
        let end = usize::try_from(entry.incoming_range().end()).ok()?;
        self.resolved_links.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    /// Aggregates incoming soft-owner and hard-owner evidence by unique target.
    pub fn ownership_evidence_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfOwnershipEvidenceDirectory, DxfError> {
        DxfOwnershipEvidenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn ownership_evidence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfOwnershipEvidenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).ownership_evidence_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn ownership_evidence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfOwnershipEvidenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).ownership_evidence_directory(cancellation)
    }
}

const fn is_ownership_class(class: DxfHandleGroupClass) -> bool {
    matches!(
        class,
        DxfHandleGroupClass::SoftOwner | DxfHandleGroupClass::HardOwner
    )
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
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

//! Source-order extended-data evidence attached to indexed DXF entities.

use std::io;

use crate::{
    DxfAsciiGroupRange, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityDirectory, DxfEntityRef, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup,
    DxfSourceId,
};

/// Whether one application XDATA list remains contiguous through its boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataApplicationState {
    Contiguous,
    Interrupted,
}

/// Context assigned to one group in the inclusive XDATA code range 1000..=1071.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataOccurrenceKind {
    ApplicationName { application_ordinal: u32 },
    ApplicationValue { application_ordinal: u32 },
    Orphan,
}

/// One source-anchored XDATA-code occurrence on an indexed entity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataOccurrence {
    entity: DxfEntityRef,
    group: DxfRawGroup,
    kind: DxfEntityXDataOccurrenceKind,
}

impl DxfEntityXDataOccurrence {
    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn kind(self) -> DxfEntityXDataOccurrenceKind {
        self.kind
    }
}

/// Half-open range in the directory's source-order occurrence array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataOccurrenceRange {
    start: u32,
    end: u32,
}

impl DxfEntityXDataOccurrenceRange {
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

/// One group-1001 application name and its following record-local XDATA values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataApplication {
    ordinal: u32,
    entity: DxfEntityRef,
    application_name: DxfRawGroup,
    source_group_range: DxfAsciiGroupRange,
    occurrences: DxfEntityXDataOccurrenceRange,
    state: DxfEntityXDataApplicationState,
}

impl DxfEntityXDataApplication {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn application_name(self) -> DxfRawGroup {
        self.application_name
    }

    #[must_use]
    pub const fn source_group_range(self) -> DxfAsciiGroupRange {
        self.source_group_range
    }

    #[must_use]
    pub const fn occurrence_range(self) -> DxfEntityXDataOccurrenceRange {
        self.occurrences
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataApplicationState {
        self.state
    }
}

/// Immutable source-order XDATA evidence for every indexed entity.
///
/// Group 1001 starts a new application list. Every following 1000..=1071
/// occurrence belongs to that list until another 1001, a non-XDATA group, or
/// the raw-record boundary. XDATA codes without a current 1001 remain visible
/// as orphan occurrences.
#[derive(Debug)]
pub struct DxfEntityXDataDirectory {
    source_id: DxfSourceId,
    entities: DxfEntityDirectory,
    applications: Box<[DxfEntityXDataApplication]>,
    occurrences: Box<[DxfEntityXDataOccurrence]>,
}

impl DxfEntityXDataDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let entities = document.entity_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        ensure_source(document.source_id(), entities.source_id())?;
        ensure_source(document.source_id(), application_groups.source_id())?;
        let mut applications = Vec::new();
        let mut occurrences = Vec::new();
        for entity in entities.entities().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let mut open = None;
            for group_occurrence in
                entity.record().group_range().start()..entity.record().group_range().end()
            {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(group_occurrence)
                    .ok_or_else(invalid_internal_data)?;
                let group_code = group.group_code().value();
                let inside_application_group = application_groups
                    .group_for_content_occurrence(group_occurrence)
                    .is_some();
                if inside_application_group {
                    continue;
                }
                if group_code == 1001 {
                    close_application(
                        &mut applications,
                        open.take(),
                        group_occurrence,
                        compact_len(occurrences.len())?,
                        DxfEntityXDataApplicationState::Contiguous,
                    )?;
                    let ordinal = compact_len(applications.len())?;
                    let occurrence_start = compact_len(occurrences.len())?;
                    occurrences.try_reserve(1).map_err(|_| out_of_memory())?;
                    occurrences.push(DxfEntityXDataOccurrence {
                        entity,
                        group,
                        kind: DxfEntityXDataOccurrenceKind::ApplicationName {
                            application_ordinal: ordinal,
                        },
                    });
                    applications.try_reserve(1).map_err(|_| out_of_memory())?;
                    applications.push(DxfEntityXDataApplication {
                        ordinal,
                        entity,
                        application_name: group,
                        source_group_range: DxfAsciiGroupRange::new(
                            compact_occurrence(group_occurrence)?,
                            compact_occurrence(entity.record().group_range().end())?,
                        )?,
                        occurrences: DxfEntityXDataOccurrenceRange::new(
                            occurrence_start,
                            compact_len(occurrences.len())?,
                        )?,
                        state: DxfEntityXDataApplicationState::Contiguous,
                    });
                    open = Some(applications.len() - 1);
                } else if (1000..=1071).contains(&group_code) {
                    let kind = match open {
                        Some(index) => DxfEntityXDataOccurrenceKind::ApplicationValue {
                            application_ordinal: applications
                                .get(index)
                                .ok_or_else(invalid_internal_data)?
                                .ordinal,
                        },
                        None => DxfEntityXDataOccurrenceKind::Orphan,
                    };
                    occurrences.try_reserve(1).map_err(|_| out_of_memory())?;
                    occurrences.push(DxfEntityXDataOccurrence {
                        entity,
                        group,
                        kind,
                    });
                } else {
                    close_application(
                        &mut applications,
                        open.take(),
                        group_occurrence,
                        compact_len(occurrences.len())?,
                        DxfEntityXDataApplicationState::Interrupted,
                    )?;
                }
            }
            close_application(
                &mut applications,
                open,
                entity.record().group_range().end(),
                compact_len(occurrences.len())?,
                DxfEntityXDataApplicationState::Contiguous,
            )?;
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entities,
            applications: applications.into_boxed_slice(),
            occurrences: occurrences.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn entity_directory(&self) -> &DxfEntityDirectory {
        &self.entities
    }

    #[must_use]
    pub fn applications(&self) -> &[DxfEntityXDataApplication] {
        &self.applications
    }

    #[must_use]
    pub fn occurrences(&self) -> &[DxfEntityXDataOccurrence] {
        &self.occurrences
    }

    #[must_use]
    pub fn application(&self, ordinal: u64) -> Option<DxfEntityXDataApplication> {
        self.applications
            .get(usize::try_from(ordinal).ok()?)
            .copied()
    }

    #[must_use]
    pub fn occurrence(&self, ordinal: u64) -> Option<DxfEntityXDataOccurrence> {
        self.occurrences
            .get(usize::try_from(ordinal).ok()?)
            .copied()
    }

    pub fn applications_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataApplication], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let ordinal = entity.record().ordinal();
        let start = self
            .applications
            .partition_point(|entry| entry.entity().record().ordinal() < ordinal);
        let end = self
            .applications
            .partition_point(|entry| entry.entity().record().ordinal() <= ordinal);
        self.applications
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }

    pub fn occurrences_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataOccurrence], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let ordinal = entity.record().ordinal();
        let start = self
            .occurrences
            .partition_point(|entry| entry.entity().record().ordinal() < ordinal);
        let end = self
            .occurrences
            .partition_point(|entry| entry.entity().record().ordinal() <= ordinal);
        self.occurrences
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }

    pub fn occurrences_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<&[DxfEntityXDataOccurrence], DxfError> {
        ensure_source(self.source_id, application.entity().source_id())?;
        if self.application(application.ordinal()) != Some(application) {
            return Err(invalid_internal_data());
        }
        let range = application.occurrence_range();
        self.occurrences
            .get(
                usize::try_from(range.start()).map_err(|_| invalid_internal_data())?
                    ..usize::try_from(range.end()).map_err(|_| invalid_internal_data())?,
            )
            .ok_or_else(invalid_internal_data)
    }

    #[must_use]
    pub fn occurrence_for_group(&self, group_occurrence: u64) -> Option<DxfEntityXDataOccurrence> {
        self.occurrences
            .binary_search_by_key(&group_occurrence, |entry| entry.group().occurrence())
            .ok()
            .and_then(|index| self.occurrences.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    /// Indexes source-order XDATA application lists attached to DXF entities.
    pub fn entity_xdata_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataDirectory, DxfError> {
        DxfEntityXDataDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_xdata_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_xdata_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_xdata_directory(cancellation)
    }
}

fn close_application(
    applications: &mut [DxfEntityXDataApplication],
    open: Option<usize>,
    boundary: u64,
    occurrence_end: u32,
    state: DxfEntityXDataApplicationState,
) -> Result<(), DxfError> {
    let Some(index) = open else {
        return Ok(());
    };
    let application = applications
        .get_mut(index)
        .ok_or_else(invalid_internal_data)?;
    application.source_group_range = DxfAsciiGroupRange::new(
        compact_occurrence(application.source_group_range.start())?,
        compact_occurrence(boundary)?,
    )?;
    application.occurrences =
        DxfEntityXDataOccurrenceRange::new(application.occurrences.start, occurrence_end)?;
    application.state = state;
    Ok(())
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

fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

fn compact_occurrence(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
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

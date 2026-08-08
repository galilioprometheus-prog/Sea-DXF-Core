//! Canonical destination-byte groups composed into exact source XDATA applications.

use std::{fmt, io};

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityRef,
    DxfEntityXDataApplication, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataEncodedDestinationDirectory, DxfEntityXDataEncodedDestinationEntry,
    DxfEntityXDataEncodedDestinationState, DxfEntityXDataHandleRemap, DxfEntityXDataOccurrenceKind,
    DxfEntityXDataPayloadDestinationEntry, DxfEntityXDataPayloadDestinationState, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfResourceProfile, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataEncodedApplicationDestinationState {
    Ready {
        member_count: u32,
        encoded_byte_count: u64,
    },
    Unavailable {
        payload_state: DxfEntityXDataPayloadDestinationState,
        member_count: u32,
        unavailable_member_count: u32,
        first_unavailable_member_ordinal: Option<u32>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataEncodedApplicationDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    application_ordinal: u32,
    payload_ordinal: u32,
    encoded_start: u32,
    encoded_end: u32,
    state: DxfEntityXDataEncodedApplicationDestinationState,
}

impl DxfEntityXDataEncodedApplicationDestinationEntry {
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
    pub const fn payload_ordinal(self) -> u64 {
        self.payload_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataEncodedApplicationDestinationState {
        self.state
    }
}

pub struct DxfEntityXDataEncodedApplicationDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    encoded: DxfEntityXDataEncodedDestinationDirectory,
    entries: Box<[DxfEntityXDataEncodedApplicationDestinationEntry]>,
}

impl DxfEntityXDataEncodedApplicationDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let encoded = source.entity_xdata_encoded_destination_directory(
            destination,
            transform,
            mappings,
            profile,
            cancellation,
        )?;
        ensure_source(source.source_id(), encoded.source_id())?;
        ensure_source(destination.source_id(), encoded.destination_id())?;
        let typed = encoded
            .logical_destination_directory()
            .payload_destination_directory()
            .typed_directory();
        let applications = typed.xdata_directory().applications();
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(applications.len())
            .map_err(|_| out_of_memory())?;
        for application in applications.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let typed_entries = typed.entries_for_application(application)?;
            let encoded_start = usize::try_from(application.occurrence_range().start())
                .map_err(|_| invalid_internal_data())?;
            let encoded_end = usize::try_from(application.occurrence_range().end())
                .map_err(|_| invalid_internal_data())?;
            let encoded_entries = encoded
                .entries()
                .get(encoded_start..encoded_end)
                .ok_or_else(invalid_internal_data)?;
            validate_members(&encoded, application, typed_entries, encoded_entries)?;
            let payload = encoded
                .logical_destination_directory()
                .payload_destination_directory()
                .entry_for_entity(application.entity())?
                .ok_or_else(invalid_internal_data)?;
            entries.push(DxfEntityXDataEncodedApplicationDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                application_ordinal: compact_u64(application.ordinal())?,
                payload_ordinal: compact_u64(payload.ordinal())?,
                encoded_start: compact_len(encoded_start)?,
                encoded_end: compact_len(encoded_end)?,
                state: readiness_state(payload, encoded_entries)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            encoded,
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
    pub const fn encoded_destination_directory(
        &self,
    ) -> &DxfEntityXDataEncodedDestinationDirectory {
        &self.encoded
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataEncodedApplicationDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataEncodedApplicationDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn application_for_entry(
        &self,
        entry: DxfEntityXDataEncodedApplicationDestinationEntry,
    ) -> Option<DxfEntityXDataApplication> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.encoded
            .logical_destination_directory()
            .payload_destination_directory()
            .typed_directory()
            .xdata_directory()
            .application(entry.application_ordinal())
    }

    #[must_use]
    pub fn payload_for_entry(
        &self,
        entry: DxfEntityXDataEncodedApplicationDestinationEntry,
    ) -> Option<DxfEntityXDataPayloadDestinationEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.encoded
            .logical_destination_directory()
            .payload_destination_directory()
            .entry(entry.payload_ordinal())
    }

    pub fn entry_for_application(
        &self,
        application: DxfEntityXDataApplication,
    ) -> Result<DxfEntityXDataEncodedApplicationDestinationEntry, DxfError> {
        ensure_source(self.source_id, application.entity().source_id())?;
        self.entry(application.ordinal())
            .filter(|entry| self.application_for_entry(*entry) == Some(application))
            .ok_or_else(invalid_internal_data)
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataEncodedApplicationDestinationEntry], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let applications = self
            .encoded
            .logical_destination_directory()
            .payload_destination_directory()
            .typed_directory()
            .xdata_directory()
            .applications_for_entity(entity)?;
        let Some(first) = applications.first() else {
            return self.entries.get(0..0).ok_or_else(invalid_internal_data);
        };
        let start = usize::try_from(first.ordinal()).map_err(|_| invalid_internal_data())?;
        let end = start
            .checked_add(applications.len())
            .ok_or_else(invalid_internal_data)?;
        let entries = self
            .entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)?;
        if entries
            .iter()
            .copied()
            .zip(applications.iter().copied())
            .all(|(entry, application)| self.application_for_entry(entry) == Some(application))
        {
            Ok(entries)
        } else {
            Err(invalid_internal_data())
        }
    }

    pub fn encoded_entries_for_entry(
        &self,
        entry: DxfEntityXDataEncodedApplicationDestinationEntry,
    ) -> Result<&[DxfEntityXDataEncodedDestinationEntry], DxfError> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return Err(invalid_internal_data());
        }
        self.encoded
            .entries()
            .get(
                usize::try_from(entry.encoded_start).map_err(|_| invalid_internal_data())?
                    ..usize::try_from(entry.encoded_end).map_err(|_| invalid_internal_data())?,
            )
            .ok_or_else(invalid_internal_data)
    }

    #[must_use]
    pub fn encoded_bytes_for_entry(
        &self,
        entry: DxfEntityXDataEncodedApplicationDestinationEntry,
    ) -> Option<&[u8]> {
        if !matches!(
            entry.state(),
            DxfEntityXDataEncodedApplicationDestinationState::Ready { .. }
        ) {
            return None;
        }
        let entries = self.encoded_entries_for_entry(entry).ok()?;
        self.encoded.encoded_bytes_for_ready_range(entries)
    }
}

impl fmt::Debug for DxfEntityXDataEncodedApplicationDestinationDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfEntityXDataEncodedApplicationDestinationDirectory")
            .field("source_id", &self.source_id)
            .field("destination_id", &self.destination_id)
            .field("encoded", &self.encoded)
            .field("entries", &self.entries)
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_encoded_application_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataEncodedApplicationDestinationDirectory, DxfError> {
        DxfEntityXDataEncodedApplicationDestinationDirectory::from_documents(
            self,
            destination,
            transform,
            mappings,
            profile,
            cancellation,
        )
    }
}

macro_rules! document_encoded_application_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_encoded_application_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                transform: DxfEntityXDataCoordinateTransform,
                mappings: &[DxfEntityXDataHandleRemap],
                profile: DxfResourceProfile,
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataEncodedApplicationDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self)
                    .entity_xdata_encoded_application_destination_directory(
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

document_encoded_application_destination_directory!(DxfAsciiRawDocument<'_>);
document_encoded_application_destination_directory!(DxfBinaryRawDocument<'_>);

fn validate_members(
    encoded: &DxfEntityXDataEncodedDestinationDirectory,
    application: DxfEntityXDataApplication,
    typed_entries: &[crate::DxfEntityXDataTypedEntry],
    encoded_entries: &[DxfEntityXDataEncodedDestinationEntry],
) -> Result<(), DxfError> {
    if typed_entries.len() != encoded_entries.len() || typed_entries.is_empty() {
        return Err(invalid_internal_data());
    }
    for (member_ordinal, (typed, encoded_entry)) in typed_entries
        .iter()
        .copied()
        .zip(encoded_entries.iter().copied())
        .enumerate()
    {
        let logical = encoded
            .logical_for_entry(encoded_entry)
            .ok_or_else(invalid_internal_data)?;
        if encoded
            .logical_destination_directory()
            .typed_for_entry(logical)
            != Some(typed)
        {
            return Err(invalid_internal_data());
        }
        let expected_kind = if member_ordinal == 0 {
            DxfEntityXDataOccurrenceKind::ApplicationName {
                application_ordinal: compact_u64(application.ordinal())?,
            }
        } else {
            DxfEntityXDataOccurrenceKind::ApplicationValue {
                application_ordinal: compact_u64(application.ordinal())?,
            }
        };
        if typed.occurrence().entity() != application.entity()
            || typed.occurrence().kind() != expected_kind
        {
            return Err(invalid_internal_data());
        }
    }
    Ok(())
}

fn readiness_state(
    payload: DxfEntityXDataPayloadDestinationEntry,
    encoded_entries: &[DxfEntityXDataEncodedDestinationEntry],
) -> Result<DxfEntityXDataEncodedApplicationDestinationState, DxfError> {
    let member_count = compact_len(encoded_entries.len())?;
    let mut encoded_byte_count = 0_u64;
    let mut unavailable_member_count = 0_u32;
    let mut first_unavailable_member_ordinal = None;
    for (member_ordinal, entry) in encoded_entries.iter().copied().enumerate() {
        match entry.state() {
            DxfEntityXDataEncodedDestinationState::Ready {
                encoded_byte_count: count,
            } => {
                encoded_byte_count = encoded_byte_count
                    .checked_add(u64::from(count))
                    .ok_or_else(invalid_internal_data)?;
            }
            _ => {
                unavailable_member_count = unavailable_member_count
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
                if first_unavailable_member_ordinal.is_none() {
                    first_unavailable_member_ordinal = Some(compact_len(member_ordinal)?);
                }
            }
        }
    }
    match (payload.state(), unavailable_member_count) {
        (DxfEntityXDataPayloadDestinationState::Ready { .. }, 0) => {
            Ok(DxfEntityXDataEncodedApplicationDestinationState::Ready {
                member_count,
                encoded_byte_count,
            })
        }
        (payload_state, _) => Ok(
            DxfEntityXDataEncodedApplicationDestinationState::Unavailable {
                payload_state,
                member_count,
                unavailable_member_count,
                first_unavailable_member_ordinal,
            },
        ),
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
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

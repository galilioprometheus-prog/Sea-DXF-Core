//! Logical destination projection for every enclosed entity XDATA occurrence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfEntityRef,
    DxfEntityXDataAppIdDestinationDirectory, DxfEntityXDataAppIdDestinationState,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataDoubleRole,
    DxfEntityXDataHandleDestinationState, DxfEntityXDataHandleRemap,
    DxfEntityXDataLayerDestinationDirectory, DxfEntityXDataLayerDestinationState,
    DxfEntityXDataOccurrenceKind, DxfEntityXDataPayloadDestinationDirectory,
    DxfEntityXDataPayloadDestinationEntry, DxfEntityXDataTransformedPointIssue,
    DxfEntityXDataTransformedPointState, DxfEntityXDataTypedEntry, DxfEntityXDataValue,
    DxfEntityXDataValueIssue, DxfError, DxfHandle, DxfIoOperation, DxfNamedSymbolTableEntry,
    DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataLogicalDestinationValue {
    SourceExact(DxfEntityXDataValue),
    Application {
        target: DxfNamedSymbolTableEntry,
    },
    Layer {
        target: DxfNamedSymbolTableEntry,
    },
    Handle {
        target: DxfHandle,
    },
    TransformedDouble {
        role: DxfEntityXDataDoubleRole,
        value: DxfDouble,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataLogicalDestinationIssue {
    Orphan,
    InvalidSource(DxfEntityXDataValueIssue),
    Application(DxfEntityXDataAppIdDestinationState),
    Layer(DxfEntityXDataLayerDestinationState),
    Handle(DxfEntityXDataHandleDestinationState),
    Coordinate(DxfEntityXDataTransformedPointIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityXDataLogicalDestinationState {
    Available(DxfEntityXDataLogicalDestinationValue),
    Unavailable(DxfEntityXDataLogicalDestinationIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityXDataLogicalDestinationEntry {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    ordinal: u32,
    typed_ordinal: u32,
    state: DxfEntityXDataLogicalDestinationState,
}

impl DxfEntityXDataLogicalDestinationEntry {
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
    pub const fn typed_ordinal(self) -> u64 {
        self.typed_ordinal as u64
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityXDataLogicalDestinationState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityXDataLogicalDestinationDirectory {
    source_id: DxfSourceId,
    destination_id: DxfSourceId,
    payloads: DxfEntityXDataPayloadDestinationDirectory,
    entries: Box<[DxfEntityXDataLogicalDestinationEntry]>,
}

impl DxfEntityXDataLogicalDestinationDirectory {
    fn from_documents(
        source: DxfRawDocumentView<'_>,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let payloads = source.entity_xdata_payload_destination_directory(
            destination,
            transform,
            mappings,
            cancellation,
        )?;
        ensure_source(source.source_id(), payloads.source_id())?;
        ensure_source(destination.source_id(), payloads.destination_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(payloads.typed_directory().entries().len())
            .map_err(|_| out_of_memory())?;
        for typed in payloads.typed_directory().entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfEntityXDataLogicalDestinationEntry {
                source_id: source.source_id(),
                destination_id: destination.source_id(),
                ordinal: compact_len(entries.len())?,
                typed_ordinal: compact_u64(typed.ordinal())?,
                state: project_state(&payloads, typed)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: source.source_id(),
            destination_id: destination.source_id(),
            payloads,
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
    pub const fn payload_destination_directory(
        &self,
    ) -> &DxfEntityXDataPayloadDestinationDirectory {
        &self.payloads
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityXDataLogicalDestinationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityXDataLogicalDestinationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn typed_for_entry(
        &self,
        entry: DxfEntityXDataLogicalDestinationEntry,
    ) -> Option<DxfEntityXDataTypedEntry> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        self.payloads.typed_directory().entry(entry.typed_ordinal())
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityXDataLogicalDestinationEntry], DxfError> {
        let typed = self.payloads.typed_directory().entries_for_entity(entity)?;
        let Some(first) = typed.first() else {
            return self.entries.get(0..0).ok_or_else(invalid_internal_data);
        };
        let start = usize::try_from(first.ordinal()).map_err(|_| invalid_internal_data())?;
        let end = start
            .checked_add(typed.len())
            .ok_or_else(invalid_internal_data)?;
        let entries = self
            .entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)?;
        if entries
            .iter()
            .copied()
            .zip(typed.iter().copied())
            .all(|(entry, typed)| self.typed_for_entry(entry) == Some(typed))
        {
            Ok(entries)
        } else {
            Err(invalid_internal_data())
        }
    }

    pub fn payload_entry_for_entry(
        &self,
        entry: DxfEntityXDataLogicalDestinationEntry,
    ) -> Result<DxfEntityXDataPayloadDestinationEntry, DxfError> {
        let typed = self
            .typed_for_entry(entry)
            .ok_or_else(invalid_internal_data)?;
        self.payloads
            .entry_for_entity(typed.occurrence().entity())?
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_xdata_logical_destination_directory(
        self,
        destination: DxfRawDocumentView<'_>,
        transform: DxfEntityXDataCoordinateTransform,
        mappings: &[DxfEntityXDataHandleRemap],
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityXDataLogicalDestinationDirectory, DxfError> {
        DxfEntityXDataLogicalDestinationDirectory::from_documents(
            self,
            destination,
            transform,
            mappings,
            cancellation,
        )
    }
}

macro_rules! document_logical_destination_directory {
    ($document:ty) => {
        impl $document {
            pub fn entity_xdata_logical_destination_directory(
                &self,
                destination: DxfRawDocumentView<'_>,
                transform: DxfEntityXDataCoordinateTransform,
                mappings: &[DxfEntityXDataHandleRemap],
                cancellation: &DxfCancellationToken,
            ) -> Result<DxfEntityXDataLogicalDestinationDirectory, DxfError> {
                DxfRawDocumentView::from(self).entity_xdata_logical_destination_directory(
                    destination,
                    transform,
                    mappings,
                    cancellation,
                )
            }
        }
    };
}

document_logical_destination_directory!(DxfAsciiRawDocument<'_>);
document_logical_destination_directory!(DxfBinaryRawDocument<'_>);

fn project_state(
    payloads: &DxfEntityXDataPayloadDestinationDirectory,
    typed: DxfEntityXDataTypedEntry,
) -> Result<DxfEntityXDataLogicalDestinationState, DxfError> {
    if matches!(
        typed.occurrence().kind(),
        DxfEntityXDataOccurrenceKind::Orphan
    ) {
        return Ok(DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Orphan,
        ));
    }
    match typed.value() {
        DxfEntityXDataValue::Invalid { issue, .. } => {
            Ok(DxfEntityXDataLogicalDestinationState::Unavailable(
                DxfEntityXDataLogicalDestinationIssue::InvalidSource(issue),
            ))
        }
        DxfEntityXDataValue::ExactText {
            kind: crate::DxfEntityXDataTextKind::ApplicationName,
            ..
        } => project_application(payloads, typed),
        DxfEntityXDataValue::ExactText {
            kind: crate::DxfEntityXDataTextKind::LayerName,
            ..
        } => project_layer(payloads, typed),
        DxfEntityXDataValue::Handle { .. } => project_handle(payloads, typed),
        DxfEntityXDataValue::Double { role, .. } => project_double(payloads, typed, role),
        value => Ok(DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::SourceExact(value),
        )),
    }
}

fn project_application(
    payloads: &DxfEntityXDataPayloadDestinationDirectory,
    typed: DxfEntityXDataTypedEntry,
) -> Result<DxfEntityXDataLogicalDestinationState, DxfError> {
    let DxfEntityXDataOccurrenceKind::ApplicationName {
        application_ordinal,
    } = typed.occurrence().kind()
    else {
        return Err(invalid_internal_data());
    };
    let application = payloads
        .typed_directory()
        .xdata_directory()
        .application(u64::from(application_ordinal))
        .ok_or_else(invalid_internal_data)?;
    let appids = appid_destination_directory(payloads);
    let entry = appids.entry_for_application(application)?;
    match appids.destination_target_for_entry(entry) {
        Some(target) => Ok(DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Application { target },
        )),
        None => Ok(DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Application(entry.state()),
        )),
    }
}

fn project_layer(
    payloads: &DxfEntityXDataPayloadDestinationDirectory,
    typed: DxfEntityXDataTypedEntry,
) -> Result<DxfEntityXDataLogicalDestinationState, DxfError> {
    let layers = layer_destination_directory(payloads);
    let resolution = layers
        .source_resolution_directory()
        .entry_for_source(typed)
        .ok_or_else(invalid_internal_data)?;
    let entry = layers
        .entry_for_source_resolution(resolution)
        .ok_or_else(invalid_internal_data)?;
    match layers.destination_target_for_entry(entry) {
        Some(target) => Ok(DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Layer { target },
        )),
        None => Ok(DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Layer(entry.state()),
        )),
    }
}

fn project_handle(
    payloads: &DxfEntityXDataPayloadDestinationDirectory,
    typed: DxfEntityXDataTypedEntry,
) -> Result<DxfEntityXDataLogicalDestinationState, DxfError> {
    let handles = payloads
        .handle_composed_destination_directory()
        .handle_destination_directory();
    let entry = handles
        .entry_for_typed(typed)
        .ok_or_else(invalid_internal_data)?;
    match entry.state() {
        DxfEntityXDataHandleDestinationState::Unique { target }
            if handles.destination_target_for_entry(entry).is_some() =>
        {
            Ok(DxfEntityXDataLogicalDestinationState::Available(
                DxfEntityXDataLogicalDestinationValue::Handle { target },
            ))
        }
        state => Ok(DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Handle(state),
        )),
    }
}

fn project_double(
    payloads: &DxfEntityXDataPayloadDestinationDirectory,
    typed: DxfEntityXDataTypedEntry,
    role: DxfEntityXDataDoubleRole,
) -> Result<DxfEntityXDataLogicalDestinationState, DxfError> {
    let coordinates = payloads
        .handle_composed_destination_directory()
        .coordinate_destination_directory()
        .coordinate_transform_directory();
    let tuples = coordinates.point_tuple_directory();
    let Some(tuple) = tuples.tuple_for_entry(typed) else {
        return Ok(DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::SourceExact(typed.value()),
        ));
    };
    let transformed = coordinates
        .entry_for_tuple(tuple)
        .ok_or_else(invalid_internal_data)?;
    match transformed.state() {
        DxfEntityXDataTransformedPointState::Available {
            transformed: values,
            ..
        } => {
            let index = coordinate_index(tuple, typed)?;
            let value = *values.get(index).ok_or_else(invalid_internal_data)?;
            Ok(DxfEntityXDataLogicalDestinationState::Available(
                DxfEntityXDataLogicalDestinationValue::TransformedDouble { role, value },
            ))
        }
        DxfEntityXDataTransformedPointState::Unavailable(issue) => {
            Ok(DxfEntityXDataLogicalDestinationState::Unavailable(
                DxfEntityXDataLogicalDestinationIssue::Coordinate(issue),
            ))
        }
    }
}

fn coordinate_index(
    tuple: crate::DxfEntityXDataPointTuple,
    typed: DxfEntityXDataTypedEntry,
) -> Result<usize, DxfError> {
    let ordinal = typed.ordinal();
    if tuple
        .x()
        .is_some_and(|member| member.entry_ordinal() == ordinal)
    {
        Ok(0)
    } else if tuple
        .y()
        .is_some_and(|member| member.entry_ordinal() == ordinal)
    {
        Ok(1)
    } else if tuple
        .z()
        .is_some_and(|member| member.entry_ordinal() == ordinal)
    {
        Ok(2)
    } else {
        Err(invalid_internal_data())
    }
}

fn appid_destination_directory(
    payloads: &DxfEntityXDataPayloadDestinationDirectory,
) -> &DxfEntityXDataAppIdDestinationDirectory {
    payloads
        .handle_composed_destination_directory()
        .coordinate_destination_directory()
        .entity_destination_directory()
        .application_destination_directory()
        .symbol_destination_directory()
        .appid_destination_directory()
}

fn layer_destination_directory(
    payloads: &DxfEntityXDataPayloadDestinationDirectory,
) -> &DxfEntityXDataLayerDestinationDirectory {
    payloads
        .handle_composed_destination_directory()
        .coordinate_destination_directory()
        .entity_destination_directory()
        .application_destination_directory()
        .symbol_destination_directory()
        .layer_destination_directory()
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

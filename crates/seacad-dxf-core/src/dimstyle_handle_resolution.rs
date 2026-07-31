//! Document-local generic resolution for DIMSTYLE handle fields.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDimStyleField,
    DxfDimStyleFieldCard, DxfDimStyleFieldCardDirectory, DxfDimStyleFieldCardState,
    DxfDimStyleValueData, DxfDimStyleValueEntry, DxfError, DxfHandleIdentityMatch,
    DxfHandleParseIssue, DxfHandleResolutionDirectory, DxfHandleResolutionEntry,
    DxfHandleResolutionState, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
    dimstyle_field::field_for_group_code,
};

const HANDLE_GROUP_CODES: [i16; 5] = [340, 341, 342, 343, 344];

/// Documented meaning of one DIMSTYLE handle field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleHandleRole {
    TextStyle,
    LeaderArrowBlock,
    FirstArrowBlock,
    SecondArrowBlock,
    CommonArrowBlock,
}

impl DxfDimStyleHandleRole {
    const fn for_group_code(group_code: i16) -> Option<Self> {
        match group_code {
            340 => Some(Self::TextStyle),
            341 => Some(Self::LeaderArrowBlock),
            342 => Some(Self::CommonArrowBlock),
            343 => Some(Self::FirstArrowBlock),
            344 => Some(Self::SecondArrowBlock),
            _ => None,
        }
    }
}

/// Combined field-card and generic document-local target state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleHandleTargetState {
    Absent,
    MultipleValues { occurrence_count: u32 },
    Invalid(DxfHandleParseIssue),
    Null,
    Missing,
    Unique,
    Ambiguous { target_count: u32 },
}

/// One fixed DIMSTYLE handle role and its generic target disposition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleHandleResolutionEntry {
    ordinal: u32,
    record: DxfDimStyleValueEntry,
    field: DxfDimStyleField,
    role: DxfDimStyleHandleRole,
    resolution: Option<DxfHandleResolutionEntry>,
    state: DxfDimStyleHandleTargetState,
}

impl DxfDimStyleHandleResolutionEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfDimStyleValueEntry {
        self.record
    }

    #[must_use]
    pub const fn field(self) -> DxfDimStyleField {
        self.field
    }

    #[must_use]
    pub const fn role(self) -> DxfDimStyleHandleRole {
        self.role
    }

    #[must_use]
    pub const fn generic_resolution(self) -> Option<DxfHandleResolutionEntry> {
        self.resolution
    }

    #[must_use]
    pub const fn state(self) -> DxfDimStyleHandleTargetState {
        self.state
    }
}

/// DIMSTYLE handle cards composed with exact generic handle resolution.
#[derive(Debug)]
pub struct DxfDimStyleHandleResolutionDirectory {
    source_id: DxfSourceId,
    cards: DxfDimStyleFieldCardDirectory,
    handles: DxfHandleResolutionDirectory,
    entries: Box<[DxfDimStyleHandleResolutionEntry]>,
}

impl DxfDimStyleHandleResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.dimstyle_field_card_directory(cancellation)?;
        let handles = document.handle_resolution_directory(cancellation)?;
        for observed in [cards.source_id(), handles.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        for record in cards.evidence_directory().records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            for group_code in HANDLE_GROUP_CODES {
                let field = field_for_group_code(group_code).ok_or_else(invalid_internal_data)?;
                let role = DxfDimStyleHandleRole::for_group_code(group_code)
                    .ok_or_else(invalid_internal_data)?;
                let card = cards
                    .card_for_field(record.table_entry().record().ordinal(), field)
                    .ok_or_else(invalid_internal_data)?;
                let (resolution, state) = resolve_card(&cards, &handles, card)?;
                entries.try_reserve(1).map_err(|_| out_of_memory())?;
                entries.push(DxfDimStyleHandleResolutionEntry {
                    ordinal: compact_len(entries.len())?,
                    record,
                    field,
                    role,
                    resolution,
                    state,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            handles,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfDimStyleFieldCardDirectory {
        &self.cards
    }

    #[must_use]
    pub const fn handle_resolution_directory(&self) -> &DxfHandleResolutionDirectory {
        &self.handles
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfDimStyleHandleResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfDimStyleHandleResolutionEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_raw_ordinal(
        &self,
        raw_ordinal: u64,
    ) -> Option<&[DxfDimStyleHandleResolutionEntry]> {
        self.cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.record().table_entry().record().ordinal() < raw_ordinal);
        let end = self.entries.partition_point(|entry| {
            entry.record().table_entry().record().ordinal() <= raw_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn entry_for_role(
        &self,
        raw_ordinal: u64,
        role: DxfDimStyleHandleRole,
    ) -> Option<DxfDimStyleHandleResolutionEntry> {
        self.entries_for_raw_ordinal(raw_ordinal)?
            .iter()
            .copied()
            .find(|entry| entry.role() == role)
    }

    #[must_use]
    pub fn targets_for_entry(
        &self,
        entry: DxfDimStyleHandleResolutionEntry,
    ) -> Option<&[DxfHandleIdentityMatch]> {
        if self.entry(entry.ordinal()) != Some(entry) {
            return None;
        }
        match entry.generic_resolution() {
            Some(resolution) => match resolution.reference().value().parse_result() {
                Ok(handle) if !handle.is_null() => {
                    Some(self.handles.identity_directory().matches_for_handle(handle))
                }
                Ok(_) | Err(_) => Some(&[]),
            },
            None => Some(&[]),
        }
    }
}

impl DxfRawDocumentView<'_> {
    pub fn dimstyle_handle_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleHandleResolutionDirectory, DxfError> {
        DxfDimStyleHandleResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn dimstyle_handle_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleHandleResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_handle_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn dimstyle_handle_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleHandleResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_handle_resolution_directory(cancellation)
    }
}

fn resolve_card(
    cards: &DxfDimStyleFieldCardDirectory,
    handles: &DxfHandleResolutionDirectory,
    card: DxfDimStyleFieldCard,
) -> Result<
    (
        Option<DxfHandleResolutionEntry>,
        DxfDimStyleHandleTargetState,
    ),
    DxfError,
> {
    match card.state() {
        DxfDimStyleFieldCardState::Absent => Ok((None, DxfDimStyleHandleTargetState::Absent)),
        DxfDimStyleFieldCardState::Multiple { occurrence_count } => Ok((
            None,
            DxfDimStyleHandleTargetState::MultipleValues { occurrence_count },
        )),
        DxfDimStyleFieldCardState::Unique => {
            let [member] = cards
                .members_for_card(card.ordinal())
                .ok_or_else(invalid_internal_data)?
            else {
                return Err(invalid_internal_data());
            };
            let value = cards
                .value_for_member(*member)
                .ok_or_else(invalid_internal_data)?;
            match value.value() {
                Ok(DxfDimStyleValueData::Handle(_))
                | Err(crate::DxfDimStyleValueIssue::InvalidHandle(_)) => {}
                Ok(_) | Err(_) => return Err(invalid_internal_data()),
            }
            let resolution = handles
                .resolution_for_group(value.group().occurrence())
                .ok_or_else(invalid_internal_data)?;
            Ok((Some(resolution), target_state(resolution.state())))
        }
    }
}

const fn target_state(state: DxfHandleResolutionState) -> DxfDimStyleHandleTargetState {
    match state {
        DxfHandleResolutionState::Invalid(issue) => DxfDimStyleHandleTargetState::Invalid(issue),
        DxfHandleResolutionState::Null => DxfDimStyleHandleTargetState::Null,
        DxfHandleResolutionState::Missing => DxfDimStyleHandleTargetState::Missing,
        DxfHandleResolutionState::Unique => DxfDimStyleHandleTargetState::Unique,
        DxfHandleResolutionState::Ambiguous { target_count } => {
            DxfDimStyleHandleTargetState::Ambiguous { target_count }
        }
    }
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

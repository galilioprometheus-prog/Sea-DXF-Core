//! Exact table-kind validation for unique DIMSTYLE handle targets.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDimStyleHandleResolutionDirectory, DxfDimStyleHandleResolutionEntry, DxfDimStyleHandleRole,
    DxfDimStyleHandleTargetState, DxfError, DxfHandleIdentityMatch, DxfHandleParseIssue,
    DxfIoOperation, DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry,
    DxfNamedSymbolTableKind, DxfRawDocumentView, DxfSourceId,
};

/// Exact closed-table validation state for one DIMSTYLE handle role.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleHandleTargetValidationState {
    Absent,
    MultipleValues { occurrence_count: u32 },
    Invalid(DxfHandleParseIssue),
    Null,
    Missing,
    Ambiguous { target_count: u32 },
    UniqueExpected,
    UniqueOtherNamedSymbol { observed: DxfNamedSymbolTableKind },
    UniqueOtherRecord,
}

/// One generic DIMSTYLE handle resolution plus exact target-kind validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleHandleTargetValidationEntry {
    ordinal: u32,
    resolution: DxfDimStyleHandleResolutionEntry,
    target: Option<DxfHandleIdentityMatch>,
    named_target: Option<DxfNamedSymbolTableEntry>,
    state: DxfDimStyleHandleTargetValidationState,
}

impl DxfDimStyleHandleTargetValidationEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn resolution(self) -> DxfDimStyleHandleResolutionEntry {
        self.resolution
    }

    #[must_use]
    pub const fn target(self) -> Option<DxfHandleIdentityMatch> {
        self.target
    }

    #[must_use]
    pub const fn named_target(self) -> Option<DxfNamedSymbolTableEntry> {
        self.named_target
    }

    #[must_use]
    pub const fn state(self) -> DxfDimStyleHandleTargetValidationState {
        self.state
    }
}

/// DIMSTYLE handle targets validated against exact closed named table entries.
#[derive(Debug)]
pub struct DxfDimStyleHandleTargetValidationDirectory {
    source_id: DxfSourceId,
    resolutions: DxfDimStyleHandleResolutionDirectory,
    named: DxfNamedSymbolTableDirectory,
    entries: Box<[DxfDimStyleHandleTargetValidationEntry]>,
}

impl DxfDimStyleHandleTargetValidationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let resolutions = document.dimstyle_handle_resolution_directory(cancellation)?;
        let named = document.named_symbol_table_directory(cancellation)?;
        for observed in [resolutions.source_id(), named.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        entries
            .try_reserve(resolutions.entries().len())
            .map_err(|_| out_of_memory())?;
        for resolution in resolutions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let (target, named_target, state) = validate_target(&resolutions, &named, resolution)?;
            entries.push(DxfDimStyleHandleTargetValidationEntry {
                ordinal: compact_len(entries.len())?,
                resolution,
                target,
                named_target,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            resolutions,
            named,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn resolution_directory(&self) -> &DxfDimStyleHandleResolutionDirectory {
        &self.resolutions
    }

    #[must_use]
    pub const fn named_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.named
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfDimStyleHandleTargetValidationEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfDimStyleHandleTargetValidationEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_raw_ordinal(
        &self,
        raw_ordinal: u64,
    ) -> Option<&[DxfDimStyleHandleTargetValidationEntry]> {
        self.resolutions.entries_for_raw_ordinal(raw_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry.resolution().record().table_entry().record().ordinal() < raw_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry.resolution().record().table_entry().record().ordinal() <= raw_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn entry_for_role(
        &self,
        raw_ordinal: u64,
        role: DxfDimStyleHandleRole,
    ) -> Option<DxfDimStyleHandleTargetValidationEntry> {
        self.entries_for_raw_ordinal(raw_ordinal)?
            .iter()
            .copied()
            .find(|entry| entry.resolution().role() == role)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn dimstyle_handle_target_validation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleHandleTargetValidationDirectory, DxfError> {
        DxfDimStyleHandleTargetValidationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn dimstyle_handle_target_validation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleHandleTargetValidationDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_handle_target_validation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn dimstyle_handle_target_validation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleHandleTargetValidationDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_handle_target_validation_directory(cancellation)
    }
}

fn validate_target(
    resolutions: &DxfDimStyleHandleResolutionDirectory,
    named: &DxfNamedSymbolTableDirectory,
    resolution: DxfDimStyleHandleResolutionEntry,
) -> Result<
    (
        Option<DxfHandleIdentityMatch>,
        Option<DxfNamedSymbolTableEntry>,
        DxfDimStyleHandleTargetValidationState,
    ),
    DxfError,
> {
    let passthrough = match resolution.state() {
        DxfDimStyleHandleTargetState::Absent => {
            Some(DxfDimStyleHandleTargetValidationState::Absent)
        }
        DxfDimStyleHandleTargetState::MultipleValues { occurrence_count } => {
            Some(DxfDimStyleHandleTargetValidationState::MultipleValues { occurrence_count })
        }
        DxfDimStyleHandleTargetState::Invalid(issue) => {
            Some(DxfDimStyleHandleTargetValidationState::Invalid(issue))
        }
        DxfDimStyleHandleTargetState::Null => Some(DxfDimStyleHandleTargetValidationState::Null),
        DxfDimStyleHandleTargetState::Missing => {
            Some(DxfDimStyleHandleTargetValidationState::Missing)
        }
        DxfDimStyleHandleTargetState::Ambiguous { target_count } => {
            Some(DxfDimStyleHandleTargetValidationState::Ambiguous { target_count })
        }
        DxfDimStyleHandleTargetState::Unique => None,
    };
    if let Some(state) = passthrough {
        return Ok((None, None, state));
    }

    let [target] = resolutions
        .targets_for_entry(resolution)
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    let target = *target;
    let named_target = named.entry_for_raw_ordinal(target.record().ordinal());
    let expected = expected_kind(resolution.role());
    let state = match named_target {
        Some(entry) if entry.kind() == expected => {
            DxfDimStyleHandleTargetValidationState::UniqueExpected
        }
        Some(entry) => DxfDimStyleHandleTargetValidationState::UniqueOtherNamedSymbol {
            observed: entry.kind(),
        },
        None => DxfDimStyleHandleTargetValidationState::UniqueOtherRecord,
    };
    Ok((Some(target), named_target, state))
}

const fn expected_kind(role: DxfDimStyleHandleRole) -> DxfNamedSymbolTableKind {
    match role {
        DxfDimStyleHandleRole::TextStyle => DxfNamedSymbolTableKind::Style,
        DxfDimStyleHandleRole::LeaderArrowBlock
        | DxfDimStyleHandleRole::FirstArrowBlock
        | DxfDimStyleHandleRole::SecondArrowBlock
        | DxfDimStyleHandleRole::CommonArrowBlock => DxfNamedSymbolTableKind::BlockRecord,
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

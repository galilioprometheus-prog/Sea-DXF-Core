//! Source-anchored `$HANDSEED` discovery shared by ASCII and Binary DXF.

use std::io;

use crate::{
    ByteSpan, DxfAsciiGroup, DxfDiagnostic, DxfDiagnosticCode, DxfError, DxfGroupCode, DxfHandle,
    DxfHandleParseIssue, DxfIoOperation, DxfSourceId, parse_dxf_handle_hex,
};

/// Classification of the group immediately following one `$HANDSEED` marker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandseedValue {
    Parsed(DxfHandle),
    InvalidGroupCode(DxfGroupCode),
    InvalidHandle(DxfHandleParseIssue),
    MissingValue,
}

/// Fixed-size provenance for one exact `$HANDSEED` occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandseedOccurrence {
    variable_occurrence: u32,
    variable_span: ByteSpan,
    value_occurrence: Option<u32>,
    value_span: Option<ByteSpan>,
    value: DxfHandseedValue,
}

impl DxfHandseedOccurrence {
    #[must_use]
    pub const fn variable_occurrence(self) -> u64 {
        self.variable_occurrence as u64
    }

    #[must_use]
    pub const fn variable_span(self) -> ByteSpan {
        self.variable_span
    }

    #[must_use]
    pub const fn value_occurrence(self) -> Option<u64> {
        match self.value_occurrence {
            Some(value) => Some(value as u64),
            None => None,
        }
    }

    #[must_use]
    pub const fn value_span(self) -> Option<ByteSpan> {
        self.value_span
    }

    #[must_use]
    pub const fn value(self) -> DxfHandseedValue {
        self.value
    }
}

/// Structural and syntactic state of the exact HEADER handle seed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandseedState {
    Absent,
    Parsed,
    Invalid,
    Ambiguous,
}

/// Immutable `$HANDSEED` result tied to one raw document identity.
#[derive(Debug)]
pub struct DxfHandseedReport {
    source_id: DxfSourceId,
    state: DxfHandseedState,
    occurrence_count: u64,
    primary: Option<DxfHandseedOccurrence>,
    conflicting: Option<DxfHandseedOccurrence>,
    diagnostics: Box<[DxfDiagnostic]>,
}

impl DxfHandseedReport {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn state(&self) -> DxfHandseedState {
        self.state
    }

    #[must_use]
    pub const fn occurrence_count(&self) -> u64 {
        self.occurrence_count
    }

    #[must_use]
    pub const fn primary_occurrence(&self) -> Option<DxfHandseedOccurrence> {
        self.primary
    }

    #[must_use]
    pub const fn conflicting_occurrence(&self) -> Option<DxfHandseedOccurrence> {
        self.conflicting
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[DxfDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Copy)]
struct VariableMarker {
    occurrence: u32,
    span: ByteSpan,
}

#[derive(Default)]
pub(crate) struct DxfHandseedTracker {
    awaiting_section_name: bool,
    inside_header: bool,
    pending: Option<VariableMarker>,
    occurrence_count: u64,
    primary: Option<DxfHandseedOccurrence>,
    conflicting: Option<DxfHandseedOccurrence>,
}

impl DxfHandseedTracker {
    pub(crate) fn observe(&mut self, group: DxfAsciiGroup<'_>) -> Result<(), DxfError> {
        self.observe_raw(
            group.occurrence(),
            group.group_code(),
            group.raw_value(),
            group.value_line().content_span(),
        )
    }

    pub(crate) fn observe_raw(
        &mut self,
        occurrence: u64,
        group_code: DxfGroupCode,
        raw_value: &[u8],
        value_span: ByteSpan,
    ) -> Result<(), DxfError> {
        if let Some(variable) = self.pending.take() {
            self.record_candidate(variable, occurrence, group_code, raw_value, value_span)?;
        }

        if self.awaiting_section_name {
            self.awaiting_section_name = false;
            self.inside_header = group_code.value() == 2 && raw_value == b"HEADER";
        }
        if group_code.value() == 0 {
            match raw_value {
                b"SECTION" => {
                    self.inside_header = false;
                    self.awaiting_section_name = true;
                }
                b"ENDSEC" => {
                    self.inside_header = false;
                    self.awaiting_section_name = false;
                }
                _ => {}
            }
        } else if self.inside_header && group_code.value() == 9 && raw_value == b"$HANDSEED" {
            self.pending = Some(VariableMarker {
                occurrence: compact_occurrence(occurrence)?,
                span: value_span,
            });
        }
        Ok(())
    }

    pub(crate) fn finish(mut self, source_id: DxfSourceId) -> Result<DxfHandseedReport, DxfError> {
        if let Some(variable) = self.pending.take() {
            self.record_occurrence(DxfHandseedOccurrence {
                variable_occurrence: variable.occurrence,
                variable_span: variable.span,
                value_occurrence: None,
                value_span: None,
                value: DxfHandseedValue::MissingValue,
            })?;
        }

        let state = if self.occurrence_count == 0 {
            DxfHandseedState::Absent
        } else if self.occurrence_count > 1 {
            DxfHandseedState::Ambiguous
        } else {
            match self.primary.map(DxfHandseedOccurrence::value) {
                Some(DxfHandseedValue::Parsed(_)) => DxfHandseedState::Parsed,
                Some(
                    DxfHandseedValue::InvalidGroupCode(_)
                    | DxfHandseedValue::InvalidHandle(_)
                    | DxfHandseedValue::MissingValue,
                ) => DxfHandseedState::Invalid,
                None => DxfHandseedState::Absent,
            }
        };

        let mut diagnostics = Vec::new();
        for occurrence in [self.primary, self.conflicting].into_iter().flatten() {
            if !matches!(occurrence.value(), DxfHandseedValue::Parsed(_)) {
                push_diagnostic(
                    &mut diagnostics,
                    DxfDiagnostic::new(
                        DxfDiagnosticCode::HANDSEED_VALUE_INVALID,
                        occurrence.value_span().or(Some(occurrence.variable_span())),
                    ),
                )?;
            }
        }
        if let Some(conflicting) = self.conflicting {
            push_diagnostic(
                &mut diagnostics,
                DxfDiagnostic::new(
                    DxfDiagnosticCode::HANDSEED_DUPLICATE,
                    Some(conflicting.variable_span()),
                ),
            )?;
        }

        Ok(DxfHandseedReport {
            source_id,
            state,
            occurrence_count: self.occurrence_count,
            primary: self.primary,
            conflicting: self.conflicting,
            diagnostics: diagnostics.into_boxed_slice(),
        })
    }

    fn record_candidate(
        &mut self,
        variable: VariableMarker,
        occurrence: u64,
        group_code: DxfGroupCode,
        raw_value: &[u8],
        value_span: ByteSpan,
    ) -> Result<(), DxfError> {
        let value = if group_code.value() != 5 {
            DxfHandseedValue::InvalidGroupCode(group_code)
        } else {
            match parse_dxf_handle_hex(raw_value) {
                Ok(handle) => DxfHandseedValue::Parsed(handle),
                Err(issue) => DxfHandseedValue::InvalidHandle(issue),
            }
        };
        self.record_occurrence(DxfHandseedOccurrence {
            variable_occurrence: variable.occurrence,
            variable_span: variable.span,
            value_occurrence: Some(compact_occurrence(occurrence)?),
            value_span: Some(value_span),
            value,
        })
    }

    fn record_occurrence(&mut self, occurrence: DxfHandseedOccurrence) -> Result<(), DxfError> {
        self.occurrence_count = self
            .occurrence_count
            .checked_add(1)
            .ok_or_else(invalid_source_data)?;
        if self.primary.is_none() {
            self.primary = Some(occurrence);
        } else if self.conflicting.is_none() {
            self.conflicting = Some(occurrence);
        }
        Ok(())
    }
}

fn compact_occurrence(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_source_data())
}

fn push_diagnostic(
    diagnostics: &mut Vec<DxfDiagnostic>,
    diagnostic: DxfDiagnostic,
) -> Result<(), DxfError> {
    diagnostics.try_reserve(1).map_err(|_| out_of_memory())?;
    diagnostics.push(diagnostic);
    Ok(())
}

fn invalid_source_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

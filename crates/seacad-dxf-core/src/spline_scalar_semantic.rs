//! Unique scalar selection and documented SPLINE tolerance defaults.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId, DxfSplineCardDirectory,
    DxfSplineCardState, DxfSplineNumber, DxfSplineNumericIssue, DxfSplineRecordEntry,
    DxfSplineValue, DxfSplineValueRole,
};

pub const DXF_SPLINE_SCALAR_ROLES: [DxfSplineValueRole; 7] = [
    DxfSplineValueRole::Degree,
    DxfSplineValueRole::KnotCount,
    DxfSplineValueRole::ControlPointCount,
    DxfSplineValueRole::FitPointCount,
    DxfSplineValueRole::KnotTolerance,
    DxfSplineValueRole::ControlPointTolerance,
    DxfSplineValueRole::FitTolerance,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineScalarState {
    Absent,
    Defaulted(DxfSplineNumber),
    Multiple {
        occurrence_count: u32,
    },
    Invalid {
        value: DxfSplineValue,
        issue: DxfAsciiNumericIssue,
    },
    Explicit(DxfSplineValue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineScalarEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    role: DxfSplineValueRole,
    state: DxfSplineScalarState,
}

impl DxfSplineScalarEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn role(self) -> DxfSplineValueRole {
        self.role
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineScalarState {
        self.state
    }
}

/// Seven stable SPLINE scalar entries per retained record.
#[derive(Debug)]
pub struct DxfSplineScalarDirectory {
    source_id: DxfSourceId,
    cards: DxfSplineCardDirectory,
    entries: Box<[DxfSplineScalarEntry]>,
}

impl DxfSplineScalarDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.spline_card_directory(cancellation)?;
        if cards.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: cards.source_id(),
            });
        }

        let mut entries = Vec::new();
        for record in cards.evidence_directory().records().iter().copied() {
            for role in DXF_SPLINE_SCALAR_ROLES {
                ensure_not_cancelled(cancellation)?;
                let state = scalar_state(&cards, record, role)?;
                entries.try_reserve(1).map_err(|_| out_of_memory())?;
                entries.push(DxfSplineScalarEntry {
                    ordinal: compact_len(entries.len())?,
                    record,
                    role,
                    state,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfSplineCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfSplineScalarEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfSplineScalarEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw_ordinal: u64) -> Option<&[DxfSplineScalarEntry]> {
        self.cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.record().record().ordinal() < raw_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.record().record().ordinal() <= raw_ordinal);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn entry_for_role(
        &self,
        raw_ordinal: u64,
        role: DxfSplineValueRole,
    ) -> Option<DxfSplineScalarEntry> {
        self.entries_for_raw_record(raw_ordinal)?
            .iter()
            .copied()
            .find(|entry| entry.role() == role)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_scalar_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineScalarDirectory, DxfError> {
        DxfSplineScalarDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_scalar_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_scalar_directory(cancellation)
    }
}

fn scalar_state(
    cards: &DxfSplineCardDirectory,
    record: DxfSplineRecordEntry,
    role: DxfSplineValueRole,
) -> Result<DxfSplineScalarState, DxfError> {
    let card = cards
        .card_for_role(record.record().ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfSplineCardState::Absent => Ok(default_for(role)
            .map(DxfSplineScalarState::Defaulted)
            .unwrap_or(DxfSplineScalarState::Absent)),
        DxfSplineCardState::Multiple { occurrence_count } => {
            Ok(DxfSplineScalarState::Multiple { occurrence_count })
        }
        DxfSplineCardState::Unique => {
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
                Ok(number) if has_expected_wire(role, number) => {
                    Ok(DxfSplineScalarState::Explicit(value))
                }
                Err(DxfSplineNumericIssue::InvalidAsciiNumber(issue)) => {
                    Ok(DxfSplineScalarState::Invalid { value, issue })
                }
                Ok(_) => Err(invalid_internal_data()),
            }
        }
    }
}

fn default_for(role: DxfSplineValueRole) -> Option<DxfSplineNumber> {
    let value = match role {
        DxfSplineValueRole::KnotTolerance | DxfSplineValueRole::ControlPointTolerance => 1.0e-7,
        DxfSplineValueRole::FitTolerance => 1.0e-10,
        _ => return None,
    };
    Some(DxfSplineNumber::Double(DxfDouble::from_f64(value)))
}

const fn has_expected_wire(role: DxfSplineValueRole, number: DxfSplineNumber) -> bool {
    match role {
        DxfSplineValueRole::Degree
        | DxfSplineValueRole::KnotCount
        | DxfSplineValueRole::ControlPointCount
        | DxfSplineValueRole::FitPointCount => matches!(number, DxfSplineNumber::Int16(_)),
        DxfSplineValueRole::KnotTolerance
        | DxfSplineValueRole::ControlPointTolerance
        | DxfSplineValueRole::FitTolerance => matches!(number, DxfSplineNumber::Double(_)),
        _ => false,
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

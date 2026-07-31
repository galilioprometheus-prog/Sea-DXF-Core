//! Declared-versus-observed SPLINE count relations.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId, DxfSplineNumber,
    DxfSplineRecordEntry, DxfSplineScalarDirectory, DxfSplineScalarState, DxfSplineValue,
    DxfSplineValueRole,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineCountKind {
    Knot,
    ControlPoint,
    FitPoint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineCountDisposition {
    Matched,
    Mismatched,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineCountState {
    Absent,
    Multiple {
        occurrence_count: u32,
    },
    Invalid {
        value: DxfSplineValue,
        issue: DxfAsciiNumericIssue,
    },
    Negative {
        value: DxfSplineValue,
        declared: i16,
        observed: u32,
    },
    Compared {
        value: DxfSplineValue,
        declared: u16,
        observed: u32,
        disposition: DxfSplineCountDisposition,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineCountEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    kind: DxfSplineCountKind,
    state: DxfSplineCountState,
}

impl DxfSplineCountEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfSplineCountKind {
        self.kind
    }

    #[must_use]
    pub const fn state(self) -> DxfSplineCountState {
        self.state
    }
}

/// Three stable declared/observed count relations per SPLINE record.
#[derive(Debug)]
pub struct DxfSplineCountDirectory {
    source_id: DxfSourceId,
    scalars: DxfSplineScalarDirectory,
    entries: Box<[DxfSplineCountEntry]>,
}

impl DxfSplineCountDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let scalars = document.spline_scalar_directory(cancellation)?;
        if scalars.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: scalars.source_id(),
            });
        }
        let mut entries = Vec::new();
        for record in scalars
            .card_directory()
            .evidence_directory()
            .records()
            .iter()
            .copied()
        {
            for kind in [
                DxfSplineCountKind::Knot,
                DxfSplineCountKind::ControlPoint,
                DxfSplineCountKind::FitPoint,
            ] {
                ensure_not_cancelled(cancellation)?;
                let state = count_state(&scalars, record, kind)?;
                entries.try_reserve(1).map_err(|_| out_of_memory())?;
                entries.push(DxfSplineCountEntry {
                    ordinal: compact_len(entries.len())?,
                    record,
                    kind,
                    state,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            scalars,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn scalar_directory(&self) -> &DxfSplineScalarDirectory {
        &self.scalars
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfSplineCountEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfSplineCountEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw_ordinal: u64) -> Option<&[DxfSplineCountEntry]> {
        self.scalars
            .card_directory()
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
    pub fn entry_for_kind(
        &self,
        raw_ordinal: u64,
        kind: DxfSplineCountKind,
    ) -> Option<DxfSplineCountEntry> {
        self.entries_for_raw_record(raw_ordinal)?
            .iter()
            .copied()
            .find(|entry| entry.kind() == kind)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_count_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineCountDirectory, DxfError> {
        DxfSplineCountDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_count_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineCountDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_count_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_count_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineCountDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_count_directory(cancellation)
    }
}

fn count_state(
    scalars: &DxfSplineScalarDirectory,
    record: DxfSplineRecordEntry,
    kind: DxfSplineCountKind,
) -> Result<DxfSplineCountState, DxfError> {
    let (declared_role, observed_role) = roles(kind);
    let scalar = scalars
        .entry_for_role(record.record().ordinal(), declared_role)
        .ok_or_else(invalid_internal_data)?;
    let observed = scalars
        .card_directory()
        .card_for_role(record.record().ordinal(), observed_role)
        .ok_or_else(invalid_internal_data)?
        .member_range()
        .len();
    let observed = u32::try_from(observed).map_err(|_| invalid_internal_data())?;
    match scalar.state() {
        DxfSplineScalarState::Absent => Ok(DxfSplineCountState::Absent),
        DxfSplineScalarState::Multiple { occurrence_count } => {
            Ok(DxfSplineCountState::Multiple { occurrence_count })
        }
        DxfSplineScalarState::Invalid { value, issue } => {
            Ok(DxfSplineCountState::Invalid { value, issue })
        }
        DxfSplineScalarState::Explicit(value) => {
            let Ok(DxfSplineNumber::Int16(declared)) = value.value() else {
                return Err(invalid_internal_data());
            };
            let Ok(declared_u16) = u16::try_from(declared) else {
                return Ok(DxfSplineCountState::Negative {
                    value,
                    declared,
                    observed,
                });
            };
            let disposition = if u32::from(declared_u16) == observed {
                DxfSplineCountDisposition::Matched
            } else {
                DxfSplineCountDisposition::Mismatched
            };
            Ok(DxfSplineCountState::Compared {
                value,
                declared: declared_u16,
                observed,
                disposition,
            })
        }
        DxfSplineScalarState::Defaulted(_) => Err(invalid_internal_data()),
    }
}

const fn roles(kind: DxfSplineCountKind) -> (DxfSplineValueRole, DxfSplineValueRole) {
    match kind {
        DxfSplineCountKind::Knot => (DxfSplineValueRole::KnotCount, DxfSplineValueRole::KnotValue),
        DxfSplineCountKind::ControlPoint => (
            DxfSplineValueRole::ControlPointCount,
            DxfSplineValueRole::ControlPointX,
        ),
        DxfSplineCountKind::FitPoint => (
            DxfSplineValueRole::FitPointCount,
            DxfSplineValueRole::FitPointX,
        ),
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

//! Degree, knot-order, and NURBS-count relations for SPLINE records.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId, DxfSplineCardDirectory,
    DxfSplineCardState, DxfSplineFlagsSemantic, DxfSplineNumber, DxfSplineNumericIssue,
    DxfSplineRecordEntry, DxfSplineRelationDirectory, DxfSplineValue, DxfSplineValueRole,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineDegreeState {
    Absent,
    Multiple {
        occurrence_count: u32,
    },
    Invalid {
        evidence: DxfSplineValue,
        issue: DxfAsciiNumericIssue,
    },
    NonPositive {
        evidence: DxfSplineValue,
        degree: i16,
    },
    Explicit {
        evidence: DxfSplineValue,
        degree: u16,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineKnotOrderState {
    Empty,
    Nondecreasing {
        knot_count: u32,
    },
    Decreasing {
        knot_count: u32,
        first_decrease_index: u32,
    },
    Unavailable {
        knot_count: u32,
        invalid_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineInvariantDisposition {
    Satisfied,
    Contradictory,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineDegreeControlRelation {
    UnavailableDegree,
    Compared {
        degree: u16,
        control_point_count: u32,
        minimum_control_point_count: u32,
        disposition: DxfSplineInvariantDisposition,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplineNurbsCountRelation {
    UnavailableDegree,
    Compared {
        degree: u16,
        control_point_count: u32,
        knot_count: u32,
        expected_knot_count: u32,
        disposition: DxfSplineInvariantDisposition,
    },
    ExpectedCountOverflow {
        degree: u16,
        control_point_count: u32,
        knot_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSplinePeriodicClosedRelation {
    UnavailableFlags,
    Open,
    NonPeriodicClosed,
    PeriodicClosed,
    PeriodicWithoutClosed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSplineTopologyEntry {
    ordinal: u32,
    record: DxfSplineRecordEntry,
    degree: DxfSplineDegreeState,
    knot_order: DxfSplineKnotOrderState,
    degree_control: DxfSplineDegreeControlRelation,
    nurbs_count: DxfSplineNurbsCountRelation,
    periodic_closed: DxfSplinePeriodicClosedRelation,
}

impl DxfSplineTopologyEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfSplineRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn degree(self) -> DxfSplineDegreeState {
        self.degree
    }

    #[must_use]
    pub const fn knot_order(self) -> DxfSplineKnotOrderState {
        self.knot_order
    }

    #[must_use]
    pub const fn degree_control(self) -> DxfSplineDegreeControlRelation {
        self.degree_control
    }

    #[must_use]
    pub const fn nurbs_count(self) -> DxfSplineNurbsCountRelation {
        self.nurbs_count
    }

    #[must_use]
    pub const fn periodic_closed(self) -> DxfSplinePeriodicClosedRelation {
        self.periodic_closed
    }
}

#[derive(Debug)]
pub struct DxfSplineTopologyDirectory {
    source_id: DxfSourceId,
    relations: DxfSplineRelationDirectory,
    entries: Box<[DxfSplineTopologyEntry]>,
}

impl DxfSplineTopologyDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let relations = document.spline_relation_directory(cancellation)?;
        if relations.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: relations.source_id(),
            });
        }
        let cards = cards(&relations);
        let mut entries = Vec::new();
        for relation in relations.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw_ordinal = relation.record().record().ordinal();
            let degree = degree_state(cards, raw_ordinal)?;
            let knot_count = observed_count(cards, raw_ordinal, DxfSplineValueRole::KnotValue)?;
            let control_point_count =
                observed_count(cards, raw_ordinal, DxfSplineValueRole::ControlPointX)?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfSplineTopologyEntry {
                ordinal: compact_len(entries.len())?,
                record: relation.record(),
                degree,
                knot_order: knot_order(cards, raw_ordinal, cancellation)?,
                degree_control: degree_control(degree, control_point_count),
                nurbs_count: nurbs_count(degree, control_point_count, knot_count),
                periodic_closed: periodic_closed(relation.flags()),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            relations,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn relation_directory(&self) -> &DxfSplineRelationDirectory {
        &self.relations
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfSplineTopologyEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfSplineTopologyEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_raw_record(&self, raw_ordinal: u64) -> Option<DxfSplineTopologyEntry> {
        self.entries
            .binary_search_by_key(&raw_ordinal, |entry| entry.record().record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn spline_topology_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineTopologyDirectory, DxfError> {
        DxfSplineTopologyDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn spline_topology_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineTopologyDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_topology_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn spline_topology_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfSplineTopologyDirectory, DxfError> {
        DxfRawDocumentView::from(self).spline_topology_directory(cancellation)
    }
}

fn cards(relations: &DxfSplineRelationDirectory) -> &DxfSplineCardDirectory {
    relations
        .semantic_directory()
        .auxiliary_directory()
        .point_directory()
        .card_directory()
}

fn degree_state(
    cards: &DxfSplineCardDirectory,
    raw_ordinal: u64,
) -> Result<DxfSplineDegreeState, DxfError> {
    let card = cards
        .card_for_role(raw_ordinal, DxfSplineValueRole::Degree)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfSplineCardState::Absent => Ok(DxfSplineDegreeState::Absent),
        DxfSplineCardState::Multiple { occurrence_count } => {
            Ok(DxfSplineDegreeState::Multiple { occurrence_count })
        }
        DxfSplineCardState::Unique => {
            let [member] = cards
                .members_for_card(card.ordinal())
                .ok_or_else(invalid_internal_data)?
            else {
                return Err(invalid_internal_data());
            };
            let evidence = cards
                .value_for_member(*member)
                .ok_or_else(invalid_internal_data)?;
            match evidence.value() {
                Err(DxfSplineNumericIssue::InvalidAsciiNumber(issue)) => {
                    Ok(DxfSplineDegreeState::Invalid { evidence, issue })
                }
                Ok(DxfSplineNumber::Int16(degree)) if degree <= 0 => {
                    Ok(DxfSplineDegreeState::NonPositive { evidence, degree })
                }
                Ok(DxfSplineNumber::Int16(degree)) => Ok(DxfSplineDegreeState::Explicit {
                    evidence,
                    degree: u16::try_from(degree).map_err(|_| invalid_internal_data())?,
                }),
                Ok(DxfSplineNumber::Double(_)) => Err(invalid_internal_data()),
            }
        }
    }
}

fn observed_count(
    cards: &DxfSplineCardDirectory,
    raw_ordinal: u64,
    role: DxfSplineValueRole,
) -> Result<u32, DxfError> {
    let count = cards
        .card_for_role(raw_ordinal, role)
        .ok_or_else(invalid_internal_data)?
        .member_range()
        .len();
    u32::try_from(count).map_err(|_| invalid_internal_data())
}

fn knot_order(
    cards: &DxfSplineCardDirectory,
    raw_ordinal: u64,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSplineKnotOrderState, DxfError> {
    let card = cards
        .card_for_role(raw_ordinal, DxfSplineValueRole::KnotValue)
        .ok_or_else(invalid_internal_data)?;
    let members = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let knot_count = compact_len(members.len())?;
    if members.is_empty() {
        return Ok(DxfSplineKnotOrderState::Empty);
    }
    let mut invalid_count = 0_u32;
    let mut previous = None;
    let mut first_decrease_index = None;
    for (index, member) in members.iter().copied().enumerate() {
        ensure_not_cancelled(cancellation)?;
        let value = cards
            .value_for_member(member)
            .ok_or_else(invalid_internal_data)?;
        match value.value() {
            Ok(DxfSplineNumber::Double(value)) => {
                if previous.is_some_and(|previous| previous > value.to_f64())
                    && first_decrease_index.is_none()
                {
                    first_decrease_index = Some(compact_len(index)?);
                }
                previous = Some(value.to_f64());
            }
            Err(DxfSplineNumericIssue::InvalidAsciiNumber(_)) => {
                invalid_count = invalid_count
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
                previous = None;
            }
            Ok(DxfSplineNumber::Int16(_)) => return Err(invalid_internal_data()),
        }
    }
    if invalid_count != 0 {
        Ok(DxfSplineKnotOrderState::Unavailable {
            knot_count,
            invalid_count,
        })
    } else if let Some(first_decrease_index) = first_decrease_index {
        Ok(DxfSplineKnotOrderState::Decreasing {
            knot_count,
            first_decrease_index,
        })
    } else {
        Ok(DxfSplineKnotOrderState::Nondecreasing { knot_count })
    }
}

const fn degree_control(
    degree: DxfSplineDegreeState,
    control_point_count: u32,
) -> DxfSplineDegreeControlRelation {
    let DxfSplineDegreeState::Explicit { degree, .. } = degree else {
        return DxfSplineDegreeControlRelation::UnavailableDegree;
    };
    let minimum_control_point_count = degree as u32 + 1;
    DxfSplineDegreeControlRelation::Compared {
        degree,
        control_point_count,
        minimum_control_point_count,
        disposition: if control_point_count >= minimum_control_point_count {
            DxfSplineInvariantDisposition::Satisfied
        } else {
            DxfSplineInvariantDisposition::Contradictory
        },
    }
}

fn nurbs_count(
    degree: DxfSplineDegreeState,
    control_point_count: u32,
    knot_count: u32,
) -> DxfSplineNurbsCountRelation {
    let DxfSplineDegreeState::Explicit { degree, .. } = degree else {
        return DxfSplineNurbsCountRelation::UnavailableDegree;
    };
    let Some(expected_knot_count) = control_point_count
        .checked_add(degree as u32)
        .and_then(|value| value.checked_add(1))
    else {
        return DxfSplineNurbsCountRelation::ExpectedCountOverflow {
            degree,
            control_point_count,
            knot_count,
        };
    };
    DxfSplineNurbsCountRelation::Compared {
        degree,
        control_point_count,
        knot_count,
        expected_knot_count,
        disposition: if knot_count == expected_knot_count {
            DxfSplineInvariantDisposition::Satisfied
        } else {
            DxfSplineInvariantDisposition::Contradictory
        },
    }
}

const fn periodic_closed(flags: DxfSplineFlagsSemantic) -> DxfSplinePeriodicClosedRelation {
    let DxfSplineFlagsSemantic::Explicit(flags) = flags else {
        return DxfSplinePeriodicClosedRelation::UnavailableFlags;
    };
    match (flags.is_periodic(), flags.is_closed()) {
        (false, false) => DxfSplinePeriodicClosedRelation::Open,
        (false, true) => DxfSplinePeriodicClosedRelation::NonPeriodicClosed,
        (true, true) => DxfSplinePeriodicClosedRelation::PeriodicClosed,
        (true, false) => DxfSplinePeriodicClosedRelation::PeriodicWithoutClosed,
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

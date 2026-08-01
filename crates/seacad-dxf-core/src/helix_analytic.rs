//! Fail-closed composition of HELIX metadata with its embedded SPLINE data.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfHelixAxisRelation, DxfHelixHeightRelation, DxfHelixRadiusDomain, DxfHelixRecordEntry,
    DxfHelixRelationDirectory, DxfHelixRelationEntry, DxfHelixTurnsDomain, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId, DxfSplineAnalyticData, DxfSplineAnalyticDirectory,
    DxfSplineAnalyticState, DxfSplineRecordKind,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixSubclassPathState {
    Ordered {
        spline_marker_occurrence: u64,
        helix_marker_occurrence: u64,
    },
    Unavailable {
        spline_marker_count: u32,
        helix_marker_count: u32,
        first_spline_marker_occurrence: Option<u64>,
        first_helix_marker_occurrence: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixAnalyticIssueKind {
    SubclassPath,
    EmbeddedSpline,
    Axis,
    Radius,
    Turns,
    Height,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DxfHelixAnalyticIssues {
    bits: u8,
}

impl DxfHelixAnalyticIssues {
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.bits
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    #[must_use]
    pub const fn contains(self, kind: DxfHelixAnalyticIssueKind) -> bool {
        self.bits & issue_bit(kind) != 0
    }

    fn insert(&mut self, kind: DxfHelixAnalyticIssueKind) {
        self.bits |= issue_bit(kind);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixAnalyticData {
    pub embedded_spline: DxfSplineAnalyticData,
    pub axis: DxfHelixAxisRelation,
    pub radius: DxfHelixRadiusDomain,
    pub turns: DxfHelixTurnsDomain,
    pub height: DxfHelixHeightRelation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHelixAnalyticState {
    Available,
    Unavailable(DxfHelixAnalyticIssues),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHelixAnalyticEntry {
    ordinal: u32,
    record: DxfHelixRecordEntry,
    subclass_path: DxfHelixSubclassPathState,
    state: DxfHelixAnalyticState,
    data: Option<DxfHelixAnalyticData>,
}

impl DxfHelixAnalyticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn record(self) -> DxfHelixRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn subclass_path(self) -> DxfHelixSubclassPathState {
        self.subclass_path
    }

    #[must_use]
    pub const fn state(self) -> DxfHelixAnalyticState {
        self.state
    }

    #[must_use]
    pub const fn data(self) -> Option<DxfHelixAnalyticData> {
        self.data
    }
}

#[derive(Debug)]
pub struct DxfHelixAnalyticDirectory {
    source_id: DxfSourceId,
    relations: DxfHelixRelationDirectory,
    splines: DxfSplineAnalyticDirectory,
    entries: Box<[DxfHelixAnalyticEntry]>,
}

impl DxfHelixAnalyticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let relations = document.helix_relation_directory(cancellation)?;
        let splines = document.spline_analytic_directory(cancellation)?;
        ensure_source(document.source_id(), relations.source_id())?;
        ensure_source(document.source_id(), splines.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(relations.entries().len())
            .map_err(|_| out_of_memory())?;
        for relation in relations.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let raw = relation.record().entity().record().ordinal();
            let spline = splines
                .entry_for_raw_record(raw)
                .ok_or_else(invalid_internal_data)?;
            if spline.record.kind() != DxfSplineRecordKind::Helix {
                return Err(invalid_internal_data());
            }
            let subclass_path = subclass_path(document, &relations, relation)?;
            let (state, data) = analytic_state(relation, spline.state, subclass_path)?;
            entries.push(DxfHelixAnalyticEntry {
                ordinal: compact_len(entries.len())?,
                record: relation.record(),
                subclass_path,
                state,
                data,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            relations,
            splines,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn relation_directory(&self) -> &DxfHelixRelationDirectory {
        &self.relations
    }

    #[must_use]
    pub const fn spline_directory(&self) -> &DxfSplineAnalyticDirectory {
        &self.splines
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHelixAnalyticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHelixAnalyticEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_raw_record(&self, raw: u64) -> Option<DxfHelixAnalyticEntry> {
        self.entries
            .binary_search_by_key(&raw, |entry| entry.record().entity().record().ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn helix_analytic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixAnalyticDirectory, DxfError> {
        DxfHelixAnalyticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn helix_analytic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixAnalyticDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_analytic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn helix_analytic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHelixAnalyticDirectory, DxfError> {
        DxfRawDocumentView::from(self).helix_analytic_directory(cancellation)
    }
}

fn subclass_path(
    document: DxfRawDocumentView<'_>,
    relations: &DxfHelixRelationDirectory,
    relation: DxfHelixRelationEntry,
) -> Result<DxfHelixSubclassPathState, DxfError> {
    let evidence = relations
        .scalar_directory()
        .card_directory()
        .evidence_directory();
    let markers = evidence
        .entity_directory()
        .subclass_path(relation.record().entity())?;
    let mut spline_count = 0_u32;
    let mut helix_count = 0_u32;
    let mut first_spline = None;
    let mut first_helix = None;
    for marker in markers.iter().copied() {
        let group = marker.group();
        if document.raw_span_equals_exact(group.value_payload_span(), b"AcDbSpline")? {
            spline_count = spline_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            first_spline.get_or_insert(group.occurrence());
        } else if document.raw_span_equals_exact(group.value_payload_span(), b"AcDbHelix")? {
            helix_count = helix_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            first_helix.get_or_insert(group.occurrence());
        }
    }
    match (spline_count, helix_count, first_spline, first_helix) {
        (1, 1, Some(spline), Some(helix)) if spline < helix => {
            Ok(DxfHelixSubclassPathState::Ordered {
                spline_marker_occurrence: spline,
                helix_marker_occurrence: helix,
            })
        }
        _ => Ok(DxfHelixSubclassPathState::Unavailable {
            spline_marker_count: spline_count,
            helix_marker_count: helix_count,
            first_spline_marker_occurrence: first_spline,
            first_helix_marker_occurrence: first_helix,
        }),
    }
}

fn analytic_state(
    relation: DxfHelixRelationEntry,
    spline: DxfSplineAnalyticState,
    subclass_path: DxfHelixSubclassPathState,
) -> Result<(DxfHelixAnalyticState, Option<DxfHelixAnalyticData>), DxfError> {
    let mut issues = DxfHelixAnalyticIssues::default();
    if !matches!(subclass_path, DxfHelixSubclassPathState::Ordered { .. }) {
        issues.insert(DxfHelixAnalyticIssueKind::SubclassPath);
    }
    let spline = match spline {
        DxfSplineAnalyticState::Available(data) => Some(data),
        DxfSplineAnalyticState::Unavailable(_) => {
            issues.insert(DxfHelixAnalyticIssueKind::EmbeddedSpline);
            None
        }
    };
    if !matches!(
        relation.axis(),
        DxfHelixAxisRelation::Compared {
            exactly_perpendicular: true,
            ..
        }
    ) {
        issues.insert(DxfHelixAnalyticIssueKind::Axis);
    }
    if !matches!(relation.radius(), DxfHelixRadiusDomain::NonNegative { .. }) {
        issues.insert(DxfHelixAnalyticIssueKind::Radius);
    }
    if !matches!(
        relation.turns(),
        DxfHelixTurnsDomain::WithinCommandLimit { .. }
            | DxfHelixTurnsDomain::AboveCommandLimit { .. }
    ) {
        issues.insert(DxfHelixAnalyticIssueKind::Turns);
    }
    if !matches!(relation.height(), DxfHelixHeightRelation::Compared { .. }) {
        issues.insert(DxfHelixAnalyticIssueKind::Height);
    }
    if !issues.is_empty() {
        return Ok((DxfHelixAnalyticState::Unavailable(issues), None));
    }
    Ok((
        DxfHelixAnalyticState::Available,
        Some(DxfHelixAnalyticData {
            embedded_spline: spline.ok_or_else(invalid_internal_data)?,
            axis: relation.axis(),
            radius: relation.radius(),
            turns: relation.turns(),
            height: relation.height(),
        }),
    ))
}

const fn issue_bit(kind: DxfHelixAnalyticIssueKind) -> u8 {
    1_u8 << kind as u8
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
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

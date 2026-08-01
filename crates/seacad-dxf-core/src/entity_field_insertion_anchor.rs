//! Source-bound insertion anchors for absent common-entity singletons.

use std::io;

use crate::{
    DxfAcadVersion, DxfAcadVersionState, DxfApplicationGroupState, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfEntityClassification, DxfEntityField,
    DxfEntityFieldCardState, DxfEntityFieldCardinality, DxfEntityFieldEvidenceDirectory,
    DxfEntityFieldScope, DxfEntityKey, DxfEntityKnownClassification, DxfEntityRef, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
};

/// Typed reason why an absent common field has no safe singleton anchor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldInsertionAnchorIssue {
    DialectUnavailable {
        state: DxfAcadVersionState,
    },
    MissingEntity {
        raw_record_ordinal: u64,
    },
    WrongSection {
        classification: DxfEntityKnownClassification,
    },
    FieldAlreadyPresent {
        occurrence_count: u32,
    },
    SequenceOperationRequired {
        occurrence_count: u32,
    },
    NestedStructureOperationRequired {
        occurrence_count: u32,
    },
    MissingAcDbEntitySubclass,
    DuplicateAcDbEntitySubclass {
        occurrence_count: u32,
    },
    ApplicationGroupUnavailable {
        start_group_occurrence: u64,
        state: DxfApplicationGroupState,
    },
    ConflictingCommonFieldOrder {
        lower_group_occurrence: u64,
        higher_group_occurrence: u64,
    },
}

/// One exact between-group byte position for a future common-field insertion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityFieldInsertionAnchor {
    key: DxfEntityKey,
    field: DxfEntityField,
    byte_offset: u64,
    preceding_group_occurrence: Option<u32>,
    following_group_occurrence: Option<u32>,
}

impl DxfEntityFieldInsertionAnchor {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.key.source_id()
    }

    #[must_use]
    pub const fn key(self) -> DxfEntityKey {
        self.key
    }

    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn byte_offset(self) -> u64 {
        self.byte_offset
    }

    #[must_use]
    pub const fn preceding_group_occurrence(self) -> Option<u64> {
        match self.preceding_group_occurrence {
            Some(occurrence) => Some(occurrence as u64),
            None => None,
        }
    }

    #[must_use]
    pub const fn following_group_occurrence(self) -> Option<u64> {
        match self.following_group_occurrence {
            Some(occurrence) => Some(occurrence as u64),
            None => None,
        }
    }
}

/// Typed result of locating a future common-field singleton insertion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldInsertionAnchorOutcome {
    Unavailable(DxfEntityFieldInsertionAnchorIssue),
    Planned(DxfEntityFieldInsertionAnchor),
}

impl DxfRawDocumentView<'_> {
    /// Locates a canonical between-group anchor without editing the source.
    pub fn plan_entity_field_insertion_anchor(
        self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldInsertionAnchorOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), evidence.source_id())?;
        ensure_source(self.source_id(), key.source_id())?;
        let version = match self.acad_version_report().state() {
            DxfAcadVersionState::Supported(version) => version,
            state => {
                return Ok(unavailable(
                    DxfEntityFieldInsertionAnchorIssue::DialectUnavailable { state },
                ));
            }
        };
        let Some(entity) = evidence.entity_directory().entity_for_key(key)? else {
            return Ok(unavailable(
                DxfEntityFieldInsertionAnchorIssue::MissingEntity {
                    raw_record_ordinal: key.raw_record_ordinal(),
                },
            ));
        };
        if let DxfEntityClassification::WrongSection(classification) = entity.classification() {
            return Ok(unavailable(
                DxfEntityFieldInsertionAnchorIssue::WrongSection { classification },
            ));
        }
        let descriptor = field.descriptor().ok_or_else(invalid_internal_data)?;
        let card = evidence
            .card_for_field(entity, field)?
            .ok_or_else(invalid_internal_data)?;
        if descriptor.cardinality() == DxfEntityFieldCardinality::OptionalSequence {
            let occurrence_count = match card.state() {
                DxfEntityFieldCardState::AbsentOptional => 0,
                DxfEntityFieldCardState::Sequence { occurrence_count } => occurrence_count,
                _ => return Err(invalid_internal_data()),
            };
            return Ok(unavailable(
                DxfEntityFieldInsertionAnchorIssue::SequenceOperationRequired { occurrence_count },
            ));
        }
        match card.state() {
            DxfEntityFieldCardState::Unique => {
                return Ok(unavailable(
                    DxfEntityFieldInsertionAnchorIssue::FieldAlreadyPresent {
                        occurrence_count: 1,
                    },
                ));
            }
            DxfEntityFieldCardState::Duplicate { occurrence_count } => {
                return Ok(unavailable(
                    DxfEntityFieldInsertionAnchorIssue::FieldAlreadyPresent { occurrence_count },
                ));
            }
            DxfEntityFieldCardState::Sequence { .. } => return Err(invalid_internal_data()),
            DxfEntityFieldCardState::AbsentRequired | DxfEntityFieldCardState::AbsentOptional => {}
        }
        if descriptor.scope() == DxfEntityFieldScope::ExtensionDictionaryApplicationGroup {
            return Ok(unavailable(
                DxfEntityFieldInsertionAnchorIssue::NestedStructureOperationRequired {
                    occurrence_count: 0,
                },
            ));
        }

        let boundary = match descriptor.scope() {
            DxfEntityFieldScope::EntityPreamble if field == DxfEntityField::HANDLE => entity
                .record()
                .marker_occurrence()
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?,
            DxfEntityFieldScope::EntityPreamble => {
                if version == DxfAcadVersion::Ac1009 {
                    match legacy_common_scope_start(self, evidence, entity, cancellation)? {
                        Ok(boundary) => boundary,
                        Err(issue) => return Ok(unavailable(issue)),
                    }
                } else {
                    match acdb_entity_scope(self, evidence, entity, cancellation)? {
                        Ok(scope) => scope.marker.occurrence(),
                        Err(issue) => return Ok(unavailable(issue)),
                    }
                }
            }
            DxfEntityFieldScope::AcDbEntity => {
                let (start, end) = if version == DxfAcadVersion::Ac1009 {
                    let start =
                        match legacy_common_scope_start(self, evidence, entity, cancellation)? {
                            Ok(start) => start,
                            Err(issue) => return Ok(unavailable(issue)),
                        };
                    (start, entity.record().group_range().end())
                } else {
                    let scope = match acdb_entity_scope(self, evidence, entity, cancellation)? {
                        Ok(scope) => scope,
                        Err(issue) => return Ok(unavailable(issue)),
                    };
                    (
                        scope
                            .marker
                            .occurrence()
                            .checked_add(1)
                            .ok_or_else(invalid_internal_data)?,
                        scope.end,
                    )
                };
                match ordered_common_boundary(evidence, entity, field, start, end)? {
                    Ok(boundary) => boundary,
                    Err(issue) => return Ok(unavailable(issue)),
                }
            }
            DxfEntityFieldScope::ExtensionDictionaryApplicationGroup => {
                return Err(invalid_internal_data());
            }
        };
        make_anchor(self, key, field, entity, boundary)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn plan_entity_field_insertion_anchor(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldInsertionAnchorOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_insertion_anchor(
            evidence,
            key,
            field,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn plan_entity_field_insertion_anchor(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldInsertionAnchorOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_insertion_anchor(
            evidence,
            key,
            field,
            cancellation,
        )
    }
}

#[derive(Clone, Copy)]
struct AcDbEntityScope {
    marker: DxfRawGroup,
    end: u64,
}

fn acdb_entity_scope(
    document: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    entity: DxfEntityRef,
    cancellation: &DxfCancellationToken,
) -> Result<Result<AcDbEntityScope, DxfEntityFieldInsertionAnchorIssue>, DxfError> {
    let path = evidence.entity_directory().subclass_path(entity)?;
    let mut matching = None;
    let mut occurrence_count = 0_u32;
    for marker in path.iter().copied() {
        ensure_not_cancelled(cancellation)?;
        if document.raw_span_equals_exact(marker.group().value_payload_span(), b"AcDbEntity")? {
            occurrence_count = occurrence_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            if matching.is_none() {
                matching = Some(marker.group());
            }
        }
    }
    let Some(marker) = matching else {
        return Ok(Err(
            DxfEntityFieldInsertionAnchorIssue::MissingAcDbEntitySubclass,
        ));
    };
    if occurrence_count != 1 {
        return Ok(Err(
            DxfEntityFieldInsertionAnchorIssue::DuplicateAcDbEntitySubclass { occurrence_count },
        ));
    }
    let end = path
        .iter()
        .map(|entry| entry.group())
        .find(|entry| entry.occurrence() > marker.occurrence())
        .map_or(entity.record().group_range().end(), DxfRawGroup::occurrence);
    Ok(Ok(AcDbEntityScope { marker, end }))
}

fn legacy_common_scope_start(
    document: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    entity: DxfEntityRef,
    cancellation: &DxfCancellationToken,
) -> Result<Result<u64, DxfEntityFieldInsertionAnchorIssue>, DxfError> {
    let mut cursor = entity
        .record()
        .marker_occurrence()
        .checked_add(1)
        .ok_or_else(invalid_internal_data)?;
    let end = entity.record().group_range().end();
    let groups = evidence
        .application_group_directory()
        .groups_for_record(entity.record().ordinal())
        .ok_or_else(invalid_internal_data)?;
    while cursor < end {
        ensure_not_cancelled(cancellation)?;
        if let Some(group) = groups
            .iter()
            .copied()
            .find(|group| group.source_group_range().start() == cursor)
        {
            if group.state() != DxfApplicationGroupState::Closed {
                return Ok(Err(
                    DxfEntityFieldInsertionAnchorIssue::ApplicationGroupUnavailable {
                        start_group_occurrence: cursor,
                        state: group.state(),
                    },
                ));
            }
            cursor = group.source_group_range().end();
            continue;
        }
        let raw = document.group(cursor).ok_or_else(invalid_internal_data)?;
        if matches!(raw.group_code().value(), 5 | 330) {
            cursor = cursor.checked_add(1).ok_or_else(invalid_internal_data)?;
        } else {
            break;
        }
    }
    Ok(Ok(cursor))
}

fn ordered_common_boundary(
    evidence: &DxfEntityFieldEvidenceDirectory,
    entity: DxfEntityRef,
    field: DxfEntityField,
    start: u64,
    end: u64,
) -> Result<Result<u64, DxfEntityFieldInsertionAnchorIssue>, DxfError> {
    if start > end {
        return Err(invalid_internal_data());
    }
    let target = field
        .descriptor()
        .ok_or_else(invalid_internal_data)?
        .write_order();
    let mut lower = None;
    let mut higher = None;
    for occurrence in evidence.occurrences_for_entity(entity)?.iter().copied() {
        let descriptor = occurrence
            .field()
            .descriptor()
            .ok_or_else(invalid_internal_data)?;
        let ordinal = occurrence.group().occurrence();
        if descriptor.scope() != DxfEntityFieldScope::AcDbEntity
            || ordinal < start
            || ordinal >= end
        {
            continue;
        }
        if descriptor.write_order() < target
            && lower.is_none_or(|group: DxfRawGroup| group.occurrence() < ordinal)
        {
            lower = Some(occurrence.group());
        } else if descriptor.write_order() > target
            && higher.is_none_or(|group: DxfRawGroup| group.occurrence() > ordinal)
        {
            higher = Some(occurrence.group());
        }
    }
    let lower_boundary = lower
        .map(DxfRawGroup::occurrence)
        .map(|ordinal| ordinal.checked_add(1).ok_or_else(invalid_internal_data))
        .transpose()?;
    if let (Some(lower_group), Some(lower_boundary), Some(higher_group)) =
        (lower, lower_boundary, higher)
        && lower_boundary > higher_group.occurrence()
    {
        return Ok(Err(
            DxfEntityFieldInsertionAnchorIssue::ConflictingCommonFieldOrder {
                lower_group_occurrence: lower_group.occurrence(),
                higher_group_occurrence: higher_group.occurrence(),
            },
        ));
    }
    let boundary = higher
        .map(DxfRawGroup::occurrence)
        .or(lower_boundary)
        .unwrap_or(start);
    if boundary < start || boundary > end {
        return Err(invalid_internal_data());
    }
    Ok(Ok(boundary))
}

fn make_anchor(
    document: DxfRawDocumentView<'_>,
    key: DxfEntityKey,
    field: DxfEntityField,
    entity: DxfEntityRef,
    boundary: u64,
) -> Result<DxfEntityFieldInsertionAnchorOutcome, DxfError> {
    let record = entity.record();
    if boundary <= record.marker_occurrence() || boundary > record.group_range().end() {
        return Err(invalid_internal_data());
    }
    let byte_offset = if boundary < record.group_range().end() {
        document
            .group(boundary)
            .ok_or_else(invalid_internal_data)?
            .full_span()
            .start()
    } else {
        document
            .group(boundary.checked_sub(1).ok_or_else(invalid_internal_data)?)
            .ok_or_else(invalid_internal_data)?
            .full_span()
            .end()
    };
    Ok(DxfEntityFieldInsertionAnchorOutcome::Planned(
        DxfEntityFieldInsertionAnchor {
            key,
            field,
            byte_offset,
            preceding_group_occurrence: Some(
                u32::try_from(boundary - 1).map_err(|_| invalid_internal_data())?,
            ),
            following_group_occurrence: if boundary < record.group_range().end() {
                Some(u32::try_from(boundary).map_err(|_| invalid_internal_data())?)
            } else {
                None
            },
        },
    ))
}

const fn unavailable(
    issue: DxfEntityFieldInsertionAnchorIssue,
) -> DxfEntityFieldInsertionAnchorOutcome {
    DxfEntityFieldInsertionAnchorOutcome::Unavailable(issue)
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
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

//! Atomic typed edits for canonical POINT family fields.

use std::io;

use crate::{
    ByteSpan, DxfAcadVersion, DxfAcadVersionState, DxfBasicGeometryCardDirectory,
    DxfBasicGeometryComponentCardState, DxfBasicGeometryComponentRole, DxfCancellationToken,
    DxfDouble, DxfEntityClassification, DxfEntityFieldEvidenceDirectory, DxfEntityFieldWireType,
    DxfEntityGroupEncodeIssue, DxfEntityGroupEncoder, DxfEntityKey, DxfEntityRef, DxfEntityTopic,
    DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfResourceProfile,
    DxfTransactionPlan,
};

use crate::entity_field_insertion::encoded_group_insertion_bytes;

/// One typed POINT-family update.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPointPatch {
    SetLocation { location: [DxfDouble; 3] },
    SetThickness { thickness: DxfDouble },
}

impl DxfPointPatch {
    #[must_use]
    pub const fn set_location(location: [DxfDouble; 3]) -> Self {
        Self::SetLocation { location }
    }

    #[must_use]
    pub const fn set_thickness(thickness: DxfDouble) -> Self {
        Self::SetThickness { thickness }
    }

    #[must_use]
    pub const fn kind(self) -> DxfPointPatchKind {
        match self {
            Self::SetLocation { .. } => DxfPointPatchKind::Location,
            Self::SetThickness { .. } => DxfPointPatchKind::Thickness,
        }
    }
}

/// Payload-free identity of one POINT patch.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPointPatchKind {
    Location,
    Thickness,
}

/// Typed reason why a POINT-family update was rejected.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPointEditIssue {
    DuplicatePatch {
        key: DxfEntityKey,
        kind: DxfPointPatchKind,
    },
    MissingEntity {
        key: DxfEntityKey,
    },
    WrongClassification {
        key: DxfEntityKey,
        observed: DxfEntityClassification,
    },
    VersionUnavailable {
        state: DxfAcadVersionState,
    },
    MissingLocationComponent {
        role: DxfBasicGeometryComponentRole,
    },
    DuplicateLocationComponent {
        role: DxfBasicGeometryComponentRole,
        occurrence_count: u32,
    },
    DuplicateThickness {
        occurrence_count: u32,
    },
    Encoding {
        group_code: i16,
        issue: DxfEntityGroupEncodeIssue,
    },
}

pub(crate) struct DxfPointThicknessEditPlan {
    transaction: DxfTransactionPlan,
    thickness: DxfDouble,
}

impl DxfPointThicknessEditPlan {
    pub(crate) fn into_parts(self) -> (DxfTransactionPlan, DxfDouble) {
        (self.transaction, self.thickness)
    }
}

pub(crate) struct DxfPointLocationEditPlan {
    transaction: DxfTransactionPlan,
    location: [DxfDouble; 3],
}

impl DxfPointLocationEditPlan {
    pub(crate) fn into_parts(self) -> (DxfTransactionPlan, [DxfDouble; 3]) {
        (self.transaction, self.location)
    }
}

pub(crate) fn plan_point_location_edit(
    document: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    location: [DxfDouble; 3],
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfPointLocationEditPlan, DxfPointEditIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let (entity, version) = match point_edit_context(document, evidence, key)? {
        Ok(context) => context,
        Err(issue) => return Ok(Err(issue)),
    };
    let cards = document.basic_geometry_card_directory(cancellation)?;
    let roles = [
        DxfBasicGeometryComponentRole::WcsLocationOrStartX,
        DxfBasicGeometryComponentRole::WcsLocationOrStartY,
        DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
    ];
    let group_codes = [10_i16, 20, 30];
    let encoder = DxfEntityGroupEncoder::new(document.format(), version, profile);
    let mut builder = document.transaction_plan_builder(profile)?;
    for ((role, group_code), value) in roles.into_iter().zip(group_codes).zip(location) {
        ensure_not_cancelled(cancellation)?;
        let card = cards
            .card_for_role(entity.record().ordinal(), role)
            .ok_or_else(invalid_internal_data)?;
        match card.state() {
            DxfBasicGeometryComponentCardState::Absent => {
                return Ok(Err(DxfPointEditIssue::MissingLocationComponent { role }));
            }
            DxfBasicGeometryComponentCardState::Multiple { occurrence_count } => {
                return Ok(Err(DxfPointEditIssue::DuplicateLocationComponent {
                    role,
                    occurrence_count,
                }));
            }
            DxfBasicGeometryComponentCardState::Unique => {}
        }
        let members = cards
            .members_for_card(card.ordinal())
            .ok_or_else(invalid_internal_data)?;
        let [member] = members else {
            return Err(invalid_internal_data());
        };
        let component = cards
            .component_for_member(*member)
            .ok_or_else(invalid_internal_data)?;
        let encoded = match encoder.encode_raw(
            group_code,
            DxfEntityFieldWireType::Double,
            crate::DxfEntityEditValue::Double(value),
            cancellation,
        )? {
            Ok(encoded) => encoded,
            Err(issue) => {
                return Ok(Err(DxfPointEditIssue::Encoding { group_code, issue }));
            }
        };
        builder.replace_raw_span(component.group().full_span(), &encoded, cancellation)?;
    }
    let transaction = builder.finish(cancellation)?;
    ensure_not_cancelled(cancellation)?;
    Ok(Ok(DxfPointLocationEditPlan {
        transaction,
        location,
    }))
}

pub(crate) fn plan_point_thickness_edit(
    document: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    thickness: DxfDouble,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfPointThicknessEditPlan, DxfPointEditIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let (entity, version) = match point_edit_context(document, evidence, key)? {
        Ok(context) => context,
        Err(issue) => return Ok(Err(issue)),
    };
    let cards = document.basic_geometry_card_directory(cancellation)?;
    let card = cards
        .card_for_role(
            entity.record().ordinal(),
            DxfBasicGeometryComponentRole::Thickness,
        )
        .ok_or_else(invalid_internal_data)?;
    let unique_thickness = match card.state() {
        DxfBasicGeometryComponentCardState::Absent => None,
        DxfBasicGeometryComponentCardState::Multiple { occurrence_count } => {
            return Ok(Err(DxfPointEditIssue::DuplicateThickness {
                occurrence_count,
            }));
        }
        DxfBasicGeometryComponentCardState::Unique => {
            let members = cards
                .members_for_card(card.ordinal())
                .ok_or_else(invalid_internal_data)?;
            let [member] = members else {
                return Err(invalid_internal_data());
            };
            Some(
                cards
                    .component_for_member(*member)
                    .ok_or_else(invalid_internal_data)?,
            )
        }
    };
    let encoder = DxfEntityGroupEncoder::new(document.format(), version, profile);
    let encoded = match encoder.encode_raw(
        39,
        DxfEntityFieldWireType::Double,
        crate::DxfEntityEditValue::Double(thickness),
        cancellation,
    )? {
        Ok(encoded) => encoded,
        Err(issue) => {
            return Ok(Err(DxfPointEditIssue::Encoding {
                group_code: 39,
                issue,
            }));
        }
    };
    let mut builder = document.transaction_plan_builder(profile)?;
    if let Some(component) = unique_thickness {
        builder.replace_raw_span(component.group().full_span(), &encoded, cancellation)?;
    } else {
        let preceding =
            match point_thickness_insertion_predecessor(&cards, entity.record().ordinal())? {
                Ok(group) => group,
                Err(issue) => return Ok(Err(issue)),
            };
        let insertion = encoded_group_insertion_bytes(
            document,
            preceding.occurrence(),
            &encoded,
            cancellation,
        )?;
        let offset = preceding.full_span().end();
        let source_span = ByteSpan::new(offset, offset).ok_or_else(invalid_internal_data)?;
        builder.replace_raw_span(source_span, &insertion, cancellation)?;
    }
    let transaction = builder.finish(cancellation)?;
    ensure_not_cancelled(cancellation)?;
    Ok(Ok(DxfPointThicknessEditPlan {
        transaction,
        thickness,
    }))
}

fn point_thickness_insertion_predecessor(
    cards: &DxfBasicGeometryCardDirectory,
    raw_record_ordinal: u64,
) -> Result<Result<DxfRawGroup, DxfPointEditIssue>, DxfError> {
    let mut preceding = None;
    for role in [
        DxfBasicGeometryComponentRole::WcsLocationOrStartX,
        DxfBasicGeometryComponentRole::WcsLocationOrStartY,
        DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
    ] {
        let card = cards
            .card_for_role(raw_record_ordinal, role)
            .ok_or_else(invalid_internal_data)?;
        match card.state() {
            DxfBasicGeometryComponentCardState::Absent => {
                return Ok(Err(DxfPointEditIssue::MissingLocationComponent { role }));
            }
            DxfBasicGeometryComponentCardState::Multiple { occurrence_count } => {
                return Ok(Err(DxfPointEditIssue::DuplicateLocationComponent {
                    role,
                    occurrence_count,
                }));
            }
            DxfBasicGeometryComponentCardState::Unique => {}
        }
        let members = cards
            .members_for_card(card.ordinal())
            .ok_or_else(invalid_internal_data)?;
        let [member] = members else {
            return Err(invalid_internal_data());
        };
        let group = cards
            .component_for_member(*member)
            .ok_or_else(invalid_internal_data)?
            .group();
        if preceding.is_none_or(|current: DxfRawGroup| current.occurrence() < group.occurrence()) {
            preceding = Some(group);
        }
    }
    preceding.ok_or_else(invalid_internal_data).map(Ok)
}

fn point_edit_context(
    document: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
) -> Result<Result<(DxfEntityRef, DxfAcadVersion), DxfPointEditIssue>, DxfError> {
    if evidence.source_id() != document.source_id() {
        return Err(DxfError::SourceIdentityMismatch {
            expected: document.source_id(),
            observed: evidence.source_id(),
        });
    }
    let Some(entity) = evidence.entity_directory().entity_for_key(key)? else {
        return Ok(Err(DxfPointEditIssue::MissingEntity { key }));
    };
    if entity.classification() != DxfEntityClassification::Canonical(DxfEntityTopic::POINT) {
        return Ok(Err(DxfPointEditIssue::WrongClassification {
            key,
            observed: entity.classification(),
        }));
    }
    let version = match document.acad_version_report().state() {
        DxfAcadVersionState::Supported(version) => version,
        state => return Ok(Err(DxfPointEditIssue::VersionUnavailable { state })),
    };
    Ok(Ok((entity, version)))
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

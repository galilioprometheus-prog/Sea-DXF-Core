use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfApplicationGroupEntry, DxfApplicationGroupKind,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityClassification,
    DxfEntityDirectory, DxfEntityField, DxfEntityFieldCardinality, DxfEntityFieldScope,
    DxfEntityRef, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    dxf_entity_common_fields,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityFieldOccurrence {
    entity: DxfEntityRef,
    field: DxfEntityField,
    group: DxfRawGroup,
    application_group_start_control_ordinal: Option<u32>,
}

impl DxfEntityFieldOccurrence {
    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn application_group_start_control_ordinal(self) -> Option<u64> {
        match self.application_group_start_control_ordinal {
            Some(ordinal) => Some(ordinal as u64),
            None => None,
        }
    }
}

/// Cardinality outcome for one entity and one common-field descriptor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldCardState {
    AbsentRequired,
    AbsentOptional,
    Unique,
    Duplicate { occurrence_count: u32 },
    Sequence { occurrence_count: u32 },
}

/// Half-open range of card-member ordinals.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityFieldCardMemberRange {
    start: u32,
    end: u32,
}

impl DxfEntityFieldCardMemberRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityFieldCardMember {
    occurrence_ordinal: u32,
}

impl DxfEntityFieldCardMember {
    #[must_use]
    pub const fn occurrence_ordinal(self) -> u64 {
        self.occurrence_ordinal as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityFieldCard {
    ordinal: u32,
    entity: DxfEntityRef,
    field: DxfEntityField,
    member_range: DxfEntityFieldCardMemberRange,
    state: DxfEntityFieldCardState,
}

impl DxfEntityFieldCard {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn member_range(self) -> DxfEntityFieldCardMemberRange {
        self.member_range
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityFieldCardState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfEntityFieldEvidenceDirectory {
    source_id: DxfSourceId,
    entities: DxfEntityDirectory,
    application_groups: DxfApplicationGroupDirectory,
    occurrences: Box<[DxfEntityFieldOccurrence]>,
    cards: Box<[DxfEntityFieldCard]>,
    members: Box<[DxfEntityFieldCardMember]>,
}

impl DxfEntityFieldEvidenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let entities = document.entity_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        ensure_source(document.source_id(), entities.source_id())?;
        ensure_source(document.source_id(), application_groups.source_id())?;

        let mut occurrences = Vec::new();
        for entity in entities.entities().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if matches!(
                entity.classification(),
                DxfEntityClassification::WrongSection(_)
            ) {
                continue;
            }
            append_entity_occurrences(
                document,
                &application_groups,
                entity,
                cancellation,
                &mut occurrences,
            )?;
        }
        ensure_not_cancelled(cancellation)?;

        let semantic_entity_count = entities
            .entities()
            .iter()
            .filter(|entity| {
                !matches!(
                    entity.classification(),
                    DxfEntityClassification::WrongSection(_)
                )
            })
            .count();
        let card_capacity = semantic_entity_count
            .checked_mul(dxf_entity_common_fields().len())
            .ok_or_else(invalid_internal_data)?;
        let mut cards = Vec::new();
        cards
            .try_reserve(card_capacity)
            .map_err(|_| out_of_memory())?;
        let mut members = Vec::new();
        members
            .try_reserve(occurrences.len())
            .map_err(|_| out_of_memory())?;
        for entity in entities.entities().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if matches!(
                entity.classification(),
                DxfEntityClassification::WrongSection(_)
            ) {
                continue;
            }
            let entity_occurrences = occurrence_bounds(&occurrences, entity.record().ordinal());
            for descriptor in dxf_entity_common_fields() {
                ensure_not_cancelled(cancellation)?;
                let member_start = compact_len(members.len())?;
                for occurrence_ordinal in entity_occurrences.clone() {
                    let occurrence = occurrences
                        .get(occurrence_ordinal)
                        .ok_or_else(invalid_internal_data)?;
                    if occurrence.field() == descriptor.field() {
                        members.push(DxfEntityFieldCardMember {
                            occurrence_ordinal: compact_len(occurrence_ordinal)?,
                        });
                    }
                }
                let member_end = compact_len(members.len())?;
                let member_range = DxfEntityFieldCardMemberRange::new(member_start, member_end)?;
                let state = card_state(descriptor.cardinality(), member_range.len())?;
                cards.push(DxfEntityFieldCard {
                    ordinal: compact_len(cards.len())?,
                    entity,
                    field: descriptor.field(),
                    member_range,
                    state,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entities,
            application_groups,
            occurrences: occurrences.into_boxed_slice(),
            cards: cards.into_boxed_slice(),
            members: members.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn entity_directory(&self) -> &DxfEntityDirectory {
        &self.entities
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        &self.application_groups
    }

    #[must_use]
    pub fn occurrences(&self) -> &[DxfEntityFieldOccurrence] {
        &self.occurrences
    }

    #[must_use]
    pub fn cards(&self) -> &[DxfEntityFieldCard] {
        &self.cards
    }

    #[must_use]
    pub fn members(&self) -> &[DxfEntityFieldCardMember] {
        &self.members
    }

    pub fn occurrences_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityFieldOccurrence], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let bounds = occurrence_bounds(&self.occurrences, entity.record().ordinal());
        self.occurrences
            .get(bounds)
            .ok_or_else(invalid_internal_data)
    }

    pub fn application_group_for_occurrence(
        &self,
        occurrence: DxfEntityFieldOccurrence,
    ) -> Result<Option<DxfApplicationGroupEntry>, DxfError> {
        ensure_source(self.source_id, occurrence.entity().source_id())?;
        let Some(start_control_ordinal) = occurrence.application_group_start_control_ordinal()
        else {
            return Ok(None);
        };
        Ok(self
            .application_groups
            .groups()
            .iter()
            .copied()
            .find(|group| group.start_control_ordinal() == start_control_ordinal))
    }

    pub fn cards_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityFieldCard], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let start = self
            .cards
            .partition_point(|card| card.entity().record().ordinal() < entity.record().ordinal());
        let end = self
            .cards
            .partition_point(|card| card.entity().record().ordinal() <= entity.record().ordinal());
        self.cards.get(start..end).ok_or_else(invalid_internal_data)
    }

    pub fn card_for_field(
        &self,
        entity: DxfEntityRef,
        field: DxfEntityField,
    ) -> Result<Option<DxfEntityFieldCard>, DxfError> {
        Ok(self
            .cards_for_entity(entity)?
            .iter()
            .copied()
            .find(|card| card.field() == field))
    }

    pub fn members_for_card(
        &self,
        card: DxfEntityFieldCard,
    ) -> Result<&[DxfEntityFieldCardMember], DxfError> {
        ensure_source(self.source_id, card.entity().source_id())?;
        let start =
            usize::try_from(card.member_range().start()).map_err(|_| invalid_internal_data())?;
        let end =
            usize::try_from(card.member_range().end()).map_err(|_| invalid_internal_data())?;
        self.members
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }

    #[must_use]
    pub fn occurrence_for_member(
        &self,
        member: DxfEntityFieldCardMember,
    ) -> Option<DxfEntityFieldOccurrence> {
        let index = usize::try_from(member.occurrence_ordinal()).ok()?;
        self.occurrences.get(index).copied()
    }

    #[must_use]
    pub fn occurrence_for_group(&self, group_occurrence: u64) -> Option<DxfEntityFieldOccurrence> {
        let index = self
            .occurrences
            .partition_point(|entry| entry.group().occurrence() < group_occurrence);
        self.occurrences
            .get(index)
            .copied()
            .filter(|entry| entry.group().occurrence() == group_occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_field_evidence_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldEvidenceDirectory, DxfError> {
        DxfEntityFieldEvidenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_field_evidence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldEvidenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_field_evidence_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_field_evidence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldEvidenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_field_evidence_directory(cancellation)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CommonSubclassContext {
    Legacy,
    AcDbEntity,
    Other,
}

fn append_entity_occurrences(
    document: DxfRawDocumentView<'_>,
    application_groups: &DxfApplicationGroupDirectory,
    entity: DxfEntityRef,
    cancellation: &DxfCancellationToken,
    occurrences: &mut Vec<DxfEntityFieldOccurrence>,
) -> Result<(), DxfError> {
    let mut context = CommonSubclassContext::Legacy;
    for raw_ordinal in
        entity.record().marker_occurrence().saturating_add(1)..entity.record().group_range().end()
    {
        ensure_not_cancelled(cancellation)?;
        let group = document
            .group(raw_ordinal)
            .ok_or_else(invalid_internal_data)?;
        if let Some(application_group) =
            application_groups.group_for_content_occurrence(raw_ordinal)
        {
            if application_group.kind() == DxfApplicationGroupKind::AcadXDictionary
                && group.group_code().value() == 360
            {
                push_occurrence(
                    occurrences,
                    entity,
                    DxfEntityField::EXTENSION_DICTIONARY,
                    group,
                    Some(
                        u32::try_from(application_group.start_control_ordinal())
                            .map_err(|_| invalid_internal_data())?,
                    ),
                )?;
            }
            continue;
        }
        if group.group_code().value() == 100 {
            ensure_not_cancelled(cancellation)?;
            context =
                if document.raw_span_equals_exact(group.value_payload_span(), b"AcDbEntity")? {
                    CommonSubclassContext::AcDbEntity
                } else {
                    CommonSubclassContext::Other
                };
            ensure_not_cancelled(cancellation)?;
            continue;
        }
        let Some(field) = DxfEntityField::from_group_code(group.group_code().value()) else {
            continue;
        };
        let Some(descriptor) = field.descriptor() else {
            return Err(invalid_internal_data());
        };
        let in_scope = match descriptor.scope() {
            DxfEntityFieldScope::EntityPreamble => context == CommonSubclassContext::Legacy,
            DxfEntityFieldScope::AcDbEntity => matches!(
                context,
                CommonSubclassContext::Legacy | CommonSubclassContext::AcDbEntity
            ),
            DxfEntityFieldScope::ExtensionDictionaryApplicationGroup => false,
        };
        if in_scope {
            push_occurrence(occurrences, entity, field, group, None)?;
        }
    }
    Ok(())
}

fn push_occurrence(
    occurrences: &mut Vec<DxfEntityFieldOccurrence>,
    entity: DxfEntityRef,
    field: DxfEntityField,
    group: DxfRawGroup,
    application_group_start_control_ordinal: Option<u32>,
) -> Result<(), DxfError> {
    occurrences.try_reserve(1).map_err(|_| out_of_memory())?;
    occurrences.push(DxfEntityFieldOccurrence {
        entity,
        field,
        group,
        application_group_start_control_ordinal,
    });
    Ok(())
}

fn occurrence_bounds(
    occurrences: &[DxfEntityFieldOccurrence],
    raw_record_ordinal: u64,
) -> std::ops::Range<usize> {
    let start =
        occurrences.partition_point(|entry| entry.entity().record().ordinal() < raw_record_ordinal);
    let end = occurrences
        .partition_point(|entry| entry.entity().record().ordinal() <= raw_record_ordinal);
    start..end
}

fn card_state(
    cardinality: DxfEntityFieldCardinality,
    count: u64,
) -> Result<DxfEntityFieldCardState, DxfError> {
    let count = u32::try_from(count).map_err(|_| invalid_internal_data())?;
    Ok(match (cardinality, count) {
        (DxfEntityFieldCardinality::RequiredSingleton, 0) => {
            DxfEntityFieldCardState::AbsentRequired
        }
        (DxfEntityFieldCardinality::OptionalSingleton, 0)
        | (DxfEntityFieldCardinality::OptionalSequence, 0) => {
            DxfEntityFieldCardState::AbsentOptional
        }
        (
            DxfEntityFieldCardinality::RequiredSingleton
            | DxfEntityFieldCardinality::OptionalSingleton,
            1,
        ) => DxfEntityFieldCardState::Unique,
        (
            DxfEntityFieldCardinality::RequiredSingleton
            | DxfEntityFieldCardinality::OptionalSingleton,
            occurrence_count,
        ) => DxfEntityFieldCardState::Duplicate { occurrence_count },
        (DxfEntityFieldCardinality::OptionalSequence, occurrence_count) => {
            DxfEntityFieldCardState::Sequence { occurrence_count }
        }
    })
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

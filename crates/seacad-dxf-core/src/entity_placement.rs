//! Source-bound insertion anchors for entity-bearing containers.

use std::io;

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockDefinitionDirectory,
    DxfBlockDefinitionState, DxfCancellationToken, DxfError, DxfGroupCode, DxfIoOperation,
    DxfRawDocumentView, DxfRawRecordDirectory, DxfRawRecordSectionKind, DxfRawRecordSectionState,
    DxfSourceId,
};

/// Entity-bearing container selected for a future insertion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityPlacementTarget {
    /// One exact ENTITIES section from the structure index.
    EntitiesSection { structure_section_ordinal: u64 },
    /// The member list belonging to one exact BLOCK marker record.
    BlockDefinition { raw_record_ordinal: u64 },
}

/// A source-bound, zero-width insertion location before an existing group-zero marker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityPlacement {
    source_id: DxfSourceId,
    target: DxfEntityPlacementTarget,
    section_kind: DxfRawRecordSectionKind,
    insertion_span: ByteSpan,
    following_group_occurrence: u32,
}

impl DxfEntityPlacement {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn target(self) -> DxfEntityPlacementTarget {
        self.target
    }

    #[must_use]
    pub const fn section_kind(self) -> DxfRawRecordSectionKind {
        self.section_kind
    }

    #[must_use]
    pub const fn insertion_span(self) -> ByteSpan {
        self.insertion_span
    }

    /// Group-zero marker that will follow bytes inserted at `insertion_span`.
    #[must_use]
    pub const fn following_group_occurrence(self) -> u64 {
        self.following_group_occurrence as u64
    }
}

/// Readiness of one candidate entity-bearing container.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityPlacementState {
    Ready(DxfEntityPlacement),
    SectionUnavailable {
        state: DxfRawRecordSectionState,
    },
    BlockDefinitionUnavailable {
        state: DxfBlockDefinitionState,
    },
    InvalidAnchorGroup {
        group_occurrence: u64,
        group_code: DxfGroupCode,
    },
}

/// Placement assessment retained even when malformed structure prevents insertion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityPlacementAssessment {
    target: DxfEntityPlacementTarget,
    state: DxfEntityPlacementState,
}

impl DxfEntityPlacementAssessment {
    #[must_use]
    pub const fn target(self) -> DxfEntityPlacementTarget {
        self.target
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityPlacementState {
        self.state
    }

    #[must_use]
    pub const fn placement(self) -> Option<DxfEntityPlacement> {
        match self.state {
            DxfEntityPlacementState::Ready(placement) => Some(placement),
            DxfEntityPlacementState::SectionUnavailable { .. }
            | DxfEntityPlacementState::BlockDefinitionUnavailable { .. }
            | DxfEntityPlacementState::InvalidAnchorGroup { .. } => None,
        }
    }
}

/// Immutable directory of safe container-start insertion anchors.
#[derive(Debug)]
pub struct DxfEntityPlacementDirectory {
    source_id: DxfSourceId,
    block_definitions: DxfBlockDefinitionDirectory,
    assessments: Box<[DxfEntityPlacementAssessment]>,
}

impl DxfEntityPlacementDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let block_definitions = document.block_definition_directory(cancellation)?;
        if block_definitions.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: block_definitions.source_id(),
            });
        }

        let raw_records = block_definitions.raw_record_directory();
        let mut assessments = Vec::new();
        for section in raw_records.sections().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            match section.kind() {
                DxfRawRecordSectionKind::Entities => {
                    let target = DxfEntityPlacementTarget::EntitiesSection {
                        structure_section_ordinal: section.structure_section_ordinal(),
                    };
                    let state = if section.state() == DxfRawRecordSectionState::Indexed {
                        assess_anchor(
                            document,
                            target,
                            DxfRawRecordSectionKind::Entities,
                            section.content_group_range().start(),
                        )?
                    } else {
                        DxfEntityPlacementState::SectionUnavailable {
                            state: section.state(),
                        }
                    };
                    push_assessment(&mut assessments, target, state)?;
                }
                DxfRawRecordSectionKind::Blocks => {
                    for definition in
                        block_definitions
                            .definitions()
                            .iter()
                            .copied()
                            .filter(|definition| {
                                definition.block_record().structure_section_ordinal()
                                    == section.structure_section_ordinal()
                            })
                    {
                        ensure_not_cancelled(cancellation)?;
                        assess_block_definition(
                            document,
                            &block_definitions,
                            definition,
                            &mut assessments,
                        )?;
                    }
                }
                DxfRawRecordSectionKind::Classes
                | DxfRawRecordSectionKind::Tables
                | DxfRawRecordSectionKind::Objects => {}
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            block_definitions,
            assessments: assessments.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn block_definition_directory(&self) -> &DxfBlockDefinitionDirectory {
        &self.block_definitions
    }

    #[must_use]
    pub const fn raw_record_directory(&self) -> &DxfRawRecordDirectory {
        self.block_definitions.raw_record_directory()
    }

    #[must_use]
    pub fn assessments(&self) -> &[DxfEntityPlacementAssessment] {
        &self.assessments
    }

    #[must_use]
    pub fn assessment_for_target(
        &self,
        target: DxfEntityPlacementTarget,
    ) -> Option<DxfEntityPlacementAssessment> {
        self.assessments
            .iter()
            .copied()
            .find(|assessment| assessment.target() == target)
    }
}

fn assess_block_definition(
    document: DxfRawDocumentView<'_>,
    block_definitions: &DxfBlockDefinitionDirectory,
    definition: crate::DxfBlockDefinitionEntry,
    assessments: &mut Vec<DxfEntityPlacementAssessment>,
) -> Result<(), DxfError> {
    let raw_record_ordinal = definition.block_record().ordinal();
    let target = DxfEntityPlacementTarget::BlockDefinition { raw_record_ordinal };
    let state = if definition.state() == DxfBlockDefinitionState::Closed {
        let members = block_definitions
            .members_for_block_raw_ordinal(raw_record_ordinal)
            .ok_or_else(invalid_internal_data)?;
        let occurrence = if let Some(member) = members.first() {
            member.marker_occurrence()
        } else {
            definition
                .boundary_record()
                .ok_or_else(invalid_internal_data)?
                .marker_occurrence()
        };
        assess_anchor(
            document,
            target,
            DxfRawRecordSectionKind::Blocks,
            occurrence,
        )?
    } else {
        DxfEntityPlacementState::BlockDefinitionUnavailable {
            state: definition.state(),
        }
    };
    push_assessment(assessments, target, state)
}

impl DxfRawDocumentView<'_> {
    pub fn entity_placement_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityPlacementDirectory, DxfError> {
        DxfEntityPlacementDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_placement_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityPlacementDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_placement_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_placement_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityPlacementDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_placement_directory(cancellation)
    }
}

fn assess_anchor(
    document: DxfRawDocumentView<'_>,
    target: DxfEntityPlacementTarget,
    section_kind: DxfRawRecordSectionKind,
    occurrence: u64,
) -> Result<DxfEntityPlacementState, DxfError> {
    let group = document
        .group(occurrence)
        .ok_or_else(invalid_internal_data)?;
    if group.group_code().value() != 0 {
        return Ok(DxfEntityPlacementState::InvalidAnchorGroup {
            group_occurrence: occurrence,
            group_code: group.group_code(),
        });
    }
    let following_group_occurrence =
        u32::try_from(occurrence).map_err(|_| invalid_internal_data())?;
    let offset = group.full_span().start();
    let insertion_span = ByteSpan::new(offset, offset).ok_or_else(invalid_internal_data)?;
    Ok(DxfEntityPlacementState::Ready(DxfEntityPlacement {
        source_id: document.source_id(),
        target,
        section_kind,
        insertion_span,
        following_group_occurrence,
    }))
}

fn push_assessment(
    assessments: &mut Vec<DxfEntityPlacementAssessment>,
    target: DxfEntityPlacementTarget,
    state: DxfEntityPlacementState,
) -> Result<(), DxfError> {
    assessments.try_reserve(1).map_err(|_| out_of_memory())?;
    assessments.push(DxfEntityPlacementAssessment { target, state });
    Ok(())
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

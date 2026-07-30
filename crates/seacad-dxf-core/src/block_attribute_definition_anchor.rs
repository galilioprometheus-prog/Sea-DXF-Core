//! Classic ATTDEF placement-anchor selection.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockAttributeDefinitionDoubleSemanticDirectory,
    DxfBlockAttributeDefinitionDoubleSemantics, DxfBlockAttributeDefinitionJustificationDirectory,
    DxfBlockAttributeDefinitionJustificationSemantics, DxfBlockAttributeDefinitionValueEntry,
    DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionPlacementAnchorState {
    JustificationUnavailable,
    TextStartUnavailable,
    AlignmentPointUnavailable,
    TextStart { point: [DxfDouble; 3] },
    AlignmentPoint { point: [DxfDouble; 3] },
}

impl DxfBlockAttributeDefinitionPlacementAnchorState {
    #[must_use]
    pub const fn point(self) -> Option<[DxfDouble; 3]> {
        match self {
            Self::TextStart { point } | Self::AlignmentPoint { point } => Some(point),
            Self::JustificationUnavailable
            | Self::TextStartUnavailable
            | Self::AlignmentPointUnavailable => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionPlacementAnchor {
    justification: DxfBlockAttributeDefinitionJustificationSemantics,
    doubles: DxfBlockAttributeDefinitionDoubleSemantics,
    state: DxfBlockAttributeDefinitionPlacementAnchorState,
}

impl DxfBlockAttributeDefinitionPlacementAnchor {
    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.doubles.record()
    }

    #[must_use]
    pub const fn justification(self) -> DxfBlockAttributeDefinitionJustificationSemantics {
        self.justification
    }

    #[must_use]
    pub const fn double_semantics(self) -> DxfBlockAttributeDefinitionDoubleSemantics {
        self.doubles
    }

    #[must_use]
    pub const fn state(self) -> DxfBlockAttributeDefinitionPlacementAnchorState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionPlacementAnchorDirectory {
    source_id: DxfSourceId,
    justification: DxfBlockAttributeDefinitionJustificationDirectory,
    doubles: DxfBlockAttributeDefinitionDoubleSemanticDirectory,
}

impl DxfBlockAttributeDefinitionPlacementAnchorDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let justification =
            document.block_attribute_definition_justification_directory(cancellation)?;
        let doubles =
            document.block_attribute_definition_double_semantic_directory(cancellation)?;
        if justification.source_id() != document.source_id()
            || doubles.source_id() != document.source_id()
        {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: if justification.source_id() != document.source_id() {
                    justification.source_id()
                } else {
                    doubles.source_id()
                },
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            justification,
            doubles,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn justification_directory(
        &self,
    ) -> &DxfBlockAttributeDefinitionJustificationDirectory {
        &self.justification
    }

    #[must_use]
    pub const fn double_directory(&self) -> &DxfBlockAttributeDefinitionDoubleSemanticDirectory {
        &self.doubles
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBlockAttributeDefinitionValueEntry] {
        self.justification.records()
    }

    pub fn anchor_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionPlacementAnchor>, DxfError> {
        let Some(justification) = self
            .justification
            .semantics_for_raw_record(raw_record_ordinal)?
        else {
            return Ok(None);
        };
        let doubles = self
            .doubles
            .semantics_for_raw_record(raw_record_ordinal)?
            .ok_or_else(invalid_internal_data)?;
        placement_anchor(justification, doubles).map(Some)
    }

    pub fn anchor_for_entry(
        &self,
        record: DxfBlockAttributeDefinitionValueEntry,
    ) -> Result<Option<DxfBlockAttributeDefinitionPlacementAnchor>, DxfError> {
        let Some(justification) = self.justification.semantics_for_entry(record)? else {
            return Ok(None);
        };
        let doubles = self
            .doubles
            .semantics_for_entry(record)?
            .ok_or_else(invalid_internal_data)?;
        placement_anchor(justification, doubles).map(Some)
    }

    pub fn anchor_for_block_attribute_definition(
        &self,
        block_raw_ordinal: u64,
        attribute_definition_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionPlacementAnchor>, DxfError> {
        let Some(justification) = self
            .justification
            .semantics_for_block_attribute_definition(
                block_raw_ordinal,
                attribute_definition_ordinal,
            )?
        else {
            return Ok(None);
        };
        let doubles = self
            .doubles
            .semantics_for_entry(justification.integer_semantics().record())?
            .ok_or_else(invalid_internal_data)?;
        placement_anchor(justification, doubles).map(Some)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_attribute_definition_placement_anchor_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionPlacementAnchorDirectory, DxfError> {
        DxfBlockAttributeDefinitionPlacementAnchorDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionPlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_placement_anchor_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionPlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_placement_anchor_directory(cancellation)
    }
}

fn placement_anchor(
    justification: DxfBlockAttributeDefinitionJustificationSemantics,
    doubles: DxfBlockAttributeDefinitionDoubleSemantics,
) -> Result<DxfBlockAttributeDefinitionPlacementAnchor, DxfError> {
    if justification.record() != doubles.record() {
        return Err(invalid_internal_data());
    }
    let state = match justification.requires_alignment_point() {
        None => DxfBlockAttributeDefinitionPlacementAnchorState::JustificationUnavailable,
        Some(false) => match doubles.text_start_value() {
            Some(point) => DxfBlockAttributeDefinitionPlacementAnchorState::TextStart { point },
            None => DxfBlockAttributeDefinitionPlacementAnchorState::TextStartUnavailable,
        },
        Some(true) => match doubles.alignment_point_value() {
            Some(point) => {
                DxfBlockAttributeDefinitionPlacementAnchorState::AlignmentPoint { point }
            }
            None => DxfBlockAttributeDefinitionPlacementAnchorState::AlignmentPointUnavailable,
        },
    };
    Ok(DxfBlockAttributeDefinitionPlacementAnchor {
        justification,
        doubles,
        state,
    })
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

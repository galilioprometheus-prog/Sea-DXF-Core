//! Classic ATTRIB placement-anchor selection.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertAttributeDoubleSemanticDirectory, DxfInsertAttributeDoubleSemantics,
    DxfInsertAttributeJustificationDirectory, DxfInsertAttributeJustificationSemantics,
    DxfInsertAttributeValueEntry, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributePlacementAnchorState {
    JustificationUnavailable,
    TextStartUnavailable,
    AlignmentPointUnavailable,
    TextStart { point: [DxfDouble; 3] },
    AlignmentPoint { point: [DxfDouble; 3] },
}

impl DxfInsertAttributePlacementAnchorState {
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
pub struct DxfInsertAttributePlacementAnchor {
    justification: DxfInsertAttributeJustificationSemantics,
    doubles: DxfInsertAttributeDoubleSemantics,
    state: DxfInsertAttributePlacementAnchorState,
}

impl DxfInsertAttributePlacementAnchor {
    #[must_use]
    pub const fn record(self) -> DxfInsertAttributeValueEntry {
        self.doubles.record()
    }

    #[must_use]
    pub const fn justification(self) -> DxfInsertAttributeJustificationSemantics {
        self.justification
    }

    #[must_use]
    pub const fn double_semantics(self) -> DxfInsertAttributeDoubleSemantics {
        self.doubles
    }

    #[must_use]
    pub const fn state(self) -> DxfInsertAttributePlacementAnchorState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributePlacementAnchorDirectory {
    source_id: DxfSourceId,
    justification: DxfInsertAttributeJustificationDirectory,
    doubles: DxfInsertAttributeDoubleSemanticDirectory,
}

impl DxfInsertAttributePlacementAnchorDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let justification = document.insert_attribute_justification_directory(cancellation)?;
        let doubles = document.insert_attribute_double_semantic_directory(cancellation)?;
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
    pub const fn justification_directory(&self) -> &DxfInsertAttributeJustificationDirectory {
        &self.justification
    }

    #[must_use]
    pub const fn double_directory(&self) -> &DxfInsertAttributeDoubleSemanticDirectory {
        &self.doubles
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInsertAttributeValueEntry] {
        self.justification.records()
    }

    pub fn anchor_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributePlacementAnchor>, DxfError> {
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
        record: DxfInsertAttributeValueEntry,
    ) -> Result<Option<DxfInsertAttributePlacementAnchor>, DxfError> {
        let Some(justification) = self.justification.semantics_for_entry(record)? else {
            return Ok(None);
        };
        let doubles = self
            .doubles
            .semantics_for_entry(record)?
            .ok_or_else(invalid_internal_data)?;
        placement_anchor(justification, doubles).map(Some)
    }

    pub fn anchor_for_insert_sequence_attribute(
        &self,
        insert_raw_ordinal: u64,
        sequence_attribute_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributePlacementAnchor>, DxfError> {
        let Some(justification) = self.justification.semantics_for_insert_sequence_attribute(
            insert_raw_ordinal,
            sequence_attribute_ordinal,
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
    pub fn insert_attribute_placement_anchor_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributePlacementAnchorDirectory, DxfError> {
        DxfInsertAttributePlacementAnchorDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributePlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_placement_anchor_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_placement_anchor_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributePlacementAnchorDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_placement_anchor_directory(cancellation)
    }
}

fn placement_anchor(
    justification: DxfInsertAttributeJustificationSemantics,
    doubles: DxfInsertAttributeDoubleSemantics,
) -> Result<DxfInsertAttributePlacementAnchor, DxfError> {
    if justification.record() != doubles.record() {
        return Err(invalid_internal_data());
    }
    let state = match justification.requires_alignment_point() {
        None => DxfInsertAttributePlacementAnchorState::JustificationUnavailable,
        Some(false) => match doubles.text_start_value() {
            Some(point) => DxfInsertAttributePlacementAnchorState::TextStart { point },
            None => DxfInsertAttributePlacementAnchorState::TextStartUnavailable,
        },
        Some(true) => match doubles.alignment_point_value() {
            Some(point) => DxfInsertAttributePlacementAnchorState::AlignmentPoint { point },
            None => DxfInsertAttributePlacementAnchorState::AlignmentPointUnavailable,
        },
    };
    Ok(DxfInsertAttributePlacementAnchor {
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

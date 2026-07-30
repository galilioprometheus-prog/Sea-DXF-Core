//! Typed classic ATTDEF justification classification.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockAttributeDefinitionIntegerSemanticDirectory,
    DxfBlockAttributeDefinitionIntegerSemanticIssue, DxfBlockAttributeDefinitionIntegerSemantics,
    DxfBlockAttributeDefinitionValueEntry, DxfCancellationToken, DxfError, DxfRawDocumentView,
    DxfSemanticValue, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionHorizontalJustification {
    Left,
    Center,
    Right,
    Aligned,
    Middle,
    Fit,
}

impl DxfBlockAttributeDefinitionHorizontalJustification {
    #[must_use]
    pub const fn code(self) -> i16 {
        match self {
            Self::Left => 0,
            Self::Center => 1,
            Self::Right => 2,
            Self::Aligned => 3,
            Self::Middle => 4,
            Self::Fit => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionVerticalJustification {
    Baseline,
    Bottom,
    Middle,
    Top,
}

impl DxfBlockAttributeDefinitionVerticalJustification {
    #[must_use]
    pub const fn code(self) -> i16 {
        match self {
            Self::Baseline => 0,
            Self::Bottom => 1,
            Self::Middle => 2,
            Self::Top => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionJustificationIssue {
    Integer(DxfBlockAttributeDefinitionIntegerSemanticIssue),
    UnsupportedHorizontalCode { code: i16 },
    UnsupportedVerticalCode { code: i16 },
}

pub type DxfBlockAttributeDefinitionHorizontalJustificationSemantic = DxfSemanticValue<
    DxfBlockAttributeDefinitionHorizontalJustification,
    DxfBlockAttributeDefinitionJustificationIssue,
>;
pub type DxfBlockAttributeDefinitionVerticalJustificationSemantic = DxfSemanticValue<
    DxfBlockAttributeDefinitionVerticalJustification,
    DxfBlockAttributeDefinitionJustificationIssue,
>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionJustificationSemantics {
    integers: DxfBlockAttributeDefinitionIntegerSemantics,
    horizontal: DxfBlockAttributeDefinitionHorizontalJustificationSemantic,
    vertical: DxfBlockAttributeDefinitionVerticalJustificationSemantic,
}

impl DxfBlockAttributeDefinitionJustificationSemantics {
    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.integers.record()
    }

    #[must_use]
    pub const fn integer_semantics(self) -> DxfBlockAttributeDefinitionIntegerSemantics {
        self.integers
    }

    #[must_use]
    pub const fn horizontal(&self) -> &DxfBlockAttributeDefinitionHorizontalJustificationSemantic {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &DxfBlockAttributeDefinitionVerticalJustificationSemantic {
        &self.vertical
    }

    #[must_use]
    pub fn requires_alignment_point(&self) -> Option<bool> {
        Some(self.horizontal.value()?.code() != 0 || self.vertical.value()?.code() != 0)
    }

    #[must_use]
    pub fn uses_text_start_point(&self) -> Option<bool> {
        self.requires_alignment_point().map(|required| !required)
    }
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionJustificationDirectory {
    source_id: DxfSourceId,
    integers: DxfBlockAttributeDefinitionIntegerSemanticDirectory,
}

impl DxfBlockAttributeDefinitionJustificationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let integers =
            document.block_attribute_definition_integer_semantic_directory(cancellation)?;
        if integers.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: integers.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            integers,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn integer_directory(&self) -> &DxfBlockAttributeDefinitionIntegerSemanticDirectory {
        &self.integers
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBlockAttributeDefinitionValueEntry] {
        self.integers.records()
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionJustificationSemantics>, DxfError> {
        Ok(self
            .integers
            .semantics_for_raw_record(raw_record_ordinal)?
            .map(justification_semantics))
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfBlockAttributeDefinitionValueEntry,
    ) -> Result<Option<DxfBlockAttributeDefinitionJustificationSemantics>, DxfError> {
        Ok(self
            .integers
            .semantics_for_entry(record)?
            .map(justification_semantics))
    }

    pub fn semantics_for_block_attribute_definition(
        &self,
        block_raw_ordinal: u64,
        attribute_definition_ordinal: u64,
    ) -> Result<Option<DxfBlockAttributeDefinitionJustificationSemantics>, DxfError> {
        Ok(self
            .integers
            .semantics_for_block_attribute_definition(
                block_raw_ordinal,
                attribute_definition_ordinal,
            )?
            .map(justification_semantics))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_attribute_definition_justification_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionJustificationDirectory, DxfError> {
        DxfBlockAttributeDefinitionJustificationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_justification_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionJustificationDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_justification_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_justification_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionJustificationDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .block_attribute_definition_justification_directory(cancellation)
    }
}

fn justification_semantics(
    integers: DxfBlockAttributeDefinitionIntegerSemantics,
) -> DxfBlockAttributeDefinitionJustificationSemantics {
    DxfBlockAttributeDefinitionJustificationSemantics {
        horizontal: project_horizontal(*integers.horizontal_justification()),
        vertical: project_vertical(*integers.vertical_justification()),
        integers,
    }
}

fn project_horizontal(
    source: crate::DxfBlockAttributeDefinitionSemanticInteger,
) -> DxfBlockAttributeDefinitionHorizontalJustificationSemantic {
    project_integer(source, |code| {
        Ok(match code {
            0 => DxfBlockAttributeDefinitionHorizontalJustification::Left,
            1 => DxfBlockAttributeDefinitionHorizontalJustification::Center,
            2 => DxfBlockAttributeDefinitionHorizontalJustification::Right,
            3 => DxfBlockAttributeDefinitionHorizontalJustification::Aligned,
            4 => DxfBlockAttributeDefinitionHorizontalJustification::Middle,
            5 => DxfBlockAttributeDefinitionHorizontalJustification::Fit,
            _ => {
                return Err(
                    DxfBlockAttributeDefinitionJustificationIssue::UnsupportedHorizontalCode {
                        code,
                    },
                );
            }
        })
    })
}

fn project_vertical(
    source: crate::DxfBlockAttributeDefinitionSemanticInteger,
) -> DxfBlockAttributeDefinitionVerticalJustificationSemantic {
    project_integer(source, |code| {
        Ok(match code {
            0 => DxfBlockAttributeDefinitionVerticalJustification::Baseline,
            1 => DxfBlockAttributeDefinitionVerticalJustification::Bottom,
            2 => DxfBlockAttributeDefinitionVerticalJustification::Middle,
            3 => DxfBlockAttributeDefinitionVerticalJustification::Top,
            _ => {
                return Err(
                    DxfBlockAttributeDefinitionJustificationIssue::UnsupportedVerticalCode { code },
                );
            }
        })
    })
}

fn project_integer<T: Copy>(
    source: crate::DxfBlockAttributeDefinitionSemanticInteger,
    classify: impl FnOnce(i16) -> Result<T, DxfBlockAttributeDefinitionJustificationIssue>,
) -> DxfSemanticValue<T, DxfBlockAttributeDefinitionJustificationIssue> {
    match source {
        DxfSemanticValue::Explicit { value, field, raw } => match classify(value) {
            Ok(value) => DxfSemanticValue::explicit(value, field, raw),
            Err(issue) => DxfSemanticValue::invalid(issue, field, Some(raw)),
        },
        DxfSemanticValue::Defaulted { value, field } => match classify(value) {
            Ok(value) => DxfSemanticValue::defaulted(value, field),
            Err(issue) => DxfSemanticValue::invalid(issue, field, None),
        },
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfBlockAttributeDefinitionJustificationIssue::Integer(issue),
            field,
            raw,
        ),
    }
}

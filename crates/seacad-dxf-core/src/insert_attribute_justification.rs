//! Typed classic ATTRIB justification classification.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfInsertAttributeIntegerSemanticDirectory, DxfInsertAttributeIntegerSemanticIssue,
    DxfInsertAttributeIntegerSemantics, DxfInsertAttributeValueEntry, DxfRawDocumentView,
    DxfSemanticValue, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeHorizontalJustification {
    Left,
    Center,
    Right,
    Aligned,
    Middle,
    Fit,
}

impl DxfInsertAttributeHorizontalJustification {
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
pub enum DxfInsertAttributeVerticalJustification {
    Baseline,
    Bottom,
    Middle,
    Top,
}

impl DxfInsertAttributeVerticalJustification {
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
pub enum DxfInsertAttributeJustificationIssue {
    Integer(DxfInsertAttributeIntegerSemanticIssue),
    UnsupportedHorizontalCode { code: i16 },
    UnsupportedVerticalCode { code: i16 },
}

pub type DxfInsertAttributeHorizontalJustificationSemantic = DxfSemanticValue<
    DxfInsertAttributeHorizontalJustification,
    DxfInsertAttributeJustificationIssue,
>;
pub type DxfInsertAttributeVerticalJustificationSemantic =
    DxfSemanticValue<DxfInsertAttributeVerticalJustification, DxfInsertAttributeJustificationIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeJustificationSemantics {
    integers: DxfInsertAttributeIntegerSemantics,
    horizontal: DxfInsertAttributeHorizontalJustificationSemantic,
    vertical: DxfInsertAttributeVerticalJustificationSemantic,
}

impl DxfInsertAttributeJustificationSemantics {
    #[must_use]
    pub const fn record(self) -> DxfInsertAttributeValueEntry {
        self.integers.record()
    }

    #[must_use]
    pub const fn integer_semantics(self) -> DxfInsertAttributeIntegerSemantics {
        self.integers
    }

    #[must_use]
    pub const fn horizontal(&self) -> &DxfInsertAttributeHorizontalJustificationSemantic {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &DxfInsertAttributeVerticalJustificationSemantic {
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
pub struct DxfInsertAttributeJustificationDirectory {
    source_id: DxfSourceId,
    integers: DxfInsertAttributeIntegerSemanticDirectory,
}

impl DxfInsertAttributeJustificationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let integers = document.insert_attribute_integer_semantic_directory(cancellation)?;
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
    pub const fn integer_directory(&self) -> &DxfInsertAttributeIntegerSemanticDirectory {
        &self.integers
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInsertAttributeValueEntry] {
        self.integers.records()
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributeJustificationSemantics>, DxfError> {
        Ok(self
            .integers
            .semantics_for_raw_record(raw_record_ordinal)?
            .map(justification_semantics))
    }

    pub fn semantics_for_entry(
        &self,
        record: DxfInsertAttributeValueEntry,
    ) -> Result<Option<DxfInsertAttributeJustificationSemantics>, DxfError> {
        Ok(self
            .integers
            .semantics_for_entry(record)?
            .map(justification_semantics))
    }

    pub fn semantics_for_insert_sequence_attribute(
        &self,
        insert_raw_ordinal: u64,
        sequence_attribute_ordinal: u64,
    ) -> Result<Option<DxfInsertAttributeJustificationSemantics>, DxfError> {
        Ok(self
            .integers
            .semantics_for_insert_sequence_attribute(
                insert_raw_ordinal,
                sequence_attribute_ordinal,
            )?
            .map(justification_semantics))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_attribute_justification_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeJustificationDirectory, DxfError> {
        DxfInsertAttributeJustificationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_justification_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeJustificationDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_justification_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_justification_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeJustificationDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_justification_directory(cancellation)
    }
}

fn justification_semantics(
    integers: DxfInsertAttributeIntegerSemantics,
) -> DxfInsertAttributeJustificationSemantics {
    DxfInsertAttributeJustificationSemantics {
        horizontal: project_horizontal(*integers.horizontal_justification()),
        vertical: project_vertical(*integers.vertical_justification()),
        integers,
    }
}

fn project_horizontal(
    source: crate::DxfInsertAttributeSemanticInteger,
) -> DxfInsertAttributeHorizontalJustificationSemantic {
    project_integer(source, |code| {
        Ok(match code {
            0 => DxfInsertAttributeHorizontalJustification::Left,
            1 => DxfInsertAttributeHorizontalJustification::Center,
            2 => DxfInsertAttributeHorizontalJustification::Right,
            3 => DxfInsertAttributeHorizontalJustification::Aligned,
            4 => DxfInsertAttributeHorizontalJustification::Middle,
            5 => DxfInsertAttributeHorizontalJustification::Fit,
            _ => {
                return Err(
                    DxfInsertAttributeJustificationIssue::UnsupportedHorizontalCode { code },
                );
            }
        })
    })
}

fn project_vertical(
    source: crate::DxfInsertAttributeSemanticInteger,
) -> DxfInsertAttributeVerticalJustificationSemantic {
    project_integer(source, |code| {
        Ok(match code {
            0 => DxfInsertAttributeVerticalJustification::Baseline,
            1 => DxfInsertAttributeVerticalJustification::Bottom,
            2 => DxfInsertAttributeVerticalJustification::Middle,
            3 => DxfInsertAttributeVerticalJustification::Top,
            _ => {
                return Err(DxfInsertAttributeJustificationIssue::UnsupportedVerticalCode { code });
            }
        })
    })
}

fn project_integer<T: Copy>(
    source: crate::DxfInsertAttributeSemanticInteger,
    classify: impl FnOnce(i16) -> Result<T, DxfInsertAttributeJustificationIssue>,
) -> DxfSemanticValue<T, DxfInsertAttributeJustificationIssue> {
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
            DxfInsertAttributeJustificationIssue::Integer(issue),
            field,
            raw,
        ),
    }
}

//! Typed MTEXT attachment, direction, and line-spacing codes.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfMTextNumericSemantics, DxfMTextToleranceScalarDirectory, DxfRawDocumentView,
    DxfSemanticValue, DxfSourceId, DxfTextSymbolInt16Value, DxfTextSymbolRecordEntry,
    DxfTextSymbolScalarIssue,
};

/// Autodesk MTEXT attachment-point code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextAttachment {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl DxfMTextAttachment {
    #[must_use]
    pub const fn code(self) -> i16 {
        match self {
            Self::TopLeft => 1,
            Self::TopCenter => 2,
            Self::TopRight => 3,
            Self::MiddleLeft => 4,
            Self::MiddleCenter => 5,
            Self::MiddleRight => 6,
            Self::BottomLeft => 7,
            Self::BottomCenter => 8,
            Self::BottomRight => 9,
        }
    }
}

/// Autodesk MTEXT drawing-direction code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextDrawingDirection {
    LeftToRight,
    TopToBottom,
    ByStyle,
}

impl DxfMTextDrawingDirection {
    #[must_use]
    pub const fn code(self) -> i16 {
        match self {
            Self::LeftToRight => 1,
            Self::TopToBottom => 3,
            Self::ByStyle => 5,
        }
    }
}

/// Autodesk MTEXT line-spacing style code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextLineSpacingStyle {
    AtLeast,
    Exact,
}

impl DxfMTextLineSpacingStyle {
    #[must_use]
    pub const fn code(self) -> i16 {
        match self {
            Self::AtLeast => 1,
            Self::Exact => 2,
        }
    }
}

/// Why an MTEXT layout code is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextLayoutIssue {
    Scalar(DxfTextSymbolScalarIssue),
    UnsupportedAttachmentCode { code: i16 },
    UnsupportedDrawingDirectionCode { code: i16 },
    UnsupportedLineSpacingStyleCode { code: i16 },
}

pub type DxfMTextAttachmentSemantic = DxfSemanticValue<DxfMTextAttachment, DxfMTextLayoutIssue>;
pub type DxfMTextDrawingDirectionSemantic =
    DxfSemanticValue<DxfMTextDrawingDirection, DxfMTextLayoutIssue>;
pub type DxfMTextLineSpacingStyleSemantic =
    DxfSemanticValue<DxfMTextLineSpacingStyle, DxfMTextLayoutIssue>;

/// MTEXT numeric evidence projected into documented layout codes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextLayoutSemantics {
    numeric: DxfMTextNumericSemantics,
    attachment: DxfMTextAttachmentSemantic,
    drawing_direction: DxfMTextDrawingDirectionSemantic,
    line_spacing_style: DxfMTextLineSpacingStyleSemantic,
}

impl DxfMTextLayoutSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.numeric.record()
    }

    #[must_use]
    pub const fn numeric_semantics(self) -> DxfMTextNumericSemantics {
        self.numeric
    }

    #[must_use]
    pub const fn attachment(&self) -> &DxfMTextAttachmentSemantic {
        &self.attachment
    }

    #[must_use]
    pub const fn drawing_direction(&self) -> &DxfMTextDrawingDirectionSemantic {
        &self.drawing_direction
    }

    #[must_use]
    pub const fn line_spacing_style(&self) -> &DxfMTextLineSpacingStyleSemantic {
        &self.line_spacing_style
    }
}

/// Lazy MTEXT layout projection retaining the complete numeric directory.
#[derive(Debug)]
pub struct DxfMTextLayoutDirectory {
    source_id: DxfSourceId,
    scalars: DxfMTextToleranceScalarDirectory,
}

impl DxfMTextLayoutDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let scalars = document.mtext_tolerance_scalar_directory(cancellation)?;
        if scalars.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: scalars.source_id(),
            });
        }
        Ok(Self {
            source_id: document.source_id(),
            scalars,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn scalar_directory(&self) -> &DxfMTextToleranceScalarDirectory {
        &self.scalars
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.scalars.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfMTextLayoutSemantics>, DxfError> {
        Ok(self
            .scalars
            .mtext_semantics_for_record(record)?
            .map(layout_semantics))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfMTextLayoutSemantics>, DxfError> {
        Ok(self
            .scalars
            .mtext_semantics_for_raw_record(raw_record_ordinal)?
            .map(layout_semantics))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_layout_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextLayoutDirectory, DxfError> {
        DxfMTextLayoutDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_layout_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextLayoutDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_layout_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_layout_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextLayoutDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_layout_directory(cancellation)
    }
}

fn layout_semantics(numeric: DxfMTextNumericSemantics) -> DxfMTextLayoutSemantics {
    DxfMTextLayoutSemantics {
        attachment: project_integer(*numeric.attachment(), |code| {
            Ok(match code {
                1 => DxfMTextAttachment::TopLeft,
                2 => DxfMTextAttachment::TopCenter,
                3 => DxfMTextAttachment::TopRight,
                4 => DxfMTextAttachment::MiddleLeft,
                5 => DxfMTextAttachment::MiddleCenter,
                6 => DxfMTextAttachment::MiddleRight,
                7 => DxfMTextAttachment::BottomLeft,
                8 => DxfMTextAttachment::BottomCenter,
                9 => DxfMTextAttachment::BottomRight,
                _ => return Err(DxfMTextLayoutIssue::UnsupportedAttachmentCode { code }),
            })
        }),
        drawing_direction: project_integer(*numeric.drawing_direction(), |code| {
            Ok(match code {
                1 => DxfMTextDrawingDirection::LeftToRight,
                3 => DxfMTextDrawingDirection::TopToBottom,
                5 => DxfMTextDrawingDirection::ByStyle,
                _ => {
                    return Err(DxfMTextLayoutIssue::UnsupportedDrawingDirectionCode { code });
                }
            })
        }),
        line_spacing_style: project_integer(*numeric.line_spacing_style(), |code| {
            Ok(match code {
                1 => DxfMTextLineSpacingStyle::AtLeast,
                2 => DxfMTextLineSpacingStyle::Exact,
                _ => {
                    return Err(DxfMTextLayoutIssue::UnsupportedLineSpacingStyleCode { code });
                }
            })
        }),
        numeric,
    }
}

fn project_integer<T: Copy>(
    source: DxfTextSymbolInt16Value,
    classify: impl FnOnce(i16) -> Result<T, DxfMTextLayoutIssue>,
) -> DxfSemanticValue<T, DxfMTextLayoutIssue> {
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
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(DxfMTextLayoutIssue::Scalar(issue), field, raw)
        }
    }
}

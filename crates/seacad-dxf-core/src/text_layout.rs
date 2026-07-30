//! Typed TEXT generation flags and justification semantics.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfRawDocumentView,
    DxfSemanticValue, DxfSourceId, DxfTextNumericSemantics, DxfTextShapeInt16Value,
    DxfTextShapeScalarDirectory, DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue,
};

/// Exact TEXT generation-flag bits with documented helpers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextGenerationFlags {
    bits: u16,
}

impl DxfTextGenerationFlags {
    pub const BACKWARD_BIT: u16 = 2;
    pub const UPSIDE_DOWN_BIT: u16 = 4;
    pub const KNOWN_BITS: u16 = Self::BACKWARD_BIT | Self::UPSIDE_DOWN_BIT;

    #[must_use]
    pub const fn from_source_value(value: i16) -> Self {
        Self { bits: value as u16 }
    }

    #[must_use]
    pub const fn source_value(self) -> i16 {
        self.bits as i16
    }

    #[must_use]
    pub const fn bits(self) -> u16 {
        self.bits
    }

    #[must_use]
    pub const fn is_backward(self) -> bool {
        self.bits & Self::BACKWARD_BIT != 0
    }

    #[must_use]
    pub const fn is_upside_down(self) -> bool {
        self.bits & Self::UPSIDE_DOWN_BIT != 0
    }

    #[must_use]
    pub const fn unknown_bits(self) -> u16 {
        self.bits & !Self::KNOWN_BITS
    }
}

/// Autodesk TEXT horizontal justification code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextHorizontalJustification {
    Left,
    Center,
    Right,
    Aligned,
    Middle,
    Fit,
}

impl DxfTextHorizontalJustification {
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

/// Autodesk TEXT vertical justification code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextVerticalJustification {
    Baseline,
    Bottom,
    Middle,
    Top,
}

impl DxfTextVerticalJustification {
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

/// Why a TEXT layout value is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextLayoutIssue {
    Scalar(DxfTextSymbolScalarIssue),
    UnsupportedHorizontalCode { code: i16 },
    UnsupportedVerticalCode { code: i16 },
}

pub type DxfTextGenerationFlagsSemantic =
    DxfSemanticValue<DxfTextGenerationFlags, DxfTextLayoutIssue>;
pub type DxfTextHorizontalJustificationSemantic =
    DxfSemanticValue<DxfTextHorizontalJustification, DxfTextLayoutIssue>;
pub type DxfTextVerticalJustificationSemantic =
    DxfSemanticValue<DxfTextVerticalJustification, DxfTextLayoutIssue>;

/// TEXT numeric evidence projected into exact layout codes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextLayoutSemantics {
    numeric: DxfTextNumericSemantics,
    generation_flags: DxfTextGenerationFlagsSemantic,
    horizontal: DxfTextHorizontalJustificationSemantic,
    vertical: DxfTextVerticalJustificationSemantic,
}

impl DxfTextLayoutSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.numeric.record()
    }

    #[must_use]
    pub const fn numeric_semantics(self) -> DxfTextNumericSemantics {
        self.numeric
    }

    #[must_use]
    pub const fn generation_flags(&self) -> &DxfTextGenerationFlagsSemantic {
        &self.generation_flags
    }

    #[must_use]
    pub const fn horizontal(&self) -> &DxfTextHorizontalJustificationSemantic {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &DxfTextVerticalJustificationSemantic {
        &self.vertical
    }

    /// Whether Autodesk's second alignment point controls placement.
    #[must_use]
    pub fn requires_second_alignment_point(&self) -> Option<bool> {
        Some(self.horizontal.value()?.code() != 0 || self.vertical.value()?.code() != 0)
    }

    #[must_use]
    pub fn uses_first_alignment_point(&self) -> Option<bool> {
        self.requires_second_alignment_point()
            .map(|required| !required)
    }
}

/// Lazy TEXT layout projection retaining the complete numeric directory.
#[derive(Debug)]
pub struct DxfTextLayoutDirectory {
    source_id: DxfSourceId,
    scalars: DxfTextShapeScalarDirectory,
}

impl DxfTextLayoutDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let scalars = document.text_shape_scalar_directory(cancellation)?;
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
    pub const fn scalar_directory(&self) -> &DxfTextShapeScalarDirectory {
        &self.scalars
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.scalars.records()
    }

    pub fn semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfTextLayoutSemantics>, DxfError> {
        Ok(self
            .scalars
            .text_semantics_for_record(record)?
            .map(layout_semantics))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfTextLayoutSemantics>, DxfError> {
        Ok(self
            .scalars
            .text_semantics_for_raw_record(raw_record_ordinal)?
            .map(layout_semantics))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn text_layout_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextLayoutDirectory, DxfError> {
        DxfTextLayoutDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_layout_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextLayoutDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_layout_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_layout_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextLayoutDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_layout_directory(cancellation)
    }
}

fn layout_semantics(numeric: DxfTextNumericSemantics) -> DxfTextLayoutSemantics {
    DxfTextLayoutSemantics {
        generation_flags: project_integer(*numeric.generation_flags(), |value| {
            Ok(DxfTextGenerationFlags::from_source_value(value))
        }),
        horizontal: project_integer(*numeric.horizontal_justification(), |code| {
            Ok(match code {
                0 => DxfTextHorizontalJustification::Left,
                1 => DxfTextHorizontalJustification::Center,
                2 => DxfTextHorizontalJustification::Right,
                3 => DxfTextHorizontalJustification::Aligned,
                4 => DxfTextHorizontalJustification::Middle,
                5 => DxfTextHorizontalJustification::Fit,
                _ => return Err(DxfTextLayoutIssue::UnsupportedHorizontalCode { code }),
            })
        }),
        vertical: project_integer(*numeric.vertical_justification(), |code| {
            Ok(match code {
                0 => DxfTextVerticalJustification::Baseline,
                1 => DxfTextVerticalJustification::Bottom,
                2 => DxfTextVerticalJustification::Middle,
                3 => DxfTextVerticalJustification::Top,
                _ => return Err(DxfTextLayoutIssue::UnsupportedVerticalCode { code }),
            })
        }),
        numeric,
    }
}

fn project_integer<T: Copy>(
    source: DxfTextShapeInt16Value,
    classify: impl FnOnce(i16) -> Result<T, DxfTextLayoutIssue>,
) -> DxfSemanticValue<T, DxfTextLayoutIssue> {
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
            DxfSemanticValue::invalid(DxfTextLayoutIssue::Scalar(issue), field, raw)
        }
    }
}

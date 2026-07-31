//! Typed MTEXT numeric domains documented independently of layout codes.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfMTextNumericSemantics, DxfMTextToleranceScalarDirectory, DxfRawDocumentView,
    DxfSemanticValue, DxfSourceId, DxfTextSymbolDoubleValue, DxfTextSymbolInt32Value,
    DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue,
};

/// Autodesk MTEXT line-spacing factor in the inclusive `0.25..=4.00` domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextLineSpacingFactor(DxfDouble);

impl DxfMTextLineSpacingFactor {
    pub const MINIMUM: f64 = 0.25;
    pub const MAXIMUM: f64 = 4.0;

    #[must_use]
    pub const fn raw(self) -> DxfDouble {
        self.0
    }

    #[must_use]
    pub fn to_f64(self) -> f64 {
        self.0.to_f64()
    }
}

/// Autodesk MTEXT background-fill setting.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextBackgroundFillSetting {
    Off,
    FillColor,
    DrawingWindowColor,
}

impl DxfMTextBackgroundFillSetting {
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::FillColor => 1,
            Self::DrawingWindowColor => 2,
        }
    }
}

/// Documented relationship between MTEXT actual and reference widths.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DxfMTextActualWidthRelation {
    WithinReference,
}

/// Why a documented MTEXT numeric domain is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextNumericDomainIssue {
    Scalar(DxfTextSymbolScalarIssue),
    LineSpacingFactorOutOfRange {
        value: DxfDouble,
    },
    UnsupportedBackgroundFillSetting {
        code: i32,
    },
    ActualWidthExceedsReference {
        actual_width: DxfDouble,
        reference_width: DxfDouble,
    },
}

pub type DxfMTextLineSpacingFactorSemantic =
    DxfSemanticValue<DxfMTextLineSpacingFactor, DxfMTextNumericDomainIssue>;
pub type DxfMTextBackgroundFillSettingSemantic =
    DxfSemanticValue<DxfMTextBackgroundFillSetting, DxfMTextNumericDomainIssue>;
pub type DxfMTextActualWidthRelationSemantic =
    DxfSemanticValue<DxfMTextActualWidthRelation, DxfMTextNumericDomainIssue>;

/// MTEXT scalars projected into independently documented numeric domains.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextNumericDomainSemantics {
    numeric: DxfMTextNumericSemantics,
    line_spacing_factor: DxfMTextLineSpacingFactorSemantic,
    background_fill_setting: DxfMTextBackgroundFillSettingSemantic,
    actual_width_relation: DxfMTextActualWidthRelationSemantic,
}

impl DxfMTextNumericDomainSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.numeric.record()
    }

    #[must_use]
    pub const fn numeric_semantics(self) -> DxfMTextNumericSemantics {
        self.numeric
    }

    #[must_use]
    pub const fn line_spacing_factor(&self) -> &DxfMTextLineSpacingFactorSemantic {
        &self.line_spacing_factor
    }

    #[must_use]
    pub const fn background_fill_setting(&self) -> &DxfMTextBackgroundFillSettingSemantic {
        &self.background_fill_setting
    }

    #[must_use]
    pub const fn actual_width_relation(&self) -> &DxfMTextActualWidthRelationSemantic {
        &self.actual_width_relation
    }
}

/// Lazy MTEXT numeric-domain projection retaining the complete scalar directory.
#[derive(Debug)]
pub struct DxfMTextNumericDomainDirectory {
    source_id: DxfSourceId,
    scalars: DxfMTextToleranceScalarDirectory,
}

impl DxfMTextNumericDomainDirectory {
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
    ) -> Result<Option<DxfMTextNumericDomainSemantics>, DxfError> {
        Ok(self
            .scalars
            .mtext_semantics_for_record(record)?
            .map(numeric_domain_semantics))
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfMTextNumericDomainSemantics>, DxfError> {
        Ok(self
            .scalars
            .mtext_semantics_for_raw_record(raw_record_ordinal)?
            .map(numeric_domain_semantics))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_numeric_domain_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextNumericDomainDirectory, DxfError> {
        DxfMTextNumericDomainDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_numeric_domain_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextNumericDomainDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_numeric_domain_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_numeric_domain_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextNumericDomainDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_numeric_domain_directory(cancellation)
    }
}

fn numeric_domain_semantics(numeric: DxfMTextNumericSemantics) -> DxfMTextNumericDomainSemantics {
    DxfMTextNumericDomainSemantics {
        line_spacing_factor: project_line_spacing_factor(*numeric.line_spacing_factor()),
        background_fill_setting: project_background_fill_setting(*numeric.background_fill()),
        actual_width_relation: project_actual_width_relation(
            *numeric.actual_width(),
            *numeric.reference_width(),
        ),
        numeric,
    }
}

fn project_line_spacing_factor(
    source: DxfTextSymbolDoubleValue,
) -> DxfMTextLineSpacingFactorSemantic {
    project_double(source, |value| {
        let number = value.to_f64();
        if (DxfMTextLineSpacingFactor::MINIMUM..=DxfMTextLineSpacingFactor::MAXIMUM)
            .contains(&number)
        {
            Ok(DxfMTextLineSpacingFactor(value))
        } else {
            Err(DxfMTextNumericDomainIssue::LineSpacingFactorOutOfRange { value })
        }
    })
}

fn project_background_fill_setting(
    source: DxfTextSymbolInt32Value,
) -> DxfMTextBackgroundFillSettingSemantic {
    project_integer(source, |code| {
        Ok(match code {
            0 => DxfMTextBackgroundFillSetting::Off,
            1 => DxfMTextBackgroundFillSetting::FillColor,
            2 => DxfMTextBackgroundFillSetting::DrawingWindowColor,
            _ => {
                return Err(DxfMTextNumericDomainIssue::UnsupportedBackgroundFillSetting { code });
            }
        })
    })
}

fn project_actual_width_relation(
    actual_width: DxfTextSymbolDoubleValue,
    reference_width: DxfTextSymbolDoubleValue,
) -> DxfMTextActualWidthRelationSemantic {
    match actual_width {
        DxfSemanticValue::Explicit {
            value: actual,
            field,
            raw,
        } => match usable_reference_width(reference_width) {
            Ok(Some(reference)) if actual.to_f64() <= reference.to_f64() => {
                DxfSemanticValue::explicit(DxfMTextActualWidthRelation::WithinReference, field, raw)
            }
            Ok(Some(reference)) => DxfSemanticValue::invalid(
                DxfMTextNumericDomainIssue::ActualWidthExceedsReference {
                    actual_width: actual,
                    reference_width: reference,
                },
                field,
                Some(raw),
            ),
            Ok(None) => DxfSemanticValue::absent(field),
            Err((issue, reference_field, reference_raw)) => DxfSemanticValue::invalid(
                DxfMTextNumericDomainIssue::Scalar(issue),
                reference_field,
                reference_raw,
            ),
        },
        DxfSemanticValue::Defaulted {
            value: actual,
            field,
        } => match usable_reference_width(reference_width) {
            Ok(Some(reference)) if actual.to_f64() <= reference.to_f64() => {
                DxfSemanticValue::defaulted(DxfMTextActualWidthRelation::WithinReference, field)
            }
            Ok(Some(reference)) => DxfSemanticValue::invalid(
                DxfMTextNumericDomainIssue::ActualWidthExceedsReference {
                    actual_width: actual,
                    reference_width: reference,
                },
                field,
                None,
            ),
            Ok(None) => DxfSemanticValue::absent(field),
            Err((issue, reference_field, reference_raw)) => DxfSemanticValue::invalid(
                DxfMTextNumericDomainIssue::Scalar(issue),
                reference_field,
                reference_raw,
            ),
        },
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(DxfMTextNumericDomainIssue::Scalar(issue), field, raw)
        }
    }
}

fn usable_reference_width(
    reference_width: DxfTextSymbolDoubleValue,
) -> Result<
    Option<DxfDouble>,
    (
        DxfTextSymbolScalarIssue,
        crate::DxfSemanticFieldProvenance,
        Option<crate::DxfRawValueProvenance>,
    ),
> {
    match reference_width {
        DxfSemanticValue::Explicit { value, .. } | DxfSemanticValue::Defaulted { value, .. } => {
            Ok(Some(value))
        }
        DxfSemanticValue::Absent { .. } => Ok(None),
        DxfSemanticValue::Invalid { issue, field, raw } => Err((issue, field, raw)),
    }
}

fn project_double<T: Copy>(
    source: DxfTextSymbolDoubleValue,
    classify: impl FnOnce(DxfDouble) -> Result<T, DxfMTextNumericDomainIssue>,
) -> DxfSemanticValue<T, DxfMTextNumericDomainIssue> {
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
            DxfSemanticValue::invalid(DxfMTextNumericDomainIssue::Scalar(issue), field, raw)
        }
    }
}

fn project_integer<T: Copy>(
    source: DxfTextSymbolInt32Value,
    classify: impl FnOnce(i32) -> Result<T, DxfMTextNumericDomainIssue>,
) -> DxfSemanticValue<T, DxfMTextNumericDomainIssue> {
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
            DxfSemanticValue::invalid(DxfMTextNumericDomainIssue::Scalar(issue), field, raw)
        }
    }
}

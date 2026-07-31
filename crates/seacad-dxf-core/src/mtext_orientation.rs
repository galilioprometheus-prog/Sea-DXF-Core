//! Source-ordered MTEXT rotation and X-axis input precedence.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfMTextNumericSemantics, DxfMTextToleranceScalarDirectory, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    DxfTextSymbolDoubleValue, DxfTextSymbolRecordEntry, DxfTextSymbolScalarIssue,
    DxfTextSymbolValue, DxfTextSymbolValueRole,
    text_symbol_scalar_value::{DxfScalarRule, semantic_double},
};

const ORIENTATION_NAMESPACE: &str = "text_symbol.mtext.orientation";

/// The last unambiguous MTEXT orientation input in source order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextOrientationInput {
    Rotation,
    XAxis,
}

/// Why an MTEXT orientation input is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextOrientationIssue {
    Scalar(DxfTextSymbolScalarIssue),
    AmbiguousGroup50Role { occurrence_count: u32 },
}

pub type DxfMTextRotationSemantic = DxfSemanticValue<DxfDouble, DxfMTextOrientationIssue>;
pub type DxfMTextOrientationInputSemantic =
    DxfSemanticValue<DxfMTextOrientationInput, DxfMTextOrientationIssue>;

/// MTEXT numeric evidence plus source-ordered rotation/X-axis precedence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextOrientationSemantics {
    numeric: DxfMTextNumericSemantics,
    rotation: DxfMTextRotationSemantic,
    effective_input: DxfMTextOrientationInputSemantic,
}

impl DxfMTextOrientationSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.numeric.record()
    }

    #[must_use]
    pub const fn numeric_semantics(self) -> DxfMTextNumericSemantics {
        self.numeric
    }

    #[must_use]
    pub const fn rotation(&self) -> &DxfMTextRotationSemantic {
        &self.rotation
    }

    #[must_use]
    pub const fn effective_input(&self) -> &DxfMTextOrientationInputSemantic {
        &self.effective_input
    }
}

/// Lazy MTEXT orientation projection retaining the complete scalar directory.
#[derive(Debug)]
pub struct DxfMTextOrientationDirectory {
    source_id: DxfSourceId,
    scalars: DxfMTextToleranceScalarDirectory,
}

impl DxfMTextOrientationDirectory {
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
    ) -> Result<Option<DxfMTextOrientationSemantics>, DxfError> {
        let Some(numeric) = self.scalars.mtext_semantics_for_record(record)? else {
            return Ok(None);
        };
        orientation_semantics(&self.scalars, numeric).map(Some)
    }

    pub fn semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfMTextOrientationSemantics>, DxfError> {
        let Some(record) = self
            .scalars
            .records()
            .iter()
            .copied()
            .find(|entry| entry.record().ordinal() == raw_record_ordinal)
        else {
            return Ok(None);
        };
        self.semantics_for_record(record)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_orientation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextOrientationDirectory, DxfError> {
        DxfMTextOrientationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_orientation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextOrientationDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_orientation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_orientation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextOrientationDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_orientation_directory(cancellation)
    }
}

fn orientation_semantics(
    scalars: &DxfMTextToleranceScalarDirectory,
    numeric: DxfMTextNumericSemantics,
) -> Result<DxfMTextOrientationSemantics, DxfError> {
    let cards = scalars.card_directory();
    let record = numeric.record();
    let values = cards
        .evidence_directory()
        .values_for_raw_record(record.record().ordinal())
        .ok_or_else(invalid_internal_data)?;
    let evidence = orientation_evidence(values)?;
    let scalar_rotation = semantic_double(
        cards,
        record,
        DxfTextSymbolValueRole::RotationOrColumnHeight,
        ORIENTATION_NAMESPACE,
        "rotation",
        DxfScalarRule::Optional,
    )?;
    let rotation = if evidence.columns_present && evidence.group_50_count != 0 {
        DxfSemanticValue::invalid(
            DxfMTextOrientationIssue::AmbiguousGroup50Role {
                occurrence_count: evidence.group_50_count,
            },
            DxfSemanticFieldProvenance::new(cards.source_id(), ORIENTATION_NAMESPACE, "rotation"),
            evidence.first_group_50,
        )
    } else {
        project_rotation(scalar_rotation)
    };
    let effective_input = effective_input(cards.source_id(), evidence, rotation);
    Ok(DxfMTextOrientationSemantics {
        numeric,
        rotation,
        effective_input,
    })
}

#[derive(Clone, Copy)]
struct OrientationEvidence {
    columns_present: bool,
    group_50_count: u32,
    first_group_50: Option<DxfRawValueProvenance>,
    last_group_50: Option<DxfRawValueProvenance>,
    last_x_axis: Option<DxfRawValueProvenance>,
}

fn orientation_evidence(values: &[DxfTextSymbolValue]) -> Result<OrientationEvidence, DxfError> {
    let mut result = OrientationEvidence {
        columns_present: false,
        group_50_count: 0,
        first_group_50: None,
        last_group_50: None,
        last_x_axis: None,
    };
    for value in values.iter().copied() {
        if is_column_role(value.role()) {
            result.columns_present = true;
        }
        if value.role() == DxfTextSymbolValueRole::RotationOrColumnHeight {
            let raw = raw_provenance(value)?;
            result.group_50_count = result
                .group_50_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            result.first_group_50 = result.first_group_50.or(Some(raw));
            result.last_group_50 = Some(raw);
        } else if matches!(
            value.role(),
            DxfTextSymbolValueRole::XAxisX
                | DxfTextSymbolValueRole::XAxisY
                | DxfTextSymbolValueRole::XAxisZ
        ) {
            result.last_x_axis = Some(raw_provenance(value)?);
        }
    }
    Ok(result)
}

fn is_column_role(role: DxfTextSymbolValueRole) -> bool {
    matches!(
        role,
        DxfTextSymbolValueRole::ColumnType
            | DxfTextSymbolValueRole::ColumnCount
            | DxfTextSymbolValueRole::ColumnFlowReversed
            | DxfTextSymbolValueRole::ColumnAutoHeight
            | DxfTextSymbolValueRole::ColumnWidth
            | DxfTextSymbolValueRole::ColumnGutter
    )
}

fn effective_input(
    source_id: DxfSourceId,
    evidence: OrientationEvidence,
    rotation: DxfMTextRotationSemantic,
) -> DxfMTextOrientationInputSemantic {
    let field =
        DxfSemanticFieldProvenance::new(source_id, ORIENTATION_NAMESPACE, "effective_input");
    if evidence.columns_present && evidence.group_50_count != 0 {
        return DxfSemanticValue::invalid(
            DxfMTextOrientationIssue::AmbiguousGroup50Role {
                occurrence_count: evidence.group_50_count,
            },
            field,
            evidence.first_group_50,
        );
    }
    let rotation_is_last = match (evidence.last_group_50, evidence.last_x_axis) {
        (Some(rotation), Some(axis)) => rotation.group_occurrence() > axis.group_occurrence(),
        (Some(_), None) => true,
        _ => false,
    };
    if rotation_is_last {
        return match rotation {
            DxfSemanticValue::Explicit { raw, .. } => {
                DxfSemanticValue::explicit(DxfMTextOrientationInput::Rotation, field, raw)
            }
            DxfSemanticValue::Defaulted { .. } => {
                DxfSemanticValue::defaulted(DxfMTextOrientationInput::Rotation, field)
            }
            DxfSemanticValue::Absent { .. } => DxfSemanticValue::absent(field),
            DxfSemanticValue::Invalid { issue, raw, .. } => {
                DxfSemanticValue::invalid(issue, field, raw)
            }
        };
    }
    match evidence.last_x_axis {
        Some(raw) => DxfSemanticValue::explicit(DxfMTextOrientationInput::XAxis, field, raw),
        None => DxfSemanticValue::absent(field),
    }
}

fn project_rotation(source: DxfTextSymbolDoubleValue) -> DxfMTextRotationSemantic {
    match source {
        DxfSemanticValue::Explicit { value, field, raw } => {
            DxfSemanticValue::explicit(value, field, raw)
        }
        DxfSemanticValue::Defaulted { value, field } => DxfSemanticValue::defaulted(value, field),
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(DxfMTextOrientationIssue::Scalar(issue), field, raw)
        }
    }
}

fn raw_provenance(value: DxfTextSymbolValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

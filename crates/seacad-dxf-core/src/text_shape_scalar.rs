//! Lazy typed numeric semantics for Autodesk TEXT and SHAPE entities.

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfRawDocumentView, DxfSemanticValue, DxfSourceId,
    DxfTextSymbolCardDirectory, DxfTextSymbolKind, DxfTextSymbolRecordEntry,
};

#[path = "text_shape_scalar_build.rs"]
mod build;
use build::{shape_semantics, text_semantics};

const TEXT_NAMESPACE: &str = "text_symbol.text";
const SHAPE_NAMESPACE: &str = "text_symbol.shape";
const ZERO: DxfDouble = DxfDouble::from_bits(0.0_f64.to_bits());
const ONE: DxfDouble = DxfDouble::from_bits(1.0_f64.to_bits());

/// Why one reviewed TEXT or SHAPE scalar has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolScalarIssue {
    MissingRequiredValue,
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    MultipleValues { occurrence_count: u32 },
}

pub type DxfTextSymbolDoubleValue = DxfSemanticValue<DxfDouble, DxfTextSymbolScalarIssue>;
pub type DxfTextSymbolInt16Value = DxfSemanticValue<i16, DxfTextSymbolScalarIssue>;
pub type DxfTextSymbolInt32Value = DxfSemanticValue<i32, DxfTextSymbolScalarIssue>;
pub type DxfTextShapeScalarIssue = DxfTextSymbolScalarIssue;
pub type DxfTextShapeDoubleValue = DxfTextSymbolDoubleValue;
pub type DxfTextShapeInt16Value = DxfTextSymbolInt16Value;

/// Selected and documented-defaulted numeric values for one TEXT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextNumericSemantics {
    record: DxfTextSymbolRecordEntry,
    first_alignment: [DxfTextShapeDoubleValue; 3],
    text_height: DxfTextShapeDoubleValue,
    thickness: DxfTextShapeDoubleValue,
    rotation: DxfTextShapeDoubleValue,
    width_factor: DxfTextShapeDoubleValue,
    oblique_angle: DxfTextShapeDoubleValue,
    generation_flags: DxfTextShapeInt16Value,
    horizontal_justification: DxfTextShapeInt16Value,
    second_alignment: [DxfTextShapeDoubleValue; 3],
    extrusion: [DxfTextShapeDoubleValue; 3],
    vertical_justification: DxfTextShapeInt16Value,
}

impl DxfTextNumericSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn first_alignment(&self) -> &[DxfTextShapeDoubleValue; 3] {
        &self.first_alignment
    }

    #[must_use]
    pub const fn text_height(&self) -> &DxfTextShapeDoubleValue {
        &self.text_height
    }

    #[must_use]
    pub const fn thickness(&self) -> &DxfTextShapeDoubleValue {
        &self.thickness
    }

    #[must_use]
    pub const fn rotation(&self) -> &DxfTextShapeDoubleValue {
        &self.rotation
    }

    #[must_use]
    pub const fn width_factor(&self) -> &DxfTextShapeDoubleValue {
        &self.width_factor
    }

    #[must_use]
    pub const fn oblique_angle(&self) -> &DxfTextShapeDoubleValue {
        &self.oblique_angle
    }

    #[must_use]
    pub const fn generation_flags(&self) -> &DxfTextShapeInt16Value {
        &self.generation_flags
    }

    #[must_use]
    pub const fn horizontal_justification(&self) -> &DxfTextShapeInt16Value {
        &self.horizontal_justification
    }

    #[must_use]
    pub const fn second_alignment(&self) -> &[DxfTextShapeDoubleValue; 3] {
        &self.second_alignment
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfTextShapeDoubleValue; 3] {
        &self.extrusion
    }

    #[must_use]
    pub const fn vertical_justification(&self) -> &DxfTextShapeInt16Value {
        &self.vertical_justification
    }
}

/// Selected and documented-defaulted numeric values for one SHAPE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfShapeNumericSemantics {
    record: DxfTextSymbolRecordEntry,
    insertion: [DxfTextShapeDoubleValue; 3],
    size: DxfTextShapeDoubleValue,
    thickness: DxfTextShapeDoubleValue,
    rotation: DxfTextShapeDoubleValue,
    width_factor: DxfTextShapeDoubleValue,
    oblique_angle: DxfTextShapeDoubleValue,
    extrusion: [DxfTextShapeDoubleValue; 3],
}

impl DxfShapeNumericSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn insertion(&self) -> &[DxfTextShapeDoubleValue; 3] {
        &self.insertion
    }

    #[must_use]
    pub const fn size(&self) -> &DxfTextShapeDoubleValue {
        &self.size
    }

    #[must_use]
    pub const fn thickness(&self) -> &DxfTextShapeDoubleValue {
        &self.thickness
    }

    #[must_use]
    pub const fn rotation(&self) -> &DxfTextShapeDoubleValue {
        &self.rotation
    }

    #[must_use]
    pub const fn width_factor(&self) -> &DxfTextShapeDoubleValue {
        &self.width_factor
    }

    #[must_use]
    pub const fn oblique_angle(&self) -> &DxfTextShapeDoubleValue {
        &self.oblique_angle
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfTextShapeDoubleValue; 3] {
        &self.extrusion
    }
}

/// Immutable lazy TEXT/SHAPE numeric semantics retaining all earlier evidence.
#[derive(Debug)]
pub struct DxfTextShapeScalarDirectory {
    source_id: DxfSourceId,
    cards: DxfTextSymbolCardDirectory,
}

impl DxfTextShapeScalarDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.text_symbol_card_directory(cancellation)?;
        if cards.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: cards.source_id(),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfTextSymbolCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn text_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfTextNumericSemantics>, DxfError> {
        let known = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal());
        if known != Some(record) || record.kind() != DxfTextSymbolKind::Text {
            return Ok(None);
        }
        text_semantics(&self.cards, record).map(Some)
    }

    pub fn shape_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfShapeNumericSemantics>, DxfError> {
        let known = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(record.record().ordinal());
        if known != Some(record) || record.kind() != DxfTextSymbolKind::Shape {
            return Ok(None);
        }
        shape_semantics(&self.cards, record).map(Some)
    }

    pub fn text_semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfTextNumericSemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        self.text_semantics_for_record(record)
    }

    pub fn shape_semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfShapeNumericSemantics>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_record_ordinal)
        else {
            return Ok(None);
        };
        self.shape_semantics_for_record(record)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn text_shape_scalar_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextShapeScalarDirectory, DxfError> {
        DxfTextShapeScalarDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_shape_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextShapeScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_shape_scalar_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_shape_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextShapeScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_shape_scalar_directory(cancellation)
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

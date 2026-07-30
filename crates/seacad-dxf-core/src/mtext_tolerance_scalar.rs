//! Lazy typed scalar semantics for Autodesk MTEXT and TOLERANCE entities.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfRawDocumentView,
    DxfSourceId, DxfTextSymbolCardDirectory, DxfTextSymbolDoubleValue, DxfTextSymbolInt16Value,
    DxfTextSymbolInt32Value, DxfTextSymbolKind, DxfTextSymbolRecordEntry,
};

#[path = "mtext_tolerance_scalar_build.rs"]
mod build;
use build::{mtext_semantics, tolerance_semantics};

pub(super) const MTEXT_NAMESPACE: &str = "text_symbol.mtext";
pub(super) const TOLERANCE_NAMESPACE: &str = "text_symbol.tolerance";

/// Selected unambiguous scalar values for one MTEXT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextNumericSemantics {
    pub(super) record: DxfTextSymbolRecordEntry,
    pub(super) insertion: [DxfTextSymbolDoubleValue; 3],
    pub(super) nominal_height: DxfTextSymbolDoubleValue,
    pub(super) reference_width: DxfTextSymbolDoubleValue,
    pub(super) attachment: DxfTextSymbolInt16Value,
    pub(super) drawing_direction: DxfTextSymbolInt16Value,
    pub(super) extrusion: [DxfTextSymbolDoubleValue; 3],
    pub(super) x_axis: [DxfTextSymbolDoubleValue; 3],
    pub(super) actual_width: DxfTextSymbolDoubleValue,
    pub(super) actual_height: DxfTextSymbolDoubleValue,
    pub(super) line_spacing_style: DxfTextSymbolInt16Value,
    pub(super) line_spacing_factor: DxfTextSymbolDoubleValue,
    pub(super) background_fill: DxfTextSymbolInt32Value,
    pub(super) fill_box_scale: DxfTextSymbolDoubleValue,
    pub(super) background_index: DxfTextSymbolInt16Value,
    pub(super) background_transparency: DxfTextSymbolInt32Value,
    pub(super) column_type: DxfTextSymbolInt16Value,
    pub(super) column_count: DxfTextSymbolInt16Value,
    pub(super) column_flow_reversed: DxfTextSymbolInt16Value,
    pub(super) column_auto_height: DxfTextSymbolInt16Value,
    pub(super) column_width: DxfTextSymbolDoubleValue,
    pub(super) column_gutter: DxfTextSymbolDoubleValue,
}

impl DxfMTextNumericSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn insertion(&self) -> &[DxfTextSymbolDoubleValue; 3] {
        &self.insertion
    }

    #[must_use]
    pub const fn nominal_height(&self) -> &DxfTextSymbolDoubleValue {
        &self.nominal_height
    }

    #[must_use]
    pub const fn reference_width(&self) -> &DxfTextSymbolDoubleValue {
        &self.reference_width
    }

    #[must_use]
    pub const fn attachment(&self) -> &DxfTextSymbolInt16Value {
        &self.attachment
    }

    #[must_use]
    pub const fn drawing_direction(&self) -> &DxfTextSymbolInt16Value {
        &self.drawing_direction
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfTextSymbolDoubleValue; 3] {
        &self.extrusion
    }

    #[must_use]
    pub const fn x_axis(&self) -> &[DxfTextSymbolDoubleValue; 3] {
        &self.x_axis
    }

    #[must_use]
    pub const fn actual_width(&self) -> &DxfTextSymbolDoubleValue {
        &self.actual_width
    }

    #[must_use]
    pub const fn actual_height(&self) -> &DxfTextSymbolDoubleValue {
        &self.actual_height
    }

    #[must_use]
    pub const fn line_spacing_style(&self) -> &DxfTextSymbolInt16Value {
        &self.line_spacing_style
    }

    #[must_use]
    pub const fn line_spacing_factor(&self) -> &DxfTextSymbolDoubleValue {
        &self.line_spacing_factor
    }

    #[must_use]
    pub const fn background_fill(&self) -> &DxfTextSymbolInt32Value {
        &self.background_fill
    }

    #[must_use]
    pub const fn fill_box_scale(&self) -> &DxfTextSymbolDoubleValue {
        &self.fill_box_scale
    }

    #[must_use]
    pub const fn background_index(&self) -> &DxfTextSymbolInt16Value {
        &self.background_index
    }

    #[must_use]
    pub const fn background_transparency(&self) -> &DxfTextSymbolInt32Value {
        &self.background_transparency
    }

    #[must_use]
    pub const fn column_type(&self) -> &DxfTextSymbolInt16Value {
        &self.column_type
    }

    #[must_use]
    pub const fn column_count(&self) -> &DxfTextSymbolInt16Value {
        &self.column_count
    }

    #[must_use]
    pub const fn column_flow_reversed(&self) -> &DxfTextSymbolInt16Value {
        &self.column_flow_reversed
    }

    #[must_use]
    pub const fn column_auto_height(&self) -> &DxfTextSymbolInt16Value {
        &self.column_auto_height
    }

    #[must_use]
    pub const fn column_width(&self) -> &DxfTextSymbolDoubleValue {
        &self.column_width
    }

    #[must_use]
    pub const fn column_gutter(&self) -> &DxfTextSymbolDoubleValue {
        &self.column_gutter
    }
}

/// Selected and documented-defaulted numeric values for one TOLERANCE record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfToleranceNumericSemantics {
    pub(super) record: DxfTextSymbolRecordEntry,
    pub(super) insertion: [DxfTextSymbolDoubleValue; 3],
    pub(super) extrusion: [DxfTextSymbolDoubleValue; 3],
    pub(super) x_axis: [DxfTextSymbolDoubleValue; 3],
}

impl DxfToleranceNumericSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn insertion(&self) -> &[DxfTextSymbolDoubleValue; 3] {
        &self.insertion
    }

    #[must_use]
    pub const fn extrusion(&self) -> &[DxfTextSymbolDoubleValue; 3] {
        &self.extrusion
    }

    #[must_use]
    pub const fn x_axis(&self) -> &[DxfTextSymbolDoubleValue; 3] {
        &self.x_axis
    }
}

/// Immutable lazy MTEXT/TOLERANCE scalar semantics retaining all raw evidence.
#[derive(Debug)]
pub struct DxfMTextToleranceScalarDirectory {
    source_id: DxfSourceId,
    cards: DxfTextSymbolCardDirectory,
}

impl DxfMTextToleranceScalarDirectory {
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

    pub fn mtext_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfMTextNumericSemantics>, DxfError> {
        if self.known_record(record) != Some(record) || record.kind() != DxfTextSymbolKind::MText {
            return Ok(None);
        }
        mtext_semantics(&self.cards, record).map(Some)
    }

    pub fn tolerance_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfToleranceNumericSemantics>, DxfError> {
        if self.known_record(record) != Some(record)
            || record.kind() != DxfTextSymbolKind::Tolerance
        {
            return Ok(None);
        }
        tolerance_semantics(&self.cards, record).map(Some)
    }

    pub fn mtext_semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfMTextNumericSemantics>, DxfError> {
        let Some(record) = self.known_raw_record(raw_record_ordinal) else {
            return Ok(None);
        };
        self.mtext_semantics_for_record(record)
    }

    pub fn tolerance_semantics_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Result<Option<DxfToleranceNumericSemantics>, DxfError> {
        let Some(record) = self.known_raw_record(raw_record_ordinal) else {
            return Ok(None);
        };
        self.tolerance_semantics_for_record(record)
    }

    fn known_record(&self, record: DxfTextSymbolRecordEntry) -> Option<DxfTextSymbolRecordEntry> {
        self.known_raw_record(record.record().ordinal())
    }

    fn known_raw_record(&self, raw_record_ordinal: u64) -> Option<DxfTextSymbolRecordEntry> {
        self.cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_record_ordinal)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_tolerance_scalar_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextToleranceScalarDirectory, DxfError> {
        DxfMTextToleranceScalarDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_tolerance_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextToleranceScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_tolerance_scalar_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_tolerance_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextToleranceScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_tolerance_scalar_directory(cancellation)
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

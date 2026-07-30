//! Source-anchored text/name fields and ordered MTEXT chunk structure.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSemanticValue, DxfSourceId, DxfTextSymbolCardDirectory,
    DxfTextSymbolKind, DxfTextSymbolRecordEntry, DxfTextSymbolValue, DxfTextSymbolValueRole,
};

#[path = "text_symbol_text_chunk.rs"]
mod chunk;
#[path = "text_symbol_text_select.rs"]
mod select;
use chunk::append_sequence;
use select::{required_text, style_name};

const TEXT_NAMESPACE: &str = "text_symbol.text_fields";
const MTEXT_NAMESPACE: &str = "text_symbol.mtext_fields";
const SHAPE_NAMESPACE: &str = "text_symbol.shape_fields";
const TOLERANCE_NAMESPACE: &str = "text_symbol.tolerance_fields";

/// Why one selected text/name field is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolTextIssue {
    MissingRequiredValue,
    MultipleValues { occurrence_count: u32 },
}

pub type DxfTextSymbolSemanticText = DxfSemanticValue<DxfTextSymbolValue, DxfTextSymbolTextIssue>;

/// Explicit text-style evidence or Autodesk's `STANDARD` default.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolStyleName {
    Source(DxfTextSymbolValue),
    Standard,
}

impl DxfTextSymbolStyleName {
    pub const STANDARD: &'static [u8] = b"STANDARD";

    #[must_use]
    pub const fn source(self) -> Option<DxfTextSymbolValue> {
        match self {
            Self::Source(value) => Some(value),
            Self::Standard => None,
        }
    }

    #[must_use]
    pub const fn is_standard_default(self) -> bool {
        matches!(self, Self::Standard)
    }
}

pub type DxfTextSymbolSemanticStyle =
    DxfSemanticValue<DxfTextSymbolStyleName, DxfTextSymbolTextIssue>;

/// One TEXT content/style selection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextFieldSemantics {
    record: DxfTextSymbolRecordEntry,
    content: DxfTextSymbolSemanticText,
    style_name: DxfTextSymbolSemanticStyle,
}

impl DxfTextFieldSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn content(&self) -> &DxfTextSymbolSemanticText {
        &self.content
    }

    #[must_use]
    pub const fn style_name(&self) -> &DxfTextSymbolSemanticStyle {
        &self.style_name
    }
}

/// One SHAPE name selection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfShapeFieldSemantics {
    record: DxfTextSymbolRecordEntry,
    shape_name: DxfTextSymbolSemanticText,
}

impl DxfShapeFieldSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn shape_name(&self) -> &DxfTextSymbolSemanticText {
        &self.shape_name
    }
}

/// One TOLERANCE content/dimension-style selection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfToleranceFieldSemantics {
    record: DxfTextSymbolRecordEntry,
    dimension_style_name: DxfTextSymbolSemanticText,
    content: DxfTextSymbolSemanticText,
}

impl DxfToleranceFieldSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn dimension_style_name(&self) -> &DxfTextSymbolSemanticText {
        &self.dimension_style_name
    }

    #[must_use]
    pub const fn content(&self) -> &DxfTextSymbolSemanticText {
        &self.content
    }
}

/// Source role of one ordered MTEXT content chunk.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextChunkKind {
    Additional,
    Terminal,
}

/// One exact group-3 or group-1 MTEXT chunk in source order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextChunkEntry {
    value: DxfTextSymbolValue,
    kind: DxfMTextChunkKind,
}

impl DxfMTextChunkEntry {
    #[must_use]
    pub const fn value(self) -> DxfTextSymbolValue {
        self.value
    }

    #[must_use]
    pub const fn kind(self) -> DxfMTextChunkKind {
        self.kind
    }
}

/// Half-open global chunk range owned by one MTEXT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextChunkRange {
    start: u32,
    end: u32,
}

impl DxfMTextChunkRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Ordered MTEXT chunk accounting without discarding malformed source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextChunkSequence {
    source_id: DxfSourceId,
    record: DxfTextSymbolRecordEntry,
    chunk_range: DxfMTextChunkRange,
    terminal_count: u32,
    additional_after_terminal_count: u32,
}

impl DxfMTextChunkSequence {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn chunk_range(self) -> DxfMTextChunkRange {
        self.chunk_range
    }

    #[must_use]
    pub const fn terminal_count(self) -> u64 {
        self.terminal_count as u64
    }

    #[must_use]
    pub const fn additional_after_terminal_count(self) -> u64 {
        self.additional_after_terminal_count as u64
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.terminal_count == 1 && self.additional_after_terminal_count == 0
    }
}

/// MTEXT style plus exact ordered chunk accounting.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextFieldSemantics {
    record: DxfTextSymbolRecordEntry,
    style_name: DxfTextSymbolSemanticStyle,
    chunks: DxfMTextChunkSequence,
}

impl DxfMTextFieldSemantics {
    #[must_use]
    pub const fn record(self) -> DxfTextSymbolRecordEntry {
        self.record
    }

    #[must_use]
    pub const fn style_name(&self) -> &DxfTextSymbolSemanticStyle {
        &self.style_name
    }

    #[must_use]
    pub const fn chunks(self) -> DxfMTextChunkSequence {
        self.chunks
    }
}

/// Immutable text/name semantics retaining cards, evidence, and MTEXT chunks.
#[derive(Debug)]
pub struct DxfTextSymbolTextDirectory {
    source_id: DxfSourceId,
    cards: DxfTextSymbolCardDirectory,
    sequences: Box<[DxfMTextChunkSequence]>,
    chunks: Box<[DxfMTextChunkEntry]>,
}

impl DxfTextSymbolTextDirectory {
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
        let mut sequences = Vec::new();
        let mut chunks = Vec::new();
        for record in cards.evidence_directory().records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if record.kind() == DxfTextSymbolKind::MText {
                append_sequence(&cards, record, &mut sequences, &mut chunks)?;
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            sequences: sequences.into_boxed_slice(),
            chunks: chunks.into_boxed_slice(),
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

    #[must_use]
    pub fn sequences(&self) -> &[DxfMTextChunkSequence] {
        &self.sequences
    }

    #[must_use]
    pub fn chunks(&self) -> &[DxfMTextChunkEntry] {
        &self.chunks
    }

    #[must_use]
    pub fn sequence_for_raw_record(&self, raw: u64) -> Option<DxfMTextChunkSequence> {
        self.sequences
            .binary_search_by_key(&raw, |entry| entry.record().record().ordinal())
            .ok()
            .and_then(|index| self.sequences.get(index))
            .copied()
    }

    #[must_use]
    pub fn chunks_for_sequence(
        &self,
        sequence: DxfMTextChunkSequence,
    ) -> Option<&[DxfMTextChunkEntry]> {
        if self.sequence_for_raw_record(sequence.record().record().ordinal()) != Some(sequence) {
            return None;
        }
        let range = sequence.chunk_range();
        self.chunks
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }

    pub fn text_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfTextFieldSemantics>, DxfError> {
        if !self.is_known_kind(record, DxfTextSymbolKind::Text) {
            return Ok(None);
        }
        Ok(Some(DxfTextFieldSemantics {
            record,
            content: required_text(
                &self.cards,
                record,
                DxfTextSymbolValueRole::Content,
                TEXT_NAMESPACE,
                "content",
            )?,
            style_name: style_name(&self.cards, record, TEXT_NAMESPACE)?,
        }))
    }

    pub fn mtext_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfMTextFieldSemantics>, DxfError> {
        if !self.is_known_kind(record, DxfTextSymbolKind::MText) {
            return Ok(None);
        }
        let chunks = self
            .sequence_for_raw_record(record.record().ordinal())
            .ok_or_else(invalid_internal_data)?;
        Ok(Some(DxfMTextFieldSemantics {
            record,
            style_name: style_name(&self.cards, record, MTEXT_NAMESPACE)?,
            chunks,
        }))
    }

    pub fn shape_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfShapeFieldSemantics>, DxfError> {
        if !self.is_known_kind(record, DxfTextSymbolKind::Shape) {
            return Ok(None);
        }
        Ok(Some(DxfShapeFieldSemantics {
            record,
            shape_name: required_text(
                &self.cards,
                record,
                DxfTextSymbolValueRole::ShapeName,
                SHAPE_NAMESPACE,
                "shape_name",
            )?,
        }))
    }

    pub fn tolerance_semantics_for_record(
        &self,
        record: DxfTextSymbolRecordEntry,
    ) -> Result<Option<DxfToleranceFieldSemantics>, DxfError> {
        if !self.is_known_kind(record, DxfTextSymbolKind::Tolerance) {
            return Ok(None);
        }
        Ok(Some(DxfToleranceFieldSemantics {
            record,
            dimension_style_name: required_text(
                &self.cards,
                record,
                DxfTextSymbolValueRole::DimensionStyleName,
                TOLERANCE_NAMESPACE,
                "dimension_style_name",
            )?,
            content: required_text(
                &self.cards,
                record,
                DxfTextSymbolValueRole::Content,
                TOLERANCE_NAMESPACE,
                "content",
            )?,
        }))
    }

    fn is_known_kind(&self, record: DxfTextSymbolRecordEntry, kind: DxfTextSymbolKind) -> bool {
        record.kind() == kind
            && self
                .cards
                .evidence_directory()
                .record_for_raw_ordinal(record.record().ordinal())
                == Some(record)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn text_symbol_text_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolTextDirectory, DxfError> {
        DxfTextSymbolTextDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_symbol_text_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolTextDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_symbol_text_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_symbol_text_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolTextDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_symbol_text_directory(cancellation)
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

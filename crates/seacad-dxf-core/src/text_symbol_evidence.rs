//! Source-anchored TEXT, MTEXT, SHAPE, and TOLERANCE field evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfSourceId,
    raw_double::decode_raw_double,
    raw_integer::{decode_raw_i16, decode_raw_i32},
    text_symbol_role::{DxfTextSymbolWireType, value_role},
};

/// Reviewed text-and-symbol entity family.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolKind {
    Text,
    MText,
    Shape,
    Tolerance,
}

/// Documented field role retained without selecting duplicate values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolValueRole {
    Thickness,
    FirstAlignmentX,
    FirstAlignmentY,
    FirstAlignmentZ,
    SecondAlignmentX,
    SecondAlignmentY,
    SecondAlignmentZ,
    InsertionX,
    InsertionY,
    InsertionZ,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
    XAxisX,
    XAxisY,
    XAxisZ,
    TextHeight,
    NominalHeight,
    ShapeSize,
    Content,
    AdditionalContent,
    ShapeName,
    StyleName,
    DimensionStyleName,
    Rotation,
    RotationOrColumnHeight,
    WidthFactor,
    ObliqueAngle,
    GenerationFlags,
    HorizontalJustification,
    VerticalJustification,
    ReferenceWidth,
    Attachment,
    DrawingDirection,
    ActualWidth,
    ActualHeight,
    LineSpacingStyle,
    LineSpacingFactor,
    BackgroundFill,
    BackgroundRgbOrEntityTrueColor,
    BackgroundNameOrEntityColorName,
    FillBoxScale,
    BackgroundIndex,
    BackgroundTransparency,
    ColumnType,
    ColumnCount,
    ColumnFlowReversed,
    ColumnAutoHeight,
    ColumnWidth,
    ColumnGutter,
}

/// Lexical reason why one documented numeric field is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Exact wire-domain value; text remains in the raw group's source span.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfTextSymbolValueData {
    Text,
    Double(Result<DxfDouble, DxfTextSymbolNumericIssue>),
    Int16(Result<i16, DxfTextSymbolNumericIssue>),
    Int32(Result<i32, DxfTextSymbolNumericIssue>),
}

/// Half-open field-value range owned by one entity record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextSymbolValueRange {
    start: u32,
    end: u32,
}

impl DxfTextSymbolValueRange {
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

/// One source-order documented group and its exact typed wire evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextSymbolValue {
    group: DxfRawGroup,
    role: DxfTextSymbolValueRole,
    data: DxfTextSymbolValueData,
}

impl DxfTextSymbolValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfTextSymbolValueRole {
        self.role
    }

    #[must_use]
    pub const fn data(self) -> DxfTextSymbolValueData {
        self.data
    }
}

/// One exact entity marker and its source-order documented field slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTextSymbolRecordEntry {
    record: DxfRawRecord,
    kind: DxfTextSymbolKind,
    value_range: DxfTextSymbolValueRange,
}

impl DxfTextSymbolRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfTextSymbolKind {
        self.kind
    }

    #[must_use]
    pub const fn value_range(self) -> DxfTextSymbolValueRange {
        self.value_range
    }
}

/// Immutable raw field directory for reviewed text-and-symbol records.
#[derive(Debug)]
pub struct DxfTextSymbolDirectory {
    source_id: DxfSourceId,
    raw_record_count: u32,
    records: Box<[DxfTextSymbolRecordEntry]>,
    values: Box<[DxfTextSymbolValue]>,
}

impl DxfTextSymbolDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        if raw_records.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: raw_records.source_id(),
            });
        }
        let raw_record_count = compact_len(raw_records.records().len())?;
        let mut records = Vec::new();
        let mut values = Vec::new();
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !matches!(
                record.section_kind(),
                DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
            ) {
                continue;
            }
            let Some(kind) = record_kind(document, record)? else {
                continue;
            };
            let start = compact_len(values.len())?;
            append_values(document, record, kind, cancellation, &mut values)?;
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfTextSymbolRecordEntry {
                record,
                kind,
                value_range: DxfTextSymbolValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            raw_record_count,
            records: records.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn raw_record_count(&self) -> u64 {
        self.raw_record_count as u64
    }

    #[must_use]
    pub fn records(&self) -> &[DxfTextSymbolRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfTextSymbolValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(&self, raw: u64) -> Option<DxfTextSymbolRecordEntry> {
        self.records
            .binary_search_by_key(&raw, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(&self, raw: u64) -> Option<&[DxfTextSymbolValue]> {
        let range = self.record_for_raw_ordinal(raw)?.value_range();
        self.values
            .get(usize::try_from(range.start()).ok()?..usize::try_from(range.end()).ok()?)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfTextSymbolValue> {
        let index = self
            .values
            .partition_point(|entry| entry.group().occurrence() < occurrence);
        self.values
            .get(index)
            .copied()
            .filter(|entry| entry.group().occurrence() == occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn text_symbol_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolDirectory, DxfError> {
        DxfTextSymbolDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn text_symbol_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_symbol_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn text_symbol_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTextSymbolDirectory, DxfError> {
        DxfRawDocumentView::from(self).text_symbol_directory(cancellation)
    }
}

fn append_values(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    kind: DxfTextSymbolKind,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfTextSymbolValue>,
) -> Result<(), DxfError> {
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
        ensure_not_cancelled(cancellation)?;
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if kind == DxfTextSymbolKind::MText
            && group.group_code().value() == 101
            && document.raw_span_equals_exact(group.value_payload_span(), b"Embedded Object")?
        {
            break;
        }
        let Some((role, wire_type)) = value_role(kind, group.group_code().value()) else {
            continue;
        };
        let data = decode_value(document, group, wire_type, cancellation)?;
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(DxfTextSymbolValue { group, role, data });
    }
    Ok(())
}

fn record_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<Option<DxfTextSymbolKind>, DxfError> {
    let span = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?
        .value_payload_span();
    if document.raw_span_equals_exact(span, b"TEXT")? {
        Ok(Some(DxfTextSymbolKind::Text))
    } else if document.raw_span_equals_exact(span, b"MTEXT")? {
        Ok(Some(DxfTextSymbolKind::MText))
    } else if document.raw_span_equals_exact(span, b"SHAPE")? {
        Ok(Some(DxfTextSymbolKind::Shape))
    } else if document.raw_span_equals_exact(span, b"TOLERANCE")? {
        Ok(Some(DxfTextSymbolKind::Tolerance))
    } else {
        Ok(None)
    }
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    wire_type: DxfTextSymbolWireType,
    cancellation: &DxfCancellationToken,
) -> Result<DxfTextSymbolValueData, DxfError> {
    let issue = DxfTextSymbolNumericIssue::InvalidAsciiNumber;
    match wire_type {
        DxfTextSymbolWireType::Text => Ok(DxfTextSymbolValueData::Text),
        DxfTextSymbolWireType::Double => decode_raw_double(document, group, cancellation)
            .map(|value| DxfTextSymbolValueData::Double(value.map_err(issue))),
        DxfTextSymbolWireType::Int16 => decode_raw_i16(document, group, cancellation)
            .map(|value| DxfTextSymbolValueData::Int16(value.map_err(issue))),
        DxfTextSymbolWireType::Int32 => decode_raw_i32(document, group, cancellation)
            .map(|value| DxfTextSymbolValueData::Int32(value.map_err(issue))),
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

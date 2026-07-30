//! Source-anchored classic ATTRIB defining-value evidence.

use crate::{
    ByteSpan, DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfDouble, DxfError, DxfInsertAttributeSequenceDirectory,
    DxfInsertAttributeSequenceEntry, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfSourceId, DxfTextEncodingResolution, DxfTextValueDecodeReceipt,
    raw_double::decode_raw_double, raw_integer::decode_raw_i16,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeValueRole {
    Thickness,
    TextStartX,
    TextStartY,
    TextStartZ,
    TextHeight,
    TextValue,
    AttributeTag,
    AttributeFlags,
    FieldLength,
    RotationAngle,
    RelativeXScale,
    ObliqueAngle,
    TextStyleName,
    TextGenerationFlags,
    HorizontalJustification,
    VerticalJustification,
    AlignmentPointX,
    AlignmentPointY,
    AlignmentPointZ,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
    VersionOrLockPosition,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeTextValue {
    source_id: DxfSourceId,
    group_occurrence: u32,
    value_span: ByteSpan,
    encoding: DxfTextEncodingResolution,
}

impl DxfInsertAttributeTextValue {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn group_occurrence(self) -> u64 {
        self.group_occurrence as u64
    }

    #[must_use]
    pub const fn value_span(self) -> ByteSpan {
        self.value_span
    }

    #[must_use]
    pub const fn encoding(self) -> DxfTextEncodingResolution {
        self.encoding
    }

    pub fn decode_to_utf8_without_replacement(
        self,
        document: DxfRawDocumentView<'_>,
        destination: &mut [u8],
    ) -> Result<DxfTextValueDecodeReceipt, DxfError> {
        if document.source_id() != self.source_id {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: document.source_id(),
            });
        }
        let receipt = document
            .decode_group_value_to_utf8_without_replacement(self.group_occurrence(), destination)?;
        if receipt.source_id() != self.source_id
            || receipt.group_occurrence() != self.group_occurrence()
            || receipt.value_span() != self.value_span
            || receipt.encoding() != self.encoding
        {
            return Err(invalid_internal_data());
        }
        Ok(receipt)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeValueData {
    Text(DxfInsertAttributeTextValue),
    Double(DxfDouble),
    Int16(i16),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeValueIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeValueRange {
    start: u32,
    end: u32,
}

impl DxfInsertAttributeValueRange {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeValue {
    group: DxfRawGroup,
    role: DxfInsertAttributeValueRole,
    value: Result<DxfInsertAttributeValueData, DxfInsertAttributeValueIssue>,
}

impl DxfInsertAttributeValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfInsertAttributeValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfInsertAttributeValueData, DxfInsertAttributeValueIssue> {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeValueEntry {
    sequence: DxfInsertAttributeSequenceEntry,
    record: DxfRawRecord,
    sequence_attribute_ordinal: u32,
    value_range: DxfInsertAttributeValueRange,
}

impl DxfInsertAttributeValueEntry {
    #[must_use]
    pub const fn sequence(self) -> DxfInsertAttributeSequenceEntry {
        self.sequence
    }

    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn sequence_attribute_ordinal(self) -> u64 {
        self.sequence_attribute_ordinal as u64
    }

    #[must_use]
    pub const fn value_range(self) -> DxfInsertAttributeValueRange {
        self.value_range
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributeValueDirectory {
    source_id: DxfSourceId,
    sequences: DxfInsertAttributeSequenceDirectory,
    records: Box<[DxfInsertAttributeValueEntry]>,
    values: Box<[DxfInsertAttributeValue]>,
}

impl DxfInsertAttributeValueDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let sequences = document.insert_attribute_sequence_directory(cancellation)?;
        if sequences.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: sequences.source_id(),
            });
        }
        let application_groups = sequences
            .semantic_directory()
            .card_directory()
            .evidence_directory()
            .application_group_directory();
        let mut records = Vec::new();
        let mut values = Vec::new();
        records
            .try_reserve(sequences.attribute_records().len())
            .map_err(|_| out_of_memory())?;
        for sequence in sequences.entries().iter().copied() {
            let Some(attributes) =
                sequences.attributes_for_insert_raw_ordinal(sequence.insert().record().ordinal())
            else {
                return Err(invalid_internal_data());
            };
            for (sequence_attribute_ordinal, record) in attributes.iter().copied().enumerate() {
                ensure_not_cancelled(cancellation)?;
                let start = compact_len(values.len())?;
                append_record_values(
                    document,
                    application_groups,
                    record,
                    cancellation,
                    &mut values,
                )?;
                let end = compact_len(values.len())?;
                records.push(DxfInsertAttributeValueEntry {
                    sequence,
                    record,
                    sequence_attribute_ordinal: u32::try_from(sequence_attribute_ordinal)
                        .map_err(|_| invalid_internal_data())?,
                    value_range: DxfInsertAttributeValueRange::new(start, end)?,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            sequences,
            records: records.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn sequence_directory(&self) -> &DxfInsertAttributeSequenceDirectory {
        &self.sequences
    }

    #[must_use]
    pub fn records(&self) -> &[DxfInsertAttributeValueEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfInsertAttributeValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertAttributeValueEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfInsertAttributeValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfInsertAttributeValue> {
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
    pub fn insert_attribute_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeValueDirectory, DxfError> {
        DxfInsertAttributeValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_value_directory(cancellation)
    }
}

fn append_record_values(
    document: DxfRawDocumentView<'_>,
    application_groups: &crate::DxfApplicationGroupDirectory,
    record: DxfRawRecord,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfInsertAttributeValue>,
) -> Result<(), DxfError> {
    let mut context = AttributeSubclassContext::Legacy;
    for occurrence in record.marker_occurrence().saturating_add(1)..record.group_range().end() {
        ensure_not_cancelled(cancellation)?;
        if application_groups
            .group_for_content_occurrence(occurrence)
            .is_some()
        {
            continue;
        }
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if group.group_code().value() == 100 {
            context = subclass_context(document, group)?;
            if context == AttributeSubclassContext::Extension {
                break;
            }
            continue;
        }
        if !matches!(
            context,
            AttributeSubclassContext::Legacy | AttributeSubclassContext::Classic
        ) {
            continue;
        }
        let Some(role) = value_role(group.group_code().value()) else {
            continue;
        };
        let value = decode_value(document, group, role, cancellation)?;
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(DxfInsertAttributeValue { group, role, value });
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AttributeSubclassContext {
    Legacy,
    Classic,
    Other,
    Extension,
}

fn subclass_context(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
) -> Result<AttributeSubclassContext, DxfError> {
    let span = group.value_payload_span();
    if span.len() == b"AcDbText".len() as u64
        && document.raw_span_equals_exact(span, b"AcDbText")?
        || span.len() == b"AcDbAttribute".len() as u64
            && document.raw_span_equals_exact(span, b"AcDbAttribute")?
    {
        return Ok(AttributeSubclassContext::Classic);
    }
    if span.len() == b"AcDbXrecord".len() as u64
        && document.raw_span_equals_exact(span, b"AcDbXrecord")?
    {
        return Ok(AttributeSubclassContext::Extension);
    }
    Ok(AttributeSubclassContext::Other)
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfInsertAttributeValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfInsertAttributeValueData, DxfInsertAttributeValueIssue>, DxfError> {
    match wire_kind(role) {
        AttributeWireKind::Text => {
            let group_occurrence =
                u32::try_from(group.occurrence()).map_err(|_| invalid_internal_data())?;
            Ok(Ok(DxfInsertAttributeValueData::Text(
                DxfInsertAttributeTextValue {
                    source_id: document.source_id(),
                    group_occurrence,
                    value_span: group.value_payload_span(),
                    encoding: document.text_encoding_report().resolution(),
                },
            )))
        }
        AttributeWireKind::Double => {
            decode_raw_double(document, group, cancellation).map(|value| {
                value
                    .map(DxfInsertAttributeValueData::Double)
                    .map_err(DxfInsertAttributeValueIssue::InvalidAsciiNumber)
            })
        }
        AttributeWireKind::Int16 => decode_raw_i16(document, group, cancellation).map(|value| {
            value
                .map(DxfInsertAttributeValueData::Int16)
                .map_err(DxfInsertAttributeValueIssue::InvalidAsciiNumber)
        }),
    }
}

#[derive(Clone, Copy)]
enum AttributeWireKind {
    Text,
    Double,
    Int16,
}

const fn wire_kind(role: DxfInsertAttributeValueRole) -> AttributeWireKind {
    use DxfInsertAttributeValueRole::{
        AlignmentPointX, AlignmentPointY, AlignmentPointZ, AttributeFlags, AttributeTag,
        ExtrusionX, ExtrusionY, ExtrusionZ, FieldLength, HorizontalJustification, ObliqueAngle,
        RelativeXScale, RotationAngle, TextGenerationFlags, TextHeight, TextStartX, TextStartY,
        TextStartZ, TextStyleName, TextValue, Thickness, VersionOrLockPosition,
        VerticalJustification,
    };
    match role {
        TextValue | AttributeTag | TextStyleName => AttributeWireKind::Text,
        Thickness | TextStartX | TextStartY | TextStartZ | TextHeight | RotationAngle
        | RelativeXScale | ObliqueAngle | AlignmentPointX | AlignmentPointY | AlignmentPointZ
        | ExtrusionX | ExtrusionY | ExtrusionZ => AttributeWireKind::Double,
        AttributeFlags
        | FieldLength
        | TextGenerationFlags
        | HorizontalJustification
        | VerticalJustification
        | VersionOrLockPosition => AttributeWireKind::Int16,
    }
}

const fn value_role(group_code: i16) -> Option<DxfInsertAttributeValueRole> {
    use DxfInsertAttributeValueRole::{
        AlignmentPointX, AlignmentPointY, AlignmentPointZ, AttributeFlags, AttributeTag,
        ExtrusionX, ExtrusionY, ExtrusionZ, FieldLength, HorizontalJustification, ObliqueAngle,
        RelativeXScale, RotationAngle, TextGenerationFlags, TextHeight, TextStartX, TextStartY,
        TextStartZ, TextStyleName, TextValue, Thickness, VersionOrLockPosition,
        VerticalJustification,
    };
    match group_code {
        39 => Some(Thickness),
        10 => Some(TextStartX),
        20 => Some(TextStartY),
        30 => Some(TextStartZ),
        40 => Some(TextHeight),
        1 => Some(TextValue),
        2 => Some(AttributeTag),
        70 => Some(AttributeFlags),
        73 => Some(FieldLength),
        50 => Some(RotationAngle),
        41 => Some(RelativeXScale),
        51 => Some(ObliqueAngle),
        7 => Some(TextStyleName),
        71 => Some(TextGenerationFlags),
        72 => Some(HorizontalJustification),
        74 => Some(VerticalJustification),
        11 => Some(AlignmentPointX),
        21 => Some(AlignmentPointY),
        31 => Some(AlignmentPointZ),
        210 => Some(ExtrusionX),
        220 => Some(ExtrusionY),
        230 => Some(ExtrusionZ),
        280 => Some(VersionOrLockPosition),
        _ => None,
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
    io_error(std::io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(std::io::ErrorKind::OutOfMemory)
}

fn io_error(kind: std::io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &std::io::Error::from(kind))
}

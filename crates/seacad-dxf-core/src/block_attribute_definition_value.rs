//! Source-anchored classic ATTDEF defining-value evidence.

use std::io;

use crate::{
    ByteSpan, DxfApplicationGroupDirectory, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfBlockAttributeDefinitionDirectory, DxfBlockAttributeDefinitionEntry,
    DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup,
    DxfSourceId, DxfTextEncodingResolution, DxfTextValueDecodeReceipt,
    raw_double::decode_raw_double, raw_integer::decode_raw_i16,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionValueRole {
    Thickness,
    TextStartX,
    TextStartY,
    TextStartZ,
    TextHeight,
    DefaultValue,
    Prompt,
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
pub struct DxfBlockAttributeDefinitionTextValue {
    source_id: DxfSourceId,
    group_occurrence: u32,
    value_span: ByteSpan,
    encoding: DxfTextEncodingResolution,
}

impl DxfBlockAttributeDefinitionTextValue {
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
pub enum DxfBlockAttributeDefinitionValueData {
    Text(DxfBlockAttributeDefinitionTextValue),
    Double(DxfDouble),
    Int16(i16),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionValueIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionValueRange {
    start: u32,
    end: u32,
}

impl DxfBlockAttributeDefinitionValueRange {
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
pub struct DxfBlockAttributeDefinitionValue {
    group: DxfRawGroup,
    role: DxfBlockAttributeDefinitionValueRole,
    value: Result<DxfBlockAttributeDefinitionValueData, DxfBlockAttributeDefinitionValueIssue>,
}

impl DxfBlockAttributeDefinitionValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfBlockAttributeDefinitionValueRole {
        self.role
    }

    pub const fn value(
        self,
    ) -> Result<DxfBlockAttributeDefinitionValueData, DxfBlockAttributeDefinitionValueIssue> {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionValueEntry {
    definition: DxfBlockAttributeDefinitionEntry,
    value_range: DxfBlockAttributeDefinitionValueRange,
}

impl DxfBlockAttributeDefinitionValueEntry {
    #[must_use]
    pub const fn definition(self) -> DxfBlockAttributeDefinitionEntry {
        self.definition
    }

    #[must_use]
    pub const fn value_range(self) -> DxfBlockAttributeDefinitionValueRange {
        self.value_range
    }
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionValueDirectory {
    source_id: DxfSourceId,
    definitions: DxfBlockAttributeDefinitionDirectory,
    application_groups: DxfApplicationGroupDirectory,
    records: Box<[DxfBlockAttributeDefinitionValueEntry]>,
    values: Box<[DxfBlockAttributeDefinitionValue]>,
}

impl DxfBlockAttributeDefinitionValueDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let definitions = document.block_attribute_definition_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        for observed in [definitions.source_id(), application_groups.source_id()] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut records = Vec::new();
        let mut values = Vec::new();
        records
            .try_reserve(definitions.entries().len())
            .map_err(|_| out_of_memory())?;
        for definition in definitions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(values.len())?;
            append_record_values(
                document,
                &application_groups,
                definition,
                cancellation,
                &mut values,
            )?;
            let end = compact_len(values.len())?;
            records.push(DxfBlockAttributeDefinitionValueEntry {
                definition,
                value_range: DxfBlockAttributeDefinitionValueRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            definitions,
            application_groups,
            records: records.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn definition_directory(&self) -> &DxfBlockAttributeDefinitionDirectory {
        &self.definitions
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        &self.application_groups
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBlockAttributeDefinitionValueEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfBlockAttributeDefinitionValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfBlockAttributeDefinitionValueEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.definition().record().ordinal()
            })
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBlockAttributeDefinitionValue]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfBlockAttributeDefinitionValue> {
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
    pub fn block_attribute_definition_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionValueDirectory, DxfError> {
        DxfBlockAttributeDefinitionValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_value_directory(cancellation)
    }
}

fn append_record_values(
    document: DxfRawDocumentView<'_>,
    application_groups: &DxfApplicationGroupDirectory,
    definition: DxfBlockAttributeDefinitionEntry,
    cancellation: &DxfCancellationToken,
    values: &mut Vec<DxfBlockAttributeDefinitionValue>,
) -> Result<(), DxfError> {
    let record = definition.record();
    let mut context = AttributeDefinitionSubclassContext::Legacy;
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
            if context == AttributeDefinitionSubclassContext::Extension {
                break;
            }
            continue;
        }
        if !matches!(
            context,
            AttributeDefinitionSubclassContext::Legacy
                | AttributeDefinitionSubclassContext::Classic
        ) {
            continue;
        }
        let Some(role) = value_role(group.group_code().value()) else {
            continue;
        };
        let value = decode_value(document, group, role, cancellation)?;
        values.try_reserve(1).map_err(|_| out_of_memory())?;
        values.push(DxfBlockAttributeDefinitionValue { group, role, value });
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AttributeDefinitionSubclassContext {
    Legacy,
    Classic,
    Other,
    Extension,
}

fn subclass_context(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
) -> Result<AttributeDefinitionSubclassContext, DxfError> {
    let span = group.value_payload_span();
    if span_equals(document, span, b"AcDbText")?
        || span_equals(document, span, b"AcDbAttributeDefinition")?
    {
        return Ok(AttributeDefinitionSubclassContext::Classic);
    }
    if span_equals(document, span, b"AcDbXrecord")? {
        return Ok(AttributeDefinitionSubclassContext::Extension);
    }
    Ok(AttributeDefinitionSubclassContext::Other)
}

fn span_equals(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    expected: &[u8],
) -> Result<bool, DxfError> {
    Ok(span.len() == expected.len() as u64 && document.raw_span_equals_exact(span, expected)?)
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfBlockAttributeDefinitionValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<
    Result<DxfBlockAttributeDefinitionValueData, DxfBlockAttributeDefinitionValueIssue>,
    DxfError,
> {
    match wire_kind(role) {
        AttributeDefinitionWireKind::Text => {
            let group_occurrence =
                u32::try_from(group.occurrence()).map_err(|_| invalid_internal_data())?;
            Ok(Ok(DxfBlockAttributeDefinitionValueData::Text(
                DxfBlockAttributeDefinitionTextValue {
                    source_id: document.source_id(),
                    group_occurrence,
                    value_span: group.value_payload_span(),
                    encoding: document.text_encoding_report().resolution(),
                },
            )))
        }
        AttributeDefinitionWireKind::Double => decode_raw_double(document, group, cancellation)
            .map(|value| {
                value
                    .map(DxfBlockAttributeDefinitionValueData::Double)
                    .map_err(DxfBlockAttributeDefinitionValueIssue::InvalidAsciiNumber)
            }),
        AttributeDefinitionWireKind::Int16 => {
            decode_raw_i16(document, group, cancellation).map(|value| {
                value
                    .map(DxfBlockAttributeDefinitionValueData::Int16)
                    .map_err(DxfBlockAttributeDefinitionValueIssue::InvalidAsciiNumber)
            })
        }
    }
}

#[derive(Clone, Copy)]
enum AttributeDefinitionWireKind {
    Text,
    Double,
    Int16,
}

const fn wire_kind(role: DxfBlockAttributeDefinitionValueRole) -> AttributeDefinitionWireKind {
    use DxfBlockAttributeDefinitionValueRole::{
        AlignmentPointX, AlignmentPointY, AlignmentPointZ, AttributeFlags, AttributeTag,
        DefaultValue, ExtrusionX, ExtrusionY, ExtrusionZ, FieldLength, HorizontalJustification,
        ObliqueAngle, Prompt, RelativeXScale, RotationAngle, TextGenerationFlags, TextHeight,
        TextStartX, TextStartY, TextStartZ, TextStyleName, Thickness, VersionOrLockPosition,
        VerticalJustification,
    };
    match role {
        DefaultValue | Prompt | AttributeTag | TextStyleName => AttributeDefinitionWireKind::Text,
        Thickness | TextStartX | TextStartY | TextStartZ | TextHeight | RotationAngle
        | RelativeXScale | ObliqueAngle | AlignmentPointX | AlignmentPointY | AlignmentPointZ
        | ExtrusionX | ExtrusionY | ExtrusionZ => AttributeDefinitionWireKind::Double,
        AttributeFlags
        | FieldLength
        | TextGenerationFlags
        | HorizontalJustification
        | VerticalJustification
        | VersionOrLockPosition => AttributeDefinitionWireKind::Int16,
    }
}

const fn value_role(group_code: i16) -> Option<DxfBlockAttributeDefinitionValueRole> {
    use DxfBlockAttributeDefinitionValueRole::{
        AlignmentPointX, AlignmentPointY, AlignmentPointZ, AttributeFlags, AttributeTag,
        DefaultValue, ExtrusionX, ExtrusionY, ExtrusionZ, FieldLength, HorizontalJustification,
        ObliqueAngle, Prompt, RelativeXScale, RotationAngle, TextGenerationFlags, TextHeight,
        TextStartX, TextStartY, TextStartZ, TextStyleName, Thickness, VersionOrLockPosition,
        VerticalJustification,
    };
    match group_code {
        39 => Some(Thickness),
        10 => Some(TextStartX),
        20 => Some(TextStartY),
        30 => Some(TextStartZ),
        40 => Some(TextHeight),
        1 => Some(DefaultValue),
        3 => Some(Prompt),
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
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

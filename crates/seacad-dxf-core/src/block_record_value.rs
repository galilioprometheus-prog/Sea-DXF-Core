//! Source-anchored defining-value evidence from exact BLOCK records.

use std::io;

use crate::{
    ByteSpan, DxfApplicationGroupDirectory, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfBlockDefinitionDirectory, DxfBlockDefinitionEntry,
    DxfCancellationToken, DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup,
    DxfSourceId, DxfTextEncodingResolution, DxfTextValueDecodeReceipt,
    raw_double::decode_raw_double, raw_integer::decode_raw_i16,
};

/// Documented defining-value role retained from one BLOCK record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockRecordValueRole {
    PrimaryName,
    Flags,
    BasePointX,
    BasePointY,
    BasePointZ,
    SecondaryName,
    XrefPath,
    Description,
}

/// Borrowed-by-identity reference to one exact raw BLOCK text value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordTextValue {
    source_id: DxfSourceId,
    group_occurrence: u32,
    value_span: ByteSpan,
    encoding: DxfTextEncodingResolution,
}

impl DxfBlockRecordTextValue {
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

    /// Decodes from the original source without replacement or normalization.
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

/// Exact text, signed integer, or binary64 BLOCK value without merging domains.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockRecordValueData {
    Text(DxfBlockRecordTextValue),
    Double(DxfDouble),
    Int16(i16),
}

/// Lexical reason why one BLOCK number cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockRecordValueIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open defining-value range owned by one BLOCK record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordValueRange {
    start: u32,
    end: u32,
}

impl DxfBlockRecordValueRange {
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

/// One source-order BLOCK defining group and its exact value evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordValue {
    group: DxfRawGroup,
    role: DxfBlockRecordValueRole,
    value: Result<DxfBlockRecordValueData, DxfBlockRecordValueIssue>,
}

impl DxfBlockRecordValue {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfBlockRecordValueRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfBlockRecordValueData, DxfBlockRecordValueIssue> {
        self.value
    }
}

/// One M10.1a definition entry and its BLOCK-record defining-value slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockRecordValueEntry {
    definition: DxfBlockDefinitionEntry,
    value_range: DxfBlockRecordValueRange,
}

impl DxfBlockRecordValueEntry {
    #[must_use]
    pub const fn definition(self) -> DxfBlockDefinitionEntry {
        self.definition
    }

    #[must_use]
    pub const fn value_range(self) -> DxfBlockRecordValueRange {
        self.value_range
    }
}

/// Immutable defining-value evidence directory for exact BLOCK records.
#[derive(Debug)]
pub struct DxfBlockRecordValueDirectory {
    source_id: DxfSourceId,
    definitions: DxfBlockDefinitionDirectory,
    application_groups: DxfApplicationGroupDirectory,
    records: Box<[DxfBlockRecordValueEntry]>,
    values: Box<[DxfBlockRecordValue]>,
}

impl DxfBlockRecordValueDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let definitions = document.block_definition_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        for observed in [
            definitions.source_id(),
            application_groups.source_id(),
            document.text_encoding_report().source_id(),
        ] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut records = Vec::new();
        let mut values = Vec::new();
        for definition in definitions.definitions().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let record = definition.block_record();
            let start = compact_len(values.len())?;
            for occurrence in
                record.marker_occurrence().saturating_add(1)..record.group_range().end()
            {
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
                let Some(role) = value_role(group.group_code().value()) else {
                    continue;
                };
                let value = decode_value(document, group, role, cancellation)?;
                values.try_reserve(1).map_err(|_| out_of_memory())?;
                values.push(DxfBlockRecordValue { group, role, value });
            }
            let end = compact_len(values.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfBlockRecordValueEntry {
                definition,
                value_range: DxfBlockRecordValueRange::new(start, end)?,
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
    pub const fn definition_directory(&self) -> &DxfBlockDefinitionDirectory {
        &self.definitions
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        &self.application_groups
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBlockRecordValueEntry] {
        &self.records
    }

    #[must_use]
    pub fn values(&self) -> &[DxfBlockRecordValue] {
        &self.values
    }

    #[must_use]
    pub fn record_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfBlockRecordValueEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.definition().block_record().ordinal()
            })
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn values_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBlockRecordValue]> {
        let entry = self.record_for_block_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.value_range().start()).ok()?;
        let end = usize::try_from(entry.value_range().end()).ok()?;
        self.values.get(start..end)
    }

    #[must_use]
    pub fn value_for_group(&self, occurrence: u64) -> Option<DxfBlockRecordValue> {
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
    pub fn block_record_value_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordValueDirectory, DxfError> {
        DxfBlockRecordValueDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_record_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_record_value_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_record_value_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockRecordValueDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_record_value_directory(cancellation)
    }
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfBlockRecordValueRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfBlockRecordValueData, DxfBlockRecordValueIssue>, DxfError> {
    match wire_kind(role) {
        DxfBlockRecordWireKind::Text => {
            let group_occurrence =
                u32::try_from(group.occurrence()).map_err(|_| invalid_internal_data())?;
            Ok(Ok(DxfBlockRecordValueData::Text(DxfBlockRecordTextValue {
                source_id: document.source_id(),
                group_occurrence,
                value_span: group.value_payload_span(),
                encoding: document.text_encoding_report().resolution(),
            })))
        }
        DxfBlockRecordWireKind::Double => {
            decode_raw_double(document, group, cancellation).map(|value| {
                value
                    .map(DxfBlockRecordValueData::Double)
                    .map_err(DxfBlockRecordValueIssue::InvalidAsciiNumber)
            })
        }
        DxfBlockRecordWireKind::Int16 => {
            decode_raw_i16(document, group, cancellation).map(|value| {
                value
                    .map(DxfBlockRecordValueData::Int16)
                    .map_err(DxfBlockRecordValueIssue::InvalidAsciiNumber)
            })
        }
    }
}

#[derive(Clone, Copy)]
enum DxfBlockRecordWireKind {
    Text,
    Double,
    Int16,
}

const fn wire_kind(role: DxfBlockRecordValueRole) -> DxfBlockRecordWireKind {
    match role {
        DxfBlockRecordValueRole::PrimaryName
        | DxfBlockRecordValueRole::SecondaryName
        | DxfBlockRecordValueRole::XrefPath
        | DxfBlockRecordValueRole::Description => DxfBlockRecordWireKind::Text,
        DxfBlockRecordValueRole::BasePointX
        | DxfBlockRecordValueRole::BasePointY
        | DxfBlockRecordValueRole::BasePointZ => DxfBlockRecordWireKind::Double,
        DxfBlockRecordValueRole::Flags => DxfBlockRecordWireKind::Int16,
    }
}

const fn value_role(group_code: i16) -> Option<DxfBlockRecordValueRole> {
    use DxfBlockRecordValueRole::{
        BasePointX, BasePointY, BasePointZ, Description, Flags, PrimaryName, SecondaryName,
        XrefPath,
    };
    match group_code {
        2 => Some(PrimaryName),
        70 => Some(Flags),
        10 => Some(BasePointX),
        20 => Some(BasePointY),
        30 => Some(BasePointZ),
        3 => Some(SecondaryName),
        1 => Some(XrefPath),
        4 => Some(Description),
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

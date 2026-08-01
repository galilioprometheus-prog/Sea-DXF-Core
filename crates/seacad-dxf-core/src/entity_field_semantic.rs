//! Typed common-entity-field semantics over generated evidence and cards.

use std::io;

use crate::{
    ByteSpan, DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfDouble, DxfEntityField, DxfEntityFieldCard, DxfEntityFieldCardMember,
    DxfEntityFieldCardState, DxfEntityFieldDefault, DxfEntityFieldEvidenceDirectory,
    DxfEntityFieldOccurrence, DxfEntityFieldWireType, DxfEntityRef, DxfError, DxfHandle,
    DxfHandleParseIssue, DxfIoOperation, DxfRawDocumentView, DxfRawValueProvenance,
    DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId, DxfTextEncodingResolution,
    DxfTextValueDecodeReceipt,
    raw_double::decode_raw_double,
    raw_handle::parse_raw_group_handle,
    raw_integer::{decode_raw_i16, decode_raw_i32},
};

const ENTITY_COMMON_NAMESPACE: &str = "entity_common";

/// Source-backed exact text from one common entity field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityFieldTextValue {
    source_id: DxfSourceId,
    group_occurrence: u32,
    value_span: ByteSpan,
    encoding: DxfTextEncodingResolution,
}

impl DxfEntityFieldTextValue {
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
        ensure_source(self.source_id, document.source_id())?;
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

/// Usable value of one singleton common field without merging wire domains.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldValue {
    Double(DxfDouble),
    Int16(i16),
    Int32(i32),
    Handle(DxfHandle),
    ExactText(DxfEntityFieldTextValue),
    SchemaExactText(&'static str),
    ByLayer,
}

/// Exact reason a singleton common field has no usable semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldSemanticIssue {
    MissingRequired,
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    InvalidHandle(DxfHandleParseIssue),
    NonFiniteDouble(DxfDouble),
}

pub type DxfEntityFieldSemanticValue =
    DxfSemanticValue<DxfEntityFieldValue, DxfEntityFieldSemanticIssue>;

/// Common fields are either typed singletons or exact opaque sequences.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldSemantics {
    Singleton(DxfEntityFieldSemanticValue),
    OpaqueSequence { occurrence_count: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityFieldSemanticEntry {
    ordinal: u32,
    card: DxfEntityFieldCard,
    semantics: DxfEntityFieldSemantics,
}

impl DxfEntityFieldSemanticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.card.entity()
    }

    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.card.field()
    }

    #[must_use]
    pub const fn card(self) -> DxfEntityFieldCard {
        self.card
    }

    #[must_use]
    pub const fn semantics(self) -> DxfEntityFieldSemantics {
        self.semantics
    }
}

#[derive(Debug)]
pub struct DxfEntityFieldSemanticDirectory {
    source_id: DxfSourceId,
    evidence: DxfEntityFieldEvidenceDirectory,
    entries: Box<[DxfEntityFieldSemanticEntry]>,
}

impl DxfEntityFieldSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(
            document.source_id(),
            document.text_encoding_report().source_id(),
        )?;
        let evidence = document.entity_field_evidence_directory(cancellation)?;
        ensure_source(document.source_id(), evidence.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(evidence.cards().len())
            .map_err(|_| out_of_memory())?;
        for card in evidence.cards().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfEntityFieldSemanticEntry {
                ordinal: compact_len(entries.len())?,
                card,
                semantics: project_card(document, &evidence, card, cancellation)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            evidence,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn evidence_directory(&self) -> &DxfEntityFieldEvidenceDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityFieldSemanticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityFieldSemanticEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityFieldSemanticEntry], DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        let raw = entity.record().ordinal();
        let start = self
            .entries
            .partition_point(|entry| entry.entity().record().ordinal() < raw);
        let end = self
            .entries
            .partition_point(|entry| entry.entity().record().ordinal() <= raw);
        self.entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }

    pub fn entry_for_field(
        &self,
        entity: DxfEntityRef,
        field: DxfEntityField,
    ) -> Result<Option<DxfEntityFieldSemanticEntry>, DxfError> {
        Ok(self
            .entries_for_entity(entity)?
            .iter()
            .copied()
            .find(|entry| entry.field() == field))
    }

    pub fn members_for_opaque_sequence(
        &self,
        entry: DxfEntityFieldSemanticEntry,
    ) -> Result<Option<&[DxfEntityFieldCardMember]>, DxfError> {
        ensure_source(self.source_id, entry.entity().source_id())?;
        if !matches!(
            entry.semantics(),
            DxfEntityFieldSemantics::OpaqueSequence { .. }
        ) {
            return Ok(None);
        }
        self.evidence.members_for_card(entry.card()).map(Some)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_field_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldSemanticDirectory, DxfError> {
        DxfEntityFieldSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_field_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_field_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_field_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_field_semantic_directory(cancellation)
    }
}

fn project_card(
    document: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    card: DxfEntityFieldCard,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityFieldSemantics, DxfError> {
    let descriptor = card
        .field()
        .descriptor()
        .ok_or_else(invalid_internal_data)?;
    if descriptor.wire_type() == DxfEntityFieldWireType::BinaryChunk {
        let occurrence_count = match card.state() {
            DxfEntityFieldCardState::AbsentOptional => 0,
            DxfEntityFieldCardState::Sequence { occurrence_count } => occurrence_count,
            _ => return Err(invalid_internal_data()),
        };
        return Ok(DxfEntityFieldSemantics::OpaqueSequence { occurrence_count });
    }
    let field = DxfSemanticFieldProvenance::new(
        document.source_id(),
        ENTITY_COMMON_NAMESPACE,
        descriptor.id(),
    );
    let value = match card.state() {
        DxfEntityFieldCardState::AbsentRequired => {
            DxfSemanticValue::invalid(DxfEntityFieldSemanticIssue::MissingRequired, field, None)
        }
        DxfEntityFieldCardState::AbsentOptional => default_or_absent(descriptor.default(), field),
        DxfEntityFieldCardState::Duplicate { occurrence_count } => DxfSemanticValue::invalid(
            DxfEntityFieldSemanticIssue::MultipleValues { occurrence_count },
            field,
            None,
        ),
        DxfEntityFieldCardState::Unique => {
            let occurrence = unique_occurrence(evidence, card)?;
            let raw = raw_provenance(occurrence)?;
            match decode_value(document, occurrence, cancellation)? {
                Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                Err(issue) => DxfSemanticValue::invalid(issue, field, Some(raw)),
            }
        }
        DxfEntityFieldCardState::Sequence { .. } => return Err(invalid_internal_data()),
    };
    Ok(DxfEntityFieldSemantics::Singleton(value))
}

fn default_or_absent(
    default: DxfEntityFieldDefault,
    field: DxfSemanticFieldProvenance,
) -> DxfEntityFieldSemanticValue {
    match default {
        DxfEntityFieldDefault::None => DxfSemanticValue::absent(field),
        DxfEntityFieldDefault::Int16(value) => {
            DxfSemanticValue::defaulted(DxfEntityFieldValue::Int16(value), field)
        }
        DxfEntityFieldDefault::DoubleBits(bits) => DxfSemanticValue::defaulted(
            DxfEntityFieldValue::Double(DxfDouble::from_bits(bits)),
            field,
        ),
        DxfEntityFieldDefault::ExactText(value) => {
            DxfSemanticValue::defaulted(DxfEntityFieldValue::SchemaExactText(value), field)
        }
        DxfEntityFieldDefault::ByLayer => {
            DxfSemanticValue::defaulted(DxfEntityFieldValue::ByLayer, field)
        }
    }
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    occurrence: DxfEntityFieldOccurrence,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfEntityFieldValue, DxfEntityFieldSemanticIssue>, DxfError> {
    let descriptor = occurrence
        .field()
        .descriptor()
        .ok_or_else(invalid_internal_data)?;
    let group = occurrence.group();
    Ok(match descriptor.wire_type() {
        DxfEntityFieldWireType::Double => match decode_raw_double(document, group, cancellation)? {
            Ok(value) if value.is_finite() => Ok(DxfEntityFieldValue::Double(value)),
            Ok(value) => Err(DxfEntityFieldSemanticIssue::NonFiniteDouble(value)),
            Err(issue) => Err(DxfEntityFieldSemanticIssue::InvalidAsciiNumber(issue)),
        },
        DxfEntityFieldWireType::Int16 => decode_raw_i16(document, group, cancellation)?
            .map(DxfEntityFieldValue::Int16)
            .map_err(DxfEntityFieldSemanticIssue::InvalidAsciiNumber),
        DxfEntityFieldWireType::Int32 => decode_raw_i32(document, group, cancellation)?
            .map(DxfEntityFieldValue::Int32)
            .map_err(DxfEntityFieldSemanticIssue::InvalidAsciiNumber),
        DxfEntityFieldWireType::Handle => parse_raw_group_handle(document, group, cancellation)?
            .map(DxfEntityFieldValue::Handle)
            .map_err(DxfEntityFieldSemanticIssue::InvalidHandle),
        DxfEntityFieldWireType::ExactText => {
            Ok(DxfEntityFieldValue::ExactText(DxfEntityFieldTextValue {
                source_id: document.source_id(),
                group_occurrence: u32::try_from(group.occurrence())
                    .map_err(|_| invalid_internal_data())?,
                value_span: group.value_payload_span(),
                encoding: document.text_encoding_report().resolution(),
            }))
        }
        DxfEntityFieldWireType::BinaryChunk => return Err(invalid_internal_data()),
    })
}

fn unique_occurrence(
    evidence: &DxfEntityFieldEvidenceDirectory,
    card: DxfEntityFieldCard,
) -> Result<DxfEntityFieldOccurrence, DxfError> {
    let [member] = evidence.members_for_card(card)? else {
        return Err(invalid_internal_data());
    };
    evidence
        .occurrence_for_member(*member)
        .ok_or_else(invalid_internal_data)
}

fn raw_provenance(occurrence: DxfEntityFieldOccurrence) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        occurrence.group().occurrence(),
        occurrence.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
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
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

//! Lazy unique-field selection over typed DIMSTYLE cardinality evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDimStyleField, DxfDimStyleFieldCard, DxfDimStyleFieldCardDirectory,
    DxfDimStyleFieldCardState, DxfDimStyleValue, DxfDimStyleValueData, DxfDimStyleValueEntry,
    DxfDimStyleValueIssue, DxfError, DxfHandleParseIssue, DxfIoOperation, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    dimstyle_field::field_for_group_code,
};

const NAMESPACE: &str = "dimstyle.record";

/// Why one DIMSTYLE field has no uniquely usable typed value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfDimStyleSemanticIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    InvalidHandle(DxfHandleParseIssue),
    MultipleValues { occurrence_count: u32 },
}

/// Source-anchored selected state for one DIMSTYLE-specific field.
pub type DxfDimStyleSemanticValue =
    DxfSemanticValue<DxfDimStyleValueData, DxfDimStyleSemanticIssue>;

/// Exact signed DIMSTYLE standard flags with reviewed bit helpers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDimStyleStandardFlags(i16);

impl DxfDimStyleStandardFlags {
    const KNOWN_BITS: u16 = 16 | 32 | 64;

    #[must_use]
    pub const fn from_raw(raw: i16) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> i16 {
        self.0
    }

    #[must_use]
    pub const fn is_externally_dependent(self) -> bool {
        self.0 & 16 != 0
    }

    #[must_use]
    pub const fn is_resolved_external_reference_or_dependent(self) -> bool {
        self.0 & 32 != 0
    }

    #[must_use]
    pub const fn is_referenced_external_reference(self) -> bool {
        self.0 & 64 != 0
    }

    #[must_use]
    pub const fn unknown_bits(self) -> u16 {
        self.0 as u16 & !Self::KNOWN_BITS
    }
}

/// Source-anchored selected state for DIMSTYLE group-70 standard flags.
pub type DxfDimStyleStandardFlagsSemantic =
    DxfSemanticValue<DxfDimStyleStandardFlags, DxfDimStyleSemanticIssue>;

/// Immutable lazy DIMSTYLE semantics retaining every lower evidence layer.
#[derive(Debug)]
pub struct DxfDimStyleSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfDimStyleFieldCardDirectory,
}

impl DxfDimStyleSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.dimstyle_field_card_directory(cancellation)?;
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
    pub const fn card_directory(&self) -> &DxfDimStyleFieldCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn records(&self) -> &[DxfDimStyleValueEntry] {
        self.cards.evidence_directory().records()
    }

    pub fn semantic_for_raw_ordinal(
        &self,
        raw_ordinal: u64,
        field: DxfDimStyleField,
    ) -> Result<Option<DxfDimStyleSemanticValue>, DxfError> {
        let Some(record) = self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_ordinal)
        else {
            return Ok(None);
        };
        select_field(&self.cards, record, field).map(Some)
    }

    pub fn semantic_for_entry(
        &self,
        record: DxfDimStyleValueEntry,
        field: DxfDimStyleField,
    ) -> Result<Option<DxfDimStyleSemanticValue>, DxfError> {
        let raw_ordinal = record.table_entry().record().ordinal();
        if self
            .cards
            .evidence_directory()
            .record_for_raw_ordinal(raw_ordinal)
            != Some(record)
        {
            return Ok(None);
        }
        select_field(&self.cards, record, field).map(Some)
    }

    pub fn standard_flags_for_raw_ordinal(
        &self,
        raw_ordinal: u64,
    ) -> Result<Option<DxfDimStyleStandardFlagsSemantic>, DxfError> {
        let Some(field) = field_for_group_code(70) else {
            return Err(invalid_internal_data());
        };
        self.semantic_for_raw_ordinal(raw_ordinal, field)?
            .map(project_standard_flags)
            .transpose()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn dimstyle_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleSemanticDirectory, DxfError> {
        DxfDimStyleSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn dimstyle_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn dimstyle_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfDimStyleSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).dimstyle_semantic_directory(cancellation)
    }
}

fn select_field(
    cards: &DxfDimStyleFieldCardDirectory,
    record: DxfDimStyleValueEntry,
    field: DxfDimStyleField,
) -> Result<DxfDimStyleSemanticValue, DxfError> {
    let provenance = DxfSemanticFieldProvenance::new(cards.source_id(), NAMESPACE, field.name());
    let card = cards
        .card_for_field(record.table_entry().record().ordinal(), field)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfDimStyleFieldCardState::Absent => Ok(DxfSemanticValue::absent(provenance)),
        DxfDimStyleFieldCardState::Unique => {
            let value = unique_value(cards, card)?;
            let raw = value_provenance(value)?;
            Ok(match value.value() {
                Ok(value) => DxfSemanticValue::explicit(value, provenance, raw),
                Err(issue) => {
                    DxfSemanticValue::invalid(semantic_issue(issue), provenance, Some(raw))
                }
            })
        }
        DxfDimStyleFieldCardState::Multiple { occurrence_count } => {
            let first = first_value(cards, card)?;
            Ok(DxfSemanticValue::invalid(
                DxfDimStyleSemanticIssue::MultipleValues { occurrence_count },
                provenance,
                Some(value_provenance(first)?),
            ))
        }
    }
}

fn project_standard_flags(
    value: DxfDimStyleSemanticValue,
) -> Result<DxfDimStyleStandardFlagsSemantic, DxfError> {
    Ok(match value {
        DxfSemanticValue::Explicit {
            value: DxfDimStyleValueData::Int16(value),
            field,
            raw,
        } => DxfSemanticValue::explicit(DxfDimStyleStandardFlags::from_raw(value), field, raw),
        DxfSemanticValue::Explicit { .. } | DxfSemanticValue::Defaulted { .. } => {
            return Err(invalid_internal_data());
        }
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(issue, field, raw)
        }
    })
}

fn semantic_issue(issue: DxfDimStyleValueIssue) -> DxfDimStyleSemanticIssue {
    match issue {
        DxfDimStyleValueIssue::InvalidAsciiNumber(issue) => {
            DxfDimStyleSemanticIssue::InvalidAsciiNumber(issue)
        }
        DxfDimStyleValueIssue::InvalidHandle(issue) => {
            DxfDimStyleSemanticIssue::InvalidHandle(issue)
        }
    }
}

fn unique_value(
    cards: &DxfDimStyleFieldCardDirectory,
    card: DxfDimStyleFieldCard,
) -> Result<DxfDimStyleValue, DxfError> {
    let [member] = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    cards
        .value_for_member(*member)
        .ok_or_else(invalid_internal_data)
}

fn first_value(
    cards: &DxfDimStyleFieldCardDirectory,
    card: DxfDimStyleFieldCard,
) -> Result<DxfDimStyleValue, DxfError> {
    cards
        .members_for_card(card.ordinal())
        .and_then(|members| members.first())
        .copied()
        .and_then(|member| cards.value_for_member(member))
        .ok_or_else(invalid_internal_data)
}

fn value_provenance(value: DxfDimStyleValue) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        value.group().occurrence(),
        value.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
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

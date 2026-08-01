//! Common entity exact-text and exact symbol-name semantics.

use std::io;

use crate::{
    ByteSpan, DXF_ENTITY_COMMON_FIELDS, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfEntityField, DxfEntityFieldSemanticDirectory,
    DxfEntityFieldSemanticEntry, DxfEntityFieldSemanticIssue, DxfEntityFieldSemanticValue,
    DxfEntityFieldSemantics, DxfEntityFieldTextValue, DxfEntityFieldValue, DxfEntityRef, DxfError,
    DxfIoOperation, DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry,
    DxfNamedSymbolTableKind, DxfRawDocumentView, DxfSemanticValue, DxfSourceId,
    source_span::{sha256_span, spans_equal},
};

type NameDigest = [u8; 32];

/// Usable exact symbol reference or the reviewed linetype default.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonSymbolValue {
    ByLayer,
    ExactMatch {
        text: DxfEntityFieldTextValue,
        target: DxfNamedSymbolTableEntry,
    },
}

/// Raw-field invalidity or exact symbol lookup failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonSymbolIssue {
    Field(DxfEntityFieldSemanticIssue),
    Missing,
    Ambiguous { target_count: u32 },
}

pub type DxfEntityCommonSymbolSemanticValue =
    DxfSemanticValue<DxfEntityCommonSymbolValue, DxfEntityCommonSymbolIssue>;

/// Reviewed symbol reference or exact pass-through text whose policy is open.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonTextSemantics {
    Symbol(DxfEntityCommonSymbolSemanticValue),
    ExactUnreviewed(DxfEntityFieldSemanticValue),
}

/// One of the four exact-text fields in the generated common-field schema.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonTextEntry {
    ordinal: u32,
    source: DxfEntityFieldSemanticEntry,
    semantics: DxfEntityCommonTextSemantics,
}

impl DxfEntityCommonTextEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.source.entity()
    }

    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.source.field()
    }

    #[must_use]
    pub const fn source_semantics(self) -> DxfEntityFieldSemanticEntry {
        self.source
    }

    #[must_use]
    pub const fn semantics(self) -> DxfEntityCommonTextSemantics {
        self.semantics
    }
}

#[derive(Clone, Copy)]
struct NameIndexEntry {
    kind: u8,
    digest: NameDigest,
    target: DxfNamedSymbolTableEntry,
}

/// Source-bound projection for common entity exact-text fields.
#[derive(Debug)]
pub struct DxfEntityCommonTextDirectory {
    source_id: DxfSourceId,
    source: DxfEntityFieldSemanticDirectory,
    named: DxfNamedSymbolTableDirectory,
    entries: Box<[DxfEntityCommonTextEntry]>,
}

impl DxfEntityCommonTextDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source = document.entity_field_semantic_directory(cancellation)?;
        let named = document.named_symbol_table_directory(cancellation)?;
        for observed in [source.source_id(), named.source_id()] {
            ensure_source(document.source_id(), observed)?;
        }
        let index = build_name_index(document, &named, cancellation)?;
        let entity_count = source.entries().len() / DXF_ENTITY_COMMON_FIELDS.len();
        let capacity = entity_count
            .checked_mul(4)
            .ok_or_else(invalid_internal_data)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(capacity)
            .map_err(|_| out_of_memory())?;
        for source_entry in source.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !is_common_text_field(source_entry.field()) {
                continue;
            }
            let semantics = match symbol_kind(source_entry.field()) {
                Some(kind) => DxfEntityCommonTextSemantics::Symbol(project_symbol(
                    document,
                    &index,
                    source_entry,
                    kind,
                    cancellation,
                )?),
                None => project_exact_unreviewed(source_entry)?,
            };
            entries.push(DxfEntityCommonTextEntry {
                ordinal: compact_len(entries.len())?,
                source: source_entry,
                semantics,
            });
        }
        if entries.len() != capacity {
            return Err(invalid_internal_data());
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            source,
            named,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_directory(&self) -> &DxfEntityFieldSemanticDirectory {
        &self.source
    }

    #[must_use]
    pub const fn named_symbol_table_directory(&self) -> &DxfNamedSymbolTableDirectory {
        &self.named
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityCommonTextEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityCommonTextEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityCommonTextEntry], DxfError> {
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
    ) -> Result<Option<DxfEntityCommonTextEntry>, DxfError> {
        if !is_common_text_field(field) {
            return Ok(None);
        }
        Ok(self
            .entries_for_entity(entity)?
            .iter()
            .copied()
            .find(|entry| entry.field() == field))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_common_text_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonTextDirectory, DxfError> {
        DxfEntityCommonTextDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_common_text_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonTextDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_text_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_common_text_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonTextDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_text_directory(cancellation)
    }
}

fn build_name_index(
    document: DxfRawDocumentView<'_>,
    named: &DxfNamedSymbolTableDirectory,
    cancellation: &DxfCancellationToken,
) -> Result<Box<[NameIndexEntry]>, DxfError> {
    let mut index = Vec::new();
    index
        .try_reserve_exact(named.entries().len())
        .map_err(|_| out_of_memory())?;
    for target in named.entries().iter().copied() {
        ensure_not_cancelled(cancellation)?;
        index.push(NameIndexEntry {
            kind: kind_ordinal(target.kind()),
            digest: sha256_span(document, target.name().value_span(), cancellation)?,
            target,
        });
    }
    index.sort_unstable_by_key(|entry| (entry.kind, entry.digest, entry.target.record().ordinal()));
    Ok(index.into_boxed_slice())
}

fn project_symbol(
    document: DxfRawDocumentView<'_>,
    index: &[NameIndexEntry],
    source: DxfEntityFieldSemanticEntry,
    kind: DxfNamedSymbolTableKind,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityCommonSymbolSemanticValue, DxfError> {
    let DxfEntityFieldSemantics::Singleton(value) = source.semantics() else {
        return Err(invalid_internal_data());
    };
    Ok(match value {
        DxfSemanticValue::Explicit {
            value: DxfEntityFieldValue::ExactText(text),
            field,
            raw,
        } => {
            let (target, count) =
                exact_matches(document, index, kind, text.value_span(), cancellation)?;
            match (target, count) {
                (None, 0) => {
                    DxfSemanticValue::invalid(DxfEntityCommonSymbolIssue::Missing, field, Some(raw))
                }
                (Some(target), 1) => DxfSemanticValue::explicit(
                    DxfEntityCommonSymbolValue::ExactMatch { text, target },
                    field,
                    raw,
                ),
                (Some(_), target_count) => DxfSemanticValue::invalid(
                    DxfEntityCommonSymbolIssue::Ambiguous { target_count },
                    field,
                    Some(raw),
                ),
                (None, _) => return Err(invalid_internal_data()),
            }
        }
        DxfSemanticValue::Defaulted {
            value: DxfEntityFieldValue::SchemaExactText("BYLAYER"),
            field,
        } if source.field() == DxfEntityField::LINETYPE => {
            DxfSemanticValue::defaulted(DxfEntityCommonSymbolValue::ByLayer, field)
        }
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(DxfEntityCommonSymbolIssue::Field(issue), field, raw)
        }
        DxfSemanticValue::Explicit { .. } | DxfSemanticValue::Defaulted { .. } => {
            return Err(invalid_internal_data());
        }
    })
}

fn exact_matches(
    document: DxfRawDocumentView<'_>,
    index: &[NameIndexEntry],
    kind: DxfNamedSymbolTableKind,
    span: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<(Option<DxfNamedSymbolTableEntry>, u32), DxfError> {
    let kind = kind_ordinal(kind);
    let digest = sha256_span(document, span, cancellation)?;
    let start = index.partition_point(|entry| (entry.kind, entry.digest) < (kind, digest));
    let end = index.partition_point(|entry| (entry.kind, entry.digest) <= (kind, digest));
    let mut first = None;
    let mut count = 0_u32;
    for candidate in index.get(start..end).ok_or_else(invalid_internal_data)? {
        ensure_not_cancelled(cancellation)?;
        if spans_equal(
            document,
            span,
            candidate.target.name().value_span(),
            cancellation,
        )? {
            count = count.checked_add(1).ok_or_else(invalid_internal_data)?;
            first.get_or_insert(candidate.target);
        }
    }
    Ok((first, count))
}

fn project_exact_unreviewed(
    source: DxfEntityFieldSemanticEntry,
) -> Result<DxfEntityCommonTextSemantics, DxfError> {
    let DxfEntityFieldSemantics::Singleton(value) = source.semantics() else {
        return Err(invalid_internal_data());
    };
    if matches!(
        value.value(),
        Some(DxfEntityFieldValue::ExactText(_))
            | Some(DxfEntityFieldValue::SchemaExactText(_))
            | None
    ) {
        Ok(DxfEntityCommonTextSemantics::ExactUnreviewed(value))
    } else {
        Err(invalid_internal_data())
    }
}

fn symbol_kind(field: DxfEntityField) -> Option<DxfNamedSymbolTableKind> {
    if field == DxfEntityField::LAYER {
        Some(DxfNamedSymbolTableKind::Layer)
    } else if field == DxfEntityField::LINETYPE {
        Some(DxfNamedSymbolTableKind::Linetype)
    } else {
        None
    }
}

fn is_common_text_field(field: DxfEntityField) -> bool {
    field == DxfEntityField::LAYOUT
        || field == DxfEntityField::LAYER
        || field == DxfEntityField::LINETYPE
        || field == DxfEntityField::COLOR_NAME
}

const fn kind_ordinal(kind: DxfNamedSymbolTableKind) -> u8 {
    match kind {
        DxfNamedSymbolTableKind::DimStyle => 0,
        DxfNamedSymbolTableKind::Style => 1,
        DxfNamedSymbolTableKind::BlockRecord => 2,
        DxfNamedSymbolTableKind::Layer => 3,
        DxfNamedSymbolTableKind::Linetype => 4,
    }
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

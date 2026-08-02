//! Source-bound admission for common entity symbol-name edits.

use std::io;

use crate::entity_common_text_semantic::reviewed_common_symbol_kind;
use crate::source_span::span_equals_bytes;
use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityEditValue,
    DxfEntityEditValueKind, DxfEntityField, DxfError, DxfIoOperation, DxfNamedSymbolTableDirectory,
    DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind, DxfRawDocumentView, DxfSourceId,
};

/// One exact common symbol name admitted against a same-document table entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonSymbolEditValue {
    field: DxfEntityField,
    kind: DxfNamedSymbolTableKind,
    target: DxfNamedSymbolTableEntry,
}

impl DxfEntityCommonSymbolEditValue {
    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn kind(self) -> DxfNamedSymbolTableKind {
        self.kind
    }

    #[must_use]
    pub const fn target(self) -> DxfNamedSymbolTableEntry {
        self.target
    }
}

/// Why a common exact-text edit cannot enter generic singleton planning.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonSymbolEditIssue {
    ColorBookResolutionRequired {
        field: DxfEntityField,
    },
    ValueKindMismatch {
        field: DxfEntityField,
        expected: DxfEntityEditValueKind,
        observed: DxfEntityEditValueKind,
    },
    Missing {
        field: DxfEntityField,
        kind: DxfNamedSymbolTableKind,
    },
    Ambiguous {
        field: DxfEntityField,
        kind: DxfNamedSymbolTableKind,
        target_count: u32,
    },
}

/// Classification of an explicit edit against reviewed common-symbol rules.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonSymbolEditOutcome {
    NotSymbol { field: DxfEntityField },
    Valid(DxfEntityCommonSymbolEditValue),
    Invalid(DxfEntityCommonSymbolEditIssue),
}

impl DxfRawDocumentView<'_> {
    /// Resolves one proposed exact name without changing the source document.
    pub fn classify_entity_common_symbol_edit(
        self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonSymbolEditOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let named = if reviewed_common_symbol_kind(field).is_some()
            && matches!(value, DxfEntityEditValue::ExactRawText(_))
        {
            Some(self.named_symbol_table_directory(cancellation)?)
        } else {
            None
        };
        classify_with_symbols(self, named.as_ref(), field, value, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn classify_entity_common_symbol_edit(
        &self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonSymbolEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_symbol_edit(
            field,
            value,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn classify_entity_common_symbol_edit(
        &self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonSymbolEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_symbol_edit(
            field,
            value,
            cancellation,
        )
    }
}

pub(crate) fn classify_with_symbols(
    document: DxfRawDocumentView<'_>,
    named: Option<&DxfNamedSymbolTableDirectory>,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityCommonSymbolEditOutcome, DxfError> {
    ensure_not_cancelled(cancellation)?;
    if field == DxfEntityField::COLOR_NAME {
        return Ok(DxfEntityCommonSymbolEditOutcome::Invalid(
            DxfEntityCommonSymbolEditIssue::ColorBookResolutionRequired { field },
        ));
    }
    let Some(kind) = reviewed_common_symbol_kind(field) else {
        return Ok(DxfEntityCommonSymbolEditOutcome::NotSymbol { field });
    };
    let DxfEntityEditValue::ExactRawText(exact_name) = value else {
        return Ok(DxfEntityCommonSymbolEditOutcome::Invalid(
            DxfEntityCommonSymbolEditIssue::ValueKindMismatch {
                field,
                expected: DxfEntityEditValueKind::ExactRawText,
                observed: value.kind(),
            },
        ));
    };
    let named = named.ok_or_else(invalid_internal_data)?;
    ensure_source(document.source_id(), named.source_id())?;
    let mut first = None;
    let mut count = 0_u32;
    for target in named.entries().iter().copied() {
        ensure_not_cancelled(cancellation)?;
        if target.kind() != kind
            || !span_equals_bytes(
                document,
                target.name().value_span(),
                exact_name,
                cancellation,
            )?
        {
            continue;
        }
        count = count.checked_add(1).ok_or_else(invalid_internal_data)?;
        first.get_or_insert(target);
    }
    match (first, count) {
        (None, 0) => Ok(DxfEntityCommonSymbolEditOutcome::Invalid(
            DxfEntityCommonSymbolEditIssue::Missing { field, kind },
        )),
        (Some(target), 1) => Ok(DxfEntityCommonSymbolEditOutcome::Valid(
            DxfEntityCommonSymbolEditValue {
                field,
                kind,
                target,
            },
        )),
        (Some(_), target_count) => Ok(DxfEntityCommonSymbolEditOutcome::Invalid(
            DxfEntityCommonSymbolEditIssue::Ambiguous {
                field,
                kind,
                target_count,
            },
        )),
        (None, _) => Err(invalid_internal_data()),
    }
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
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

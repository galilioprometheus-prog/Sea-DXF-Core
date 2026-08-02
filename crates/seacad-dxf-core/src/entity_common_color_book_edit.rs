//! Source-bound admission for common entity color-book edits.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityCommonFieldDomainDirectory, DxfEntityCommonFieldDomainSemanticValue,
    DxfEntityCommonFieldDomainSemantics, DxfEntityCommonFieldDomainValue, DxfEntityEditValue,
    DxfEntityEditValueKind, DxfEntityField, DxfEntityIndexedColor, DxfEntityRef,
    DxfEntityTrueColor, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// One syntactically valid color-book name admitted against usable color fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonColorBookEditValue {
    field: DxfEntityField,
    separator_offset: usize,
    proposed_len: usize,
    indexed_color: DxfEntityIndexedColor,
    true_color: DxfEntityTrueColor,
}

impl DxfEntityCommonColorBookEditValue {
    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub fn book_name(self, proposed: &[u8]) -> Option<&[u8]> {
        (proposed.len() == self.proposed_len).then(|| proposed.get(..self.separator_offset))?
    }

    #[must_use]
    pub fn color_name(self, proposed: &[u8]) -> Option<&[u8]> {
        (proposed.len() == self.proposed_len)
            .then(|| proposed.get(self.separator_offset.checked_add(1)?..))?
    }

    #[must_use]
    pub const fn indexed_color(self) -> DxfEntityIndexedColor {
        self.indexed_color
    }

    #[must_use]
    pub const fn true_color(self) -> DxfEntityTrueColor {
        self.true_color
    }
}

/// Why a common color-book edit cannot enter singleton planning.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonColorBookEditIssue {
    ValueKindMismatch {
        field: DxfEntityField,
        expected: DxfEntityEditValueKind,
        observed: DxfEntityEditValueKind,
    },
    MissingSeparator {
        field: DxfEntityField,
    },
    EmptyBookName {
        field: DxfEntityField,
    },
    EmptyColorName {
        field: DxfEntityField,
    },
    MultipleSeparators {
        field: DxfEntityField,
        separator_count: u64,
    },
    RelatedColor {
        field: DxfEntityField,
        related_field: DxfEntityField,
        semantics: DxfEntityCommonFieldDomainSemanticValue,
    },
}

/// Classification of an explicit edit against reviewed color-book rules.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonColorBookEditOutcome {
    NotColorBook { field: DxfEntityField },
    Valid(DxfEntityCommonColorBookEditValue),
    Invalid(DxfEntityCommonColorBookEditIssue),
}

impl DxfRawDocumentView<'_> {
    /// Validates a proposed group 430 against one same-document entity.
    pub fn classify_entity_common_color_book_edit(
        self,
        entity: DxfEntityRef,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonColorBookEditOutcome, DxfError> {
        ensure_source(self.source_id(), entity.source_id())?;
        ensure_not_cancelled(cancellation)?;
        let domains = if field == DxfEntityField::COLOR_NAME
            && matches!(value, DxfEntityEditValue::ExactRawText(_))
        {
            Some(self.entity_common_field_domain_directory(cancellation)?)
        } else {
            None
        };
        classify_with_color_domains(domains.as_ref(), Some(entity), field, value, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn classify_entity_common_color_book_edit(
        &self,
        entity: DxfEntityRef,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonColorBookEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_color_book_edit(
            entity,
            field,
            value,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn classify_entity_common_color_book_edit(
        &self,
        entity: DxfEntityRef,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonColorBookEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_color_book_edit(
            entity,
            field,
            value,
            cancellation,
        )
    }
}

pub(crate) fn classify_with_color_domains(
    domains: Option<&DxfEntityCommonFieldDomainDirectory>,
    entity: Option<DxfEntityRef>,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityCommonColorBookEditOutcome, DxfError> {
    ensure_not_cancelled(cancellation)?;
    if field != DxfEntityField::COLOR_NAME {
        return Ok(DxfEntityCommonColorBookEditOutcome::NotColorBook { field });
    }
    let DxfEntityEditValue::ExactRawText(proposed) = value else {
        return Ok(DxfEntityCommonColorBookEditOutcome::Invalid(
            DxfEntityCommonColorBookEditIssue::ValueKindMismatch {
                field,
                expected: DxfEntityEditValueKind::ExactRawText,
                observed: value.kind(),
            },
        ));
    };
    let separator_offset = match split_proposed(proposed, cancellation)? {
        Ok(offset) => offset,
        Err(issue) => return Ok(DxfEntityCommonColorBookEditOutcome::Invalid(issue)),
    };
    let domains = domains.ok_or_else(invalid_internal_data)?;
    let entity = entity.ok_or_else(invalid_internal_data)?;
    ensure_source(domains.source_id(), entity.source_id())?;
    let true_color = related_color(domains, entity, DxfEntityField::TRUE_COLOR)?;
    let indexed_color = related_color(domains, entity, DxfEntityField::COLOR)?;
    match (true_color, indexed_color) {
        (
            Ok(DxfEntityCommonFieldDomainValue::TrueColor(true_color)),
            Ok(DxfEntityCommonFieldDomainValue::IndexedColor(indexed_color)),
        ) => Ok(DxfEntityCommonColorBookEditOutcome::Valid(
            DxfEntityCommonColorBookEditValue {
                field,
                separator_offset,
                proposed_len: proposed.len(),
                indexed_color,
                true_color,
            },
        )),
        (Err(semantics), _) => Ok(DxfEntityCommonColorBookEditOutcome::Invalid(
            DxfEntityCommonColorBookEditIssue::RelatedColor {
                field,
                related_field: DxfEntityField::TRUE_COLOR,
                semantics,
            },
        )),
        (_, Err(semantics)) => Ok(DxfEntityCommonColorBookEditOutcome::Invalid(
            DxfEntityCommonColorBookEditIssue::RelatedColor {
                field,
                related_field: DxfEntityField::COLOR,
                semantics,
            },
        )),
        _ => Err(invalid_internal_data()),
    }
}

fn split_proposed(
    proposed: &[u8],
    cancellation: &DxfCancellationToken,
) -> Result<Result<usize, DxfEntityCommonColorBookEditIssue>, DxfError> {
    let field = DxfEntityField::COLOR_NAME;
    let mut first = None;
    let mut separator_count = 0_u64;
    for (chunk_index, chunk) in proposed.chunks(4096).enumerate() {
        ensure_not_cancelled(cancellation)?;
        for (index, byte) in chunk.iter().copied().enumerate() {
            if byte != b'$' {
                continue;
            }
            separator_count = separator_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            first.get_or_insert(chunk_index * 4096 + index);
        }
    }
    let Some(first) = first else {
        return Ok(Err(DxfEntityCommonColorBookEditIssue::MissingSeparator {
            field,
        }));
    };
    if separator_count != 1 {
        return Ok(Err(DxfEntityCommonColorBookEditIssue::MultipleSeparators {
            field,
            separator_count,
        }));
    }
    if first == 0 {
        return Ok(Err(DxfEntityCommonColorBookEditIssue::EmptyBookName {
            field,
        }));
    }
    if first.checked_add(1) == Some(proposed.len()) {
        return Ok(Err(DxfEntityCommonColorBookEditIssue::EmptyColorName {
            field,
        }));
    }
    Ok(Ok(first))
}

fn related_color(
    domains: &DxfEntityCommonFieldDomainDirectory,
    entity: DxfEntityRef,
    field: DxfEntityField,
) -> Result<
    Result<DxfEntityCommonFieldDomainValue, DxfEntityCommonFieldDomainSemanticValue>,
    DxfError,
> {
    let Some(entry) = domains.entry_for_field(entity, field)? else {
        return Err(invalid_internal_data());
    };
    let DxfEntityCommonFieldDomainSemantics::Reviewed(semantics) = entry.semantics() else {
        return Err(invalid_internal_data());
    };
    match semantics.value().copied() {
        Some(value) => Ok(Ok(value)),
        None => Ok(Err(semantics)),
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

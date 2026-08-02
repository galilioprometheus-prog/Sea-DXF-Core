//! Source-bound admission for common entity layout-name edits.

use std::io;

use crate::source_span::span_equals_bytes;
use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityEditValue,
    DxfEntityEditValueKind, DxfEntityField, DxfError, DxfIoOperation, DxfLayoutObjectDirectory,
    DxfLayoutObjectEntry, DxfLayoutObjectNameState, DxfRawDocumentView, DxfSourceId,
};

/// One exact common layout name admitted against a same-document layout object.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonLayoutEditValue {
    field: DxfEntityField,
    target: DxfLayoutObjectEntry,
}

impl DxfEntityCommonLayoutEditValue {
    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn target(self) -> DxfLayoutObjectEntry {
        self.target
    }
}

/// Why a common layout-name edit cannot enter generic singleton planning.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonLayoutEditIssue {
    ValueKindMismatch {
        field: DxfEntityField,
        expected: DxfEntityEditValueKind,
        observed: DxfEntityEditValueKind,
    },
    Missing {
        field: DxfEntityField,
    },
    Ambiguous {
        field: DxfEntityField,
        target_count: u32,
    },
}

/// Classification of an explicit edit against reviewed common-layout rules.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonLayoutEditOutcome {
    NotLayout { field: DxfEntityField },
    Valid(DxfEntityCommonLayoutEditValue),
    Invalid(DxfEntityCommonLayoutEditIssue),
}

impl DxfRawDocumentView<'_> {
    /// Resolves one proposed exact layout name without changing the source.
    pub fn classify_entity_common_layout_edit(
        self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonLayoutEditOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let layouts = if field == DxfEntityField::LAYOUT
            && matches!(value, DxfEntityEditValue::ExactRawText(_))
        {
            Some(self.layout_object_directory(cancellation)?)
        } else {
            None
        };
        classify_with_layouts(self, layouts.as_ref(), field, value, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn classify_entity_common_layout_edit(
        &self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonLayoutEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_layout_edit(
            field,
            value,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn classify_entity_common_layout_edit(
        &self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonLayoutEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_layout_edit(
            field,
            value,
            cancellation,
        )
    }
}

pub(crate) fn classify_with_layouts(
    document: DxfRawDocumentView<'_>,
    layouts: Option<&DxfLayoutObjectDirectory>,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityCommonLayoutEditOutcome, DxfError> {
    ensure_not_cancelled(cancellation)?;
    if field != DxfEntityField::LAYOUT {
        return Ok(DxfEntityCommonLayoutEditOutcome::NotLayout { field });
    }
    let DxfEntityEditValue::ExactRawText(exact_name) = value else {
        return Ok(DxfEntityCommonLayoutEditOutcome::Invalid(
            DxfEntityCommonLayoutEditIssue::ValueKindMismatch {
                field,
                expected: DxfEntityEditValueKind::ExactRawText,
                observed: value.kind(),
            },
        ));
    };
    let layouts = layouts.ok_or_else(invalid_internal_data)?;
    ensure_source(document.source_id(), layouts.source_id())?;
    let mut first = None;
    let mut count = 0_u32;
    for target in layouts.entries().iter().copied() {
        ensure_not_cancelled(cancellation)?;
        let DxfLayoutObjectNameState::Unique(name) = target.name() else {
            continue;
        };
        if !span_equals_bytes(document, name.value_span(), exact_name, cancellation)? {
            continue;
        }
        count = count.checked_add(1).ok_or_else(invalid_internal_data)?;
        first.get_or_insert(target);
    }
    match (first, count) {
        (None, 0) => Ok(DxfEntityCommonLayoutEditOutcome::Invalid(
            DxfEntityCommonLayoutEditIssue::Missing { field },
        )),
        (Some(target), 1) => Ok(DxfEntityCommonLayoutEditOutcome::Valid(
            DxfEntityCommonLayoutEditValue { field, target },
        )),
        (Some(_), target_count) => Ok(DxfEntityCommonLayoutEditOutcome::Invalid(
            DxfEntityCommonLayoutEditIssue::Ambiguous {
                field,
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

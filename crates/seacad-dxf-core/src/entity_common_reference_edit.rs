//! Source-bound admission for common entity handle edits.

use std::io;

use crate::entity_common_reference_target::{
    common_reference_target_matches, reviewed_common_reference_target_kind,
};
use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityCommonReferenceTargetKind, DxfEntityEditValue, DxfEntityEditValueKind, DxfEntityField,
    DxfError, DxfHandle, DxfHandleIdentityDirectory, DxfHandleIdentityLookup,
    DxfHandleIdentityMatch, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// One explicit common reference admitted against an exact document target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonReferenceEditValue {
    field: DxfEntityField,
    handle: DxfHandle,
    target: DxfHandleIdentityMatch,
    expected: DxfEntityCommonReferenceTargetKind,
}

impl DxfEntityCommonReferenceEditValue {
    #[must_use]
    pub const fn field(self) -> DxfEntityField {
        self.field
    }

    #[must_use]
    pub const fn handle(self) -> DxfHandle {
        self.handle
    }

    #[must_use]
    pub const fn target(self) -> DxfHandleIdentityMatch {
        self.target
    }

    #[must_use]
    pub const fn expected_target_kind(self) -> DxfEntityCommonReferenceTargetKind {
        self.expected
    }
}

/// Why a common handle edit cannot use generic singleton replacement.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonReferenceEditIssue {
    HandleRemapRequired {
        field: DxfEntityField,
    },
    OwnerPlacementRequired {
        field: DxfEntityField,
    },
    ValueKindMismatch {
        field: DxfEntityField,
        expected: DxfEntityEditValueKind,
        observed: DxfEntityEditValueKind,
    },
    Null {
        field: DxfEntityField,
    },
    Missing {
        field: DxfEntityField,
        handle: DxfHandle,
    },
    Ambiguous {
        field: DxfEntityField,
        handle: DxfHandle,
        target_count: u32,
    },
    IncompatibleTarget {
        field: DxfEntityField,
        expected: DxfEntityCommonReferenceTargetKind,
        target: DxfHandleIdentityMatch,
    },
}

/// Classification of an explicit edit against reviewed common-reference rules.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonReferenceEditOutcome {
    NotReference { field: DxfEntityField },
    Valid(DxfEntityCommonReferenceEditValue),
    Invalid(DxfEntityCommonReferenceEditIssue),
}

impl DxfRawDocumentView<'_> {
    /// Resolves and validates one proposed common reference without editing.
    pub fn classify_entity_common_reference_edit(
        self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonReferenceEditOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let identities = if reviewed_common_reference_target_kind(field).is_some()
            && matches!(value, DxfEntityEditValue::Handle(handle) if !handle.is_null())
        {
            Some(self.handle_identity_directory(cancellation)?)
        } else {
            None
        };
        classify_with_identities(self, identities.as_ref(), field, value, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn classify_entity_common_reference_edit(
        &self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonReferenceEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_reference_edit(
            field,
            value,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn classify_entity_common_reference_edit(
        &self,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonReferenceEditOutcome, DxfError> {
        DxfRawDocumentView::from(self).classify_entity_common_reference_edit(
            field,
            value,
            cancellation,
        )
    }
}

pub(crate) fn classify_with_identities(
    document: DxfRawDocumentView<'_>,
    identities: Option<&DxfHandleIdentityDirectory>,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityCommonReferenceEditOutcome, DxfError> {
    ensure_not_cancelled(cancellation)?;
    if field == DxfEntityField::HANDLE {
        return Ok(DxfEntityCommonReferenceEditOutcome::Invalid(
            DxfEntityCommonReferenceEditIssue::HandleRemapRequired { field },
        ));
    }
    if field == DxfEntityField::OWNER {
        return Ok(DxfEntityCommonReferenceEditOutcome::Invalid(
            DxfEntityCommonReferenceEditIssue::OwnerPlacementRequired { field },
        ));
    }
    let Some(expected) = reviewed_common_reference_target_kind(field) else {
        return Ok(DxfEntityCommonReferenceEditOutcome::NotReference { field });
    };
    let DxfEntityEditValue::Handle(handle) = value else {
        return Ok(DxfEntityCommonReferenceEditOutcome::Invalid(
            DxfEntityCommonReferenceEditIssue::ValueKindMismatch {
                field,
                expected: DxfEntityEditValueKind::Handle,
                observed: value.kind(),
            },
        ));
    };
    if handle.is_null() {
        return Ok(DxfEntityCommonReferenceEditOutcome::Invalid(
            DxfEntityCommonReferenceEditIssue::Null { field },
        ));
    }
    let identities = identities.ok_or_else(invalid_internal_data)?;
    ensure_source(document.source_id(), identities.source_id())?;
    let target = match identities.lookup(handle) {
        DxfHandleIdentityLookup::Missing => {
            return Ok(DxfEntityCommonReferenceEditOutcome::Invalid(
                DxfEntityCommonReferenceEditIssue::Missing { field, handle },
            ));
        }
        DxfHandleIdentityLookup::Ambiguous(targets) => {
            return Ok(DxfEntityCommonReferenceEditOutcome::Invalid(
                DxfEntityCommonReferenceEditIssue::Ambiguous {
                    field,
                    handle,
                    target_count: compact_len(targets.len())?,
                },
            ));
        }
        DxfHandleIdentityLookup::Unique(target) => target,
    };
    ensure_not_cancelled(cancellation)?;
    if !common_reference_target_matches(document, expected, target)? {
        return Ok(DxfEntityCommonReferenceEditOutcome::Invalid(
            DxfEntityCommonReferenceEditIssue::IncompatibleTarget {
                field,
                expected,
                target,
            },
        ));
    }
    Ok(DxfEntityCommonReferenceEditOutcome::Valid(
        DxfEntityCommonReferenceEditValue {
            field,
            handle,
            target,
            expected,
        },
    ))
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
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

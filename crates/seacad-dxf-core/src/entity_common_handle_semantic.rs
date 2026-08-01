//! Common entity identity and document-local handle-reference semantics.

use std::io;

use crate::{
    DXF_ENTITY_COMMON_FIELDS, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityField, DxfEntityFieldSemanticDirectory, DxfEntityFieldSemanticEntry,
    DxfEntityFieldSemanticIssue, DxfEntityFieldSemanticValue, DxfEntityFieldSemantics,
    DxfEntityFieldValue, DxfEntityRef, DxfError, DxfHandle, DxfHandleIdentityMatch,
    DxfHandleResolutionDirectory, DxfHandleResolutionState, DxfIoOperation, DxfRawDocumentView,
    DxfSemanticValue, DxfSourceId,
};

/// Usable meaning of one common entity reference field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonReferenceValue {
    ByLayer,
    Resolved {
        handle: DxfHandle,
        target: DxfHandleIdentityMatch,
    },
}

/// Raw-field invalidity or document-local target-resolution failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonReferenceIssue {
    Field(DxfEntityFieldSemanticIssue),
    Null,
    Missing,
    Ambiguous { target_count: u32 },
}

pub type DxfEntityCommonReferenceSemanticValue =
    DxfSemanticValue<DxfEntityCommonReferenceValue, DxfEntityCommonReferenceIssue>;

/// Identity fields stay lexical; reference fields add document-local resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonHandleSemantics {
    Identity(DxfEntityFieldSemanticValue),
    Reference(DxfEntityCommonReferenceSemanticValue),
}

/// One of the five handle-valued fields in the generated common-field schema.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonHandleEntry {
    ordinal: u32,
    source: DxfEntityFieldSemanticEntry,
    semantics: DxfEntityCommonHandleSemantics,
}

impl DxfEntityCommonHandleEntry {
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
    pub const fn semantics(self) -> DxfEntityCommonHandleSemantics {
        self.semantics
    }
}

/// Source-bound identity/reference projection for common entity handle fields.
#[derive(Debug)]
pub struct DxfEntityCommonHandleDirectory {
    source_id: DxfSourceId,
    source: DxfEntityFieldSemanticDirectory,
    resolutions: DxfHandleResolutionDirectory,
    entries: Box<[DxfEntityCommonHandleEntry]>,
}

impl DxfEntityCommonHandleDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source = document.entity_field_semantic_directory(cancellation)?;
        let resolutions = document.handle_resolution_directory(cancellation)?;
        for observed in [source.source_id(), resolutions.source_id()] {
            ensure_source(document.source_id(), observed)?;
        }

        let entity_count = source.entries().len() / DXF_ENTITY_COMMON_FIELDS.len();
        let capacity = entity_count
            .checked_mul(5)
            .ok_or_else(invalid_internal_data)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(capacity)
            .map_err(|_| out_of_memory())?;
        for source_entry in source.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !is_common_handle_field(source_entry.field()) {
                continue;
            }
            let semantics = if source_entry.field() == DxfEntityField::HANDLE {
                project_identity(source_entry)?
            } else {
                DxfEntityCommonHandleSemantics::Reference(project_reference(
                    &resolutions,
                    source_entry,
                )?)
            };
            entries.push(DxfEntityCommonHandleEntry {
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
            resolutions,
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
    pub const fn resolution_directory(&self) -> &DxfHandleResolutionDirectory {
        &self.resolutions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityCommonHandleEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityCommonHandleEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityCommonHandleEntry], DxfError> {
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
    ) -> Result<Option<DxfEntityCommonHandleEntry>, DxfError> {
        if !is_common_handle_field(field) {
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
    pub fn entity_common_handle_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonHandleDirectory, DxfError> {
        DxfEntityCommonHandleDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_common_handle_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonHandleDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_handle_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_common_handle_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonHandleDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_handle_directory(cancellation)
    }
}

fn project_identity(
    source: DxfEntityFieldSemanticEntry,
) -> Result<DxfEntityCommonHandleSemantics, DxfError> {
    let DxfEntityFieldSemantics::Singleton(value) = source.semantics() else {
        return Err(invalid_internal_data());
    };
    if matches!(value.value(), Some(DxfEntityFieldValue::Handle(_)) | None) {
        Ok(DxfEntityCommonHandleSemantics::Identity(value))
    } else {
        Err(invalid_internal_data())
    }
}

fn project_reference(
    resolutions: &DxfHandleResolutionDirectory,
    source: DxfEntityFieldSemanticEntry,
) -> Result<DxfEntityCommonReferenceSemanticValue, DxfError> {
    let DxfEntityFieldSemantics::Singleton(value) = source.semantics() else {
        return Err(invalid_internal_data());
    };
    Ok(match value {
        DxfSemanticValue::Explicit {
            value: DxfEntityFieldValue::Handle(handle),
            field,
            raw,
        } => match resolutions
            .resolution_for_group(raw.group_occurrence())
            .ok_or_else(invalid_internal_data)?
            .state()
        {
            DxfHandleResolutionState::Invalid(_) => return Err(invalid_internal_data()),
            DxfHandleResolutionState::Null => {
                DxfSemanticValue::invalid(DxfEntityCommonReferenceIssue::Null, field, Some(raw))
            }
            DxfHandleResolutionState::Missing => {
                DxfSemanticValue::invalid(DxfEntityCommonReferenceIssue::Missing, field, Some(raw))
            }
            DxfHandleResolutionState::Ambiguous { target_count } => DxfSemanticValue::invalid(
                DxfEntityCommonReferenceIssue::Ambiguous { target_count },
                field,
                Some(raw),
            ),
            DxfHandleResolutionState::Unique => {
                let [target] = resolutions.identity_directory().matches_for_handle(handle) else {
                    return Err(invalid_internal_data());
                };
                DxfSemanticValue::explicit(
                    DxfEntityCommonReferenceValue::Resolved {
                        handle,
                        target: *target,
                    },
                    field,
                    raw,
                )
            }
        },
        DxfSemanticValue::Defaulted {
            value: DxfEntityFieldValue::ByLayer,
            field,
        } if source.field() == DxfEntityField::MATERIAL => {
            DxfSemanticValue::defaulted(DxfEntityCommonReferenceValue::ByLayer, field)
        }
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => {
            DxfSemanticValue::invalid(DxfEntityCommonReferenceIssue::Field(issue), field, raw)
        }
        DxfSemanticValue::Explicit { .. } | DxfSemanticValue::Defaulted { .. } => {
            return Err(invalid_internal_data());
        }
    })
}

fn is_common_handle_field(field: DxfEntityField) -> bool {
    field == DxfEntityField::HANDLE
        || field == DxfEntityField::OWNER
        || field == DxfEntityField::EXTENSION_DICTIONARY
        || field == DxfEntityField::MATERIAL
        || field == DxfEntityField::PLOT_STYLE
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

//! Existing-document semantic projection for reviewed common-field domains.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityCommonFieldDomainIssue, DxfEntityCommonFieldDomainOutcome,
    DxfEntityCommonFieldDomainValue, DxfEntityEditValue, DxfEntityField,
    DxfEntityFieldSemanticDirectory, DxfEntityFieldSemanticEntry, DxfEntityFieldSemanticIssue,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityRef, DxfError, DxfIoOperation,
    DxfRawDocumentView, DxfSemanticValue, DxfSourceId,
};

/// Invalidity inherited from raw scalar decoding or reviewed domain validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonFieldDomainSemanticIssue {
    Field(DxfEntityFieldSemanticIssue),
    Domain(DxfEntityCommonFieldDomainIssue),
}

pub type DxfEntityCommonFieldDomainSemanticValue =
    DxfSemanticValue<DxfEntityCommonFieldDomainValue, DxfEntityCommonFieldDomainSemanticIssue>;

/// Reviewed typed domain semantics or an exact pass-through unreviewed field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonFieldDomainSemantics {
    Reviewed(DxfEntityCommonFieldDomainSemanticValue),
    Unreviewed(DxfEntityFieldSemantics),
}

/// One fixed common-field entry aligned with the underlying semantic directory.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonFieldDomainEntry {
    ordinal: u32,
    source: DxfEntityFieldSemanticEntry,
    semantics: DxfEntityCommonFieldDomainSemantics,
}

impl DxfEntityCommonFieldDomainEntry {
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
    pub const fn semantics(self) -> DxfEntityCommonFieldDomainSemantics {
        self.semantics
    }
}

/// Source-bound domain projection for every common field of every entity.
#[derive(Debug)]
pub struct DxfEntityCommonFieldDomainDirectory {
    source_id: DxfSourceId,
    source: DxfEntityFieldSemanticDirectory,
    entries: Box<[DxfEntityCommonFieldDomainEntry]>,
}

impl DxfEntityCommonFieldDomainDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source = document.entity_field_semantic_directory(cancellation)?;
        ensure_source(document.source_id(), source.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(source.entries().len())
            .map_err(|_| out_of_memory())?;
        for source_entry in source.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfEntityCommonFieldDomainEntry {
                ordinal: u32::try_from(entries.len()).map_err(|_| invalid_internal_data())?,
                source: source_entry,
                semantics: project_entry(source_entry)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            source,
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
    pub fn entries(&self) -> &[DxfEntityCommonFieldDomainEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityCommonFieldDomainEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityCommonFieldDomainEntry], DxfError> {
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
    ) -> Result<Option<DxfEntityCommonFieldDomainEntry>, DxfError> {
        Ok(self
            .entries_for_entity(entity)?
            .iter()
            .copied()
            .find(|entry| entry.field() == field))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_common_field_domain_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonFieldDomainDirectory, DxfError> {
        DxfEntityCommonFieldDomainDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_common_field_domain_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonFieldDomainDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_field_domain_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_common_field_domain_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonFieldDomainDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_field_domain_directory(cancellation)
    }
}

fn project_entry(
    entry: DxfEntityFieldSemanticEntry,
) -> Result<DxfEntityCommonFieldDomainSemantics, DxfError> {
    if !crate::entity_common_field_domain::has_reviewed_entity_common_field_domain(entry.field()) {
        return Ok(DxfEntityCommonFieldDomainSemantics::Unreviewed(
            entry.semantics(),
        ));
    }
    let DxfEntityFieldSemantics::Singleton(source) = entry.semantics() else {
        return Err(invalid_internal_data());
    };
    Ok(DxfEntityCommonFieldDomainSemantics::Reviewed(
        project_value(entry.field(), source)?,
    ))
}

fn project_value(
    field: DxfEntityField,
    source: crate::DxfEntityFieldSemanticValue,
) -> Result<DxfEntityCommonFieldDomainSemanticValue, DxfError> {
    Ok(match source {
        DxfSemanticValue::Explicit {
            value,
            field: provenance,
            raw,
        } => match classify_value(field, value)? {
            Ok(value) => DxfSemanticValue::explicit(value, provenance, raw),
            Err(issue) => DxfSemanticValue::invalid(
                DxfEntityCommonFieldDomainSemanticIssue::Domain(issue),
                provenance,
                Some(raw),
            ),
        },
        DxfSemanticValue::Defaulted {
            value,
            field: provenance,
        } => match classify_value(field, value)? {
            Ok(value) => DxfSemanticValue::defaulted(value, provenance),
            Err(issue) => DxfSemanticValue::invalid(
                DxfEntityCommonFieldDomainSemanticIssue::Domain(issue),
                provenance,
                None,
            ),
        },
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfEntityCommonFieldDomainSemanticIssue::Field(issue),
            field,
            raw,
        ),
    })
}

fn classify_value(
    field: DxfEntityField,
    value: DxfEntityFieldValue,
) -> Result<Result<DxfEntityCommonFieldDomainValue, DxfEntityCommonFieldDomainIssue>, DxfError> {
    let edit = match value {
        DxfEntityFieldValue::Double(value) => DxfEntityEditValue::Double(value),
        DxfEntityFieldValue::Int16(value) => DxfEntityEditValue::Int16(value),
        DxfEntityFieldValue::Int32(value) => DxfEntityEditValue::Int32(value),
        _ => return Err(invalid_internal_data()),
    };
    match crate::classify_entity_common_field_edit_domain(field, edit) {
        DxfEntityCommonFieldDomainOutcome::Valid(value) => Ok(Ok(value)),
        DxfEntityCommonFieldDomainOutcome::Invalid(issue) => Ok(Err(issue)),
        DxfEntityCommonFieldDomainOutcome::Unreviewed { .. } => Err(invalid_internal_data()),
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
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

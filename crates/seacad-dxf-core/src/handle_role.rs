//! Conservative semantic-role evidence over contextual handle references.

use std::io;

use crate::{
    DxfApplicationGroupState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfContextualHandleReferenceDirectory, DxfContextualHandleReferenceEntry, DxfError,
    DxfHandleGroupClass, DxfHandleReferenceContext, DxfIoOperation, DxfRawDocumentView,
    DxfRawRecordSectionKind, DxfSourceId,
};

/// Conservative role evidence for one contextual pointer or owner occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleRoleEvidence {
    GenericPointer,
    GenericOwnership,
    PersistentReactorCandidate,
    ExtensionDictionaryCandidate,
    CommonOwnerPointerCandidate,
}

/// One contextual reference and its conservative role evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHandleRoleEntry {
    contextual: DxfContextualHandleReferenceEntry,
    role: DxfHandleRoleEvidence,
}

impl DxfHandleRoleEntry {
    #[must_use]
    pub const fn contextual(self) -> DxfContextualHandleReferenceEntry {
        self.contextual
    }

    #[must_use]
    pub const fn role(self) -> DxfHandleRoleEvidence {
        self.role
    }
}

/// Immutable source-order role evidence for every contextual handle reference.
///
/// Candidate roles require exact documented code, context, closed-group, and
/// record-section shapes. They remain evidence rather than validity claims.
#[derive(Debug)]
pub struct DxfHandleRoleDirectory {
    source_id: DxfSourceId,
    contextual: DxfContextualHandleReferenceDirectory,
    entries: Box<[DxfHandleRoleEntry]>,
}

impl DxfHandleRoleDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let contextual = document.contextual_handle_reference_directory(cancellation)?;
        if contextual.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: contextual.source_id(),
            });
        }

        let mut entries = Vec::new();
        entries
            .try_reserve(contextual.entries().len())
            .map_err(|_| out_of_memory())?;
        for (ordinal, entry) in contextual.entries().iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            let role = classify_role(&contextual, ordinal, entry)?;
            entries.push(DxfHandleRoleEntry {
                contextual: entry,
                role,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            contextual,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn contextual_directory(&self) -> &DxfContextualHandleReferenceDirectory {
        &self.contextual
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHandleRoleEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, reference_ordinal: u64) -> Option<DxfHandleRoleEntry> {
        let index = usize::try_from(reference_ordinal).ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_record(&self, record_ordinal: u64) -> Option<&[DxfHandleRoleEntry]> {
        self.contextual.entries_for_record(record_ordinal)?;
        let start = self.entries.partition_point(|entry| {
            entry
                .contextual()
                .resolution()
                .reference()
                .record()
                .ordinal()
                < record_ordinal
        });
        let end = self.entries.partition_point(|entry| {
            entry
                .contextual()
                .resolution()
                .reference()
                .record()
                .ordinal()
                <= record_ordinal
        });
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn entry_for_group(&self, occurrence: u64) -> Option<DxfHandleRoleEntry> {
        let index = self.entries.partition_point(|entry| {
            entry
                .contextual()
                .resolution()
                .reference()
                .value()
                .group()
                .occurrence()
                < occurrence
        });
        self.entries.get(index).copied().filter(|entry| {
            entry
                .contextual()
                .resolution()
                .reference()
                .value()
                .group()
                .occurrence()
                == occurrence
        })
    }
}

impl DxfRawDocumentView<'_> {
    /// Classifies conservative semantic-role evidence for contextual handles.
    pub fn handle_role_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleRoleDirectory, DxfError> {
        DxfHandleRoleDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn handle_role_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleRoleDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_role_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn handle_role_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleRoleDirectory, DxfError> {
        DxfRawDocumentView::from(self).handle_role_directory(cancellation)
    }
}

fn classify_role(
    directory: &DxfContextualHandleReferenceDirectory,
    ordinal: usize,
    entry: DxfContextualHandleReferenceEntry,
) -> Result<DxfHandleRoleEvidence, DxfError> {
    let reference = entry.resolution().reference();
    let code = reference.value().group().group_code().value();
    let section = reference.record().section_kind();
    let common_section = is_common_object_section(section);
    let closed_group = directory
        .application_group_for_entry(u64::try_from(ordinal).map_err(|_| invalid_internal_data())?)
        .is_some_and(|group| group.state() == DxfApplicationGroupState::Closed);

    if common_section
        && closed_group
        && code == 330
        && reference.class() == DxfHandleGroupClass::SoftPointer
        && entry.context() == DxfHandleReferenceContext::AcadReactors
    {
        return Ok(DxfHandleRoleEvidence::PersistentReactorCandidate);
    }
    if common_section
        && closed_group
        && code == 360
        && reference.class() == DxfHandleGroupClass::HardOwner
        && entry.context() == DxfHandleReferenceContext::AcadXDictionary
    {
        return Ok(DxfHandleRoleEvidence::ExtensionDictionaryCandidate);
    }
    if common_section
        && code == 330
        && reference.class() == DxfHandleGroupClass::SoftPointer
        && entry.context() == DxfHandleReferenceContext::OutsideApplicationGroup
    {
        return Ok(DxfHandleRoleEvidence::CommonOwnerPointerCandidate);
    }
    Ok(match reference.class() {
        DxfHandleGroupClass::SoftPointer | DxfHandleGroupClass::HardPointer => {
            DxfHandleRoleEvidence::GenericPointer
        }
        DxfHandleGroupClass::SoftOwner | DxfHandleGroupClass::HardOwner => {
            DxfHandleRoleEvidence::GenericOwnership
        }
        DxfHandleGroupClass::ObjectIdentity | DxfHandleGroupClass::Arbitrary => {
            return Err(invalid_internal_data());
        }
    })
}

const fn is_common_object_section(section: DxfRawRecordSectionKind) -> bool {
    matches!(
        section,
        DxfRawRecordSectionKind::Tables
            | DxfRawRecordSectionKind::Blocks
            | DxfRawRecordSectionKind::Entities
            | DxfRawRecordSectionKind::Objects
    )
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

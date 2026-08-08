//! Exact subclass-scoped raw evidence for HATCH and modern MESH entities.

use std::io;

use crate::{
    DxfApplicationGroupDirectory, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityClassification, DxfEntityDirectory, DxfEntityRef, DxfEntityTopic, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfFillMeshFamily {
    Hatch,
    Mesh,
}

impl DxfFillMeshFamily {
    const fn topic(self) -> DxfEntityTopic {
        match self {
            Self::Hatch => DxfEntityTopic::HATCH,
            Self::Mesh => DxfEntityTopic::MESH,
        }
    }

    const fn subclass_name(self) -> &'static [u8] {
        match self {
            Self::Hatch => b"AcDbHatch",
            Self::Mesh => b"AcDbSubDMesh",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfFillMeshRange {
    start: u32,
    end: u32,
}

impl DxfFillMeshRange {
    pub(crate) fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_internal_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfFillMeshRecordEntry {
    entity: DxfEntityRef,
    family: DxfFillMeshFamily,
    subclass_range: DxfFillMeshRange,
}

impl DxfFillMeshRecordEntry {
    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn family(self) -> DxfFillMeshFamily {
        self.family
    }

    #[must_use]
    pub const fn subclass_range(self) -> DxfFillMeshRange {
        self.subclass_range
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfFillMeshSubclassEntry {
    entity: DxfEntityRef,
    family: DxfFillMeshFamily,
    marker: DxfRawGroup,
    field_range: DxfFillMeshRange,
}

impl DxfFillMeshSubclassEntry {
    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn family(self) -> DxfFillMeshFamily {
        self.family
    }

    #[must_use]
    pub const fn marker(self) -> DxfRawGroup {
        self.marker
    }

    #[must_use]
    pub const fn field_range(self) -> DxfFillMeshRange {
        self.field_range
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfFillMeshField {
    group: DxfRawGroup,
    family: DxfFillMeshFamily,
    subclass_ordinal: u32,
}

impl DxfFillMeshField {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn family(self) -> DxfFillMeshFamily {
        self.family
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }
}

/// Raw HATCH/MESH fields without role assignment, decoding, or applicability claims.
#[derive(Debug)]
pub struct DxfFillMeshEvidenceDirectory {
    source_id: DxfSourceId,
    entities: DxfEntityDirectory,
    application_groups: DxfApplicationGroupDirectory,
    records: Box<[DxfFillMeshRecordEntry]>,
    subclasses: Box<[DxfFillMeshSubclassEntry]>,
    fields: Box<[DxfFillMeshField]>,
}

impl DxfFillMeshEvidenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let entities = document.entity_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        ensure_source(document.source_id(), entities.source_id())?;
        ensure_source(document.source_id(), application_groups.source_id())?;

        let mut records = Vec::new();
        let mut subclasses = Vec::new();
        let mut fields = Vec::new();
        for entity in entities.entities().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let Some(family) = family_for_entity(entity) else {
                continue;
            };
            let subclass_start = compact_len(subclasses.len())?;
            let path = entities.subclass_path(entity)?;
            for (index, subclass) in path.iter().copied().enumerate() {
                ensure_not_cancelled(cancellation)?;
                if !document.raw_span_equals_exact(
                    subclass.group().value_payload_span(),
                    family.subclass_name(),
                )? {
                    continue;
                }
                let field_start = compact_len(fields.len())?;
                let end = path
                    .get(index.saturating_add(1))
                    .map_or(entity.record().group_range().end(), |next| {
                        next.group().occurrence()
                    });
                let subclass_ordinal = compact_len(subclasses.len())?;
                append_fields(
                    document,
                    family,
                    subclass_ordinal,
                    subclass.group().occurrence().saturating_add(1)..end,
                    &application_groups,
                    cancellation,
                    &mut fields,
                )?;
                let field_end = compact_len(fields.len())?;
                subclasses.try_reserve(1).map_err(|_| out_of_memory())?;
                subclasses.push(DxfFillMeshSubclassEntry {
                    entity,
                    family,
                    marker: subclass.group(),
                    field_range: DxfFillMeshRange::new(field_start, field_end)?,
                });
            }
            let subclass_end = compact_len(subclasses.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfFillMeshRecordEntry {
                entity,
                family,
                subclass_range: DxfFillMeshRange::new(subclass_start, subclass_end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entities,
            application_groups,
            records: records.into_boxed_slice(),
            subclasses: subclasses.into_boxed_slice(),
            fields: fields.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn entity_directory(&self) -> &DxfEntityDirectory {
        &self.entities
    }

    #[must_use]
    pub const fn application_group_directory(&self) -> &DxfApplicationGroupDirectory {
        &self.application_groups
    }

    #[must_use]
    pub fn records(&self) -> &[DxfFillMeshRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn subclasses(&self) -> &[DxfFillMeshSubclassEntry] {
        &self.subclasses
    }

    #[must_use]
    pub fn fields(&self) -> &[DxfFillMeshField] {
        &self.fields
    }

    #[must_use]
    pub fn record_for_raw_ordinal(&self, raw: u64) -> Option<DxfFillMeshRecordEntry> {
        self.records
            .binary_search_by_key(&raw, |entry| entry.entity().record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn subclasses_for_raw_record(&self, raw: u64) -> Option<&[DxfFillMeshSubclassEntry]> {
        let entry = self.record_for_raw_ordinal(raw)?;
        slice_for_range(&self.subclasses, entry.subclass_range())
    }

    #[must_use]
    pub fn fields_for_subclass(&self, ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let index = usize::try_from(ordinal).ok()?;
        let entry = self.subclasses.get(index)?;
        slice_for_range(&self.fields, entry.field_range())
    }

    #[must_use]
    pub fn field_for_group(&self, occurrence: u64) -> Option<DxfFillMeshField> {
        let index = self
            .fields
            .partition_point(|field| field.group().occurrence() < occurrence);
        self.fields
            .get(index)
            .copied()
            .filter(|field| field.group().occurrence() == occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn fill_mesh_evidence_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfFillMeshEvidenceDirectory, DxfError> {
        DxfFillMeshEvidenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn fill_mesh_evidence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfFillMeshEvidenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).fill_mesh_evidence_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn fill_mesh_evidence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfFillMeshEvidenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).fill_mesh_evidence_directory(cancellation)
    }
}

fn append_fields(
    document: DxfRawDocumentView<'_>,
    family: DxfFillMeshFamily,
    subclass_ordinal: u32,
    occurrences: std::ops::Range<u64>,
    application_groups: &DxfApplicationGroupDirectory,
    cancellation: &DxfCancellationToken,
    fields: &mut Vec<DxfFillMeshField>,
) -> Result<(), DxfError> {
    for occurrence in occurrences {
        ensure_not_cancelled(cancellation)?;
        if application_groups
            .group_for_content_occurrence(occurrence)
            .is_some()
        {
            continue;
        }
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if group.group_code().value() == 102 || group.group_code().value() >= 1000 {
            continue;
        }
        fields.try_reserve(1).map_err(|_| out_of_memory())?;
        fields.push(DxfFillMeshField {
            group,
            family,
            subclass_ordinal,
        });
    }
    Ok(())
}

fn family_for_entity(entity: DxfEntityRef) -> Option<DxfFillMeshFamily> {
    [DxfFillMeshFamily::Hatch, DxfFillMeshFamily::Mesh]
        .into_iter()
        .find(|family| {
            entity.classification() == DxfEntityClassification::Canonical(family.topic())
        })
}

fn slice_for_range<T>(values: &[T], range: DxfFillMeshRange) -> Option<&[T]> {
    let start = usize::try_from(range.start()).ok()?;
    let end = usize::try_from(range.end()).ok()?;
    values.get(start..end)
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

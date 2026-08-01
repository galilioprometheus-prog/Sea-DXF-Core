//! Target-kind validation for reviewed common entity references.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityCommonHandleDirectory, DxfEntityCommonHandleEntry, DxfEntityCommonHandleSemantics,
    DxfEntityCommonReferenceIssue, DxfEntityCommonReferenceSemanticValue,
    DxfEntityCommonReferenceValue, DxfEntityField, DxfEntityRef, DxfError, DxfHandleIdentityMatch,
    DxfIoOperation, DxfRawDocumentView, DxfRawRecordSectionKind, DxfSemanticValue, DxfSourceId,
};

const REVIEWED_REFERENCE_COUNT: usize = 4;
const COMMON_HANDLE_COUNT: usize = REVIEWED_REFERENCE_COUNT + 1;

/// Exact public object kind required by one reviewed common reference.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonReferenceTargetKind {
    ExtensionDictionary,
    Material,
    PlotStyle,
}

impl DxfEntityCommonReferenceTargetKind {
    #[must_use]
    pub const fn section(self) -> DxfRawRecordSectionKind {
        DxfRawRecordSectionKind::Objects
    }

    #[must_use]
    pub const fn marker(self) -> &'static [u8] {
        match self {
            Self::ExtensionDictionary => b"DICTIONARY",
            Self::Material => b"MATERIAL",
            Self::PlotStyle => b"ACDBPLACEHOLDER",
        }
    }
}

/// Source resolution failure or unique target with an incompatible record kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonReferenceTargetIssue {
    Source(DxfEntityCommonReferenceIssue),
    IncompatibleTarget {
        expected: DxfEntityCommonReferenceTargetKind,
        target: DxfHandleIdentityMatch,
    },
}

pub type DxfEntityCommonReferenceTargetSemanticValue =
    DxfSemanticValue<DxfEntityCommonReferenceValue, DxfEntityCommonReferenceTargetIssue>;

/// Owner remains open by family; the other common references validate target kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityCommonReferenceTargetSemantics {
    Unreviewed(DxfEntityCommonReferenceSemanticValue),
    Reviewed(DxfEntityCommonReferenceTargetSemanticValue),
}

/// One common reference aligned with the underlying common-handle directory.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCommonReferenceTargetEntry {
    ordinal: u32,
    source: DxfEntityCommonHandleEntry,
    expected: Option<DxfEntityCommonReferenceTargetKind>,
    semantics: DxfEntityCommonReferenceTargetSemantics,
}

impl DxfEntityCommonReferenceTargetEntry {
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
    pub const fn source_semantics(self) -> DxfEntityCommonHandleEntry {
        self.source
    }

    #[must_use]
    pub const fn expected_target_kind(self) -> Option<DxfEntityCommonReferenceTargetKind> {
        self.expected
    }

    #[must_use]
    pub const fn semantics(self) -> DxfEntityCommonReferenceTargetSemantics {
        self.semantics
    }
}

/// Source-bound target-kind projection for the four common reference fields.
#[derive(Debug)]
pub struct DxfEntityCommonReferenceTargetDirectory {
    source_id: DxfSourceId,
    source: DxfEntityCommonHandleDirectory,
    entries: Box<[DxfEntityCommonReferenceTargetEntry]>,
}

impl DxfEntityCommonReferenceTargetDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let source = document.entity_common_handle_directory(cancellation)?;
        ensure_source(document.source_id(), source.source_id())?;
        let entity_count = source.entries().len() / COMMON_HANDLE_COUNT;
        let capacity = entity_count
            .checked_mul(REVIEWED_REFERENCE_COUNT)
            .ok_or_else(invalid_internal_data)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(capacity)
            .map_err(|_| out_of_memory())?;
        for source_entry in source.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if source_entry.field() == DxfEntityField::HANDLE {
                continue;
            }
            let DxfEntityCommonHandleSemantics::Reference(reference) = source_entry.semantics()
            else {
                return Err(invalid_internal_data());
            };
            let expected = reviewed_common_reference_target_kind(source_entry.field());
            let semantics = match expected {
                Some(kind) => DxfEntityCommonReferenceTargetSemantics::Reviewed(project_reviewed(
                    document, kind, reference,
                )?),
                None if source_entry.field() == DxfEntityField::OWNER => {
                    DxfEntityCommonReferenceTargetSemantics::Unreviewed(reference)
                }
                None => return Err(invalid_internal_data()),
            };
            entries.push(DxfEntityCommonReferenceTargetEntry {
                ordinal: compact_len(entries.len())?,
                source: source_entry,
                expected,
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
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_directory(&self) -> &DxfEntityCommonHandleDirectory {
        &self.source
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityCommonReferenceTargetEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityCommonReferenceTargetEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entries_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntityCommonReferenceTargetEntry], DxfError> {
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
    ) -> Result<Option<DxfEntityCommonReferenceTargetEntry>, DxfError> {
        if field == DxfEntityField::HANDLE || !is_common_reference(field) {
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
    pub fn entity_common_reference_target_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonReferenceTargetDirectory, DxfError> {
        DxfEntityCommonReferenceTargetDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_common_reference_target_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonReferenceTargetDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_reference_target_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_common_reference_target_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityCommonReferenceTargetDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_common_reference_target_directory(cancellation)
    }
}

fn project_reviewed(
    document: DxfRawDocumentView<'_>,
    expected: DxfEntityCommonReferenceTargetKind,
    source: DxfEntityCommonReferenceSemanticValue,
) -> Result<DxfEntityCommonReferenceTargetSemanticValue, DxfError> {
    Ok(match source {
        DxfSemanticValue::Explicit {
            value: value @ DxfEntityCommonReferenceValue::Resolved { target, .. },
            field,
            raw,
        } => {
            if common_reference_target_matches(document, expected, target)? {
                DxfSemanticValue::explicit(value, field, raw)
            } else {
                DxfSemanticValue::invalid(
                    DxfEntityCommonReferenceTargetIssue::IncompatibleTarget { expected, target },
                    field,
                    Some(raw),
                )
            }
        }
        DxfSemanticValue::Defaulted {
            value: DxfEntityCommonReferenceValue::ByLayer,
            field,
        } if expected == DxfEntityCommonReferenceTargetKind::Material => {
            DxfSemanticValue::defaulted(DxfEntityCommonReferenceValue::ByLayer, field)
        }
        DxfSemanticValue::Absent { field } => DxfSemanticValue::absent(field),
        DxfSemanticValue::Invalid { issue, field, raw } => DxfSemanticValue::invalid(
            DxfEntityCommonReferenceTargetIssue::Source(issue),
            field,
            raw,
        ),
        DxfSemanticValue::Explicit { .. } | DxfSemanticValue::Defaulted { .. } => {
            return Err(invalid_internal_data());
        }
    })
}

pub(crate) fn reviewed_common_reference_target_kind(
    field: DxfEntityField,
) -> Option<DxfEntityCommonReferenceTargetKind> {
    if field == DxfEntityField::EXTENSION_DICTIONARY {
        Some(DxfEntityCommonReferenceTargetKind::ExtensionDictionary)
    } else if field == DxfEntityField::MATERIAL {
        Some(DxfEntityCommonReferenceTargetKind::Material)
    } else if field == DxfEntityField::PLOT_STYLE {
        Some(DxfEntityCommonReferenceTargetKind::PlotStyle)
    } else {
        None
    }
}

pub(crate) fn common_reference_target_matches(
    document: DxfRawDocumentView<'_>,
    expected: DxfEntityCommonReferenceTargetKind,
    target: DxfHandleIdentityMatch,
) -> Result<bool, DxfError> {
    let record = target.record();
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    Ok(record.section_kind() == expected.section()
        && marker.value_payload_span().len()
            == u64::try_from(expected.marker().len()).map_err(|_| invalid_internal_data())?
        && document.raw_span_equals_exact(marker.value_payload_span(), expected.marker())?)
}

fn is_common_reference(field: DxfEntityField) -> bool {
    field == DxfEntityField::OWNER
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

//! Unified, source-anchored index for reviewed and unknown DXF entities.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfEntityAlias,
    DxfEntityNameClassification, DxfEntityTopic, DxfError, DxfIoOperation, DxfRawDocumentView,
    DxfRawGroup, DxfRawRecord, DxfRawRecordSectionKind, DxfSourceId,
    classify_exact_dxf_entity_name,
};

const MAX_REVIEWED_ENTITY_NAME_BYTES: usize = 64;

/// A reviewed canonical topic or one of its reviewed exact aliases.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityKnownClassification {
    Canonical(DxfEntityTopic),
    Alias(DxfEntityAlias),
}

impl DxfEntityKnownClassification {
    #[must_use]
    pub fn topic(self) -> Option<DxfEntityTopic> {
        match self {
            Self::Canonical(topic) => Some(topic),
            Self::Alias(alias) => alias.descriptor().map(|descriptor| descriptor.topic()),
        }
    }
}

/// Placement-aware classification of an exact group-zero record marker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityClassification {
    Canonical(DxfEntityTopic),
    Alias(DxfEntityAlias),
    Unknown,
    WrongSection(DxfEntityKnownClassification),
}

impl DxfEntityClassification {
    #[must_use]
    pub fn topic(self) -> Option<DxfEntityTopic> {
        match self {
            Self::Canonical(topic) => Some(topic),
            Self::Alias(alias) => alias.descriptor().map(|descriptor| descriptor.topic()),
            Self::WrongSection(classification) => classification.topic(),
            Self::Unknown => None,
        }
    }
}

/// Half-open range of source-order subclass markers in an entity directory.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntitySubclassRange {
    start: u32,
    end: u32,
}

impl DxfEntitySubclassRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
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

/// One exact group-code 100 occurrence belonging to an indexed entity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntitySubclassMarker {
    record: DxfRawRecord,
    group: DxfRawGroup,
}

impl DxfEntitySubclassMarker {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }
}

/// One entity-like raw record with its exact marker and subclass path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityRef {
    source_id: DxfSourceId,
    record: DxfRawRecord,
    marker: DxfRawGroup,
    classification: DxfEntityClassification,
    subclass_range: DxfEntitySubclassRange,
}

impl DxfEntityRef {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn marker(self) -> DxfRawGroup {
        self.marker
    }

    #[must_use]
    pub const fn classification(self) -> DxfEntityClassification {
        self.classification
    }

    #[must_use]
    pub const fn subclass_range(self) -> DxfEntitySubclassRange {
        self.subclass_range
    }
}

/// Immutable source-order index shared by all entity families.
#[derive(Debug)]
pub struct DxfEntityDirectory {
    source_id: DxfSourceId,
    raw_record_count: u32,
    entities: Box<[DxfEntityRef]>,
    subclass_markers: Box<[DxfEntitySubclassMarker]>,
}

impl DxfEntityDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let records = document.raw_record_directory(cancellation)?;
        let application_groups = document.application_group_directory(cancellation)?;
        if records.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: records.source_id(),
            });
        }
        if application_groups.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: application_groups.source_id(),
            });
        }

        let raw_record_count = compact_len(records.records().len())?;
        let mut entities = Vec::new();
        let mut subclass_markers = Vec::new();
        for record in records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let marker = document
                .group(record.marker_occurrence())
                .ok_or_else(invalid_internal_data)?;
            if marker.group_code().value() != 0 {
                return Err(invalid_internal_data());
            }
            let marker_name = read_marker_name(document, marker, cancellation)?;
            let known = marker_name.classification;
            let in_entity_section = matches!(
                record.section_kind(),
                DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
            );
            if in_entity_section && marker_name.is_block_control(record.section_kind()) {
                continue;
            }
            let classification = match (in_entity_section, known) {
                (true, DxfEntityNameClassification::Canonical(topic)) => {
                    DxfEntityClassification::Canonical(topic)
                }
                (true, DxfEntityNameClassification::Alias(alias)) => {
                    DxfEntityClassification::Alias(alias)
                }
                (true, DxfEntityNameClassification::Unknown) => DxfEntityClassification::Unknown,
                (false, DxfEntityNameClassification::Canonical(topic)) => {
                    DxfEntityClassification::WrongSection(DxfEntityKnownClassification::Canonical(
                        topic,
                    ))
                }
                (false, DxfEntityNameClassification::Alias(alias)) => {
                    DxfEntityClassification::WrongSection(DxfEntityKnownClassification::Alias(
                        alias,
                    ))
                }
                (false, DxfEntityNameClassification::Unknown) => continue,
            };

            let subclass_start = compact_len(subclass_markers.len())?;
            for occurrence in record.group_range().start()..record.group_range().end() {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                if group.group_code().value() == 100
                    && application_groups
                        .group_for_content_occurrence(occurrence)
                        .is_none()
                {
                    subclass_markers
                        .try_reserve(1)
                        .map_err(|_| out_of_memory())?;
                    subclass_markers.push(DxfEntitySubclassMarker { record, group });
                }
            }
            let subclass_end = compact_len(subclass_markers.len())?;
            entities.try_reserve(1).map_err(|_| out_of_memory())?;
            entities.push(DxfEntityRef {
                source_id: document.source_id(),
                record,
                marker,
                classification,
                subclass_range: DxfEntitySubclassRange::new(subclass_start, subclass_end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            raw_record_count,
            entities: entities.into_boxed_slice(),
            subclass_markers: subclass_markers.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn raw_record_count(&self) -> u64 {
        self.raw_record_count as u64
    }

    #[must_use]
    pub fn entities(&self) -> &[DxfEntityRef] {
        &self.entities
    }

    #[must_use]
    pub fn subclass_markers(&self) -> &[DxfEntitySubclassMarker] {
        &self.subclass_markers
    }

    #[must_use]
    pub fn entity_for_raw_ordinal(&self, raw_record_ordinal: u64) -> Option<DxfEntityRef> {
        self.entities
            .binary_search_by_key(&raw_record_ordinal, |entity| entity.record().ordinal())
            .ok()
            .and_then(|index| self.entities.get(index).copied())
    }

    #[must_use]
    pub fn entity_for_group(&self, occurrence: u64) -> Option<DxfEntityRef> {
        let index = self
            .entities
            .partition_point(|entity| entity.record().group_range().end() <= occurrence);
        self.entities.get(index).copied().filter(|entity| {
            occurrence >= entity.record().group_range().start()
                && occurrence < entity.record().group_range().end()
        })
    }

    pub fn subclass_path(
        &self,
        entity: DxfEntityRef,
    ) -> Result<&[DxfEntitySubclassMarker], DxfError> {
        if entity.source_id() != self.source_id {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: entity.source_id(),
            });
        }
        let start = usize::try_from(entity.subclass_range().start())
            .map_err(|_| invalid_internal_data())?;
        let end =
            usize::try_from(entity.subclass_range().end()).map_err(|_| invalid_internal_data())?;
        self.subclass_markers
            .get(start..end)
            .ok_or_else(invalid_internal_data)
    }
}

impl DxfRawDocumentView<'_> {
    /// Indexes entities, exact aliases, unknown markers, and reviewed wrong-section records.
    pub fn entity_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityDirectory, DxfError> {
        DxfEntityDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_directory(cancellation)
    }
}

struct MarkerName {
    bytes: [u8; MAX_REVIEWED_ENTITY_NAME_BYTES],
    len: usize,
    classification: DxfEntityNameClassification,
}

impl MarkerName {
    fn is_block_control(&self, section: DxfRawRecordSectionKind) -> bool {
        section == DxfRawRecordSectionKind::Blocks
            && matches!(self.as_bytes(), b"BLOCK" | b"ENDBLK")
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

fn read_marker_name(
    document: DxfRawDocumentView<'_>,
    marker: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<MarkerName, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let mut bytes = [0_u8; MAX_REVIEWED_ENTITY_NAME_BYTES];
    let Ok(len) = usize::try_from(marker.value_payload_span().len()) else {
        return Ok(MarkerName {
            bytes,
            len: 0,
            classification: DxfEntityNameClassification::Unknown,
        });
    };
    if len > bytes.len() {
        return Ok(MarkerName {
            bytes,
            len: 0,
            classification: DxfEntityNameClassification::Unknown,
        });
    }
    document.read_span(marker.value_payload_span(), &mut bytes[..len])?;
    ensure_not_cancelled(cancellation)?;
    let classification = classify_exact_dxf_entity_name(&bytes[..len]);
    Ok(MarkerName {
        bytes,
        len,
        classification,
    })
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
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

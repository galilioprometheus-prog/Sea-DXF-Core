//! Source-anchored POINT and LINE coordinate-component evidence.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfSourceId, raw_double::decode_raw_double,
};

/// Basic entity family whose documented coordinate components are indexed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBasicGeometryKind {
    Point,
    Line,
}

/// Documented coordinate role without applying a coordinate transformation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBasicGeometryComponentRole {
    WcsLocationOrStartX,
    WcsLocationOrStartY,
    WcsLocationOrStartZ,
    WcsEndpointX,
    WcsEndpointY,
    WcsEndpointZ,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
}

/// Lexical reason why one coordinate component cannot be decoded exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfBasicGeometryNumericIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

/// Half-open component range owned by one basic-geometry record entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBasicGeometryComponentRange {
    start: u32,
    end: u32,
}

impl DxfBasicGeometryComponentRange {
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

/// One source-order coordinate component and its exact binary64 evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBasicGeometryComponent {
    group: DxfRawGroup,
    role: DxfBasicGeometryComponentRole,
    value: Result<DxfDouble, DxfBasicGeometryNumericIssue>,
}

impl DxfBasicGeometryComponent {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfBasicGeometryComponentRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfDouble, DxfBasicGeometryNumericIssue> {
        self.value
    }
}

/// One exact POINT or LINE marker and its source-order component slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBasicGeometryRecordEntry {
    record: DxfRawRecord,
    kind: DxfBasicGeometryKind,
    component_range: DxfBasicGeometryComponentRange,
}

impl DxfBasicGeometryRecordEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        self.record
    }

    #[must_use]
    pub const fn kind(self) -> DxfBasicGeometryKind {
        self.kind
    }

    #[must_use]
    pub const fn component_range(self) -> DxfBasicGeometryComponentRange {
        self.component_range
    }
}

/// Immutable evidence directory for exact POINT and LINE coordinate groups.
///
/// Entries preserve source order, duplicates, lexical invalidity, and raw group
/// spans. They do not choose canonical components, apply documented defaults,
/// validate entity completeness, or transform coordinates.
#[derive(Debug)]
pub struct DxfBasicGeometryDirectory {
    source_id: DxfSourceId,
    record_count: u32,
    records: Box<[DxfBasicGeometryRecordEntry]>,
    components: Box<[DxfBasicGeometryComponent]>,
}

impl DxfBasicGeometryDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_records = document.raw_record_directory(cancellation)?;
        if raw_records.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: raw_records.source_id(),
            });
        }

        let record_count = compact_len(raw_records.records().len())?;
        let mut records = Vec::new();
        let mut components = Vec::new();
        for record in raw_records.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            if !matches!(
                record.section_kind(),
                DxfRawRecordSectionKind::Blocks | DxfRawRecordSectionKind::Entities
            ) {
                continue;
            }
            let Some(kind) = record_kind(document, record)? else {
                continue;
            };

            let start = compact_len(components.len())?;
            for occurrence in
                record.marker_occurrence().saturating_add(1)..record.group_range().end()
            {
                ensure_not_cancelled(cancellation)?;
                let group = document
                    .group(occurrence)
                    .ok_or_else(invalid_internal_data)?;
                let Some(role) = component_role(kind, group.group_code().value()) else {
                    continue;
                };
                let value = decode_raw_double(document, group, cancellation)?
                    .map_err(DxfBasicGeometryNumericIssue::InvalidAsciiNumber);
                components.try_reserve(1).map_err(|_| out_of_memory())?;
                components.push(DxfBasicGeometryComponent { group, role, value });
            }
            let end = compact_len(components.len())?;
            records.try_reserve(1).map_err(|_| out_of_memory())?;
            records.push(DxfBasicGeometryRecordEntry {
                record,
                kind,
                component_range: DxfBasicGeometryComponentRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            record_count,
            records: records.into_boxed_slice(),
            components: components.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn raw_record_count(&self) -> u64 {
        self.record_count as u64
    }

    #[must_use]
    pub fn records(&self) -> &[DxfBasicGeometryRecordEntry] {
        &self.records
    }

    #[must_use]
    pub fn components(&self) -> &[DxfBasicGeometryComponent] {
        &self.components
    }

    #[must_use]
    pub fn record_for_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfBasicGeometryRecordEntry> {
        self.records
            .binary_search_by_key(&raw_record_ordinal, |entry| entry.record().ordinal())
            .ok()
            .and_then(|index| self.records.get(index).copied())
    }

    #[must_use]
    pub fn components_for_raw_record(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBasicGeometryComponent]> {
        let entry = self.record_for_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.component_range().start()).ok()?;
        let end = usize::try_from(entry.component_range().end()).ok()?;
        self.components.get(start..end)
    }

    #[must_use]
    pub fn component_for_group(&self, occurrence: u64) -> Option<DxfBasicGeometryComponent> {
        let index = self
            .components
            .partition_point(|entry| entry.group().occurrence() < occurrence);
        self.components
            .get(index)
            .copied()
            .filter(|entry| entry.group().occurrence() == occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn basic_geometry_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometryDirectory, DxfError> {
        DxfBasicGeometryDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn basic_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).basic_geometry_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn basic_geometry_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBasicGeometryDirectory, DxfError> {
        DxfRawDocumentView::from(self).basic_geometry_directory(cancellation)
    }
}

fn record_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<Option<DxfBasicGeometryKind>, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    if document.raw_span_equals_exact(marker.value_payload_span(), b"POINT")? {
        Ok(Some(DxfBasicGeometryKind::Point))
    } else if document.raw_span_equals_exact(marker.value_payload_span(), b"LINE")? {
        Ok(Some(DxfBasicGeometryKind::Line))
    } else {
        Ok(None)
    }
}

const fn component_role(
    kind: DxfBasicGeometryKind,
    group_code: i16,
) -> Option<DxfBasicGeometryComponentRole> {
    use DxfBasicGeometryComponentRole::{
        ExtrusionX, ExtrusionY, ExtrusionZ, WcsEndpointX, WcsEndpointY, WcsEndpointZ,
        WcsLocationOrStartX, WcsLocationOrStartY, WcsLocationOrStartZ,
    };
    match (kind, group_code) {
        (_, 10) => Some(WcsLocationOrStartX),
        (_, 20) => Some(WcsLocationOrStartY),
        (_, 30) => Some(WcsLocationOrStartZ),
        (DxfBasicGeometryKind::Line, 11) => Some(WcsEndpointX),
        (DxfBasicGeometryKind::Line, 21) => Some(WcsEndpointY),
        (DxfBasicGeometryKind::Line, 31) => Some(WcsEndpointZ),
        (_, 210) => Some(ExtrusionX),
        (_, 220) => Some(ExtrusionY),
        (_, 230) => Some(ExtrusionZ),
        (DxfBasicGeometryKind::Point, 11 | 21 | 31) | (_, _) => None,
    }
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

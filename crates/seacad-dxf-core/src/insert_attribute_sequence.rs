//! Exact INSERT/ATTRIB/SEQEND record-sequence evidence.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfInsertRecordSemanticDirectory, DxfInsertRecordValueEntry, DxfRawDocumentView, DxfRawRecord,
    DxfRawRecordDirectory, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeSequenceState {
    /// The reviewed attributes-follow value is zero, so no sequence is consumed.
    NoAttributesFollow,
    /// The attributes-follow field is invalid or has duplicate occurrences.
    FlagUnavailable,
    /// Zero or more consecutive ATTRIB records are followed by exact SEQEND.
    Closed,
    /// Another same-section record appears before exact SEQEND.
    Interrupted,
    /// The containing complete section ends before exact SEQEND.
    Unclosed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeRecordRange {
    start: u32,
    end: u32,
}

impl DxfInsertAttributeRecordRange {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeSequenceEntry {
    insert: DxfInsertRecordValueEntry,
    attributes_follow_value: Option<i16>,
    attribute_range: DxfInsertAttributeRecordRange,
    boundary_record: Option<DxfRawRecord>,
    state: DxfInsertAttributeSequenceState,
}

impl DxfInsertAttributeSequenceEntry {
    #[must_use]
    pub const fn insert(self) -> DxfInsertRecordValueEntry {
        self.insert
    }

    #[must_use]
    pub const fn attributes_follow_value(self) -> Option<i16> {
        self.attributes_follow_value
    }

    #[must_use]
    pub const fn attribute_range(self) -> DxfInsertAttributeRecordRange {
        self.attribute_range
    }

    /// Exact SEQEND when closed or first unexpected record when interrupted.
    #[must_use]
    pub const fn boundary_record(self) -> Option<DxfRawRecord> {
        self.boundary_record
    }

    #[must_use]
    pub const fn state(self) -> DxfInsertAttributeSequenceState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributeSequenceDirectory {
    source_id: DxfSourceId,
    semantics: DxfInsertRecordSemanticDirectory,
    entries: Box<[DxfInsertAttributeSequenceEntry]>,
    attributes: Box<[DxfRawRecord]>,
}

impl DxfInsertAttributeSequenceDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.insert_record_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }
        let raw_records = semantics
            .card_directory()
            .evidence_directory()
            .raw_record_directory();
        let mut entries = Vec::new();
        let mut attributes = Vec::new();
        entries
            .try_reserve(semantics.records().len())
            .map_err(|_| out_of_memory())?;
        for insert in semantics.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(derive_sequence(
                document,
                &semantics,
                raw_records,
                insert,
                cancellation,
                &mut attributes,
            )?);
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            entries: entries.into_boxed_slice(),
            attributes: attributes.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfInsertRecordSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfInsertAttributeSequenceEntry] {
        &self.entries
    }

    #[must_use]
    pub fn attribute_records(&self) -> &[DxfRawRecord] {
        &self.attributes
    }

    #[must_use]
    pub fn entry_for_insert_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertAttributeSequenceEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.insert().record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn attributes_for_insert_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfRawRecord]> {
        let entry = self.entry_for_insert_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.attribute_range().start()).ok()?;
        let end = usize::try_from(entry.attribute_range().end()).ok()?;
        self.attributes.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_attribute_sequence_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeSequenceDirectory, DxfError> {
        DxfInsertAttributeSequenceDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_sequence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeSequenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_sequence_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_sequence_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeSequenceDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_attribute_sequence_directory(cancellation)
    }
}

fn derive_sequence(
    document: DxfRawDocumentView<'_>,
    semantics: &DxfInsertRecordSemanticDirectory,
    raw_records: &DxfRawRecordDirectory,
    insert: DxfInsertRecordValueEntry,
    cancellation: &DxfCancellationToken,
    attributes: &mut Vec<DxfRawRecord>,
) -> Result<DxfInsertAttributeSequenceEntry, DxfError> {
    let semantic = semantics
        .semantics_for_entry(insert)?
        .ok_or_else(invalid_internal_data)?;
    let attributes_follow_value = semantic.attributes_follow_value();
    let start = compact_len(attributes.len())?;
    let (boundary_record, state) = match attributes_follow_value {
        None => (None, DxfInsertAttributeSequenceState::FlagUnavailable),
        Some(0) => (None, DxfInsertAttributeSequenceState::NoAttributesFollow),
        Some(_) => scan_following_attributes(
            document,
            raw_records,
            insert.record(),
            cancellation,
            attributes,
        )?,
    };
    let end = compact_len(attributes.len())?;
    Ok(DxfInsertAttributeSequenceEntry {
        insert,
        attributes_follow_value,
        attribute_range: DxfInsertAttributeRecordRange::new(start, end)?,
        boundary_record,
        state,
    })
}

fn scan_following_attributes(
    document: DxfRawDocumentView<'_>,
    raw_records: &DxfRawRecordDirectory,
    insert: DxfRawRecord,
    cancellation: &DxfCancellationToken,
    attributes: &mut Vec<DxfRawRecord>,
) -> Result<(Option<DxfRawRecord>, DxfInsertAttributeSequenceState), DxfError> {
    let mut cursor = usize::try_from(insert.ordinal())
        .map_err(|_| invalid_internal_data())?
        .checked_add(1)
        .ok_or_else(invalid_internal_data)?;
    while let Some(candidate) = raw_records.records().get(cursor).copied() {
        ensure_not_cancelled(cancellation)?;
        if candidate.structure_section_ordinal() != insert.structure_section_ordinal()
            || marker_kind(document, candidate)? != InsertSequenceMarkerKind::Attrib
        {
            break;
        }
        attributes.try_reserve(1).map_err(|_| out_of_memory())?;
        attributes.push(candidate);
        cursor = cursor.checked_add(1).ok_or_else(invalid_internal_data)?;
    }
    let boundary = raw_records
        .records()
        .get(cursor)
        .copied()
        .filter(|candidate| {
            candidate.structure_section_ordinal() == insert.structure_section_ordinal()
        });
    match boundary {
        Some(record) if marker_kind(document, record)? == InsertSequenceMarkerKind::Seqend => {
            Ok((Some(record), DxfInsertAttributeSequenceState::Closed))
        }
        Some(record) => Ok((Some(record), DxfInsertAttributeSequenceState::Interrupted)),
        None => Ok((None, DxfInsertAttributeSequenceState::Unclosed)),
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum InsertSequenceMarkerKind {
    Attrib,
    Seqend,
    Other,
}

fn marker_kind(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<InsertSequenceMarkerKind, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    let span = marker.value_payload_span();
    if span.len() == b"ATTRIB".len() as u64 && document.raw_span_equals_exact(span, b"ATTRIB")? {
        return Ok(InsertSequenceMarkerKind::Attrib);
    }
    if span.len() == b"SEQEND".len() as u64 && document.raw_span_equals_exact(span, b"SEQEND")? {
        return Ok(InsertSequenceMarkerKind::Seqend);
    }
    Ok(InsertSequenceMarkerKind::Other)
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
    DxfError::from_io(
        crate::DxfIoOperation::Read,
        &std::io::Error::from(std::io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        crate::DxfIoOperation::Read,
        &std::io::Error::from(std::io::ErrorKind::OutOfMemory),
    )
}

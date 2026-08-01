//! Exact size relation for opaque common proxy-graphics chunks.

use std::io;

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEntityCommonFieldDomainDirectory, DxfEntityCommonFieldDomainSemanticIssue,
    DxfEntityCommonFieldDomainSemantics, DxfEntityCommonFieldDomainValue, DxfEntityField,
    DxfEntityFieldSemantics, DxfEntityRef, DxfError, DxfIoOperation, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfRawValueProvenance, DxfSemanticValue, DxfSourceId,
};

const SCAN_CHUNK_BYTES: usize = 256;

/// Why one ASCII group-310 chunk cannot be counted exactly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityProxyGraphicsChunkIssue {
    OddHexLength { encoded_bytes: u64 },
    InvalidHexDigit { byte_offset: u64 },
}

/// Relation between the declared group-92 byte count and opaque group-310 data.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityProxyGraphicsState {
    Absent,
    Matched {
        declared_bytes: u32,
        payload_bytes: u64,
        chunk_count: u32,
    },
    MissingSize {
        payload_bytes: u64,
        chunk_count: u32,
    },
    CountMismatch {
        declared_bytes: u32,
        payload_bytes: u64,
        chunk_count: u32,
    },
    InvalidSize {
        issue: DxfEntityCommonFieldDomainSemanticIssue,
    },
    InvalidChunk {
        sequence_ordinal: u32,
        raw: DxfRawValueProvenance,
        issue: DxfEntityProxyGraphicsChunkIssue,
    },
}

/// One source entity and its proxy-graphics size relation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityProxyGraphicsEntry {
    ordinal: u32,
    entity: DxfEntityRef,
    state: DxfEntityProxyGraphicsState,
}

impl DxfEntityProxyGraphicsEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn entity(self) -> DxfEntityRef {
        self.entity
    }

    #[must_use]
    pub const fn state(self) -> DxfEntityProxyGraphicsState {
        self.state
    }
}

/// Source-bound proxy-graphics relations for all semantic entities.
#[derive(Debug)]
pub struct DxfEntityProxyGraphicsDirectory {
    source_id: DxfSourceId,
    domains: DxfEntityCommonFieldDomainDirectory,
    entries: Box<[DxfEntityProxyGraphicsEntry]>,
}

impl DxfEntityProxyGraphicsDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let domains = document.entity_common_field_domain_directory(cancellation)?;
        ensure_source(document.source_id(), domains.source_id())?;
        let mut entries = Vec::new();
        let entity_count = domains
            .entries()
            .iter()
            .filter(|entry| entry.field() == DxfEntityField::PROXY_GRAPHICS_SIZE)
            .count();
        entries
            .try_reserve_exact(entity_count)
            .map_err(|_| out_of_memory())?;
        for size_entry in domains
            .entries()
            .iter()
            .copied()
            .filter(|entry| entry.field() == DxfEntityField::PROXY_GRAPHICS_SIZE)
        {
            ensure_not_cancelled(cancellation)?;
            let entity = size_entry.entity();
            let data_entry = domains
                .entry_for_field(entity, DxfEntityField::PROXY_GRAPHICS_DATA)?
                .ok_or_else(invalid_internal_data)?;
            let state = project_relation(
                document,
                &domains,
                size_entry.semantics(),
                data_entry,
                cancellation,
            )?;
            entries.push(DxfEntityProxyGraphicsEntry {
                ordinal: compact_len(entries.len())?,
                entity,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            domains,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn domain_directory(&self) -> &DxfEntityCommonFieldDomainDirectory {
        &self.domains
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfEntityProxyGraphicsEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfEntityProxyGraphicsEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    pub fn entry_for_entity(
        &self,
        entity: DxfEntityRef,
    ) -> Result<Option<DxfEntityProxyGraphicsEntry>, DxfError> {
        ensure_source(self.source_id, entity.source_id())?;
        Ok(self
            .entries
            .iter()
            .copied()
            .find(|entry| entry.entity().record().ordinal() == entity.record().ordinal()))
    }
}

impl DxfRawDocumentView<'_> {
    pub fn entity_proxy_graphics_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityProxyGraphicsDirectory, DxfError> {
        DxfEntityProxyGraphicsDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn entity_proxy_graphics_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityProxyGraphicsDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_proxy_graphics_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn entity_proxy_graphics_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityProxyGraphicsDirectory, DxfError> {
        DxfRawDocumentView::from(self).entity_proxy_graphics_directory(cancellation)
    }
}

fn project_relation(
    document: DxfRawDocumentView<'_>,
    domains: &DxfEntityCommonFieldDomainDirectory,
    size: DxfEntityCommonFieldDomainSemantics,
    data: crate::DxfEntityCommonFieldDomainEntry,
    cancellation: &DxfCancellationToken,
) -> Result<DxfEntityProxyGraphicsState, DxfError> {
    let DxfEntityCommonFieldDomainSemantics::Unreviewed(DxfEntityFieldSemantics::OpaqueSequence {
        occurrence_count,
    }) = data.semantics()
    else {
        return Err(invalid_internal_data());
    };
    let source = data.source_semantics();
    let members = domains
        .source_directory()
        .members_for_opaque_sequence(source)?
        .ok_or_else(invalid_internal_data)?;
    if u64::from(occurrence_count) != members.len() as u64 {
        return Err(invalid_internal_data());
    }
    let mut payload_bytes = 0_u64;
    for (index, member) in members.iter().copied().enumerate() {
        ensure_not_cancelled(cancellation)?;
        let occurrence = domains
            .source_directory()
            .evidence_directory()
            .occurrence_for_member(member)
            .ok_or_else(invalid_internal_data)?;
        let group = occurrence.group();
        let raw = DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
            .ok_or_else(invalid_internal_data)?;
        let chunk_bytes = match count_chunk(document, group.value_payload_span(), cancellation)? {
            Ok(value) => value,
            Err(issue) => {
                return Ok(DxfEntityProxyGraphicsState::InvalidChunk {
                    sequence_ordinal: compact_len(index)?,
                    raw,
                    issue,
                });
            }
        };
        payload_bytes = payload_bytes
            .checked_add(chunk_bytes)
            .ok_or_else(invalid_internal_data)?;
    }
    let declared = match size {
        DxfEntityCommonFieldDomainSemantics::Reviewed(DxfSemanticValue::Explicit {
            value: DxfEntityCommonFieldDomainValue::ProxyGraphicsSize(value),
            ..
        })
        | DxfEntityCommonFieldDomainSemantics::Reviewed(DxfSemanticValue::Defaulted {
            value: DxfEntityCommonFieldDomainValue::ProxyGraphicsSize(value),
            ..
        }) => Some(value),
        DxfEntityCommonFieldDomainSemantics::Reviewed(DxfSemanticValue::Absent { .. }) => None,
        DxfEntityCommonFieldDomainSemantics::Reviewed(DxfSemanticValue::Invalid {
            issue, ..
        }) => return Ok(DxfEntityProxyGraphicsState::InvalidSize { issue }),
        _ => return Err(invalid_internal_data()),
    };
    Ok(match declared {
        None if occurrence_count == 0 => DxfEntityProxyGraphicsState::Absent,
        None => DxfEntityProxyGraphicsState::MissingSize {
            payload_bytes,
            chunk_count: occurrence_count,
        },
        Some(declared_bytes) if u64::from(declared_bytes) == payload_bytes => {
            DxfEntityProxyGraphicsState::Matched {
                declared_bytes,
                payload_bytes,
                chunk_count: occurrence_count,
            }
        }
        Some(declared_bytes) => DxfEntityProxyGraphicsState::CountMismatch {
            declared_bytes,
            payload_bytes,
            chunk_count: occurrence_count,
        },
    })
}

fn count_chunk(
    document: DxfRawDocumentView<'_>,
    span: ByteSpan,
    cancellation: &DxfCancellationToken,
) -> Result<Result<u64, DxfEntityProxyGraphicsChunkIssue>, DxfError> {
    if document.format() == DxfRawDocumentFormat::Binary {
        return Ok(Ok(span.len()));
    }
    if !span.len().is_multiple_of(2) {
        return Ok(Err(DxfEntityProxyGraphicsChunkIssue::OddHexLength {
            encoded_bytes: span.len(),
        }));
    }
    let mut buffer = [0_u8; SCAN_CHUNK_BYTES];
    let mut scanned = 0_u64;
    while scanned < span.len() {
        ensure_not_cancelled(cancellation)?;
        let remaining = span.len() - scanned;
        let take = usize::try_from(remaining.min(SCAN_CHUNK_BYTES as u64))
            .map_err(|_| invalid_internal_data())?;
        let part = ByteSpan::from_start_and_len(span.start() + scanned, take as u64)
            .ok_or_else(invalid_internal_data)?;
        document.read_span(part, &mut buffer[..take])?;
        if let Some(offset) = buffer[..take]
            .iter()
            .position(|byte| !byte.is_ascii_hexdigit())
        {
            return Ok(Err(DxfEntityProxyGraphicsChunkIssue::InvalidHexDigit {
                byte_offset: scanned + offset as u64,
            }));
        }
        scanned += take as u64;
    }
    Ok(Ok(span.len() / 2))
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

fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
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

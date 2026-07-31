//! Duplicate-preserving block-local exact ATTDEF-tag lookup.

use std::io;

use sha2::{Digest, Sha256};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockAttributeDefinitionTextSemanticDirectory, DxfBlockAttributeDefinitionTextValue,
    DxfBlockAttributeDefinitionValueEntry, DxfBlockDefinitionEntry, DxfCancellationToken, DxfError,
    DxfIoOperation, DxfRawDocumentView, DxfSourceId,
    source_span::{sha256_span, span_equals_bytes, spans_equal},
};
type TagDigest = [u8; 32];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionTagIndexBlock {
    block: DxfBlockDefinitionEntry,
    definition_count: u32,
    indexed_tag_count: u32,
    unusable_tag_count: u32,
}

impl DxfBlockAttributeDefinitionTagIndexBlock {
    #[must_use]
    pub const fn block(self) -> DxfBlockDefinitionEntry {
        self.block
    }

    #[must_use]
    pub const fn definition_count(self) -> u64 {
        self.definition_count as u64
    }

    #[must_use]
    pub const fn indexed_tag_count(self) -> u64 {
        self.indexed_tag_count as u64
    }

    #[must_use]
    pub const fn unusable_tag_count(self) -> u64 {
        self.unusable_tag_count as u64
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockAttributeDefinitionTagIndexMatch {
    record: DxfBlockAttributeDefinitionValueEntry,
    tag: DxfBlockAttributeDefinitionTextValue,
}

impl DxfBlockAttributeDefinitionTagIndexMatch {
    #[must_use]
    pub const fn record(self) -> DxfBlockAttributeDefinitionValueEntry {
        self.record
    }

    #[must_use]
    pub const fn tag(self) -> DxfBlockAttributeDefinitionTextValue {
        self.tag
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DxfBlockAttributeDefinitionTagIndexLookup<'a> {
    Missing,
    Unique(DxfBlockAttributeDefinitionTagIndexMatch),
    Ambiguous(&'a [DxfBlockAttributeDefinitionTagIndexMatch]),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TagIndexKey {
    block_raw_ordinal: u64,
    digest: TagDigest,
    exact_group_ordinal: u32,
}

#[derive(Clone, Copy)]
struct PendingTag {
    digest: TagDigest,
    exact_group_ordinal: u32,
    indexed: DxfBlockAttributeDefinitionTagIndexMatch,
}

#[derive(Clone, Copy)]
struct ExactRepresentative {
    span: ByteSpan,
    ordinal: u32,
}

#[derive(Debug)]
pub struct DxfBlockAttributeDefinitionTagIndexDirectory {
    source_id: DxfSourceId,
    semantics: DxfBlockAttributeDefinitionTextSemanticDirectory,
    blocks: Box<[DxfBlockAttributeDefinitionTagIndexBlock]>,
    keys: Box<[TagIndexKey]>,
    matches: Box<[DxfBlockAttributeDefinitionTagIndexMatch]>,
}

impl DxfBlockAttributeDefinitionTagIndexDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics =
            document.block_attribute_definition_text_semantic_directory(cancellation)?;
        if semantics.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: semantics.source_id(),
            });
        }

        let definitions = semantics
            .card_directory()
            .evidence_directory()
            .definition_directory();
        let mut blocks = Vec::new();
        let mut pending = Vec::new();
        let all_blocks = definitions.block_definition_directory().definitions();
        blocks
            .try_reserve(all_blocks.len())
            .map_err(|_| out_of_memory())?;
        for block in all_blocks.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let records = definitions.entries_for_block_raw_ordinal(block.block_record().ordinal());
            let mut indexed_tag_count = 0_u32;
            let mut unusable_tag_count = 0_u32;
            for definition in records.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let projected = semantics
                    .semantics_for_raw_record(definition.record().ordinal())?
                    .ok_or_else(invalid_internal_data)?;
                let Some(tag) = projected.attribute_tag().value().copied() else {
                    unusable_tag_count = unusable_tag_count
                        .checked_add(1)
                        .ok_or_else(invalid_internal_data)?;
                    continue;
                };
                if tag.source_id() != document.source_id() {
                    return Err(DxfError::SourceIdentityMismatch {
                        expected: document.source_id(),
                        observed: tag.source_id(),
                    });
                }
                pending.try_reserve(1).map_err(|_| out_of_memory())?;
                pending.push(PendingTag {
                    digest: sha256_span(document, tag.value_span(), cancellation)?,
                    exact_group_ordinal: 0,
                    indexed: DxfBlockAttributeDefinitionTagIndexMatch {
                        record: projected.record(),
                        tag,
                    },
                });
                indexed_tag_count = indexed_tag_count
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
            }
            blocks.push(DxfBlockAttributeDefinitionTagIndexBlock {
                block,
                definition_count: compact_len(records.len())?,
                indexed_tag_count,
                unusable_tag_count,
            });
        }

        classify_exact_groups(document, &mut pending, cancellation)?;
        pending.sort_unstable_by_key(|candidate| {
            (
                candidate
                    .indexed
                    .record()
                    .definition()
                    .owner()
                    .block_record()
                    .ordinal(),
                candidate.digest,
                candidate.exact_group_ordinal,
                candidate
                    .indexed
                    .record()
                    .definition()
                    .attribute_definition_ordinal(),
            )
        });
        let mut keys = Vec::new();
        let mut matches = Vec::new();
        keys.try_reserve(pending.len())
            .map_err(|_| out_of_memory())?;
        matches
            .try_reserve(pending.len())
            .map_err(|_| out_of_memory())?;
        for candidate in pending {
            keys.push(TagIndexKey {
                block_raw_ordinal: candidate
                    .indexed
                    .record()
                    .definition()
                    .owner()
                    .block_record()
                    .ordinal(),
                digest: candidate.digest,
                exact_group_ordinal: candidate.exact_group_ordinal,
            });
            matches.push(candidate.indexed);
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            blocks: blocks.into_boxed_slice(),
            keys: keys.into_boxed_slice(),
            matches: matches.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfBlockAttributeDefinitionTextSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn blocks(&self) -> &[DxfBlockAttributeDefinitionTagIndexBlock] {
        &self.blocks
    }

    #[must_use]
    pub fn matches(&self) -> &[DxfBlockAttributeDefinitionTagIndexMatch] {
        &self.matches
    }

    #[must_use]
    pub fn block_for_raw_ordinal(
        &self,
        block_raw_ordinal: u64,
    ) -> Option<DxfBlockAttributeDefinitionTagIndexBlock> {
        self.blocks
            .binary_search_by_key(&block_raw_ordinal, |entry| {
                entry.block().block_record().ordinal()
            })
            .ok()
            .and_then(|index| self.blocks.get(index).copied())
    }

    pub fn matches_for_block_exact_tag<'a>(
        &'a self,
        document: DxfRawDocumentView<'_>,
        block_raw_ordinal: u64,
        exact_tag: &[u8],
        cancellation: &DxfCancellationToken,
    ) -> Result<&'a [DxfBlockAttributeDefinitionTagIndexMatch], DxfError> {
        self.ensure_document(document)?;
        ensure_not_cancelled(cancellation)?;
        self.matches_for_digest(
            document,
            block_raw_ordinal,
            digest_bytes(exact_tag),
            cancellation,
            |span, cancellation| span_equals_bytes(document, span, exact_tag, cancellation),
        )
    }

    pub fn matches_for_block_exact_source_span<'a>(
        &'a self,
        document: DxfRawDocumentView<'_>,
        block_raw_ordinal: u64,
        exact_span: ByteSpan,
        cancellation: &DxfCancellationToken,
    ) -> Result<&'a [DxfBlockAttributeDefinitionTagIndexMatch], DxfError> {
        self.ensure_document(document)?;
        ensure_not_cancelled(cancellation)?;
        self.matches_for_digest(
            document,
            block_raw_ordinal,
            sha256_span(document, exact_span, cancellation)?,
            cancellation,
            |span, cancellation| spans_equal(document, span, exact_span, cancellation),
        )
    }

    pub fn lookup<'a>(
        &'a self,
        document: DxfRawDocumentView<'_>,
        block_raw_ordinal: u64,
        exact_tag: &[u8],
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionTagIndexLookup<'a>, DxfError> {
        Ok(
            match self.matches_for_block_exact_tag(
                document,
                block_raw_ordinal,
                exact_tag,
                cancellation,
            )? {
                [] => DxfBlockAttributeDefinitionTagIndexLookup::Missing,
                [indexed] => DxfBlockAttributeDefinitionTagIndexLookup::Unique(*indexed),
                matches => DxfBlockAttributeDefinitionTagIndexLookup::Ambiguous(matches),
            },
        )
    }

    fn matches_for_digest<'a>(
        &'a self,
        _document: DxfRawDocumentView<'_>,
        block_raw_ordinal: u64,
        digest: TagDigest,
        cancellation: &DxfCancellationToken,
        mut exact: impl FnMut(ByteSpan, &DxfCancellationToken) -> Result<bool, DxfError>,
    ) -> Result<&'a [DxfBlockAttributeDefinitionTagIndexMatch], DxfError> {
        let (start, end) = self.candidate_bounds(block_raw_ordinal, digest);
        let mut cursor = start;
        while cursor < end {
            ensure_not_cancelled(cancellation)?;
            let key = *self.keys.get(cursor).ok_or_else(invalid_internal_data)?;
            let relative_end = self.keys[cursor..end].partition_point(|candidate| {
                candidate.exact_group_ordinal == key.exact_group_ordinal
            });
            let group_end = cursor
                .checked_add(relative_end)
                .ok_or_else(invalid_internal_data)?;
            let representative = self
                .matches
                .get(cursor)
                .copied()
                .ok_or_else(invalid_internal_data)?;
            if exact(representative.tag().value_span(), cancellation)? {
                return self
                    .matches
                    .get(cursor..group_end)
                    .ok_or_else(invalid_internal_data);
            }
            if group_end <= cursor {
                return Err(invalid_internal_data());
            }
            cursor = group_end;
        }
        ensure_not_cancelled(cancellation)?;
        Ok(self.matches.get(0..0).map_or(&[], |matches| matches))
    }

    fn candidate_bounds(&self, block_raw_ordinal: u64, digest: TagDigest) -> (usize, usize) {
        (
            self.keys.partition_point(|key| {
                (key.block_raw_ordinal, key.digest) < (block_raw_ordinal, digest)
            }),
            self.keys.partition_point(|key| {
                (key.block_raw_ordinal, key.digest) <= (block_raw_ordinal, digest)
            }),
        )
    }

    fn ensure_document(&self, document: DxfRawDocumentView<'_>) -> Result<(), DxfError> {
        if document.source_id() == self.source_id {
            Ok(())
        } else {
            Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: document.source_id(),
            })
        }
    }
}

impl DxfRawDocumentView<'_> {
    pub fn block_attribute_definition_tag_index_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionTagIndexDirectory, DxfError> {
        DxfBlockAttributeDefinitionTagIndexDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn block_attribute_definition_tag_index_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionTagIndexDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_tag_index_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn block_attribute_definition_tag_index_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfBlockAttributeDefinitionTagIndexDirectory, DxfError> {
        DxfRawDocumentView::from(self).block_attribute_definition_tag_index_directory(cancellation)
    }
}

fn classify_exact_groups(
    document: DxfRawDocumentView<'_>,
    pending: &mut [PendingTag],
    cancellation: &DxfCancellationToken,
) -> Result<(), DxfError> {
    pending.sort_unstable_by_key(|candidate| {
        (
            candidate.digest,
            candidate.indexed.record().definition().record().ordinal(),
        )
    });
    let mut representatives: Vec<ExactRepresentative> = Vec::new();
    let mut start = 0_usize;
    while start < pending.len() {
        ensure_not_cancelled(cancellation)?;
        let digest = pending.get(start).ok_or_else(invalid_internal_data)?.digest;
        let relative_end = pending[start..].partition_point(|candidate| candidate.digest == digest);
        let end = start
            .checked_add(relative_end)
            .ok_or_else(invalid_internal_data)?;
        representatives.clear();
        for candidate in pending
            .get_mut(start..end)
            .ok_or_else(invalid_internal_data)?
        {
            let span = candidate.indexed.tag().value_span();
            let mut exact_group_ordinal = None;
            for representative in representatives.iter().copied() {
                if spans_equal(document, representative.span, span, cancellation)? {
                    exact_group_ordinal = Some(representative.ordinal);
                    break;
                }
            }
            candidate.exact_group_ordinal = match exact_group_ordinal {
                Some(ordinal) => ordinal,
                None => {
                    let ordinal = compact_len(representatives.len())?;
                    representatives
                        .try_reserve(1)
                        .map_err(|_| out_of_memory())?;
                    representatives.push(ExactRepresentative { span, ordinal });
                    ordinal
                }
            };
        }
        if end <= start {
            return Err(invalid_internal_data());
        }
        start = end;
    }
    Ok(())
}

fn digest_bytes(bytes: &[u8]) -> TagDigest {
    Sha256::digest(bytes).into()
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

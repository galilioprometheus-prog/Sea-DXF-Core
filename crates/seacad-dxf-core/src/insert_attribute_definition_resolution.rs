//! Exact ATTRIB-to-ATTDEF resolution within uniquely targeted BLOCK definitions.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockAttributeDefinitionTagIndexDirectory,
    DxfBlockAttributeDefinitionTagIndexMatch, DxfBlockNameIndexMatch, DxfCancellationToken,
    DxfError, DxfInsertAttributeTextSemanticDirectory, DxfInsertAttributeValueEntry,
    DxfInsertBlockResolutionDirectory, DxfInsertBlockResolutionState, DxfIoOperation,
    DxfRawDocumentView, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertAttributeDefinitionResolutionState {
    TargetUnavailable {
        resolution: DxfInsertBlockResolutionState,
    },
    TagUnavailable,
    Missing,
    Indeterminate {
        unusable_definition_count: u32,
    },
    Unique,
    Ambiguous {
        definition_count: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertAttributeDefinitionRange {
    start: u32,
    end: u32,
}

impl DxfInsertAttributeDefinitionRange {
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
pub struct DxfInsertAttributeDefinitionResolutionEntry {
    attribute: DxfInsertAttributeValueEntry,
    target: Option<DxfBlockNameIndexMatch>,
    definition_range: DxfInsertAttributeDefinitionRange,
    state: DxfInsertAttributeDefinitionResolutionState,
}

impl DxfInsertAttributeDefinitionResolutionEntry {
    #[must_use]
    pub const fn attribute(self) -> DxfInsertAttributeValueEntry {
        self.attribute
    }

    #[must_use]
    pub const fn target(self) -> Option<DxfBlockNameIndexMatch> {
        self.target
    }

    #[must_use]
    pub const fn definition_range(self) -> DxfInsertAttributeDefinitionRange {
        self.definition_range
    }

    #[must_use]
    pub const fn state(self) -> DxfInsertAttributeDefinitionResolutionState {
        self.state
    }
}

#[derive(Debug)]
pub struct DxfInsertAttributeDefinitionResolutionDirectory {
    source_id: DxfSourceId,
    attributes: DxfInsertAttributeTextSemanticDirectory,
    insert_resolutions: DxfInsertBlockResolutionDirectory,
    definition_tags: DxfBlockAttributeDefinitionTagIndexDirectory,
    entries: Box<[DxfInsertAttributeDefinitionResolutionEntry]>,
    definitions: Box<[DxfBlockAttributeDefinitionTagIndexMatch]>,
}

impl DxfInsertAttributeDefinitionResolutionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let attributes = document.insert_attribute_text_semantic_directory(cancellation)?;
        let insert_resolutions = document.insert_block_resolution_directory(cancellation)?;
        let definition_tags =
            document.block_attribute_definition_tag_index_directory(cancellation)?;
        for observed in [
            attributes.source_id(),
            insert_resolutions.source_id(),
            definition_tags.source_id(),
        ] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }

        let mut entries = Vec::new();
        let mut definitions = Vec::new();
        entries
            .try_reserve(attributes.records().len())
            .map_err(|_| out_of_memory())?;
        for attribute in attributes.records().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let start = compact_len(definitions.len())?;
            let insert_raw_ordinal = attribute.sequence().insert().record().ordinal();
            let resolution = insert_resolutions
                .entry_for_insert_raw_ordinal(insert_raw_ordinal)
                .ok_or_else(invalid_internal_data)?;
            let (target, state) = if resolution.state() != DxfInsertBlockResolutionState::Unique {
                (
                    None,
                    DxfInsertAttributeDefinitionResolutionState::TargetUnavailable {
                        resolution: resolution.state(),
                    },
                )
            } else {
                let targets = insert_resolutions
                    .targets_for_insert_raw_ordinal(insert_raw_ordinal)
                    .ok_or_else(invalid_internal_data)?;
                let [target] = targets else {
                    return Err(invalid_internal_data());
                };
                let semantic = attributes
                    .semantics_for_entry(attribute)?
                    .ok_or_else(invalid_internal_data)?;
                if let Some(tag) = semantic.attribute_tag().value().copied() {
                    let block_raw_ordinal = target.record().definition().block_record().ordinal();
                    let matches = definition_tags.matches_for_block_exact_source_span(
                        document,
                        block_raw_ordinal,
                        tag.value_span(),
                        cancellation,
                    )?;
                    definitions
                        .try_reserve(matches.len())
                        .map_err(|_| out_of_memory())?;
                    definitions.extend_from_slice(matches);
                    let state = match matches {
                        [] => {
                            let block = definition_tags
                                .block_for_raw_ordinal(block_raw_ordinal)
                                .ok_or_else(invalid_internal_data)?;
                            match u32::try_from(block.unusable_tag_count()) {
                                Ok(0) => DxfInsertAttributeDefinitionResolutionState::Missing,
                                Ok(unusable_definition_count) => {
                                    DxfInsertAttributeDefinitionResolutionState::Indeterminate {
                                        unusable_definition_count,
                                    }
                                }
                                Err(_) => return Err(invalid_internal_data()),
                            }
                        }
                        [_] => DxfInsertAttributeDefinitionResolutionState::Unique,
                        multiple => DxfInsertAttributeDefinitionResolutionState::Ambiguous {
                            definition_count: compact_len(multiple.len())?,
                        },
                    };
                    (Some(*target), state)
                } else {
                    (
                        Some(*target),
                        DxfInsertAttributeDefinitionResolutionState::TagUnavailable,
                    )
                }
            };
            let end = compact_len(definitions.len())?;
            entries.push(DxfInsertAttributeDefinitionResolutionEntry {
                attribute,
                target,
                definition_range: DxfInsertAttributeDefinitionRange::new(start, end)?,
                state,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            attributes,
            insert_resolutions,
            definition_tags,
            entries: entries.into_boxed_slice(),
            definitions: definitions.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn attribute_text_semantic_directory(
        &self,
    ) -> &DxfInsertAttributeTextSemanticDirectory {
        &self.attributes
    }

    #[must_use]
    pub const fn insert_block_resolution_directory(&self) -> &DxfInsertBlockResolutionDirectory {
        &self.insert_resolutions
    }

    #[must_use]
    pub const fn definition_tag_index_directory(
        &self,
    ) -> &DxfBlockAttributeDefinitionTagIndexDirectory {
        &self.definition_tags
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfInsertAttributeDefinitionResolutionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn definitions(&self) -> &[DxfBlockAttributeDefinitionTagIndexMatch] {
        &self.definitions
    }

    #[must_use]
    pub fn entry_for_attribute_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertAttributeDefinitionResolutionEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.attribute().record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn definitions_for_attribute_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBlockAttributeDefinitionTagIndexMatch]> {
        let entry = self.entry_for_attribute_raw_ordinal(raw_record_ordinal)?;
        let start = usize::try_from(entry.definition_range().start()).ok()?;
        let end = usize::try_from(entry.definition_range().end()).ok()?;
        self.definitions.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_attribute_definition_resolution_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeDefinitionResolutionDirectory, DxfError> {
        DxfInsertAttributeDefinitionResolutionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_attribute_definition_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeDefinitionResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .insert_attribute_definition_resolution_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_attribute_definition_resolution_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertAttributeDefinitionResolutionDirectory, DxfError> {
        DxfRawDocumentView::from(self)
            .insert_attribute_definition_resolution_directory(cancellation)
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
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

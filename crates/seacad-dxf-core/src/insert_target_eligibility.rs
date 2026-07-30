//! Closed-target and recursive-expansion evidence for resolved INSERT records.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfBlockDefinitionDirectory,
    DxfBlockDefinitionEntry, DxfBlockDefinitionState, DxfBlockNameIndexMatch, DxfCancellationToken,
    DxfError, DxfInsertBlockResolutionDirectory, DxfInsertBlockResolutionEntry,
    DxfInsertBlockResolutionState, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
};

/// Expansion eligibility of one exact INSERT after name resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertTargetEligibilityState {
    NotUniquelyResolved,
    TargetDefinitionNotClosed { state: DxfBlockDefinitionState },
    RecursiveExpansion,
    Eligible,
}

/// One exact INSERT resolution and its reviewed expansion eligibility.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertTargetEligibilityEntry {
    resolution: DxfInsertBlockResolutionEntry,
    state: DxfInsertTargetEligibilityState,
}

impl DxfInsertTargetEligibilityEntry {
    #[must_use]
    pub const fn resolution(self) -> DxfInsertBlockResolutionEntry {
        self.resolution
    }

    #[must_use]
    pub const fn state(self) -> DxfInsertTargetEligibilityState {
        self.state
    }
}

/// One uniquely resolved INSERT member edge in the BLOCK expansion graph.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfBlockExpansionEdge {
    owner: DxfBlockDefinitionEntry,
    insert: DxfInsertBlockResolutionEntry,
    target: DxfBlockNameIndexMatch,
}

impl DxfBlockExpansionEdge {
    #[must_use]
    pub const fn owner(self) -> DxfBlockDefinitionEntry {
        self.owner
    }

    #[must_use]
    pub const fn insert(self) -> DxfInsertBlockResolutionEntry {
        self.insert
    }

    #[must_use]
    pub const fn target(self) -> DxfBlockNameIndexMatch {
        self.target
    }
}

/// Immutable INSERT target eligibility and bounded recursion evidence.
#[derive(Debug)]
pub struct DxfInsertTargetEligibilityDirectory {
    source_id: DxfSourceId,
    resolutions: DxfInsertBlockResolutionDirectory,
    entries: Box<[DxfInsertTargetEligibilityEntry]>,
    expansion_edges: Box<[DxfBlockExpansionEdge]>,
}

impl DxfInsertTargetEligibilityDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let resolutions = document.insert_block_resolution_directory(cancellation)?;
        if resolutions.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: resolutions.source_id(),
            });
        }
        let definitions = definition_directory(&resolutions);
        if definitions.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: definitions.source_id(),
            });
        }

        let expansion_edges = build_expansion_edges(&resolutions, definitions, cancellation)?;
        let mut entries = Vec::new();
        entries
            .try_reserve(resolutions.entries().len())
            .map_err(|_| out_of_memory())?;
        for resolution in resolutions.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let state = eligibility_state(
                &resolutions,
                definitions,
                &expansion_edges,
                resolution,
                cancellation,
            )?;
            entries.push(DxfInsertTargetEligibilityEntry { resolution, state });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            resolutions,
            entries: entries.into_boxed_slice(),
            expansion_edges: expansion_edges.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn resolution_directory(&self) -> &DxfInsertBlockResolutionDirectory {
        &self.resolutions
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfInsertTargetEligibilityEntry] {
        &self.entries
    }

    #[must_use]
    pub fn expansion_edges(&self) -> &[DxfBlockExpansionEdge] {
        &self.expansion_edges
    }

    #[must_use]
    pub fn entry_for_insert_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertTargetEligibilityEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry.resolution().insert().record().ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn expansion_edges_for_block_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<&[DxfBlockExpansionEdge]> {
        definition_directory(&self.resolutions)
            .definition_for_block_raw_ordinal(raw_record_ordinal)?;
        let start = self
            .expansion_edges
            .partition_point(|edge| edge.owner().block_record().ordinal() < raw_record_ordinal);
        let end = self
            .expansion_edges
            .partition_point(|edge| edge.owner().block_record().ordinal() <= raw_record_ordinal);
        self.expansion_edges.get(start..end)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_target_eligibility_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertTargetEligibilityDirectory, DxfError> {
        DxfInsertTargetEligibilityDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_target_eligibility_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertTargetEligibilityDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_target_eligibility_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_target_eligibility_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertTargetEligibilityDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_target_eligibility_directory(cancellation)
    }
}

fn definition_directory(
    resolutions: &DxfInsertBlockResolutionDirectory,
) -> &DxfBlockDefinitionDirectory {
    resolutions
        .block_name_index_directory()
        .consistency_directory()
        .semantic_directory()
        .card_directory()
        .evidence_directory()
        .definition_directory()
}

fn build_expansion_edges(
    resolutions: &DxfInsertBlockResolutionDirectory,
    definitions: &DxfBlockDefinitionDirectory,
    cancellation: &DxfCancellationToken,
) -> Result<Vec<DxfBlockExpansionEdge>, DxfError> {
    let mut edges = Vec::new();
    for owner in definitions.definitions().iter().copied() {
        ensure_not_cancelled(cancellation)?;
        let members = definitions
            .members_for_block_raw_ordinal(owner.block_record().ordinal())
            .ok_or_else(invalid_internal_data)?;
        for member in members {
            ensure_not_cancelled(cancellation)?;
            let Some(insert) = resolutions.entry_for_insert_raw_ordinal(member.ordinal()) else {
                continue;
            };
            if insert.state() != DxfInsertBlockResolutionState::Unique {
                continue;
            }
            let [target] = resolutions
                .targets_for_insert_raw_ordinal(member.ordinal())
                .ok_or_else(invalid_internal_data)?
            else {
                return Err(invalid_internal_data());
            };
            edges.try_reserve(1).map_err(|_| out_of_memory())?;
            edges.push(DxfBlockExpansionEdge {
                owner,
                insert,
                target: *target,
            });
        }
    }
    Ok(edges)
}

fn eligibility_state(
    resolutions: &DxfInsertBlockResolutionDirectory,
    definitions: &DxfBlockDefinitionDirectory,
    edges: &[DxfBlockExpansionEdge],
    resolution: DxfInsertBlockResolutionEntry,
    cancellation: &DxfCancellationToken,
) -> Result<DxfInsertTargetEligibilityState, DxfError> {
    if resolution.state() != DxfInsertBlockResolutionState::Unique {
        return Ok(DxfInsertTargetEligibilityState::NotUniquelyResolved);
    }
    let [target] = resolutions
        .targets_for_insert_raw_ordinal(resolution.insert().record().ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    let definition = target.record().definition();
    if definition.state() != DxfBlockDefinitionState::Closed {
        return Ok(DxfInsertTargetEligibilityState::TargetDefinitionNotClosed {
            state: definition.state(),
        });
    }
    if expansion_is_recursive(
        definition.block_record().ordinal(),
        definitions,
        edges,
        cancellation,
    )? {
        Ok(DxfInsertTargetEligibilityState::RecursiveExpansion)
    } else {
        Ok(DxfInsertTargetEligibilityState::Eligible)
    }
}

#[derive(Clone, Copy)]
struct TraversalFrame {
    definition_index: usize,
    next_edge: usize,
    edge_end: usize,
}

fn expansion_is_recursive(
    start_raw_ordinal: u64,
    definitions: &DxfBlockDefinitionDirectory,
    edges: &[DxfBlockExpansionEdge],
    cancellation: &DxfCancellationToken,
) -> Result<bool, DxfError> {
    let start_index = definition_index(definitions, start_raw_ordinal)?;
    let mut colors = Vec::new();
    colors
        .try_reserve(definitions.definitions().len())
        .map_err(|_| out_of_memory())?;
    colors.resize(definitions.definitions().len(), 0_u8);
    let mut stack = Vec::new();
    stack
        .try_reserve(definitions.definitions().len())
        .map_err(|_| out_of_memory())?;
    colors[start_index] = 1;
    stack.push(frame_for(start_index, definitions, edges)?);

    while let Some(frame) = stack.last_mut() {
        ensure_not_cancelled(cancellation)?;
        if frame.next_edge >= frame.edge_end {
            colors[frame.definition_index] = 2;
            stack.pop();
            continue;
        }
        let edge = *edges
            .get(frame.next_edge)
            .ok_or_else(invalid_internal_data)?;
        frame.next_edge = frame
            .next_edge
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        let target = edge.target().record().definition();
        if target.state() != DxfBlockDefinitionState::Closed {
            continue;
        }
        let target_index = definition_index(definitions, target.block_record().ordinal())?;
        match colors
            .get(target_index)
            .copied()
            .ok_or_else(invalid_internal_data)?
        {
            1 => return Ok(true),
            2 => {}
            0 => {
                colors[target_index] = 1;
                stack.push(frame_for(target_index, definitions, edges)?);
            }
            _ => return Err(invalid_internal_data()),
        }
    }
    Ok(false)
}

fn frame_for(
    definition_index: usize,
    definitions: &DxfBlockDefinitionDirectory,
    edges: &[DxfBlockExpansionEdge],
) -> Result<TraversalFrame, DxfError> {
    let raw = definitions
        .definitions()
        .get(definition_index)
        .copied()
        .ok_or_else(invalid_internal_data)?
        .block_record()
        .ordinal();
    let start = edges.partition_point(|edge| edge.owner().block_record().ordinal() < raw);
    let end = edges.partition_point(|edge| edge.owner().block_record().ordinal() <= raw);
    Ok(TraversalFrame {
        definition_index,
        next_edge: start,
        edge_end: end,
    })
}

fn definition_index(
    definitions: &DxfBlockDefinitionDirectory,
    raw_ordinal: u64,
) -> Result<usize, DxfError> {
    definitions
        .definitions()
        .binary_search_by_key(&raw_ordinal, |entry| entry.block_record().ordinal())
        .map_err(|_| invalid_internal_data())
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

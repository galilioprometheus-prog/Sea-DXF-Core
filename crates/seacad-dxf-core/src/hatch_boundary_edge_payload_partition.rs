//! Grammar-aware edge-data and path-trailer partitioning for HATCH boundary paths.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfFillMeshField,
    DxfFillMeshRange, DxfHatchBoundaryEdgeTypeDirectory, DxfHatchBoundaryEdgeTypeEntry,
    DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    read_support::{
        compact_len, compact_u64, ensure_not_cancelled, ensure_source, invalid_internal_data,
        out_of_memory,
    },
};

/// Why the final edge of one path cannot be separated from its outer trailer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchBoundaryEdgePayloadPartitionIssue {
    SourceBoundaryCountAbsent,
    SourceBoundaryReferenceBeforeCount { group: DxfRawGroup },
    UnexpectedTrailerField { group: DxfRawGroup },
}

/// Exact edge-data range and, for a final edge, its outer path trailer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgePayloadPartition {
    edge_data_range: DxfFillMeshRange,
    path_trailer_range: Option<DxfFillMeshRange>,
    source_boundary_count: Option<DxfFillMeshField>,
    final_edge: bool,
}

impl DxfHatchBoundaryEdgePayloadPartition {
    #[must_use]
    pub const fn edge_data_range(self) -> DxfFillMeshRange {
        self.edge_data_range
    }

    #[must_use]
    pub const fn path_trailer_range(self) -> Option<DxfFillMeshRange> {
        self.path_trailer_range
    }

    #[must_use]
    pub const fn source_boundary_count(self) -> Option<DxfFillMeshField> {
        self.source_boundary_count
    }

    #[must_use]
    pub const fn is_final_edge(self) -> bool {
        self.final_edge
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchBoundaryEdgePayloadPartitionEntry {
    ordinal: u32,
    edge_type: DxfHatchBoundaryEdgeTypeEntry,
    partition:
        Result<DxfHatchBoundaryEdgePayloadPartition, DxfHatchBoundaryEdgePayloadPartitionIssue>,
}

impl DxfHatchBoundaryEdgePayloadPartitionEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn edge_type_entry(self) -> DxfHatchBoundaryEdgeTypeEntry {
        self.edge_type
    }

    pub const fn partition(
        self,
    ) -> Result<DxfHatchBoundaryEdgePayloadPartition, DxfHatchBoundaryEdgePayloadPartitionIssue>
    {
        self.partition
    }
}

/// One grammar-aware payload partition per grouped HATCH boundary edge.
#[derive(Debug)]
pub struct DxfHatchBoundaryEdgePayloadPartitionDirectory {
    source_id: DxfSourceId,
    edge_types: DxfHatchBoundaryEdgeTypeDirectory,
    entries: Box<[DxfHatchBoundaryEdgePayloadPartitionEntry]>,
}

impl DxfHatchBoundaryEdgePayloadPartitionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let edge_types = document.hatch_boundary_edge_type_directory(cancellation)?;
        ensure_source(document.source_id(), edge_types.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(edge_types.entries().len())
            .map_err(|_| out_of_memory())?;
        for (index, edge_type) in edge_types.entries().iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            let edge = edge_type.edge();
            let final_edge = edge_types
                .entries()
                .get(index.saturating_add(1))
                .is_none_or(|next| next.edge().path_ordinal() != edge.path_ordinal());
            let partition = if final_edge {
                partition_final_edge(&edge_types, edge_type, cancellation)?
            } else {
                Ok(DxfHatchBoundaryEdgePayloadPartition {
                    edge_data_range: edge.payload_range(),
                    path_trailer_range: None,
                    source_boundary_count: None,
                    final_edge: false,
                })
            };
            entries.push(DxfHatchBoundaryEdgePayloadPartitionEntry {
                ordinal: compact_len(entries.len())?,
                edge_type,
                partition,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            edge_types,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn edge_type_directory(&self) -> &DxfHatchBoundaryEdgeTypeDirectory {
        &self.edge_types
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchBoundaryEdgePayloadPartitionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchBoundaryEdgePayloadPartitionEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_edge(
        &self,
        edge_ordinal: u64,
    ) -> Option<DxfHatchBoundaryEdgePayloadPartitionEntry> {
        let index = self
            .entries
            .binary_search_by_key(&edge_ordinal, |entry| {
                entry.edge_type_entry().edge().ordinal()
            })
            .ok()?;
        self.entries.get(index).copied()
    }

    #[must_use]
    pub fn entries_for_path(
        &self,
        path_ordinal: u64,
    ) -> Option<&[DxfHatchBoundaryEdgePayloadPartitionEntry]> {
        self.edge_types.entries_for_path(path_ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.edge_type_entry().edge().path_ordinal() < path_ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.edge_type_entry().edge().path_ordinal() <= path_ordinal);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn edge_data_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let entry = self.entry_for_edge(edge_ordinal)?;
        let partition = entry.partition().ok()?;
        self.fields_for_partition(entry, partition.edge_data_range())
    }

    #[must_use]
    pub fn path_trailer_fields_for_edge(&self, edge_ordinal: u64) -> Option<&[DxfFillMeshField]> {
        let entry = self.entry_for_edge(edge_ordinal)?;
        let partition = entry.partition().ok()?;
        self.fields_for_partition(entry, partition.path_trailer_range()?)
    }

    fn fields_for_partition(
        &self,
        entry: DxfHatchBoundaryEdgePayloadPartitionEntry,
        range: DxfFillMeshRange,
    ) -> Option<&[DxfFillMeshField]> {
        let edge = entry.edge_type_entry().edge();
        let fields = self.edge_types.payload_fields_for_entry(edge.ordinal())?;
        slice_subrange(fields, edge.payload_range(), range)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_boundary_edge_payload_partition_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgePayloadPartitionDirectory, DxfError> {
        DxfHatchBoundaryEdgePayloadPartitionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_boundary_edge_payload_partition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgePayloadPartitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_edge_payload_partition_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_boundary_edge_payload_partition_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchBoundaryEdgePayloadPartitionDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_boundary_edge_payload_partition_directory(cancellation)
    }
}

fn partition_final_edge(
    edge_types: &DxfHatchBoundaryEdgeTypeDirectory,
    edge_type: DxfHatchBoundaryEdgeTypeEntry,
    cancellation: &DxfCancellationToken,
) -> Result<
    Result<DxfHatchBoundaryEdgePayloadPartition, DxfHatchBoundaryEdgePayloadPartitionIssue>,
    DxfError,
> {
    let edge = edge_type.edge();
    let fields = edge_types
        .payload_fields_for_entry(edge.ordinal())
        .ok_or_else(invalid_internal_data)?;
    let Some(count_index) = fields
        .iter()
        .rposition(|field| field.group().group_code().value() == 97)
    else {
        return Ok(Err(
            DxfHatchBoundaryEdgePayloadPartitionIssue::SourceBoundaryCountAbsent,
        ));
    };
    for field in fields.iter().take(count_index) {
        ensure_not_cancelled(cancellation)?;
        if field.group().group_code().value() == 330 {
            return Ok(Err(
                DxfHatchBoundaryEdgePayloadPartitionIssue::SourceBoundaryReferenceBeforeCount {
                    group: field.group(),
                },
            ));
        }
    }
    for field in fields.iter().skip(count_index.saturating_add(1)) {
        ensure_not_cancelled(cancellation)?;
        if field.group().group_code().value() != 330 {
            return Ok(Err(
                DxfHatchBoundaryEdgePayloadPartitionIssue::UnexpectedTrailerField {
                    group: field.group(),
                },
            ));
        }
    }
    let source_boundary_count = fields
        .get(count_index)
        .copied()
        .ok_or_else(invalid_internal_data)?;
    let start = compact_u64(edge.payload_range().start())?;
    let end = compact_u64(edge.payload_range().end())?;
    let trailer_start = start
        .checked_add(compact_len(count_index)?)
        .ok_or_else(invalid_internal_data)?;
    Ok(Ok(DxfHatchBoundaryEdgePayloadPartition {
        edge_data_range: DxfFillMeshRange::new(start, trailer_start)?,
        path_trailer_range: Some(DxfFillMeshRange::new(trailer_start, end)?),
        source_boundary_count: Some(source_boundary_count),
        final_edge: true,
    }))
}

pub(crate) fn slice_subrange<T>(
    values: &[T],
    outer: DxfFillMeshRange,
    inner: DxfFillMeshRange,
) -> Option<&[T]> {
    if inner.start() < outer.start() || inner.end() > outer.end() {
        return None;
    }
    let start = usize::try_from(inner.start().checked_sub(outer.start())?).ok()?;
    let end = usize::try_from(inner.end().checked_sub(outer.start())?).ok()?;
    values.get(start..end)
}

//! Shared exact destination-document named-symbol lookup.

use std::io;

use crate::{
    ByteSpan, DxfCancellationToken, DxfError, DxfIoOperation, DxfNamedSymbolTableDirectory,
    DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind, DxfRawDocumentView, DxfSourceId,
    source_span::{sha256_span, spans_equal_across_documents},
};

type NameDigest = [u8; 32];

#[derive(Clone, Copy)]
struct IndexedName {
    digest: NameDigest,
    target: DxfNamedSymbolTableEntry,
}

pub(crate) struct NamedSymbolDestinationIndex {
    destination_id: DxfSourceId,
    kind: DxfNamedSymbolTableKind,
    entries: Box<[IndexedName]>,
}

impl NamedSymbolDestinationIndex {
    pub(crate) fn from_directory(
        destination: DxfRawDocumentView<'_>,
        symbols: &DxfNamedSymbolTableDirectory,
        kind: DxfNamedSymbolTableKind,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(destination.source_id(), symbols.source_id())?;
        let count = symbols
            .entries()
            .iter()
            .filter(|entry| entry.kind() == kind)
            .count();
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| out_of_memory())?;
        for target in symbols
            .entries()
            .iter()
            .copied()
            .filter(|entry| entry.kind() == kind)
        {
            ensure_not_cancelled(cancellation)?;
            entries.push(IndexedName {
                digest: sha256_span(destination, target.name().value_span(), cancellation)?,
                target,
            });
        }
        entries.sort_unstable_by_key(|entry| (entry.digest, entry.target.record().ordinal()));
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            destination_id: destination.source_id(),
            kind,
            entries: entries.into_boxed_slice(),
        })
    }

    pub(crate) fn exact_matches(
        &self,
        source: DxfRawDocumentView<'_>,
        source_name: ByteSpan,
        destination: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<(Option<DxfNamedSymbolTableEntry>, u32), DxfError> {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.destination_id, destination.source_id())?;
        let digest = sha256_span(source, source_name, cancellation)?;
        let start = self.entries.partition_point(|entry| entry.digest < digest);
        let end = self.entries.partition_point(|entry| entry.digest <= digest);
        let mut first = None;
        let mut count = 0_u32;
        for candidate in self
            .entries
            .get(start..end)
            .ok_or_else(invalid_internal_data)?
        {
            ensure_not_cancelled(cancellation)?;
            if candidate.target.kind() != self.kind {
                return Err(invalid_internal_data());
            }
            if spans_equal_across_documents(
                source,
                source_name,
                destination,
                candidate.target.name().value_span(),
                cancellation,
            )? {
                count = count.checked_add(1).ok_or_else(invalid_internal_data)?;
                first.get_or_insert(candidate.target);
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok((first, count))
    }
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

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

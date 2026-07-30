//! Ordered MTEXT group-3/group-1 chunk accounting.

use std::io;

use super::{DxfMTextChunkEntry, DxfMTextChunkKind, DxfMTextChunkRange, DxfMTextChunkSequence};
use crate::{
    DxfError, DxfIoOperation, DxfTextSymbolCardDirectory, DxfTextSymbolRecordEntry,
    DxfTextSymbolValueRole,
};

pub(super) fn append_sequence(
    cards: &DxfTextSymbolCardDirectory,
    record: DxfTextSymbolRecordEntry,
    sequences: &mut Vec<DxfMTextChunkSequence>,
    chunks: &mut Vec<DxfMTextChunkEntry>,
) -> Result<(), DxfError> {
    let start = compact_len(chunks.len())?;
    let values = cards
        .evidence_directory()
        .values_for_raw_record(record.record().ordinal())
        .ok_or_else(invalid_internal_data)?;
    let mut terminal_count = 0_u32;
    let mut additional_after_terminal_count = 0_u32;
    for value in values.iter().copied() {
        let kind = match value.role() {
            DxfTextSymbolValueRole::AdditionalContent => DxfMTextChunkKind::Additional,
            DxfTextSymbolValueRole::Content => DxfMTextChunkKind::Terminal,
            _ => continue,
        };
        if kind == DxfMTextChunkKind::Terminal {
            terminal_count = terminal_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
        } else if terminal_count != 0 {
            additional_after_terminal_count = additional_after_terminal_count
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
        }
        chunks.try_reserve(1).map_err(|_| out_of_memory())?;
        chunks.push(DxfMTextChunkEntry { value, kind });
    }
    let end = compact_len(chunks.len())?;
    sequences.try_reserve(1).map_err(|_| out_of_memory())?;
    sequences.push(DxfMTextChunkSequence {
        source_id: cards.source_id(),
        record,
        chunk_range: DxfMTextChunkRange::new(start, end)?,
        terminal_count,
        additional_after_terminal_count,
    });
    Ok(())
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
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

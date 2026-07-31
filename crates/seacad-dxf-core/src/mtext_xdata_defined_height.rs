//! Exact R2007-era MTEXT defined-height XDATA framing.

use std::io;

use crate::{
    DxfCancellationToken, DxfError, DxfIoOperation, DxfMTextXDataColumnEntry,
    DxfMTextXDataColumnRole, DxfMTextXDataColumnValue, DxfRawDocumentView, DxfRawGroup,
    DxfRawRecord, DxfTextSymbolNumericIssue, DxfTextSymbolValueData, raw_double::decode_raw_double,
    raw_integer::decode_raw_i16,
};

const END: &[u8] = b"ACAD_MTEXT_DEFINED_HEIGHT_END";
const DEFINED_HEIGHT_SELECTOR: i16 = 46;

pub(super) fn append_defined_height(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    occurrence: u64,
    cancellation: &DxfCancellationToken,
    entries: &mut [DxfMTextXDataColumnEntry],
    values: &mut Vec<DxfMTextXDataColumnValue>,
) -> Result<u64, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let record_end = record.group_range().end();
    if occurrence >= record_end {
        return Ok(record_end);
    }
    let Some(field_id_group) = document.group(occurrence) else {
        return Ok(record_end);
    };
    if field_id_group.group_code().value() != 1070
        || decode_raw_i16(document, field_id_group, cancellation)? != Ok(DEFINED_HEIGHT_SELECTOR)
    {
        return Ok(occurrence.saturating_add(1));
    }

    let value_occurrence = occurrence.saturating_add(1);
    if value_occurrence >= record_end {
        return Ok(record_end);
    }
    let Some(value_group) = document.group(value_occurrence) else {
        return Ok(record_end);
    };
    if value_group.group_code().value() != 1040 {
        return Ok(value_occurrence.saturating_add(1));
    }

    let end_occurrence = value_occurrence.saturating_add(1);
    if end_occurrence >= record_end {
        return Ok(record_end);
    }
    let Some(end_group) = document.group(end_occurrence) else {
        return Ok(record_end);
    };
    if !exact_text(document, end_group, 1000, END)? {
        return Ok(end_occurrence.saturating_add(1));
    }

    let compact_value_end = compact_len(values.len())?;
    let Some(entry) = entries.last_mut() else {
        return Ok(end_occurrence.saturating_add(1));
    };
    if entry.record() != record || entry.value_end != compact_value_end {
        return Ok(end_occurrence.saturating_add(1));
    }

    ensure_not_cancelled(cancellation)?;
    let issue = DxfTextSymbolNumericIssue::InvalidAsciiNumber;
    let data = decode_raw_double(document, value_group, cancellation)?.map_err(issue);
    values.try_reserve(1).map_err(|_| out_of_memory())?;
    values.push(DxfMTextXDataColumnValue {
        field_id_group,
        value_group,
        role: DxfMTextXDataColumnRole::DefinedHeight,
        data: DxfTextSymbolValueData::Double(data),
    });
    entry.value_end = compact_len(values.len())?;
    Ok(end_occurrence.saturating_add(1))
}

fn exact_text(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    code: i16,
    text: &[u8],
) -> Result<bool, DxfError> {
    Ok(group.group_code().value() == code
        && document.raw_span_equals_exact(group.value_payload_span(), text)?)
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

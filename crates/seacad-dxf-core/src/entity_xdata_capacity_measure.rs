//! Internal logical-value sizing for the public entity XDATA capacity directory.

use std::io;

use crate::{
    DxfCancellationToken, DxfEntityRef, DxfEntityXDataAppIdResolutionDirectory,
    DxfEntityXDataAppIdResolutionState, DxfEntityXDataCapacityIssue,
    DxfEntityXDataCapacityIssueKind, DxfEntityXDataCapacityTextIssue,
    DxfEntityXDataCapacityTextIssue::*, DxfEntityXDataControl, DxfEntityXDataDoubleRole,
    DxfEntityXDataLayerResolutionDirectory, DxfEntityXDataLayerResolutionState,
    DxfEntityXDataOccurrenceKind, DxfEntityXDataPointTuple, DxfEntityXDataPointTupleDirectory,
    DxfEntityXDataPointTupleState, DxfEntityXDataStructureDirectory, DxfEntityXDataStructureState,
    DxfEntityXDataTextKind, DxfEntityXDataTypedDirectory, DxfEntityXDataTypedEntry,
    DxfEntityXDataValue, DxfError, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
    DxfTextDecodeStatus, DxfTextEscapeDecodeStatus,
    decode_dxf_text_escapes_to_utf8_without_replacement,
};

const RAW_STRING_BYTES: usize = 255;
const DECODED_STRING_BYTES: usize = RAW_STRING_BYTES * 4;

#[allow(clippy::too_many_arguments)]
pub(crate) fn measure_entity(
    document: DxfRawDocumentView<'_>,
    typed: &DxfEntityXDataTypedDirectory,
    appids: &DxfEntityXDataAppIdResolutionDirectory,
    structure: &DxfEntityXDataStructureDirectory,
    tuples: &DxfEntityXDataPointTupleDirectory,
    layers: &DxfEntityXDataLayerResolutionDirectory,
    entity: DxfEntityRef,
    cancellation: &DxfCancellationToken,
    issues: &mut Vec<DxfEntityXDataCapacityIssue>,
) -> Result<u64, DxfError> {
    let mut accounted = 0_u64;
    for source in typed_entries_for_entity(typed, entity)? {
        if source.occurrence().kind() == DxfEntityXDataOccurrenceKind::Orphan {
            push_issue(
                issues,
                DxfEntityXDataCapacityIssueKind::OrphanValue {
                    source_entry_ordinal: compact_u64(source.ordinal())?,
                },
            )?;
        }
    }
    for application in typed
        .xdata_directory()
        .applications_for_entity(entity)?
        .iter()
        .copied()
    {
        ensure_not_cancelled(cancellation)?;
        let target_count = match appids.entry_for_application(application)?.state() {
            DxfEntityXDataAppIdResolutionState::Unique { .. } => None,
            DxfEntityXDataAppIdResolutionState::Missing => Some(0),
            DxfEntityXDataAppIdResolutionState::Ambiguous { target_count } => Some(target_count),
        };
        if let Some(target_count) = target_count {
            push_issue(
                issues,
                DxfEntityXDataCapacityIssueKind::ApplicationResolution {
                    application_ordinal: compact_u64(application.ordinal())?,
                    target_count,
                },
            )?;
        }
        let structure_issue_count = match structure.entry_for_application(application)?.state() {
            DxfEntityXDataStructureState::Valid => None,
            DxfEntityXDataStructureState::Invalid { issue_count } => Some(issue_count),
        };
        if let Some(issue_count) = structure_issue_count {
            push_issue(
                issues,
                DxfEntityXDataCapacityIssueKind::InvalidApplicationStructure {
                    application_ordinal: compact_u64(application.ordinal())?,
                    issue_count,
                },
            )?;
        }
        if target_count.is_some() || structure_issue_count.is_some() {
            continue;
        }
        let values = typed.entries_for_application(application)?;
        if values.len() <= 1 {
            continue;
        }
        add_size(&mut accounted, 3)?;
        for source in values.iter().copied().skip(1) {
            ensure_not_cancelled(cancellation)?;
            measure_value(
                document,
                source,
                tuples,
                layers,
                cancellation,
                &mut accounted,
                issues,
            )?;
        }
    }
    Ok(accounted)
}

fn measure_value(
    document: DxfRawDocumentView<'_>,
    source: DxfEntityXDataTypedEntry,
    tuples: &DxfEntityXDataPointTupleDirectory,
    layers: &DxfEntityXDataLayerResolutionDirectory,
    cancellation: &DxfCancellationToken,
    accounted: &mut u64,
    issues: &mut Vec<DxfEntityXDataCapacityIssue>,
) -> Result<(), DxfError> {
    if is_point_code(source.occurrence().group().group_code().value()) {
        return measure_point(source, tuples, accounted, issues);
    }
    match source.value() {
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::String,
            ..
        } => match string_size(document, source, cancellation)? {
            Ok(bytes) => add_size(accounted, bytes)?,
            Err(issue) => push_issue(
                issues,
                DxfEntityXDataCapacityIssueKind::Text {
                    source_entry_ordinal: compact_u64(source.ordinal())?,
                    issue,
                },
            )?,
        },
        DxfEntityXDataValue::Control {
            control: DxfEntityXDataControl::OpenList | DxfEntityXDataControl::CloseList,
            ..
        } => add_size(accounted, 2)?,
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::LayerName,
            ..
        } => {
            let resolution = layers
                .entry_for_source(source)
                .ok_or_else(invalid_internal_data)?;
            match resolution.state() {
                DxfEntityXDataLayerResolutionState::Unique { .. } => add_size(accounted, 3)?,
                DxfEntityXDataLayerResolutionState::Missing => push_issue(
                    issues,
                    DxfEntityXDataCapacityIssueKind::LayerResolution {
                        source_entry_ordinal: compact_u64(source.ordinal())?,
                        target_count: 0,
                    },
                )?,
                DxfEntityXDataLayerResolutionState::Ambiguous { target_count } => push_issue(
                    issues,
                    DxfEntityXDataCapacityIssueKind::LayerResolution {
                        source_entry_ordinal: compact_u64(source.ordinal())?,
                        target_count,
                    },
                )?,
            }
        }
        DxfEntityXDataValue::BinaryChunk { decoded_bytes, .. } => {
            add_size(accounted, u64::from(decoded_bytes) + 2)?;
        }
        DxfEntityXDataValue::Handle { .. } => add_size(accounted, 9)?,
        DxfEntityXDataValue::Double {
            role:
                DxfEntityXDataDoubleRole::Real
                | DxfEntityXDataDoubleRole::Distance
                | DxfEntityXDataDoubleRole::ScaleFactor,
            ..
        } => add_size(accounted, 9)?,
        DxfEntityXDataValue::Int16 { .. } => add_size(accounted, 3)?,
        DxfEntityXDataValue::Int32 { .. } => add_size(accounted, 5)?,
        DxfEntityXDataValue::Invalid { .. } => push_issue(
            issues,
            DxfEntityXDataCapacityIssueKind::InvalidValue {
                source_entry_ordinal: compact_u64(source.ordinal())?,
            },
        )?,
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::ApplicationName,
            ..
        }
        | DxfEntityXDataValue::Double { .. } => return Err(invalid_internal_data()),
    }
    Ok(())
}

fn measure_point(
    source: DxfEntityXDataTypedEntry,
    tuples: &DxfEntityXDataPointTupleDirectory,
    accounted: &mut u64,
    issues: &mut Vec<DxfEntityXDataCapacityIssue>,
) -> Result<(), DxfError> {
    let tuple = tuples
        .tuple_for_entry(source)
        .ok_or_else(invalid_internal_data)?;
    let first = first_member(tuple).ok_or_else(invalid_internal_data)?;
    if source.ordinal() == first {
        match tuple.state() {
            DxfEntityXDataPointTupleState::Partial(_) => push_issue(
                issues,
                DxfEntityXDataCapacityIssueKind::PartialPointTuple {
                    tuple_ordinal: compact_u64(tuple.ordinal())?,
                },
            )?,
            DxfEntityXDataPointTupleState::Complete if tuple_values_valid(tuples, tuple)? => {
                add_size(accounted, 25)?;
            }
            DxfEntityXDataPointTupleState::Complete => {}
        }
    }
    if matches!(source.value(), DxfEntityXDataValue::Invalid { .. }) {
        push_issue(
            issues,
            DxfEntityXDataCapacityIssueKind::InvalidValue {
                source_entry_ordinal: compact_u64(source.ordinal())?,
            },
        )?;
    }
    Ok(())
}

fn tuple_values_valid(
    tuples: &DxfEntityXDataPointTupleDirectory,
    tuple: DxfEntityXDataPointTuple,
) -> Result<bool, DxfError> {
    for member in [tuple.x(), tuple.y(), tuple.z()] {
        let member = member.ok_or_else(invalid_internal_data)?;
        let source = tuples
            .entry_for_member(tuple, member)
            .ok_or_else(invalid_internal_data)?;
        if !matches!(source.value(), DxfEntityXDataValue::Double { .. }) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn string_size(
    document: DxfRawDocumentView<'_>,
    source: DxfEntityXDataTypedEntry,
    cancellation: &DxfCancellationToken,
) -> Result<Result<u64, DxfEntityXDataCapacityTextIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let span = source.occurrence().group().value_payload_span();
    let len = usize::try_from(span.len()).map_err(|_| invalid_internal_data())?;
    let raw = &mut [0_u8; RAW_STRING_BYTES];
    let raw = raw.get_mut(..len).ok_or_else(invalid_internal_data)?;
    document.read_span(span, raw)?;
    ensure_not_cancelled(cancellation)?;
    let mut decoded_storage = [0_u8; DECODED_STRING_BYTES];
    let decoded_len = if raw.is_ascii() {
        decoded_storage
            .get_mut(..raw.len())
            .ok_or_else(invalid_internal_data)?
            .copy_from_slice(raw);
        raw.len()
    } else {
        let receipt = document.decode_group_value_to_utf8_without_replacement(
            source.occurrence().group().occurrence(),
            &mut decoded_storage,
        )?;
        if receipt.source_id() != document.source_id() || receipt.value_span() != span {
            return Err(invalid_internal_data());
        }
        let Some(result) = receipt.decode_result() else {
            return Ok(Err(EncodingUnavailable));
        };
        match result.status() {
            DxfTextDecodeStatus::Complete => result.written(),
            DxfTextDecodeStatus::OutputFull => return Ok(Err(StorageOutputLimit)),
            DxfTextDecodeStatus::Malformed { .. } => return Ok(Err(StorageMalformed)),
        }
    };
    let decoded = std::str::from_utf8(
        decoded_storage
            .get(..decoded_len)
            .ok_or_else(invalid_internal_data)?,
    )
    .map_err(|_| invalid_internal_data())?;
    let mut semantic_storage = [0_u8; DECODED_STRING_BYTES];
    let semantic =
        decode_dxf_text_escapes_to_utf8_without_replacement(decoded, &mut semantic_storage);
    let semantic_len = match semantic.status() {
        DxfTextEscapeDecodeStatus::Complete => semantic.written(),
        DxfTextEscapeDecodeStatus::OutputFull => return Ok(Err(EscapeOutputLimit)),
        DxfTextEscapeDecodeStatus::Malformed { .. } => return Ok(Err(EscapeMalformed)),
    };
    let semantic = std::str::from_utf8(
        semantic_storage
            .get(..semantic_len)
            .ok_or_else(invalid_internal_data)?,
    )
    .map_err(|_| invalid_internal_data())?;
    let scalar_count =
        u64::try_from(semantic.chars().count()).map_err(|_| invalid_internal_data())?;
    scalar_count
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(3))
        .ok_or_else(invalid_internal_data)
        .map(Ok)
}

fn typed_entries_for_entity(
    typed: &DxfEntityXDataTypedDirectory,
    entity: DxfEntityRef,
) -> Result<&[DxfEntityXDataTypedEntry], DxfError> {
    ensure_source(typed.source_id(), entity.source_id())?;
    let ordinal = entity.record().ordinal();
    let start = typed
        .entries()
        .partition_point(|entry| entry.occurrence().entity().record().ordinal() < ordinal);
    let end = typed
        .entries()
        .partition_point(|entry| entry.occurrence().entity().record().ordinal() <= ordinal);
    typed
        .entries()
        .get(start..end)
        .ok_or_else(invalid_internal_data)
}

const fn is_point_code(code: i16) -> bool {
    matches!(code, 1010..=1013 | 1020..=1023 | 1030..=1033)
}

const fn first_member(tuple: DxfEntityXDataPointTuple) -> Option<u64> {
    if let Some(member) = tuple.x() {
        Some(member.entry_ordinal())
    } else if let Some(member) = tuple.y() {
        Some(member.entry_ordinal())
    } else if let Some(member) = tuple.z() {
        Some(member.entry_ordinal())
    } else {
        None
    }
}

fn add_size(total: &mut u64, bytes: u64) -> Result<(), DxfError> {
    *total = total.checked_add(bytes).ok_or_else(invalid_internal_data)?;
    Ok(())
}

fn push_issue(
    issues: &mut Vec<DxfEntityXDataCapacityIssue>,
    kind: DxfEntityXDataCapacityIssueKind,
) -> Result<(), DxfError> {
    issues.try_reserve(1).map_err(|_| out_of_memory())?;
    issues.push(DxfEntityXDataCapacityIssue::new(kind));
    Ok(())
}

fn compact_u64(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
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

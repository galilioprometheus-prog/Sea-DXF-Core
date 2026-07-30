//! Atomic transaction planning for new object identities and `$HANDSEED`.

use std::io;

use crate::{
    ByteSpan, DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfError, DxfHandle, DxfHandleAllocationOutcome, DxfHandleAllocationPolicyDirectory,
    DxfHandleAllocationPolicyState, DxfHandleAllocationProposal, DxfHandleIdentityState,
    DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView, DxfRawGroup, DxfRawRecord,
    DxfRawRecordSectionKind, DxfResourceProfile, DxfSourceId, DxfTransactionPlan,
};

/// Why one requested record set cannot receive new object identities.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHandleAssignmentTargetState {
    DuplicateTarget {
        record_ordinal: u64,
    },
    MissingRecord {
        record_ordinal: u64,
    },
    IdentityAlreadyPresent {
        record_ordinal: u64,
        state: DxfHandleIdentityState,
    },
    UnsupportedSection {
        record_ordinal: u64,
        section: DxfRawRecordSectionKind,
    },
    TableBoundary {
        record_ordinal: u64,
    },
    TableNameUnavailable {
        record_ordinal: u64,
    },
}

/// Complete allocation evidence paired with one immutable raw-byte plan.
#[derive(Debug)]
pub struct DxfHandleAssignmentPlan {
    allocation: DxfHandleAllocationProposal,
    transaction: DxfTransactionPlan,
}

impl DxfHandleAssignmentPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.allocation.source_id()
    }

    #[must_use]
    pub const fn allocation(&self) -> DxfHandleAllocationProposal {
        self.allocation
    }

    #[must_use]
    pub const fn transaction(&self) -> &DxfTransactionPlan {
        &self.transaction
    }

    #[must_use]
    pub fn into_transaction(self) -> DxfTransactionPlan {
        self.transaction
    }
}

/// Typed result of planning object identities and the successor `$HANDSEED`.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfHandleAssignmentPlanOutcome {
    PolicyUnavailable {
        state: DxfHandleAllocationPolicyState,
    },
    Exhausted {
        handseed: DxfHandle,
        requested_count: u64,
    },
    TargetsUnavailable {
        state: DxfHandleAssignmentTargetState,
    },
    Planned(DxfHandleAssignmentPlan),
}

#[derive(Clone, Copy)]
struct PreparedTarget {
    anchor: DxfRawGroup,
    identity_group_code: i16,
}

impl DxfRawDocumentView<'_> {
    /// Plans insertion of new record identities and one successor `$HANDSEED`.
    pub fn plan_handle_assignments(
        self,
        policy: &DxfHandleAllocationPolicyDirectory,
        record_ordinals: &[u64],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleAssignmentPlanOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        if policy.source_id() != self.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id(),
                observed: policy.source_id(),
            });
        }

        let requested_count =
            u64::try_from(record_ordinals.len()).map_err(|_| invalid_internal_data())?;
        let allocation = match policy.propose_allocation(requested_count, profile)? {
            DxfHandleAllocationOutcome::Unavailable { state } => {
                return Ok(DxfHandleAssignmentPlanOutcome::PolicyUnavailable { state });
            }
            DxfHandleAllocationOutcome::Exhausted {
                handseed,
                requested_count,
            } => {
                return Ok(DxfHandleAssignmentPlanOutcome::Exhausted {
                    handseed,
                    requested_count,
                });
            }
            DxfHandleAllocationOutcome::Proposed(proposal) => proposal,
        };

        if let Some(state) = first_duplicate_target(record_ordinals)? {
            return Ok(DxfHandleAssignmentPlanOutcome::TargetsUnavailable { state });
        }

        let mut prepared = Vec::new();
        prepared
            .try_reserve_exact(record_ordinals.len())
            .map_err(|_| out_of_memory())?;
        for record_ordinal in record_ordinals.iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let Some(entry) = policy.identity_directory().entry(record_ordinal) else {
                return Ok(DxfHandleAssignmentPlanOutcome::TargetsUnavailable {
                    state: DxfHandleAssignmentTargetState::MissingRecord { record_ordinal },
                });
            };
            if entry.state() != DxfHandleIdentityState::Absent {
                return Ok(DxfHandleAssignmentPlanOutcome::TargetsUnavailable {
                    state: DxfHandleAssignmentTargetState::IdentityAlreadyPresent {
                        record_ordinal,
                        state: entry.state(),
                    },
                });
            }
            match prepare_target(self, entry.record())? {
                Ok(target) => prepared.push(target),
                Err(state) => {
                    return Ok(DxfHandleAssignmentPlanOutcome::TargetsUnavailable { state });
                }
            }
        }

        let mut builder = self.transaction_plan_builder(profile)?;
        for (index, target) in prepared.iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            let index = u64::try_from(index).map_err(|_| invalid_internal_data())?;
            let handle = allocation
                .handle_at(index)
                .ok_or_else(invalid_internal_data)?;
            let insertion = encode_identity_group(self, target, handle)?;
            let insertion_span = ByteSpan::new(
                target.anchor.full_span().end(),
                target.anchor.full_span().end(),
            )
            .ok_or_else(invalid_internal_data)?;
            builder.replace_raw_span(insertion_span, insertion.as_slice(), cancellation)?;
        }

        if requested_count != 0 {
            let handseed_span = policy
                .handseed_occurrence()
                .and_then(|occurrence| occurrence.value_span())
                .ok_or_else(invalid_internal_data)?;
            let mut encoded_handseed = [0_u8; 16];
            let encoded_handseed = encode_handle(allocation.next_handseed(), &mut encoded_handseed);
            builder.replace_raw_span(handseed_span, encoded_handseed, cancellation)?;
        }
        let transaction = builder.finish(cancellation)?;
        Ok(DxfHandleAssignmentPlanOutcome::Planned(
            DxfHandleAssignmentPlan {
                allocation,
                transaction,
            },
        ))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn plan_handle_assignments(
        &self,
        policy: &DxfHandleAllocationPolicyDirectory,
        record_ordinals: &[u64],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleAssignmentPlanOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_handle_assignments(
            policy,
            record_ordinals,
            profile,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn plan_handle_assignments(
        &self,
        policy: &DxfHandleAllocationPolicyDirectory,
        record_ordinals: &[u64],
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHandleAssignmentPlanOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_handle_assignments(
            policy,
            record_ordinals,
            profile,
            cancellation,
        )
    }
}

fn first_duplicate_target(
    record_ordinals: &[u64],
) -> Result<Option<DxfHandleAssignmentTargetState>, DxfError> {
    let mut sorted = Vec::new();
    sorted
        .try_reserve_exact(record_ordinals.len())
        .map_err(|_| out_of_memory())?;
    sorted.extend_from_slice(record_ordinals);
    sorted.sort_unstable();
    for pair in sorted.windows(2) {
        if pair[0] == pair[1] {
            return Ok(Some(DxfHandleAssignmentTargetState::DuplicateTarget {
                record_ordinal: pair[0],
            }));
        }
    }
    Ok(None)
}

fn prepare_target(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
) -> Result<Result<PreparedTarget, DxfHandleAssignmentTargetState>, DxfError> {
    let marker = document
        .group(record.marker_occurrence())
        .ok_or_else(invalid_internal_data)?;
    if marker.group_code().value() != 0 {
        return Err(invalid_internal_data());
    }
    match record.section_kind() {
        DxfRawRecordSectionKind::Classes => {
            Ok(Err(DxfHandleAssignmentTargetState::UnsupportedSection {
                record_ordinal: record.ordinal(),
                section: record.section_kind(),
            }))
        }
        DxfRawRecordSectionKind::Tables => prepare_table_target(document, record, marker),
        DxfRawRecordSectionKind::Blocks
        | DxfRawRecordSectionKind::Entities
        | DxfRawRecordSectionKind::Objects => Ok(Ok(PreparedTarget {
            anchor: marker,
            identity_group_code: 5,
        })),
    }
}

fn prepare_table_target(
    document: DxfRawDocumentView<'_>,
    record: DxfRawRecord,
    marker: DxfRawGroup,
) -> Result<Result<PreparedTarget, DxfHandleAssignmentTargetState>, DxfError> {
    if document.raw_span_equals_exact(marker.value_payload_span(), b"ENDTAB")? {
        return Ok(Err(DxfHandleAssignmentTargetState::TableBoundary {
            record_ordinal: record.ordinal(),
        }));
    }
    let is_dimstyle = document.raw_span_equals_exact(marker.value_payload_span(), b"DIMSTYLE")?;
    for occurrence in record.group_range().start().saturating_add(1)..record.group_range().end() {
        let group = document
            .group(occurrence)
            .ok_or_else(invalid_internal_data)?;
        if group.group_code().value() == 2 {
            return Ok(Ok(PreparedTarget {
                anchor: group,
                identity_group_code: if is_dimstyle { 105 } else { 5 },
            }));
        }
    }
    Ok(Err(DxfHandleAssignmentTargetState::TableNameUnavailable {
        record_ordinal: record.ordinal(),
    }))
}

struct EncodedIdentityGroup {
    bytes: [u8; 24],
    len: usize,
}

impl EncodedIdentityGroup {
    fn as_slice(&self) -> &[u8] {
        self.bytes.get(..self.len).map_or(&[], |bytes| bytes)
    }
}

fn encode_identity_group(
    document: DxfRawDocumentView<'_>,
    target: PreparedTarget,
    handle: DxfHandle,
) -> Result<EncodedIdentityGroup, DxfError> {
    let mut encoded = EncodedIdentityGroup {
        bytes: [0_u8; 24],
        len: 0,
    };
    let mut handle_bytes = [0_u8; 16];
    let handle_bytes = encode_handle(handle, &mut handle_bytes);
    match document.format() {
        DxfRawDocumentFormat::Ascii => {
            let ending = ascii_line_ending(document, target.anchor)?;
            let group_code = if target.identity_group_code == 105 {
                b"105".as_slice()
            } else {
                b"5".as_slice()
            };
            append_encoded(&mut encoded, group_code)?;
            append_encoded(&mut encoded, ending)?;
            append_encoded(&mut encoded, handle_bytes)?;
            append_encoded(&mut encoded, ending)?;
        }
        DxfRawDocumentFormat::Binary => {
            let version = match document.acad_version_report().state() {
                DxfAcadVersionState::Supported(version) => version,
                _ => return Err(invalid_internal_data()),
            };
            match version.binary_group_code_encoding() {
                crate::DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape => {
                    let code = u8::try_from(target.identity_group_code)
                        .map_err(|_| invalid_internal_data())?;
                    append_encoded(&mut encoded, &[code])?;
                }
                crate::DxfBinaryGroupCodeEncoding::TwoByteLittleEndian => {
                    append_encoded(&mut encoded, &target.identity_group_code.to_le_bytes())?;
                }
            }
            append_encoded(&mut encoded, handle_bytes)?;
            append_encoded(&mut encoded, &[0])?;
        }
    }
    Ok(encoded)
}

fn ascii_line_ending(
    document: DxfRawDocumentView<'_>,
    anchor: DxfRawGroup,
) -> Result<&'static [u8], DxfError> {
    let full_span = anchor.full_span();
    if full_span.is_empty() {
        return Err(invalid_internal_data());
    }
    let tail_len = full_span.len().min(2);
    let tail_span = ByteSpan::new(full_span.end() - tail_len, full_span.end())
        .ok_or_else(invalid_internal_data)?;
    let mut tail = [0_u8; 2];
    let start = usize::try_from(2 - tail_len).map_err(|_| invalid_internal_data())?;
    document.read_span(
        tail_span,
        tail.get_mut(start..).ok_or_else(invalid_internal_data)?,
    )?;
    let tail = tail.get(start..).ok_or_else(invalid_internal_data)?;
    if tail.ends_with(b"\r\n") {
        Ok(b"\r\n")
    } else if tail.ends_with(b"\n") {
        Ok(b"\n")
    } else if tail.ends_with(b"\r") {
        Ok(b"\r")
    } else {
        Err(invalid_internal_data())
    }
}

fn append_encoded(encoded: &mut EncodedIdentityGroup, fragment: &[u8]) -> Result<(), DxfError> {
    let end = encoded
        .len
        .checked_add(fragment.len())
        .ok_or_else(invalid_internal_data)?;
    let destination = encoded
        .bytes
        .get_mut(encoded.len..end)
        .ok_or_else(invalid_internal_data)?;
    destination.copy_from_slice(fragment);
    encoded.len = end;
    Ok(())
}

fn encode_handle(handle: DxfHandle, destination: &mut [u8; 16]) -> &[u8] {
    let mut value = handle.value();
    let mut start = destination.len();
    loop {
        start -= 1;
        let digit = (value & 0xF) as u8;
        destination[start] = if digit < 10 {
            b'0' + digit
        } else {
            b'A' + (digit - 10)
        };
        value >>= 4;
        if value == 0 {
            break;
        }
    }
    &destination[start..]
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

//! Immutable, source-bound raw-byte transaction plans with inverse capture.

use std::{fmt, io};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView, DxfResource, DxfResourceLimits,
    DxfResourceProfile, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTransactionByteRange {
    start: u64,
    end: u64,
}

impl DxfTransactionByteRange {
    fn from_start_and_len(start: u64, len: u64) -> Result<Self, DxfError> {
        let end = start.checked_add(len).ok_or(DxfError::OffsetOverflow {
            offset: start,
            requested: len,
        })?;
        Ok(Self { start, end })
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        self.end - self.start
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfTransactionPatch {
    ordinal: u32,
    source_span: ByteSpan,
    replacement_range: DxfTransactionByteRange,
    inverse_range: DxfTransactionByteRange,
}

impl DxfTransactionPatch {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn source_span(self) -> ByteSpan {
        self.source_span
    }

    #[must_use]
    pub const fn replacement_range(self) -> DxfTransactionByteRange {
        self.replacement_range
    }

    #[must_use]
    pub const fn inverse_range(self) -> DxfTransactionByteRange {
        self.inverse_range
    }
}

/// Immutable source-order patch plan. Payload bytes are redacted from `Debug`.
pub struct DxfTransactionPlan {
    source_id: DxfSourceId,
    source_len: u64,
    format: DxfRawDocumentFormat,
    projected_len: u64,
    patches: Box<[DxfTransactionPatch]>,
    replacement_bytes: Box<[u8]>,
    inverse_bytes: Box<[u8]>,
}

impl DxfTransactionPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_len(&self) -> u64 {
        self.source_len
    }

    #[must_use]
    pub const fn format(&self) -> DxfRawDocumentFormat {
        self.format
    }

    #[must_use]
    pub const fn projected_len(&self) -> u64 {
        self.projected_len
    }

    #[must_use]
    pub fn patches(&self) -> &[DxfTransactionPatch] {
        &self.patches
    }

    #[must_use]
    pub fn replacement_bytes_for_patch_ordinal(&self, patch_ordinal: u64) -> Option<&[u8]> {
        let patch = self.patches.get(usize::try_from(patch_ordinal).ok()?)?;
        bytes_for_range(&self.replacement_bytes, patch.replacement_range())
    }

    #[must_use]
    pub fn inverse_bytes_for_patch_ordinal(&self, patch_ordinal: u64) -> Option<&[u8]> {
        let patch = self.patches.get(usize::try_from(patch_ordinal).ok()?)?;
        bytes_for_range(&self.inverse_bytes, patch.inverse_range())
    }

    pub fn validate_source_precondition(
        &self,
        document: DxfRawDocumentView<'_>,
    ) -> Result<(), DxfError> {
        if document.source_id() != self.source_id {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: document.source_id(),
            });
        }
        if document.source_len() != self.source_len || document.format() != self.format {
            return Err(invalid_internal_data());
        }
        Ok(())
    }
}

impl fmt::Debug for DxfTransactionPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfTransactionPlan")
            .field("source_id", &self.source_id)
            .field("source_len", &self.source_len)
            .field("format", &self.format)
            .field("projected_len", &self.projected_len)
            .field("patch_count", &self.patches.len())
            .field("replacement_byte_count", &self.replacement_bytes.len())
            .field("inverse_byte_count", &self.inverse_bytes.len())
            .finish()
    }
}

#[derive(Clone, Copy)]
struct PendingPatch {
    source_span: ByteSpan,
    replacement_range: DxfTransactionByteRange,
    inverse_range: DxfTransactionByteRange,
}

/// Mutable construction boundary for an immutable transaction plan.
pub struct DxfTransactionPlanBuilder<'a> {
    document: DxfRawDocumentView<'a>,
    limits: DxfResourceLimits,
    projected_len: u64,
    pending: Vec<PendingPatch>,
    replacement_bytes: Vec<u8>,
    inverse_bytes: Vec<u8>,
}

impl<'a> DxfTransactionPlanBuilder<'a> {
    fn new(
        document: DxfRawDocumentView<'a>,
        profile: DxfResourceProfile,
    ) -> Result<Self, DxfError> {
        let limits = profile.limits();
        enforce_limit(
            DxfResource::SourceBytes,
            limits.max_source_bytes(),
            document.source_len(),
        )?;
        Ok(Self {
            document,
            limits,
            projected_len: document.source_len(),
            pending: Vec::new(),
            replacement_bytes: Vec::new(),
            inverse_bytes: Vec::new(),
        })
    }

    #[must_use]
    pub fn patch_count(&self) -> u64 {
        self.pending.len() as u64
    }

    #[must_use]
    pub const fn projected_len(&self) -> u64 {
        self.projected_len
    }

    pub fn replace_raw_span(
        &mut self,
        source_span: ByteSpan,
        replacement: &[u8],
        cancellation: &DxfCancellationToken,
    ) -> Result<(), DxfError> {
        ensure_not_cancelled(cancellation)?;
        if source_span.end() > self.document.source_len() {
            return Err(DxfError::TransactionSpanOutOfBounds {
                span: source_span,
                source_len: self.document.source_len(),
            });
        }
        if source_span.is_empty() && replacement.is_empty() {
            return Ok(());
        }
        enforce_limit(
            DxfResource::ValueBytes,
            self.limits.max_value_bytes(),
            source_span.len(),
        )?;
        let replacement_len =
            u64::try_from(replacement.len()).map_err(|_| DxfError::OffsetOverflow {
                offset: 0,
                requested: u64::MAX,
            })?;
        enforce_limit(
            DxfResource::ValueBytes,
            self.limits.max_value_bytes(),
            replacement_len,
        )?;
        let next_count = self
            .patch_count()
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        enforce_limit(DxfResource::Records, self.limits.max_records(), next_count)?;

        let insert_at = self.pending.partition_point(|patch| {
            (patch.source_span.start(), patch.source_span.end())
                < (source_span.start(), source_span.end())
        });
        for existing in [
            insert_at
                .checked_sub(1)
                .and_then(|index| self.pending.get(index)),
            self.pending.get(insert_at),
        ]
        .into_iter()
        .flatten()
        {
            if spans_conflict(existing.source_span, source_span) {
                return Err(DxfError::TransactionPatchConflict {
                    existing: existing.source_span,
                    proposed: source_span,
                });
            }
        }

        let projected_len = self
            .projected_len
            .checked_sub(source_span.len())
            .and_then(|len| len.checked_add(replacement_len))
            .ok_or(DxfError::OffsetOverflow {
                offset: self.projected_len,
                requested: replacement_len,
            })?;
        enforce_limit(
            DxfResource::SourceBytes,
            self.limits.max_source_bytes(),
            projected_len,
        )?;

        let replacement_start = vec_len_u64(&self.replacement_bytes)?;
        let inverse_start = vec_len_u64(&self.inverse_bytes)?;
        let replacement_range =
            DxfTransactionByteRange::from_start_and_len(replacement_start, replacement_len)?;
        let inverse_range =
            DxfTransactionByteRange::from_start_and_len(inverse_start, source_span.len())?;
        enforce_limit(
            DxfResource::SourceBytes,
            self.limits.max_source_bytes(),
            replacement_range.end(),
        )?;
        enforce_limit(
            DxfResource::SourceBytes,
            self.limits.max_source_bytes(),
            inverse_range.end(),
        )?;

        let inverse_len =
            usize::try_from(source_span.len()).map_err(|_| DxfError::OffsetOverflow {
                offset: source_span.start(),
                requested: source_span.len(),
            })?;
        let mut inverse = Vec::new();
        inverse
            .try_reserve_exact(inverse_len)
            .map_err(|_| out_of_memory())?;
        inverse.resize(inverse_len, 0);
        self.document.read_span(source_span, &mut inverse)?;
        ensure_not_cancelled(cancellation)?;

        self.pending.try_reserve(1).map_err(|_| out_of_memory())?;
        self.replacement_bytes
            .try_reserve(replacement.len())
            .map_err(|_| out_of_memory())?;
        self.inverse_bytes
            .try_reserve(inverse.len())
            .map_err(|_| out_of_memory())?;
        self.replacement_bytes.extend_from_slice(replacement);
        self.inverse_bytes.extend_from_slice(&inverse);
        self.pending.insert(
            insert_at,
            PendingPatch {
                source_span,
                replacement_range,
                inverse_range,
            },
        );
        self.projected_len = projected_len;
        Ok(())
    }

    pub(crate) fn replace_raw_span_fragments(
        &mut self,
        source_span: ByteSpan,
        fragments: &[&[u8]],
        cancellation: &DxfCancellationToken,
    ) -> Result<(), DxfError> {
        ensure_not_cancelled(cancellation)?;
        if source_span.end() > self.document.source_len() {
            return Err(DxfError::TransactionSpanOutOfBounds {
                span: source_span,
                source_len: self.document.source_len(),
            });
        }
        let mut replacement_len = 0_usize;
        for fragment in fragments {
            replacement_len =
                replacement_len
                    .checked_add(fragment.len())
                    .ok_or(DxfError::OffsetOverflow {
                        offset: 0,
                        requested: u64::MAX,
                    })?;
        }
        if source_span.is_empty() && replacement_len == 0 {
            return Ok(());
        }
        let replacement_len_u64 =
            u64::try_from(replacement_len).map_err(|_| DxfError::OffsetOverflow {
                offset: 0,
                requested: u64::MAX,
            })?;
        for observed in [source_span.len(), replacement_len_u64] {
            enforce_limit(
                DxfResource::SourceBytes,
                self.limits.max_source_bytes(),
                observed,
            )?;
        }
        let next_count = self
            .patch_count()
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        enforce_limit(DxfResource::Records, self.limits.max_records(), next_count)?;

        let insert_at = self.pending.partition_point(|patch| {
            (patch.source_span.start(), patch.source_span.end())
                < (source_span.start(), source_span.end())
        });
        for existing in [
            insert_at
                .checked_sub(1)
                .and_then(|index| self.pending.get(index)),
            self.pending.get(insert_at),
        ]
        .into_iter()
        .flatten()
        {
            if spans_conflict(existing.source_span, source_span) {
                return Err(DxfError::TransactionPatchConflict {
                    existing: existing.source_span,
                    proposed: source_span,
                });
            }
        }

        let projected_len = self
            .projected_len
            .checked_sub(source_span.len())
            .and_then(|len| len.checked_add(replacement_len_u64))
            .ok_or(DxfError::OffsetOverflow {
                offset: self.projected_len,
                requested: replacement_len_u64,
            })?;
        enforce_limit(
            DxfResource::SourceBytes,
            self.limits.max_source_bytes(),
            projected_len,
        )?;
        let replacement_start = vec_len_u64(&self.replacement_bytes)?;
        let inverse_start = vec_len_u64(&self.inverse_bytes)?;
        let replacement_range =
            DxfTransactionByteRange::from_start_and_len(replacement_start, replacement_len_u64)?;
        let inverse_range =
            DxfTransactionByteRange::from_start_and_len(inverse_start, source_span.len())?;
        for observed in [replacement_range.end(), inverse_range.end()] {
            enforce_limit(
                DxfResource::SourceBytes,
                self.limits.max_source_bytes(),
                observed,
            )?;
        }

        let inverse_len =
            usize::try_from(source_span.len()).map_err(|_| DxfError::OffsetOverflow {
                offset: source_span.start(),
                requested: source_span.len(),
            })?;
        let mut inverse = Vec::new();
        inverse
            .try_reserve_exact(inverse_len)
            .map_err(|_| out_of_memory())?;
        inverse.resize(inverse_len, 0);
        self.document.read_span(source_span, &mut inverse)?;
        ensure_not_cancelled(cancellation)?;

        self.pending.try_reserve(1).map_err(|_| out_of_memory())?;
        self.replacement_bytes
            .try_reserve(replacement_len)
            .map_err(|_| out_of_memory())?;
        self.inverse_bytes
            .try_reserve(inverse.len())
            .map_err(|_| out_of_memory())?;
        for fragment in fragments {
            self.replacement_bytes.extend_from_slice(fragment);
        }
        self.inverse_bytes.extend_from_slice(&inverse);
        self.pending.insert(
            insert_at,
            PendingPatch {
                source_span,
                replacement_range,
                inverse_range,
            },
        );
        self.projected_len = projected_len;
        Ok(())
    }

    pub fn finish(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfTransactionPlan, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let mut patches = Vec::new();
        patches
            .try_reserve_exact(self.pending.len())
            .map_err(|_| out_of_memory())?;
        for (ordinal, pending) in self.pending.into_iter().enumerate() {
            patches.push(DxfTransactionPatch {
                ordinal: u32::try_from(ordinal).map_err(|_| invalid_internal_data())?,
                source_span: pending.source_span,
                replacement_range: pending.replacement_range,
                inverse_range: pending.inverse_range,
            });
        }
        Ok(DxfTransactionPlan {
            source_id: self.document.source_id(),
            source_len: self.document.source_len(),
            format: self.document.format(),
            projected_len: self.projected_len,
            patches: patches.into_boxed_slice(),
            replacement_bytes: self.replacement_bytes.into_boxed_slice(),
            inverse_bytes: self.inverse_bytes.into_boxed_slice(),
        })
    }
}

impl<'a> DxfRawDocumentView<'a> {
    pub fn transaction_plan_builder(
        self,
        profile: DxfResourceProfile,
    ) -> Result<DxfTransactionPlanBuilder<'a>, DxfError> {
        DxfTransactionPlanBuilder::new(self, profile)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn transaction_plan_builder<'document>(
        &'document self,
        profile: DxfResourceProfile,
    ) -> Result<DxfTransactionPlanBuilder<'document>, DxfError> {
        DxfRawDocumentView::from(self).transaction_plan_builder(profile)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn transaction_plan_builder<'document>(
        &'document self,
        profile: DxfResourceProfile,
    ) -> Result<DxfTransactionPlanBuilder<'document>, DxfError> {
        DxfRawDocumentView::from(self).transaction_plan_builder(profile)
    }
}

fn spans_conflict(left: ByteSpan, right: ByteSpan) -> bool {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => left.start() == right.start(),
        (true, false) => left.start() > right.start() && left.start() < right.end(),
        (false, true) => right.start() > left.start() && right.start() < left.end(),
        (false, false) => left.start() < right.end() && right.start() < left.end(),
    }
}

fn bytes_for_range(bytes: &[u8], range: DxfTransactionByteRange) -> Option<&[u8]> {
    let start = usize::try_from(range.start()).ok()?;
    let end = usize::try_from(range.end()).ok()?;
    bytes.get(start..end)
}

fn vec_len_u64(bytes: &[u8]) -> Result<u64, DxfError> {
    u64::try_from(bytes.len()).map_err(|_| DxfError::OffsetOverflow {
        offset: 0,
        requested: u64::MAX,
    })
}

fn enforce_limit(resource: DxfResource, limit: u64, observed: u64) -> Result<(), DxfError> {
    if observed > limit {
        Err(DxfError::resource_limit(resource, limit, observed))
    } else {
        Ok(())
    }
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

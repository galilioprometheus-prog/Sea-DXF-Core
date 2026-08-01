//! Explicit insertion planning for one absent common-entity singleton.

use std::io;

use crate::{
    ByteSpan, DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfEncodedEntityGroup, DxfEntityEditValue, DxfEntityField, DxfEntityFieldEvidenceDirectory,
    DxfEntityFieldInsertionAnchor, DxfEntityFieldInsertionAnchorIssue,
    DxfEntityFieldInsertionAnchorOutcome, DxfEntityGroupEncodeIssue, DxfEntityGroupEncoder,
    DxfEntityKey, DxfError, DxfIoOperation, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfResourceProfile, DxfSourceId, DxfTransactionPlan,
};

/// Typed reason why one common-field singleton insertion cannot be planned.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityFieldInsertionIssue {
    Anchor(DxfEntityFieldInsertionAnchorIssue),
    Encoding(DxfEntityGroupEncodeIssue),
}

/// One encoded source-bound singleton insertion and its inverse-capable plan.
#[derive(Debug)]
pub struct DxfEntityFieldInsertionPlan {
    anchor: DxfEntityFieldInsertionAnchor,
    transaction: DxfTransactionPlan,
}

impl DxfEntityFieldInsertionPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.anchor.source_id()
    }

    #[must_use]
    pub const fn key(&self) -> DxfEntityKey {
        self.anchor.key()
    }

    #[must_use]
    pub const fn field(&self) -> DxfEntityField {
        self.anchor.field()
    }

    #[must_use]
    pub const fn anchor(&self) -> DxfEntityFieldInsertionAnchor {
        self.anchor
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

/// Typed result of requesting one explicit common-field singleton insertion.
#[derive(Debug)]
#[non_exhaustive]
pub enum DxfEntityFieldInsertionOutcome {
    Unavailable(DxfEntityFieldInsertionIssue),
    Planned(DxfEntityFieldInsertionPlan),
}

impl DxfRawDocumentView<'_> {
    /// Encodes and plans insertion of one absent common-field singleton.
    pub fn plan_entity_field_insertion(
        self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldInsertionOutcome, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let anchor =
            match self.plan_entity_field_insertion_anchor(evidence, key, field, cancellation)? {
                DxfEntityFieldInsertionAnchorOutcome::Unavailable(issue) => {
                    return Ok(unavailable(DxfEntityFieldInsertionIssue::Anchor(issue)));
                }
                DxfEntityFieldInsertionAnchorOutcome::Planned(anchor) => anchor,
            };
        let version = match self.acad_version_report().state() {
            DxfAcadVersionState::Supported(version) => version,
            _ => return Err(invalid_internal_data()),
        };
        let descriptor = field.descriptor().ok_or_else(invalid_internal_data)?;
        let encoded = match DxfEntityGroupEncoder::new(self.format(), version, profile).encode(
            *descriptor,
            value,
            cancellation,
        )? {
            Ok(encoded) => encoded,
            Err(issue) => {
                return Ok(unavailable(DxfEntityFieldInsertionIssue::Encoding(issue)));
            }
        };
        let insertion = insertion_bytes(self, anchor, &encoded, cancellation)?;
        let source_span = ByteSpan::new(anchor.byte_offset(), anchor.byte_offset())
            .ok_or_else(invalid_internal_data)?;
        let mut builder = self.transaction_plan_builder(profile)?;
        builder.replace_raw_span(source_span, &insertion, cancellation)?;
        let transaction = builder.finish(cancellation)?;
        ensure_not_cancelled(cancellation)?;
        Ok(DxfEntityFieldInsertionOutcome::Planned(
            DxfEntityFieldInsertionPlan {
                anchor,
                transaction,
            },
        ))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn plan_entity_field_insertion(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldInsertionOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_insertion(
            evidence,
            key,
            field,
            value,
            profile,
            cancellation,
        )
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn plan_entity_field_insertion(
        &self,
        evidence: &DxfEntityFieldEvidenceDirectory,
        key: DxfEntityKey,
        field: DxfEntityField,
        value: DxfEntityEditValue<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfEntityFieldInsertionOutcome, DxfError> {
        DxfRawDocumentView::from(self).plan_entity_field_insertion(
            evidence,
            key,
            field,
            value,
            profile,
            cancellation,
        )
    }
}

fn insertion_bytes(
    document: DxfRawDocumentView<'_>,
    anchor: DxfEntityFieldInsertionAnchor,
    encoded: &DxfEncodedEntityGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Vec<u8>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let ending = match document.format() {
        DxfRawDocumentFormat::Ascii => ascii_line_ending(document, anchor)?,
        DxfRawDocumentFormat::Binary => b"".as_slice(),
    };
    let extra = if ending.len() > 1 {
        encoded
            .bytes()
            .iter()
            .filter(|byte| **byte == b'\n')
            .count()
            .checked_mul(ending.len() - 1)
            .ok_or_else(invalid_internal_data)?
    } else {
        0
    };
    let capacity = encoded
        .bytes()
        .len()
        .checked_add(extra)
        .ok_or_else(invalid_internal_data)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| out_of_memory())?;
    if document.format() == DxfRawDocumentFormat::Binary {
        bytes.extend_from_slice(encoded.bytes());
    } else {
        let mut separators = 0_u8;
        for byte in encoded.bytes().iter().copied() {
            if byte == b'\n' {
                bytes.extend_from_slice(ending);
                separators = separators
                    .checked_add(1)
                    .ok_or_else(invalid_internal_data)?;
            } else {
                bytes.push(byte);
            }
        }
        if separators != 2 {
            return Err(invalid_internal_data());
        }
    }
    ensure_not_cancelled(cancellation)?;
    Ok(bytes)
}

fn ascii_line_ending(
    document: DxfRawDocumentView<'_>,
    anchor: DxfEntityFieldInsertionAnchor,
) -> Result<&'static [u8], DxfError> {
    let occurrence = anchor
        .preceding_group_occurrence()
        .ok_or_else(invalid_internal_data)?;
    let group = document
        .group(occurrence)
        .ok_or_else(invalid_internal_data)?;
    let full_span = group.full_span();
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

const fn unavailable(issue: DxfEntityFieldInsertionIssue) -> DxfEntityFieldInsertionOutcome {
    DxfEntityFieldInsertionOutcome::Unavailable(issue)
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
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::OutOfMemory),
    )
}

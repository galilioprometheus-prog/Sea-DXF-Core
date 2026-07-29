//! Source-anchored handle projection over format-neutral raw groups.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfHandle,
    DxfHandleGroupClass, DxfHandleParseIssue, DxfIoOperation, DxfRawDocumentView, DxfRawGroup,
    DxfSourceId, classify_dxf_handle_group_code, parse_dxf_handle_hex,
};

/// Exact result of looking up one raw group occurrence as a handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfRawHandleLookup {
    MissingOccurrence,
    NotHandleGroup(DxfRawGroup),
    Handle(DxfRawHandleValue),
}

/// Parsed or lexically invalid handle tied to its authoritative source group.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfRawHandleValue {
    source_id: DxfSourceId,
    group: DxfRawGroup,
    class: DxfHandleGroupClass,
    parse_result: Result<DxfHandle, DxfHandleParseIssue>,
}

impl DxfRawHandleValue {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn class(self) -> DxfHandleGroupClass {
        self.class
    }

    pub const fn parse_result(self) -> Result<DxfHandle, DxfHandleParseIssue> {
        self.parse_result
    }

    /// Reads the original, unnormalized hexadecimal spelling.
    pub fn read_raw_spelling(
        self,
        document: DxfRawDocumentView<'_>,
        destination: &mut [u8],
    ) -> Result<(), DxfError> {
        if document.source_id() != self.source_id {
            return Err(DxfError::SourceIdentityMismatch {
                expected: self.source_id,
                observed: document.source_id(),
            });
        }
        if document.group(self.group.occurrence()) != Some(self.group) {
            return Err(invalid_internal_data());
        }
        document.read_span(self.group.value_payload_span(), destination)
    }
}

impl DxfRawDocumentView<'_> {
    /// Projects one raw group occurrence as a handle without resolving a target.
    pub fn raw_handle_at(
        self,
        occurrence: u64,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfRawHandleLookup, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let Some(group) = self.group(occurrence) else {
            return Ok(DxfRawHandleLookup::MissingOccurrence);
        };
        let Some(class) = classify_dxf_handle_group_code(group.group_code()) else {
            return Ok(DxfRawHandleLookup::NotHandleGroup(group));
        };
        let parse_result = parse_raw_group_handle(self, group, cancellation)?;
        Ok(DxfRawHandleLookup::Handle(DxfRawHandleValue {
            source_id: self.source_id(),
            group,
            class,
            parse_result,
        }))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn raw_handle_at(
        &self,
        occurrence: u64,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfRawHandleLookup, DxfError> {
        DxfRawDocumentView::from(self).raw_handle_at(occurrence, cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn raw_handle_at(
        &self,
        occurrence: u64,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfRawHandleLookup, DxfError> {
        DxfRawDocumentView::from(self).raw_handle_at(occurrence, cancellation)
    }
}

pub(crate) fn parse_raw_group_handle(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfHandle, DxfHandleParseIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let span = group.value_payload_span();
    if span.len() > DxfHandle::MAX_HEX_DIGITS as u64 {
        return Ok(Err(DxfHandleParseIssue::TooLong));
    }
    let len = usize::try_from(span.len()).map_err(|_| invalid_internal_data())?;
    let mut raw = [0_u8; DxfHandle::MAX_HEX_DIGITS];
    document.read_span(span, &mut raw[..len])?;
    ensure_not_cancelled(cancellation)?;
    Ok(parse_dxf_handle_hex(&raw[..len]))
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

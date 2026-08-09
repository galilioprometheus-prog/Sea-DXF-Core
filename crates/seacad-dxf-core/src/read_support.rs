//! Shared fallible construction primitives for source-backed read projections.

use std::io;

use crate::{DxfCancellationToken, DxfError, DxfIoOperation, DxfSourceId};

pub(crate) fn compact_len(value: usize) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

pub(crate) fn compact_u64(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_internal_data())
}

pub(crate) fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
}

pub(crate) fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

pub(crate) fn invalid_internal_data() -> DxfError {
    read_io_error(io::ErrorKind::InvalidData)
}

pub(crate) fn out_of_memory() -> DxfError {
    read_io_error(io::ErrorKind::OutOfMemory)
}

fn read_io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

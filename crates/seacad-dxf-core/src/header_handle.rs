//! Source-anchored HEADER handle semantics over the shared raw document API.

use std::{fmt, io, num::NonZeroU64};

use crate::{
    ByteSpan, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError,
    DxfGroupCode, DxfHandle, DxfHandleParseIssue, DxfHeaderSchemaMatch, DxfHeaderVariable,
    DxfHeaderVariableLookupState, DxfIoOperation, DxfRawDocumentView, DxfRawGroup,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
    generated::header_schema::{
        DxfHeaderSchemaField, DxfSchemaStorageKind, HEADER_FIELDS, SCHEMA_VERSION,
    },
    raw_handle::parse_raw_group_handle,
};

const HEADER_NAMESPACE: &str = "header";

/// Exact structural or lexical reason why a HEADER handle has no usable value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHeaderHandleIssue {
    InvalidGroupCode(DxfGroupCode),
    InvalidHandle(DxfHandleParseIssue),
    MissingValue,
    MultipleValueGroups { group_count: NonZeroU64 },
    MultipleVariables { occurrence_count: NonZeroU64 },
}

/// Parsed handle tied to its exact source spelling and document identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderHandleValue {
    source_id: DxfSourceId,
    group_occurrence: u32,
    value_span: ByteSpan,
    handle: DxfHandle,
}

impl DxfHeaderHandleValue {
    #[must_use]
    pub const fn source_id(self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn group_occurrence(self) -> u64 {
        self.group_occurrence as u64
    }

    #[must_use]
    pub const fn value_span(self) -> ByteSpan {
        self.value_span
    }

    #[must_use]
    pub const fn handle(self) -> DxfHandle {
        self.handle
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
        let group = document
            .group(self.group_occurrence())
            .ok_or_else(invalid_internal_data)?;
        if group.value_payload_span() != self.value_span {
            return Err(invalid_internal_data());
        }
        document.read_span(self.value_span, destination)
    }
}

/// One handle field selected from the complete generated HEADER schema.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderHandleEntry {
    schema_ordinal: u32,
    schema_field_id: &'static str,
    dxf_name: &'static str,
    group_code: i16,
    value: DxfSemanticValue<DxfHeaderHandleValue, DxfHeaderHandleIssue>,
}

impl DxfHeaderHandleEntry {
    /// Append-only position in the complete HEADER schema.
    #[must_use]
    pub const fn schema_ordinal(self) -> u64 {
        self.schema_ordinal as u64
    }

    #[must_use]
    pub const fn schema_field_id(self) -> &'static str {
        self.schema_field_id
    }

    #[must_use]
    pub const fn dxf_name(self) -> &'static str {
        self.dxf_name
    }

    #[must_use]
    pub const fn group_code(self) -> i16 {
        self.group_code
    }

    #[must_use]
    pub const fn value(&self) -> &DxfSemanticValue<DxfHeaderHandleValue, DxfHeaderHandleIssue> {
        &self.value
    }
}

/// Schema-ordered directory of every generated HEADER handle field.
pub struct DxfHeaderHandleDirectory {
    source_id: DxfSourceId,
    entries: Box<[DxfHeaderHandleEntry]>,
}

impl DxfHeaderHandleDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let raw_directory = document.resolve_header_schema(cancellation)?;
        let handle_count = HEADER_FIELDS
            .iter()
            .filter(|field| field.storage == DxfSchemaStorageKind::Handle)
            .count();
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(handle_count)
            .map_err(|_| out_of_memory())?;
        for (ordinal, field) in HEADER_FIELDS.iter().enumerate() {
            ensure_not_cancelled(cancellation)?;
            if field.storage != DxfSchemaStorageKind::Handle {
                continue;
            }
            let matched = schema_match(&raw_directory, ordinal, field)?;
            let spec = FieldSpec::from_schema(field)?;
            entries.push(DxfHeaderHandleEntry {
                schema_ordinal: u32::try_from(ordinal).map_err(|_| invalid_internal_data())?,
                schema_field_id: field.id,
                dxf_name: field.dxf_name,
                group_code: spec.group_code,
                value: handle_field(document, matched, spec, cancellation)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> &'static str {
        SCHEMA_VERSION
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHeaderHandleEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, schema_field_id: &str) -> Option<&DxfHeaderHandleEntry> {
        self.entries
            .iter()
            .find(|entry| entry.schema_field_id == schema_field_id)
    }

    #[must_use]
    pub fn entry_at_schema_ordinal(&self, schema_ordinal: u64) -> Option<&DxfHeaderHandleEntry> {
        let schema_ordinal = u32::try_from(schema_ordinal).ok()?;
        self.entries
            .binary_search_by_key(&schema_ordinal, |entry| entry.schema_ordinal)
            .ok()
            .and_then(|index| self.entries.get(index))
    }
}

impl fmt::Debug for DxfHeaderHandleDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfHeaderHandleDirectory")
            .field("source_id", &self.source_id)
            .field("handle_field_count", &self.entries.len())
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    /// Resolves every generated HEADER handle field without following references.
    pub fn header_handle_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderHandleDirectory, DxfError> {
        DxfHeaderHandleDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn header_handle_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderHandleDirectory, DxfError> {
        DxfRawDocumentView::from(self).header_handle_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn header_handle_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderHandleDirectory, DxfError> {
        DxfRawDocumentView::from(self).header_handle_directory(cancellation)
    }
}

#[derive(Clone, Copy)]
struct FieldSpec {
    id: &'static str,
    group_code: i16,
}

impl FieldSpec {
    fn from_schema(field: &DxfHeaderSchemaField) -> Result<Self, DxfError> {
        let [group_code] = field.group_codes else {
            return Err(invalid_internal_data());
        };
        Ok(Self {
            id: field.id,
            group_code: *group_code,
        })
    }
}

fn schema_match<'a>(
    directory: &'a crate::DxfHeaderSchemaDirectory,
    ordinal: usize,
    field: &DxfHeaderSchemaField,
) -> Result<&'a DxfHeaderSchemaMatch, DxfError> {
    let ordinal = u64::try_from(ordinal).map_err(|_| invalid_internal_data())?;
    let matched = directory
        .match_at(ordinal)
        .ok_or_else(invalid_internal_data)?;
    if matched.schema_ordinal() != ordinal
        || matched.schema_field_id() != field.id
        || matched.dxf_name() != field.dxf_name
    {
        return Err(invalid_internal_data());
    }
    Ok(matched)
}

fn handle_field(
    document: DxfRawDocumentView<'_>,
    matched: &DxfHeaderSchemaMatch,
    spec: FieldSpec,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSemanticValue<DxfHeaderHandleValue, DxfHeaderHandleIssue>, DxfError> {
    ensure_not_cancelled(cancellation)?;
    let field = DxfSemanticFieldProvenance::new(document.source_id(), HEADER_NAMESPACE, spec.id);
    match matched.state() {
        DxfHeaderVariableLookupState::Absent => Ok(DxfSemanticValue::absent(field)),
        DxfHeaderVariableLookupState::Ambiguous => {
            let occurrence_count =
                NonZeroU64::new(matched.occurrence_count()).ok_or_else(invalid_internal_data)?;
            let raw = matched
                .conflicting()
                .or_else(|| matched.primary())
                .map(|variable| variable_provenance(document, variable))
                .transpose()?;
            Ok(DxfSemanticValue::invalid(
                DxfHeaderHandleIssue::MultipleVariables { occurrence_count },
                field,
                raw,
            ))
        }
        DxfHeaderVariableLookupState::Unique => {
            let variable = matched.primary().ok_or_else(invalid_internal_data)?;
            handle_from_unique_variable(document, variable, spec, field, cancellation)
        }
    }
}

fn handle_from_unique_variable(
    document: DxfRawDocumentView<'_>,
    variable: DxfHeaderVariable,
    spec: FieldSpec,
    field: DxfSemanticFieldProvenance,
    cancellation: &DxfCancellationToken,
) -> Result<DxfSemanticValue<DxfHeaderHandleValue, DxfHeaderHandleIssue>, DxfError> {
    let range = variable.value_groups();
    match range.len() {
        0 => Ok(DxfSemanticValue::invalid(
            DxfHeaderHandleIssue::MissingValue,
            field,
            Some(marker_provenance(variable)?),
        )),
        1 => {
            let group = document
                .group(range.start())
                .ok_or_else(invalid_internal_data)?;
            let raw = group_provenance(group)?;
            if group.group_code().value() != spec.group_code {
                return Ok(DxfSemanticValue::invalid(
                    DxfHeaderHandleIssue::InvalidGroupCode(group.group_code()),
                    field,
                    Some(raw),
                ));
            }
            match parse_raw_group_handle(document, group, cancellation)? {
                Ok(handle) => {
                    let group_occurrence =
                        u32::try_from(group.occurrence()).map_err(|_| invalid_internal_data())?;
                    Ok(DxfSemanticValue::explicit(
                        DxfHeaderHandleValue {
                            source_id: document.source_id(),
                            group_occurrence,
                            value_span: group.value_payload_span(),
                            handle,
                        },
                        field,
                        raw,
                    ))
                }
                Err(issue) => Ok(DxfSemanticValue::invalid(
                    DxfHeaderHandleIssue::InvalidHandle(issue),
                    field,
                    Some(raw),
                )),
            }
        }
        count => {
            let group_count = NonZeroU64::new(count).ok_or_else(invalid_internal_data)?;
            let conflict_occurrence = range
                .start()
                .checked_add(1)
                .ok_or_else(invalid_internal_data)?;
            let conflict = document
                .group(conflict_occurrence)
                .ok_or_else(invalid_internal_data)?;
            Ok(DxfSemanticValue::invalid(
                DxfHeaderHandleIssue::MultipleValueGroups { group_count },
                field,
                Some(group_provenance(conflict)?),
            ))
        }
    }
}

fn variable_provenance(
    document: DxfRawDocumentView<'_>,
    variable: DxfHeaderVariable,
) -> Result<DxfRawValueProvenance, DxfError> {
    if variable.value_groups().is_empty() {
        return marker_provenance(variable);
    }
    let group = document
        .group(variable.value_groups().start())
        .ok_or_else(invalid_internal_data)?;
    group_provenance(group)
}

fn marker_provenance(variable: DxfHeaderVariable) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(variable.marker_occurrence(), variable.name_span())
        .ok_or_else(invalid_internal_data)
}

fn group_provenance(group: DxfRawGroup) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(group.occurrence(), group.value_payload_span())
        .ok_or_else(invalid_internal_data)
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

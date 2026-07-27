//! One-pass exact matching from raw HEADER variables to generated schema fields.

use std::{fmt, io};

use crate::{
    DxfCancellationToken, DxfError, DxfHeaderVariable, DxfHeaderVariableIndex,
    DxfHeaderVariableLookupState, DxfIoOperation, DxfRawDocumentView, DxfSourceId,
    generated::header_schema::{DxfHeaderSchemaField, HEADER_FIELDS, SCHEMA_VERSION},
};

const CANCELLATION_CHECK_INTERVAL: usize = 4_096;

/// Exact occurrence evidence for one generated HEADER schema field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderSchemaMatch {
    schema_ordinal: u32,
    schema_field_id: &'static str,
    dxf_name: &'static str,
    occurrence_count: u32,
    primary: Option<DxfHeaderVariable>,
    conflicting: Option<DxfHeaderVariable>,
}

impl DxfHeaderSchemaMatch {
    /// Append-only position in this schema namespace.
    ///
    /// Persist `schema_field_id` as the canonical identity.
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
    pub const fn state(self) -> DxfHeaderVariableLookupState {
        match self.occurrence_count {
            0 => DxfHeaderVariableLookupState::Absent,
            1 => DxfHeaderVariableLookupState::Unique,
            _ => DxfHeaderVariableLookupState::Ambiguous,
        }
    }

    #[must_use]
    pub const fn occurrence_count(self) -> u64 {
        self.occurrence_count as u64
    }

    #[must_use]
    pub const fn primary(self) -> Option<DxfHeaderVariable> {
        self.primary
    }

    #[must_use]
    pub const fn conflicting(self) -> Option<DxfHeaderVariable> {
        self.conflicting
    }

    fn empty(schema_ordinal: u32, field: &DxfHeaderSchemaField) -> Self {
        Self {
            schema_ordinal,
            schema_field_id: field.id,
            dxf_name: field.dxf_name,
            occurrence_count: 0,
            primary: None,
            conflicting: None,
        }
    }

    fn observe(&mut self, variable: DxfHeaderVariable) -> Result<(), DxfError> {
        self.occurrence_count = self
            .occurrence_count
            .checked_add(1)
            .ok_or_else(invalid_internal_data)?;
        if self.primary.is_none() {
            self.primary = Some(variable);
        } else if self.conflicting.is_none() {
            self.conflicting = Some(variable);
        }
        Ok(())
    }
}

/// Lazy schema-wide directory resolved from one immutable raw document.
pub struct DxfHeaderSchemaDirectory {
    source_id: DxfSourceId,
    matches: Box<[DxfHeaderSchemaMatch]>,
}

impl DxfHeaderSchemaDirectory {
    #[must_use]
    pub const fn schema_version(&self) -> &'static str {
        SCHEMA_VERSION
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub fn matches(&self) -> &[DxfHeaderSchemaMatch] {
        &self.matches
    }

    #[must_use]
    pub fn entry(&self, schema_field_id: &str) -> Option<&DxfHeaderSchemaMatch> {
        self.matches
            .iter()
            .find(|entry| entry.schema_field_id == schema_field_id)
    }

    /// Finds one field by its append-only schema position.
    #[must_use]
    pub fn match_at(&self, schema_ordinal: u64) -> Option<&DxfHeaderSchemaMatch> {
        usize::try_from(schema_ordinal)
            .ok()
            .and_then(|ordinal| self.matches.get(ordinal))
    }
}

impl fmt::Debug for DxfHeaderSchemaDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DxfHeaderSchemaDirectory")
            .field("source_id", &self.source_id)
            .field("schema_field_count", &self.matches.len())
            .finish()
    }
}

#[derive(Clone, Copy)]
struct SchemaCandidate {
    name_hash: u64,
    schema_ordinal: u32,
}

impl DxfRawDocumentView<'_> {
    /// Resolves every generated HEADER field in one bounded schema-wide pass.
    pub fn resolve_header_schema(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHeaderSchemaDirectory, DxfError> {
        resolve_header_schema_in_index(self, self.header_variable_index(), cancellation)
    }
}

fn resolve_header_schema_in_index(
    view: DxfRawDocumentView<'_>,
    index: &DxfHeaderVariableIndex,
    cancellation: &DxfCancellationToken,
) -> Result<DxfHeaderSchemaDirectory, DxfError> {
    ensure_not_cancelled(cancellation)?;
    if index.source_id() != view.source_id() {
        return Err(DxfError::SourceIdentityMismatch {
            expected: view.source_id(),
            observed: index.source_id(),
        });
    }

    let mut matches = Vec::new();
    matches
        .try_reserve_exact(HEADER_FIELDS.len())
        .map_err(|_| out_of_memory())?;
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(HEADER_FIELDS.len())
        .map_err(|_| out_of_memory())?;
    for (ordinal, field) in HEADER_FIELDS.iter().enumerate() {
        let schema_ordinal = u32::try_from(ordinal).map_err(|_| invalid_internal_data())?;
        matches.push(DxfHeaderSchemaMatch::empty(schema_ordinal, field));
        candidates.push(SchemaCandidate {
            name_hash: index.hash_name(field.dxf_name.as_bytes()),
            schema_ordinal,
        });
    }
    candidates.sort_unstable_by(|left, right| {
        left.name_hash
            .cmp(&right.name_hash)
            .then(left.schema_ordinal.cmp(&right.schema_ordinal))
    });

    for (variable_ordinal, variable) in index.variables().iter().copied().enumerate() {
        if variable_ordinal % CANCELLATION_CHECK_INTERVAL == 0 {
            ensure_not_cancelled(cancellation)?;
        }
        let name_hash = index
            .variable_name_hash(variable_ordinal)
            .ok_or_else(invalid_internal_data)?;
        let first = candidates.partition_point(|candidate| candidate.name_hash < name_hash);
        let end = candidates.partition_point(|candidate| candidate.name_hash <= name_hash);
        for candidate in &candidates[first..end] {
            let ordinal =
                usize::try_from(candidate.schema_ordinal).map_err(|_| invalid_internal_data())?;
            let field = HEADER_FIELDS
                .get(ordinal)
                .ok_or_else(invalid_internal_data)?;
            if view.raw_span_equals_exact(variable.name_span(), field.dxf_name.as_bytes())? {
                matches
                    .get_mut(ordinal)
                    .ok_or_else(invalid_internal_data)?
                    .observe(variable)?;
            }
        }
    }
    ensure_not_cancelled(cancellation)?;

    Ok(DxfHeaderSchemaDirectory {
        source_id: view.source_id(),
        matches: matches.into_boxed_slice(),
    })
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        io,
        mem::size_of,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{DxfHeaderSchemaDirectory, DxfHeaderSchemaMatch, resolve_header_schema_in_index};
    use crate::{
        DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
        DxfByteSource, DxfCancellationToken, DxfError, DxfHeaderVariableLookupState,
        DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
        NoopDxfReadObserver,
    };

    #[test]
    fn every_supported_version_has_ascii_binary_schema_parity() -> Result<(), Box<dyn Error>> {
        for version in DxfAcadVersion::SUPPORTED {
            let ascii_bytes = ascii_fixture(version.code(), 0);
            let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
            let ascii = open_ascii(&ascii_source)?;
            assert_standard_directory(DxfRawDocumentView::from(&ascii))?;

            let binary_bytes = binary_fixture(version)?;
            let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
            let binary = open_binary(&binary_source)?;
            assert_standard_directory(DxfRawDocumentView::from(&binary))?;
        }
        Ok(())
    }

    #[test]
    fn duplicate_and_unknown_variables_remain_distinct() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$UNKNOWN_LONG_NAME\n1\nX\n9\n$HANDSEED\n5\nA\n9\n$HANDSEED\n5\nB\n0\nENDSEC\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let directory = DxfRawDocumentView::from(&document)
            .resolve_header_schema(&DxfCancellationToken::default())?;

        assert_eq!(directory.matches().len(), 117);
        assert!(directory.entry("unknown").is_none());
        assert_match(
            directory.entry("acadver"),
            DxfHeaderVariableLookupState::Unique,
            1,
            2,
            None,
        )?;
        assert_match(
            directory.entry("dwgcodepage"),
            DxfHeaderVariableLookupState::Absent,
            0,
            0,
            None,
        )?;
        assert_match(
            directory.entry("handseed"),
            DxfHeaderVariableLookupState::Ambiguous,
            2,
            6,
            Some(8),
        )?;
        assert_eq!(document.header_variable_index().variables().len(), 4);
        Ok(())
    }

    #[test]
    fn unknown_names_do_not_trigger_source_reads() -> Result<(), Box<dyn Error>> {
        let bytes = ascii_fixture("AC1032", 256);
        let source = CountingSource::new(&bytes);
        let document = open_ascii(&source)?;
        let reads_after_open = source.reads();
        let directory = DxfRawDocumentView::from(&document)
            .resolve_header_schema(&DxfCancellationToken::default())?;

        assert_eq!(directory.matches().len(), 117);
        assert_eq!(source.reads(), reads_after_open + 9);
        Ok(())
    }

    #[test]
    fn fingerprint_collision_cannot_claim_a_schema_field() -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$HANDSXXX\n5\nA\n0\nENDSEC\n0\nEOF\n";
        let source = CountingSource::new(bytes);
        let document = open_ascii(&source)?;
        let view = DxfRawDocumentView::from(&document);
        let index = view.header_variable_index();
        let forced = index
            .with_name_hash_for_test(1, index.hash_name(b"$HANDSEED"))
            .ok_or(io::Error::other("missing variable hash"))?;
        let reads_after_open = source.reads();

        let directory =
            resolve_header_schema_in_index(view, &forced, &DxfCancellationToken::default())?;
        assert_match(
            directory.entry("handseed"),
            DxfHeaderVariableLookupState::Absent,
            0,
            0,
            None,
        )?;
        assert!(source.reads() > reads_after_open);
        Ok(())
    }

    #[test]
    fn mismatched_source_identity_fails_closed() -> Result<(), Box<dyn Error>> {
        let first_bytes = ascii_fixture("AC1032", 0);
        let first_source = DxfMemorySource::new(&first_bytes, DxfResourceProfile::Safe)?;
        let first = open_ascii(&first_source)?;
        let second_bytes = ascii_fixture("AC1027", 0);
        let second_source = DxfMemorySource::new(&second_bytes, DxfResourceProfile::Safe)?;
        let second = open_ascii(&second_source)?;

        assert!(
            resolve_header_schema_in_index(
                DxfRawDocumentView::from(&first),
                second.header_variable_index(),
                &DxfCancellationToken::default(),
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn pre_cancelled_resolution_stops_before_source_reads() -> Result<(), Box<dyn Error>> {
        let bytes = ascii_fixture("AC1032", 256);
        let source = CountingSource::new(&bytes);
        let document = open_ascii(&source)?;
        let reads_after_open = source.reads();
        let cancellation = DxfCancellationToken::default();
        cancellation.cancel();

        assert!(matches!(
            DxfRawDocumentView::from(&document).resolve_header_schema(&cancellation),
            Err(DxfError::Cancelled)
        ));
        assert_eq!(source.reads(), reads_after_open);
        Ok(())
    }

    #[test]
    fn public_metadata_is_bounded_send_and_sync() {
        assert_copy::<DxfHeaderSchemaMatch>();
        assert_send_sync::<DxfHeaderSchemaMatch>();
        assert_send_sync::<DxfHeaderSchemaDirectory>();
        assert!(size_of::<DxfHeaderSchemaMatch>() <= 128);
    }

    fn assert_standard_directory(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
        let directory = view.resolve_header_schema(&DxfCancellationToken::default())?;
        assert_eq!(directory.source_id(), view.source_id());
        assert_eq!(directory.schema_version(), "dxf.v1");
        assert_eq!(directory.matches().len(), 117);
        assert_match(
            directory.entry("acadmaintver"),
            DxfHeaderVariableLookupState::Unique,
            1,
            4,
            None,
        )?;
        assert_match(
            directory.entry("acadver"),
            DxfHeaderVariableLookupState::Unique,
            1,
            2,
            None,
        )?;
        assert_match(
            directory.entry("angbase"),
            DxfHeaderVariableLookupState::Unique,
            1,
            6,
            None,
        )?;
        assert_match(
            directory.entry("angdir"),
            DxfHeaderVariableLookupState::Unique,
            1,
            8,
            None,
        )?;
        assert_match(
            directory.entry("attmode"),
            DxfHeaderVariableLookupState::Unique,
            1,
            10,
            None,
        )?;
        assert_match(
            directory.entry("aunits"),
            DxfHeaderVariableLookupState::Unique,
            1,
            12,
            None,
        )?;
        assert_match(
            directory.entry("auprec"),
            DxfHeaderVariableLookupState::Unique,
            1,
            14,
            None,
        )?;
        assert_match(
            directory.entry("dwgcodepage"),
            DxfHeaderVariableLookupState::Unique,
            1,
            16,
            None,
        )?;
        assert_match(
            directory.entry("handseed"),
            DxfHeaderVariableLookupState::Unique,
            1,
            18,
            None,
        )?;
        assert_eq!(directory.matches()[0].schema_ordinal(), 0);
        assert_eq!(directory.matches()[0].dxf_name(), "$ACADMAINTVER");
        assert_eq!(
            directory.match_at(10).map(|entry| entry.schema_field_id()),
            Some("handseed")
        );
        let debug = format!("{directory:?}");
        assert!(debug.contains("schema_field_count"));
        assert!(!debug.contains("$ACADVER"));
        Ok(())
    }

    fn assert_match(
        entry: Option<&DxfHeaderSchemaMatch>,
        state: DxfHeaderVariableLookupState,
        count: u64,
        primary: u64,
        conflicting: Option<u64>,
    ) -> Result<(), Box<dyn Error>> {
        let entry = entry.ok_or(io::Error::other("missing schema entry"))?;
        assert_eq!(entry.state(), state);
        assert_eq!(entry.occurrence_count(), count);
        assert_eq!(
            entry.primary().map(|variable| variable.marker_occurrence()),
            (count != 0).then_some(primary)
        );
        assert_eq!(
            entry
                .conflicting()
                .map(|variable| variable.marker_occurrence()),
            conflicting
        );
        Ok(())
    }

    fn ascii_fixture(version: &str, unknown_count: usize) -> Vec<u8> {
        let mut bytes = format!(
            "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n9\n$ACADMAINTVER\n70\n1\n9\n$ANGBASE\n50\n0.5\n9\n$ANGDIR\n70\n1\n9\n$ATTMODE\n70\n2\n9\n$AUNITS\n70\n0\n9\n$AUPREC\n70\n4\n9\n$DWGCODEPAGE\n3\nANSI_1252\n9\n$HANDSEED\n5\nFF\n"
        )
        .into_bytes();
        for index in 0..unknown_count {
            bytes.extend_from_slice(format!("9\n$UNKNOWN_NAME_{index:08}\n1\nX\n").as_bytes());
        }
        bytes.extend_from_slice(b"0\nENDSEC\n0\nEOF\n");
        bytes
    }

    fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        for (code, value) in [
            (0_i16, b"SECTION".as_slice()),
            (2, b"HEADER"),
            (9, b"$ACADVER"),
            (1, version.code().as_bytes()),
        ] {
            push_binary_string(&mut bytes, version, code, value)?;
        }
        push_binary_string(&mut bytes, version, 9, b"$ACADMAINTVER")?;
        push_binary_i16(&mut bytes, version, 70, 1)?;
        push_binary_string(&mut bytes, version, 9, b"$ANGBASE")?;
        push_binary_f64(&mut bytes, version, 50, 0.5)?;
        for (name, value) in [
            (b"$ANGDIR".as_slice(), 1_i16),
            (b"$ATTMODE", 2),
            (b"$AUNITS", 0),
            (b"$AUPREC", 4),
        ] {
            push_binary_string(&mut bytes, version, 9, name)?;
            push_binary_i16(&mut bytes, version, 70, value)?;
        }
        for (code, value) in [
            (9_i16, b"$DWGCODEPAGE".as_slice()),
            (3, b"ANSI_1252"),
            (9, b"$HANDSEED"),
            (5, b"FF"),
            (0, b"ENDSEC"),
            (0, b"EOF"),
        ] {
            push_binary_string(&mut bytes, version, code, value)?;
        }
        Ok(bytes)
    }

    fn push_binary_string(
        bytes: &mut Vec<u8>,
        version: DxfAcadVersion,
        group_code: i16,
        value: &[u8],
    ) -> Result<(), io::Error> {
        push_binary_group_code(bytes, version, group_code)?;
        bytes.extend_from_slice(value);
        bytes.push(0);
        Ok(())
    }

    fn push_binary_i16(
        bytes: &mut Vec<u8>,
        version: DxfAcadVersion,
        group_code: i16,
        value: i16,
    ) -> Result<(), io::Error> {
        push_binary_group_code(bytes, version, group_code)?;
        bytes.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn push_binary_f64(
        bytes: &mut Vec<u8>,
        version: DxfAcadVersion,
        group_code: i16,
        value: f64,
    ) -> Result<(), io::Error> {
        push_binary_group_code(bytes, version, group_code)?;
        bytes.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn push_binary_group_code(
        bytes: &mut Vec<u8>,
        version: DxfAcadVersion,
        group_code: i16,
    ) -> Result<(), io::Error> {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
        } else {
            bytes.extend_from_slice(&group_code.to_le_bytes());
        }
        Ok(())
    }

    fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
        let mut observer = NoopDxfReadObserver;
        DxfAsciiRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &DxfCancellationToken::default(),
            &mut observer,
        )
    }

    fn open_binary<'a>(
        source: &'a dyn DxfByteSource,
    ) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
        let mut observer = NoopDxfReadObserver;
        DxfBinaryRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &DxfCancellationToken::default(),
            &mut observer,
        )
    }

    struct CountingSource<'a> {
        bytes: &'a [u8],
        reads: AtomicU64,
    }

    impl<'a> CountingSource<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self {
                bytes,
                reads: AtomicU64::new(0),
            }
        }

        fn reads(&self) -> u64 {
            self.reads.load(Ordering::Relaxed)
        }
    }

    impl DxfByteSource for CountingSource<'_> {
        fn len(&self) -> u64 {
            self.bytes.len() as u64
        }

        fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
                offset,
                requested: destination.len() as u64,
            })?;
            let Some(available) = self.bytes.get(start..) else {
                return Ok(0);
            };
            let count = available.len().min(destination.len());
            destination[..count].copy_from_slice(&available[..count]);
            Ok(count)
        }
    }

    fn assert_copy<T: Copy>() {}
    fn assert_send_sync<T: Send + Sync>() {}
}

//! Compact one-pass index over every exact variable in exact HEADER sections.

use std::io;

use crate::{ByteSpan, DxfAsciiGroup, DxfError, DxfGroupCode, DxfIoOperation, DxfSourceId};

/// Half-open range of raw group occurrences belonging to a HEADER variable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderGroupRange {
    start: u32,
    end: u32,
}

impl DxfHeaderGroupRange {
    fn new(start: u32, end: u32) -> Result<Self, DxfError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(invalid_source_data())
        }
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start as u64
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end as u64
    }

    #[must_use]
    pub const fn len(self) -> u64 {
        (self.end - self.start) as u64
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// Exact source location and value-group range for one group-code 9 marker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHeaderVariable {
    header_section_occurrence: u32,
    marker_occurrence: u32,
    name_span: ByteSpan,
    value_groups: DxfHeaderGroupRange,
}

impl DxfHeaderVariable {
    #[must_use]
    pub const fn header_section_occurrence(self) -> u64 {
        self.header_section_occurrence as u64
    }

    #[must_use]
    pub const fn marker_occurrence(self) -> u64 {
        self.marker_occurrence as u64
    }

    #[must_use]
    pub const fn name_span(self) -> ByteSpan {
        self.name_span
    }

    #[must_use]
    pub const fn value_groups(self) -> DxfHeaderGroupRange {
        self.value_groups
    }

    #[must_use]
    pub const fn group_range(self) -> DxfHeaderGroupRange {
        DxfHeaderGroupRange {
            start: self.marker_occurrence,
            end: self.value_groups.end,
        }
    }
}

/// Immutable ordered directory of every exact HEADER variable marker.
#[derive(Debug)]
pub struct DxfHeaderVariableIndex {
    source_id: DxfSourceId,
    header_section_count: u32,
    variables: Box<[DxfHeaderVariable]>,
}

impl DxfHeaderVariableIndex {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn header_section_count(&self) -> u64 {
        self.header_section_count as u64
    }

    #[must_use]
    pub fn variables(&self) -> &[DxfHeaderVariable] {
        &self.variables
    }

    #[must_use]
    pub fn variable(&self, ordinal: u64) -> Option<DxfHeaderVariable> {
        usize::try_from(ordinal)
            .ok()
            .and_then(|index| self.variables.get(index))
            .copied()
    }
}

#[derive(Clone, Copy)]
struct OpenVariable {
    header_section_occurrence: u32,
    marker_occurrence: u32,
    name_span: ByteSpan,
}

#[derive(Default)]
pub(crate) struct DxfHeaderVariableTracker {
    pending_section: Option<u32>,
    header_section: Option<u32>,
    open_variable: Option<OpenVariable>,
    header_section_count: u32,
    variables: Vec<DxfHeaderVariable>,
}

impl DxfHeaderVariableTracker {
    pub(crate) fn observe(
        &mut self,
        group: DxfAsciiGroup<'_>,
        is_eof: bool,
    ) -> Result<(), DxfError> {
        self.observe_raw(
            group.occurrence(),
            group.group_code(),
            group.raw_value(),
            group.value_line().content_span(),
            is_eof,
        )
    }

    pub(crate) fn observe_raw(
        &mut self,
        occurrence: u64,
        group_code: DxfGroupCode,
        raw_value: &[u8],
        value_span: ByteSpan,
        is_eof: bool,
    ) -> Result<(), DxfError> {
        let occurrence = compact_occurrence(occurrence)?;
        if let Some(section_occurrence) = self.pending_section.take()
            && group_code.value() == 2
            && raw_value == b"HEADER"
        {
            self.header_section_count = self
                .header_section_count
                .checked_add(1)
                .ok_or_else(invalid_source_data)?;
            self.header_section = Some(section_occurrence);
        }

        if is_eof {
            self.close_variable(occurrence)?;
            self.header_section = None;
            self.pending_section = None;
            return Ok(());
        }

        if group_code.value() == 0 {
            match raw_value {
                b"SECTION" => {
                    self.close_variable(occurrence)?;
                    self.header_section = None;
                    self.pending_section = Some(occurrence);
                }
                b"ENDSEC" => {
                    self.close_variable(occurrence)?;
                    self.header_section = None;
                    self.pending_section = None;
                }
                _ => {}
            }
        } else if let Some(header_section_occurrence) = self.header_section
            && group_code.value() == 9
        {
            self.close_variable(occurrence)?;
            self.open_variable = Some(OpenVariable {
                header_section_occurrence,
                marker_occurrence: occurrence,
                name_span: value_span,
            });
        }
        Ok(())
    }

    pub(crate) fn finish(
        mut self,
        source_id: DxfSourceId,
        total_group_count: u64,
    ) -> Result<DxfHeaderVariableIndex, DxfError> {
        self.close_variable(compact_occurrence(total_group_count)?)?;
        Ok(DxfHeaderVariableIndex {
            source_id,
            header_section_count: self.header_section_count,
            variables: self.variables.into_boxed_slice(),
        })
    }

    fn close_variable(&mut self, end: u32) -> Result<(), DxfError> {
        let Some(open) = self.open_variable.take() else {
            return Ok(());
        };
        let value_start = open
            .marker_occurrence
            .checked_add(1)
            .ok_or_else(invalid_source_data)?;
        let variable = DxfHeaderVariable {
            header_section_occurrence: open.header_section_occurrence,
            marker_occurrence: open.marker_occurrence,
            name_span: open.name_span,
            value_groups: DxfHeaderGroupRange::new(value_start, end)?,
        };
        self.variables.try_reserve(1).map_err(|_| out_of_memory())?;
        self.variables.push(variable);
        Ok(())
    }
}

fn compact_occurrence(value: u64) -> Result<u32, DxfError> {
    u32::try_from(value).map_err(|_| invalid_source_data())
}

fn invalid_source_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io, mem::size_of};

    use super::{DxfHeaderVariable, DxfHeaderVariableIndex, DxfHeaderVariableTracker};
    use crate::{
        ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
        DxfCancellationToken, DxfGroupCode, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
        DxfSourceId, NoopDxfReadObserver,
    };

    #[test]
    fn every_supported_version_has_ascii_binary_index_parity() -> Result<(), Box<dyn Error>> {
        for version in DxfAcadVersion::SUPPORTED {
            let ascii_bytes = ascii_fixture(version.code());
            let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
            let ascii = open_ascii(&ascii_source, DxfReadOptions::strict())?;

            let binary_bytes = binary_fixture(version)?;
            let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
            let binary = open_binary(&binary_source, DxfReadOptions::strict())?;

            for document in [
                &ascii as &dyn IndexedDocument,
                &binary as &dyn IndexedDocument,
            ] {
                assert_standard_index(document)?;
            }
        }
        Ok(())
    }

    #[test]
    fn exact_sections_unknown_names_and_malformed_boundaries_are_accounted()
    -> Result<(), Box<dyn Error>> {
        let bytes = b"0\nSECTION\n2\nENTITIES\n9\n$OUTSIDE\n1\nX\n0\nENDSEC\n0\nSECTION\n2\nheader\n9\n$LOWER\n1\nX\n0\nENDSEC\n0\nSECTION\n2\nHEADER\n1\nPREAMBLE\n9\n$FIRST\n1\nA\n9\n$MULTI\n10\n1.0\n20\n2.0\n0\nSECTION\n2\nHEADER\n9\n$LAST\n0\nEOF\n";
        let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source, DxfReadOptions::strict())?;
        let index = document.header_variable_index();

        assert_eq!(index.source_id(), document.source_id());
        assert_eq!(index.header_section_count(), 2);
        assert_eq!(index.variables().len(), 3);
        assert_variable(index, 0, 10, 13, 14, 15)?;
        assert_variable(index, 1, 10, 15, 16, 18)?;
        assert_variable(index, 2, 18, 20, 21, 21)?;
        assert_name(&document, index.variable(0), b"$FIRST")?;
        assert_name(&document, index.variable(1), b"$MULTI")?;
        assert_name(&document, index.variable(2), b"$LAST")?;
        Ok(())
    }

    #[test]
    fn many_unknown_variables_stay_compact_and_ordered() -> Result<(), Box<dyn Error>> {
        let mut text = String::from("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n");
        for index in 0..256_u16 {
            text.push_str(&format!("9\n$CUSTOM_{index:03}\n1\nVALUE\n"));
        }
        text.push_str("0\nENDSEC\n0\nEOF\n");
        let source = DxfMemorySource::new(text.as_bytes(), DxfResourceProfile::Safe)?;
        let document = open_ascii(&source, DxfReadOptions::strict())?;
        let variables = document.header_variable_index().variables();

        assert_eq!(variables.len(), 257);
        assert!(variables.len() <= document.groups().len());
        assert!(size_of::<DxfHeaderVariable>() <= 32);
        for pair in variables.windows(2) {
            assert_eq!(pair[0].group_range().end(), pair[1].group_range().start());
        }
        assert_name(&document, variables.get(1).copied(), b"$CUSTOM_000")?;
        assert_name(&document, variables.last().copied(), b"$CUSTOM_255")?;
        Ok(())
    }

    #[test]
    fn occurrence_width_overflow_fails_closed() -> Result<(), Box<dyn Error>> {
        let mut tracker = DxfHeaderVariableTracker::default();
        let code = DxfGroupCode::new(9).ok_or(io::Error::other("group code"))?;
        let span = ByteSpan::new(0, 1).ok_or(io::Error::other("span"))?;
        assert!(
            tracker
                .observe_raw(u64::MAX, code, b"$X", span, false)
                .is_err()
        );
        assert!(
            DxfHeaderVariableTracker::default()
                .finish(DxfSourceId::from([0_u8; 32]), u64::MAX)
                .is_err()
        );
        Ok(())
    }

    fn assert_standard_index(document: &dyn IndexedDocument) -> Result<(), Box<dyn Error>> {
        let index = document.header_variable_index();
        assert_eq!(index.source_id(), document.source_id());
        assert_eq!(index.header_section_count(), 1);
        assert_eq!(index.variables().len(), 3);
        assert_variable(index, 0, 0, 2, 3, 4)?;
        assert_variable(index, 1, 0, 4, 5, 7)?;
        assert_variable(index, 2, 0, 7, 8, 8)?;
        assert_name(document, index.variable(0), b"$ACADVER")?;
        assert_name(document, index.variable(1), b"$CUSTOM")?;
        assert_name(document, index.variable(2), b"$EMPTY")?;
        assert_eq!(index.variable(3), None);
        Ok(())
    }

    fn assert_variable(
        index: &DxfHeaderVariableIndex,
        ordinal: u64,
        section: u64,
        marker: u64,
        value_start: u64,
        value_end: u64,
    ) -> Result<(), Box<dyn Error>> {
        let variable = index
            .variable(ordinal)
            .ok_or(io::Error::other("missing variable"))?;
        assert_eq!(variable.header_section_occurrence(), section);
        assert_eq!(variable.marker_occurrence(), marker);
        assert_eq!(variable.value_groups().start(), value_start);
        assert_eq!(variable.value_groups().end(), value_end);
        assert_eq!(variable.value_groups().len(), value_end - value_start);
        assert_eq!(variable.value_groups().is_empty(), value_start == value_end);
        assert_eq!(variable.group_range().start(), marker);
        assert_eq!(variable.group_range().end(), value_end);
        Ok(())
    }

    fn assert_name(
        document: &dyn IndexedDocument,
        variable: Option<DxfHeaderVariable>,
        expected: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let variable = variable.ok_or(io::Error::other("missing variable"))?;
        let mut name = vec![0_u8; expected.len()];
        document.read_span(variable.name_span(), &mut name)?;
        assert_eq!(name, expected);
        Ok(())
    }

    fn ascii_fixture(version: &str) -> Vec<u8> {
        format!(
            "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n9\n$CUSTOM\n1\nA\n3\nB\n9\n$EMPTY\n0\nENDSEC\n0\nEOF\n"
        )
        .into_bytes()
    }

    fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
        let mut bytes = DXF_BINARY_SENTINEL.to_vec();
        for (code, value) in [
            (0_i16, "SECTION"),
            (2, "HEADER"),
            (9, "$ACADVER"),
            (1, version.code()),
            (9, "$CUSTOM"),
            (1, "A"),
            (3, "B"),
            (9, "$EMPTY"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ] {
            push_binary_string(&mut bytes, version, code, value.as_bytes())?;
        }
        Ok(bytes)
    }

    fn push_binary_string(
        bytes: &mut Vec<u8>,
        version: DxfAcadVersion,
        group_code: i16,
        value: &[u8],
    ) -> Result<(), io::Error> {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
        } else {
            bytes.extend_from_slice(&group_code.to_le_bytes());
        }
        bytes.extend_from_slice(value);
        bytes.push(0);
        Ok(())
    }

    fn open_ascii<'a>(
        source: &'a DxfMemorySource<'_>,
        options: DxfReadOptions,
    ) -> Result<DxfAsciiRawDocument<'a>, crate::DxfError> {
        let mut observer = NoopDxfReadObserver;
        DxfAsciiRawDocument::open(
            source,
            options,
            &DxfCancellationToken::default(),
            &mut observer,
        )
    }

    fn open_binary<'a>(
        source: &'a DxfMemorySource<'_>,
        options: DxfReadOptions,
    ) -> Result<DxfBinaryRawDocument<'a>, crate::DxfError> {
        let mut observer = NoopDxfReadObserver;
        DxfBinaryRawDocument::open(
            source,
            options,
            &DxfCancellationToken::default(),
            &mut observer,
        )
    }

    trait IndexedDocument {
        fn source_id(&self) -> DxfSourceId;
        fn header_variable_index(&self) -> &DxfHeaderVariableIndex;
        fn read_span(&self, span: ByteSpan, destination: &mut [u8]) -> Result<(), crate::DxfError>;
    }

    impl IndexedDocument for DxfAsciiRawDocument<'_> {
        fn source_id(&self) -> DxfSourceId {
            self.source_id()
        }

        fn header_variable_index(&self) -> &DxfHeaderVariableIndex {
            self.header_variable_index()
        }

        fn read_span(&self, span: ByteSpan, destination: &mut [u8]) -> Result<(), crate::DxfError> {
            self.read_span(span, destination)
        }
    }

    impl IndexedDocument for DxfBinaryRawDocument<'_> {
        fn source_id(&self) -> DxfSourceId {
            self.source_id()
        }

        fn header_variable_index(&self) -> &DxfHeaderVariableIndex {
            self.header_variable_index()
        }

        fn read_span(&self, span: ByteSpan, destination: &mut [u8]) -> Result<(), crate::DxfError> {
            self.read_span(span, destination)
        }
    }
}

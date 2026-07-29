use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfRawDocumentView, DxfRawRecord,
    DxfRawRecordDirectory, DxfRawRecordSection, DxfRawRecordSectionKind, DxfRawRecordSectionState,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_raw_record_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        assert_directory(DxfRawDocumentView::from(&ascii))?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_directory(DxfRawDocumentView::from(&binary))?;
    }
    Ok(())
}

#[test]
fn interrupted_and_unclosed_sections_are_typed_without_partial_records()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n5\n1\n0\nSECTION\n2\nOBJECTS\n0\nDICTIONARY\n5\n2\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n5\n3\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.raw_record_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.sections().len(), 4);
    assert_eq!(directory.records().len(), 1);
    let expected = [
        (
            DxfRawRecordSectionKind::Entities,
            DxfRawRecordSectionState::Interrupted,
            0,
        ),
        (
            DxfRawRecordSectionKind::Objects,
            DxfRawRecordSectionState::Indexed,
            1,
        ),
        (
            DxfRawRecordSectionKind::Tables,
            DxfRawRecordSectionState::Indexed,
            0,
        ),
        (
            DxfRawRecordSectionKind::Blocks,
            DxfRawRecordSectionState::Unclosed,
            0,
        ),
    ];
    for (section, (kind, state, count)) in directory.sections().iter().zip(expected) {
        assert_eq!(section.kind(), kind);
        assert_eq!(section.state(), state);
        assert_eq!(section.record_range().len(), count);
        assert_eq!(
            directory
                .records_for_structure_section(section.structure_section_ordinal())
                .map(<[DxfRawRecord]>::len),
            Some(usize::try_from(count)?)
        );
    }
    assert_eq!(
        record_type(DxfRawDocumentView::from(&document), directory.records()[0])?,
        b"DICTIONARY"
    );
    Ok(())
}

#[test]
fn directory_is_metadata_only_cancellable_and_publicly_bounded() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfRawRecordDirectory>();
    assert_copy::<DxfRawRecord>();
    assert_copy::<DxfRawRecordSection>();
    assert_send_sync::<DxfRawRecord>();
    assert_send_sync::<DxfRawRecordSection>();

    let bytes = ascii_fixture("AC1032");
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.raw_record_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads(), reads_after_open);
    assert!(std::mem::size_of::<DxfRawRecord>() <= 48);
    assert!(std::mem::size_of::<DxfRawRecordSection>() <= 96);

    let debug = format!("{directory:?}");
    assert!(!debug.contains("XRECORD"));
    assert!(!debug.contains("DICTIONARY"));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.raw_record_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let directory = view.raw_record_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.sections().len(), 5);
    assert_eq!(directory.records().len(), 11);

    let expected_sections = [
        (1, DxfRawRecordSectionKind::Classes, 1),
        (2, DxfRawRecordSectionKind::Tables, 3),
        (3, DxfRawRecordSectionKind::Blocks, 3),
        (4, DxfRawRecordSectionKind::Entities, 2),
        (5, DxfRawRecordSectionKind::Objects, 2),
    ];
    let mut expected_record_start = 0_u64;
    for (section, (structure_ordinal, kind, count)) in
        directory.sections().iter().zip(expected_sections)
    {
        assert_eq!(section.structure_section_ordinal(), structure_ordinal);
        assert_eq!(section.kind(), kind);
        assert_eq!(section.state(), DxfRawRecordSectionState::Indexed);
        assert_eq!(section.record_range().start(), expected_record_start);
        assert_eq!(section.record_range().len(), count);
        assert!(!section.record_range().is_empty());
        assert!(
            section.content_group_range().start() >= section.source_group_range().start()
                && section.content_group_range().end() <= section.source_group_range().end()
        );
        let records = directory
            .records_for_structure_section(structure_ordinal)
            .ok_or(io::Error::other("missing section records"))?;
        assert_eq!(records.len(), usize::try_from(count)?);
        expected_record_start += count;
    }
    assert_eq!(expected_record_start, directory.records().len() as u64);
    assert_eq!(directory.section_by_structure_ordinal(0), None);
    assert_eq!(directory.section_by_structure_ordinal(6), None);
    assert_eq!(directory.section_by_structure_ordinal(7), None);
    assert_eq!(directory.records_for_structure_section(0), None);
    assert_eq!(directory.record(u64::MAX), None);

    let expected_types: [&[u8]; 11] = [
        b"CLASS",
        b"TABLE",
        b"LAYER",
        b"ENDTAB",
        b"BLOCK",
        b"LINE",
        b"ENDBLK",
        b"LINE",
        b"CIRCLE",
        b"DICTIONARY",
        b"XRECORD",
    ];
    for (ordinal, (record, expected_type)) in
        directory.records().iter().zip(expected_types).enumerate()
    {
        let ordinal = u64::try_from(ordinal)?;
        assert_eq!(record.ordinal(), ordinal);
        assert_eq!(directory.record(ordinal), Some(*record));
        assert_eq!(record.marker_occurrence(), record.group_range().start());
        assert!(record.group_range().start() < record.group_range().end());
        assert_eq!(record_type(view, *record)?, expected_type);
        assert_eq!(
            directory.record_for_group(record.marker_occurrence()),
            Some(*record)
        );
        assert_eq!(
            directory.record_for_group(record.group_range().end() - 1),
            Some(*record)
        );
    }

    let header_variable = occurrence_of(view, 9, b"$ACADVER")?;
    assert_eq!(directory.record_for_group(header_variable), None);
    let eof = occurrence_of(view, 0, b"EOF")?;
    assert_eq!(directory.record_for_group(eof), None);
    Ok(())
}

fn record_type(view: DxfRawDocumentView<'_>, record: DxfRawRecord) -> Result<Vec<u8>, DxfError> {
    let group = view
        .group(record.marker_occurrence())
        .ok_or_else(invalid_test_data)?;
    if group.group_code().value() != 0 {
        return Err(invalid_test_data());
    }
    let mut value = vec![
        0_u8;
        usize::try_from(group.value_payload_span().len())
            .map_err(|_| invalid_test_data())?
    ];
    view.read_span(group.value_payload_span(), &mut value)?;
    Ok(value)
}

fn occurrence_of(
    view: DxfRawDocumentView<'_>,
    code: i16,
    value: &[u8],
) -> Result<u64, Box<dyn Error>> {
    for occurrence in 0..view.group_count() {
        let Some(group) = view.group(occurrence) else {
            continue;
        };
        if group.group_code().value() != code
            || group.value_payload_span().len() != value.len() as u64
        {
            continue;
        }
        let mut observed = vec![0_u8; value.len()];
        view.read_span(group.value_payload_span(), &mut observed)?;
        if observed == value {
            return Ok(occurrence);
        }
    }
    Err(io::Error::other("missing group").into())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    let mut text = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n");
    push_ascii_section(&mut text, "CLASSES", &[(0, "CLASS"), (1, "C")]);
    push_ascii_section(
        &mut text,
        "TABLES",
        &[
            (0, "TABLE"),
            (2, "LAYER"),
            (0, "LAYER"),
            (5, "1"),
            (0, "ENDTAB"),
        ],
    );
    push_ascii_section(
        &mut text,
        "BLOCKS",
        &[(0, "BLOCK"), (2, "B"), (0, "LINE"), (5, "2"), (0, "ENDBLK")],
    );
    push_ascii_section(
        &mut text,
        "ENTITIES",
        &[(0, "LINE"), (5, "3"), (1, "A"), (0, "CIRCLE"), (5, "4")],
    );
    push_ascii_section(
        &mut text,
        "OBJECTS",
        &[(0, "DICTIONARY"), (5, "5"), (0, "XRECORD"), (5, "6")],
    );
    push_ascii_section(&mut text, "THUMBNAILIMAGE", &[(1, "THUMB")]);
    push_ascii_section(&mut text, "ACDSDATA", &[(0, "CUSTOM"), (1, "OPAQUE")]);
    text.push_str("0\nEOF\n");
    text.into_bytes()
}

fn push_ascii_section(text: &mut String, name: &str, groups: &[(i16, &str)]) {
    text.push_str(&format!("0\nSECTION\n2\n{name}\n"));
    for (code, value) in groups {
        text.push_str(&format!("{code}\n{value}\n"));
    }
    text.push_str("0\nENDSEC\n");
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary_section(
        &mut bytes,
        version,
        "HEADER",
        &[(9, "$ACADVER"), (1, version.code())],
    )?;
    push_binary_section(&mut bytes, version, "CLASSES", &[(0, "CLASS"), (1, "C")])?;
    push_binary_section(
        &mut bytes,
        version,
        "TABLES",
        &[
            (0, "TABLE"),
            (2, "LAYER"),
            (0, "LAYER"),
            (5, "1"),
            (0, "ENDTAB"),
        ],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "BLOCKS",
        &[(0, "BLOCK"), (2, "B"), (0, "LINE"), (5, "2"), (0, "ENDBLK")],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "ENTITIES",
        &[(0, "LINE"), (5, "3"), (1, "A"), (0, "CIRCLE"), (5, "4")],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "OBJECTS",
        &[(0, "DICTIONARY"), (5, "5"), (0, "XRECORD"), (5, "6")],
    )?;
    push_binary_section(&mut bytes, version, "THUMBNAILIMAGE", &[(1, "THUMB")])?;
    push_binary_section(
        &mut bytes,
        version,
        "ACDSDATA",
        &[(0, "CUSTOM"), (1, "OPAQUE")],
    )?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_section(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &str,
    groups: &[(i16, &str)],
) -> Result<(), io::Error> {
    for (code, value) in [(0, "SECTION"), (2, name)] {
        push_binary_string(bytes, version, code, value.as_bytes())?;
    }
    for (code, value) in groups {
        push_binary_string(bytes, version, *code, value.as_bytes())?;
    }
    push_binary_string(bytes, version, 0, b"ENDSEC")
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

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
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

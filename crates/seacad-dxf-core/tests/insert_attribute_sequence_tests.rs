use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInsertAttributeSequenceDirectory,
    DxfInsertAttributeSequenceEntry, DxfInsertAttributeSequenceState, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type SequenceSummary = (
    Option<i16>,
    DxfInsertAttributeSequenceState,
    u64,
    Option<u64>,
);

#[test]
fn every_supported_dialect_has_ascii_binary_sequence_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_sequence_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_sequence_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory);
        assert_directory(&binary_directory);
        assert_eq!(
            sequence_summaries(&ascii_directory),
            sequence_summaries(&binary_directory)
        );
    }
    Ok(())
}

#[test]
fn invalid_duplicate_flags_and_exact_marker_case_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n10\n0\n20\n0\n30\n0\n66\nx\n0\nATTRIB\n0\nSEQEND\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n66\n1\n0\nATTRIB\n0\nSEQEND\n0\nINSERT\n2\nC\n10\n0\n20\n0\n30\n0\n66\n1\n0\nattrib\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_sequence_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfInsertAttributeSequenceState::FlagUnavailable,
            DxfInsertAttributeSequenceState::FlagUnavailable,
            DxfInsertAttributeSequenceState::Interrupted,
        ]
    );
    for entry in &directory.entries()[0..2] {
        assert_eq!(
            directory
                .attributes_for_insert_raw_ordinal(entry.insert().record().ordinal())
                .map(|records| records.len()),
            Some(0)
        );
        assert_eq!(entry.boundary_record(), None);
    }
    let interrupted = directory.entries()[2];
    assert_eq!(interrupted.attribute_range().len(), 0);
    assert!(interrupted.boundary_record().is_some());
    Ok(())
}

#[test]
fn zero_flag_does_not_consume_following_attribute_records() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n10\n0\n20\n0\n30\n0\n0\nATTRIB\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_sequence_directory(&DxfCancellationToken::default())?;
    let entry = directory.entries()[0];
    assert_eq!(entry.attributes_follow_value(), Some(0));
    assert_eq!(
        entry.state(),
        DxfInsertAttributeSequenceState::NoAttributesFollow
    );
    assert_eq!(entry.attribute_range().len(), 0);
    assert_eq!(entry.boundary_record(), None);
    assert!(directory.attribute_records().is_empty());
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeSequenceEntry>();
    assert_send_sync::<DxfInsertAttributeSequenceDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_sequence_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.insert_attribute_sequence_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.semantic_directory().source_id()
    );
    assert_eq!(directory.entry_for_insert_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.attributes_for_insert_raw_ordinal(u64::MAX), None);
    for entry in directory.entries() {
        let ordinal = entry.insert().record().ordinal();
        assert_eq!(
            directory.entry_for_insert_raw_ordinal(ordinal),
            Some(*entry)
        );
        assert_eq!(
            directory
                .attributes_for_insert_raw_ordinal(ordinal)
                .map(|records| records.len() as u64),
            Some(entry.attribute_range().len())
        );
    }
    Ok(())
}

fn assert_directory(directory: &DxfInsertAttributeSequenceDirectory) {
    assert_eq!(directory.entries().len(), 5);
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| (
                entry.attributes_follow_value(),
                entry.state(),
                entry.attribute_range().len(),
                entry.boundary_record().is_some(),
            ))
            .collect::<Vec<_>>(),
        [
            (Some(1), DxfInsertAttributeSequenceState::Closed, 2, true,),
            (Some(2), DxfInsertAttributeSequenceState::Closed, 0, true,),
            (
                Some(0),
                DxfInsertAttributeSequenceState::NoAttributesFollow,
                0,
                false,
            ),
            (
                Some(1),
                DxfInsertAttributeSequenceState::Interrupted,
                0,
                true,
            ),
            (Some(1), DxfInsertAttributeSequenceState::Unclosed, 0, false,),
        ]
    );
    assert_eq!(directory.attribute_records().len(), 2);
}

fn sequence_summaries(directory: &DxfInsertAttributeSequenceDirectory) -> Vec<SequenceSummary> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.attributes_follow_value(),
                entry.state(),
                entry.attribute_range().len(),
                entry
                    .boundary_record()
                    .map(|record| record.section_record_ordinal()),
            )
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n1\nx\n0\nATTRIB\n1\ny\n0\nSEQEND\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n2\n0\nSEQEND\n0\nINSERT\n2\nC\n10\n0\n20\n0\n30\n0\n0\nATTRIB\n1\norphan\n0\nSEQEND\n0\nINSERT\n2\nD\n10\n0\n20\n0\n30\n0\n66\n1\n0\nLINE\n0\nINSERT\n2\nE\n10\n0\n20\n0\n30\n0\n66\n1\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_insert(&mut bytes, version, b"A", Some(1))?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 1, b"x")?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 1, b"y")?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_insert(&mut bytes, version, b"B", Some(2))?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_insert(&mut bytes, version, b"C", None)?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 1, b"orphan")?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_insert(&mut bytes, version, b"D", Some(1))?;
    push_string(&mut bytes, version, 0, b"LINE")?;
    push_insert(&mut bytes, version, b"E", Some(1))?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_insert(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
    flag: Option<i16>,
) -> io::Result<()> {
    push_string(bytes, version, 0, b"INSERT")?;
    push_string(bytes, version, 2, name)?;
    for code in [10, 20, 30] {
        push_double(bytes, version, code, 0.0)?;
    }
    if let Some(value) = flag {
        push_i16(bytes, version, 66, value)?;
    }
    Ok(())
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
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

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

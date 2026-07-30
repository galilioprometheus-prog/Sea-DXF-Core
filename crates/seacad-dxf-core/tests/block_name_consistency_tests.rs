use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockDefinitionState, DxfBlockNameConsistencyDirectory, DxfBlockNameConsistencyEntry,
    DxfBlockNameConsistencyState, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_ascii_binary_name_consistency_parity() -> Result<(), Box<dyn Error>>
{
    let expected = [
        DxfBlockNameConsistencyState::Matched,
        DxfBlockNameConsistencyState::Conflicting,
        DxfBlockNameConsistencyState::NotComparable,
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.block_name_consistency_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.block_name_consistency_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory, &expected)?;
        assert_directory(&binary_directory, &expected)?;
    }
    Ok(())
}

#[test]
fn empty_case_utf8_duplicate_and_missing_names_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\n\n3\n\n0\nENDBLK\n0\nBLOCK\n2\nCase\n3\ncase\n0\nENDBLK\n0\nBLOCK\n2\nBiển\n3\nBiển\n0\nENDBLK\n0\nBLOCK\n2\nA\n2\nA\n3\nA\n0\nENDBLK\n0\nBLOCK\n3\nA\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes.as_bytes(), DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_name_consistency_directory(&DxfCancellationToken::default())?;

    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::Conflicting,
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::NotComparable,
            DxfBlockNameConsistencyState::NotComparable,
        ]
    );
    Ok(())
}

#[test]
fn long_untrusted_names_compare_in_bounded_chunks() -> Result<(), Box<dyn Error>> {
    let primary = vec![b'a'; 4 * 1024 + 1];
    let mut conflicting = primary.clone();
    let last = conflicting
        .last_mut()
        .ok_or(io::Error::other("name byte"))?;
    *last = b'b';

    let mut bytes =
        b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\n"
            .to_vec();
    bytes.extend_from_slice(&primary);
    bytes.extend_from_slice(b"\n3\n");
    bytes.extend_from_slice(&primary);
    bytes.extend_from_slice(b"\n0\nENDBLK\n0\nBLOCK\n2\n");
    bytes.extend_from_slice(&primary);
    bytes.extend_from_slice(b"\n3\n");
    bytes.extend_from_slice(&conflicting);
    bytes.extend_from_slice(b"\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n");

    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_name_consistency_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::Conflicting,
        ]
    );
    Ok(())
}

#[test]
fn comparisons_remain_available_for_every_definition_state() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n3\nA\n0\nLINE\n0\nBLOCK\n2\nB\n3\nB\n0\nLINE\n0\nENDBLK\n0\nBLOCK\n2\nC\n3\nC\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_name_consistency_directory(&DxfCancellationToken::default())?;
    let expected_states = [
        DxfBlockDefinitionState::Interrupted,
        DxfBlockDefinitionState::Closed,
        DxfBlockDefinitionState::Unclosed,
    ];

    assert_eq!(directory.entries().len(), expected_states.len());
    for (entry, definition_state) in directory.entries().iter().zip(expected_states) {
        assert_eq!(entry.record().definition().state(), definition_state);
        assert_eq!(entry.state(), DxfBlockNameConsistencyState::Matched);
    }
    Ok(())
}

#[test]
fn cancellation_lookups_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.block_name_consistency_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.block_name_consistency_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.semantic_directory().source_id()
    );
    assert_eq!(directory.entry_for_block_raw_ordinal(u64::MAX), None);
    for entry in directory.entries() {
        assert_eq!(
            directory
                .entry_for_block_raw_ordinal(entry.record().definition().block_record().ordinal()),
            Some(*entry)
        );
    }
    assert_copy::<DxfBlockNameConsistencyEntry>();
    assert_copy::<DxfBlockNameConsistencyState>();
    assert_send_sync::<DxfBlockNameConsistencyDirectory>();
    Ok(())
}

fn assert_directory(
    directory: &DxfBlockNameConsistencyDirectory,
    expected: &[DxfBlockNameConsistencyState],
) -> Result<(), io::Error> {
    assert_eq!(directory.entries().len(), expected.len());
    assert_eq!(
        directory.semantic_directory().records().len(),
        expected.len()
    );
    for (entry, state) in directory.entries().iter().zip(expected) {
        assert_eq!(entry.state(), *state);
        assert_eq!(
            directory
                .entry_for_block_raw_ordinal(entry.record().definition().block_record().ordinal()),
            Some(*entry)
        );
    }
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n3\nA\n0\nENDBLK\n0\nBLOCK\n2\nB\n3\nC\n0\nENDBLK\n0\nBLOCK\n2\nD\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"BLOCKS")?;
    for names in [
        (Some(b"A".as_slice()), Some(b"A".as_slice())),
        (Some(b"B".as_slice()), Some(b"C".as_slice())),
        (Some(b"D".as_slice()), None),
    ] {
        push_string(&mut bytes, version, 0, b"BLOCK")?;
        if let Some(primary) = names.0 {
            push_string(&mut bytes, version, 2, primary)?;
        }
        if let Some(secondary) = names.1 {
            push_string(&mut bytes, version, 3, secondary)?;
        }
        push_string(&mut bytes, version, 0, b"ENDBLK")?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
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

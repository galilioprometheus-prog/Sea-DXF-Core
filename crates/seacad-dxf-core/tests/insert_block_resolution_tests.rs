use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInsertBlockResolutionDirectory,
    DxfInsertBlockResolutionEntry, DxfInsertBlockResolutionState, DxfInsertBlockTargetRange,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

const EXPECTED_STATES: [DxfInsertBlockResolutionState; 7] = [
    DxfInsertBlockResolutionState::Unique,
    DxfInsertBlockResolutionState::Ambiguous { target_count: 2 },
    DxfInsertBlockResolutionState::Missing,
    DxfInsertBlockResolutionState::UnusableName,
    DxfInsertBlockResolutionState::UnusableName,
    DxfInsertBlockResolutionState::Missing,
    DxfInsertBlockResolutionState::Unique,
];

#[test]
fn every_supported_dialect_has_exact_resolution_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_block_resolution_directory(&DxfCancellationToken::default())?;
        assert_directory(&ascii_directory)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_block_resolution_directory(&DxfCancellationToken::default())?;
        assert_directory(&binary_directory)?;
    }
    Ok(())
}

#[test]
fn long_same_source_names_resolve_without_name_sized_query_buffers() -> Result<(), Box<dyn Error>> {
    let name = vec![b'x'; 4 * 1024 + 1];
    let mut bytes =
        b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\n"
            .to_vec();
    bytes.extend_from_slice(&name);
    bytes.extend_from_slice(b"\n3\n");
    bytes.extend_from_slice(&name);
    bytes.extend_from_slice(b"\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\n");
    bytes.extend_from_slice(&name);
    bytes.extend_from_slice(b"\n0\nENDSEC\n0\nEOF\n");

    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_block_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 1);
    assert_eq!(
        directory.entries()[0].state(),
        DxfInsertBlockResolutionState::Unique
    );
    assert_eq!(directory.targets().len(), 1);
    Ok(())
}

#[test]
fn conflicting_block_names_never_become_resolution_targets() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nConflict\n3\nOther\n0\nENDBLK\n0\nBLOCK\n2\nDuplicate\n2\nDuplicate\n3\nDuplicate\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nConflict\n0\nINSERT\n2\nDuplicate\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_block_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfInsertBlockResolutionState::Missing,
            DxfInsertBlockResolutionState::Missing,
        ]
    );
    assert!(directory.targets().is_empty());
    Ok(())
}

#[test]
fn cancellation_lookups_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertBlockResolutionEntry>();
    assert_copy::<DxfInsertBlockTargetRange>();
    assert_send_sync::<DxfInsertBlockResolutionDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_block_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.insert_block_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.insert_semantic_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.block_name_index_directory().source_id()
    );
    assert_eq!(directory.entry_for_insert_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.targets_for_insert_raw_ordinal(u64::MAX), None);
    for entry in directory.entries() {
        let ordinal = entry.insert().record().ordinal();
        assert_eq!(
            directory.entry_for_insert_raw_ordinal(ordinal),
            Some(*entry)
        );
        assert_eq!(
            directory
                .targets_for_insert_raw_ordinal(ordinal)
                .ok_or_else(invalid_test_data)?
                .len() as u64,
            entry.target_range().len()
        );
    }
    Ok(())
}

fn assert_directory(directory: &DxfInsertBlockResolutionDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), EXPECTED_STATES.len());
    assert_eq!(directory.targets().len(), 4);
    for (entry, expected) in directory.entries().iter().zip(EXPECTED_STATES) {
        assert_eq!(entry.state(), expected);
        let targets = directory
            .targets_for_insert_raw_ordinal(entry.insert().record().ordinal())
            .ok_or_else(invalid_test_data)?;
        assert_eq!(targets.len() as u64, entry.target_range().len());
        match expected {
            DxfInsertBlockResolutionState::Unique => assert_eq!(targets.len(), 1),
            DxfInsertBlockResolutionState::Ambiguous { target_count } => {
                assert_eq!(targets.len(), target_count as usize);
                assert!(targets.windows(2).all(|pair| {
                    pair[0].record().definition().block_record().ordinal()
                        < pair[1].record().definition().block_record().ordinal()
                }));
            }
            DxfInsertBlockResolutionState::UnusableName
            | DxfInsertBlockResolutionState::Missing => assert!(targets.is_empty()),
            _ => return Err(io::Error::other("unknown resolution state").into()),
        }
    }
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n3\nA\n0\nENDBLK\n0\nBLOCK\n2\nB\n3\nB\n0\nENDBLK\n0\nBLOCK\n2\nB\n3\nB\n0\nENDBLK\n0\nBLOCK\n2\n\n3\n\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n0\nINSERT\n2\nB\n0\nINSERT\n2\nC\n0\nINSERT\n0\nINSERT\n2\nA\n2\nA\n0\nINSERT\n2\na\n0\nINSERT\n2\n\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section_start(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section_start(&mut bytes, version, b"BLOCKS")?;
    for name in [
        b"A".as_slice(),
        b"B".as_slice(),
        b"B".as_slice(),
        b"".as_slice(),
    ] {
        push_string(&mut bytes, version, 0, b"BLOCK")?;
        push_string(&mut bytes, version, 2, name)?;
        push_string(&mut bytes, version, 3, name)?;
        push_string(&mut bytes, version, 0, b"ENDBLK")?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section_start(&mut bytes, version, b"ENTITIES")?;
    for names in [
        vec![b"A".as_slice()],
        vec![b"B".as_slice()],
        vec![b"C".as_slice()],
        vec![],
        vec![b"A".as_slice(), b"A".as_slice()],
        vec![b"a".as_slice()],
        vec![b"".as_slice()],
    ] {
        push_string(&mut bytes, version, 0, b"INSERT")?;
        for name in names {
            push_string(&mut bytes, version, 2, name)?;
        }
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section_start(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

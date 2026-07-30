use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInsertAttributeDefinitionResolutionDirectory,
    DxfInsertAttributeDefinitionResolutionState as State, DxfInsertBlockResolutionState,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_ascii_binary_resolution_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = parity_ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii
            .insert_attribute_definition_resolution_directory(&DxfCancellationToken::default())?;

        let binary_bytes = parity_binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary
            .insert_attribute_definition_resolution_directory(&DxfCancellationToken::default())?;

        assert_eq!(states(&ascii_directory), states(&binary_directory));
        assert_eq!(states(&ascii_directory), [State::Unique]);
        assert_eq!(ascii_directory.definitions().len(), 1);
        assert_eq!(binary_directory.definitions().len(), 1);
    }
    Ok(())
}

#[test]
fn unique_target_exact_tag_resolution_preserves_every_fail_closed_state()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document
        .insert_attribute_definition_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 7);
    assert_eq!(
        states(&directory),
        [
            State::Ambiguous {
                definition_count: 2
            },
            State::Unique,
            State::Indeterminate {
                unusable_definition_count: 1
            },
            State::TagUnavailable,
            State::Missing,
            State::TargetUnavailable {
                resolution: DxfInsertBlockResolutionState::Missing,
            },
            State::TargetUnavailable {
                resolution: DxfInsertBlockResolutionState::Ambiguous { target_count: 2 },
            },
        ]
    );
    assert_eq!(directory.definitions().len(), 3);
    assert_eq!(directory.entries()[0].definition_range().len(), 2);
    assert_eq!(directory.entries()[1].definition_range().len(), 1);
    assert!(directory.entries()[5].target().is_none());
    assert!(directory.entries()[0].target().is_some());
    let first_raw = directory.entries()[0].attribute().record().ordinal();
    assert_eq!(
        directory
            .definitions_for_attribute_raw_ordinal(first_raw)
            .ok_or_else(invalid_test_data)?
            .len(),
        2
    );
    assert!(
        directory
            .entry_for_attribute_raw_ordinal(u64::MAX)
            .is_none()
    );
    Ok(())
}

#[test]
fn cancellation_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<seacad_dxf_core::DxfInsertAttributeDefinitionRange>();
    assert_copy::<seacad_dxf_core::DxfInsertAttributeDefinitionResolutionEntry>();
    assert_send_sync::<DxfInsertAttributeDefinitionResolutionDirectory>();
    let bytes = fixture();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_definition_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn states(directory: &DxfInsertAttributeDefinitionResolutionDirectory) -> Vec<State> {
    directory
        .entries()
        .iter()
        .map(|entry| entry.state())
        .collect()
}

fn fixture() -> Vec<u8> {
    b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n\
0\nBLOCK\n2\nU\n3\nU\n0\nATTDEF\n2\nA\n0\nATTDEF\n2\nA\n0\nATTDEF\n2\nB\n0\nATTDEF\n0\nENDBLK\n\
0\nBLOCK\n2\nZ\n3\nZ\n0\nENDBLK\n\
0\nBLOCK\n2\nX\n3\nX\n0\nATTDEF\n2\nA\n0\nENDBLK\n\
0\nBLOCK\n2\nX\n3\nX\n0\nATTDEF\n2\nA\n0\nENDBLK\n\
0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
0\nINSERT\n2\nU\n66\n1\n\
0\nATTRIB\n1\nv\n2\nA\n70\n0\n\
0\nATTRIB\n1\nv\n2\nB\n70\n0\n\
0\nATTRIB\n1\nv\n2\nQ\n70\n0\n\
0\nATTRIB\n1\nv\n70\n0\n0\nSEQEND\n\
0\nINSERT\n2\nZ\n66\n1\n0\nATTRIB\n1\nv\n2\nQ\n70\n0\n0\nSEQEND\n\
0\nINSERT\n2\nNOPE\n66\n1\n0\nATTRIB\n1\nv\n2\nA\n70\n0\n0\nSEQEND\n\
0\nINSERT\n2\nX\n66\n1\n0\nATTRIB\n1\nv\n2\nA\n70\n0\n0\nSEQEND\n\
0\nENDSEC\n0\nEOF\n"
        .to_vec()
}

fn parity_ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nU\n3\nU\n0\nATTDEF\n2\nA\n0\nENDBLK\n\
0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nU\n66\n1\n\
0\nATTRIB\n1\nv\n2\nA\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn parity_binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 2, b"U")?;
    push_string(&mut bytes, version, 3, b"U")?;
    push_string(&mut bytes, version, 0, b"ATTDEF")?;
    push_string(&mut bytes, version, 2, b"A")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"U")?;
    push_i16(&mut bytes, version, 66, 1)?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 1, b"v")?;
    push_string(&mut bytes, version, 2, b"A")?;
    push_i16(&mut bytes, version, 70, 0)?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

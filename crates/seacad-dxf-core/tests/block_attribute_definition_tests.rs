use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockAttributeDefinitionDirectory, DxfBlockAttributeDefinitionEntry,
    DxfBlockDefinitionState, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type EntrySummary = (u64, u64, DxfBlockDefinitionState);

#[test]
fn every_supported_dialect_has_ascii_binary_attdef_topology_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.block_attribute_definition_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.block_attribute_definition_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory);
        assert_directory(&binary_directory);
        assert_eq!(summaries(&ascii_directory), summaries(&binary_directory));
    }
    Ok(())
}

#[test]
fn exact_case_scope_and_parent_boundaries_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nATTDEF\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nATTDEF\n0\nBLOCK\n2\nA\n0\nattdef\n0\nATTDEF \n0\nATTDEF\n2\nA1\n0\nBLOCK\n2\nB\n0\nATTDEF\n2\nB1\n0\nENDBLK\n0\nBLOCK\n2\nC\n0\nLINE\n0\nATTDEF\n2\nC1\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.block_attribute_definition_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.entries().len(), 3);
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| (
                entry.member_ordinal(),
                entry.attribute_definition_ordinal(),
                entry.owner().state(),
            ))
            .collect::<Vec<_>>(),
        [
            (2, 0, DxfBlockDefinitionState::Interrupted),
            (0, 0, DxfBlockDefinitionState::Closed),
            (1, 0, DxfBlockDefinitionState::Unclosed),
        ]
    );
    assert_eq!(
        directory
            .entries_for_block_raw_ordinal(directory.entries()[0].owner().block_record().ordinal()),
        &directory.entries()[0..1]
    );
    assert!(directory.entries_for_block_raw_ordinal(u64::MAX).is_empty());
    Ok(())
}

#[test]
fn incomplete_blocks_sections_publish_no_attdef_evidence() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n0\nATTDEF\n2\nA1\n0\nSECTION\n2\nENTITIES\n0\nLINE\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.block_attribute_definition_directory(&DxfCancellationToken::default())?;

    assert!(directory.entries().is_empty());
    assert!(
        directory
            .block_definition_directory()
            .definitions()
            .is_empty()
    );
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfBlockAttributeDefinitionEntry>();
    assert_send_sync::<DxfBlockAttributeDefinitionDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.block_attribute_definition_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.block_attribute_definition_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.block_definition_directory().source_id()
    );
    assert_eq!(directory.entry_for_attdef_raw_ordinal(u64::MAX), None);
    for entry in directory.entries().iter().copied() {
        assert_eq!(
            directory.entry_for_attdef_raw_ordinal(entry.record().ordinal()),
            Some(entry)
        );
    }
    Ok(())
}

fn assert_directory(directory: &DxfBlockAttributeDefinitionDirectory) {
    assert_eq!(directory.entries().len(), 3);
    assert_eq!(
        summaries(directory),
        [
            (0, 0, DxfBlockDefinitionState::Closed),
            (2, 1, DxfBlockDefinitionState::Closed),
            (0, 0, DxfBlockDefinitionState::Closed),
        ]
    );
    let first_owner = directory.entries()[0].owner().block_record().ordinal();
    assert_eq!(
        directory.entries_for_block_raw_ordinal(first_owner),
        &directory.entries()[0..2]
    );
}

fn summaries(directory: &DxfBlockAttributeDefinitionDirectory) -> Vec<EntrySummary> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.member_ordinal(),
                entry.attribute_definition_ordinal(),
                entry.owner().state(),
            )
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n0\nATTDEF\n2\nA1\n0\nLINE\n0\nATTDEF\n2\nA2\n0\nENDBLK\n0\nBLOCK\n2\nB\n0\nATTDEF\n2\nB1\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nATTDEF\n2\nORPHAN\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_block(&mut bytes, version, b"A", &[b"ATTDEF", b"LINE", b"ATTDEF"])?;
    push_block(&mut bytes, version, b"B", &[b"ATTDEF"])?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"ATTDEF")?;
    push_string(&mut bytes, version, 2, b"ORPHAN")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_block(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
    members: &[&[u8]],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"BLOCK")?;
    push_string(bytes, version, 2, name)?;
    for (index, marker) in members.iter().enumerate() {
        push_string(bytes, version, 0, marker)?;
        if *marker == b"ATTDEF" {
            let tag = format!("{name:?}{index}");
            push_string(bytes, version, 2, tag.as_bytes())?;
        }
    }
    push_string(bytes, version, 0, b"ENDBLK")
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

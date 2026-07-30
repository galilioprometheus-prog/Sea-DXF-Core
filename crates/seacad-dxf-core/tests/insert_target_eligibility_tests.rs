use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockDefinitionState, DxfBlockExpansionEdge, DxfByteSource, DxfCancellationToken, DxfError,
    DxfInsertTargetEligibilityDirectory, DxfInsertTargetEligibilityEntry,
    DxfInsertTargetEligibilityState, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

const PARITY_STATES: [DxfInsertTargetEligibilityState; 5] = [
    DxfInsertTargetEligibilityState::RecursiveExpansion,
    DxfInsertTargetEligibilityState::Eligible,
    DxfInsertTargetEligibilityState::RecursiveExpansion,
    DxfInsertTargetEligibilityState::TargetDefinitionNotClosed {
        state: DxfBlockDefinitionState::Unclosed,
    },
    DxfInsertTargetEligibilityState::NotUniquelyResolved,
];

#[test]
fn every_supported_dialect_has_closed_target_and_recursion_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = parity_ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_target_eligibility_directory(&DxfCancellationToken::default())?;
        assert_parity_directory(&ascii_directory)?;

        let binary_bytes = parity_binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_target_eligibility_directory(&DxfCancellationToken::default())?;
        assert_parity_directory(&binary_directory)?;
    }
    Ok(())
}

#[test]
fn indirect_deep_cycles_and_acyclic_chains_are_distinguished() -> Result<(), Box<dyn Error>> {
    let bytes = complex_ascii_fixture();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_target_eligibility_directory(&DxfCancellationToken::default())?;

    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfInsertTargetEligibilityState::RecursiveExpansion,
            DxfInsertTargetEligibilityState::RecursiveExpansion,
            DxfInsertTargetEligibilityState::RecursiveExpansion,
            DxfInsertTargetEligibilityState::Eligible,
            DxfInsertTargetEligibilityState::RecursiveExpansion,
            DxfInsertTargetEligibilityState::RecursiveExpansion,
            DxfInsertTargetEligibilityState::Eligible,
            DxfInsertTargetEligibilityState::TargetDefinitionNotClosed {
                state: DxfBlockDefinitionState::Interrupted,
            },
            DxfInsertTargetEligibilityState::NotUniquelyResolved,
            DxfInsertTargetEligibilityState::NotUniquelyResolved,
        ]
    );
    assert_eq!(directory.expansion_edges().len(), 4);

    let edges = directory.expansion_edges();
    assert!(edges.iter().all(|edge| {
        edge.owner().block_record().ordinal()
            != edge.target().record().definition().block_record().ordinal()
    }));
    for edge in edges {
        assert_eq!(
            edge.insert().insert().record().section_kind(),
            seacad_dxf_core::DxfRawRecordSectionKind::Blocks
        );
    }
    Ok(())
}

#[test]
fn cancellation_lookups_graph_ranges_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertTargetEligibilityEntry>();
    assert_copy::<DxfBlockExpansionEdge>();
    assert_send_sync::<DxfInsertTargetEligibilityDirectory>();

    let bytes = parity_ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_target_eligibility_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.insert_target_eligibility_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.resolution_directory().source_id()
    );
    assert_eq!(directory.entry_for_insert_raw_ordinal(u64::MAX), None);
    assert_eq!(
        directory.expansion_edges_for_block_raw_ordinal(u64::MAX),
        None
    );
    for entry in directory.entries() {
        let ordinal = entry.resolution().insert().record().ordinal();
        assert_eq!(
            directory.entry_for_insert_raw_ordinal(ordinal),
            Some(*entry)
        );
    }
    for edge in directory.expansion_edges() {
        let owner_raw = edge.owner().block_record().ordinal();
        assert!(
            directory
                .expansion_edges_for_block_raw_ordinal(owner_raw)
                .ok_or_else(invalid_test_data)?
                .contains(edge)
        );
    }
    Ok(())
}

fn assert_parity_directory(
    directory: &DxfInsertTargetEligibilityDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), PARITY_STATES.len());
    assert_eq!(directory.expansion_edges().len(), 1);
    for (entry, expected) in directory.entries().iter().zip(PARITY_STATES) {
        assert_eq!(entry.state(), expected);
    }
    let edge = directory.expansion_edges()[0];
    assert_eq!(
        edge.owner().block_record().ordinal(),
        edge.target().record().definition().block_record().ordinal()
    );
    let owner_edges = directory
        .expansion_edges_for_block_raw_ordinal(edge.owner().block_record().ordinal())
        .ok_or_else(invalid_test_data)?;
    assert_eq!(owner_edges, [edge]);
    Ok(())
}

fn parity_ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n3\nA\n0\nENDBLK\n0\nBLOCK\n2\nSelf\n3\nSelf\n0\nINSERT\n2\nSelf\n0\nENDBLK\n0\nBLOCK\n2\nF\n3\nF\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n0\nINSERT\n2\nSelf\n0\nINSERT\n2\nF\n0\nINSERT\n2\nMissing\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn parity_binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section_start(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section_start(&mut bytes, version, b"BLOCKS")?;
    push_block_start(&mut bytes, version, b"A")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_block_start(&mut bytes, version, b"Self")?;
    push_insert(&mut bytes, version, Some(b"Self"))?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_block_start(&mut bytes, version, b"F")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section_start(&mut bytes, version, b"ENTITIES")?;
    for name in [b"A".as_slice(), b"Self", b"F", b"Missing"] {
        push_insert(&mut bytes, version, Some(name))?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn complex_ascii_fixture() -> Vec<u8> {
    b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n3\nA\n0\nENDBLK\n0\nBLOCK\n2\nB\n3\nB\n0\nINSERT\n2\nC\n0\nENDBLK\n0\nBLOCK\n2\nC\n3\nC\n0\nINSERT\n2\nB\n0\nENDBLK\n0\nBLOCK\n2\nD\n3\nD\n0\nINSERT\n2\nB\n0\nENDBLK\n0\nBLOCK\n2\nH\n3\nH\n0\nINSERT\n2\nA\n0\nENDBLK\n0\nBLOCK\n2\nF\n3\nF\n0\nBLOCK\n2\nG\n3\nG\n0\nENDBLK\n0\nBLOCK\n2\nX\n3\nX\n0\nENDBLK\n0\nBLOCK\n2\nX\n3\nX\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n0\nINSERT\n2\nD\n0\nINSERT\n2\nH\n0\nINSERT\n2\nF\n0\nINSERT\n2\nX\n0\nINSERT\n0\nENDSEC\n0\nEOF\n".to_vec()
}

fn push_section_start(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_block_start(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"BLOCK")?;
    push_string(bytes, version, 2, name)?;
    push_string(bytes, version, 3, name)
}

fn push_insert(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: Option<&[u8]>,
) -> io::Result<()> {
    push_string(bytes, version, 0, b"INSERT")?;
    if let Some(name) = name {
        push_string(bytes, version, 2, name)?;
    }
    Ok(())
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

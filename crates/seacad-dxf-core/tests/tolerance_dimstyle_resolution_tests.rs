use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDimStyleTableDirectory, DxfError, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, DxfToleranceDimStyleResolutionDirectory,
    DxfToleranceDimStyleResolutionEntry, DxfToleranceDimStyleResolutionState, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    state: DxfToleranceDimStyleResolutionState,
    target_count: u64,
}

#[test]
fn every_dialect_has_ascii_binary_exact_resolution_parity() -> Result<(), Box<dyn Error>> {
    let expected = vec![
        signature(DxfToleranceDimStyleResolutionState::Unique, 1),
        signature(DxfToleranceDimStyleResolutionState::Missing, 0),
        signature(
            DxfToleranceDimStyleResolutionState::Ambiguous { target_count: 2 },
            2,
        ),
        signature(DxfToleranceDimStyleResolutionState::UnusableName, 0),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        let ascii_directory = ascii.tolerance_dimstyle_resolution_directory(&cancellation)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.tolerance_dimstyle_resolution_directory(&cancellation)?;

        assert_eq!(signatures(&ascii_directory), expected);
        assert_eq!(signatures(&binary_directory), expected);
        assert_eq!(
            ascii_directory.dimstyle_table_directory().entries().len(),
            3
        );
        assert_eq!(
            binary_directory.dimstyle_table_directory().entries().len(),
            3
        );
    }
    Ok(())
}

#[test]
fn only_complete_exact_table_envelopes_and_unique_names_are_admitted() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n\
0\nTABLE\n2\nDIMSTYLE\n0\nDIMSTYLE\n2\nClosed\n0\nENDTAB\n\
0\nTABLE\n2\nDIMSTYLE\n0\nDIMSTYLE\n2\nInterrupted\n0\nTABLE\n2\nLAYER\n0\nENDTAB\n\
0\nTABLE\n2\ndimstyle\n0\nDIMSTYLE\n2\nWrongCaseTable\n0\nENDTAB\n\
0\nTABLE\n2\nDIMSTYLE\n0\ndimstyle\n2\nWrongCaseRecord\n\
0\nDIMSTYLE\n2\nFirst\n2\nSecond\n\
0\nDIMSTYLE\n102\n{APP\n2\nHidden\n102\n}\n2\nVisible\n0\nENDTAB\n\
0\nTABLE\n2\nDIMSTYLE\n0\nDIMSTYLE\n2\nUnclosed\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.dimstyle_table_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    assert_name(&document, directory.entries()[0].name(), b"Closed")?;
    assert_name(&document, directory.entries()[1].name(), b"Visible")?;
    assert_eq!(
        directory.source_id(),
        directory.raw_record_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.application_group_directory().source_id()
    );
    Ok(())
}

#[test]
fn names_are_byte_exact_case_sensitive_and_duplicate_preserving() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nDIMSTYLE\n\
0\nDIMSTYLE\n2\nCase\n0\nDIMSTYLE\n2\nCase\n0\nDIMSTYLE\n2\ncase\n0\nENDTAB\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTOLERANCE\n3\nCase\n1\nA\n0\nTOLERANCE\n3\ncase\n1\nB\n\
0\nTOLERANCE\n3\nCASE\n1\nC\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.tolerance_dimstyle_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.entries()[0].state(),
        DxfToleranceDimStyleResolutionState::Ambiguous { target_count: 2 }
    );
    assert_eq!(
        directory.entries()[1].state(),
        DxfToleranceDimStyleResolutionState::Unique
    );
    assert_eq!(
        directory.entries()[2].state(),
        DxfToleranceDimStyleResolutionState::Missing
    );
    let first = directory.entries()[0].tolerance().record().ordinal();
    assert_eq!(
        directory
            .targets_for_tolerance_raw_ordinal(first)
            .ok_or_else(invalid_test_data)?
            .len(),
        2
    );
    assert_eq!(directory.targets_for_tolerance_raw_ordinal(u64::MAX), None);
    Ok(())
}

#[test]
fn cancellation_scope_lookup_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfToleranceDimStyleResolutionEntry>();
    assert_send_sync::<DxfToleranceDimStyleResolutionDirectory>();
    assert_send_sync::<DxfDimStyleTableDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.tolerance_dimstyle_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        document.dimstyle_table_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.tolerance_dimstyle_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.field_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.dimstyle_table_directory().source_id()
    );
    for entry in directory.entries().iter().copied() {
        assert_eq!(
            directory.entry_for_tolerance_raw_ordinal(entry.tolerance().record().ordinal()),
            Some(entry)
        );
    }
    assert_eq!(directory.entry_for_tolerance_raw_ordinal(u64::MAX), None);
    Ok(())
}

fn signatures(directory: &DxfToleranceDimStyleResolutionDirectory) -> Vec<Signature> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| Signature {
            state: entry.state(),
            target_count: entry.target_range().len(),
        })
        .collect()
}

fn signature(state: DxfToleranceDimStyleResolutionState, target_count: u64) -> Signature {
    Signature {
        state,
        target_count,
    }
}

fn assert_name(
    document: &DxfAsciiRawDocument<'_>,
    name: seacad_dxf_core::DxfRawValueProvenance,
    expected: &[u8],
) -> Result<(), DxfError> {
    let mut actual = vec![0_u8; expected.len()];
    document.read_span(name.value_span(), &mut actual)?;
    assert_eq!(actual, expected);
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nDIMSTYLE\n\
0\nDIMSTYLE\n2\nExact\n0\nDIMSTYLE\n2\nDuplicate\n0\nDIMSTYLE\n2\nDuplicate\n\
0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
0\nTOLERANCE\n3\nExact\n1\nA\n0\nTOLERANCE\n3\nMissing\n1\nB\n\
0\nTOLERANCE\n3\nDuplicate\n1\nC\n0\nTOLERANCE\n1\nD\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"TABLES")?;
    push_string(&mut bytes, version, 0, b"TABLE")?;
    push_string(&mut bytes, version, 2, b"DIMSTYLE")?;
    for name in [b"Exact".as_slice(), b"Duplicate", b"Duplicate"] {
        push_string(&mut bytes, version, 0, b"DIMSTYLE")?;
        push_string(&mut bytes, version, 2, name)?;
    }
    push_string(&mut bytes, version, 0, b"ENDTAB")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    for name in [
        Some(b"Exact".as_slice()),
        Some(b"Missing"),
        Some(b"Duplicate"),
        None,
    ] {
        push_string(&mut bytes, version, 0, b"TOLERANCE")?;
        if let Some(name) = name {
            push_string(&mut bytes, version, 3, name)?;
        }
        push_string(&mut bytes, version, 1, b"A")?;
    }
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDimStyleHandleResolutionDirectory, DxfDimStyleHandleResolutionEntry,
    DxfDimStyleHandleRole, DxfDimStyleHandleTargetState, DxfError, DxfHandleParseIssue,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_handle_resolution_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.dimstyle_handle_resolution_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.dimstyle_handle_resolution_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory, version)?;
        assert_directory(&binary_directory, version)?;
        assert_eq!(states(&ascii_directory), states(&binary_directory));
    }
    Ok(())
}

#[test]
fn cancellation_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfDimStyleHandleResolutionEntry>();
    assert_copy::<DxfDimStyleHandleRole>();
    assert_copy::<DxfDimStyleHandleTargetState>();
    assert_send_sync::<DxfDimStyleHandleResolutionDirectory>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.dimstyle_handle_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.dimstyle_handle_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.handle_resolution_directory().source_id()
    );
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_raw_ordinal(u64::MAX), None);
    Ok(())
}

fn assert_directory(
    directory: &DxfDimStyleHandleResolutionDirectory,
    version: DxfAcadVersion,
) -> Result<(), DxfError> {
    assert_eq!(directory.entries().len(), 10);
    let expected = if version == DxfAcadVersion::Ac1009 {
        vec![DxfDimStyleHandleTargetState::Absent; 10]
    } else {
        vec![
            DxfDimStyleHandleTargetState::Unique,
            DxfDimStyleHandleTargetState::Missing,
            DxfDimStyleHandleTargetState::Null,
            DxfDimStyleHandleTargetState::Invalid(DxfHandleParseIssue::InvalidDigit { offset: 1 }),
            DxfDimStyleHandleTargetState::Ambiguous { target_count: 2 },
            DxfDimStyleHandleTargetState::MultipleValues {
                occurrence_count: 2,
            },
            DxfDimStyleHandleTargetState::Absent,
            DxfDimStyleHandleTargetState::Absent,
            DxfDimStyleHandleTargetState::Absent,
            DxfDimStyleHandleTargetState::Absent,
        ]
    };
    assert_eq!(states(directory), expected);

    let first_raw = directory.entries()[0]
        .record()
        .table_entry()
        .record()
        .ordinal();
    let entries = directory
        .entries_for_raw_ordinal(first_raw)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(entries.len(), 5);
    assert_eq!(
        entries.iter().map(|entry| entry.role()).collect::<Vec<_>>(),
        [
            DxfDimStyleHandleRole::TextStyle,
            DxfDimStyleHandleRole::LeaderArrowBlock,
            DxfDimStyleHandleRole::CommonArrowBlock,
            DxfDimStyleHandleRole::FirstArrowBlock,
            DxfDimStyleHandleRole::SecondArrowBlock,
        ]
    );
    for entry in directory.entries().iter().copied() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let target_count = directory
            .targets_for_entry(entry)
            .ok_or_else(invalid_test_data)?
            .len();
        let expected_count = match entry.state() {
            DxfDimStyleHandleTargetState::Unique => 1,
            DxfDimStyleHandleTargetState::Ambiguous { target_count } => target_count as usize,
            DxfDimStyleHandleTargetState::Absent
            | DxfDimStyleHandleTargetState::MultipleValues { .. }
            | DxfDimStyleHandleTargetState::Invalid(_)
            | DxfDimStyleHandleTargetState::Null
            | DxfDimStyleHandleTargetState::Missing => 0,
            _ => return Err(invalid_test_data()),
        };
        assert_eq!(target_count, expected_count);
    }
    Ok(())
}

fn states(directory: &DxfDimStyleHandleResolutionDirectory) -> Vec<DxfDimStyleHandleTargetState> {
    directory
        .entries()
        .iter()
        .map(|entry| entry.state())
        .collect()
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let handles = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "340\nA\n341\nF\n342\n0\n343\n1G\n344\nC\n".to_owned()
    };
    let duplicate = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "340\nA\n102\n{APP\n340\nC\n102\n}\n340\nB\n".to_owned()
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n\
0\nTABLE\n2\nSTYLE\n0\nSTYLE\n2\nS\n5\nA\n0\nENDTAB\n\
0\nTABLE\n2\nBLOCK_RECORD\n0\nBLOCK_RECORD\n2\nB\n5\nB\n0\nBLOCK_RECORD\n2\nC1\n5\nC\n0\nBLOCK_RECORD\n2\nC2\n5\nC\n0\nENDTAB\n\
0\nTABLE\n2\nDIMSTYLE\n0\nDIMSTYLE\n2\nFull\n{handles}0\nDIMSTYLE\n2\nMultiple\n{duplicate}0\nENDTAB\n0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_header(&mut bytes, version)?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"TABLES")?;
    push_table_start(&mut bytes, version, b"STYLE")?;
    push_named_identity(&mut bytes, version, b"STYLE", b"S", b"A")?;
    push_string(&mut bytes, version, 0, b"ENDTAB")?;
    push_table_start(&mut bytes, version, b"BLOCK_RECORD")?;
    for (name, handle) in [
        (b"B".as_slice(), b"B".as_slice()),
        (b"C1", b"C"),
        (b"C2", b"C"),
    ] {
        push_named_identity(&mut bytes, version, b"BLOCK_RECORD", name, handle)?;
    }
    push_string(&mut bytes, version, 0, b"ENDTAB")?;
    push_table_start(&mut bytes, version, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 0, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 2, b"Full")?;
    if version != DxfAcadVersion::Ac1009 {
        for (code, value) in [
            (340, b"A".as_slice()),
            (341, b"F"),
            (342, b"0"),
            (343, b"1G"),
            (344, b"C"),
        ] {
            push_string(&mut bytes, version, code, value)?;
        }
    }
    push_string(&mut bytes, version, 0, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 2, b"Multiple")?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 340, b"A")?;
        push_string(&mut bytes, version, 102, b"{APP")?;
        push_string(&mut bytes, version, 340, b"C")?;
        push_string(&mut bytes, version, 102, b"}")?;
        push_string(&mut bytes, version, 340, b"B")?;
    }
    push_string(&mut bytes, version, 0, b"ENDTAB")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_header(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, b"HEADER")?;
    push_string(bytes, version, 9, b"$ACADVER")?;
    push_string(bytes, version, 1, version.code().as_bytes())?;
    push_string(bytes, version, 0, b"ENDSEC")
}

fn push_table_start(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"TABLE")?;
    push_string(bytes, version, 2, name)
}

fn push_named_identity(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    kind: &[u8],
    name: &[u8],
    handle: &[u8],
) -> io::Result<()> {
    push_string(bytes, version, 0, kind)?;
    push_string(bytes, version, 2, name)?;
    push_string(bytes, version, 5, handle)
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDimStyleHandleResolutionDirectory, DxfDimStyleHandleResolutionEntry,
    DxfDimStyleHandleRole, DxfDimStyleHandleTargetState,
    DxfDimStyleHandleTargetValidationDirectory, DxfDimStyleHandleTargetValidationEntry,
    DxfDimStyleHandleTargetValidationState, DxfError, DxfHandleParseIssue, DxfMemorySource,
    DxfNamedSymbolTableDirectory, DxfNamedSymbolTableEntry, DxfNamedSymbolTableKind,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_handle_resolution_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.dimstyle_handle_resolution_directory(&DxfCancellationToken::default())?;
        let ascii_validation =
            ascii.dimstyle_handle_target_validation_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.dimstyle_handle_resolution_directory(&DxfCancellationToken::default())?;
        let binary_validation =
            binary.dimstyle_handle_target_validation_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory, version)?;
        assert_directory(&binary_directory, version)?;
        assert_validation_directory(&ascii_validation, version)?;
        assert_validation_directory(&binary_validation, version)?;
        assert_eq!(states(&ascii_directory), states(&binary_directory));
        assert_eq!(
            validation_states(&ascii_validation),
            validation_states(&binary_validation)
        );
    }
    Ok(())
}

#[test]
fn cancellation_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfDimStyleHandleResolutionEntry>();
    assert_copy::<DxfDimStyleHandleRole>();
    assert_copy::<DxfDimStyleHandleTargetState>();
    assert_copy::<DxfDimStyleHandleTargetValidationEntry>();
    assert_copy::<DxfDimStyleHandleTargetValidationState>();
    assert_copy::<DxfNamedSymbolTableEntry>();
    assert_copy::<DxfNamedSymbolTableKind>();
    assert_send_sync::<DxfDimStyleHandleResolutionDirectory>();
    assert_send_sync::<DxfDimStyleHandleTargetValidationDirectory>();
    assert_send_sync::<DxfNamedSymbolTableDirectory>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.dimstyle_handle_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        document.dimstyle_handle_target_validation_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        document.named_symbol_table_directory(&cancellation),
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
    let validation =
        document.dimstyle_handle_target_validation_directory(&DxfCancellationToken::default())?;
    assert_eq!(validation.entry(u64::MAX), None);
    assert_eq!(validation.entries_for_raw_ordinal(u64::MAX), None);
    assert_eq!(
        validation.entry_for_role(u64::MAX, DxfDimStyleHandleRole::TextStyle),
        None
    );
    assert_eq!(
        validation
            .named_symbol_table_directory()
            .entry_for_raw_ordinal(u64::MAX),
        None
    );
    Ok(())
}

#[test]
fn named_membership_requires_exact_closed_reviewed_tables() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n\
0\nTABLE\n2\nSTYLE\n0\nSTYLE\n2\nS\n5\nA\n0\nENDTAB\n\
0\nTABLE\n2\nBLOCK_RECORD\n0\nBLOCK_RECORD\n2\nB\n5\nB\n0\nENDTAB\n\
0\nTABLE\n2\nstyle\n0\nSTYLE\n2\nLower\n5\nC\n0\nENDTAB\n\
0\nTABLE\n2\nSTYLE\n0\nBLOCK_RECORD\n2\nWrong\n5\nD\n\
0\nSTYLE\n2\nDuplicate\n2\nName\n5\nE\n\
0\nSTYLE\n102\n{APP\n2\nDecoy\n102\n}\n2\nVisible\n5\nF\n0\nENDTAB\n\
0\nTABLE\n2\nSTYLE\n0\nSTYLE\n2\nUnclosed\n5\n10\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.named_symbol_table_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.kind())
            .collect::<Vec<_>>(),
        [
            DxfNamedSymbolTableKind::Style,
            DxfNamedSymbolTableKind::BlockRecord,
            DxfNamedSymbolTableKind::Style,
        ]
    );
    for entry in directory.entries().iter().copied() {
        assert_eq!(
            directory.entry_for_raw_ordinal(entry.record().ordinal()),
            Some(entry)
        );
    }
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
            DxfDimStyleHandleTargetState::Unique,
            DxfDimStyleHandleTargetState::Unique,
            DxfDimStyleHandleTargetState::Unique,
            DxfDimStyleHandleTargetState::Ambiguous { target_count: 2 },
            DxfDimStyleHandleTargetState::Missing,
            DxfDimStyleHandleTargetState::Null,
            DxfDimStyleHandleTargetState::Invalid(DxfHandleParseIssue::InvalidDigit { offset: 1 }),
            DxfDimStyleHandleTargetState::MultipleValues {
                occurrence_count: 2,
            },
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

fn assert_validation_directory(
    directory: &DxfDimStyleHandleTargetValidationDirectory,
    version: DxfAcadVersion,
) -> Result<(), DxfError> {
    assert_eq!(directory.entries().len(), 10);
    let expected = if version == DxfAcadVersion::Ac1009 {
        vec![DxfDimStyleHandleTargetValidationState::Absent; 10]
    } else {
        vec![
            DxfDimStyleHandleTargetValidationState::UniqueExpected,
            DxfDimStyleHandleTargetValidationState::UniqueExpected,
            DxfDimStyleHandleTargetValidationState::UniqueOtherNamedSymbol {
                observed: DxfNamedSymbolTableKind::Style,
            },
            DxfDimStyleHandleTargetValidationState::UniqueOtherRecord,
            DxfDimStyleHandleTargetValidationState::Ambiguous { target_count: 2 },
            DxfDimStyleHandleTargetValidationState::Missing,
            DxfDimStyleHandleTargetValidationState::Null,
            DxfDimStyleHandleTargetValidationState::Invalid(DxfHandleParseIssue::InvalidDigit {
                offset: 1,
            }),
            DxfDimStyleHandleTargetValidationState::MultipleValues {
                occurrence_count: 2,
            },
            DxfDimStyleHandleTargetValidationState::Absent,
        ]
    };
    assert_eq!(validation_states(directory), expected);
    assert_eq!(
        directory.resolution_directory().source_id(),
        directory.source_id()
    );
    assert_eq!(
        directory.named_symbol_table_directory().source_id(),
        directory.source_id()
    );
    assert_eq!(directory.named_symbol_table_directory().entries().len(), 6);

    let first_raw = directory.entries()[0]
        .resolution()
        .record()
        .table_entry()
        .record()
        .ordinal();
    assert_eq!(
        directory
            .entries_for_raw_ordinal(first_raw)
            .ok_or_else(invalid_test_data)?
            .len(),
        5
    );
    for entry in directory.entries().iter().copied() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
    }
    if version != DxfAcadVersion::Ac1009 {
        let entries = directory.entries();
        assert!(entries[0].target().is_some());
        assert_eq!(
            entries[0]
                .named_target()
                .map(DxfNamedSymbolTableEntry::kind),
            Some(DxfNamedSymbolTableKind::Style)
        );
        assert_eq!(
            entries[2]
                .named_target()
                .map(DxfNamedSymbolTableEntry::kind),
            Some(DxfNamedSymbolTableKind::Style)
        );
        assert!(entries[3].target().is_some());
        assert_eq!(entries[3].named_target(), None);
        assert_eq!(entries[4].target(), None);
        assert_eq!(
            directory.entry_for_role(first_raw, DxfDimStyleHandleRole::SecondArrowBlock),
            Some(entries[4])
        );
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

fn validation_states(
    directory: &DxfDimStyleHandleTargetValidationDirectory,
) -> Vec<DxfDimStyleHandleTargetValidationState> {
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
        "340\nA\n341\nB\n342\nA\n343\nE\n344\nC\n".to_owned()
    };
    let duplicate = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "340\nF\n341\n0\n342\n1G\n343\nA\n102\n{APP\n343\nC\n102\n}\n343\nB\n".to_owned()
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n\
0\nTABLE\n2\nSTYLE\n0\nSTYLE\n2\nS\n5\nA\n0\nENDTAB\n\
0\nTABLE\n2\nBLOCK_RECORD\n0\nBLOCK_RECORD\n2\nB\n5\nB\n0\nBLOCK_RECORD\n2\nC1\n5\nC\n0\nBLOCK_RECORD\n2\nC2\n5\nC\n0\nENDTAB\n\
0\nTABLE\n2\nLAYER\n5\nE\n0\nENDTAB\n\
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
    push_table_start(&mut bytes, version, b"LAYER")?;
    push_string(&mut bytes, version, 5, b"E")?;
    push_string(&mut bytes, version, 0, b"ENDTAB")?;
    push_table_start(&mut bytes, version, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 0, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 2, b"Full")?;
    if version != DxfAcadVersion::Ac1009 {
        for (code, value) in [
            (340, b"A".as_slice()),
            (341, b"B"),
            (342, b"A"),
            (343, b"E"),
            (344, b"C"),
        ] {
            push_string(&mut bytes, version, code, value)?;
        }
    }
    push_string(&mut bytes, version, 0, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 2, b"Multiple")?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 340, b"F")?;
        push_string(&mut bytes, version, 341, b"0")?;
        push_string(&mut bytes, version, 342, b"1G")?;
        push_string(&mut bytes, version, 343, b"A")?;
        push_string(&mut bytes, version, 102, b"{APP")?;
        push_string(&mut bytes, version, 343, b"C")?;
        push_string(&mut bytes, version, 102, b"}")?;
        push_string(&mut bytes, version, 343, b"B")?;
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

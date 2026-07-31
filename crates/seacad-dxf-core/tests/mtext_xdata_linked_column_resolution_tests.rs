use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandleParseIssue,
    DxfMTextXDataLinkedColumnResolutionDirectory, DxfMTextXDataLinkedColumnResolutionEntry,
    DxfMTextXDataLinkedColumnTarget, DxfMTextXDataLinkedColumnTargetState, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_unique_mtext_resolution_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_parity_document(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        assert_parity(
            &ascii
                .mtext_xdata_linked_column_resolution_directory(&DxfCancellationToken::default())?,
        )?;

        let binary_bytes = binary_parity_document(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_parity(
            &binary
                .mtext_xdata_linked_column_resolution_directory(&DxfCancellationToken::default())?,
        )?;
    }
    Ok(())
}

#[test]
fn every_generic_resolution_state_and_target_kind_remain_distinct() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_all_states_document();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.mtext_xdata_linked_column_resolution_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entries()
        .first()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert!(entry.column_info().is_some());
    assert_eq!(
        directory
            .targets_for_entry(entry)
            .ok_or_else(invalid_test_data)?
            .iter()
            .map(|target| target.state())
            .collect::<Vec<_>>(),
        [
            DxfMTextXDataLinkedColumnTargetState::UniqueMText,
            DxfMTextXDataLinkedColumnTargetState::UniqueOtherRecord,
            DxfMTextXDataLinkedColumnTargetState::Missing,
            DxfMTextXDataLinkedColumnTargetState::Null,
            DxfMTextXDataLinkedColumnTargetState::Invalid(DxfHandleParseIssue::InvalidDigit {
                offset: 0
            }),
            DxfMTextXDataLinkedColumnTargetState::Ambiguous { target_count: 2 },
        ]
    );
    assert!(directory.targets()[0].target_record().is_some());
    assert!(directory.targets()[1].target_record().is_some());
    assert!(
        directory.targets()[2..]
            .iter()
            .all(|target| target.target_record().is_none())
    );
    Ok(())
}

#[test]
fn absent_info_cancellation_identity_and_public_traits_remain_explicit()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextXDataLinkedColumnResolutionEntry>();
    assert_copy::<DxfMTextXDataLinkedColumnTarget>();
    assert_send_sync::<DxfMTextXDataLinkedColumnResolutionDirectory>();

    let bytes = ascii_without_info();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_xdata_linked_column_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.mtext_xdata_linked_column_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(directory.entries().len(), 1);
    assert_eq!(directory.entries()[0].column_info(), None);
    assert_eq!(directory.entries()[0].target_count(), 1);
    assert_eq!(
        directory.targets()[0].state(),
        DxfMTextXDataLinkedColumnTargetState::Missing
    );
    Ok(())
}

const APP: &[u8] = b"ACAD";
const INFO_BEGIN: &[u8] = b"ACAD_MTEXT_COLUMN_INFO_BEGIN";
const INFO_END: &[u8] = b"ACAD_MTEXT_COLUMN_INFO_END";
const LINKS_BEGIN: &[u8] = b"ACAD_MTEXT_COLUMNS_BEGIN";
const LINKS_END: &[u8] = b"ACAD_MTEXT_COLUMNS_END";

fn assert_parity(
    directory: &DxfMTextXDataLinkedColumnResolutionDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 1);
    let entry = directory.entries()[0];
    assert!(entry.column_info().is_some());
    assert_eq!(entry.target_count(), 1);
    let targets = directory
        .targets_for_entry(entry)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        targets[0].state(),
        DxfMTextXDataLinkedColumnTargetState::UniqueMText
    );
    assert!(targets[0].target_record().is_some());
    Ok(())
}

fn ascii_parity_document(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n5\n1\n1001\nACAD\n\
1000\nACAD_MTEXT_COLUMN_INFO_BEGIN\n1000\nACAD_MTEXT_COLUMN_INFO_END\n\
1000\nACAD_MTEXT_COLUMNS_BEGIN\n1070\n47\n1070\n2\n1005\nA\n\
1000\nACAD_MTEXT_COLUMNS_END\n0\nMTEXT\n5\nA\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn ascii_all_states_document() -> Vec<u8> {
    b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1021\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n5\n1\n1001\nACAD\n\
1000\nACAD_MTEXT_COLUMN_INFO_BEGIN\n1000\nACAD_MTEXT_COLUMN_INFO_END\n\
1000\nACAD_MTEXT_COLUMNS_BEGIN\n1070\n47\n1070\n7\n\
1005\nA\n1005\nB\n1005\nC\n1005\n0\n1005\nG\n1005\nD\n\
1000\nACAD_MTEXT_COLUMNS_END\n\
0\nMTEXT\n5\nA\n0\nLINE\n5\nB\n0\nMTEXT\n5\nD\n0\nLINE\n5\nD\n\
0\nENDSEC\n0\nEOF\n"
        .to_vec()
}

fn ascii_without_info() -> Vec<u8> {
    b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1021\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n1001\nACAD\n\
1000\nACAD_MTEXT_COLUMNS_BEGIN\n1070\n47\n1070\n2\n1005\nC\n\
1000\nACAD_MTEXT_COLUMNS_END\n0\nENDSEC\n0\nEOF\n"
        .to_vec()
}

fn binary_parity_document(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"MTEXT"),
        (5, b"1"),
        (1001, APP),
        (1000, INFO_BEGIN),
        (1000, INFO_END),
        (1000, LINKS_BEGIN),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 1070, 47)?;
    push_i16(&mut bytes, version, 1070, 2)?;
    push_string(&mut bytes, version, 1005, b"A")?;
    for (code, value) in [
        (1000, LINKS_END),
        (0, b"MTEXT".as_slice()),
        (5, b"A"),
        (0, b"ENDSEC"),
        (0, b"EOF"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
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

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 && (0..=255).contains(&code) {
        bytes.push(u8::try_from(code).map_err(|_| invalid_test_data())?);
    } else if version == DxfAcadVersion::Ac1009 {
        bytes.push(0xff);
        bytes.extend_from_slice(&code.to_le_bytes());
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

fn invalid_test_data() -> io::Error {
    io::Error::other("invalid test data")
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

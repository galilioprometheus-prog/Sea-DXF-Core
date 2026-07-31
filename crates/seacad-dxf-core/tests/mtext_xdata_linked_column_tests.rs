use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandleParseIssue, DxfMTextXDataLinkedColumnDirectory,
    DxfMTextXDataLinkedColumnEntry, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfTextSymbolNumericIssue, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_linked_column_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_document(version.code(), APP, BEGIN, 47, 1070, END);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        assert_directory(
            &ascii.mtext_xdata_linked_column_directory(&DxfCancellationToken::default())?,
        )?;

        let binary_bytes = binary_document(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_directory(
            &binary.mtext_xdata_linked_column_directory(&DxfCancellationToken::default())?,
        )?;
    }
    Ok(())
}

#[test]
fn exact_framing_rejects_inexact_and_incomplete_blocks_without_partial_handles()
-> Result<(), Box<dyn Error>> {
    for bytes in [
        ascii_document("AC1021", "NOT_ACAD", BEGIN, 47, 1070, END),
        ascii_document("AC1021", APP, "NOT_COLUMNS_BEGIN", 47, 1070, END),
        ascii_document("AC1021", APP, BEGIN, 46, 1070, END),
        ascii_document("AC1021", APP, BEGIN, 47, 1071, END),
        ascii_document("AC1021", APP, BEGIN, 47, 1070, "NOT_COLUMNS_END"),
    ] {
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let directory =
            document.mtext_xdata_linked_column_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert!(directory.handles().is_empty());
    }
    Ok(())
}

#[test]
fn invalid_values_cancellation_lookup_identity_and_traits_remain_typed()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextXDataLinkedColumnEntry>();
    assert_send_sync::<DxfMTextXDataLinkedColumnDirectory>();

    let bytes = ascii_invalid_values();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_xdata_linked_column_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.mtext_xdata_linked_column_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    let entry = directory
        .entries()
        .first()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert!(matches!(
        entry.declared_column_count(),
        Err(DxfTextSymbolNumericIssue::InvalidAsciiNumber(_))
    ));
    assert_eq!(
        directory.entry_for_begin_occurrence(entry.begin().occurrence()),
        Some(entry)
    );
    assert_eq!(directory.entry_for_begin_occurrence(u64::MAX), None);
    let handles = directory
        .handles_for_entry(entry)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(handles.len(), 1);
    assert_eq!(
        handles[0].parse_result(),
        Err(DxfHandleParseIssue::InvalidDigit { offset: 0 })
    );
    Ok(())
}

const APP: &str = "ACAD";
const BEGIN: &str = "ACAD_MTEXT_COLUMNS_BEGIN";
const END: &str = "ACAD_MTEXT_COLUMNS_END";

fn assert_directory(directory: &DxfMTextXDataLinkedColumnDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 1);
    let entry = directory.entries()[0];
    assert_eq!(entry.selector_group().group_code().value(), 1070);
    assert_eq!(entry.count_group().group_code().value(), 1070);
    assert_eq!(entry.declared_column_count(), Ok(3));
    assert_eq!(entry.linked_handle_count(), 2);
    assert_eq!(entry.end().group_code().value(), 1000);
    let handles = directory
        .handles_for_entry(entry)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        handles
            .iter()
            .map(|handle| handle.parse_result().map(|value| value.value()))
            .collect::<Vec<_>>(),
        [Ok(0x2e), Ok(0x2f)]
    );
    assert!(
        handles
            .iter()
            .all(|handle| handle.group().group_code().value() == 1005)
    );
    Ok(())
}

fn ascii_document(
    version: &str,
    app: &str,
    begin: &str,
    selector: i16,
    count_code: i16,
    end: &str,
) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n1001\n{app}\n1000\n{begin}\n\
1070\n{selector}\n{count_code}\n3\n1005\n2E\n1005\n2F\n1000\n{end}\n\
0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn ascii_invalid_values() -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1021\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n1001\n{APP}\n1000\n{BEGIN}\n\
1070\n47\n1070\nNOT_COUNT\n1005\nG\n1000\n{END}\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_document(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
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
        (1001, APP.as_bytes()),
        (1000, BEGIN.as_bytes()),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 1070, 47)?;
    push_i16(&mut bytes, version, 1070, 3)?;
    push_string(&mut bytes, version, 1005, b"2E")?;
    push_string(&mut bytes, version, 1005, b"2F")?;
    push_string(&mut bytes, version, 1000, END.as_bytes())?;
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

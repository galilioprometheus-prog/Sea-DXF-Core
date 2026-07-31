use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextXDataColumnDirectory, DxfMTextXDataColumnEntry,
    DxfMTextXDataColumnRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfTextSymbolValueData, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    role: DxfMTextXDataColumnRole,
    bits: u64,
}

#[test]
fn every_dialect_has_ascii_binary_xdata_column_parity() -> Result<(), Box<dyn Error>> {
    let expected = [
        integer(DxfMTextXDataColumnRole::ColumnType, 2),
        integer(DxfMTextXDataColumnRole::ColumnAutoHeight, 0),
        integer(DxfMTextXDataColumnRole::ColumnCount, 3),
        integer(DxfMTextXDataColumnRole::ColumnFlowReversed, 0),
        double(DxfMTextXDataColumnRole::ColumnWidth, 20.0),
        double(DxfMTextXDataColumnRole::ColumnGutter, 1.0),
        integer(DxfMTextXDataColumnRole::ColumnHeightCount, 3),
        double(DxfMTextXDataColumnRole::ColumnHeight, 20.0),
        double(DxfMTextXDataColumnRole::ColumnHeight, 30.0),
        double(DxfMTextXDataColumnRole::ColumnHeight, 0.0),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_document(version.code(), APP, BEGIN, true);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_columns = ascii.mtext_xdata_column_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_document(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_columns =
            binary.mtext_xdata_column_directory(&DxfCancellationToken::default())?;

        assert_eq!(signatures(&ascii_columns)?, expected);
        assert_eq!(signatures(&binary_columns)?, expected);
        assert_eq!(ascii_columns.entries().len(), 1);
        assert_eq!(binary_columns.entries().len(), 1);
    }
    Ok(())
}

#[test]
fn app_markers_are_exact_and_incomplete_blocks_leave_no_partial_values()
-> Result<(), Box<dyn Error>> {
    for bytes in [
        ascii_document("AC1021", "NOT_ACAD", BEGIN, true),
        ascii_document("AC1021", APP, "NOT_COLUMN_INFO_BEGIN", true),
        ascii_document("AC1021", APP, BEGIN, false),
    ] {
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let directory = document.mtext_xdata_column_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert!(directory.values().is_empty());
    }
    Ok(())
}

#[test]
fn provenance_lookup_cancellation_scope_and_traits_are_bounded() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextXDataColumnEntry>();
    assert_send_sync::<DxfMTextXDataColumnDirectory>();

    let bytes = ascii_document("AC1021", APP, BEGIN, true);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_xdata_column_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.mtext_xdata_column_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    let entry = directory
        .entries()
        .first()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        directory.entry_for_begin_occurrence(entry.begin().occurrence()),
        Some(entry)
    );
    assert_eq!(directory.entry_for_begin_occurrence(u64::MAX), None);
    assert_eq!(
        directory
            .values_for_entry(entry)
            .ok_or_else(invalid_test_data)?
            .len() as u64,
        entry.value_count()
    );
    for value in directory.values() {
        assert_eq!(value.field_id_group().group_code().value(), 1070);
        assert!(matches!(
            value.value_group().group_code().value(),
            1070 | 1040
        ));
    }
    Ok(())
}

const APP: &str = "ACAD";
const BEGIN: &str = "ACAD_MTEXT_COLUMN_INFO_BEGIN";
const END: &str = "ACAD_MTEXT_COLUMN_INFO_END";

fn signatures(directory: &DxfMTextXDataColumnDirectory) -> Result<Vec<Signature>, Box<dyn Error>> {
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            let bits = match value.data() {
                DxfTextSymbolValueData::Int16(Ok(number)) => number as i64 as u64,
                DxfTextSymbolValueData::Double(Ok(number)) => number.to_bits(),
                _ => return Err(invalid_test_data().into()),
            };
            Ok(Signature {
                role: value.role(),
                bits,
            })
        })
        .collect()
}

fn integer(role: DxfMTextXDataColumnRole, value: i16) -> Signature {
    Signature {
        role,
        bits: value as i64 as u64,
    }
}

fn double(role: DxfMTextXDataColumnRole, value: f64) -> Signature {
    Signature {
        role,
        bits: value.to_bits(),
    }
}

fn ascii_document(version: &str, app: &str, begin: &str, complete: bool) -> Vec<u8> {
    let end = if complete {
        format!("1000\n{END}\n")
    } else {
        String::new()
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n1001\n{app}\n1000\n{begin}\n\
1070\n999\n1040\n123\n\
1070\n75\n1070\n2\n1070\n79\n1070\n0\n1070\n76\n1070\n3\n\
1070\n78\n1070\n0\n1070\n48\n1040\n20\n1070\n49\n1040\n1\n\
1070\n50\n1070\n3\n1040\n20\n1040\n30\n1040\n0\n{end}\
0\nTEXT\n40\n1\n0\nENDSEC\n0\nEOF\n"
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
    push_i16(&mut bytes, version, 1070, 999)?;
    push_double(&mut bytes, version, 1040, 123.0)?;
    for (field, value) in [(75, 2), (79, 0), (76, 3), (78, 0)] {
        push_i16(&mut bytes, version, 1070, field)?;
        push_i16(&mut bytes, version, 1070, value)?;
    }
    for (field, value) in [(48, 20.0), (49, 1.0)] {
        push_i16(&mut bytes, version, 1070, field)?;
        push_double(&mut bytes, version, 1040, value)?;
    }
    push_i16(&mut bytes, version, 1070, 50)?;
    push_i16(&mut bytes, version, 1070, 3)?;
    for height in [20.0, 30.0, 0.0] {
        push_double(&mut bytes, version, 1040, height)?;
    }
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

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
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

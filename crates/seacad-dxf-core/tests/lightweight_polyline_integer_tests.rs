use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfLightweightPolylineInteger, DxfLightweightPolylineIntegerDirectory,
    DxfLightweightPolylineIntegerIssue, DxfLightweightPolylineIntegerRecordEntry,
    DxfLightweightPolylineIntegerRole, DxfLightweightPolylineIntegerValue, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type IntegerEvidence = (
    DxfLightweightPolylineIntegerRole,
    DxfLightweightPolylineInteger,
);

#[test]
fn every_supported_dialect_preserves_ascii_binary_integer_evidence() -> Result<(), Box<dyn Error>> {
    let expected = vec![
        (
            DxfLightweightPolylineIntegerRole::VertexCount,
            DxfLightweightPolylineInteger::I32(2),
        ),
        (
            DxfLightweightPolylineIntegerRole::Flags,
            DxfLightweightPolylineInteger::I16(129),
        ),
        (
            DxfLightweightPolylineIntegerRole::VertexIdentifier,
            DxfLightweightPolylineInteger::I32(7),
        ),
        (
            DxfLightweightPolylineIntegerRole::VertexIdentifier,
            DxfLightweightPolylineInteger::I32(8),
        ),
    ];

    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.lightweight_polyline_integer_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.lightweight_polyline_integer_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, expected);
        assert_eq!(evidence(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn signed_boundaries_duplicates_and_ascii_failures_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n-2147483648\n90\n+2147483647\n70\n-32768\n70\n+32767\n91\n2147483648\n70\n32768\n91\n12x\n0\nLWPOLYLINE\n10\n1\n20\n2\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.lightweight_polyline_integer_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    let first = directory.records()[0];
    let values = directory
        .values_for_raw_record(first.record().ordinal())
        .ok_or(io::Error::other("first lwpolyline integer values"))?;
    assert_eq!(values.len(), 7);
    assert_eq!(
        values[0].value(),
        Ok(DxfLightweightPolylineInteger::I32(i32::MIN))
    );
    assert_eq!(
        values[1].value(),
        Ok(DxfLightweightPolylineInteger::I32(i32::MAX))
    );
    assert_eq!(
        values[2].value(),
        Ok(DxfLightweightPolylineInteger::I16(i16::MIN))
    );
    assert_eq!(
        values[3].value(),
        Ok(DxfLightweightPolylineInteger::I16(i16::MAX))
    );
    assert_eq!(
        values[4].value(),
        Err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[5].value(),
        Err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[6].value(),
        Err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 2 }
        ))
    );

    let second = directory.records()[1];
    assert!(second.value_range().is_empty());
    assert_eq!(
        directory.values_for_raw_record(second.record().ordinal()),
        Some([].as_slice())
    );
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    Ok(())
}

#[test]
fn matching_is_exact_and_limited_to_complete_block_and_entity_sections()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nLWPOLYLINE\n90\n1\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nLWPOLYLINE\n70\n2\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nlwpolyline\n90\n3\n0\nLWPOLYLINE \n90\n4\n0\nLWPOLYLINE\n91\n5\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nLWPOLYLINE\n90\n6\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.lightweight_polyline_integer_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.values().len(), 2);
    assert_eq!(
        directory.values()[0].role(),
        DxfLightweightPolylineIntegerRole::Flags
    );
    assert_eq!(
        directory.values()[1].role(),
        DxfLightweightPolylineIntegerRole::VertexIdentifier
    );
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.lightweight_polyline_integer_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.lightweight_polyline_integer_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_copy::<DxfLightweightPolylineIntegerValue>();
    assert_copy::<DxfLightweightPolylineIntegerRecordEntry>();
    assert_send_sync::<DxfLightweightPolylineIntegerDirectory>();
    Ok(())
}

fn assert_directory(
    directory: &DxfLightweightPolylineIntegerDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.values().len(), 4);
    assert!(directory.raw_record_count() >= 1);
    let record = directory.records()[0];
    let values = directory
        .values_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("lwpolyline integer values"))?;
    assert_eq!(values.len() as u64, record.value_range().len());
    for value in values {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(*value)
        );
    }
    Ok(())
}

fn evidence(
    directory: &DxfLightweightPolylineIntegerDirectory,
) -> Result<Vec<IntegerEvidence>, io::Error> {
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            Ok((
                value.role(),
                value
                    .value()
                    .map_err(|_| io::Error::other("lwpolyline integer value"))?,
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n70\n129\n10\n1\n20\n2\n91\n7\n10\n3\n20\n4\n91\n8\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"LWPOLYLINE")?;
    push_i32(&mut bytes, version, 90, 2)?;
    push_i16(&mut bytes, version, 70, 129)?;
    push_double(&mut bytes, version, 10, 1.0)?;
    push_double(&mut bytes, version, 20, 2.0)?;
    push_i32(&mut bytes, version, 91, 7)?;
    push_double(&mut bytes, version, 10, 3.0)?;
    push_double(&mut bytes, version, 20, 4.0)?;
    push_i32(&mut bytes, version, 91, 8)?;
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

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_i32(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i32) -> io::Result<()> {
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfLightweightPolylineDirectory, DxfLightweightPolylineNumericIssue,
    DxfLightweightPolylineRecordEntry, DxfLightweightPolylineValue,
    DxfLightweightPolylineValueRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

type ValueEvidence = (DxfLightweightPolylineValueRole, u64);

#[test]
fn every_supported_dialect_preserves_ascii_binary_lwpolyline_values() -> Result<(), Box<dyn Error>>
{
    let expected = vec![
        (
            DxfLightweightPolylineValueRole::OcsElevation,
            (-0.0_f64).to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::Thickness,
            0.5_f64.to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::ConstantWidth,
            1.5_f64.to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::OcsVertexX,
            1.0_f64.to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::OcsVertexY,
            2.0_f64.to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::StartWidth,
            0.25_f64.to_bits(),
        ),
        (DxfLightweightPolylineValueRole::EndWidth, 0.5_f64.to_bits()),
        (DxfLightweightPolylineValueRole::Bulge, (-0.0_f64).to_bits()),
        (
            DxfLightweightPolylineValueRole::OcsVertexX,
            3.0_f64.to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::OcsVertexY,
            4.0_f64.to_bits(),
        ),
        (DxfLightweightPolylineValueRole::Bulge, 1.0_f64.to_bits()),
        (
            DxfLightweightPolylineValueRole::ExtrusionX,
            0.0_f64.to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::ExtrusionY,
            (-0.0_f64).to_bits(),
        ),
        (
            DxfLightweightPolylineValueRole::ExtrusionZ,
            1.0_f64.to_bits(),
        ),
    ];

    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.lightweight_polyline_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.lightweight_polyline_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, expected);
        assert_eq!(evidence(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn duplicates_invalid_ascii_and_integer_metadata_remain_separate() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n70\n129\n10\n.\n10\n1e-9999\n20\n2\n91\n7\n0\nLWPOLYLINE\n90\n0\n70\n0\n91\n8\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.lightweight_polyline_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    let first = directory.records()[0];
    let values = directory
        .values_for_raw_record(first.record().ordinal())
        .ok_or(io::Error::other("first lwpolyline values"))?;
    assert_eq!(values.len(), 3);
    assert_eq!(
        values[0].value(),
        Err(DxfLightweightPolylineNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        values[1].value(),
        Err(DxfLightweightPolylineNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[2].value().map(DxfDouble::to_bits),
        Ok(2.0_f64.to_bits())
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
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nLWPOLYLINE\n10\n1\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nLWPOLYLINE\n20\n2\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nlwpolyline\n10\n3\n0\nLWPOLYLINE \n10\n4\n0\nLWPOLYLINE\n42\n5\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nLWPOLYLINE\n10\n6\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.lightweight_polyline_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.values().len(), 2);
    assert_eq!(
        directory.values()[0].role(),
        DxfLightweightPolylineValueRole::OcsVertexY
    );
    assert_eq!(
        directory.values()[1].role(),
        DxfLightweightPolylineValueRole::Bulge
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
        document.lightweight_polyline_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.lightweight_polyline_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_copy::<DxfLightweightPolylineValue>();
    assert_copy::<DxfLightweightPolylineRecordEntry>();
    assert_send_sync::<DxfLightweightPolylineDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfLightweightPolylineDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.values().len(), 14);
    assert!(directory.raw_record_count() >= 1);
    let record = directory.records()[0];
    let values = directory
        .values_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("lwpolyline values"))?;
    assert_eq!(values.len() as u64, record.value_range().len());
    for value in values {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(*value)
        );
    }
    Ok(())
}

fn evidence(directory: &DxfLightweightPolylineDirectory) -> Result<Vec<ValueEvidence>, io::Error> {
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            Ok((
                value.role(),
                value
                    .value()
                    .map_err(|_| io::Error::other("lwpolyline numeric value"))?
                    .to_bits(),
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n70\n129\n38\n-0\n39\n0.5\n43\n1.5\n10\n1\n20\n2\n91\n7\n40\n0.25\n41\n0.5\n42\n-0\n10\n3\n20\n4\n91\n8\n42\n1\n210\n0\n220\n-0\n230\n1\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [(38, -0.0), (39, 0.5), (43, 1.5), (10, 1.0), (20, 2.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 7)?;
    for (code, value) in [(40, 0.25), (41, 0.5), (42, -0.0), (10, 3.0), (20, 4.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 8)?;
    for (code, value) in [(42, 1.0), (210, 0.0), (220, -0.0), (230, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
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

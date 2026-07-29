use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfInfiniteLineGeometryDirectory, DxfInfiniteLineGeometryKind,
    DxfInfiniteLineGeometryNumericIssue, DxfInfiniteLineGeometryRecordEntry,
    DxfInfiniteLineGeometryValue, DxfInfiniteLineGeometryValueRole, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type ValueEvidence = (
    DxfInfiniteLineGeometryKind,
    DxfInfiniteLineGeometryValueRole,
    u64,
);

#[test]
fn every_supported_dialect_preserves_ascii_binary_ray_xline_evidence() -> Result<(), Box<dyn Error>>
{
    let expected = vec![
        (
            DxfInfiniteLineGeometryKind::Ray,
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointX,
            (-0.0_f64).to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Ray,
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointY,
            2.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Ray,
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointZ,
            3.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Ray,
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionX,
            1.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Ray,
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionY,
            0.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Ray,
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionZ,
            0.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Xline,
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointX,
            4.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Xline,
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointY,
            5.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Xline,
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointZ,
            6.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Xline,
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionX,
            0.0_f64.to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Xline,
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionY,
            (-0.0_f64).to_bits(),
        ),
        (
            DxfInfiniteLineGeometryKind::Xline,
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionZ,
            1.0_f64.to_bits(),
        ),
    ];

    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.infinite_line_geometry_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.infinite_line_geometry_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, expected);
        assert_eq!(evidence(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn duplicates_invalid_ascii_and_empty_slices_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nRAY\n10\n.\n10\n1e-9999\n20\n2\n210\n7\n0\nXLINE\n8\nLayer\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.infinite_line_geometry_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    let ray = directory.records()[0];
    let values = directory
        .values_for_raw_record(ray.record().ordinal())
        .ok_or(io::Error::other("ray values"))?;
    assert_eq!(values.len(), 3);
    assert_eq!(
        values[0].value(),
        Err(DxfInfiniteLineGeometryNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        values[1].value(),
        Err(DxfInfiniteLineGeometryNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[2].value().map(DxfDouble::to_bits),
        Ok(2.0_f64.to_bits())
    );

    let xline = directory.records()[1];
    assert!(xline.value_range().is_empty());
    assert_eq!(
        directory.values_for_raw_record(xline.record().ordinal()),
        Some([].as_slice())
    );
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    Ok(())
}

#[test]
fn matching_is_exact_and_limited_to_complete_entity_sections() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nRAY\n10\n1\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nXLINE\n11\n2\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nray\n10\n3\n0\nRAY \n10\n4\n0\nRAY\n21\n5\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nXLINE\n11\n6\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.infinite_line_geometry_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    assert_eq!(
        directory.records()[0].kind(),
        DxfInfiniteLineGeometryKind::Xline
    );
    assert_eq!(
        directory.records()[1].kind(),
        DxfInfiniteLineGeometryKind::Ray
    );
    assert_eq!(directory.values().len(), 2);
    assert_eq!(
        directory.values()[0].role(),
        DxfInfiniteLineGeometryValueRole::WcsUnitDirectionX
    );
    assert_eq!(
        directory.values()[1].role(),
        DxfInfiniteLineGeometryValueRole::WcsUnitDirectionY
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
        document.infinite_line_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.infinite_line_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_copy::<DxfInfiniteLineGeometryValue>();
    assert_copy::<DxfInfiniteLineGeometryRecordEntry>();
    assert_send_sync::<DxfInfiniteLineGeometryDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfInfiniteLineGeometryDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.values().len(), 12);
    assert!(directory.raw_record_count() >= 2);
    for record in directory.records() {
        let values = directory
            .values_for_raw_record(record.record().ordinal())
            .ok_or(io::Error::other("infinite-line values"))?;
        assert_eq!(values.len() as u64, record.value_range().len());
        for value in values {
            assert_eq!(
                directory.value_for_group(value.group().occurrence()),
                Some(*value)
            );
        }
    }
    Ok(())
}

fn evidence(directory: &DxfInfiniteLineGeometryDirectory) -> Result<Vec<ValueEvidence>, io::Error> {
    let mut result = Vec::new();
    for record in directory.records() {
        for value in directory
            .values_for_raw_record(record.record().ordinal())
            .ok_or(io::Error::other("infinite-line values"))?
        {
            result.push((
                record.kind(),
                value.role(),
                value
                    .value()
                    .map_err(|_| io::Error::other("infinite-line numeric value"))?
                    .to_bits(),
            ));
        }
    }
    Ok(result)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nRAY\n10\n-0\n20\n2\n30\n3\n11\n1\n21\n0\n31\n0\n40\n99\n0\nXLINE\n10\n4\n20\n5\n30\n6\n11\n0\n21\n-0\n31\n1\n210\n7\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"RAY")?;
    for (code, value) in [
        (10, -0.0),
        (20, 2.0),
        (30, 3.0),
        (11, 1.0),
        (21, 0.0),
        (31, 0.0),
        (40, 99.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"XLINE")?;
    for (code, value) in [
        (10, 4.0),
        (20, 5.0),
        (30, 6.0),
        (11, 0.0),
        (21, -0.0),
        (31, 1.0),
        (210, 7.0),
    ] {
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

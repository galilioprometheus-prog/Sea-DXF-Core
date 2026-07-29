use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfCircularGeometryDirectory,
    DxfCircularGeometryKind, DxfCircularGeometryNumericIssue, DxfCircularGeometryRecordEntry,
    DxfCircularGeometryValue, DxfCircularGeometryValueRole, DxfDouble, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type ValueEvidence = (DxfCircularGeometryValueRole, u64);
type RecordEvidence = (DxfCircularGeometryKind, Vec<ValueEvidence>);

#[test]
fn every_supported_dialect_preserves_ascii_binary_circle_arc_evidence() -> Result<(), Box<dyn Error>>
{
    let expected = vec![
        (
            DxfCircularGeometryKind::Circle,
            vec![
                (
                    DxfCircularGeometryValueRole::OcsCenterX,
                    (-0.0_f64).to_bits(),
                ),
                (DxfCircularGeometryValueRole::OcsCenterY, 2.5_f64.to_bits()),
                (DxfCircularGeometryValueRole::OcsCenterZ, 3.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::Radius, 4.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::ExtrusionX, 0.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::ExtrusionZ, 1.0_f64.to_bits()),
            ],
        ),
        (
            DxfCircularGeometryKind::Arc,
            vec![
                (DxfCircularGeometryValueRole::OcsCenterX, 1.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::OcsCenterY, 2.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::OcsCenterZ, 3.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::Radius, 4.0_f64.to_bits()),
                (
                    DxfCircularGeometryValueRole::StartAngle,
                    (-0.0_f64).to_bits(),
                ),
                (DxfCircularGeometryValueRole::EndAngle, 270.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::ExtrusionX, 0.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::ExtrusionY, 1.0_f64.to_bits()),
                (DxfCircularGeometryValueRole::ExtrusionZ, 0.0_f64.to_bits()),
            ],
        ),
    ];

    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.circular_geometry_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.circular_geometry_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, expected);
        assert_eq!(evidence(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn duplicates_invalid_ascii_and_unrelated_codes_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n10\n.\n10\n1e-9999\n20\n2\n50\n90\n39\n7\n0\nARC\n8\nLayer\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.circular_geometry_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    let circle = directory.records()[0];
    let circle_values = directory
        .values_for_raw_record(circle.record().ordinal())
        .ok_or(io::Error::other("circle values"))?;
    assert_eq!(circle_values.len(), 3);
    assert_eq!(
        circle_values[0].value(),
        Err(DxfCircularGeometryNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        circle_values[1].value(),
        Err(DxfCircularGeometryNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        circle_values[2].value().map(DxfDouble::to_bits),
        Ok(2.0_f64.to_bits())
    );
    assert!(circle_values.iter().all(|value| !matches!(
        value.role(),
        DxfCircularGeometryValueRole::StartAngle | DxfCircularGeometryValueRole::EndAngle
    )));

    let arc = directory.records()[1];
    assert!(arc.value_range().is_empty());
    assert_eq!(
        directory.values_for_raw_record(arc.record().ordinal()),
        Some([].as_slice())
    );
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    Ok(())
}

#[test]
fn matching_is_exact_and_limited_to_complete_entity_sections() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nCIRCLE\n10\n1\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nCIRCLE\n40\n2\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\ncircle\n40\n3\n0\nARC \n50\n4\n0\nARC\n51\n5\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nARC\n50\n6\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.circular_geometry_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    assert_eq!(
        directory.records()[0].kind(),
        DxfCircularGeometryKind::Circle
    );
    assert_eq!(directory.records()[1].kind(), DxfCircularGeometryKind::Arc);
    assert_eq!(directory.values().len(), 2);
    assert_eq!(
        directory.values()[0].role(),
        DxfCircularGeometryValueRole::Radius
    );
    assert_eq!(
        directory.values()[1].role(),
        DxfCircularGeometryValueRole::EndAngle
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
        document.circular_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.circular_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_copy::<DxfCircularGeometryValue>();
    assert_copy::<DxfCircularGeometryRecordEntry>();
    assert_send_sync::<DxfCircularGeometryDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfCircularGeometryDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert!(directory.raw_record_count() >= directory.records().len() as u64);
    for entry in directory.records().iter().copied() {
        let values = directory
            .values_for_raw_record(entry.record().ordinal())
            .ok_or(io::Error::other("circular record values"))?;
        assert_eq!(values.len() as u64, entry.value_range().len());
        for value in values {
            assert!(
                value.group().occurrence() >= entry.record().group_range().start()
                    && value.group().occurrence() < entry.record().group_range().end()
            );
            assert_eq!(
                directory.value_for_group(value.group().occurrence()),
                Some(*value)
            );
        }
    }
    Ok(())
}

fn evidence(directory: &DxfCircularGeometryDirectory) -> Result<Vec<RecordEvidence>, io::Error> {
    directory
        .records()
        .iter()
        .copied()
        .map(|entry| {
            let values = directory
                .values_for_raw_record(entry.record().ordinal())
                .ok_or(io::Error::other("circular record values"))?
                .iter()
                .filter_map(|value| {
                    value
                        .value()
                        .ok()
                        .map(|number| (value.role(), number.to_bits()))
                })
                .collect();
            Ok((entry.kind(), values))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n10\n-0\n20\n2.5\n30\n3\n40\n4\n50\n99\n210\n0\n230\n1\n0\nARC\n10\n1\n20\n2\n30\n3\n40\n4\n50\n-0\n51\n270\n210\n0\n220\n1\n230\n0\n0\ncircle\n40\n88\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nARC\n50\n77\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary_string(&mut bytes, version, 0, b"SECTION")?;
    push_binary_string(&mut bytes, version, 2, b"HEADER")?;
    push_binary_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_binary_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"SECTION")?;
    push_binary_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_binary_string(&mut bytes, version, 0, b"CIRCLE")?;
    for (code, value) in [
        (10, -0.0),
        (20, 2.5),
        (30, 3.0),
        (40, 4.0),
        (50, 99.0),
        (210, 0.0),
        (230, 1.0),
    ] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_string(&mut bytes, version, 0, b"ARC")?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (40, 4.0),
        (50, -0.0),
        (51, 270.0),
        (210, 0.0),
        (220, 1.0),
        (230, 0.0),
    ] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_string(&mut bytes, version, 0, b"circle")?;
    push_binary_double(&mut bytes, version, 40, 88.0)?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"SECTION")?;
    push_binary_string(&mut bytes, version, 2, b"OBJECTS")?;
    push_binary_string(&mut bytes, version, 0, b"ARC")?;
    push_binary_double(&mut bytes, version, 50, 77.0)?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_binary_code(bytes, version, group_code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_binary_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: f64,
) -> io::Result<()> {
    push_binary_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_binary_code(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
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

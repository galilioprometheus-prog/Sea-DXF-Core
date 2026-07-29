use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBasicGeometryComponent, DxfBasicGeometryComponentRole, DxfBasicGeometryDirectory,
    DxfBasicGeometryKind, DxfBasicGeometryNumericIssue, DxfBasicGeometryRecordEntry,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type ComponentEvidence = (DxfBasicGeometryComponentRole, u64);
type RecordEvidence = (DxfBasicGeometryKind, Vec<ComponentEvidence>);

#[test]
fn ascii_and_binary_preserve_point_line_component_order_and_bits() -> Result<(), Box<dyn Error>> {
    let expected = vec![
        (
            DxfBasicGeometryKind::Point,
            vec![
                (
                    DxfBasicGeometryComponentRole::WcsLocationOrStartX,
                    (-0.0_f64).to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsLocationOrStartY,
                    2.5_f64.to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
                    3.0_f64.to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsLocationOrStartX,
                    4.0_f64.to_bits(),
                ),
                (DxfBasicGeometryComponentRole::ExtrusionX, 0.0_f64.to_bits()),
                (DxfBasicGeometryComponentRole::ExtrusionY, 0.0_f64.to_bits()),
                (DxfBasicGeometryComponentRole::ExtrusionZ, 1.0_f64.to_bits()),
            ],
        ),
        (
            DxfBasicGeometryKind::Line,
            vec![
                (
                    DxfBasicGeometryComponentRole::WcsLocationOrStartX,
                    1.0_f64.to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsLocationOrStartY,
                    2.0_f64.to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsLocationOrStartZ,
                    3.0_f64.to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsEndpointX,
                    4.0_f64.to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsEndpointY,
                    5.0_f64.to_bits(),
                ),
                (
                    DxfBasicGeometryComponentRole::WcsEndpointZ,
                    6.0_f64.to_bits(),
                ),
                (DxfBasicGeometryComponentRole::ExtrusionX, 0.0_f64.to_bits()),
                (DxfBasicGeometryComponentRole::ExtrusionY, 1.0_f64.to_bits()),
                (DxfBasicGeometryComponentRole::ExtrusionZ, 0.0_f64.to_bits()),
            ],
        ),
    ];

    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.basic_geometry_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.basic_geometry_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        let ascii_evidence = evidence(&ascii_directory)?;
        let binary_evidence = evidence(&binary_directory)?;
        assert_eq!(ascii_evidence, binary_evidence);
        assert_eq!(ascii_evidence, expected);
    }
    Ok(())
}

#[test]
fn invalid_ascii_duplicates_and_absence_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOINT\n10\n.\n10\n1e-9999\n20\n2\n0\nLINE\n8\nLayer\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.basic_geometry_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    let point = directory.records()[0];
    let point_components = directory
        .components_for_raw_record(point.record().ordinal())
        .ok_or(io::Error::other("point components"))?;
    assert_eq!(point_components.len(), 3);
    assert_eq!(
        point_components[0].value(),
        Err(DxfBasicGeometryNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        point_components[1].value(),
        Err(DxfBasicGeometryNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        point_components[2].value().map(DxfDouble::to_bits),
        Ok(2.0_f64.to_bits())
    );

    let line = directory.records()[1];
    assert!(line.component_range().is_empty());
    assert_eq!(
        directory.components_for_raw_record(line.record().ordinal()),
        Some([].as_slice())
    );
    assert_eq!(directory.components_for_raw_record(u64::MAX), None);
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    Ok(())
}

#[test]
fn matching_is_exact_and_limited_to_entity_bearing_sections() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nPOINT\n10\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\npoint\n10\n2\n0\nLINE \n10\n3\n0\nPOINT\n11\n4\n21\n5\n31\n6\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nLINE\n10\n7\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.basic_geometry_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.records()[0].kind(), DxfBasicGeometryKind::Point);
    assert!(directory.records()[0].component_range().is_empty());
    assert!(directory.components().is_empty());
    Ok(())
}

#[test]
fn cancellation_is_observed_and_public_evidence_is_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.basic_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    assert_copy::<DxfBasicGeometryComponent>();
    assert_copy::<DxfBasicGeometryRecordEntry>();
    assert_send_sync::<DxfBasicGeometryDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfBasicGeometryDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.records()[0].kind(), DxfBasicGeometryKind::Point);
    assert_eq!(directory.records()[1].kind(), DxfBasicGeometryKind::Line);
    assert!(directory.raw_record_count() >= directory.records().len() as u64);

    for entry in directory.records().iter().copied() {
        let components = directory
            .components_for_raw_record(entry.record().ordinal())
            .ok_or(io::Error::other("components"))?;
        assert_eq!(components.len() as u64, entry.component_range().len());
        for component in components {
            assert!(
                component.group().occurrence() >= entry.record().group_range().start()
                    && component.group().occurrence() < entry.record().group_range().end()
            );
            assert_eq!(
                directory.component_for_group(component.group().occurrence()),
                Some(*component)
            );
        }
    }
    Ok(())
}

fn evidence(directory: &DxfBasicGeometryDirectory) -> Result<Vec<RecordEvidence>, io::Error> {
    directory
        .records()
        .iter()
        .copied()
        .map(|entry| {
            let values = directory
                .components_for_raw_record(entry.record().ordinal())
                .ok_or(io::Error::other("geometry record components"))?
                .iter()
                .filter_map(|component| {
                    component
                        .value()
                        .ok()
                        .map(|value| (component.role(), value.to_bits()))
                })
                .collect();
            Ok((entry.kind(), values))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOINT\n10\n-0\n20\n2.5\n30\n3\n10\n4\n210\n0\n220\n0\n230\n1\n0\nLINE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n210\n0\n220\n1\n230\n0\n0\npoint\n10\n99\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nPOINT\n10\n88\n0\nENDSEC\n0\nEOF\n"
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
    push_binary_string(&mut bytes, version, 0, b"POINT")?;
    for (code, value) in [
        (10, -0.0),
        (20, 2.5),
        (30, 3.0),
        (10, 4.0),
        (210, 0.0),
        (220, 0.0),
        (230, 1.0),
    ] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_string(&mut bytes, version, 0, b"LINE")?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (11, 4.0),
        (21, 5.0),
        (31, 6.0),
        (210, 0.0),
        (220, 1.0),
        (230, 0.0),
    ] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_string(&mut bytes, version, 0, b"point")?;
    push_binary_double(&mut bytes, version, 10, 99.0)?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"SECTION")?;
    push_binary_string(&mut bytes, version, 2, b"OBJECTS")?;
    push_binary_string(&mut bytes, version, 0, b"POINT")?;
    push_binary_double(&mut bytes, version, 10, 88.0)?;
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

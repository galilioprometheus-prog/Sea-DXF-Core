use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfPlanarFaceDirectory, DxfPlanarFaceKind, DxfPlanarFaceNumber, DxfPlanarFaceNumericIssue,
    DxfPlanarFaceRecordEntry, DxfPlanarFaceValue, DxfPlanarFaceValueRole, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NumberSignature {
    Double(u64),
    Int16(i16),
}

type ValueSignature = (DxfPlanarFaceValueRole, NumberSignature);
type RecordSignature = (DxfPlanarFaceKind, Vec<ValueSignature>);

#[test]
fn ascii_and_binary_preserve_all_three_face_families() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.planar_face_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.planar_face_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        let ascii_signatures = signatures(&ascii_directory)?;
        assert_eq!(ascii_signatures, signatures(&binary_directory)?);
        assert_eq!(
            ascii_signatures[0].1.last(),
            Some(&(
                DxfPlanarFaceValueRole::InvisibleEdgeFlags,
                NumberSignature::Int16(9)
            ))
        );
        assert_eq!(
            ascii_signatures[1].1.last(),
            Some(&(
                DxfPlanarFaceValueRole::ExtrusionZ,
                NumberSignature::Double(1.0_f64.to_bits())
            ))
        );
        assert_eq!(
            ascii_signatures[2].1.first(),
            Some(&(
                DxfPlanarFaceValueRole::FirstCornerX,
                NumberSignature::Double(21.0_f64.to_bits())
            ))
        );
    }
    Ok(())
}

#[test]
fn invalid_ascii_duplicates_and_wire_domains_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\n3DFACE\n10\n.\n10\n2\n20\n3\n70\n32768\n39\n4\n\
0\nSOLID\n10\n1\n20\n2\n39\n1e-9999\n70\n7\n\
0\nTRACE\n8\nLayer\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.planar_face_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 3);
    let face = directory.records()[0];
    let face_values = directory
        .values_for_raw_record(face.record().ordinal())
        .ok_or(io::Error::other("3DFACE values"))?;
    assert_eq!(face_values.len(), 4);
    assert_eq!(
        face_values[0].value(),
        Err(DxfPlanarFaceNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        face_values[3].value(),
        Err(DxfPlanarFaceNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert!(
        face_values
            .iter()
            .all(|value| value.role() != DxfPlanarFaceValueRole::Thickness)
    );

    let solid = directory.records()[1];
    let solid_values = directory
        .values_for_raw_record(solid.record().ordinal())
        .ok_or(io::Error::other("SOLID values"))?;
    assert_eq!(solid_values.len(), 3);
    assert_eq!(
        solid_values[2].value(),
        Err(DxfPlanarFaceNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert!(
        solid_values
            .iter()
            .all(|value| value.role() != DxfPlanarFaceValueRole::InvisibleEdgeFlags)
    );

    let trace = directory.records()[2];
    assert!(trace.value_range().is_empty());
    assert_eq!(
        directory.values_for_raw_record(trace.record().ordinal()),
        Some([].as_slice())
    );
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    Ok(())
}

#[test]
fn matching_is_exact_and_limited_to_entity_bearing_sections() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\n3DFACE\n10\n1\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\n3dface\n10\n2\n0\nSOLID \n10\n3\n\
0\nTRACE\n10\n4\n0\nENDSEC\n\
0\nSECTION\n2\nOBJECTS\n0\nSOLID\n10\n5\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.planar_face_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.records()[0].kind(), DxfPlanarFaceKind::Trace);
    assert_eq!(directory.records()[0].value_range().len(), 1);
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
        document.planar_face_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    assert_copy::<DxfPlanarFaceValue>();
    assert_copy::<DxfPlanarFaceRecordEntry>();
    assert_send_sync::<DxfPlanarFaceDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfPlanarFaceDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 3);
    assert_eq!(directory.records()[0].kind(), DxfPlanarFaceKind::Face3d);
    assert_eq!(directory.records()[1].kind(), DxfPlanarFaceKind::Solid);
    assert_eq!(directory.records()[2].kind(), DxfPlanarFaceKind::Trace);
    assert!(directory.raw_record_count() >= directory.records().len() as u64);

    for record in directory.records().iter().copied() {
        let values = directory
            .values_for_raw_record(record.record().ordinal())
            .ok_or(io::Error::other("record values"))?;
        assert_eq!(values.len() as u64, record.value_range().len());
        for value in values {
            assert!(
                value.group().occurrence() >= record.record().group_range().start()
                    && value.group().occurrence() < record.record().group_range().end()
            );
            assert_eq!(
                directory.value_for_group(value.group().occurrence()),
                Some(*value)
            );
        }
    }
    Ok(())
}

fn signatures(directory: &DxfPlanarFaceDirectory) -> Result<Vec<RecordSignature>, io::Error> {
    directory
        .records()
        .iter()
        .copied()
        .map(|record| {
            let values = directory
                .values_for_raw_record(record.record().ordinal())
                .ok_or(io::Error::other("record values"))?
                .iter()
                .copied()
                .map(|value| {
                    let number = match value
                        .value()
                        .map_err(|_| io::Error::other("numeric value"))?
                    {
                        DxfPlanarFaceNumber::Double(number) => {
                            NumberSignature::Double(number.to_bits())
                        }
                        DxfPlanarFaceNumber::Int16(number) => NumberSignature::Int16(number),
                        _ => return Err(io::Error::other("unknown numeric wire domain")),
                    };
                    Ok((value.role(), number))
                })
                .collect::<Result<Vec<_>, io::Error>>()?;
            Ok((record.kind(), values))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n\
0\n3DFACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n\
12\n7\n22\n8\n32\n9\n13\n10\n23\n11\n33\n12\n70\n9\n\
0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
0\nSOLID\n10\n-0\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n\
12\n7\n22\n8\n32\n9\n13\n10\n23\n11\n33\n12\n39\n2.5\n\
210\n0\n220\n0\n230\n1\n\
0\nTRACE\n10\n21\n20\n22\n30\n23\n11\n24\n21\n25\n31\n26\n\
12\n27\n22\n28\n32\n29\n13\n30\n23\n31\n33\n32\n39\n-1\n\
210\n0\n220\n1\n230\n0\n\
0\nENDSEC\n0\nEOF\n"
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
    push_binary_string(&mut bytes, version, 2, b"BLOCKS")?;
    push_binary_string(&mut bytes, version, 0, b"3DFACE")?;
    for (code, value) in corner_values(1.0) {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_i16(&mut bytes, version, 70, 9)?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"SECTION")?;
    push_binary_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_binary_string(&mut bytes, version, 0, b"SOLID")?;
    for (code, value) in [
        (10, -0.0),
        (20, 2.0),
        (30, 3.0),
        (11, 4.0),
        (21, 5.0),
        (31, 6.0),
        (12, 7.0),
        (22, 8.0),
        (32, 9.0),
        (13, 10.0),
        (23, 11.0),
        (33, 12.0),
    ] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(39, 2.5), (210, 0.0), (220, 0.0), (230, 1.0)] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_string(&mut bytes, version, 0, b"TRACE")?;
    for (code, value) in corner_values(21.0) {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(39, -1.0), (210, 0.0), (220, 1.0), (230, 0.0)] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn corner_values(first: f64) -> [(i16, f64); 12] {
    [
        (10, first),
        (20, first + 1.0),
        (30, first + 2.0),
        (11, first + 3.0),
        (21, first + 4.0),
        (31, first + 5.0),
        (12, first + 6.0),
        (22, first + 7.0),
        (32, first + 8.0),
        (13, first + 9.0),
        (23, first + 10.0),
        (33, first + 11.0),
    ]
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

fn push_binary_i16(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: i16,
) -> io::Result<()> {
    push_binary_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
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

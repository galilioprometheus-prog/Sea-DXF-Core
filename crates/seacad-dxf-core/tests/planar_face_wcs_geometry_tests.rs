use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPlanarFaceKind, DxfPlanarFaceWcsGeometry,
    DxfPlanarFaceWcsGeometryDirectory, DxfPlanarFaceWcsGeometryEntry,
    DxfPlanarFaceWcsGeometryIssue, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type GeometrySignature = (
    DxfPlanarFaceKind,
    [[u64; 3]; 4],
    Option<[u64; 3]>,
    Option<u64>,
);

#[test]
fn every_dialect_has_ascii_binary_reorder_and_wcs_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_geometry =
            ascii.planar_face_wcs_geometry_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_geometry =
            binary.planar_face_wcs_geometry_directory(&DxfCancellationToken::default())?;

        assert_geometry(&ascii_geometry)?;
        assert_geometry(&binary_geometry)?;
        assert_eq!(signatures(&ascii_geometry)?, signatures(&binary_geometry)?);
    }
    Ok(())
}

#[test]
fn unavailable_zero_and_nonfinite_inputs_fail_typed() -> Result<(), Box<dyn Error>> {
    let ascii_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\n3DFACE\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
0\nSOLID\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
210\n0\n220\n0\n230\n0\n\
0\nTRACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
13\n10\n23\n11\n33\n12\n39\n.\n\
0\nENDSEC\n0\nEOF\n";
    let ascii_source = DxfMemorySource::new(ascii_bytes, DxfResourceProfile::Safe)?;
    let ascii = open_ascii(&ascii_source)?;
    let directory = ascii.planar_face_wcs_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.entries()[0].geometry(),
        Err(DxfPlanarFaceWcsGeometryIssue::CornerUnavailable { corner_index: 0 })
    );
    assert_eq!(
        directory.entries()[1].geometry(),
        Err(DxfPlanarFaceWcsGeometryIssue::ZeroLengthExtrusion)
    );
    assert_eq!(
        directory.entries()[2].geometry(),
        Err(DxfPlanarFaceWcsGeometryIssue::ThicknessUnavailable)
    );

    let binary_bytes = nonfinite_binary_fixture(DxfAcadVersion::Ac1032)?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    let directory = binary.planar_face_wcs_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.entries()[0].geometry(),
        Err(DxfPlanarFaceWcsGeometryIssue::NonFiniteCorner { corner_index: 0 })
    );
    assert_eq!(
        directory.entries()[1].geometry(),
        Err(DxfPlanarFaceWcsGeometryIssue::NonFiniteThickness)
    );
    assert_eq!(
        directory.entries()[2].geometry(),
        Err(DxfPlanarFaceWcsGeometryIssue::NonFiniteExtrusion)
    );
    Ok(())
}

#[test]
fn cancellation_lookup_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.planar_face_wcs_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.planar_face_wcs_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.semantic_directory().source_id()
    );
    assert_eq!(directory.entry_for_raw_record(u64::MAX), None);
    assert_copy::<DxfPlanarFaceWcsGeometry>();
    assert_copy::<DxfPlanarFaceWcsGeometryEntry>();
    assert_send_sync::<DxfPlanarFaceWcsGeometryDirectory>();
    Ok(())
}

fn assert_geometry(directory: &DxfPlanarFaceWcsGeometryDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 3);
    let face = directory.entries()[0]
        .geometry()
        .map_err(|_| io::Error::other("3DFACE geometry"))?;
    assert_eq!(
        bits4(face.corners()),
        [
            bits3f([1.0, 2.0, 3.0]),
            bits3f([4.0, 5.0, 6.0]),
            bits3f([7.0, 8.0, 9.0]),
            bits3f([7.0, 8.0, 9.0]),
        ]
    );
    assert_eq!(face.normal(), None);
    assert_eq!(face.thickness(), None);

    let solid = directory.entries()[1]
        .geometry()
        .map_err(|_| io::Error::other("SOLID geometry"))?;
    assert_eq!(
        bits4(solid.corners()),
        [
            bits3f([-1.0, 3.0, 2.0]),
            bits3f([-4.0, 6.0, 5.0]),
            bits3f([-10.0, 12.0, 11.0]),
            bits3f([-7.0, 9.0, 8.0]),
        ]
    );
    assert_eq!(solid.normal().map(bits3), Some(bits3f([0.0, 1.0, 0.0])));
    assert_eq!(solid.thickness().map(|value| value.to_bits()), Some(0));

    let trace = directory.entries()[2]
        .geometry()
        .map_err(|_| io::Error::other("TRACE geometry"))?;
    assert_eq!(
        bits4(trace.corners()),
        [
            bits3f([21.0, 22.0, 23.0]),
            bits3f([24.0, 25.0, 26.0]),
            bits3f([30.0, 31.0, 32.0]),
            bits3f([27.0, 28.0, 29.0]),
        ]
    );
    assert_eq!(trace.normal().map(bits3), Some(bits3f([0.0, 0.0, 1.0])));
    Ok(())
}

fn signatures(
    directory: &DxfPlanarFaceWcsGeometryDirectory,
) -> Result<Vec<GeometrySignature>, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| {
            let geometry = entry
                .geometry()
                .map_err(|_| io::Error::other("geometry signature"))?;
            Ok((
                geometry.kind(),
                bits4(geometry.corners()),
                geometry.normal().map(bits3),
                geometry.thickness().map(|value| value.to_bits()),
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\n3DFACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
0\nSOLID\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
13\n10\n23\n11\n33\n12\n39\n-0\n210\n0\n220\n1\n230\n0\n\
0\nTRACE\n10\n21\n20\n22\n30\n23\n11\n24\n21\n25\n31\n26\n12\n27\n22\n28\n32\n29\n\
13\n30\n23\n31\n33\n32\n\
0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = binary_header(version)?;
    push_string(&mut bytes, version, 0, b"3DFACE")?;
    push_corners(&mut bytes, version, 1.0, 3)?;
    push_string(&mut bytes, version, 0, b"SOLID")?;
    push_corners(&mut bytes, version, 1.0, 4)?;
    push_double(&mut bytes, version, 39, -0.0)?;
    for (code, value) in [(210, 0.0), (220, 1.0), (230, 0.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"TRACE")?;
    push_corners(&mut bytes, version, 21.0, 4)?;
    binary_footer(&mut bytes, version)?;
    Ok(bytes)
}

fn nonfinite_binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = binary_header(version)?;
    push_string(&mut bytes, version, 0, b"3DFACE")?;
    push_double(&mut bytes, version, 10, f64::NAN)?;
    push_remaining_corners(&mut bytes, version, 1.0)?;
    push_string(&mut bytes, version, 0, b"SOLID")?;
    push_corners(&mut bytes, version, 1.0, 4)?;
    push_double(&mut bytes, version, 39, f64::INFINITY)?;
    push_string(&mut bytes, version, 0, b"TRACE")?;
    push_corners(&mut bytes, version, 21.0, 4)?;
    push_double(&mut bytes, version, 210, f64::INFINITY)?;
    binary_footer(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_header(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    Ok(bytes)
}

fn binary_footer(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
}

fn push_corners(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    first: f64,
    count: usize,
) -> io::Result<()> {
    let codes = [[10, 20, 30], [11, 21, 31], [12, 22, 32], [13, 23, 33]];
    for (index, corner) in codes.iter().take(count).enumerate() {
        for (axis, code) in corner.iter().enumerate() {
            push_double(bytes, version, *code, first + (index * 3 + axis) as f64)?;
        }
    }
    Ok(())
}

fn push_remaining_corners(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    first: f64,
) -> io::Result<()> {
    let codes = [[10, 20, 30], [11, 21, 31], [12, 22, 32], [13, 23, 33]];
    for (index, corner) in codes.iter().enumerate() {
        for (axis, code) in corner.iter().enumerate() {
            if index == 0 && axis == 0 {
                continue;
            }
            push_double(bytes, version, *code, first + (index * 3 + axis) as f64)?;
        }
    }
    Ok(())
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, group_code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    Ok(())
}

fn bits4(values: [[seacad_dxf_core::DxfDouble; 3]; 4]) -> [[u64; 3]; 4] {
    values.map(bits3)
}

fn bits3(values: [seacad_dxf_core::DxfDouble; 3]) -> [u64; 3] {
    values.map(seacad_dxf_core::DxfDouble::to_bits)
}

fn bits3f(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
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

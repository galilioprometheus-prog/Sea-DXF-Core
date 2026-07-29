use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfMemorySource, DxfPolylineSegmentGeometryIssue,
    DxfPolylineTransformedWcsArcSegment, DxfPolylineWcsSegmentGeometry,
    DxfPolylineWcsSegmentGeometryDirectory, DxfPolylineWcsSegmentGeometryIssue,
    DxfPolylineWcsSegmentGeometrySemantics, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum GeometryEvidence {
    TransformedLine {
        start: [u64; 3],
        end: [u64; 3],
        normal: [u64; 3],
    },
    TransformedArc {
        start: [u64; 3],
        end: [u64; 3],
        center: [u64; 3],
        normal: [u64; 3],
        radius: u64,
        sweep: u64,
        bulge: u64,
    },
    NativeLine {
        start: [u64; 3],
        end: [u64; 3],
    },
}

#[test]
fn every_supported_dialect_has_ascii_binary_wcs_geometry_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_wcs_segment_geometry_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_wcs_segment_geometry_directory(&DxfCancellationToken::default())?;

        assert_valid_directory(&ascii_directory)?;
        assert_valid_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn source_extrusion_zero_length_and_overflow_fail_typed_while_3d_ignores_extrusion()
-> Result<(), Box<dyn Error>> {
    let mut body = String::new();
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n210\n0\n220\n0\n230\n1\n",
        "70\n0\n10\n.\n20\n0\n30\n0\n42\n0\n",
        "70\n0\n10\n1\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n210\n.\n220\n0\n230\n1\n",
        "70\n0\n10\n0\n20\n0\n30\n0\n42\n0\n",
        "70\n0\n10\n1\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n210\n0\n220\n0\n230\n0\n",
        "70\n0\n10\n0\n20\n0\n30\n0\n42\n0\n",
        "70\n0\n10\n1\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n210\n1\n220\n1\n230\n1\n",
        "70\n0\n10\n-1.7e308\n20\n-1.7e308\n30\n0\n42\n0\n",
        "70\n0\n10\n0\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n8\n210\n.\n220\n.\n230\n.\n",
        "70\n32\n10\n1\n20\n2\n30\n3\n",
        "70\n32\n10\n4\n20\n5\n30\n6\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n3\n70\n0\n",
        "70\n0\n10\n1\n20\n2\n30\n99\n42\n0\n",
        "70\n0\n10\n4\n20\n5\n30\n-99\n",
    ));
    let bytes = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_wcs_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.segments().len(), 6);
    let expected = [
        DxfPolylineWcsSegmentGeometryIssue::SourceGeometry(
            DxfPolylineSegmentGeometryIssue::StartPositionUnavailable,
        ),
        DxfPolylineWcsSegmentGeometryIssue::ExtrusionUnavailable,
        DxfPolylineWcsSegmentGeometryIssue::ZeroLengthExtrusion,
        DxfPolylineWcsSegmentGeometryIssue::NonFiniteDerivedGeometry,
    ];
    for (ordinal, issue) in (0_u64..).zip(expected) {
        assert_eq!(
            directory
                .geometry_for_segment(ordinal)?
                .ok_or(io::Error::other("geometry result"))?
                .geometry(),
            Err(issue)
        );
    }
    let DxfPolylineWcsSegmentGeometry::NativeThreeDimensionalStraight(line) =
        geometry(&directory, 4)?
    else {
        return Err(io::Error::other("native 3D line").into());
    };
    assert_eq!(line.start().map(DxfDouble::to_bits), bits3([1.0, 2.0, 3.0]));
    assert_eq!(line.end().map(DxfDouble::to_bits), bits3([4.0, 5.0, 6.0]));
    let DxfPolylineWcsSegmentGeometry::TransformedTwoDimensionalStraight(line) =
        geometry(&directory, 5)?
    else {
        return Err(io::Error::other("default-extrusion 2D line").into());
    };
    assert_eq!(line.start().map(DxfDouble::to_bits), bits3([1.0, 2.0, 3.0]));
    assert_eq!(line.end().map(DxfDouble::to_bits), bits3([4.0, 5.0, 3.0]));
    assert_eq!(
        line.normal().map(DxfDouble::to_bits),
        bits3([0.0, 0.0, 1.0])
    );
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_traits_remain_bounded()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_wcs_segment_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_wcs_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.source_geometry_directory().source_id()
    );
    assert!(directory.geometry_for_segment(u64::MAX)?.is_none());
    assert_copy::<DxfPolylineTransformedWcsArcSegment>();
    assert_copy::<DxfPolylineWcsSegmentGeometrySemantics>();
    assert_send_sync::<DxfPolylineWcsSegmentGeometryDirectory>();
    Ok(())
}

fn assert_valid_directory(
    directory: &DxfPolylineWcsSegmentGeometryDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.segments().len(), 3);
    let DxfPolylineWcsSegmentGeometry::TransformedTwoDimensionalStraight(line) =
        geometry(directory, 0)?
    else {
        return Err(io::Error::other("transformed 2D line").into());
    };
    assert_eq!(
        line.start().map(DxfDouble::to_bits),
        bits3([-1.0, 3.0, 2.0])
    );
    assert_eq!(line.end().map(DxfDouble::to_bits), bits3([-4.0, 3.0, 5.0]));
    assert_eq!(
        line.normal().map(DxfDouble::to_bits),
        bits3([0.0, 1.0, 0.0])
    );

    let DxfPolylineWcsSegmentGeometry::TransformedTwoDimensionalArc(arc) = geometry(directory, 1)?
    else {
        return Err(io::Error::other("transformed 2D arc").into());
    };
    assert_eq!(arc.start().map(DxfDouble::to_bits), bits3([-4.0, 3.0, 5.0]));
    assert_eq!(arc.end().map(DxfDouble::to_bits), bits3([-6.0, 3.0, 5.0]));
    assert_eq!(
        arc.center().map(DxfDouble::to_bits),
        bits3([-5.0, 3.0, 5.0])
    );
    assert_eq!(arc.normal().map(DxfDouble::to_bits), bits3([0.0, 1.0, 0.0]));
    assert_eq!(arc.radius().to_bits(), 1.0_f64.to_bits());
    assert_eq!(arc.bulge().to_bits(), 1.0_f64.to_bits());

    let DxfPolylineWcsSegmentGeometry::NativeThreeDimensionalStraight(line) =
        geometry(directory, 2)?
    else {
        return Err(io::Error::other("native 3D line").into());
    };
    assert_eq!(line.start().map(DxfDouble::to_bits), bits3([1.0, 2.0, 3.0]));
    assert_eq!(line.end().map(DxfDouble::to_bits), bits3([4.0, 5.0, 6.0]));
    Ok(())
}

fn geometry(
    directory: &DxfPolylineWcsSegmentGeometryDirectory,
    ordinal: u64,
) -> Result<DxfPolylineWcsSegmentGeometry, Box<dyn Error>> {
    directory
        .geometry_for_segment(ordinal)?
        .ok_or(io::Error::other("geometry result"))?
        .geometry()
        .map_err(|_| io::Error::other("usable geometry").into())
}

fn evidence(
    directory: &DxfPolylineWcsSegmentGeometryDirectory,
) -> Result<Vec<GeometryEvidence>, Box<dyn Error>> {
    directory
        .segments()
        .iter()
        .map(|segment| match geometry(directory, segment.ordinal())? {
            DxfPolylineWcsSegmentGeometry::TransformedTwoDimensionalStraight(line) => {
                Ok(GeometryEvidence::TransformedLine {
                    start: line.start().map(DxfDouble::to_bits),
                    end: line.end().map(DxfDouble::to_bits),
                    normal: line.normal().map(DxfDouble::to_bits),
                })
            }
            DxfPolylineWcsSegmentGeometry::TransformedTwoDimensionalArc(arc) => {
                Ok(GeometryEvidence::TransformedArc {
                    start: arc.start().map(DxfDouble::to_bits),
                    end: arc.end().map(DxfDouble::to_bits),
                    center: arc.center().map(DxfDouble::to_bits),
                    normal: arc.normal().map(DxfDouble::to_bits),
                    radius: arc.radius().to_bits(),
                    sweep: arc.signed_sweep_radians().to_bits(),
                    bulge: arc.bulge().to_bits(),
                })
            }
            DxfPolylineWcsSegmentGeometry::NativeThreeDimensionalStraight(line) => {
                Ok(GeometryEvidence::NativeLine {
                    start: line.start().map(DxfDouble::to_bits),
                    end: line.end().map(DxfDouble::to_bits),
                })
            }
            _ => Err(io::Error::other("unknown geometry variant").into()),
        })
        .collect()
}

fn bits3(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
}

fn polyline(parent: &str, start: &str, end: &str) -> String {
    format!("0\nPOLYLINE\n{parent}0\nVERTEX\n{start}0\nVERTEX\n{end}0\nSEQEND\n")
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n0\n20\n0\n30\n3\n70\n0\n210\n0\n220\n2\n230\n0\n0\nVERTEX\n70\n0\n10\n1\n20\n2\n30\n99\n42\n0\n0\nVERTEX\n70\n0\n10\n4\n20\n5\n30\n-99\n42\n1\n0\nVERTEX\n70\n0\n10\n6\n20\n5\n30\n123\n0\nSEQEND\n0\nPOLYLINE\n10\n0\n20\n0\n30\n0\n70\n8\n0\nVERTEX\n70\n32\n10\n1\n20\n2\n30\n3\n0\nVERTEX\n70\n32\n10\n4\n20\n5\n30\n6\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_polyline(
        &mut bytes,
        version,
        0,
        3.0,
        [0.0, 2.0, 0.0],
        &[[1.0, 2.0, 99.0], [4.0, 5.0, -99.0], [6.0, 5.0, 123.0]],
        &[0.0, 1.0, 0.0],
    )?;
    push_polyline(
        &mut bytes,
        version,
        8,
        0.0,
        [0.0, 0.0, 1.0],
        &[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        &[0.0, 0.0],
    )?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_polyline(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    flags: i16,
    elevation: f64,
    extrusion: [f64; 3],
    vertices: &[[f64; 3]],
    bulges: &[f64],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"POLYLINE")?;
    for (code, value) in [
        (10, 0.0),
        (20, 0.0),
        (30, elevation),
        (210, extrusion[0]),
        (220, extrusion[1]),
        (230, extrusion[2]),
    ] {
        push_double(bytes, version, code, value)?;
    }
    push_i16(bytes, version, 70, flags)?;
    for (position, bulge) in vertices.iter().zip(bulges) {
        push_string(bytes, version, 0, b"VERTEX")?;
        push_i16(bytes, version, 70, if flags == 8 { 32 } else { 0 })?;
        for (code, value) in [
            (10, position[0]),
            (20, position[1]),
            (30, position[2]),
            (42, *bulge),
        ] {
            push_double(bytes, version, code, value)?;
        }
    }
    push_string(bytes, version, 0, b"SEQEND")
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

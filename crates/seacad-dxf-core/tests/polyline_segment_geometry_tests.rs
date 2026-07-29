use std::{error::Error, f64::consts, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfMemorySource, DxfPolylineOcsArcSegment,
    DxfPolylineSegmentGeometry, DxfPolylineSegmentGeometryDirectory,
    DxfPolylineSegmentGeometryIssue, DxfPolylineSegmentGeometrySemantics, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum GeometryEvidence {
    OcsLine {
        start: [u64; 2],
        end: [u64; 2],
        elevation: u64,
    },
    OcsArc {
        start: [u64; 2],
        end: [u64; 2],
        center: [u64; 2],
        elevation: u64,
        radius: u64,
        sweep: u64,
        bulge: u64,
    },
    WcsLine {
        start: [u64; 3],
        end: [u64; 3],
    },
}

#[test]
fn every_supported_dialect_has_ascii_binary_classic_segment_geometry_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_segment_geometry_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_segment_geometry_directory(&DxfCancellationToken::default())?;

        assert_valid_directory(&ascii_directory)?;
        assert_valid_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn unavailable_contradictory_degenerate_and_overflow_inputs_fail_typed()
-> Result<(), Box<dyn Error>> {
    let mut body = String::new();
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n",
        "70\n0\n10\n.\n20\n0\n30\n0\n42\n0\n",
        "70\n0\n10\n1\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n",
        "70\n0\n10\n0\n20\n0\n30\n0\n42\n0\n",
        "70\n0\n10\n.\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n.\n70\n0\n",
        "70\n0\n10\n0\n20\n0\n30\n0\n42\n0\n",
        "70\n0\n10\n1\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n",
        "70\n0\n10\n0\n20\n0\n30\n0\n42\n.\n",
        "70\n0\n10\n1\n20\n0\n30\n0\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n8\n",
        "70\n32\n10\n0\n20\n0\n30\n0\n42\n1\n",
        "70\n32\n10\n1\n20\n1\n30\n1\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n",
        "70\n0\n10\n1\n20\n1\n30\n99\n42\n1\n",
        "70\n0\n10\n1\n20\n1\n30\n-99\n",
    ));
    body.push_str(&polyline(
        "10\n0\n20\n0\n30\n0\n70\n0\n",
        "70\n0\n10\n-1e308\n20\n0\n30\n0\n42\n1\n",
        "70\n0\n10\n1e308\n20\n0\n30\n0\n",
    ));
    let bytes = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n").into_bytes();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.segments().len(), 7);
    let expected = [
        DxfPolylineSegmentGeometryIssue::StartPositionUnavailable,
        DxfPolylineSegmentGeometryIssue::EndPositionUnavailable,
        DxfPolylineSegmentGeometryIssue::ElevationUnavailable,
        DxfPolylineSegmentGeometryIssue::BulgeUnavailable,
        DxfPolylineSegmentGeometryIssue::BulgeUnsupportedForThreeDimensional,
        DxfPolylineSegmentGeometryIssue::DegenerateArcChord,
        DxfPolylineSegmentGeometryIssue::NonFiniteDerivedGeometry,
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
        document.polyline_segment_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.segment_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.segment_semantic_directory().source_id()
    );
    assert!(directory.geometry_for_segment(u64::MAX)?.is_none());
    assert_copy::<DxfPolylineOcsArcSegment>();
    assert_copy::<DxfPolylineSegmentGeometrySemantics>();
    assert_send_sync::<DxfPolylineSegmentGeometryDirectory>();
    Ok(())
}

fn assert_valid_directory(
    directory: &DxfPolylineSegmentGeometryDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.segments().len(), 3);
    let DxfPolylineSegmentGeometry::TwoDimensionalStraight(line) = geometry(directory, 0)? else {
        return Err(io::Error::other("2D line").into());
    };
    assert_eq!(line.start().map(DxfDouble::to_bits), bits2([0.0, 0.0]));
    assert_eq!(line.end().map(DxfDouble::to_bits), bits2([2.0, 0.0]));
    assert_eq!(line.elevation().to_bits(), 7.0_f64.to_bits());

    let DxfPolylineSegmentGeometry::TwoDimensionalArc(arc) = geometry(directory, 1)? else {
        return Err(io::Error::other("2D arc").into());
    };
    assert_eq!(arc.center().map(DxfDouble::to_bits), bits2([3.0, 0.0]));
    assert_eq!(arc.elevation().to_bits(), 7.0_f64.to_bits());
    assert_eq!(arc.radius().to_bits(), 1.0_f64.to_bits());
    assert_eq!(arc.bulge().to_bits(), 1.0_f64.to_bits());
    assert_close(arc.signed_sweep_radians().to_f64(), consts::PI, 0.0)?;

    let DxfPolylineSegmentGeometry::ThreeDimensionalStraight(line) = geometry(directory, 2)? else {
        return Err(io::Error::other("3D line").into());
    };
    assert_eq!(line.start().map(DxfDouble::to_bits), bits3([1.0, 2.0, 3.0]));
    assert_eq!(line.end().map(DxfDouble::to_bits), bits3([4.0, 5.0, 6.0]));
    Ok(())
}

fn geometry(
    directory: &DxfPolylineSegmentGeometryDirectory,
    ordinal: u64,
) -> Result<DxfPolylineSegmentGeometry, Box<dyn Error>> {
    directory
        .geometry_for_segment(ordinal)?
        .ok_or(io::Error::other("geometry result"))?
        .geometry()
        .map_err(|_| io::Error::other("usable geometry").into())
}

fn evidence(
    directory: &DxfPolylineSegmentGeometryDirectory,
) -> Result<Vec<GeometryEvidence>, Box<dyn Error>> {
    directory
        .segments()
        .iter()
        .map(|segment| match geometry(directory, segment.ordinal())? {
            DxfPolylineSegmentGeometry::TwoDimensionalStraight(line) => {
                Ok(GeometryEvidence::OcsLine {
                    start: line.start().map(DxfDouble::to_bits),
                    end: line.end().map(DxfDouble::to_bits),
                    elevation: line.elevation().to_bits(),
                })
            }
            DxfPolylineSegmentGeometry::TwoDimensionalArc(arc) => Ok(GeometryEvidence::OcsArc {
                start: arc.start().map(DxfDouble::to_bits),
                end: arc.end().map(DxfDouble::to_bits),
                center: arc.center().map(DxfDouble::to_bits),
                elevation: arc.elevation().to_bits(),
                radius: arc.radius().to_bits(),
                sweep: arc.signed_sweep_radians().to_bits(),
                bulge: arc.bulge().to_bits(),
            }),
            DxfPolylineSegmentGeometry::ThreeDimensionalStraight(line) => {
                Ok(GeometryEvidence::WcsLine {
                    start: line.start().map(DxfDouble::to_bits),
                    end: line.end().map(DxfDouble::to_bits),
                })
            }
            _ => Err(io::Error::other("unknown geometry variant").into()),
        })
        .collect()
}

fn bits2(values: [f64; 2]) -> [u64; 2] {
    values.map(f64::to_bits)
}
fn bits3(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) -> Result<(), io::Error> {
    if (actual - expected).abs() <= tolerance {
        Ok(())
    } else {
        Err(io::Error::other("floating geometry mismatch"))
    }
}

fn polyline(parent: &str, start: &str, end: &str) -> String {
    format!("0\nPOLYLINE\n{parent}0\nVERTEX\n{start}0\nVERTEX\n{end}0\nSEQEND\n")
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n0\n20\n0\n30\n7\n70\n0\n0\nVERTEX\n70\n0\n10\n0\n20\n0\n30\n99\n42\n0\n0\nVERTEX\n70\n0\n10\n2\n20\n0\n30\n-99\n42\n1\n0\nVERTEX\n70\n0\n10\n4\n20\n0\n30\n123\n0\nSEQEND\n0\nPOLYLINE\n10\n0\n20\n0\n30\n0\n70\n8\n0\nVERTEX\n70\n32\n10\n1\n20\n2\n30\n3\n0\nVERTEX\n70\n32\n10\n4\n20\n5\n30\n6\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n").into_bytes()
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
        7.0,
        &[[0.0, 0.0, 99.0], [2.0, 0.0, -99.0], [4.0, 0.0, 123.0]],
        &[0.0, 1.0, 0.0],
    )?;
    push_polyline(
        &mut bytes,
        version,
        8,
        0.0,
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
    vertices: &[[f64; 3]],
    bulges: &[f64],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"POLYLINE")?;
    for (code, value) in [(10, 0.0), (20, 0.0), (30, elevation)] {
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

use std::{error::Error, f64::consts, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfLightweightPolylineOcsArcSegment,
    DxfLightweightPolylineOcsSegmentGeometry, DxfLightweightPolylineSegmentGeometryDirectory,
    DxfLightweightPolylineSegmentGeometryIssue, DxfLightweightPolylineSegmentGeometrySemantics,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum GeometryEvidence {
    Straight {
        start: [u64; 2],
        end: [u64; 2],
    },
    Arc {
        start: [u64; 2],
        end: [u64; 2],
        center: [u64; 2],
        radius: u64,
        sweep: u64,
        bulge: u64,
    },
}

#[test]
fn every_supported_dialect_has_ascii_binary_ocs_geometry_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii
            .lightweight_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary
            .lightweight_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;

        assert_valid_directory(&ascii_directory)?;
        assert_valid_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn quarter_arc_unavailable_inputs_degenerate_chords_and_overflow_fail_typed()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n10\n0\n20\n0\n42\n0.41421356237309503\n10\n2\n20\n0\n0\nLWPOLYLINE\n10\n.\n20\n0\n42\n0\n10\n1\n20\n0\n0\nLWPOLYLINE\n10\n0\n20\n0\n42\n0\n10\n1\n0\nLWPOLYLINE\n10\n0\n20\n0\n42\n.\n10\n1\n20\n0\n0\nLWPOLYLINE\n10\n1\n20\n1\n42\n1\n10\n1\n20\n1\n0\nLWPOLYLINE\n10\n-1e308\n20\n0\n42\n1\n10\n1e308\n20\n0\n0\nLWPOLYLINE\n10\n2\n20\n2\n42\n-0\n10\n2\n20\n2\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document
        .lightweight_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.segments().len(), 7);

    let quarter = geometry(&directory, 0)?;
    let DxfLightweightPolylineOcsSegmentGeometry::Arc(quarter) = quarter else {
        return Err(io::Error::other("quarter arc").into());
    };
    assert_close(quarter.center()[0].to_f64(), 1.0, 1.0e-12)?;
    assert_close(quarter.center()[1].to_f64(), 1.0, 1.0e-12)?;
    assert_close(quarter.radius().to_f64(), 2.0_f64.sqrt(), 1.0e-12)?;
    assert_close(
        quarter.signed_sweep_radians().to_f64(),
        consts::FRAC_PI_2,
        1.0e-12,
    )?;

    let expected = [
        DxfLightweightPolylineSegmentGeometryIssue::StartPositionUnavailable,
        DxfLightweightPolylineSegmentGeometryIssue::EndPositionUnavailable,
        DxfLightweightPolylineSegmentGeometryIssue::BulgeUnavailable,
        DxfLightweightPolylineSegmentGeometryIssue::DegenerateArcChord,
        DxfLightweightPolylineSegmentGeometryIssue::NonFiniteDerivedGeometry,
    ];
    for (ordinal, issue) in (1_u64..=5).zip(expected) {
        assert_eq!(
            directory
                .geometry_for_segment(ordinal)?
                .ok_or(io::Error::other("invalid geometry result"))?
                .geometry(),
            Err(issue)
        );
    }
    assert!(matches!(
        geometry(&directory, 6)?,
        DxfLightweightPolylineOcsSegmentGeometry::Straight(_)
    ));
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
        document.lightweight_polyline_segment_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .lightweight_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert!(directory.geometry_for_segment(u64::MAX)?.is_none());
    assert_copy::<DxfLightweightPolylineOcsArcSegment>();
    assert_copy::<DxfLightweightPolylineSegmentGeometrySemantics>();
    assert_send_sync::<DxfLightweightPolylineSegmentGeometryDirectory>();
    Ok(())
}

fn assert_valid_directory(
    directory: &DxfLightweightPolylineSegmentGeometryDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        directory.source_id(),
        directory.segment_directory().source_id()
    );
    assert_eq!(directory.segments().len(), 3);
    let straight = geometry(directory, 0)?;
    let DxfLightweightPolylineOcsSegmentGeometry::Straight(straight) = straight else {
        return Err(io::Error::other("straight segment").into());
    };
    assert_eq!(
        straight.start().map(DxfDouble::to_bits),
        [0.0_f64.to_bits(), 0.0_f64.to_bits()]
    );
    assert_eq!(
        straight.end().map(DxfDouble::to_bits),
        [2.0_f64.to_bits(), 0.0_f64.to_bits()]
    );

    let positive = arc(directory, 1)?;
    assert_eq!(
        positive.center().map(DxfDouble::to_bits),
        [3.0_f64.to_bits(), 0.0_f64.to_bits()]
    );
    assert_eq!(positive.radius().to_bits(), 1.0_f64.to_bits());
    assert_eq!(positive.bulge().to_bits(), 1.0_f64.to_bits());
    assert_close(positive.signed_sweep_radians().to_f64(), consts::PI, 0.0)?;

    let negative = arc(directory, 2)?;
    assert_eq!(
        negative.center().map(DxfDouble::to_bits),
        [5.0_f64.to_bits(), 0.0_f64.to_bits()]
    );
    assert_eq!(negative.radius().to_bits(), 1.0_f64.to_bits());
    assert_eq!(negative.bulge().to_bits(), (-1.0_f64).to_bits());
    assert_close(negative.signed_sweep_radians().to_f64(), -consts::PI, 0.0)?;
    Ok(())
}

fn geometry(
    directory: &DxfLightweightPolylineSegmentGeometryDirectory,
    ordinal: u64,
) -> Result<DxfLightweightPolylineOcsSegmentGeometry, Box<dyn Error>> {
    directory
        .geometry_for_segment(ordinal)?
        .ok_or(io::Error::other("geometry result"))?
        .geometry()
        .map_err(|_| io::Error::other("usable geometry").into())
}

fn arc(
    directory: &DxfLightweightPolylineSegmentGeometryDirectory,
    ordinal: u64,
) -> Result<DxfLightweightPolylineOcsArcSegment, Box<dyn Error>> {
    let DxfLightweightPolylineOcsSegmentGeometry::Arc(arc) = geometry(directory, ordinal)? else {
        return Err(io::Error::other("arc segment").into());
    };
    Ok(arc)
}

fn evidence(
    directory: &DxfLightweightPolylineSegmentGeometryDirectory,
) -> Result<Vec<GeometryEvidence>, Box<dyn Error>> {
    directory
        .segments()
        .iter()
        .map(|segment| match geometry(directory, segment.ordinal())? {
            DxfLightweightPolylineOcsSegmentGeometry::Straight(line) => {
                Ok(GeometryEvidence::Straight {
                    start: line.start().map(DxfDouble::to_bits),
                    end: line.end().map(DxfDouble::to_bits),
                })
            }
            DxfLightweightPolylineOcsSegmentGeometry::Arc(arc) => Ok(GeometryEvidence::Arc {
                start: arc.start().map(DxfDouble::to_bits),
                end: arc.end().map(DxfDouble::to_bits),
                center: arc.center().map(DxfDouble::to_bits),
                radius: arc.radius().to_bits(),
                sweep: arc.signed_sweep_radians().to_bits(),
                bulge: arc.bulge().to_bits(),
            }),
            _ => Err(io::Error::other("unknown geometry variant").into()),
        })
        .collect()
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) -> Result<(), io::Error> {
    if (actual - expected).abs() <= tolerance {
        Ok(())
    } else {
        Err(io::Error::other("floating geometry mismatch"))
    }
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n10\n0\n20\n0\n42\n0\n10\n2\n20\n0\n42\n1\n10\n4\n20\n0\n42\n-1\n10\n6\n20\n0\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [
        (10, 0.0),
        (20, 0.0),
        (42, 0.0),
        (10, 2.0),
        (20, 0.0),
        (42, 1.0),
        (10, 4.0),
        (20, 0.0),
        (42, -1.0),
        (10, 6.0),
        (20, 0.0),
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

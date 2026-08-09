use std::{error::Error, f64::consts, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchPolylineBulgeIssue, DxfHatchPolylineOcsArcSegment, DxfHatchPolylineOcsSegmentGeometry,
    DxfHatchPolylineSegmentGeometryDirectory, DxfHatchPolylineSegmentGeometryEntry,
    DxfHatchPolylineSegmentGeometryIssue, DxfHatchPolylineVertexNumericIssue,
    DxfHatchPolylineVertexPositionIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
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
fn every_dialect_has_ascii_binary_straight_and_signed_arc_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii(
            version,
            2,
            true,
            false,
            "10\n0\n20\n0\n42\n0\n10\n2\n20\n0\n42\n1\n10\n4\n20\n0\n42\n-1\n10\n6\n20\n0\n",
            4,
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
        let binary = binary(
            version,
            2,
            true,
            false,
            4,
            &[
                (
                    0.0_f64.to_bits(),
                    0.0_f64.to_bits(),
                    Some(0.0_f64.to_bits()),
                ),
                (
                    2.0_f64.to_bits(),
                    0.0_f64.to_bits(),
                    Some(1.0_f64.to_bits()),
                ),
                (
                    4.0_f64.to_bits(),
                    0.0_f64.to_bits(),
                    Some((-1.0_f64).to_bits()),
                ),
                (6.0_f64.to_bits(), 0.0_f64.to_bits(), None),
            ],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.entries().len(), 3);
            assert_eq!(
                directory.entries().len(),
                directory.line_geometry_directory().entries().len()
            );
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));

            let DxfHatchPolylineOcsSegmentGeometry::Straight(line) =
                geometry(directory.entries()[0])?
            else {
                return Err(io::Error::other("straight geometry").into());
            };
            assert_eq!(
                line.values().map(|point| point.map(DxfDouble::to_bits)),
                [
                    [0.0_f64.to_bits(), 0.0_f64.to_bits()],
                    [2.0_f64.to_bits(), 0.0_f64.to_bits()]
                ]
            );

            let positive = arc(directory.entries()[1])?;
            assert_eq!(
                positive.center().map(DxfDouble::to_bits),
                [3.0_f64.to_bits(), 0.0_f64.to_bits()]
            );
            assert_eq!(positive.radius().to_bits(), 1.0_f64.to_bits());
            assert_eq!(positive.bulge().to_bits(), 1.0_f64.to_bits());
            assert_close(positive.signed_sweep_radians().to_f64(), consts::PI, 0.0)?;

            let negative = arc(directory.entries()[2])?;
            assert_eq!(
                negative.center().map(DxfDouble::to_bits),
                [5.0_f64.to_bits(), 0.0_f64.to_bits()]
            );
            assert_eq!(negative.radius().to_bits(), 1.0_f64.to_bits());
            assert_eq!(negative.bulge().to_bits(), (-1.0_f64).to_bits());
            assert_close(negative.signed_sweep_radians().to_f64(), -consts::PI, 0.0)?;
        }
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn quarter_arcs_and_typed_shape_or_endpoint_failures_hold() -> Result<(), Box<dyn Error>> {
    let quarter = ascii(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        "10\n0\n20\n0\n42\n0.41421356237309503\n10\n2\n20\n0\n",
        2,
    );
    let source = DxfMemorySource::new(&quarter, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    let quarter = arc(directory.entries()[0])?;
    assert_close(quarter.center()[0].to_f64(), 1.0, 1.0e-12)?;
    assert_close(quarter.center()[1].to_f64(), 1.0, 1.0e-12)?;
    assert_close(quarter.radius().to_f64(), 2.0_f64.sqrt(), 1.0e-12)?;
    assert_close(
        quarter.signed_sweep_radians().to_f64(),
        consts::FRAC_PI_2,
        1.0e-12,
    )?;

    let indeterminate = ascii(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        "10\nbad\n20\n0\n42\nbad\n10\n2\n20\n0\n",
        2,
    );
    let source = DxfMemorySource::new(&indeterminate, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].geometry(),
        Err(DxfHatchPolylineSegmentGeometryIssue::ShapeIndeterminate(
            DxfHatchPolylineBulgeIssue::Numeric(
                DxfHatchPolylineVertexNumericIssue::InvalidAsciiNumber(
                    DxfAsciiNumericIssue::InvalidSyntax { .. }
                )
            )
        ))
    ));

    let start_invalid = ascii(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        "10\nbad\n20\n0\n42\n0.5\n10\n2\n20\n0\n",
        2,
    );
    let source = DxfMemorySource::new(&start_invalid, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].geometry(),
        Err(DxfHatchPolylineSegmentGeometryIssue::StartPositionUnavailable(
            DxfHatchPolylineVertexPositionIssue::CoordinatesUnavailable(mask)
        )) if mask.x() && !mask.y()
    ));

    let end_invalid = ascii(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        "10\n0\n20\n0\n42\n0.5\n10\n2\n",
        2,
    );
    let source = DxfMemorySource::new(&end_invalid, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].geometry(),
        Err(DxfHatchPolylineSegmentGeometryIssue::EndPositionUnavailable(
            DxfHatchPolylineVertexPositionIssue::CoordinatesUnavailable(mask)
        )) if !mask.x() && mask.y()
    ));
    Ok(())
}

#[test]
fn degenerate_overflow_and_underflow_arcs_fail_typed_but_zero_line_holds()
-> Result<(), Box<dyn Error>> {
    for (payload, expected) in [
        (
            "10\n1\n20\n1\n42\n1\n10\n1\n20\n1\n",
            DxfHatchPolylineSegmentGeometryIssue::DegenerateArcChord,
        ),
        (
            "10\n-1e308\n20\n0\n42\n1\n10\n1e308\n20\n0\n",
            DxfHatchPolylineSegmentGeometryIssue::NonFiniteDerivedGeometry,
        ),
        (
            "10\n0\n20\n0\n42\n5e-324\n10\n1\n20\n0\n",
            DxfHatchPolylineSegmentGeometryIssue::NonFiniteDerivedGeometry,
        ),
    ] {
        let bytes = ascii(DxfAcadVersion::Ac1032, 2, true, false, payload, 2);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = open_ascii(&source)?
            .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
        assert_eq!(directory.entries()[0].geometry(), Err(expected));
    }

    let zero_line = ascii(
        DxfAcadVersion::Ac1032,
        2,
        false,
        false,
        "10\n1\n20\n2\n10\n1\n20\n2\n",
        2,
    );
    let source = DxfMemorySource::new(&zero_line, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    let DxfHatchPolylineOcsSegmentGeometry::Straight(line) = geometry(directory.entries()[0])?
    else {
        return Err(io::Error::other("zero-length straight geometry").into());
    };
    assert_eq!(line.start(), line.end());
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_redaction_and_empty_paths_hold() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        "10\n12345.625\n20\n0\n42\n1\n10\n12347.625\n20\n0\n",
        2,
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_polyline_segment_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.line_geometry_directory().source_id()
    );
    assert!(size_of::<DxfHatchPolylineSegmentGeometryEntry>() <= 192);
    assert!(!format!("{:?}", directory.entries()[0]).contains("12345.625"));
    copy::<DxfHatchPolylineOcsArcSegment>();
    copy::<DxfHatchPolylineSegmentGeometryEntry>();
    send_sync::<DxfHatchPolylineSegmentGeometryDirectory>();

    for (path_flag, payload, declared) in [
        (2, "10\n1\n20\n2\n", 2),
        (0, "", 0),
        (2, "72\n.\n73\n0\n93\n0\n", 0),
    ] {
        let bytes = ascii(
            DxfAcadVersion::Ac1032,
            path_flag,
            false,
            false,
            payload,
            declared,
        );
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = open_ascii(&source)?
            .hatch_polyline_segment_geometry_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    }
    Ok(())
}

fn geometry(
    entry: DxfHatchPolylineSegmentGeometryEntry,
) -> Result<DxfHatchPolylineOcsSegmentGeometry, io::Error> {
    entry
        .geometry()
        .map_err(|_| io::Error::other("usable segment geometry"))
}

fn arc(
    entry: DxfHatchPolylineSegmentGeometryEntry,
) -> Result<DxfHatchPolylineOcsArcSegment, io::Error> {
    let DxfHatchPolylineOcsSegmentGeometry::Arc(arc) = geometry(entry)? else {
        return Err(io::Error::other("arc segment geometry"));
    };
    Ok(arc)
}

fn evidence(
    directory: &DxfHatchPolylineSegmentGeometryDirectory,
) -> Result<Vec<GeometryEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| match geometry(entry)? {
            DxfHatchPolylineOcsSegmentGeometry::Straight(line) => Ok(GeometryEvidence::Straight {
                start: line.start().values().map(DxfDouble::to_bits),
                end: line.end().values().map(DxfDouble::to_bits),
            }),
            DxfHatchPolylineOcsSegmentGeometry::Arc(arc) => Ok(GeometryEvidence::Arc {
                start: arc.start().values().map(DxfDouble::to_bits),
                end: arc.end().values().map(DxfDouble::to_bits),
                center: arc.center().map(DxfDouble::to_bits),
                radius: arc.radius().to_bits(),
                sweep: arc.signed_sweep_radians().to_bits(),
                bulge: arc.bulge().to_bits(),
            }),
            _ => Err(io::Error::other("unknown segment geometry")),
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

fn ascii(
    version: DxfAcadVersion,
    path_flag: i32,
    has_bulge: bool,
    closed: bool,
    payload: &str,
    declared: u32,
) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n{}\n72\n{}\n73\n{}\n93\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        path_flag,
        i16::from(has_bulge),
        i16::from(closed),
        declared,
        payload
    )
    .into_bytes()
}

fn binary(
    version: DxfAcadVersion,
    path_flag: i32,
    has_bulge: bool,
    closed: bool,
    declared: i32,
    vertices: &[(u64, u64, Option<u64>)],
) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, path_flag)?;
    push_i16(&mut bytes, version, 72, i16::from(has_bulge))?;
    push_i16(&mut bytes, version, 73, i16::from(closed))?;
    push_i32(&mut bytes, version, 93, declared)?;
    for (x, y, bulge) in vertices.iter().copied() {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
        if let Some(bits) = bulge {
            push_double(&mut bytes, version, 42, bits)?;
        }
    }
    push_i32(&mut bytes, version, 97, 0)?;
    push_i16(&mut bytes, version, 75, 0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
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

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    bits: u64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&bits.to_le_bytes());
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

fn copy<T: Copy>() {}
fn send_sync<T: Send + Sync>() {}

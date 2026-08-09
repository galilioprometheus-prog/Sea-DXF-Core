use std::{error::Error, f64::consts, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfHatchElevationIssue, DxfHatchExtrusionIssue,
    DxfHatchPolylineSegmentGeometryIssue, DxfHatchPolylineWcsArcSegment,
    DxfHatchPolylineWcsGeometryDirectory, DxfHatchPolylineWcsGeometryEntry,
    DxfHatchPolylineWcsGeometryIssue, DxfHatchPolylineWcsSegmentGeometry, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum GeometryEvidence {
    Straight {
        start: [u64; 3],
        end: [u64; 3],
        normal: [u64; 3],
    },
    Arc {
        start: [u64; 3],
        end: [u64; 3],
        center: [u64; 3],
        normal: [u64; 3],
        radius: u64,
        sweep: u64,
        bulge: u64,
    },
}

#[test]
fn every_dialect_has_ascii_binary_default_normal_wcs_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii(
            version,
            "10\n0\n20\n-0\n30\n5\n",
            "",
            2,
            true,
            "10\n0\n20\n0\n42\n0\n10\n2\n20\n0\n42\n1\n10\n4\n20\n0\n",
            3,
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_polyline_wcs_geometry_directory(&DxfCancellationToken::default())?;
        let binary = binary(
            version,
            [0.0_f64.to_bits(), (-0.0_f64).to_bits(), 5.0_f64.to_bits()],
            None,
            2,
            true,
            3,
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
                (4.0_f64.to_bits(), 0.0_f64.to_bits(), None),
            ],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_polyline_wcs_geometry_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.entries().len(), 2);
            assert_eq!(
                directory.entries().len(),
                directory.source_geometry_directory().entries().len()
            );
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            let DxfHatchPolylineWcsSegmentGeometry::Straight(line) =
                geometry(directory.entries()[0])?
            else {
                return Err(io::Error::other("WCS line").into());
            };
            assert_eq!(
                line.start().map(DxfDouble::to_bits),
                [0.0_f64.to_bits(), 0.0_f64.to_bits(), 5.0_f64.to_bits()]
            );
            assert_eq!(
                line.end().map(DxfDouble::to_bits),
                [2.0_f64.to_bits(), 0.0_f64.to_bits(), 5.0_f64.to_bits()]
            );
            assert_eq!(
                line.normal().map(DxfDouble::to_bits),
                [0.0_f64.to_bits(), 0.0_f64.to_bits(), 1.0_f64.to_bits()]
            );
            let arc = arc(directory.entries()[1])?;
            assert_eq!(
                arc.center().map(DxfDouble::to_bits),
                [3.0_f64.to_bits(), 0.0_f64.to_bits(), 5.0_f64.to_bits()]
            );
            assert_eq!(arc.radius().to_bits(), 1.0_f64.to_bits());
            assert_close(arc.signed_sweep_radians().to_f64(), consts::PI, 0.0)?;
            assert_eq!(arc.bulge().to_bits(), 1.0_f64.to_bits());
        }
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn arbitrary_axis_and_negative_normal_projection_preserve_arc_invariants()
-> Result<(), Box<dyn Error>> {
    let arbitrary = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n5\n",
        "210\n0\n220\n1\n230\n0\n",
        2,
        true,
        "10\n1\n20\n2\n42\n1\n10\n3\n20\n2\n",
        2,
    );
    let source = DxfMemorySource::new(&arbitrary, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_wcs_geometry_directory(&DxfCancellationToken::default())?;
    let arc = arc(directory.entries()[0])?;
    assert_vec_close(arc.start(), [-1.0, 5.0, 2.0], 0.0)?;
    assert_vec_close(arc.end(), [-3.0, 5.0, 2.0], 0.0)?;
    assert_vec_close(arc.center(), [-2.0, 5.0, 2.0], 0.0)?;
    assert_vec_close(arc.normal(), [0.0, 1.0, 0.0], 0.0)?;
    assert_eq!(arc.radius().to_bits(), 1.0_f64.to_bits());
    assert_close(arc.signed_sweep_radians().to_f64(), consts::PI, 0.0)?;

    let negative = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n5\n",
        "210\n0\n220\n0\n230\n-1\n",
        2,
        false,
        "10\n1\n20\n2\n10\n3\n20\n2\n",
        2,
    );
    let source = DxfMemorySource::new(&negative, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_wcs_geometry_directory(&DxfCancellationToken::default())?;
    let DxfHatchPolylineWcsSegmentGeometry::Straight(line) = geometry(directory.entries()[0])?
    else {
        return Err(io::Error::other("negative-normal line").into());
    };
    assert_vec_close(line.start(), [-1.0, 2.0, -5.0], 0.0)?;
    assert_vec_close(line.end(), [-3.0, 2.0, -5.0], 0.0)?;
    assert_vec_close(line.normal(), [0.0, 0.0, -1.0], 0.0)?;
    Ok(())
}

#[test]
fn source_elevation_extrusion_and_transform_failures_follow_precedence()
-> Result<(), Box<dyn Error>> {
    let source_failure = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n",
        "210\n0\n220\n0\n230\n0\n",
        2,
        true,
        "10\n1\n20\n1\n42\n1\n10\n1\n20\n1\n",
        2,
    );
    assert_eq!(
        only_result(&source_failure)?,
        Err(DxfHatchPolylineWcsGeometryIssue::SourceGeometry(
            DxfHatchPolylineSegmentGeometryIssue::DegenerateArcChord
        ))
    );

    let elevation_failure = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n",
        "210\n0\n220\n0\n230\n0\n",
        2,
        false,
        "10\n0\n20\n0\n10\n1\n20\n0\n",
        2,
    );
    assert!(matches!(
        only_result(&elevation_failure)?,
        Err(DxfHatchPolylineWcsGeometryIssue::ElevationUnavailable(
            DxfHatchElevationIssue::ComponentsUnavailable(mask)
        )) if !mask.x() && !mask.y() && mask.z()
    ));

    let extrusion_failure = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n1\n",
        "210\n0\n220\n-0\n230\n0\n",
        2,
        false,
        "10\n0\n20\n0\n10\n1\n20\n0\n",
        2,
    );
    assert_eq!(
        only_result(&extrusion_failure)?,
        Err(DxfHatchPolylineWcsGeometryIssue::ExtrusionUnavailable(
            DxfHatchExtrusionIssue::ZeroVector
        ))
    );

    let transform_failure = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n1.5e308\n",
        "210\n1\n220\n0\n230\n1\n",
        2,
        false,
        "10\n0\n20\n1.5e308\n10\n1\n20\n1.5e308\n",
        2,
    );
    assert_eq!(
        only_result(&transform_failure)?,
        Err(DxfHatchPolylineWcsGeometryIssue::NonFiniteDerivedGeometry)
    );
    Ok(())
}

#[test]
fn cancellation_bounds_identity_receipts_redaction_traits_and_empty_paths_hold()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n5\n",
        "",
        2,
        true,
        "10\n12345.625\n20\n0\n42\n1\n10\n12347.625\n20\n0\n",
        2,
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_polyline_wcs_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.hatch_polyline_wcs_geometry_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one WCS entry").into());
    };
    assert_eq!(directory.entry(0), Some(*entry));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.source_geometry_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.elevation_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.extrusion_directory().source_id()
    );
    assert_eq!(
        directory
            .elevation_for_entry(*entry)
            .ok_or(io::Error::other("elevation receipt"))?
            .subclass_ordinal(),
        entry.subclass_ordinal()
    );
    assert_eq!(
        directory
            .extrusion_for_entry(*entry)
            .ok_or(io::Error::other("extrusion receipt"))?
            .scalar_entry()
            .subclass_ordinal(),
        entry.subclass_ordinal()
    );
    assert!(size_of::<DxfHatchPolylineWcsGeometryEntry>() <= 320);
    assert!(!format!("{entry:?}").contains("12345.625"));
    copy::<DxfHatchPolylineWcsArcSegment>();
    copy::<DxfHatchPolylineWcsGeometryEntry>();
    send_sync::<DxfHatchPolylineWcsGeometryDirectory>();

    for (path_flag, payload, declared) in [
        (2, "10\n1\n20\n2\n", 2),
        (0, "", 0),
        (2, "72\n.\n73\n0\n93\n0\n", 0),
    ] {
        let bytes = ascii(
            DxfAcadVersion::Ac1032,
            "10\n0\n20\n0\n30\n5\n",
            "",
            path_flag,
            false,
            payload,
            declared,
        );
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = open_ascii(&source)?
            .hatch_polyline_wcs_geometry_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    }
    Ok(())
}

fn only_result(
    bytes: &[u8],
) -> Result<
    Result<DxfHatchPolylineWcsSegmentGeometry, DxfHatchPolylineWcsGeometryIssue>,
    Box<dyn Error>,
> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_wcs_geometry_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one WCS result").into());
    };
    Ok(entry.geometry())
}

fn geometry(
    entry: DxfHatchPolylineWcsGeometryEntry,
) -> Result<DxfHatchPolylineWcsSegmentGeometry, io::Error> {
    entry
        .geometry()
        .map_err(|_| io::Error::other("usable WCS geometry"))
}

fn arc(
    entry: DxfHatchPolylineWcsGeometryEntry,
) -> Result<DxfHatchPolylineWcsArcSegment, io::Error> {
    let DxfHatchPolylineWcsSegmentGeometry::Arc(arc) = geometry(entry)? else {
        return Err(io::Error::other("WCS arc"));
    };
    Ok(arc)
}

fn evidence(
    directory: &DxfHatchPolylineWcsGeometryDirectory,
) -> Result<Vec<GeometryEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| match geometry(entry)? {
            DxfHatchPolylineWcsSegmentGeometry::Straight(line) => Ok(GeometryEvidence::Straight {
                start: line.start().map(DxfDouble::to_bits),
                end: line.end().map(DxfDouble::to_bits),
                normal: line.normal().map(DxfDouble::to_bits),
            }),
            DxfHatchPolylineWcsSegmentGeometry::Arc(arc) => Ok(GeometryEvidence::Arc {
                start: arc.start().map(DxfDouble::to_bits),
                end: arc.end().map(DxfDouble::to_bits),
                center: arc.center().map(DxfDouble::to_bits),
                normal: arc.normal().map(DxfDouble::to_bits),
                radius: arc.radius().to_bits(),
                sweep: arc.signed_sweep_radians().to_bits(),
                bulge: arc.bulge().to_bits(),
            }),
            _ => Err(io::Error::other("unknown WCS geometry")),
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

fn assert_vec_close(
    actual: [DxfDouble; 3],
    expected: [f64; 3],
    tolerance: f64,
) -> Result<(), io::Error> {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert_close(actual.to_f64(), expected, tolerance)?;
    }
    Ok(())
}

fn ascii(
    version: DxfAcadVersion,
    elevation: &str,
    extrusion: &str,
    path_flag: i32,
    has_bulge: bool,
    payload: &str,
    declared: u32,
) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}{}91\n1\n92\n{}\n72\n{}\n73\n0\n93\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), elevation, extrusion, path_flag, i16::from(has_bulge), declared, payload
    )
    .into_bytes()
}

fn binary(
    version: DxfAcadVersion,
    elevation: [u64; 3],
    extrusion: Option<[u64; 3]>,
    path_flag: i32,
    has_bulge: bool,
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
    for (code, bits) in [(10, elevation[0]), (20, elevation[1]), (30, elevation[2])] {
        push_double(&mut bytes, version, code, bits)?;
    }
    if let Some(values) = extrusion {
        for (code, bits) in [(210, values[0]), (220, values[1]), (230, values[2])] {
            push_double(&mut bytes, version, code, bits)?;
        }
    }
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, path_flag)?;
    push_i16(&mut bytes, version, 72, i16::from(has_bulge))?;
    push_i16(&mut bytes, version, 73, 0)?;
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

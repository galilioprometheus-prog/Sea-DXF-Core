use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfHatchBoundaryCircularArcEdgeDirection,
    DxfHatchBoundaryCircularArcEdgeGeometryIssue,
    DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory,
    DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry,
    DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue, DxfHatchBoundaryCircularArcEdgeWcsSegment,
    DxfHatchElevationIssue, DxfHatchExtrusionIssue, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

type WcsEvidence = ([[u64; 3]; 4], [u64; 3], i16);

#[test]
fn every_dialect_has_ascii_binary_default_frame_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii(
            version,
            "10\n0\n20\n-0\n30\n5\n",
            "",
            2,
            1,
            "10\n0\n20\n0\n40\n2\n50\n-450\n51\n810\n73\n1\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_circular_arc_edge_wcs_geometry_directory(
                &DxfCancellationToken::default(),
            )?;
        let binary = binary(
            version,
            [0.0_f64.to_bits(), (-0.0_f64).to_bits(), 5.0_f64.to_bits()],
            None,
            1,
            ArcWire {
                doubles: [
                    0.0_f64.to_bits(),
                    0.0_f64.to_bits(),
                    2.0_f64.to_bits(),
                    (-450.0_f64).to_bits(),
                    810.0_f64.to_bits(),
                ],
                direction: 1,
            },
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_circular_arc_edge_wcs_geometry_directory(
                &DxfCancellationToken::default(),
            )?;

        for directory in [&ascii_directory, &binary_directory] {
            let [entry] = directory.entries() else {
                return Err(io::Error::other("one WCS CircularArc edge").into());
            };
            assert_eq!(entry.ordinal(), 0);
            assert_eq!(entry.subclass_ordinal(), 0);
            assert_eq!(entry.source().semantics().numeric().edge_ordinal(), 0);
            assert_eq!(directory.entry_for_edge(0), Some(*entry));
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            let arc = geometry(*entry)?;
            assert_vec_close(arc.center(), [0.0, 0.0, 5.0], 0.0)?;
            assert_vec_close(arc.x_axis(), [1.0, 0.0, 0.0], 0.0)?;
            assert_vec_close(arc.y_axis(), [0.0, 1.0, 0.0], 0.0)?;
            assert_vec_close(arc.normal(), [0.0, 0.0, 1.0], 0.0)?;
            assert_eq!(arc.radius().to_bits(), 2.0_f64.to_bits());
            assert_eq!(arc.start_angle_degrees().to_bits(), (-450.0_f64).to_bits());
            assert_eq!(arc.end_angle_degrees().to_bits(), 810.0_f64.to_bits());
            assert_eq!(
                arc.direction(),
                DxfHatchBoundaryCircularArcEdgeDirection::Counterclockwise
            );
        }
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn arbitrary_axis_and_negative_normal_frames_are_finite() -> Result<(), Box<dyn Error>> {
    let arbitrary = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n5\n",
        "210\n0\n220\n1\n230\n0\n",
        2,
        1,
        "10\n1\n20\n2\n40\n3\n50\n-90\n51\n450\n73\n0\n",
    );
    let arc = only_geometry(&arbitrary)?;
    assert_vec_close(arc.center(), [-1.0, 5.0, 2.0], 0.0)?;
    assert_vec_close(arc.x_axis(), [-1.0, 0.0, 0.0], 0.0)?;
    assert_vec_close(arc.y_axis(), [0.0, 0.0, 1.0], 0.0)?;
    assert_vec_close(arc.normal(), [0.0, 1.0, 0.0], 0.0)?;
    assert_eq!(arc.radius().to_f64(), 3.0);
    assert_eq!(arc.start_angle_degrees().to_f64(), -90.0);
    assert_eq!(arc.end_angle_degrees().to_f64(), 450.0);
    assert_eq!(
        arc.direction(),
        DxfHatchBoundaryCircularArcEdgeDirection::Clockwise
    );

    let negative = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n5\n",
        "210\n0\n220\n0\n230\n-1\n",
        2,
        1,
        "10\n1\n20\n2\n40\n3\n50\n-90\n51\n450\n73\n1\n",
    );
    let arc = only_geometry(&negative)?;
    assert_vec_close(arc.center(), [-1.0, 2.0, -5.0], 0.0)?;
    assert_vec_close(arc.x_axis(), [-1.0, 0.0, 0.0], 0.0)?;
    assert_vec_close(arc.y_axis(), [0.0, 1.0, 0.0], 0.0)?;
    assert_vec_close(arc.normal(), [0.0, 0.0, -1.0], 0.0)?;
    assert_eq!(
        arc.direction(),
        DxfHatchBoundaryCircularArcEdgeDirection::Counterclockwise
    );
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
        1,
        "20\n1\n40\n2\n50\n0\n51\n90\n73\n1\n",
    );
    assert!(matches!(
        only_result(&source_failure)?,
        Err(DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::SourceGeometry(
            DxfHatchBoundaryCircularArcEdgeGeometryIssue::ValuesUnavailable(mask)
        )) if mask.center_x() && mask.count() == 1
    ));

    let elevation_failure = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n",
        "210\n0\n220\n0\n230\n0\n",
        2,
        1,
        "10\n0\n20\n0\n40\n2\n50\n0\n51\n90\n73\n1\n",
    );
    assert!(matches!(
        only_result(&elevation_failure)?,
        Err(DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::ElevationUnavailable(
            DxfHatchElevationIssue::ComponentsUnavailable(mask)
        )) if !mask.x() && !mask.y() && mask.z()
    ));

    let extrusion_failure = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n1\n",
        "210\n0\n220\n-0\n230\n0\n",
        2,
        1,
        "10\n0\n20\n0\n40\n2\n50\n0\n51\n90\n73\n1\n",
    );
    assert_eq!(
        only_result(&extrusion_failure)?,
        Err(
            DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::ExtrusionUnavailable(
                DxfHatchExtrusionIssue::ZeroVector
            )
        )
    );

    let transform_failure = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n1.5e308\n",
        "210\n1\n220\n0\n230\n1\n",
        2,
        1,
        "10\n0\n20\n1.5e308\n40\n2\n50\n0\n51\n90\n73\n1\n",
    );
    assert_eq!(
        only_result(&transform_failure)?,
        Err(DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue::NonFiniteDerivedGeometry)
    );
    Ok(())
}

#[test]
fn cancellation_bounds_receipts_exclusion_redaction_and_traits_hold() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n5\n",
        "",
        2,
        0,
        "10\n12345.625\n20\n0\n40\n2\n50\n0\n51\n90\n73\n1\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_circular_arc_edge_wcs_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document
        .hatch_boundary_circular_arc_edge_wcs_geometry_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one WCS CircularArc entry").into());
    };
    assert_eq!(directory.entry(0), Some(*entry));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
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
            .ok_or_else(|| io::Error::other("elevation receipt"))?
            .subclass_ordinal(),
        entry.subclass_ordinal()
    );
    assert_eq!(
        directory
            .extrusion_for_entry(*entry)
            .ok_or_else(|| io::Error::other("extrusion receipt"))?
            .scalar_entry()
            .subclass_ordinal(),
        entry.subclass_ordinal()
    );
    assert!(!format!("{entry:?}").contains("12345.625"));
    assert!(size_of::<DxfHatchBoundaryCircularArcEdgeWcsSegment>() <= 192);
    assert!(size_of::<DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry>() <= 4096);
    copy::<DxfHatchBoundaryCircularArcEdgeWcsSegment>();
    copy::<DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry>();
    send_sync::<DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory>();

    let unavailable = ascii(
        DxfAcadVersion::Ac1032,
        "10\n0\n20\n0\n30\n5\n",
        "",
        1,
        1,
        "10\n5\n20\n6\n11\n7\n21\n8\n",
    );
    let source = DxfMemorySource::new(&unavailable, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_circular_arc_edge_wcs_geometry_directory(&DxfCancellationToken::default())?;
    assert!(directory.entries().is_empty());
    assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    Ok(())
}

fn only_result(
    bytes: &[u8],
) -> Result<
    Result<
        DxfHatchBoundaryCircularArcEdgeWcsSegment,
        DxfHatchBoundaryCircularArcEdgeWcsGeometryIssue,
    >,
    Box<dyn Error>,
> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_circular_arc_edge_wcs_geometry_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one WCS result").into());
    };
    Ok(entry.geometry())
}

fn only_geometry(
    bytes: &[u8],
) -> Result<DxfHatchBoundaryCircularArcEdgeWcsSegment, Box<dyn Error>> {
    only_result(bytes)?.map_err(|_| io::Error::other("usable WCS CircularArc").into())
}

fn geometry(
    entry: DxfHatchBoundaryCircularArcEdgeWcsGeometryEntry,
) -> Result<DxfHatchBoundaryCircularArcEdgeWcsSegment, io::Error> {
    entry
        .geometry()
        .map_err(|_| io::Error::other("usable WCS CircularArc geometry"))
}

fn evidence(
    directory: &DxfHatchBoundaryCircularArcEdgeWcsGeometryDirectory,
) -> Result<Vec<WcsEvidence>, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| {
            let arc = geometry(entry)?;
            Ok((
                [
                    arc.center().map(DxfDouble::to_bits),
                    arc.x_axis().map(DxfDouble::to_bits),
                    arc.y_axis().map(DxfDouble::to_bits),
                    arc.normal().map(DxfDouble::to_bits),
                ],
                [
                    arc.radius().to_bits(),
                    arc.start_angle_degrees().to_bits(),
                    arc.end_angle_degrees().to_bits(),
                ],
                arc.direction().flag(),
            ))
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
    edge_type: i16,
    declared: u32,
    payload: &str,
) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}{}91\n1\n92\n0\n93\n{}\n72\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), elevation, extrusion, declared, edge_type, payload
    )
    .into_bytes()
}

#[derive(Clone, Copy)]
struct ArcWire {
    doubles: [u64; 5],
    direction: i16,
}

fn binary(
    version: DxfAcadVersion,
    elevation: [u64; 3],
    extrusion: Option<[u64; 3]>,
    declared: i32,
    arc: ArcWire,
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
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, declared)?;
    push_i16(&mut bytes, version, 72, 2)?;
    for (code, bits) in [
        (10, arc.doubles[0]),
        (20, arc.doubles[1]),
        (40, arc.doubles[2]),
        (50, arc.doubles[3]),
        (51, arc.doubles[4]),
    ] {
        push_double(&mut bytes, version, code, bits)?;
    }
    push_i16(&mut bytes, version, 73, arc.direction)?;
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

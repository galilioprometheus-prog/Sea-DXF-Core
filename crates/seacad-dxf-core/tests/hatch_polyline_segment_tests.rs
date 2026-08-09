use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchPolylineSegmentDirectory, DxfHatchPolylineSegmentEntry,
    DxfHatchPolylineSegmentPathState, DxfHatchPolylineSegmentTopology, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};
use std::{error::Error, io, mem::size_of};

#[test]
fn every_dialect_builds_open_and_closed_segment_topology() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for closed in [false, true] {
            let ascii = ascii(
                version,
                2,
                closed,
                "10\n1\n20\n2\n10\n3\n20\n4\n10\n5\n20\n6\n",
                3,
            );
            let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
            let ascii_directory = open_ascii(&source)?
                .hatch_polyline_segment_directory(&DxfCancellationToken::default())?;
            let binary = binary(version, 2, closed, 3, &[(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)])?;
            let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
            let binary_directory = open_binary(&source)?
                .hatch_polyline_segment_directory(&DxfCancellationToken::default())?;

            for directory in [&ascii_directory, &binary_directory] {
                assert_eq!(
                    directory.paths()[0].state(),
                    DxfHatchPolylineSegmentPathState::Available { closed }
                );
                let segments = directory
                    .segments_for_path(0)
                    .ok_or_else(|| io::Error::other("path segments"))?;
                assert_eq!(segments.len(), if closed { 3 } else { 2 });
                assert_eq!(
                    segments[0].topology(),
                    DxfHatchPolylineSegmentTopology::Consecutive
                );
                assert_eq!(segments[0].start_vertex_ordinal(), 0);
                assert_eq!(segments[0].end_vertex_ordinal(), 1);
                assert_eq!(segments[1].path_segment_ordinal(), 1);
                if closed {
                    assert_eq!(
                        segments[2].topology(),
                        DxfHatchPolylineSegmentTopology::Closing
                    );
                    assert_eq!(segments[2].start_vertex_ordinal(), 2);
                    assert_eq!(segments[2].end_vertex_ordinal(), 0);
                }
                let endpoints = directory
                    .endpoints_for_segment(0)
                    .ok_or_else(|| io::Error::other("segment endpoints"))?;
                let start = endpoints
                    .start()
                    .ocs_position()
                    .map_err(|_| io::Error::other("start position"))?;
                let end = endpoints
                    .end()
                    .ocs_position()
                    .map_err(|_| io::Error::other("end position"))?;
                assert_eq!(start.values().map(|v| v.to_f64()), [1.0, 2.0]);
                assert_eq!(end.values().map(|v| v.to_f64()), [3.0, 4.0]);
            }
        }
    }
    Ok(())
}

#[test]
fn count_mismatch_and_degenerate_closed_paths_are_explicit() -> Result<(), Box<dyn Error>> {
    let mismatch = ascii(
        DxfAcadVersion::Ac1032,
        2,
        false,
        "10\n1\n20\n2\n10\n3\n20\n4\n",
        3,
    );
    let source = DxfMemorySource::new(&mismatch, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.paths()[0].state(),
        DxfHatchPolylineSegmentPathState::VertexCountMismatched {
            declared: 3,
            observed: 2
        }
    );
    assert!(directory.segments().is_empty());
    assert_eq!(directory.segments_for_path(0), Some(&[][..]));

    let single = ascii(DxfAcadVersion::Ac1032, 2, true, "10\n1\n20\n2\n", 1);
    let source = DxfMemorySource::new(&single, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_polyline_segment_directory(&DxfCancellationToken::default())?;
    let [segment] = directory.segments() else {
        return Err(io::Error::other("self-closing segment").into());
    };
    assert_eq!(segment.topology(), DxfHatchPolylineSegmentTopology::Closing);
    assert_eq!(segment.start_vertex_ordinal(), 0);
    assert_eq!(segment.end_vertex_ordinal(), 0);

    let empty = ascii(DxfAcadVersion::Ac1032, 2, true, "", 0);
    let source = DxfMemorySource::new(&empty, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.paths()[0].state(),
        DxfHatchPolylineSegmentPathState::Available { closed: true }
    );
    assert!(directory.segments().is_empty());
    Ok(())
}

#[test]
fn coordinate_failures_keep_topology_but_unavailable_paths_emit_none() -> Result<(), Box<dyn Error>>
{
    let invalid_coordinates = ascii(
        DxfAcadVersion::Ac1032,
        2,
        false,
        "10\nbad\n20\n2\n10\n3\n",
        2,
    );
    let source = DxfMemorySource::new(&invalid_coordinates, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.segments().len(), 1);
    let endpoints = directory
        .endpoints_for_segment(0)
        .ok_or_else(|| io::Error::other("invalid endpoints"))?;
    assert!(endpoints.start().ocs_position().is_err());
    assert!(endpoints.end().ocs_position().is_err());

    for (path_flag, payload, expected_header_failure) in
        [(0, "", false), (2, "72\n.\n73\n0\n93\n0\n", true)]
    {
        let ascii = ascii(DxfAcadVersion::Ac1032, path_flag, false, payload, 0);
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let directory = open_ascii(&source)?
            .hatch_polyline_segment_directory(&DxfCancellationToken::default())?;
        if expected_header_failure {
            assert!(matches!(
                directory.paths()[0].state(),
                DxfHatchPolylineSegmentPathState::HeaderUnavailable(_)
            ));
        } else {
            assert_eq!(
                directory.paths()[0].state(),
                DxfHatchPolylineSegmentPathState::NotPolyline
            );
        }
        assert!(directory.segments().is_empty());
    }
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_metadata_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii(
        DxfAcadVersion::Ac1032,
        2,
        false,
        "10\n1\n20\n2\n10\n3\n20\n4\n",
        2,
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_polyline_segment_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.hatch_polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.path(0), Some(directory.paths()[0]));
    assert_eq!(directory.path(u64::MAX), None);
    assert_eq!(directory.segment(0), Some(directory.segments()[0]));
    assert_eq!(directory.segment(u64::MAX), None);
    assert!(directory.segments_for_path(u64::MAX).is_none());
    assert!(directory.endpoints_for_segment(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.coordinate_directory().source_id()
    );
    assert!(size_of::<DxfHatchPolylineSegmentEntry>() <= 24);
    send_sync::<DxfHatchPolylineSegmentDirectory>();
    Ok(())
}

fn ascii(
    version: DxfAcadVersion,
    path_flag: i32,
    closed: bool,
    payload: &str,
    declared: u32,
) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n{}\n72\n0\n73\n{}\n93\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        path_flag,
        i16::from(closed),
        declared,
        payload
    )
    .into_bytes()
}

fn binary(
    version: DxfAcadVersion,
    path_flag: i32,
    closed: bool,
    declared: i32,
    vertices: &[(f64, f64)],
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
    push_i16(&mut bytes, version, 72, 0)?;
    push_i16(&mut bytes, version, 73, i16::from(closed))?;
    push_i32(&mut bytes, version, 93, declared)?;
    for (x, y) in vertices.iter().copied() {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
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
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
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

fn send_sync<T: Send + Sync>() {}

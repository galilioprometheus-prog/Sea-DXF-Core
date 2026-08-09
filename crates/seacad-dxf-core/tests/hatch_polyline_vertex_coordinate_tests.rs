use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchPolylineVertexCoordinateDirectory, DxfHatchPolylineVertexCoordinateIssue,
    DxfHatchPolylineVertexNumericIssue, DxfHatchPolylineVertexPositionIssue, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};
use std::{error::Error, io};

#[test]
fn every_dialect_assembles_exact_required_ocs_positions() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii(version, 2, "10\n-0\n20\n2.5\n10\n3\n20\n-0\n", 2);
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_polyline_vertex_coordinate_directory(&DxfCancellationToken::default())?;
        let binary = binary(
            version,
            2,
            &[
                ((-0.0_f64).to_bits(), 2.5_f64.to_bits()),
                (3.0_f64.to_bits(), (-0.0_f64).to_bits()),
            ],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_polyline_vertex_coordinate_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            let [first, second] = directory.entries() else {
                return Err(io::Error::other("two coordinate entries").into());
            };
            let first_position = first
                .ocs_position()
                .map_err(|_| io::Error::other("first OCS position"))?;
            let second_position = second
                .ocs_position()
                .map_err(|_| io::Error::other("second OCS position"))?;
            assert_eq!(
                first_position.values().map(|value| value.to_bits()),
                [(-0.0_f64).to_bits(), 2.5_f64.to_bits()]
            );
            assert_eq!(
                second_position.values().map(|value| value.to_bits()),
                [3.0_f64.to_bits(), (-0.0_f64).to_bits()]
            );
            for entry in [first, second] {
                for coordinate in [entry.coordinates().x(), entry.coordinates().y()] {
                    assert_eq!(coordinate.state(), DxfSemanticValueState::Explicit);
                    assert!(coordinate.raw_provenance().is_some());
                }
                assert_eq!(
                    entry.bulge().bulge().state(),
                    DxfSemanticValueState::Defaulted
                );
            }
        }
    }
    Ok(())
}

#[test]
fn missing_duplicate_and_malformed_required_coordinates_are_typed() -> Result<(), Box<dyn Error>> {
    let ascii = ascii(
        DxfAcadVersion::Ac1032,
        2,
        "10\n1\n10\n2\n20\n3\n20\n4\n10\nbad\n20\nbad\n",
        3,
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_vertex_coordinate_directory(&DxfCancellationToken::default())?;
    let [missing, duplicate, malformed] = directory.entries() else {
        return Err(io::Error::other("three coordinate entries").into());
    };
    assert_eq!(
        missing.coordinates().y().invalid_issue(),
        Some(&DxfHatchPolylineVertexCoordinateIssue::MissingRequiredValue)
    );
    assert_eq!(missing.coordinates().y().raw_provenance(), None);
    assert_mask(*missing, false, true, 1)?;
    assert_eq!(
        duplicate.coordinates().y().invalid_issue(),
        Some(&DxfHatchPolylineVertexCoordinateIssue::Numeric(
            DxfHatchPolylineVertexNumericIssue::MultipleValues {
                occurrence_count: 2
            }
        ))
    );
    assert_mask(*duplicate, false, true, 1)?;
    for coordinate in [malformed.coordinates().x(), malformed.coordinates().y()] {
        assert!(matches!(
            coordinate.invalid_issue(),
            Some(DxfHatchPolylineVertexCoordinateIssue::Numeric(
                DxfHatchPolylineVertexNumericIssue::InvalidAsciiNumber(
                    DxfAsciiNumericIssue::InvalidSyntax { .. }
                )
            ))
        ));
        assert!(coordinate.raw_provenance().is_some());
    }
    assert_mask(*malformed, true, true, 2)?;
    Ok(())
}

#[test]
fn binary_nonfinite_and_unavailable_paths_publish_no_position() -> Result<(), Box<dyn Error>> {
    let binary = binary(
        DxfAcadVersion::Ac1032,
        2,
        &[(f64::INFINITY.to_bits(), f64::NAN.to_bits())],
    )?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_polyline_vertex_coordinate_directory(&DxfCancellationToken::default())?;
    let entry = directory.entries()[0];
    for coordinate in [entry.coordinates().x(), entry.coordinates().y()] {
        assert!(matches!(
            coordinate.invalid_issue(),
            Some(DxfHatchPolylineVertexCoordinateIssue::Numeric(
                DxfHatchPolylineVertexNumericIssue::NonFiniteDouble(_)
            ))
        ));
        assert!(coordinate.raw_provenance().is_some());
    }
    assert_mask(entry, true, true, 2)?;

    for (path_flag, payload) in [(0, ""), (2, "72\n.\n73\n0\n93\n0\n")] {
        let ascii = ascii(DxfAcadVersion::Ac1032, path_flag, payload, 0);
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let directory = open_ascii(&source)?
            .hatch_polyline_vertex_coordinate_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert!(directory.entries_for_path(0).is_none());
    }
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii(DxfAcadVersion::Ac1032, 2, "10\n12345.625\n20\n2\n", 1);
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_polyline_vertex_coordinate_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.hatch_polyline_vertex_coordinate_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.bulge_directory().source_id()
    );
    assert!(!format!("{:?}", directory.entries()[0].coordinates()).contains("12345.625"));
    send_sync::<DxfHatchPolylineVertexCoordinateDirectory>();
    Ok(())
}

fn assert_mask(
    entry: seacad_dxf_core::DxfHatchPolylineVertexCoordinateEntry,
    x: bool,
    y: bool,
    count: u32,
) -> Result<(), Box<dyn Error>> {
    let Err(DxfHatchPolylineVertexPositionIssue::CoordinatesUnavailable(mask)) =
        entry.ocs_position()
    else {
        return Err(io::Error::other("unavailable coordinates").into());
    };
    assert_eq!(mask.x(), x);
    assert_eq!(mask.y(), y);
    assert_eq!(mask.count(), count);
    Ok(())
}

fn ascii(version: DxfAcadVersion, path_flag: i32, payload: &str, count: u32) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n{}\n72\n0\n73\n1\n93\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        path_flag,
        count,
        payload
    )
    .into_bytes()
}

fn binary(version: DxfAcadVersion, path_flag: i32, vertices: &[(u64, u64)]) -> io::Result<Vec<u8>> {
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
    push_i16(&mut bytes, version, 73, 1)?;
    push_i32(
        &mut bytes,
        version,
        93,
        i32::try_from(vertices.len()).map_err(|_| io::Error::other("vertex count"))?,
    )?;
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

fn send_sync<T: Send + Sync>() {}

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchPolylineBulgeIssue, DxfHatchPolylineSegmentShape,
    DxfHatchPolylineSegmentShapeDirectory, DxfHatchPolylineVertexCardState,
    DxfHatchPolylineVertexNumericIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};
use std::{error::Error, io, mem::size_of};

#[test]
fn every_dialect_classifies_default_zero_explicit_zero_and_arc() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii(
            version,
            2,
            true,
            true,
            "10\n1\n20\n2\n10\n3\n20\n4\n42\n-0\n10\n5\n20\n6\n42\n0.5\n",
            3,
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;
        let binary = binary(
            version,
            2,
            true,
            true,
            3,
            &[
                (1.0, 2.0, None),
                (3.0, 4.0, Some((-0.0_f64).to_bits())),
                (5.0, 6.0, Some(0.5_f64.to_bits())),
            ],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            let [first, second, closing] = directory.entries() else {
                return Err(io::Error::other("three shape entries").into());
            };
            assert_eq!(first.shape(), DxfHatchPolylineSegmentShape::Straight);
            assert_eq!(second.shape(), DxfHatchPolylineSegmentShape::Straight);
            assert_eq!(
                closing.shape(),
                DxfHatchPolylineSegmentShape::Arc {
                    bulge: seacad_dxf_core::DxfDouble::from_bits(0.5_f64.to_bits())
                }
            );
            let first_bulge = directory
                .start_bulge(0)
                .ok_or_else(|| io::Error::other("default bulge"))?;
            let second_bulge = directory
                .start_bulge(1)
                .ok_or_else(|| io::Error::other("explicit zero"))?;
            assert_eq!(
                first_bulge.bulge().state(),
                DxfSemanticValueState::Defaulted
            );
            assert_eq!(first_bulge.bulge().raw_provenance(), None);
            assert_eq!(
                second_bulge.bulge().state(),
                DxfSemanticValueState::Explicit
            );
            assert!(second_bulge.bulge().raw_provenance().is_some());
        }
    }
    Ok(())
}

#[test]
fn invalid_numeric_and_header_relation_bulges_are_indeterminate() -> Result<(), Box<dyn Error>> {
    let malformed = ascii(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        "10\n1\n20\n2\n42\nbad\n10\n3\n20\n4\n",
        2,
    );
    let source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].shape(),
        DxfHatchPolylineSegmentShape::Indeterminate(DxfHatchPolylineBulgeIssue::Numeric(
            DxfHatchPolylineVertexNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));

    let nonfinite = binary(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        2,
        &[(1.0, 2.0, Some(f64::INFINITY.to_bits())), (3.0, 4.0, None)],
    )?;
    let source = DxfMemorySource::new(&nonfinite, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].shape(),
        DxfHatchPolylineSegmentShape::Indeterminate(DxfHatchPolylineBulgeIssue::Numeric(
            DxfHatchPolylineVertexNumericIssue::NonFiniteDouble(_)
        ))
    ));

    let disallowed = ascii(
        DxfAcadVersion::Ac1032,
        2,
        false,
        false,
        "10\n1\n20\n2\n42\n0.5\n10\n3\n20\n4\n",
        2,
    );
    let source = DxfMemorySource::new(&disallowed, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.entries()[0].shape(),
        DxfHatchPolylineSegmentShape::Indeterminate(
            DxfHatchPolylineBulgeIssue::PresentWhenHeaderDisallows {
                state: DxfHatchPolylineVertexCardState::Unique
            }
        )
    );
    Ok(())
}

#[test]
fn coordinate_failures_do_not_change_shape_and_unavailable_paths_emit_none()
-> Result<(), Box<dyn Error>> {
    let invalid_coordinates = ascii(
        DxfAcadVersion::Ac1032,
        2,
        false,
        false,
        "10\nbad\n20\n2\n10\n3\n",
        2,
    );
    let source = DxfMemorySource::new(&invalid_coordinates, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.entries()[0].shape(),
        DxfHatchPolylineSegmentShape::Straight
    );
    let endpoints = directory
        .segment_directory()
        .endpoints_for_segment(0)
        .ok_or_else(|| io::Error::other("coordinate-failure endpoints"))?;
    assert!(endpoints.start().ocs_position().is_err());
    assert!(endpoints.end().ocs_position().is_err());

    for (path_flag, payload, declared) in [
        (2, "10\n1\n20\n2\n", 2),
        (0, "", 0),
        (2, "72\n.\n73\n0\n93\n0\n", 0),
    ] {
        let ascii = ascii(
            DxfAcadVersion::Ac1032,
            path_flag,
            false,
            false,
            payload,
            declared,
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let directory = open_ascii(&source)?
            .hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    }
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_redaction_and_metadata_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii(
        DxfAcadVersion::Ac1032,
        2,
        true,
        false,
        "10\n1\n20\n2\n42\n12345.625\n10\n3\n20\n4\n",
        2,
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_polyline_segment_shape_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.hatch_polyline_segment_shape_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert!(directory.start_bulge(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.segment_directory().source_id()
    );
    assert!(size_of::<seacad_dxf_core::DxfHatchPolylineSegmentShapeEntry>() <= 64);
    assert!(!format!("{:?}", directory.start_bulge(0)).contains("12345.625"));
    send_sync::<DxfHatchPolylineSegmentShapeDirectory>();
    Ok(())
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
    vertices: &[(f64, f64, Option<u64>)],
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
        push_double(&mut bytes, version, 10, x.to_bits())?;
        push_double(&mut bytes, version, 20, y.to_bits())?;
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

fn send_sync<T: Send + Sync>() {}

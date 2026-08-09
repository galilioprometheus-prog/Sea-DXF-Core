use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchBoundaryLineEdgeCoordinateDirectory, DxfHatchBoundaryLineEdgeCoordinateEntry,
    DxfHatchBoundaryLineEdgeCoordinateIssue, DxfHatchBoundaryLineEdgeEndpointIssue,
    DxfHatchBoundaryLineEdgeNumericIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_assembles_exact_required_ocs_endpoints() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n1\n10\n-0\n20\n2.5\n11\n-3.25\n21\n4\n\
             72\n2\n10\n5\n20\n6\n40\n7\n50\n0\n51\n90\n73\n1\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_line_edge_coordinate_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(
            version,
            2,
            [
                (-0.0_f64).to_bits(),
                2.5_f64.to_bits(),
                (-3.25_f64).to_bits(),
                4.0_f64.to_bits(),
            ],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_line_edge_coordinate_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            let [entry] = directory.entries() else {
                return Err(io::Error::other("one coordinate line edge").into());
            };
            assert_eq!(entry.ordinal(), 0);
            assert_eq!(entry.numeric().edge_ordinal(), 0);
            assert_eq!(entry.numeric().path_ordinal(), 0);
            assert_eq!(directory.entry_for_edge(0), Some(*entry));
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            let endpoints = entry
                .ocs_endpoints()
                .map_err(|_| io::Error::other("usable OCS endpoints"))?;
            assert_eq!(
                endpoints.start().values().map(|value| value.to_bits()),
                [(-0.0_f64).to_bits(), 2.5_f64.to_bits()]
            );
            assert_eq!(
                endpoints.end().values().map(|value| value.to_bits()),
                [(-3.25_f64).to_bits(), 4.0_f64.to_bits()]
            );
            for value in [
                entry.coordinates().start_x(),
                entry.coordinates().start_y(),
                entry.coordinates().end_x(),
                entry.coordinates().end_y(),
            ] {
                assert_eq!(value.state(), DxfSemanticValueState::Explicit);
                assert!(value.raw_provenance().is_some());
            }
        }
    }
    Ok(())
}

#[test]
fn missing_duplicate_and_malformed_required_coordinates_are_typed() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n3\n\
         72\n1\n20\n2\n11\n3\n21\n4\n\
         72\n1\n10\n1\n20\n2\n20\n3\n11\nbad\n21\n4\n\
         72\n1\n10\nbad\n20\nbad\n11\n3\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_line_edge_coordinate_directory(&DxfCancellationToken::default())?;
    let [missing, duplicate, malformed] = directory.entries() else {
        return Err(io::Error::other("three coordinate line entries").into());
    };

    assert_eq!(
        missing.coordinates().start_x().invalid_issue(),
        Some(&DxfHatchBoundaryLineEdgeCoordinateIssue::MissingRequiredValue)
    );
    assert_eq!(missing.coordinates().start_x().raw_provenance(), None);
    assert_mask(*missing, true, false, false, false, 1)?;

    assert_eq!(
        duplicate.coordinates().start_y().invalid_issue(),
        Some(&DxfHatchBoundaryLineEdgeCoordinateIssue::Numeric(
            DxfHatchBoundaryLineEdgeNumericIssue::MultipleValues {
                occurrence_count: 2
            }
        ))
    );
    assert!(matches!(
        duplicate.coordinates().end_x().invalid_issue(),
        Some(DxfHatchBoundaryLineEdgeCoordinateIssue::Numeric(
            DxfHatchBoundaryLineEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert_mask(*duplicate, false, true, true, false, 2)?;

    for value in [
        malformed.coordinates().start_x(),
        malformed.coordinates().start_y(),
    ] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfHatchBoundaryLineEdgeCoordinateIssue::Numeric(
                DxfHatchBoundaryLineEdgeNumericIssue::InvalidAsciiNumber(
                    DxfAsciiNumericIssue::InvalidSyntax { .. }
                )
            ))
        ));
        assert!(value.raw_provenance().is_some());
    }
    assert_eq!(
        malformed.coordinates().end_y().invalid_issue(),
        Some(&DxfHatchBoundaryLineEdgeCoordinateIssue::MissingRequiredValue)
    );
    assert_mask(*malformed, true, true, false, true, 3)?;
    Ok(())
}

#[test]
fn binary_nonfinite_and_non_line_paths_publish_no_endpoints() -> Result<(), Box<dyn Error>> {
    let binary = binary_fixture(
        DxfAcadVersion::Ac1032,
        0,
        [
            f64::INFINITY.to_bits(),
            f64::NAN.to_bits(),
            3.0_f64.to_bits(),
            4.0_f64.to_bits(),
        ],
    )?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_line_edge_coordinate_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry_for_edge(0)
        .ok_or_else(|| io::Error::other("count-mismatched line entry"))?;
    for value in [entry.coordinates().start_x(), entry.coordinates().start_y()] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfHatchBoundaryLineEdgeCoordinateIssue::Numeric(
                DxfHatchBoundaryLineEdgeNumericIssue::NonFiniteDouble(_)
            ))
        ));
        assert!(value.raw_provenance().is_some());
    }
    assert_mask(entry, true, true, false, false, 2)?;

    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n\
         92\n0\n93\n2\n\
         72\n2\n10\n5\n20\n6\n40\n7\n50\n0\n51\n90\n73\n1\n\
         72\n.\n10\n8\n20\n9\n11\n10\n21\n11\n97\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_line_edge_coordinate_directory(&DxfCancellationToken::default())?;
    assert!(directory.entries().is_empty());
    assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    assert!(directory.entries_for_path(1).is_none());
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n1\n10\n12345.625\n20\n2\n11\n3\n21\n4\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_line_edge_coordinate_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.hatch_boundary_line_edge_coordinate_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.numeric_directory().source_id()
    );
    assert!(size_of::<DxfHatchBoundaryLineEdgeCoordinateEntry>() <= 1024);
    assert!(!format!("{:?}", directory.entries()[0].coordinates()).contains("12345.625"));
    copy::<DxfHatchBoundaryLineEdgeCoordinateEntry>();
    send_sync::<DxfHatchBoundaryLineEdgeCoordinateDirectory>();
    Ok(())
}

fn assert_mask(
    entry: DxfHatchBoundaryLineEdgeCoordinateEntry,
    start_x: bool,
    start_y: bool,
    end_x: bool,
    end_y: bool,
    count: u32,
) -> Result<(), Box<dyn Error>> {
    let Err(DxfHatchBoundaryLineEdgeEndpointIssue::CoordinatesUnavailable(mask)) =
        entry.ocs_endpoints()
    else {
        return Err(io::Error::other("unavailable endpoint coordinates").into());
    };
    assert_eq!(mask.start_x(), start_x);
    assert_eq!(mask.start_y(), start_y);
    assert_eq!(mask.end_x(), end_x);
    assert_eq!(mask.end_y(), end_y);
    assert_eq!(mask.count(), count);
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
    )
    .into_bytes()
}

fn binary_fixture(
    version: DxfAcadVersion,
    declared_edge_count: i32,
    line_bits: [u64; 4],
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
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, declared_edge_count)?;
    push_i16(&mut bytes, version, 72, 1)?;
    for (code, bits) in [
        (10, line_bits[0]),
        (20, line_bits[1]),
        (11, line_bits[2]),
        (21, line_bits[3]),
    ] {
        push_double_bits(&mut bytes, version, code, bits)?;
    }
    push_i16(&mut bytes, version, 72, 2)?;
    for (code, value) in [(10, 5.0), (20, 6.0), (40, 7.0), (50, 0.0), (51, 90.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 73, 1)?;
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
    push_double_bits(bytes, version, code, value.to_bits())
}

fn push_double_bits(
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

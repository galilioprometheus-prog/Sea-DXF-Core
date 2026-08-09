use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryEllipticArcEdgeDirection, DxfHatchBoundaryEllipticArcEdgeMajorAxisState,
    DxfHatchBoundaryEllipticArcEdgeNumericIssue, DxfHatchBoundaryEllipticArcEdgeSemanticDirectory,
    DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
    DxfHatchBoundaryEllipticArcEdgeSemanticEntry, DxfHatchBoundaryEllipticArcEdgeSemanticIssue,
    DxfHatchBoundaryEllipticArcEdgeSemantics, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_preserves_valid_boundaries_angles_and_directions() -> Result<(), Box<dyn Error>> {
    let expected = [
        EllipseWire {
            doubles: [
                (-0.0_f64).to_bits(),
                2.5_f64.to_bits(),
                3.0_f64.to_bits(),
                4.0_f64.to_bits(),
                1.0e-6_f64.to_bits(),
                (-450.0_f64).to_bits(),
                810.25_f64.to_bits(),
            ],
            direction: 0,
        },
        EllipseWire {
            doubles: [
                5.0_f64.to_bits(),
                6.0_f64.to_bits(),
                (-7.0_f64).to_bits(),
                8.0_f64.to_bits(),
                1.0_f64.to_bits(),
                (-45.0_f64).to_bits(),
                360.0_f64.to_bits(),
            ],
            direction: 1,
        },
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n3\n10\n-0\n20\n2.5\n11\n3\n21\n4\n40\n0.000001\n50\n-450\n51\n810.25\n73\n0\n\
             72\n3\n10\n5\n20\n6\n11\n-7\n21\n8\n40\n1\n50\n-45\n51\n360\n73\n1\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_elliptic_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(version, 2, &expected)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_elliptic_arc_edge_semantic_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.entries().len(), expected.len());
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            for (index, (entry, expected)) in directory.entries().iter().zip(expected).enumerate() {
                assert_eq!(entry.ordinal(), index as u64);
                assert_eq!(entry.numeric().edge_ordinal(), index as u64);
                assert_eq!(entry.numeric().path_ordinal(), 0);
                assert_eq!(directory.entry_for_edge(index as u64), Some(*entry));
                let semantics = entry.semantics();
                assert_eq!(
                    semantics.major_axis_state(),
                    DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Usable
                );
                assert_eq!(semantic_bits(semantics)?, expected.doubles);
                assert_eq!(
                    semantics.direction_value(),
                    Some(if expected.direction == 0 {
                        DxfHatchBoundaryEllipticArcEdgeDirection::Clockwise
                    } else {
                        DxfHatchBoundaryEllipticArcEdgeDirection::Counterclockwise
                    })
                );
                assert_eq!(
                    semantics.direction_value().map(|value| value.flag()),
                    Some(expected.direction)
                );
                assert_eq!(
                    semantics
                        .direction_value()
                        .map(|value| value.is_counterclockwise()),
                    Some(index == 1)
                );
                for (value, field_id) in [
                    (semantics.center_x(), "boundary_elliptic_arc_center_x"),
                    (semantics.center_y(), "boundary_elliptic_arc_center_y"),
                    (
                        semantics.major_axis_endpoint_x(),
                        "boundary_elliptic_arc_major_axis_endpoint_x",
                    ),
                    (
                        semantics.major_axis_endpoint_y(),
                        "boundary_elliptic_arc_major_axis_endpoint_y",
                    ),
                    (
                        semantics.minor_to_major_ratio(),
                        "boundary_elliptic_arc_minor_to_major_ratio",
                    ),
                    (
                        semantics.start_angle_degrees(),
                        "boundary_elliptic_arc_start_angle",
                    ),
                    (
                        semantics.end_angle_degrees(),
                        "boundary_elliptic_arc_end_angle",
                    ),
                ] {
                    assert_eq!(value.state(), DxfSemanticValueState::Explicit);
                    assert_eq!(value.field_provenance().schema_namespace(), "entity.hatch");
                    assert_eq!(value.field_provenance().schema_field_id(), field_id);
                    assert!(value.raw_provenance().is_some());
                }
                assert_eq!(
                    semantics.direction().state(),
                    DxfSemanticValueState::Explicit
                );
                assert!(semantics.direction().raw_provenance().is_some());
            }
        }
    }
    Ok(())
}

#[test]
fn required_and_numeric_failures_keep_provenance_and_axis_mask() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n3\n\
         20\n2\n20\n3\n11\nbad\n40\n0.5\n51\n90\n73\n.\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_elliptic_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("ASCII semantic entry"))?
        .semantics()
        .to_owned();
    for value in [semantics.center_x(), semantics.start_angle_degrees()] {
        assert_eq!(
            value.invalid_issue(),
            Some(&DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MissingRequiredValue)
        );
        assert_eq!(value.raw_provenance(), None);
    }
    assert_eq!(
        semantics.center_y().invalid_issue(),
        Some(&DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::MultipleValues {
                occurrence_count: 2,
            }
        ))
    );
    assert_eq!(semantics.center_y().raw_provenance(), None);
    assert!(matches!(
        semantics.major_axis_endpoint_x().invalid_issue(),
        Some(DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert!(semantics.major_axis_endpoint_x().raw_provenance().is_some());
    let DxfHatchBoundaryEllipticArcEdgeMajorAxisState::ComponentsUnavailable(mask) =
        semantics.major_axis_state()
    else {
        return Err(io::Error::other("major-axis mask").into());
    };
    assert!(mask.x());
    assert!(mask.y());
    assert_eq!(mask.count(), 2);
    assert!(matches!(
        semantics.direction().invalid_issue(),
        Some(DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert!(semantics.direction().raw_provenance().is_some());
    assert_eq!(
        semantics
            .minor_to_major_ratio_value()
            .map(DxfDouble::to_f64),
        Some(0.5)
    );
    assert_eq!(
        semantics.end_angle_degrees_value().map(DxfDouble::to_f64),
        Some(90.0)
    );

    let binary = binary_fixture(
        DxfAcadVersion::Ac1032,
        1,
        &[EllipseWire {
            doubles: [
                0x7ff8_0000_0000_0042,
                2.0_f64.to_bits(),
                3.0_f64.to_bits(),
                4.0_f64.to_bits(),
                0.5_f64.to_bits(),
                6.0_f64.to_bits(),
                f64::INFINITY.to_bits(),
            ],
            direction: 1,
        }],
    )?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_elliptic_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("Binary semantic entry"))?
        .semantics()
        .to_owned();
    for value in [semantics.center_x(), semantics.end_angle_degrees()] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(
                DxfHatchBoundaryEllipticArcEdgeNumericIssue::NonFiniteDouble(_)
            ))
        ));
        assert!(value.raw_provenance().is_some());
    }
    assert_eq!(
        semantics.major_axis_state(),
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Usable
    );
    assert_eq!(
        semantics.direction_value(),
        Some(DxfHatchBoundaryEllipticArcEdgeDirection::Counterclockwise)
    );
    Ok(())
}

#[test]
fn ratio_axis_and_direction_domains_retain_exact_values() -> Result<(), Box<dyn Error>> {
    let ratios = [
        0.0_f64.to_bits(),
        1.0e-7_f64.to_bits(),
        1.000001_f64.to_bits(),
    ];
    let directions = [-1_i16, 2, i16::MIN];
    let expected_axis_states = [
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Degenerate,
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Degenerate,
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::NonFiniteDerivedMagnitude,
    ];
    let ellipses = [
        EllipseWire {
            doubles: [
                1.0_f64.to_bits(),
                2.0_f64.to_bits(),
                0.0_f64.to_bits(),
                (-0.0_f64).to_bits(),
                ratios[0],
                (-720.0_f64).to_bits(),
                1080.0_f64.to_bits(),
            ],
            direction: directions[0],
        },
        EllipseWire {
            doubles: [
                1.0_f64.to_bits(),
                2.0_f64.to_bits(),
                1.0e-200_f64.to_bits(),
                0.0_f64.to_bits(),
                ratios[1],
                (-720.0_f64).to_bits(),
                1080.0_f64.to_bits(),
            ],
            direction: directions[1],
        },
        EllipseWire {
            doubles: [
                1.0_f64.to_bits(),
                2.0_f64.to_bits(),
                f64::MAX.to_bits(),
                f64::MAX.to_bits(),
                ratios[2],
                (-720.0_f64).to_bits(),
                1080.0_f64.to_bits(),
            ],
            direction: directions[2],
        },
    ];
    let binary = binary_fixture(DxfAcadVersion::Ac1032, 0, &ellipses)?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_elliptic_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 3);
    for (((entry, ratio), direction), axis_state) in directory
        .entries()
        .iter()
        .zip(ratios)
        .zip(directions)
        .zip(expected_axis_states)
    {
        let semantics = entry.semantics();
        assert_eq!(semantics.major_axis_state(), axis_state);
        assert_eq!(semantics.ocs_major_axis_endpoint_value(), None);
        match semantics.minor_to_major_ratio().invalid_issue() {
            Some(DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MinorToMajorRatioOutOfDomain {
                ratio: observed,
            }) => assert_eq!(observed.to_bits(), ratio),
            _ => return Err(io::Error::other("ratio domain issue").into()),
        }
        assert!(semantics.minor_to_major_ratio().raw_provenance().is_some());
        assert_eq!(
            semantics.direction().invalid_issue(),
            Some(
                &DxfHatchBoundaryEllipticArcEdgeSemanticIssue::DirectionFlagOutOfDomain {
                    value: direction,
                }
            )
        );
        assert!(semantics.direction().raw_provenance().is_some());
        assert_eq!(
            semantics.start_angle_degrees_value().map(DxfDouble::to_f64),
            Some(-720.0)
        );
        assert_eq!(
            semantics.end_angle_degrees_value().map(DxfDouble::to_f64),
            Some(1080.0)
        );
    }
    Ok(())
}

#[test]
fn cancellation_exclusions_bounds_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n3\n\
         10\n1\n20\n2\n11\n12345.625\n21\n4\n40\n0.123456789\n50\n0\n51\n90\n73\n1\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_elliptic_arc_edge_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document
        .hatch_boundary_elliptic_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.numeric_directory().source_id()
    );
    let debug = format!("{:?}", directory.entries()[0].semantics());
    assert!(!debug.contains("12345.625"));
    assert!(!debug.contains("0.123456789"));
    assert!(size_of::<DxfHatchBoundaryEllipticArcEdgeSemantics>() <= 2_048);
    assert!(size_of::<DxfHatchBoundaryEllipticArcEdgeSemanticEntry>() <= 2_080);
    copy::<DxfHatchBoundaryEllipticArcEdgeSemantics>();
    copy::<DxfHatchBoundaryEllipticArcEdgeSemanticEntry>();
    send_sync::<DxfHatchBoundaryEllipticArcEdgeSemanticDirectory>();

    let excluded = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n92\n0\n93\n4\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n2\n10\n5\n20\n6\n40\n7\n50\n0\n51\n90\n73\n1\n\
         72\n4\n94\n3\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n\
         72\n.\n10\n8\n20\n9\n97\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&excluded, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_elliptic_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    assert!(directory.entries().is_empty());
    assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    assert!(directory.entries_for_path(1).is_none());
    Ok(())
}

fn semantic_bits(
    semantics: &DxfHatchBoundaryEllipticArcEdgeSemantics,
) -> Result<[u64; 7], io::Error> {
    let center = semantics
        .ocs_center_value()
        .ok_or_else(|| io::Error::other("OCS center"))?;
    let axis = semantics
        .ocs_major_axis_endpoint_value()
        .ok_or_else(|| io::Error::other("OCS major axis"))?;
    Ok([
        center[0].to_bits(),
        center[1].to_bits(),
        axis[0].to_bits(),
        axis[1].to_bits(),
        double_bits(semantics.minor_to_major_ratio())?,
        double_bits(semantics.start_angle_degrees())?,
        double_bits(semantics.end_angle_degrees())?,
    ])
}

fn double_bits(
    value: &DxfHatchBoundaryEllipticArcEdgeSemanticDoubleValue,
) -> Result<u64, io::Error> {
    value
        .value()
        .copied()
        .map(DxfDouble::to_bits)
        .ok_or_else(|| io::Error::other("usable double"))
}

#[derive(Clone, Copy)]
struct EllipseWire {
    doubles: [u64; 7],
    direction: i16,
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
    ellipses: &[EllipseWire],
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
    for ellipse in ellipses {
        push_i16(&mut bytes, version, 72, 3)?;
        for (code, bits) in [
            (10, ellipse.doubles[0]),
            (20, ellipse.doubles[1]),
            (11, ellipse.doubles[2]),
            (21, ellipse.doubles[3]),
            (40, ellipse.doubles[4]),
            (50, ellipse.doubles[5]),
            (51, ellipse.doubles[6]),
        ] {
            push_double_bits(&mut bytes, version, code, bits)?;
        }
        push_i16(&mut bytes, version, 73, ellipse.direction)?;
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

use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryCircularArcEdgeDirection, DxfHatchBoundaryCircularArcEdgeNumericIssue,
    DxfHatchBoundaryCircularArcEdgeSemanticDirectory,
    DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue,
    DxfHatchBoundaryCircularArcEdgeSemanticEntry, DxfHatchBoundaryCircularArcEdgeSemanticIssue,
    DxfHatchBoundaryCircularArcEdgeSemantics, DxfHatchBoundaryEdgeCountRelation,
    DxfHatchBoundaryEdgePathState, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_preserves_angles_centers_and_both_directions() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n2\n10\n-0\n20\n2.5\n40\n3.25\n50\n-450\n51\n810.25\n73\n0\n\
             72\n2\n10\n5\n20\n6\n40\n7\n50\n-45\n51\n360\n73\n1\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_circular_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(
            version,
            2,
            &[
                ArcWire {
                    doubles: [
                        (-0.0_f64).to_bits(),
                        2.5_f64.to_bits(),
                        3.25_f64.to_bits(),
                        (-450.0_f64).to_bits(),
                        810.25_f64.to_bits(),
                    ],
                    direction: 0,
                },
                ArcWire {
                    doubles: [
                        5.0_f64.to_bits(),
                        6.0_f64.to_bits(),
                        7.0_f64.to_bits(),
                        (-45.0_f64).to_bits(),
                        360.0_f64.to_bits(),
                    ],
                    direction: 1,
                },
            ],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_circular_arc_edge_semantic_directory(&DxfCancellationToken::default())?;

        let expected = [
            (
                [
                    (-0.0_f64).to_bits(),
                    2.5_f64.to_bits(),
                    3.25_f64.to_bits(),
                    (-450.0_f64).to_bits(),
                    810.25_f64.to_bits(),
                ],
                DxfHatchBoundaryCircularArcEdgeDirection::Clockwise,
            ),
            (
                [
                    5.0_f64.to_bits(),
                    6.0_f64.to_bits(),
                    7.0_f64.to_bits(),
                    (-45.0_f64).to_bits(),
                    360.0_f64.to_bits(),
                ],
                DxfHatchBoundaryCircularArcEdgeDirection::Counterclockwise,
            ),
        ];
        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.entries().len(), expected.len());
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            for (index, (entry, expected)) in directory.entries().iter().zip(expected).enumerate() {
                assert_eq!(entry.ordinal(), index as u64);
                assert_eq!(entry.numeric().edge_ordinal(), index as u64);
                assert_eq!(entry.numeric().path_ordinal(), 0);
                assert_eq!(directory.entry_for_edge(index as u64), Some(*entry));
                assert_eq!(semantic_signature(*entry)?, expected);
                let semantics = entry.semantics();
                assert_eq!(
                    semantics
                        .ocs_center_value()
                        .ok_or_else(|| io::Error::other("OCS center"))?
                        .map(DxfDouble::to_bits),
                    [expected.0[0], expected.0[1]]
                );
                assert_eq!(
                    semantics.radius_value().map(DxfDouble::to_bits),
                    Some(expected.0[2])
                );
                assert_eq!(
                    semantics
                        .start_angle_degrees_value()
                        .map(DxfDouble::to_bits),
                    Some(expected.0[3])
                );
                assert_eq!(
                    semantics.end_angle_degrees_value().map(DxfDouble::to_bits),
                    Some(expected.0[4])
                );
                assert_eq!(semantics.direction_value(), Some(expected.1));
                assert_eq!(expected.1.flag(), index as i16);
                assert_eq!(expected.1.is_counterclockwise(), index == 1);
                for (value, field_id) in [
                    (semantics.center_x(), "boundary_circular_arc_center_x"),
                    (semantics.center_y(), "boundary_circular_arc_center_y"),
                    (semantics.radius(), "boundary_circular_arc_radius"),
                    (
                        semantics.start_angle_degrees(),
                        "boundary_circular_arc_start_angle",
                    ),
                    (
                        semantics.end_angle_degrees(),
                        "boundary_circular_arc_end_angle",
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
                assert_eq!(
                    semantics.direction().field_provenance().schema_field_id(),
                    "boundary_circular_arc_counterclockwise"
                );
                assert!(semantics.direction().raw_provenance().is_some());
            }
        }
    }
    Ok(())
}

#[test]
fn required_and_numeric_failures_keep_exact_provenance() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n2\n\
         20\n2\n20\n3\n40\nbad\n51\n90\n73\n.\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_circular_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("ASCII semantic entry"))?;
    let semantics = entry.semantics();
    for value in [semantics.center_x(), semantics.start_angle_degrees()] {
        assert_eq!(
            value.invalid_issue(),
            Some(&DxfHatchBoundaryCircularArcEdgeSemanticIssue::MissingRequiredValue)
        );
        assert_eq!(value.raw_provenance(), None);
    }
    assert_eq!(
        semantics.center_y().invalid_issue(),
        Some(&DxfHatchBoundaryCircularArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryCircularArcEdgeNumericIssue::MultipleValues {
                occurrence_count: 2,
            }
        ))
    );
    assert_eq!(semantics.center_y().raw_provenance(), None);
    assert!(matches!(
        semantics.radius().invalid_issue(),
        Some(DxfHatchBoundaryCircularArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryCircularArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert!(semantics.radius().raw_provenance().is_some());
    assert!(matches!(
        semantics.direction().invalid_issue(),
        Some(DxfHatchBoundaryCircularArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryCircularArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert!(semantics.direction().raw_provenance().is_some());
    assert_eq!(
        semantics.end_angle_degrees().state(),
        DxfSemanticValueState::Explicit
    );

    let binary = binary_fixture(
        DxfAcadVersion::Ac1032,
        1,
        &[ArcWire {
            doubles: [
                0x7ff8_0000_0000_0042,
                2.0_f64.to_bits(),
                3.0_f64.to_bits(),
                4.0_f64.to_bits(),
                f64::INFINITY.to_bits(),
            ],
            direction: 1,
        }],
    )?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_circular_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("Binary semantic entry"))?
        .semantics()
        .to_owned();
    for value in [semantics.center_x(), semantics.end_angle_degrees()] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfHatchBoundaryCircularArcEdgeSemanticIssue::Numeric(
                DxfHatchBoundaryCircularArcEdgeNumericIssue::NonFiniteDouble(_)
            ))
        ));
        assert!(value.raw_provenance().is_some());
    }
    assert_eq!(
        semantics.direction_value(),
        Some(DxfHatchBoundaryCircularArcEdgeDirection::Counterclockwise)
    );
    Ok(())
}

#[test]
fn nonpositive_radius_and_direction_domain_retain_exact_raw_values() -> Result<(), Box<dyn Error>> {
    let radius_bits = [
        0.0_f64.to_bits(),
        (-0.0_f64).to_bits(),
        (-5.25_f64).to_bits(),
    ];
    let direction_values = [-1_i16, 2, i16::MIN];
    let arcs = radius_bits
        .into_iter()
        .zip(direction_values)
        .map(|(radius, direction)| ArcWire {
            doubles: [
                1.0_f64.to_bits(),
                2.0_f64.to_bits(),
                radius,
                (-720.0_f64).to_bits(),
                1080.0_f64.to_bits(),
            ],
            direction,
        })
        .collect::<Vec<_>>();
    let binary = binary_fixture(DxfAcadVersion::Ac1032, 3, &arcs)?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_circular_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 3);
    for ((entry, expected_radius), expected_direction) in directory
        .entries()
        .iter()
        .zip(radius_bits)
        .zip(direction_values)
    {
        let semantics = entry.semantics();
        assert_eq!(semantics.radius_value(), None);
        match semantics.radius().invalid_issue() {
            Some(DxfHatchBoundaryCircularArcEdgeSemanticIssue::NonPositiveRadius { radius }) => {
                assert_eq!(radius.to_bits(), expected_radius);
            }
            _ => return Err(io::Error::other("non-positive radius issue").into()),
        }
        assert!(semantics.radius().raw_provenance().is_some());
        assert_eq!(semantics.direction_value(), None);
        assert_eq!(
            semantics.direction().invalid_issue(),
            Some(
                &DxfHatchBoundaryCircularArcEdgeSemanticIssue::DirectionFlagOutOfDomain {
                    value: expected_direction,
                }
            )
        );
        assert!(semantics.direction().raw_provenance().is_some());
        assert_eq!(
            semantics
                .start_angle_degrees_value()
                .map(DxfDouble::to_bits),
            Some((-720.0_f64).to_bits())
        );
        assert_eq!(
            semantics.end_angle_degrees_value().map(DxfDouble::to_bits),
            Some(1080.0_f64.to_bits())
        );
    }
    Ok(())
}

#[test]
fn mismatch_exclusion_cancellation_bounds_traits_and_redaction_hold() -> Result<(), Box<dyn Error>>
{
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n\
         92\n0\n93\n0\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n2\n10\n5\n20\n6\n40\n12345.625\n50\n0\n51\n90\n73\n1\n\
         72\n.\n10\n8\n20\n9\n40\n10\n50\n0\n51\n90\n73\n1\n97\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_boundary_circular_arc_edge_semantic_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .hatch_boundary_circular_arc_edge_semantic_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one retained CircularArc semantic entry").into());
    };
    assert_eq!(entry.numeric().edge_ordinal(), 1);
    assert_eq!(directory.entry(0), Some(*entry));
    assert_eq!(directory.entry_for_edge(1), Some(*entry));
    assert!(directory.entry_for_edge(0).is_none());
    assert!(directory.entry_for_edge(2).is_none());
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(1).is_none());
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.numeric_directory().source_id()
    );
    assert_eq!(
        directory.numeric_directory().entries().len(),
        directory.entries().len()
    );
    let edge_directory = directory
        .numeric_directory()
        .card_directory()
        .edge_type_directory()
        .edge_directory();
    let DxfHatchBoundaryEdgePathState::Grouped(path) = edge_directory.paths()[0].state() else {
        return Err(io::Error::other("grouped mismatch path").into());
    };
    assert_eq!(
        path.count_relation(),
        DxfHatchBoundaryEdgeCountRelation::Mismatched {
            declared: 0,
            observed: 3,
        }
    );
    let debug = format!("{:?}", entry.semantics().radius());
    assert!(!debug.contains("12345.625"));
    assert!(size_of::<DxfHatchBoundaryCircularArcEdgeSemantics>() <= 1024);
    assert!(size_of::<DxfHatchBoundaryCircularArcEdgeSemanticEntry>() <= 2048);
    copy::<DxfHatchBoundaryCircularArcEdgeDirection>();
    copy::<DxfHatchBoundaryCircularArcEdgeSemantics>();
    copy::<DxfHatchBoundaryCircularArcEdgeSemanticEntry>();
    send_sync::<DxfHatchBoundaryCircularArcEdgeSemanticDirectory>();
    Ok(())
}

fn semantic_signature(
    entry: DxfHatchBoundaryCircularArcEdgeSemanticEntry,
) -> Result<([u64; 5], DxfHatchBoundaryCircularArcEdgeDirection), io::Error> {
    let semantics = entry.semantics();
    Ok((
        [
            explicit_double(semantics.center_x())?.to_bits(),
            explicit_double(semantics.center_y())?.to_bits(),
            explicit_double(semantics.radius())?.to_bits(),
            explicit_double(semantics.start_angle_degrees())?.to_bits(),
            explicit_double(semantics.end_angle_degrees())?.to_bits(),
        ],
        semantics
            .direction_value()
            .ok_or_else(|| io::Error::other("explicit direction"))?,
    ))
}

fn explicit_double(
    value: &DxfHatchBoundaryCircularArcEdgeSemanticDoubleValue,
) -> Result<DxfDouble, io::Error> {
    value
        .value()
        .copied()
        .ok_or_else(|| io::Error::other("explicit double"))
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
    )
    .into_bytes()
}

#[derive(Clone, Copy)]
struct ArcWire {
    doubles: [u64; 5],
    direction: i16,
}

fn binary_fixture(
    version: DxfAcadVersion,
    declared_edge_count: i32,
    arcs: &[ArcWire],
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
    for arc in arcs {
        push_i16(&mut bytes, version, 72, 2)?;
        for (code, bits) in [
            (10, arc.doubles[0]),
            (20, arc.doubles[1]),
            (40, arc.doubles[2]),
            (50, arc.doubles[3]),
            (51, arc.doubles[4]),
        ] {
            push_double_bits(&mut bytes, version, code, bits)?;
        }
        push_i16(&mut bytes, version, 73, arc.direction)?;
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

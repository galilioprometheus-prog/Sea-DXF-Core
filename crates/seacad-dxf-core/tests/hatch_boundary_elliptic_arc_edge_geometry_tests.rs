use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryEdgeCountRelation, DxfHatchBoundaryEdgePathState,
    DxfHatchBoundaryEllipticArcEdgeDirection, DxfHatchBoundaryEllipticArcEdgeGeometryDirectory,
    DxfHatchBoundaryEllipticArcEdgeGeometryEntry, DxfHatchBoundaryEllipticArcEdgeGeometryIssue,
    DxfHatchBoundaryEllipticArcEdgeMajorAxisState, DxfHatchBoundaryEllipticArcEdgeNumericIssue,
    DxfHatchBoundaryEllipticArcEdgeOcsSegment, DxfHatchBoundaryEllipticArcEdgeSemanticIssue,
    DxfHatchBoundaryEllipticArcEdgeUnavailableValues, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_publishes_exact_unnormalized_ocs_segments() -> Result<(), Box<dyn Error>> {
    let ellipses = [
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
                360.0_f64.to_bits(),
                (-360.0_f64).to_bits(),
            ],
            direction: 1,
        },
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n3\n10\n-0\n20\n2.5\n11\n3\n21\n4\n40\n0.000001\n50\n-450\n51\n810.25\n73\n0\n\
             72\n3\n10\n5\n20\n6\n11\n-7\n21\n8\n40\n1\n50\n360\n51\n-360\n73\n1\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_elliptic_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(version, 2, &ellipses)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_elliptic_arc_edge_geometry_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.entries().len(), ellipses.len());
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            for (index, (entry, expected)) in directory.entries().iter().zip(ellipses).enumerate() {
                assert_eq!(entry.ordinal(), index as u64);
                assert_eq!(entry.semantics().numeric().edge_ordinal(), index as u64);
                assert_eq!(entry.semantics().numeric().path_ordinal(), 0);
                assert_eq!(directory.entry_for_edge(index as u64), Some(*entry));
                let geometry = segment(*entry)?;
                assert_eq!(
                    geometry.center().map(DxfDouble::to_bits),
                    expected.doubles[0..2]
                );
                assert_eq!(
                    geometry
                        .major_axis_endpoint_relative_to_center()
                        .map(DxfDouble::to_bits),
                    expected.doubles[2..4]
                );
                assert_eq!(
                    geometry.minor_to_major_ratio().to_bits(),
                    expected.doubles[4]
                );
                assert_eq!(
                    geometry.start_angle_degrees().to_bits(),
                    expected.doubles[5]
                );
                assert_eq!(geometry.end_angle_degrees().to_bits(), expected.doubles[6]);
                assert_eq!(
                    geometry.direction(),
                    if expected.direction == 0 {
                        DxfHatchBoundaryEllipticArcEdgeDirection::Clockwise
                    } else {
                        DxfHatchBoundaryEllipticArcEdgeDirection::Counterclockwise
                    }
                );
            }
        }
    }
    Ok(())
}

#[test]
fn unavailable_mask_retains_every_lower_semantic_issue() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n3\n\
         20\n2\n20\n3\n11\nbad\n40\n0\n50\nbad\n73\n2\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_elliptic_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("OCS EllipticArc entry"))?;
    let Err(DxfHatchBoundaryEllipticArcEdgeGeometryIssue::ValuesUnavailable(mask)) =
        entry.geometry()
    else {
        return Err(io::Error::other("unavailable EllipticArc values").into());
    };
    assert_mask(mask, [true, true, true, true, true, true, true, true], 8);

    let semantic_entry = entry.semantics();
    let semantics = semantic_entry.semantics();
    assert_eq!(
        semantics.center_x().invalid_issue(),
        Some(&DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(
        semantics.center_y().invalid_issue(),
        Some(&DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::MultipleValues {
                occurrence_count: 2,
            }
        ))
    );
    assert!(matches!(
        semantics.major_axis_endpoint_x().invalid_issue(),
        Some(DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert_eq!(
        semantics.major_axis_endpoint_y().invalid_issue(),
        Some(&DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MissingRequiredValue)
    );
    assert!(matches!(
        semantics.minor_to_major_ratio().invalid_issue(),
        Some(DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MinorToMajorRatioOutOfDomain {
            ratio
        }) if ratio.to_bits() == 0.0_f64.to_bits()
    ));
    assert!(matches!(
        semantics.start_angle_degrees().invalid_issue(),
        Some(DxfHatchBoundaryEllipticArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert_eq!(
        semantics.end_angle_degrees().invalid_issue(),
        Some(&DxfHatchBoundaryEllipticArcEdgeSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(
        semantics.direction().invalid_issue(),
        Some(&DxfHatchBoundaryEllipticArcEdgeSemanticIssue::DirectionFlagOutOfDomain { value: 2 })
    );
    assert!(matches!(
        semantics.major_axis_state(),
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::ComponentsUnavailable(_)
    ));
    Ok(())
}

#[test]
fn major_axis_relation_and_count_mismatch_preserve_geometry_policy() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n0\n\
         72\n3\n10\n1\n20\n2\n11\n0\n21\n-0\n40\n0.5\n50\n0\n51\n90\n73\n0\n\
         72\n3\n10\n1\n20\n2\n11\n1.7976931348623157e308\n21\n1.7976931348623157e308\n40\n0.5\n50\n0\n51\n90\n73\n1\n\
         72\n3\n10\n1\n20\n2\n11\n3\n21\n4\n40\n0.5\n50\n720\n51\n720\n73\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_elliptic_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 3);
    for (entry, expected) in directory.entries()[..2].iter().zip([
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::Degenerate,
        DxfHatchBoundaryEllipticArcEdgeMajorAxisState::NonFiniteDerivedMagnitude,
    ]) {
        assert_eq!(
            entry.geometry(),
            Err(DxfHatchBoundaryEllipticArcEdgeGeometryIssue::MajorAxisUnavailable(expected,))
        );
        assert_eq!(entry.semantics().semantics().major_axis_state(), expected);
    }
    let geometry = segment(directory.entries()[2])?;
    assert_eq!(geometry.start_angle_degrees().to_f64(), 720.0);
    assert_eq!(geometry.end_angle_degrees().to_f64(), 720.0);
    assert_eq!(
        geometry.direction(),
        DxfHatchBoundaryEllipticArcEdgeDirection::Clockwise
    );
    let edge_directory = directory
        .semantic_directory()
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
    Ok(())
}

#[test]
fn exclusion_cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>>
{
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n\
         92\n0\n93\n4\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n2\n10\n5\n20\n6\n40\n7\n50\n0\n51\n90\n73\n1\n\
         72\n3\n10\n8\n20\n9\n11\n12345.625\n21\n4\n40\n0.5\n50\n0\n51\n90\n73\n1\n\
         72\n.\n10\n8\n20\n9\n97\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_elliptic_arc_edge_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document
        .hatch_boundary_elliptic_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one OCS EllipticArc geometry").into());
    };
    assert_eq!(entry.semantics().numeric().edge_ordinal(), 2);
    assert_eq!(directory.entry(0), Some(*entry));
    assert_eq!(directory.entry_for_edge(2), Some(*entry));
    assert!(directory.entry_for_edge(0).is_none());
    assert!(directory.entry_for_edge(1).is_none());
    assert!(directory.entry_for_edge(3).is_none());
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(1).is_none());
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.semantic_directory().source_id()
    );
    assert!(!format!("{entry:?}").contains("12345.625"));
    assert!(size_of::<DxfHatchBoundaryEllipticArcEdgeOcsSegment>() <= 96);
    assert!(size_of::<DxfHatchBoundaryEllipticArcEdgeGeometryEntry>() <= 4096);
    copy::<DxfHatchBoundaryEllipticArcEdgeOcsSegment>();
    copy::<DxfHatchBoundaryEllipticArcEdgeUnavailableValues>();
    copy::<DxfHatchBoundaryEllipticArcEdgeGeometryEntry>();
    send_sync::<DxfHatchBoundaryEllipticArcEdgeGeometryDirectory>();
    Ok(())
}

fn assert_mask(
    mask: DxfHatchBoundaryEllipticArcEdgeUnavailableValues,
    expected: [bool; 8],
    count: u32,
) {
    assert_eq!(
        [
            mask.center_x(),
            mask.center_y(),
            mask.major_axis_x(),
            mask.major_axis_y(),
            mask.ratio(),
            mask.start_angle(),
            mask.end_angle(),
            mask.direction(),
        ],
        expected
    );
    assert_eq!(mask.count(), count);
}

fn segment(
    entry: DxfHatchBoundaryEllipticArcEdgeGeometryEntry,
) -> Result<DxfHatchBoundaryEllipticArcEdgeOcsSegment, io::Error> {
    entry
        .geometry()
        .map_err(|_| io::Error::other("usable OCS EllipticArc geometry"))
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
    )
    .into_bytes()
}

#[derive(Clone, Copy)]
struct EllipseWire {
    doubles: [u64; 7],
    direction: i16,
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

use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryCircularArcEdgeDirection, DxfHatchBoundaryCircularArcEdgeGeometryDirectory,
    DxfHatchBoundaryCircularArcEdgeGeometryEntry, DxfHatchBoundaryCircularArcEdgeGeometryIssue,
    DxfHatchBoundaryCircularArcEdgeNumericIssue, DxfHatchBoundaryCircularArcEdgeOcsSegment,
    DxfHatchBoundaryCircularArcEdgeSemanticIssue, DxfHatchBoundaryCircularArcEdgeUnavailableValues,
    DxfHatchBoundaryEdgeCountRelation, DxfHatchBoundaryEdgePathState, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_publishes_exact_unnormalized_ocs_segments() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n2\n10\n-0\n20\n2.5\n40\n3.25\n50\n-450\n51\n810.25\n73\n0\n\
             72\n2\n10\n5\n20\n6\n40\n7\n50\n360\n51\n-360\n73\n1\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_circular_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
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
                        360.0_f64.to_bits(),
                        (-360.0_f64).to_bits(),
                    ],
                    direction: 1,
                },
            ],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_circular_arc_edge_geometry_directory(&DxfCancellationToken::default())?;

        let expected = [
            (
                [(-0.0_f64).to_bits(), 2.5_f64.to_bits()],
                3.25_f64.to_bits(),
                (-450.0_f64).to_bits(),
                810.25_f64.to_bits(),
                DxfHatchBoundaryCircularArcEdgeDirection::Clockwise,
            ),
            (
                [5.0_f64.to_bits(), 6.0_f64.to_bits()],
                7.0_f64.to_bits(),
                360.0_f64.to_bits(),
                (-360.0_f64).to_bits(),
                DxfHatchBoundaryCircularArcEdgeDirection::Counterclockwise,
            ),
        ];
        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.entries().len(), expected.len());
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            for (index, (entry, expected)) in directory.entries().iter().zip(expected).enumerate() {
                assert_eq!(entry.ordinal(), index as u64);
                assert_eq!(entry.semantics().numeric().edge_ordinal(), index as u64);
                assert_eq!(entry.semantics().numeric().path_ordinal(), 0);
                assert_eq!(directory.entry_for_edge(index as u64), Some(*entry));
                let geometry = segment(*entry)?;
                assert_eq!(geometry.center().map(DxfDouble::to_bits), expected.0);
                assert_eq!(geometry.radius().to_bits(), expected.1);
                assert_eq!(geometry.start_angle_degrees().to_bits(), expected.2);
                assert_eq!(geometry.end_angle_degrees().to_bits(), expected.3);
                assert_eq!(geometry.direction(), expected.4);
            }
        }
    }
    Ok(())
}

#[test]
fn unavailable_mask_retains_every_lower_semantic_issue() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n2\n\
         20\n2\n20\n3\n40\n0\n50\nbad\n73\n2\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_circular_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("OCS CircularArc entry"))?;
    let Err(DxfHatchBoundaryCircularArcEdgeGeometryIssue::ValuesUnavailable(mask)) =
        entry.geometry()
    else {
        return Err(io::Error::other("unavailable CircularArc values").into());
    };
    assert_mask(mask, [true, true, true, true, true, true], 6);

    let semantics_entry = entry.semantics();
    let semantics = semantics_entry.semantics();
    assert_eq!(
        semantics.center_x().invalid_issue(),
        Some(&DxfHatchBoundaryCircularArcEdgeSemanticIssue::MissingRequiredValue)
    );
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
        Some(DxfHatchBoundaryCircularArcEdgeSemanticIssue::NonPositiveRadius { radius })
            if radius.to_bits() == 0.0_f64.to_bits()
    ));
    assert!(semantics.radius().raw_provenance().is_some());
    assert!(matches!(
        semantics.start_angle_degrees().invalid_issue(),
        Some(DxfHatchBoundaryCircularArcEdgeSemanticIssue::Numeric(
            DxfHatchBoundaryCircularArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert_eq!(
        semantics.end_angle_degrees().invalid_issue(),
        Some(&DxfHatchBoundaryCircularArcEdgeSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(
        semantics.direction().invalid_issue(),
        Some(&DxfHatchBoundaryCircularArcEdgeSemanticIssue::DirectionFlagOutOfDomain { value: 2 })
    );
    Ok(())
}

#[test]
fn count_mismatch_and_equal_angles_keep_exact_geometry_without_sweep_policy()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n0\n\
         72\n2\n10\n1\n20\n2\n40\n5\n50\n720\n51\n720\n73\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_circular_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry_for_edge(0)
        .ok_or_else(|| io::Error::other("count-mismatched OCS CircularArc"))?;
    let geometry = segment(entry)?;
    assert_eq!(geometry.start_angle_degrees().to_f64(), 720.0);
    assert_eq!(geometry.end_angle_degrees().to_f64(), 720.0);
    assert_eq!(
        geometry.direction(),
        DxfHatchBoundaryCircularArcEdgeDirection::Clockwise
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
            observed: 1,
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
         92\n0\n93\n3\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n2\n10\n5\n20\n6\n40\n12345.625\n50\n0\n51\n90\n73\n1\n\
         72\n.\n10\n8\n20\n9\n40\n10\n50\n0\n51\n90\n73\n1\n97\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_circular_arc_edge_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document
        .hatch_boundary_circular_arc_edge_geometry_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one OCS CircularArc geometry").into());
    };
    assert_eq!(entry.semantics().numeric().edge_ordinal(), 1);
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
        directory.semantic_directory().source_id()
    );
    assert!(!format!("{entry:?}").contains("12345.625"));
    assert!(size_of::<DxfHatchBoundaryCircularArcEdgeOcsSegment>() <= 64);
    assert!(size_of::<DxfHatchBoundaryCircularArcEdgeGeometryEntry>() <= 4096);
    copy::<DxfHatchBoundaryCircularArcEdgeOcsSegment>();
    copy::<DxfHatchBoundaryCircularArcEdgeUnavailableValues>();
    copy::<DxfHatchBoundaryCircularArcEdgeGeometryEntry>();
    send_sync::<DxfHatchBoundaryCircularArcEdgeGeometryDirectory>();
    Ok(())
}

fn assert_mask(
    mask: DxfHatchBoundaryCircularArcEdgeUnavailableValues,
    expected: [bool; 6],
    count: u32,
) {
    assert_eq!(
        [
            mask.center_x(),
            mask.center_y(),
            mask.radius(),
            mask.start_angle(),
            mask.end_angle(),
            mask.direction(),
        ],
        expected
    );
    assert_eq!(mask.count(), count);
}

fn segment(
    entry: DxfHatchBoundaryCircularArcEdgeGeometryEntry,
) -> Result<DxfHatchBoundaryCircularArcEdgeOcsSegment, io::Error> {
    entry
        .geometry()
        .map_err(|_| io::Error::other("usable OCS CircularArc geometry"))
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

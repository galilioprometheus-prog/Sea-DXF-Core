use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchBoundaryEllipticArcEdgeNumericComponents,
    DxfHatchBoundaryEllipticArcEdgeNumericDirectory, DxfHatchBoundaryEllipticArcEdgeNumericEntry,
    DxfHatchBoundaryEllipticArcEdgeNumericIssue, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_decodes_finite_ascii_binary_elliptic_arc_components() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
             72\n3\n10\n-0\n20\n2.5\n11\n-3.25\n21\n4.5\n40\n0.5\n50\n-45\n51\n360\n73\n-1\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_elliptic_arc_edge_numeric_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(
            version,
            2,
            [
                (-0.0_f64).to_bits(),
                2.5_f64.to_bits(),
                (-3.25_f64).to_bits(),
                4.5_f64.to_bits(),
                0.5_f64.to_bits(),
                (-45.0_f64).to_bits(),
                360.0_f64.to_bits(),
            ],
            -1,
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_elliptic_arc_edge_numeric_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            let [entry] = directory.entries() else {
                return Err(io::Error::other("one numeric EllipticArc edge").into());
            };
            assert_eq!(entry.ordinal(), 0);
            assert_eq!(entry.edge_ordinal(), 1);
            assert_eq!(entry.path_ordinal(), 0);
            assert_eq!(directory.entry_for_edge(1), Some(*entry));
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            let components = entry.components();
            assert_eq!(
                double_number(components.center_x())?.to_bits(),
                (-0.0_f64).to_bits()
            );
            assert_eq!(double_number(components.center_y())?.to_f64(), 2.5);
            assert_eq!(
                double_number(components.major_axis_endpoint_x())?.to_f64(),
                -3.25
            );
            assert_eq!(
                double_number(components.major_axis_endpoint_y())?.to_f64(),
                4.5
            );
            assert_eq!(
                double_number(components.minor_to_major_ratio())?.to_f64(),
                0.5
            );
            assert_eq!(double_number(components.start_angle())?.to_f64(), -45.0);
            assert_eq!(double_number(components.end_angle())?.to_f64(), 360.0);
            assert_eq!(integer_number(components.counterclockwise())?, -1);
            for (value, field_id) in [
                (components.center_x(), "boundary_elliptic_arc_center_x"),
                (components.center_y(), "boundary_elliptic_arc_center_y"),
                (
                    components.major_axis_endpoint_x(),
                    "boundary_elliptic_arc_major_axis_endpoint_x",
                ),
                (
                    components.major_axis_endpoint_y(),
                    "boundary_elliptic_arc_major_axis_endpoint_y",
                ),
                (
                    components.minor_to_major_ratio(),
                    "boundary_elliptic_arc_minor_to_major_ratio",
                ),
                (
                    components.start_angle(),
                    "boundary_elliptic_arc_start_angle",
                ),
                (components.end_angle(), "boundary_elliptic_arc_end_angle"),
            ] {
                assert_eq!(value.state(), DxfSemanticValueState::Explicit);
                assert_eq!(value.field_provenance().schema_namespace(), "entity.hatch");
                assert_eq!(value.field_provenance().schema_field_id(), field_id);
                assert!(value.raw_provenance().is_some());
            }
            let direction = components.counterclockwise();
            assert_eq!(direction.state(), DxfSemanticValueState::Explicit);
            assert_eq!(
                direction.field_provenance().schema_field_id(),
                "boundary_elliptic_arc_counterclockwise"
            );
            assert!(direction.raw_provenance().is_some());
        }
    }
    Ok(())
}

#[test]
fn absent_multiple_and_malformed_double_and_integer_values_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n3\n\
         10\nbad\n20\n2\n20\n3\n11\n-4\n40\n0.5\n51\n90\n73\n.\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_elliptic_arc_edge_numeric_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("numeric EllipticArc entry"))?;
    let components = entry.components();
    assert!(matches!(
        components.center_x().invalid_issue(),
        Some(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        )
    ));
    assert!(components.center_x().raw_provenance().is_some());
    assert_eq!(
        components.center_y().invalid_issue(),
        Some(
            &DxfHatchBoundaryEllipticArcEdgeNumericIssue::MultipleValues {
                occurrence_count: 2
            }
        )
    );
    assert_eq!(components.center_y().raw_provenance(), None);
    assert_eq!(
        components.major_axis_endpoint_x().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        components.major_axis_endpoint_y().state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(
        components.minor_to_major_ratio().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        components.start_angle().state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(
        components.end_angle().state(),
        DxfSemanticValueState::Explicit
    );
    assert!(matches!(
        components.counterclockwise().invalid_issue(),
        Some(
            DxfHatchBoundaryEllipticArcEdgeNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        )
    ));
    assert!(components.counterclockwise().raw_provenance().is_some());
    Ok(())
}

#[test]
fn nonfinite_count_mismatch_and_non_elliptic_paths_remain_fail_closed() -> Result<(), Box<dyn Error>>
{
    let binary = binary_fixture(
        DxfAcadVersion::Ac1032,
        0,
        [
            0x7ff8_0000_0000_0042,
            2.0_f64.to_bits(),
            3.0_f64.to_bits(),
            4.0_f64.to_bits(),
            0.5_f64.to_bits(),
            6.0_f64.to_bits(),
            f64::INFINITY.to_bits(),
        ],
        i16::MIN,
    )?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_elliptic_arc_edge_numeric_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry_for_edge(1)
        .ok_or_else(|| io::Error::other("count-mismatched EllipticArc entry"))?;
    for value in [
        entry.components().center_x(),
        entry.components().end_angle(),
    ] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfHatchBoundaryEllipticArcEdgeNumericIssue::NonFiniteDouble(_))
        ));
        assert!(value.raw_provenance().is_some());
    }
    assert_eq!(
        integer_number(entry.components().counterclockwise())?,
        i16::MIN
    );
    assert!(directory.entry_for_edge(0).is_none());

    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n\
         92\n0\n93\n4\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n2\n10\n5\n20\n6\n40\n7\n50\n0\n51\n90\n73\n1\n\
         72\n4\n94\n3\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n\
         72\n.\n10\n8\n20\n9\n97\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_elliptic_arc_edge_numeric_directory(&DxfCancellationToken::default())?;
    assert!(directory.entries().is_empty());
    assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    assert!(directory.entries_for_path(1).is_none());
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n3\n\
         10\n1\n20\n2\n11\n3\n21\n4\n40\n12345.625\n50\n0\n51\n90\n73\n1\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_elliptic_arc_edge_numeric_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .hatch_boundary_elliptic_arc_edge_numeric_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert!(size_of::<DxfHatchBoundaryEllipticArcEdgeNumericComponents>() <= 1_024);
    assert!(size_of::<DxfHatchBoundaryEllipticArcEdgeNumericEntry>() <= 1_040);
    let debug = format!(
        "{:?}",
        directory.entries()[0].components().minor_to_major_ratio()
    );
    assert!(!debug.contains("12345.625"));
    copy::<DxfHatchBoundaryEllipticArcEdgeNumericComponents>();
    copy::<DxfHatchBoundaryEllipticArcEdgeNumericEntry>();
    send_sync::<DxfHatchBoundaryEllipticArcEdgeNumericDirectory>();
    Ok(())
}

fn double_number(
    value: &seacad_dxf_core::DxfHatchBoundaryEllipticArcEdgeNumericDoubleValue,
) -> Result<DxfDouble, io::Error> {
    value
        .value()
        .copied()
        .ok_or_else(|| io::Error::other("explicit double"))
}

fn integer_number(
    value: &seacad_dxf_core::DxfHatchBoundaryEllipticArcEdgeNumericIntegerValue,
) -> Result<i16, io::Error> {
    value
        .value()
        .copied()
        .ok_or_else(|| io::Error::other("explicit integer"))
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
    ellipse_bits: [u64; 7],
    counterclockwise: i16,
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
    for (code, value) in [(10, 1.0), (20, 2.0), (11, 3.0), (21, 4.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 72, 3)?;
    for (code, bits) in [
        (10, ellipse_bits[0]),
        (20, ellipse_bits[1]),
        (11, ellipse_bits[2]),
        (21, ellipse_bits[3]),
        (40, ellipse_bits[4]),
        (50, ellipse_bits[5]),
        (51, ellipse_bits[6]),
    ] {
        push_double_bits(&mut bytes, version, code, bits)?;
    }
    push_i16(&mut bytes, version, 73, counterclockwise)?;
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

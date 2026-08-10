use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundarySplineEdgePointKind,
    DxfHatchBoundarySplineEdgePointNumericDirectory, DxfHatchBoundarySplineEdgePointNumericEntry,
    DxfHatchBoundarySplineEdgePointNumericIssue, DxfHatchBoundarySplineEdgePointNumericValue,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

type ValueEvidence = (DxfSemanticValueState, Option<u64>);
type EntryEvidence = (
    DxfHatchBoundarySplineEdgePointKind,
    u64,
    ValueEvidence,
    ValueEvidence,
    Option<ValueEvidence>,
);

#[test]
fn every_dialect_decodes_exact_control_and_fit_point_numbers() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(version, complete_payload());
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_spline_edge_point_numeric_directory(&DxfCancellationToken::default())?;

        let binary = complete_binary_fixture(version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_spline_edge_point_numeric_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_complete_numbers(directory)?;
        }
        assert_eq!(evidence(&ascii_directory), evidence(&binary_directory));
    }
    Ok(())
}

#[test]
fn malformed_nonfinite_missing_and_multiple_components_remain_typed() -> Result<(), Box<dyn Error>>
{
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n94\n1\n73\n1\n74\n0\n95\n0\n96\n2\n\
         10\n1\n20\n2\n20\n3\n10\nbad-x\n97\n1\n11\n4\n21\nbad-y\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_point_numeric_directory(&DxfCancellationToken::default())?;
    let controls = directory
        .entries_for_edge(0, DxfHatchBoundarySplineEdgePointKind::ControlPoint)
        .ok_or_else(|| io::Error::other("control entries"))?;
    assert_eq!(controls.len(), 2);
    assert!(matches!(
        controls[0].components().y().invalid_issue(),
        Some(
            DxfHatchBoundarySplineEdgePointNumericIssue::MultipleValues {
                occurrence_count: 2
            }
        )
    ));
    assert_eq!(controls[0].components().y().raw_provenance(), None);
    let weight = controls[0]
        .components()
        .weight()
        .ok_or_else(|| io::Error::other("control weight"))?;
    assert_eq!(weight.state(), DxfSemanticValueState::Absent);
    assert_eq!(weight.raw_provenance(), None);
    assert!(matches!(
        controls[1].components().x().invalid_issue(),
        Some(DxfHatchBoundarySplineEdgePointNumericIssue::InvalidAsciiNumber(_))
    ));
    assert!(controls[1].components().x().raw_provenance().is_some());
    assert_eq!(
        controls[1].components().y().state(),
        DxfSemanticValueState::Absent
    );
    let fits = directory
        .entries_for_edge(0, DxfHatchBoundarySplineEdgePointKind::FitPoint)
        .ok_or_else(|| io::Error::other("fit entries"))?;
    assert_eq!(fits.len(), 1);
    assert!(matches!(
        fits[0].components().y().invalid_issue(),
        Some(DxfHatchBoundarySplineEdgePointNumericIssue::InvalidAsciiNumber(_))
    ));
    assert!(fits[0].components().y().raw_provenance().is_some());
    assert!(fits[0].components().weight().is_none());

    let binary = nonfinite_binary_fixture(DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_spline_edge_point_numeric_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("non-finite entry"))?;
    assert!(matches!(
        entry.components().x().invalid_issue(),
        Some(DxfHatchBoundarySplineEdgePointNumericIssue::NonFiniteDouble(value))
            if value.to_bits() == f64::INFINITY.to_bits()
    ));
    assert!(entry.components().x().raw_provenance().is_some());
    Ok(())
}

#[test]
fn empty_unavailable_mismatched_and_non_spline_edges_remain_distinct() -> Result<(), Box<dyn Error>>
{
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n3\n\
         92\n0\n93\n2\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n97\n0\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n41\n9\n97\n0\n\
         92\n0\n93\n1\n72\n1\n10\n0\n20\n0\n11\n1\n21\n1\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_point_numeric_directory(&DxfCancellationToken::default())?;
    assert!(directory.entries().is_empty());
    assert!(directory.card_directory().cards().is_empty());
    assert_eq!(
        directory
            .card_directory()
            .point_tuple_directory()
            .entries()
            .len(),
        2
    );
    for kind in [
        DxfHatchBoundarySplineEdgePointKind::ControlPoint,
        DxfHatchBoundarySplineEdgePointKind::FitPoint,
    ] {
        assert!(
            directory
                .entries_for_edge(0, kind)
                .is_some_and(<[_]>::is_empty)
        );
        assert!(directory.entries_for_edge(1, kind).is_none());
        assert!(directory.entries_for_edge(2, kind).is_none());
    }
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n1\n\
         10\n12345.625\n20\n2\n42\n1\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_spline_edge_point_numeric_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .hatch_boundary_spline_edge_point_numeric_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_tuple(u64::MAX).is_none());
    assert!(
        directory
            .entries_for_edge(u64::MAX, DxfHatchBoundarySplineEdgePointKind::ControlPoint,)
            .is_none()
    );
    assert!(size_of::<DxfHatchBoundarySplineEdgePointNumericEntry>() <= 528);
    assert!(!format!("{:?}", directory.entries()[0]).contains("12345.625"));
    copy::<DxfHatchBoundarySplineEdgePointNumericEntry>();
    send_sync::<DxfHatchBoundarySplineEdgePointNumericDirectory>();
    Ok(())
}

fn assert_complete_numbers(
    directory: &DxfHatchBoundarySplineEdgePointNumericDirectory,
) -> Result<(), io::Error> {
    assert_eq!(directory.entries().len(), 4);
    let controls = directory
        .entries_for_edge(0, DxfHatchBoundarySplineEdgePointKind::ControlPoint)
        .ok_or_else(|| io::Error::other("control entries"))?;
    assert_eq!(controls.len(), 2);
    assert_eq!(
        controls[0]
            .components()
            .x()
            .value()
            .map(|value| value.to_bits()),
        Some(0.0_f64.to_bits())
    );
    assert_eq!(
        controls[0]
            .components()
            .y()
            .value()
            .map(|value| value.to_bits()),
        Some((-0.0_f64).to_bits())
    );
    assert_eq!(
        controls[0]
            .components()
            .weight()
            .and_then(DxfHatchBoundarySplineEdgePointNumericValue::value)
            .map(|value| value.to_bits()),
        Some(2.0_f64.to_bits())
    );
    assert_eq!(
        controls[1]
            .components()
            .weight()
            .map(DxfHatchBoundarySplineEdgePointNumericValue::state),
        Some(DxfSemanticValueState::Absent)
    );
    assert_eq!(
        controls[0]
            .components()
            .x()
            .field_provenance()
            .schema_field_id(),
        "boundary_spline_control_point_x"
    );
    assert_eq!(
        controls[0]
            .components()
            .y()
            .field_provenance()
            .schema_field_id(),
        "boundary_spline_control_point_y"
    );
    assert_eq!(
        controls[0]
            .components()
            .weight()
            .ok_or_else(|| io::Error::other("control weight"))?
            .field_provenance()
            .schema_field_id(),
        "boundary_spline_control_point_weight"
    );
    let fits = directory
        .entries_for_edge(0, DxfHatchBoundarySplineEdgePointKind::FitPoint)
        .ok_or_else(|| io::Error::other("fit entries"))?;
    assert_eq!(fits.len(), 2);
    assert!(
        fits.iter()
            .all(|entry| entry.components().weight().is_none())
    );
    assert_eq!(
        fits[0]
            .components()
            .x()
            .field_provenance()
            .schema_field_id(),
        "boundary_spline_fit_point_x"
    );
    assert_eq!(
        fits[0]
            .components()
            .y()
            .field_provenance()
            .schema_field_id(),
        "boundary_spline_fit_point_y"
    );
    for entry in directory.entries().iter().copied() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        assert_eq!(
            directory.entry_for_tuple(entry.point_tuple().ordinal()),
            Some(entry)
        );
    }
    Ok(())
}

fn evidence(directory: &DxfHatchBoundarySplineEdgePointNumericDirectory) -> Vec<EntryEvidence> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.point_tuple().kind(),
                entry.point_tuple().point_index(),
                value_evidence(entry.components().x()),
                value_evidence(entry.components().y()),
                entry.components().weight().map(value_evidence),
            )
        })
        .collect()
}

fn value_evidence(value: &DxfHatchBoundarySplineEdgePointNumericValue) -> ValueEvidence {
    (value.state(), value.value().map(|number| number.to_bits()))
}

const fn complete_payload() -> &'static str {
    "91\n1\n92\n0\n93\n1\n72\n4\n94\n2\n73\n1\n74\n0\n95\n2\n96\n2\n\
     40\n0\n40\n1\n10\n0\n20\n-0\n42\n2\n10\n1\n20\n2\n\
     97\n2\n11\n0.5\n21\n0.25\n11\n0.75\n21\n0.5\n97\n0\n"
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
    )
    .into_bytes()
}

fn complete_binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = binary_prefix(version)?;
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, 1)?;
    push_i16(&mut bytes, version, 72, 4)?;
    push_i32(&mut bytes, version, 94, 2)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i16(&mut bytes, version, 74, 0)?;
    push_i32(&mut bytes, version, 95, 2)?;
    push_i32(&mut bytes, version, 96, 2)?;
    for value in [0.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (x, y, weight) in [(0.0, -0.0, Some(2.0)), (1.0, 2.0, None)] {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
        if let Some(weight) = weight {
            push_double(&mut bytes, version, 42, weight)?;
        }
    }
    push_i32(&mut bytes, version, 97, 2)?;
    for (x, y) in [(0.5, 0.25), (0.75, 0.5)] {
        push_double(&mut bytes, version, 11, x)?;
        push_double(&mut bytes, version, 21, y)?;
    }
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn nonfinite_binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = binary_prefix(version)?;
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, 1)?;
    push_i16(&mut bytes, version, 72, 4)?;
    push_i32(&mut bytes, version, 94, 1)?;
    push_i16(&mut bytes, version, 73, 0)?;
    push_i16(&mut bytes, version, 74, 0)?;
    push_i32(&mut bytes, version, 95, 0)?;
    push_i32(&mut bytes, version, 96, 1)?;
    push_double(&mut bytes, version, 10, f64::INFINITY)?;
    push_double(&mut bytes, version, 20, 2.0)?;
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_prefix(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
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
    Ok(bytes)
}

fn binary_suffix(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_i32(bytes, version, 97, 0)?;
    push_i16(bytes, version, 75, 0)?;
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
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

fn copy<T: Copy>() {}

fn send_sync<T: Send + Sync>() {}

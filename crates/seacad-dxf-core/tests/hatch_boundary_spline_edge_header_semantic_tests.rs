use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchBoundarySplineEdgeHeaderPeriodicity, DxfHatchBoundarySplineEdgeHeaderRationality,
    DxfHatchBoundarySplineEdgeHeaderSemanticDirectory,
    DxfHatchBoundarySplineEdgeHeaderSemanticEntry, DxfHatchBoundarySplineEdgeHeaderSemanticIssue,
    DxfHatchBoundarySplineEdgeHeaderSemantics, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_exact_required_header_and_boolean_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
             72\n4\n94\n3\n73\n1\n74\n0\n95\n8\n96\n4\n\
             40\n0\n40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n40\n1\n\
             10\n0\n20\n0\n10\n1\n20\n0\n10\n1\n20\n1\n10\n0\n20\n1\n\
             97\n0\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_spline_edge_header_semantic_directory(
                &DxfCancellationToken::default(),
            )?;
        let binary = binary_fixture(version, 2, 3, 1, 0, 8, 4)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_spline_edge_header_semantic_directory(
                &DxfCancellationToken::default(),
            )?;

        for directory in [&ascii_directory, &binary_directory] {
            let [entry] = directory.entries() else {
                return Err(io::Error::other("one semantic Spline edge").into());
            };
            assert_eq!(entry.ordinal(), 0);
            assert_eq!(entry.numeric().edge_ordinal(), 1);
            assert_eq!(entry.numeric().path_ordinal(), 0);
            assert_eq!(directory.entry_for_edge(1), Some(*entry));
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            let semantics = entry.semantics();
            assert_eq!(i32_number(semantics.degree())?, 3);
            assert_eq!(
                semantics.rationality_value(),
                Some(DxfHatchBoundarySplineEdgeHeaderRationality::Rational)
            );
            assert_eq!(
                semantics.periodicity_value(),
                Some(DxfHatchBoundarySplineEdgeHeaderPeriodicity::NonPeriodic)
            );
            assert_eq!(i32_number(semantics.knot_count())?, 8);
            assert_eq!(i32_number(semantics.control_point_count())?, 4);
            assert!(
                semantics
                    .rationality_value()
                    .is_some_and(DxfHatchBoundarySplineEdgeHeaderRationality::is_rational)
            );
            assert!(
                semantics
                    .periodicity_value()
                    .is_some_and(|value| !value.is_periodic())
            );
            for value in [
                semantics.degree(),
                semantics.knot_count(),
                semantics.control_point_count(),
            ] {
                assert_eq!(value.state(), DxfSemanticValueState::Explicit);
                assert_eq!(value.field_provenance().schema_namespace(), "entity.hatch");
                assert!(value.raw_provenance().is_some());
            }
            assert_eq!(
                semantics.rationality().state(),
                DxfSemanticValueState::Explicit
            );
            assert_eq!(
                semantics.periodicity().state(),
                DxfSemanticValueState::Explicit
            );
            assert!(semantics.rationality().raw_provenance().is_some());
            assert!(semantics.periodicity().raw_provenance().is_some());
        }
    }
    Ok(())
}

#[test]
fn missing_duplicate_malformed_and_out_of_domain_flags_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n\
         73\n2\n74\n-1\n95\n4\n95\n5\n96\nbad\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_header_semantic_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("semantic Spline header entry"))?;
    let semantics = entry.semantics();
    assert_eq!(
        semantics.degree().invalid_issue(),
        Some(&DxfHatchBoundarySplineEdgeHeaderSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(semantics.degree().raw_provenance(), None);
    assert_eq!(
        semantics.rationality().invalid_issue(),
        Some(&DxfHatchBoundarySplineEdgeHeaderSemanticIssue::RationalFlagOutOfDomain { value: 2 })
    );
    assert!(semantics.rationality().raw_provenance().is_some());
    assert_eq!(
        semantics.periodicity().invalid_issue(),
        Some(&DxfHatchBoundarySplineEdgeHeaderSemanticIssue::PeriodicFlagOutOfDomain { value: -1 })
    );
    assert!(semantics.periodicity().raw_provenance().is_some());
    assert_eq!(
        semantics.knot_count().invalid_issue(),
        Some(&DxfHatchBoundarySplineEdgeHeaderSemanticIssue::Numeric(
            seacad_dxf_core::DxfHatchBoundarySplineEdgeHeaderNumericIssue::MultipleValues {
                occurrence_count: 2,
            }
        ))
    );
    assert_eq!(semantics.knot_count().raw_provenance(), None);
    assert!(matches!(
        semantics.control_point_count().invalid_issue(),
        Some(DxfHatchBoundarySplineEdgeHeaderSemanticIssue::Numeric(
            seacad_dxf_core::DxfHatchBoundarySplineEdgeHeaderNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert!(semantics.control_point_count().raw_provenance().is_some());
    Ok(())
}

#[test]
fn exact_signed_degree_and_counts_remain_open_while_non_spline_paths_publish_none()
-> Result<(), Box<dyn Error>> {
    let binary = binary_fixture(DxfAcadVersion::Ac1032, 0, -7, 0, 1, -4, 0)?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_spline_edge_header_semantic_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry_for_edge(1)
        .ok_or_else(|| io::Error::other("count-mismatched Spline semantic entry"))?;
    assert_eq!(i32_number(entry.semantics().degree())?, -7);
    assert_eq!(i32_number(entry.semantics().knot_count())?, -4);
    assert_eq!(i32_number(entry.semantics().control_point_count())?, 0);
    assert_eq!(
        entry.semantics().rationality_value(),
        Some(DxfHatchBoundarySplineEdgeHeaderRationality::NonRational)
    );
    assert_eq!(
        entry.semantics().periodicity_value(),
        Some(DxfHatchBoundarySplineEdgeHeaderPeriodicity::Periodic)
    );
    assert!(directory.entry_for_edge(0).is_none());

    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n\
         92\n0\n93\n1\n72\n3\n10\n5\n20\n6\n11\n1\n21\n0\n40\n0.5\n50\n0\n51\n90\n73\n1\n97\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_header_semantic_directory(&DxfCancellationToken::default())?;
    assert!(directory.entries().is_empty());
    assert_eq!(directory.entries_for_path(0), Some(&[][..]));
    assert!(directory.entries_for_path(1).is_none());
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n\
         94\n123456789\n73\n1\n74\n0\n95\n4\n96\n2\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_spline_edge_header_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .hatch_boundary_spline_edge_header_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.numeric_directory().source_id()
    );
    assert!(size_of::<DxfHatchBoundarySplineEdgeHeaderSemantics>() <= 704);
    assert!(size_of::<DxfHatchBoundarySplineEdgeHeaderSemanticEntry>() <= 1408);
    let debug = format!("{:?}", directory.entries()[0].semantics().degree());
    assert!(!debug.contains("123456789"));
    copy::<DxfHatchBoundarySplineEdgeHeaderSemantics>();
    copy::<DxfHatchBoundarySplineEdgeHeaderSemanticEntry>();
    send_sync::<DxfHatchBoundarySplineEdgeHeaderSemanticDirectory>();
    Ok(())
}

fn i32_number(
    value: &seacad_dxf_core::DxfHatchBoundarySplineEdgeHeaderSemanticI32Value,
) -> Result<i32, io::Error> {
    value
        .value()
        .copied()
        .ok_or_else(|| io::Error::other("explicit semantic i32"))
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
    degree: i32,
    rational: i16,
    periodic: i16,
    knot_count: i32,
    control_point_count: i32,
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
    push_i16(&mut bytes, version, 72, 4)?;
    push_i32(&mut bytes, version, 94, degree)?;
    push_i16(&mut bytes, version, 73, rational)?;
    push_i16(&mut bytes, version, 74, periodic)?;
    push_i32(&mut bytes, version, 95, knot_count)?;
    push_i32(&mut bytes, version, 96, control_point_count)?;
    for value in [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (code, value) in [
        (10, 0.0),
        (20, 0.0),
        (10, 1.0),
        (20, 0.0),
        (10, 1.0),
        (20, 1.0),
        (10, 0.0),
        (20, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 97, 0)?;
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

fn copy<T: Copy>() {}

fn send_sync<T: Send + Sync>() {}

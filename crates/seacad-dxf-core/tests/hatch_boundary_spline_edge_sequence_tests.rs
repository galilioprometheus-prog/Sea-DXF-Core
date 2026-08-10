use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundarySplineEdgeSequenceDirectory,
    DxfHatchBoundarySplineEdgeSequenceEntry, DxfHatchBoundarySplineEdgeSequenceIssue,
    DxfHatchBoundarySplineEdgeSequencePartition, DxfHatchBoundarySplineEdgeSequencePhase,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_partitions_all_documented_spline_sequence_phases() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
             72\n4\n94\n2\n73\n1\n74\n0\n95\n2\n96\n2\n\
             40\n0\n40\n1\n\
             10\n0\n20\n0\n42\n1\n10\n1\n20\n1\n42\n2\n\
             97\n1\n11\n0.5\n21\n0.5\n12\n1\n22\n0\n13\n0\n23\n1\n\
             97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_spline_edge_sequence_directory(&DxfCancellationToken::default())?;

        let binary = binary_fixture(version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_spline_edge_sequence_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_complete_sequence(directory)?;
        }
    }
    Ok(())
}

#[test]
fn empty_optional_phases_and_count_mismatch_remain_explicit() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n3\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_sequence_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one Spline sequence entry").into());
    };
    assert_eq!(
        entry
            .payload_partition_entry()
            .edge_type_entry()
            .edge()
            .ordinal(),
        1
    );
    let sequence = available(*entry)?;
    assert_eq!(
        codes(directory.header_fields_for_edge(1).unwrap_or_default()),
        [94, 73, 74, 95, 96]
    );
    assert!(
        directory
            .knot_fields_for_edge(1)
            .unwrap_or_default()
            .is_empty()
    );
    assert!(
        directory
            .control_point_fields_for_edge(1)
            .unwrap_or_default()
            .is_empty()
    );
    assert!(sequence.fit_data_count().is_none());
    assert!(
        directory
            .fit_point_fields_for_edge(1)
            .unwrap_or_default()
            .is_empty()
    );
    assert!(
        directory
            .start_tangent_fields_for_edge(1)
            .unwrap_or_default()
            .is_empty()
    );
    assert!(
        directory
            .end_tangent_fields_for_edge(1)
            .unwrap_or_default()
            .is_empty()
    );
    assert_eq!(directory.entry_for_edge(0), None);
    assert!(sequence.knot_range().is_empty());
    assert!(sequence.control_point_range().is_empty());
    assert!(sequence.fit_point_range().is_empty());
    Ok(())
}

#[test]
fn malformed_order_counts_and_unknown_fields_fail_typed_without_losing_edges()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n4\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n11\n1\n21\n2\n97\n0\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n97\n0\n97\n0\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n97\n1\n11\n1\n21\n2\n10\n3\n20\n4\n97\n0\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n1\n96\n0\n41\n9\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_sequence_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 4);
    assert!(matches!(
        directory.entries()[0].sequence(),
        Err(DxfHatchBoundarySplineEdgeSequenceIssue::FitDataCountAbsent { group })
            if group.group_code().value() == 11
    ));
    assert!(matches!(
        directory.entries()[1].sequence(),
        Err(DxfHatchBoundarySplineEdgeSequenceIssue::DuplicateFitDataCount { group })
            if group.group_code().value() == 97
    ));
    assert!(matches!(
        directory.entries()[2].sequence(),
        Err(DxfHatchBoundarySplineEdgeSequenceIssue::FieldOutOfOrder {
            group,
            phase: DxfHatchBoundarySplineEdgeSequencePhase::ControlPoints,
        }) if group.group_code().value() == 10
    ));
    assert!(matches!(
        directory.entries()[3].sequence(),
        Err(DxfHatchBoundarySplineEdgeSequenceIssue::UnexpectedField { group })
            if group.group_code().value() == 41
    ));
    for ordinal in 0..4 {
        let entry = directory
            .entry_for_edge(ordinal)
            .ok_or_else(|| io::Error::other("retained malformed Spline edge"))?;
        assert!(entry.sequence().is_err());
        assert!(directory.header_fields_for_edge(ordinal).is_none());
        assert!(directory.knot_fields_for_edge(ordinal).is_none());
    }
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n1\n96\n1\n\
         40\n12345.625\n10\n1\n20\n2\n42\n1\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_spline_edge_sequence_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.hatch_boundary_spline_edge_sequence_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.payload_partition_directory().source_id()
    );
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_edge(u64::MAX), None);
    assert!(directory.header_fields_for_edge(u64::MAX).is_none());
    assert!(directory.end_tangent_fields_for_edge(u64::MAX).is_none());
    assert!(size_of::<DxfHatchBoundarySplineEdgeSequencePartition>() <= 256);
    assert!(size_of::<DxfHatchBoundarySplineEdgeSequenceEntry>() <= 512);
    assert!(!format!("{:?}", directory.entries()[0]).contains("12345.625"));
    copy::<DxfHatchBoundarySplineEdgeSequencePhase>();
    copy::<DxfHatchBoundarySplineEdgeSequenceIssue>();
    copy::<DxfHatchBoundarySplineEdgeSequencePartition>();
    copy::<DxfHatchBoundarySplineEdgeSequenceEntry>();
    send_sync::<DxfHatchBoundarySplineEdgeSequenceDirectory>();
    Ok(())
}

fn assert_complete_sequence(
    directory: &DxfHatchBoundarySplineEdgeSequenceDirectory,
) -> Result<(), io::Error> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one Spline sequence entry"));
    };
    assert_eq!(entry.ordinal(), 0);
    assert_eq!(
        entry
            .payload_partition_entry()
            .edge_type_entry()
            .edge()
            .ordinal(),
        1
    );
    let sequence = available(*entry)?;
    assert_eq!(
        codes(
            directory
                .header_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("header"))?
        ),
        [94, 73, 74, 95, 96]
    );
    assert_eq!(
        codes(
            directory
                .knot_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("knots"))?
        ),
        [40, 40]
    );
    assert_eq!(
        codes(
            directory
                .control_point_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("control points"))?
        ),
        [10, 20, 42, 10, 20, 42]
    );
    assert_eq!(
        sequence
            .fit_data_count()
            .map(|field| field.group().group_code().value()),
        Some(97)
    );
    assert_eq!(
        codes(
            directory
                .fit_point_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("fit points"))?
        ),
        [11, 21]
    );
    assert_eq!(
        codes(
            directory
                .start_tangent_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("start tangent"))?
        ),
        [12, 22]
    );
    assert_eq!(
        codes(
            directory
                .end_tangent_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("end tangent"))?
        ),
        [13, 23]
    );
    assert_eq!(sequence.header_range().len(), 5);
    assert_eq!(sequence.knot_range().len(), 2);
    assert_eq!(sequence.control_point_range().len(), 6);
    assert_eq!(sequence.fit_point_range().len(), 2);
    assert_eq!(sequence.start_tangent_range().len(), 2);
    assert_eq!(sequence.end_tangent_range().len(), 2);
    Ok(())
}

fn available(
    entry: DxfHatchBoundarySplineEdgeSequenceEntry,
) -> Result<DxfHatchBoundarySplineEdgeSequencePartition, io::Error> {
    entry
        .sequence()
        .map_err(|_| io::Error::other("available Spline edge sequence"))
}

fn codes(fields: &[seacad_dxf_core::DxfFillMeshField]) -> Vec<i16> {
    fields
        .iter()
        .map(|field| field.group().group_code().value())
        .collect()
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
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
    push_i32(&mut bytes, version, 93, 2)?;
    push_i16(&mut bytes, version, 72, 1)?;
    for (code, value) in [(10, 1.0), (20, 2.0), (11, 3.0), (21, 4.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 72, 4)?;
    push_i32(&mut bytes, version, 94, 2)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i16(&mut bytes, version, 74, 0)?;
    push_i32(&mut bytes, version, 95, 2)?;
    push_i32(&mut bytes, version, 96, 2)?;
    for value in [0.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (x, y, weight) in [(0.0, 0.0, 1.0), (1.0, 1.0, 2.0)] {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
        push_double(&mut bytes, version, 42, weight)?;
    }
    push_i32(&mut bytes, version, 97, 1)?;
    for (code, value) in [
        (11, 0.5),
        (21, 0.5),
        (12, 1.0),
        (22, 0.0),
        (13, 0.0),
        (23, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
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

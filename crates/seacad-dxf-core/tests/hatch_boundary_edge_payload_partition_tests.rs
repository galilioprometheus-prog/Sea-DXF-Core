use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundaryEdgePayloadPartition,
    DxfHatchBoundaryEdgePayloadPartitionDirectory, DxfHatchBoundaryEdgePayloadPartitionEntry,
    DxfHatchBoundaryEdgePayloadPartitionIssue, DxfHatchBoundaryEdgeType, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_separates_final_spline_data_from_the_path_trailer() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
             72\n4\n94\n1\n73\n0\n74\n0\n95\n1\n96\n1\n\
             40\n0\n10\n5\n20\n6\n97\n0\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_edge_payload_partition_directory(&DxfCancellationToken::default())?;

        let binary = binary_fixture(version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_edge_payload_partition_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_standard_partition(directory)?;
        }
    }
    Ok(())
}

#[test]
fn final_spline_retains_fit_data_and_tangents_before_exact_source_references()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n\
         94\n2\n73\n1\n74\n0\n95\n2\n96\n2\n\
         40\n0\n40\n1\n10\n0\n20\n0\n42\n1\n10\n1\n20\n1\n42\n2\n\
         97\n1\n11\n0.5\n21\n0.5\n12\n1\n22\n0\n13\n0\n23\n1\n\
         97\n2\n330\nAA\n330\nBB\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_edge_payload_partition_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry_for_edge(0)
        .ok_or_else(|| io::Error::other("partition entry"))?;
    let partition = available(entry)?;
    assert!(partition.is_final_edge());
    assert_eq!(
        codes(
            directory
                .edge_data_fields_for_edge(0)
                .ok_or_else(|| io::Error::other("Spline edge data"))?
        ),
        [
            94, 73, 74, 95, 96, 40, 40, 10, 20, 42, 10, 20, 42, 97, 11, 21, 12, 22, 13, 23,
        ]
    );
    let trailer = directory
        .path_trailer_fields_for_edge(0)
        .ok_or_else(|| io::Error::other("path trailer"))?;
    assert_eq!(codes(trailer), [97, 330, 330]);
    let count = partition
        .source_boundary_count()
        .ok_or_else(|| io::Error::other("source-boundary count"))?;
    assert_eq!(count.group().group_code().value(), 97);
    assert_eq!(count, trailer[0]);
    Ok(())
}

#[test]
fn missing_or_structurally_invalid_path_trailers_fail_closed_per_final_edge()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n3\n\
         92\n0\n93\n1\n72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n1\n96\n1\n\
         40\n0\n10\n5\n20\n6\n97\n1\n11\n5\n21\n6\n\
         92\n0\n93\n1\n72\n1\n10\n7\n20\n8\n11\n9\n21\n10\n\
         330\nAA\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_edge_payload_partition_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 3);
    assert_eq!(
        directory.entries()[0].partition(),
        Err(DxfHatchBoundaryEdgePayloadPartitionIssue::SourceBoundaryCountAbsent)
    );
    assert!(matches!(
        directory.entries()[1].partition(),
        Err(DxfHatchBoundaryEdgePayloadPartitionIssue::UnexpectedTrailerField { group })
            if group.group_code().value() == 11
    ));
    assert!(matches!(
        directory.entries()[2].partition(),
        Err(DxfHatchBoundaryEdgePayloadPartitionIssue::SourceBoundaryReferenceBeforeCount {
            group,
        }) if group.group_code().value() == 330
    ));
    for ordinal in 0..3 {
        assert!(directory.edge_data_fields_for_edge(ordinal).is_none());
        assert!(directory.path_trailer_fields_for_edge(ordinal).is_none());
    }
    Ok(())
}

#[test]
fn non_final_edges_bounds_identity_cancellation_traits_and_redaction_hold()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n2\n\
         72\n4\n94\n1\n73\n0\n74\n0\n95\n1\n96\n1\n40\n0\n10\n1\n20\n2\n97\n0\n\
         72\n2\n10\n12345.625\n20\n4\n40\n5\n50\n0\n51\n90\n73\n1\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_edge_payload_partition_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .hatch_boundary_edge_payload_partition_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.entry_for_edge(u64::MAX).is_none());
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert!(directory.edge_data_fields_for_edge(u64::MAX).is_none());
    assert!(directory.path_trailer_fields_for_edge(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.edge_type_directory().source_id()
    );
    let interior = available(directory.entries()[0])?;
    assert!(!interior.is_final_edge());
    assert!(interior.source_boundary_count().is_none());
    assert!(interior.path_trailer_range().is_none());
    assert_eq!(
        codes(directory.edge_data_fields_for_edge(0).unwrap_or_default()).last(),
        Some(&97)
    );
    assert!(directory.path_trailer_fields_for_edge(0).is_none());
    assert!(available(directory.entries()[1])?.is_final_edge());
    assert!(size_of::<DxfHatchBoundaryEdgePayloadPartition>() <= 128);
    assert!(size_of::<DxfHatchBoundaryEdgePayloadPartitionEntry>() <= 320);
    assert!(!format!("{:?}", directory.entries()[1]).contains("12345.625"));
    copy::<DxfHatchBoundaryEdgePayloadPartition>();
    copy::<DxfHatchBoundaryEdgePayloadPartitionEntry>();
    send_sync::<DxfHatchBoundaryEdgePayloadPartitionDirectory>();
    Ok(())
}

fn assert_standard_partition(
    directory: &DxfHatchBoundaryEdgePayloadPartitionDirectory,
) -> Result<(), io::Error> {
    let [line, spline] = directory.entries() else {
        return Err(io::Error::other("two partition entries"));
    };
    assert_eq!(line.ordinal(), 0);
    assert_eq!(
        line.edge_type_entry().edge_type(),
        Ok(DxfHatchBoundaryEdgeType::Line)
    );
    assert_eq!(spline.ordinal(), 1);
    assert_eq!(
        spline.edge_type_entry().edge_type(),
        Ok(DxfHatchBoundaryEdgeType::Spline)
    );
    assert!(!available(*line)?.is_final_edge());
    assert!(available(*spline)?.is_final_edge());
    assert_eq!(
        codes(
            directory
                .edge_data_fields_for_edge(0)
                .ok_or_else(|| io::Error::other("Line data"))?
        ),
        [10, 20, 11, 21]
    );
    assert_eq!(
        codes(
            directory
                .edge_data_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("Spline data"))?
        ),
        [94, 73, 74, 95, 96, 40, 10, 20, 97]
    );
    assert!(directory.path_trailer_fields_for_edge(0).is_none());
    assert_eq!(
        codes(
            directory
                .path_trailer_fields_for_edge(1)
                .ok_or_else(|| io::Error::other("final path trailer"))?
        ),
        [97]
    );
    Ok(())
}

fn available(
    entry: DxfHatchBoundaryEdgePayloadPartitionEntry,
) -> Result<DxfHatchBoundaryEdgePayloadPartition, io::Error> {
    entry
        .partition()
        .map_err(|_| io::Error::other("available edge payload partition"))
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
    push_i32(&mut bytes, version, 94, 1)?;
    push_i16(&mut bytes, version, 73, 0)?;
    push_i16(&mut bytes, version, 74, 0)?;
    push_i32(&mut bytes, version, 95, 1)?;
    push_i32(&mut bytes, version, 96, 1)?;
    push_double(&mut bytes, version, 40, 0.0)?;
    push_double(&mut bytes, version, 10, 5.0)?;
    push_double(&mut bytes, version, 20, 6.0)?;
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

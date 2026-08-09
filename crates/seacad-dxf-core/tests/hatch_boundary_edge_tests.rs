use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundaryEdgeCountRelation,
    DxfHatchBoundaryEdgeDirectory, DxfHatchBoundaryEdgeEntry, DxfHatchBoundaryEdgePath,
    DxfHatchBoundaryEdgePathEntry, DxfHatchBoundaryEdgePathIssue, DxfHatchBoundaryEdgePathState,
    DxfHatchBoundaryPathFlagIssue, DxfMemorySource, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_edge_anchor_and_slice_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n4\n72\n1\n10\n1\n72\n2\n10\n2\n72\n3\n10\n3\n72\n4\n10\n4\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory =
            open_ascii(&source)?.hatch_boundary_edge_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory =
            open_binary(&source)?.hatch_boundary_edge_directory(&DxfCancellationToken::default())?;
        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn polyline_flag_and_header_failures_publish_no_edges() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n7\n\
         92\n2\n72\n0\n73\n0\n93\n0\n\
         92\n32\n\
         92\n0\n72\n1\n\
         92\n0\n93\n1\n93\n1\n72\n1\n\
         92\n0\n93\n.\n72\n1\n\
         92\n0\n93\n-1\n72\n1\n\
         92\n0\n72\n1\n93\n1\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_boundary_edge_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.paths().len(), 7);
    assert!(directory.edges().is_empty());
    assert_eq!(
        directory.paths()[0].state(),
        DxfHatchBoundaryEdgePathState::NotEdges
    );
    assert!(matches!(
        directory.paths()[1].state(),
        DxfHatchBoundaryEdgePathState::Unavailable(
            DxfHatchBoundaryEdgePathIssue::PathFlagsUnavailable(
                DxfHatchBoundaryPathFlagIssue::UnsupportedBits { .. }
            )
        )
    ));
    assert_eq!(
        directory.paths()[2].state(),
        DxfHatchBoundaryEdgePathState::Unavailable(DxfHatchBoundaryEdgePathIssue::EdgeCountAbsent)
    );
    assert_eq!(
        directory.paths()[3].state(),
        DxfHatchBoundaryEdgePathState::Unavailable(
            DxfHatchBoundaryEdgePathIssue::EdgeCountMultiple {
                occurrence_count: 2
            }
        )
    );
    assert!(matches!(
        directory.paths()[4].state(),
        DxfHatchBoundaryEdgePathState::Unavailable(
            DxfHatchBoundaryEdgePathIssue::InvalidAsciiNumber { .. }
        )
    ));
    assert!(matches!(
        directory.paths()[5].state(),
        DxfHatchBoundaryEdgePathState::Unavailable(
            DxfHatchBoundaryEdgePathIssue::ValueOutOfDomain { value: -1, .. }
        )
    ));
    assert!(matches!(
        directory.paths()[6].state(),
        DxfHatchBoundaryEdgePathState::Unavailable(
            DxfHatchBoundaryEdgePathIssue::EdgeMarkerBeforeCount { .. }
        )
    ));
    for ordinal in 0..7 {
        assert!(directory.edges_for_path(ordinal).is_none());
    }
    Ok(())
}

#[test]
fn matched_mismatched_empty_ranges_and_raw_marker_values_remain_explicit()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n3\n\
         92\n0\n93\n2\n72\n1\n10\n10\n72\n99\n40\n20\n\
         92\n0\n93\n3\n72\n2\n10\n30\n72\n3\n10\n40\n\
         92\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_boundary_edge_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.paths().len(), 3);
    assert_eq!(directory.edges().len(), 4);
    let first = grouped(directory.paths()[0])?;
    assert_eq!(first.edge_count().value(), 2);
    assert_eq!(
        first.count_relation(),
        DxfHatchBoundaryEdgeCountRelation::Matched { count: 2 }
    );
    assert_eq!(first.edge_range().start(), 0);
    assert_eq!(first.edge_range().end(), 2);
    let second = grouped(directory.paths()[1])?;
    assert_eq!(
        second.count_relation(),
        DxfHatchBoundaryEdgeCountRelation::Mismatched {
            declared: 3,
            observed: 2
        }
    );
    assert_eq!(second.edge_range().start(), 2);
    assert_eq!(second.edge_range().end(), 4);
    let third = grouped(directory.paths()[2])?;
    assert!(third.edge_range().is_empty());
    assert_eq!(directory.edges_for_path(2), Some(&[][..]));
    assert_eq!(
        directory
            .edges()
            .iter()
            .map(|edge| edge.path_ordinal())
            .collect::<Vec<_>>(),
        [0, 0, 1, 1]
    );
    assert_eq!(
        directory
            .edges()
            .iter()
            .map(|edge| edge.path_edge_ordinal())
            .collect::<Vec<_>>(),
        [0, 1, 0, 1]
    );
    for (edge, expected) in directory.edges().iter().zip([1_i16, 99, 2, 3]) {
        assert_eq!(edge.marker().group().group_code().value(), 72);
        assert_eq!(marker_i16(&source, *edge)?, expected);
    }
    assert_eq!(payload_codes(&directory, 0)?, [10]);
    assert_eq!(payload_codes(&directory, 1)?, [40]);
    assert_eq!(payload_codes(&directory, 2)?, [10]);
    assert_eq!(payload_codes(&directory, 3)?, [10]);
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_redaction_and_duplicate_subclasses_hold()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n1\n10\n12345.625\n75\n0\n\
         100\nAcDbHatch\n91\n1\n92\n0\n93\n1\n72\n2\n10\n2\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_edge_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.hatch_boundary_edge_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.paths().len(), 2);
    assert_eq!(directory.edges().len(), 2);
    assert_eq!(directory.path(0), Some(directory.paths()[0]));
    assert_eq!(directory.path(u64::MAX), None);
    assert_eq!(directory.edge(0), Some(directory.edges()[0]));
    assert_eq!(directory.edge(u64::MAX), None);
    assert!(directory.payload_fields_for_edge(u64::MAX).is_none());
    assert_ne!(
        directory.paths()[0].subclass_ordinal(),
        directory.paths()[1].subclass_ordinal()
    );
    assert_eq!(
        directory.paths()[0].raw_record_ordinal(),
        directory.paths()[1].raw_record_ordinal()
    );
    assert_eq!(
        directory.source_id(),
        directory.flag_directory().source_id()
    );
    assert!(size_of::<DxfHatchBoundaryEdgeEntry>() <= 80);
    assert!(size_of::<DxfHatchBoundaryEdgePathEntry>() <= 320);
    assert!(!format!("{:?}", directory.edges()[0]).contains("12345.625"));
    copy::<DxfHatchBoundaryEdgeEntry>();
    copy::<DxfHatchBoundaryEdgePathEntry>();
    send_sync::<DxfHatchBoundaryEdgeDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfHatchBoundaryEdgeDirectory) -> Result<(), io::Error> {
    let [path] = directory.paths() else {
        return Err(io::Error::other("one edge path"));
    };
    let grouped = grouped(*path)?;
    assert_eq!(grouped.edge_count().value(), 4);
    assert_eq!(
        grouped.count_relation(),
        DxfHatchBoundaryEdgeCountRelation::Matched { count: 4 }
    );
    let edges = directory
        .edges_for_path(0)
        .ok_or(io::Error::other("grouped edges"))?;
    assert_eq!(edges.len(), 4);
    assert_eq!(
        edges
            .iter()
            .map(|edge| edge.marker().group().group_code().value())
            .collect::<Vec<_>>(),
        [72, 72, 72, 72]
    );
    assert_eq!(payload_codes(directory, 0)?, [10]);
    assert_eq!(payload_codes(directory, 1)?, [10]);
    assert_eq!(payload_codes(directory, 2)?, [10]);
    assert_eq!(payload_codes(directory, 3)?, [10, 97]);
    Ok(())
}

fn evidence(directory: &DxfHatchBoundaryEdgeDirectory) -> Result<Vec<Vec<i16>>, io::Error> {
    directory
        .edges()
        .iter()
        .map(|edge| payload_codes(directory, edge.ordinal()))
        .collect()
}

fn payload_codes(
    directory: &DxfHatchBoundaryEdgeDirectory,
    edge_ordinal: u64,
) -> Result<Vec<i16>, io::Error> {
    Ok(directory
        .payload_fields_for_edge(edge_ordinal)
        .ok_or(io::Error::other("edge payload"))?
        .iter()
        .map(|field| field.group().group_code().value())
        .collect())
}

fn grouped(entry: DxfHatchBoundaryEdgePathEntry) -> Result<DxfHatchBoundaryEdgePath, io::Error> {
    let DxfHatchBoundaryEdgePathState::Grouped(path) = entry.state() else {
        return Err(io::Error::other("grouped edge path"));
    };
    Ok(path)
}

fn marker_i16(
    source: &dyn DxfByteSource,
    edge: DxfHatchBoundaryEdgeEntry,
) -> Result<i16, Box<dyn Error>> {
    let mut observer = NoopDxfReadObserver;
    let document = DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )?;
    let view = DxfRawDocumentView::from(&document);
    let span = edge.marker().group().value_payload_span();
    let mut bytes = vec![0_u8; usize::try_from(span.len())?];
    view.read_span(span, &mut bytes)?;
    Ok(std::str::from_utf8(&bytes)?.trim().parse::<i16>()?)
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
    push_i32(&mut bytes, version, 93, 4)?;
    for edge_type in 1_i16..=4 {
        push_i16(&mut bytes, version, 72, edge_type)?;
        push_double(&mut bytes, version, 10, f64::from(edge_type))?;
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfLightweightPolylineClosureState,
    DxfLightweightPolylineSegmentDirectory, DxfLightweightPolylineSegmentEntry,
    DxfLightweightPolylineSegmentRecordEntry, DxfLightweightPolylineSegmentSemantics,
    DxfLightweightPolylineSegmentShape, DxfLightweightPolylineSegmentTopology, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct SegmentEvidence {
    topology: DxfLightweightPolylineSegmentTopology,
    start: Option<[u64; 2]>,
    end: Option<[u64; 2]>,
    local_widths: Option<[u64; 2]>,
    shape: Option<u64>,
}

#[test]
fn every_supported_dialect_has_ascii_binary_segment_topology_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.lightweight_polyline_segment_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.lightweight_polyline_segment_directory(&DxfCancellationToken::default())?;

        assert_valid_directory(&ascii_directory)?;
        assert_valid_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn invalid_or_multiple_flags_never_close_while_one_closed_vertex_forms_one_segment()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n70\n.\n10\n0\n20\n0\n42\n.\n10\n1\n20\n0\n10\n2\n20\n0\n0\nLWPOLYLINE\n70\n0\n70\n1\n10\n3\n20\n0\n10\n4\n20\n0\n10\n5\n20\n0\n0\nLWPOLYLINE\n10\n6\n20\n0\n0\nLWPOLYLINE\n70\n1\n10\n7\n20\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.lightweight_polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 4);
    assert_eq!(directory.segments().len(), 5);

    let invalid = directory.records()[0];
    assert_eq!(
        invalid.closure(),
        DxfLightweightPolylineClosureState::IndeterminateInvalid
    );
    assert_eq!(invalid.closure().is_closed(), None);
    assert_eq!(invalid.segment_range().len(), 2);
    assert!(
        directory
            .segments_for_raw_record(invalid.grouped_record().record().ordinal())
            .ok_or(io::Error::other("invalid-flag segments"))?
            .iter()
            .all(|segment| {
                segment.topology() == DxfLightweightPolylineSegmentTopology::Consecutive
            })
    );
    let first_semantics = directory
        .semantics_for_segment(invalid.segment_range().start())?
        .ok_or(io::Error::other("invalid-bulge segment"))?;
    assert_eq!(
        first_semantics.shape(),
        DxfLightweightPolylineSegmentShape::Indeterminate
    );

    let multiple = directory.records()[1];
    assert_eq!(
        multiple.closure(),
        DxfLightweightPolylineClosureState::IndeterminateMultiple {
            occurrence_count: 2
        }
    );
    assert_eq!(multiple.segment_range().len(), 2);

    let single_open = directory.records()[2];
    assert_eq!(
        single_open.closure(),
        DxfLightweightPolylineClosureState::DefaultedOpen
    );
    assert!(single_open.segment_range().is_empty());
    let single_closed = directory.records()[3];
    assert_eq!(
        single_closed.closure(),
        DxfLightweightPolylineClosureState::ExplicitClosed { flags: 1 }
    );
    assert_eq!(single_closed.segment_range().len(), 1);
    let self_closing = directory
        .segments_for_raw_record(single_closed.grouped_record().record().ordinal())
        .and_then(|segments| segments.first())
        .copied()
        .ok_or(io::Error::other("self-closing segment"))?;
    assert_eq!(
        self_closing.topology(),
        DxfLightweightPolylineSegmentTopology::Closing
    );
    assert_eq!(
        self_closing.start_vertex_ordinal(),
        self_closing.end_vertex_ordinal()
    );
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.lightweight_polyline_segment_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.lightweight_polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.segments_for_raw_record(u64::MAX), None);
    assert_eq!(directory.segment(u64::MAX), None);
    assert!(directory.semantics_for_segment(u64::MAX)?.is_none());
    assert_copy::<DxfLightweightPolylineSegmentRecordEntry>();
    assert_copy::<DxfLightweightPolylineSegmentEntry>();
    assert_copy::<DxfLightweightPolylineSegmentSemantics>();
    assert_send_sync::<DxfLightweightPolylineSegmentDirectory>();
    Ok(())
}

fn assert_valid_directory(
    directory: &DxfLightweightPolylineSegmentDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        directory.source_id(),
        directory.vertex_semantic_directory().source_id()
    );
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.segments().len(), 5);
    let open = directory.records()[0];
    assert_eq!(
        open.closure(),
        DxfLightweightPolylineClosureState::DefaultedOpen
    );
    assert_eq!(open.segment_range().len(), 2);

    let closed = directory.records()[1];
    assert_eq!(
        closed.closure(),
        DxfLightweightPolylineClosureState::ExplicitClosed { flags: 129 }
    );
    assert_eq!(closed.closure().is_closed(), Some(true));
    assert_eq!(closed.closure().explicit_flags(), Some(129));
    assert_eq!(closed.segment_range().len(), 3);
    let closed_segments = directory
        .segments_for_raw_record(closed.grouped_record().record().ordinal())
        .ok_or(io::Error::other("closed segments"))?;
    assert_eq!(
        closed_segments[0].topology(),
        DxfLightweightPolylineSegmentTopology::Consecutive
    );
    assert_eq!(
        closed_segments[1].topology(),
        DxfLightweightPolylineSegmentTopology::Consecutive
    );
    assert_eq!(
        closed_segments[2].topology(),
        DxfLightweightPolylineSegmentTopology::Closing
    );
    assert_eq!(closed_segments[2].record_segment_ordinal(), 2);
    assert_eq!(closed_segments[2].start_vertex_ordinal(), 5);
    assert_eq!(closed_segments[2].end_vertex_ordinal(), 3);

    let open_first = directory
        .semantics_for_segment(open.segment_range().start())?
        .ok_or(io::Error::other("open first segment"))?;
    assert_eq!(
        open_first.shape(),
        DxfLightweightPolylineSegmentShape::Straight
    );
    assert_eq!(
        open_first
            .start_vertex_local_widths()
            .map(|values| values.map(DxfDouble::to_bits)),
        Some([1.0_f64.to_bits(), 2.0_f64.to_bits()])
    );
    let closing = directory
        .semantics_for_segment(closed_segments[2].ordinal())?
        .ok_or(io::Error::other("closing semantics"))?;
    assert_eq!(
        closing.shape(),
        DxfLightweightPolylineSegmentShape::Arc {
            bulge: DxfDouble::from_f64(-0.5)
        }
    );
    assert_eq!(
        closing
            .start_ocs_position()
            .map(|values| values.map(DxfDouble::to_bits)),
        Some([12.0_f64.to_bits(), 0.0_f64.to_bits()])
    );
    assert_eq!(
        closing
            .end_ocs_position()
            .map(|values| values.map(DxfDouble::to_bits)),
        Some([10.0_f64.to_bits(), 0.0_f64.to_bits()])
    );
    Ok(())
}

fn evidence(
    directory: &DxfLightweightPolylineSegmentDirectory,
) -> Result<Vec<SegmentEvidence>, Box<dyn Error>> {
    directory
        .segments()
        .iter()
        .map(|segment| {
            let semantics = directory
                .semantics_for_segment(segment.ordinal())?
                .ok_or(io::Error::other("segment semantics"))?;
            let shape = match semantics.shape() {
                DxfLightweightPolylineSegmentShape::Straight => Some(0.0_f64.to_bits()),
                DxfLightweightPolylineSegmentShape::Arc { bulge } => Some(bulge.to_bits()),
                DxfLightweightPolylineSegmentShape::Indeterminate => None,
                _ => return Err(io::Error::other("unknown segment shape").into()),
            };
            Ok(SegmentEvidence {
                topology: segment.topology(),
                start: semantics
                    .start_ocs_position()
                    .map(|values| values.map(DxfDouble::to_bits)),
                end: semantics
                    .end_ocs_position()
                    .map(|values| values.map(DxfDouble::to_bits)),
                local_widths: semantics
                    .start_vertex_local_widths()
                    .map(|values| values.map(DxfDouble::to_bits)),
                shape,
            })
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n3\n10\n0\n20\n0\n40\n1\n41\n2\n42\n-0\n10\n1\n20\n0\n42\n1\n10\n2\n20\n0\n42\n-0.5\n0\nLWPOLYLINE\n90\n3\n70\n129\n10\n10\n20\n0\n42\n0\n10\n11\n20\n0\n42\n1\n10\n12\n20\n0\n42\n-0.5\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"LWPOLYLINE")?;
    push_i32(&mut bytes, version, 90, 3)?;
    for (code, value) in [
        (10, 0.0),
        (20, 0.0),
        (40, 1.0),
        (41, 2.0),
        (42, -0.0),
        (10, 1.0),
        (20, 0.0),
        (42, 1.0),
        (10, 2.0),
        (20, 0.0),
        (42, -0.5),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"LWPOLYLINE")?;
    push_i32(&mut bytes, version, 90, 3)?;
    push_i16(&mut bytes, version, 70, 129)?;
    for (code, value) in [
        (10, 10.0),
        (20, 0.0),
        (42, 0.0),
        (10, 11.0),
        (20, 0.0),
        (42, 1.0),
        (10, 12.0),
        (20, 0.0),
        (42, -0.5),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
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

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

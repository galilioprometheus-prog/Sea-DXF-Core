use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineFamily,
    DxfPolylineSegmentDirectory, DxfPolylineSegmentEntry, DxfPolylineSegmentRecordEntry,
    DxfPolylineSegmentRecordState, DxfPolylineSegmentTopology, DxfPolylineSequenceState,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type SegmentEvidence = (DxfPolylineSegmentTopology, u64, u64, u64);

#[test]
fn every_supported_dialect_has_ascii_binary_segment_topology_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.polyline_segment_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_segment_directory(&DxfCancellationToken::default())?;

        assert_valid_topology(&ascii_directory)?;
        assert_valid_topology(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory), evidence(&binary_directory));
    }
    Ok(())
}

#[test]
fn incomplete_unsupported_indeterminate_and_inconsistent_records_emit_no_segments()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n0\n0\nVERTEX\n70\n0\n0\nPOINT\n0\nPOLYLINE\n70\n16\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n192\n0\nSEQEND\n0\nPOLYLINE\n70\n24\n0\nVERTEX\n70\n32\n0\nSEQEND\n0\nPOLYLINE\n70\n8\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nPOLYLINE\n70\n0\n0\nVERTEX\n70\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 6);
    assert!(directory.segments().is_empty());
    assert_eq!(
        directory.records()[0].state(),
        DxfPolylineSegmentRecordState::IncompleteSequence {
            sequence_state: DxfPolylineSequenceState::Interrupted
        }
    );
    assert_eq!(
        directory.records()[1].state(),
        DxfPolylineSegmentRecordState::UnsupportedFamily {
            family: DxfPolylineFamily::PolygonMesh
        }
    );
    assert_eq!(
        directory.records()[2].state(),
        DxfPolylineSegmentRecordState::UnsupportedFamily {
            family: DxfPolylineFamily::PolyfaceMesh
        }
    );
    assert_eq!(
        directory.records()[3].state(),
        DxfPolylineSegmentRecordState::IndeterminateFamily
    );
    assert_eq!(
        directory.records()[4].state(),
        DxfPolylineSegmentRecordState::InconsistentVertex {
            sequence_vertex_ordinal: 0
        }
    );
    assert_eq!(
        directory.records()[5].state(),
        DxfPolylineSegmentRecordState::IncompleteSequence {
            sequence_state: DxfPolylineSequenceState::Unclosed
        }
    );
    for record in directory.records() {
        assert!(record.segment_range().is_empty());
    }
    Ok(())
}

#[test]
fn empty_and_single_vertex_paths_cancellation_lookup_and_traits_are_bounded()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n0\n0\nSEQEND\n0\nPOLYLINE\n70\n1\n0\nVERTEX\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_segment_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_segment_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.records()[0].segment_range().len(), 0);
    assert_eq!(directory.records()[1].segment_range().len(), 1);
    assert_eq!(
        directory.segments()[0].topology(),
        DxfPolylineSegmentTopology::Closing
    );
    assert_eq!(
        directory.segments()[0].start_vertex(),
        directory.segments()[0].end_vertex()
    );
    let raw = directory.records()[1]
        .record()
        .sequence()
        .polyline_record()
        .ordinal();
    assert_eq!(
        directory
            .segments_for_polyline_raw_ordinal(raw)
            .ok_or(io::Error::other("segments"))?
            .len(),
        1
    );
    assert!(directory.segment(u64::MAX).is_none());
    assert!(
        directory
            .record_for_polyline_raw_ordinal(u64::MAX)
            .is_none()
    );
    assert_send_sync::<DxfPolylineSegmentDirectory>();
    assert_copy::<DxfPolylineSegmentRecordEntry>();
    assert_copy::<DxfPolylineSegmentEntry>();
    Ok(())
}

fn assert_valid_topology(directory: &DxfPolylineSegmentDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.segments().len(), 5);
    assert_eq!(
        directory.records()[0].state(),
        DxfPolylineSegmentRecordState::Available {
            family: DxfPolylineFamily::TwoDimensional,
            closed: false
        }
    );
    assert_eq!(directory.records()[0].segment_range().len(), 2);
    assert_eq!(
        directory.records()[1].state(),
        DxfPolylineSegmentRecordState::Available {
            family: DxfPolylineFamily::ThreeDimensional,
            closed: true
        }
    );
    assert_eq!(directory.records()[1].segment_range().len(), 3);
    assert_eq!(
        directory.segments()[4].topology(),
        DxfPolylineSegmentTopology::Closing
    );
    assert_eq!(
        directory.segments()[4]
            .start_vertex()
            .sequence_vertex_ordinal(),
        2
    );
    assert_eq!(
        directory.segments()[4]
            .end_vertex()
            .sequence_vertex_ordinal(),
        0
    );
    assert_eq!(
        directory.source_id(),
        directory.family_semantic_directory().source_id()
    );
    Ok(())
}

fn evidence(directory: &DxfPolylineSegmentDirectory) -> Vec<SegmentEvidence> {
    directory
        .segments()
        .iter()
        .map(|segment| {
            (
                segment.topology(),
                segment.record_segment_ordinal(),
                segment.start_vertex().sequence_vertex_ordinal(),
                segment.end_vertex().sequence_vertex_ordinal(),
            )
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n0\n0\nVERTEX\n70\n0\n0\nVERTEX\n70\n0\n0\nVERTEX\n70\n0\n0\nSEQEND\n0\nPOLYLINE\n70\n9\n0\nVERTEX\n70\n32\n0\nVERTEX\n70\n32\n0\nVERTEX\n70\n32\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    for (parent_flags, vertex_flags) in [(0, 0), (9, 32)] {
        push_string(&mut bytes, version, 0, b"POLYLINE")?;
        push_i16(&mut bytes, version, 70, parent_flags)?;
        for _ in 0..3 {
            push_string(&mut bytes, version, 0, b"VERTEX")?;
            push_i16(&mut bytes, version, 70, vertex_flags)?;
        }
        push_string(&mut bytes, version, 0, b"SEQEND")?;
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

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
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

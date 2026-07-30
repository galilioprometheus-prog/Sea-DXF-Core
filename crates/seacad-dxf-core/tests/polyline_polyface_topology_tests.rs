use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineFamily,
    DxfPolylinePolyfaceCoordinateEntry, DxfPolylinePolyfaceFaceEntry, DxfPolylinePolyfaceOrdering,
    DxfPolylinePolyfaceRecordEntry, DxfPolylinePolyfaceRecordState,
    DxfPolylinePolyfaceTopologyDirectory, DxfPolylineSequenceState, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

type PartitionEvidence = (Vec<(u64, u64)>, Vec<(u64, u64)>);

#[test]
fn every_dialect_has_ascii_binary_odd_order_partition_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let a = ascii.polyline_polyface_topology_directory(&DxfCancellationToken::default())?;
        let b = binary.polyline_polyface_topology_directory(&DxfCancellationToken::default())?;
        assert_partition(&a)?;
        assert_partition(&b)?;
        assert_eq!(evidence(&a), evidence(&b));
    }
    Ok(())
}

#[test]
fn incomplete_and_family_failures_emit_no_partition_members() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n192\n0\nPOINT\n0\nPOLYLINE\n70\n0\n0\nSEQEND\n0\nPOLYLINE\n70\n72\n0\nSEQEND\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_polyface_topology_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 4);
    assert!(directory.coordinates().is_empty());
    assert!(directory.faces().is_empty());
    assert_eq!(
        directory.records()[0].state(),
        DxfPolylinePolyfaceRecordState::IncompleteSequence {
            sequence_state: DxfPolylineSequenceState::Interrupted
        }
    );
    assert_eq!(
        directory.records()[1].state(),
        DxfPolylinePolyfaceRecordState::UnsupportedFamily {
            family: DxfPolylineFamily::TwoDimensional
        }
    );
    assert_eq!(
        directory.records()[2].state(),
        DxfPolylinePolyfaceRecordState::IndeterminateFamily
    );
    assert_eq!(
        directory.records()[3].state(),
        DxfPolylinePolyfaceRecordState::InconsistentVertex {
            sequence_vertex_ordinal: 0
        }
    );
    assert!(
        directory.records().iter().all(|record| {
            record.coordinate_range().is_empty() && record.face_range().is_empty()
        })
    );
    Ok(())
}

#[test]
fn canonical_order_with_unusable_reported_counts_remains_available() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n64\n71\n.\n72\n1\n72\n2\n0\nVERTEX\n70\n192\n0\nVERTEX\n70\n128\n0\nSEQEND\n0\nPOLYLINE\n70\n64\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_polyface_topology_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.records()[0].state(),
        DxfPolylinePolyfaceRecordState::Available {
            reported_coordinate_count: None,
            reported_face_count: None,
            observed_coordinate_count: 1,
            observed_face_count: 1,
            ordering: DxfPolylinePolyfaceOrdering::CoordinatesThenFaces,
        }
    );
    assert_eq!(evidence(&directory).0, [(0, 0)]);
    assert_eq!(evidence(&directory).1, [(0, 1)]);
    assert_eq!(
        directory.records()[1].state(),
        DxfPolylinePolyfaceRecordState::Available {
            reported_coordinate_count: None,
            reported_face_count: None,
            observed_coordinate_count: 0,
            observed_face_count: 0,
            ordering: DxfPolylinePolyfaceOrdering::CoordinatesThenFaces,
        }
    );
    Ok(())
}

#[test]
fn cancellation_lookups_identity_and_traits_are_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_polyface_topology_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_polyface_topology_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.family_semantic_directory().source_id()
    );
    assert!(directory.coordinate(u64::MAX).is_none());
    assert!(directory.face(u64::MAX).is_none());
    assert!(
        directory
            .record_for_polyline_raw_ordinal(u64::MAX)
            .is_none()
    );
    assert_send_sync::<DxfPolylinePolyfaceTopologyDirectory>();
    assert_copy::<DxfPolylinePolyfaceRecordEntry>();
    assert_copy::<DxfPolylinePolyfaceCoordinateEntry>();
    assert_copy::<DxfPolylinePolyfaceFaceEntry>();
    Ok(())
}

fn assert_partition(
    directory: &DxfPolylinePolyfaceTopologyDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.coordinates().len(), 2);
    assert_eq!(directory.faces().len(), 2);
    let record = directory.records()[0];
    assert_eq!(
        record.state(),
        DxfPolylinePolyfaceRecordState::Available {
            reported_coordinate_count: Some(99),
            reported_face_count: Some(-2),
            observed_coordinate_count: 2,
            observed_face_count: 2,
            ordering: DxfPolylinePolyfaceOrdering::Odd,
        }
    );
    assert_eq!(record.coordinate_range().len(), 2);
    assert_eq!(record.face_range().len(), 2);
    assert_eq!(evidence(directory).0, [(0, 1), (1, 3)]);
    assert_eq!(evidence(directory).1, [(0, 0), (1, 2)]);
    let raw = record.record().sequence().polyline_record().ordinal();
    assert_eq!(
        directory
            .coordinates_for_polyline_raw_ordinal(raw)
            .ok_or(io::Error::other("coordinates"))?
            .len(),
        2
    );
    assert_eq!(
        directory
            .faces_for_polyline_raw_ordinal(raw)
            .ok_or(io::Error::other("faces"))?
            .len(),
        2
    );
    Ok(())
}

fn evidence(directory: &DxfPolylinePolyfaceTopologyDirectory) -> PartitionEvidence {
    let coordinates = directory
        .coordinates()
        .iter()
        .map(|entry| {
            (
                entry.record_coordinate_ordinal(),
                entry.vertex().sequence_vertex_ordinal(),
            )
        })
        .collect();
    let faces = directory
        .faces()
        .iter()
        .map(|entry| {
            (
                entry.record_face_ordinal(),
                entry.vertex().sequence_vertex_ordinal(),
            )
        })
        .collect();
    (coordinates, faces)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n64\n71\n99\n72\n-2\n0\nVERTEX\n70\n128\n0\nVERTEX\n70\n192\n0\nVERTEX\n70\n128\n0\nVERTEX\n70\n192\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"POLYLINE"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 70, 64)?;
    push_i16(&mut bytes, version, 71, 99)?;
    push_i16(&mut bytes, version, 72, -2)?;
    for flags in [128, 192, 128, 192] {
        push_string(&mut bytes, version, 0, b"VERTEX")?;
        push_i16(&mut bytes, version, 70, flags)?;
    }
    push_string(&mut bytes, version, 0, b"SEQEND")?;
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

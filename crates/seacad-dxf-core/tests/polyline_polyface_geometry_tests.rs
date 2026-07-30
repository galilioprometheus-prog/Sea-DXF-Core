use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylinePolyfaceFaceGeometryDirectory,
    DxfPolylinePolyfaceFaceGeometryEntry, DxfPolylinePolyfaceFaceGeometryState,
    DxfPolylinePolyfaceFaceResolutionState, DxfPolylinePolyfacePointEntry, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

type PointEvidence = (u8, u64, [u64; 3], bool);

#[test]
fn every_dialect_has_ascii_binary_odd_order_wcs_point_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let a = ascii.polyline_polyface_face_geometry_directory(&DxfCancellationToken::default())?;
        let b =
            binary.polyline_polyface_face_geometry_directory(&DxfCancellationToken::default())?;
        assert_geometry(&a)?;
        assert_geometry(&b)?;
        assert_eq!(evidence(&a), evidence(&b));
    }
    Ok(())
}

#[test]
fn coordinate_and_face_resolution_failures_emit_typed_zero_point_faces()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n192\n10\n1\n20\n2\n0\nVERTEX\n70\n128\n71\n1\n0\nVERTEX\n70\n128\n71\n2\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_polyface_face_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.faces().len(), 2);
    assert!(directory.points().is_empty());
    assert_eq!(
        directory.faces()[0].state(),
        DxfPolylinePolyfaceFaceGeometryState::CoordinateUnavailable {
            face_corner_ordinal: 0,
            component_states: [
                DxfSemanticValueState::Explicit,
                DxfSemanticValueState::Explicit,
                DxfSemanticValueState::Invalid,
            ],
        }
    );
    assert_eq!(
        directory.faces()[1].state(),
        DxfPolylinePolyfaceFaceGeometryState::FaceResolutionFailure {
            state: DxfPolylinePolyfaceFaceResolutionState::CoordinateIndexOutOfRange {
                slot: 0,
                index: 2,
                coordinate_count: 1,
            }
        }
    );
    assert!(
        directory
            .faces()
            .iter()
            .all(|face| face.point_range().is_empty())
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
        document.polyline_polyface_face_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_polyface_face_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.face_resolution_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.vertex_semantic_directory().source_id()
    );
    assert!(directory.face(u64::MAX).is_none());
    assert!(directory.point(u64::MAX).is_none());
    assert!(directory.points_for_face(u64::MAX).is_none());
    assert_send_sync::<DxfPolylinePolyfaceFaceGeometryDirectory>();
    assert_copy::<DxfPolylinePolyfaceFaceGeometryEntry>();
    assert_copy::<DxfPolylinePolyfacePointEntry>();
    Ok(())
}

fn assert_geometry(
    directory: &DxfPolylinePolyfaceFaceGeometryDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.faces().len(), 1);
    assert_eq!(directory.points().len(), 3);
    let face = directory.faces()[0];
    assert_eq!(
        face.state(),
        DxfPolylinePolyfaceFaceGeometryState::Available { point_count: 3 }
    );
    assert_eq!(face.point_range().len(), 3);
    assert_eq!(
        evidence(directory),
        [
            (0, 0, bits([1.0, 2.0, 3.0]), true),
            (1, 1, bits([4.0, 5.0, 6.0]), false),
            (2, 2, bits([7.0, 8.0, 9.0]), true),
        ]
    );
    assert_eq!(
        directory
            .points_for_face(face.face().face().ordinal())
            .ok_or(io::Error::other("face points"))?
            .len(),
        3
    );
    Ok(())
}

fn evidence(directory: &DxfPolylinePolyfaceFaceGeometryDirectory) -> Vec<PointEvidence> {
    directory
        .points()
        .iter()
        .map(|point| {
            (
                point.face_point_ordinal(),
                point.corner().coordinate().record_coordinate_ordinal(),
                point.position().map(|value| value.to_bits()),
                point.corner().edge_visible(),
            )
        })
        .collect()
}

fn bits(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n192\n10\n1\n20\n2\n30\n3\n0\nVERTEX\n70\n128\n71\n1\n72\n-2\n73\n3\n0\nVERTEX\n70\n192\n10\n4\n20\n5\n30\n6\n0\nVERTEX\n70\n192\n10\n7\n20\n8\n30\n9\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    push_coordinate(&mut bytes, version, [1.0, 2.0, 3.0])?;
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    push_i16(&mut bytes, version, 70, 128)?;
    for (code, value) in [(71, 1), (72, -2), (73, 3)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_coordinate(&mut bytes, version, [4.0, 5.0, 6.0])?;
    push_coordinate(&mut bytes, version, [7.0, 8.0, 9.0])?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_coordinate(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    position: [f64; 3],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"VERTEX")?;
    push_i16(bytes, version, 70, 192)?;
    for (code, value) in [10, 20, 30].into_iter().zip(position) {
        push_double(bytes, version, code, value)?;
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

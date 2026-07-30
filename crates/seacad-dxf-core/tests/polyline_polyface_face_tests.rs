use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylinePolyfaceFaceResolutionDirectory,
    DxfPolylinePolyfaceFaceResolutionState, DxfPolylinePolyfaceResolvedCornerEntry,
    DxfPolylinePolyfaceResolvedFaceEntry, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

type CornerEvidence = (u8, u64, i16, bool);

#[test]
fn every_dialect_has_ascii_binary_signed_face_resolution_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let a =
            ascii.polyline_polyface_face_resolution_directory(&DxfCancellationToken::default())?;
        let b =
            binary.polyline_polyface_face_resolution_directory(&DxfCancellationToken::default())?;
        assert_resolved(&a)?;
        assert_resolved(&b)?;
        assert_eq!(evidence(&a), evidence(&b));
    }
    Ok(())
}

#[test]
fn invalid_terminated_overflow_and_out_of_range_faces_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n192\n0\nVERTEX\n70\n192\n0\nVERTEX\n70\n128\n71\n.\n0\nVERTEX\n70\n128\n71\n1\n72\n0\n73\n2\n0\nVERTEX\n70\n128\n71\n-32768\n0\nVERTEX\n70\n128\n71\n3\n0\nVERTEX\n70\n128\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_polyface_face_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.faces().len(), 5);
    assert!(directory.corners().is_empty());
    assert_eq!(
        directory.faces()[0].state(),
        DxfPolylinePolyfaceFaceResolutionState::InvalidIndex {
            slot: 0,
            state: DxfSemanticValueState::Invalid,
        }
    );
    assert_eq!(
        directory.faces()[1].state(),
        DxfPolylinePolyfaceFaceResolutionState::NonZeroAfterTerminator { slot: 2, value: 2 }
    );
    assert_eq!(
        directory.faces()[2].state(),
        DxfPolylinePolyfaceFaceResolutionState::IndexMagnitudeOverflow { slot: 0 }
    );
    assert_eq!(
        directory.faces()[3].state(),
        DxfPolylinePolyfaceFaceResolutionState::CoordinateIndexOutOfRange {
            slot: 0,
            index: 3,
            coordinate_count: 2,
        }
    );
    assert_eq!(
        directory.faces()[4].state(),
        DxfPolylinePolyfaceFaceResolutionState::Available { corner_count: 0 }
    );
    assert!(
        directory
            .faces()
            .iter()
            .all(|face| face.corner_range().is_empty())
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
        document.polyline_polyface_face_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_polyface_face_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.topology_directory().source_id()
    );
    assert!(directory.face(u64::MAX).is_none());
    assert!(directory.corner(u64::MAX).is_none());
    assert!(directory.corners_for_face(u64::MAX).is_none());
    assert!(directory.face_for_vertex_raw_ordinal(u64::MAX).is_none());
    assert_send_sync::<DxfPolylinePolyfaceFaceResolutionDirectory>();
    assert_copy::<DxfPolylinePolyfaceResolvedFaceEntry>();
    assert_copy::<DxfPolylinePolyfaceResolvedCornerEntry>();
    Ok(())
}

fn assert_resolved(
    directory: &DxfPolylinePolyfaceFaceResolutionDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.faces().len(), 1);
    assert_eq!(directory.corners().len(), 3);
    let face = directory.faces()[0];
    assert_eq!(
        face.state(),
        DxfPolylinePolyfaceFaceResolutionState::Available { corner_count: 3 }
    );
    assert_eq!(face.corner_range().len(), 3);
    assert_eq!(
        evidence(directory),
        [(0, 0, 1, true), (1, 1, -2, false), (2, 2, 3, true)]
    );
    assert_eq!(
        directory
            .corners_for_face(face.face().ordinal())
            .ok_or(io::Error::other("face corners"))?
            .len(),
        3
    );
    assert_eq!(
        directory.face_for_vertex_raw_ordinal(face.face().vertex().vertex_record().ordinal()),
        Some(face)
    );
    Ok(())
}

fn evidence(directory: &DxfPolylinePolyfaceFaceResolutionDirectory) -> Vec<CornerEvidence> {
    directory
        .corners()
        .iter()
        .map(|corner| {
            (
                corner.face_corner_ordinal(),
                corner.coordinate().record_coordinate_ordinal(),
                corner.source_index(),
                corner.edge_visible(),
            )
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n192\n0\nVERTEX\n70\n128\n71\n1\n72\n-2\n73\n3\n74\n0\n0\nVERTEX\n70\n192\n0\nVERTEX\n70\n192\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    push_vertex(&mut bytes, version, 192, &[])?;
    push_vertex(
        &mut bytes,
        version,
        128,
        &[(71, 1), (72, -2), (73, 3), (74, 0)],
    )?;
    push_vertex(&mut bytes, version, 192, &[])?;
    push_vertex(&mut bytes, version, 192, &[])?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_vertex(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    flags: i16,
    indices: &[(i16, i16)],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"VERTEX")?;
    push_i16(bytes, version, 70, flags)?;
    for (code, value) in indices {
        push_i16(bytes, version, *code, *value)?;
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

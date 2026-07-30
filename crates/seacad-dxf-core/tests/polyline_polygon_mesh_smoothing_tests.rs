use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineFamily,
    DxfPolylinePolygonMeshRecordState, DxfPolylinePolygonMeshSmoothSurfaceType,
    DxfPolylinePolygonMeshSmoothingDirectory, DxfPolylinePolygonMeshSmoothingEntry,
    DxfPolylinePolygonMeshSmoothingState, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_smoothing_metadata_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let a = ascii.polyline_polygon_mesh_smoothing_directory(&DxfCancellationToken::default())?;
        let b =
            binary.polyline_polygon_mesh_smoothing_directory(&DxfCancellationToken::default())?;
        assert_smoothing(&a)?;
        assert_smoothing(&b)?;
        assert_eq!(a.entries()[0].state(), b.entries()[0].state());
    }
    Ok(())
}

#[test]
fn topology_metadata_and_surface_type_failures_remain_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n0\n0\nSEQEND\n0\nPOLYLINE\n70\n16\n71\n1\n72\n1\n73\n.\n75\n0\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nPOLYLINE\n70\n16\n71\n1\n72\n1\n73\n1\n74\n2\n75\n7\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_polygon_mesh_smoothing_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 3);
    assert_eq!(
        directory.entries()[0].state(),
        DxfPolylinePolygonMeshSmoothingState::TopologyUnavailable {
            state: DxfPolylinePolygonMeshRecordState::UnsupportedFamily {
                family: DxfPolylineFamily::TwoDimensional,
            }
        }
    );
    assert_eq!(
        directory.entries()[1].state(),
        DxfPolylinePolygonMeshSmoothingState::MetadataUnavailable {
            component_states: [
                DxfSemanticValueState::Invalid,
                DxfSemanticValueState::Defaulted,
                DxfSemanticValueState::Explicit,
            ]
        }
    );
    assert_eq!(
        directory.entries()[2].state(),
        DxfPolylinePolygonMeshSmoothingState::UnsupportedSurfaceType {
            value: 7,
            m_density: 1,
            n_density: 2,
        }
    );
    Ok(())
}

#[test]
fn cancellation_lookup_identity_and_traits_are_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_polygon_mesh_smoothing_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_polygon_mesh_smoothing_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.topology_directory().source_id()
    );
    assert!(directory.entry_for_polyline_raw_ordinal(u64::MAX).is_none());
    assert_send_sync::<DxfPolylinePolygonMeshSmoothingDirectory>();
    assert_copy::<DxfPolylinePolygonMeshSmoothingEntry>();
    Ok(())
}

fn assert_smoothing(
    directory: &DxfPolylinePolygonMeshSmoothingDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 1);
    assert_eq!(
        directory.entries()[0].state(),
        DxfPolylinePolygonMeshSmoothingState::Available {
            m_density: -2,
            n_density: 3,
            surface_type: DxfPolylinePolygonMeshSmoothSurfaceType::QuadraticBSpline,
        }
    );
    let raw = directory.entries()[0]
        .record()
        .record()
        .sequence()
        .polyline_record()
        .ordinal();
    assert_eq!(
        directory.entry_for_polyline_raw_ordinal(raw),
        Some(directory.entries()[0])
    );
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n16\n71\n1\n72\n1\n73\n-2\n74\n3\n75\n5\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n").into_bytes()
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
    for (code, value) in [(70, 16), (71, 1), (72, 1), (73, -2), (74, 3), (75, 5)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    push_i16(&mut bytes, version, 70, 64)?;
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

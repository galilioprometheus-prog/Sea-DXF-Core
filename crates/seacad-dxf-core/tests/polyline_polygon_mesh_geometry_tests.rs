use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylinePolygonMeshCellCorner,
    DxfPolylinePolygonMeshCellGeometryDirectory, DxfPolylinePolygonMeshCellGeometryEntry,
    DxfPolylinePolygonMeshCellGeometryState, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_wcs_cell_corner_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let a =
            ascii.polyline_polygon_mesh_cell_geometry_directory(&DxfCancellationToken::default())?;
        let b = binary
            .polyline_polygon_mesh_cell_geometry_directory(&DxfCancellationToken::default())?;
        assert_geometry(&a)?;
        assert_geometry(&b)?;
        assert_eq!(
            state_bits(a.cells()[0].state())?,
            state_bits(b.cells()[0].state())?
        );
    }
    Ok(())
}

#[test]
fn unavailable_corner_components_fail_the_cell_with_exact_corner() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n16\n71\n2\n72\n2\n0\nVERTEX\n70\n64\n10\n1\n20\n2\n30\n3\n0\nVERTEX\n70\n64\n10\n4\n20\n5\n0\nVERTEX\n70\n64\n10\n7\n20\n8\n30\n9\n0\nVERTEX\n70\n64\n10\n10\n20\n11\n30\n12\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_polygon_mesh_cell_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.cells().len(), 1);
    assert_eq!(
        directory.cells()[0].state(),
        DxfPolylinePolygonMeshCellGeometryState::CoordinateUnavailable {
            corner: DxfPolylinePolygonMeshCellCorner::M0N1,
            component_states: [
                DxfSemanticValueState::Explicit,
                DxfSemanticValueState::Explicit,
                DxfSemanticValueState::Invalid,
            ],
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
        document.polyline_polygon_mesh_cell_geometry_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_polygon_mesh_cell_geometry_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.topology_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.vertex_semantic_directory().source_id()
    );
    assert!(directory.cell(u64::MAX).is_none());
    assert!(directory.cells_for_polyline_raw_ordinal(u64::MAX).is_none());
    assert_send_sync::<DxfPolylinePolygonMeshCellGeometryDirectory>();
    assert_copy::<DxfPolylinePolygonMeshCellGeometryEntry>();
    Ok(())
}

fn assert_geometry(
    directory: &DxfPolylinePolygonMeshCellGeometryDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cells().len(), 1);
    assert_eq!(
        state_bits(directory.cells()[0].state())?,
        [
            bits([1.0, 2.0, 3.0]),
            bits([4.0, 5.0, 6.0]),
            bits([10.0, 11.0, 12.0]),
            bits([7.0, 8.0, 9.0]),
        ]
    );
    let raw = directory.cells()[0]
        .cell()
        .record()
        .sequence()
        .polyline_record()
        .ordinal();
    assert_eq!(
        directory
            .cells_for_polyline_raw_ordinal(raw)
            .ok_or(io::Error::other("mesh cells"))?
            .len(),
        1
    );
    Ok(())
}

fn state_bits(state: DxfPolylinePolygonMeshCellGeometryState) -> Result<[[u64; 3]; 4], io::Error> {
    let DxfPolylinePolygonMeshCellGeometryState::Available {
        m0_n0,
        m0_n1,
        m1_n1,
        m1_n0,
    } = state
    else {
        return Err(io::Error::other("available cell geometry"));
    };
    Ok([
        m0_n0.map(|value| value.to_bits()),
        m0_n1.map(|value| value.to_bits()),
        m1_n1.map(|value| value.to_bits()),
        m1_n0.map(|value| value.to_bits()),
    ])
}

fn bits(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    let mut body = String::from("0\nPOLYLINE\n70\n16\n71\n2\n72\n2\n");
    for position in [[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12]] {
        body.push_str(&format!(
            "0\nVERTEX\n70\n64\n10\n{}\n20\n{}\n30\n{}\n",
            position[0], position[1], position[2]
        ));
    }
    body.push_str("0\nSEQEND\n");
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n").into_bytes()
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
    push_i16(&mut bytes, version, 70, 16)?;
    push_i16(&mut bytes, version, 71, 2)?;
    push_i16(&mut bytes, version, 72, 2)?;
    for position in [
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
        [7.0, 8.0, 9.0],
        [10.0, 11.0, 12.0],
    ] {
        push_string(&mut bytes, version, 0, b"VERTEX")?;
        push_i16(&mut bytes, version, 70, 64)?;
        for (code, value) in [10, 20, 30].into_iter().zip(position) {
            push_double(&mut bytes, version, code, value)?;
        }
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

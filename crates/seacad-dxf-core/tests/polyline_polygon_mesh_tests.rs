use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineFamily,
    DxfPolylinePolygonMeshCellEntry, DxfPolylinePolygonMeshDirectory,
    DxfPolylinePolygonMeshRecordEntry, DxfPolylinePolygonMeshRecordState, DxfPolylineSequenceState,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type CellEvidence = (u64, u16, u16, bool, bool, [u64; 4]);

#[test]
fn every_dialect_has_ascii_binary_closed_grid_topology_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let a = ascii.polyline_polygon_mesh_directory(&DxfCancellationToken::default())?;
        let b = binary.polyline_polygon_mesh_directory(&DxfCancellationToken::default())?;
        assert_valid(&a)?;
        assert_valid(&b)?;
        assert_eq!(evidence(&a), evidence(&b));
    }
    Ok(())
}

#[test]
fn incomplete_family_count_and_vertex_failures_emit_no_cells() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n16\n71\n1\n72\n1\n0\nVERTEX\n70\n64\n0\nPOINT\n0\nPOLYLINE\n70\n0\n0\nSEQEND\n0\nPOLYLINE\n70\n24\n0\nSEQEND\n0\nPOLYLINE\n70\n16\n71\n1\n72\n1\n0\nVERTEX\n70\n32\n0\nSEQEND\n0\nPOLYLINE\n70\n16\n71\n.\n72\n1\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nPOLYLINE\n70\n16\n71\n0\n72\n2\n0\nSEQEND\n0\nPOLYLINE\n70\n16\n71\n2\n72\n2\n0\nVERTEX\n70\n64\n0\nVERTEX\n70\n64\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_polygon_mesh_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 7);
    assert!(directory.cells().is_empty());
    assert_eq!(
        directory.records()[0].state(),
        DxfPolylinePolygonMeshRecordState::IncompleteSequence {
            sequence_state: DxfPolylineSequenceState::Interrupted
        }
    );
    assert_eq!(
        directory.records()[1].state(),
        DxfPolylinePolygonMeshRecordState::UnsupportedFamily {
            family: DxfPolylineFamily::TwoDimensional
        }
    );
    assert_eq!(
        directory.records()[2].state(),
        DxfPolylinePolygonMeshRecordState::IndeterminateFamily
    );
    assert_eq!(
        directory.records()[3].state(),
        DxfPolylinePolygonMeshRecordState::InconsistentVertex {
            sequence_vertex_ordinal: 0
        }
    );
    assert_eq!(
        directory.records()[4].state(),
        DxfPolylinePolygonMeshRecordState::GridCountsUnavailable
    );
    assert_eq!(
        directory.records()[5].state(),
        DxfPolylinePolygonMeshRecordState::NonPositiveGridCount {
            m_vertex_count: 0,
            n_vertex_count: 2
        }
    );
    assert_eq!(
        directory.records()[6].state(),
        DxfPolylinePolygonMeshRecordState::VertexCountMismatch {
            expected: 4,
            observed: 3
        }
    );
    assert!(
        directory
            .records()
            .iter()
            .all(|record| record.cell_range().is_empty())
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
        document.polyline_polygon_mesh_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.polyline_polygon_mesh_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.family_semantic_directory().source_id()
    );
    assert!(directory.cell(u64::MAX).is_none());
    assert!(
        directory
            .record_for_polyline_raw_ordinal(u64::MAX)
            .is_none()
    );
    assert_send_sync::<DxfPolylinePolygonMeshDirectory>();
    assert_copy::<DxfPolylinePolygonMeshRecordEntry>();
    assert_copy::<DxfPolylinePolygonMeshCellEntry>();
    Ok(())
}

fn assert_valid(directory: &DxfPolylinePolygonMeshDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.cells().len(), 6);
    assert_eq!(
        directory.records()[0].state(),
        DxfPolylinePolygonMeshRecordState::Available {
            m_vertex_count: 2,
            n_vertex_count: 3,
            closed_m: true,
            closed_n: true
        }
    );
    assert_eq!(directory.records()[0].cell_range().len(), 6);
    assert_eq!(
        evidence(directory)[0],
        (0, 0, 0, false, false, [0, 1, 4, 3])
    );
    assert_eq!(evidence(directory)[2], (2, 0, 2, false, true, [2, 0, 3, 5]));
    assert_eq!(evidence(directory)[3], (3, 1, 0, true, false, [3, 4, 1, 0]));
    let raw = directory.records()[0]
        .record()
        .sequence()
        .polyline_record()
        .ordinal();
    assert_eq!(
        directory
            .cells_for_polyline_raw_ordinal(raw)
            .ok_or(io::Error::other("cells"))?
            .len(),
        6
    );
    Ok(())
}

fn evidence(directory: &DxfPolylinePolygonMeshDirectory) -> Vec<CellEvidence> {
    directory
        .cells()
        .iter()
        .map(|cell| {
            (
                cell.record_cell_ordinal(),
                cell.m_index(),
                cell.n_index(),
                cell.wraps_m(),
                cell.wraps_n(),
                [
                    cell.m0_n0().sequence_vertex_ordinal(),
                    cell.m0_n1().sequence_vertex_ordinal(),
                    cell.m1_n1().sequence_vertex_ordinal(),
                    cell.m1_n0().sequence_vertex_ordinal(),
                ],
            )
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    let mut body = String::from("0\nPOLYLINE\n70\n49\n71\n2\n72\n3\n");
    for _ in 0..6 {
        body.push_str("0\nVERTEX\n70\n64\n");
    }
    body.push_str("0\nSEQEND\n");
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{body}0\nENDSEC\n0\nEOF\n").into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut b = DXF_BINARY_SENTINEL.to_vec();
    for (c, v) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"POLYLINE"),
    ] {
        string(&mut b, version, c, v)?;
    }
    i16v(&mut b, version, 70, 49)?;
    i16v(&mut b, version, 71, 2)?;
    i16v(&mut b, version, 72, 3)?;
    for _ in 0..6 {
        string(&mut b, version, 0, b"VERTEX")?;
        i16v(&mut b, version, 70, 64)?;
    }
    string(&mut b, version, 0, b"SEQEND")?;
    string(&mut b, version, 0, b"ENDSEC")?;
    string(&mut b, version, 0, b"EOF")?;
    Ok(b)
}
fn code(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16) -> io::Result<()> {
    if v == DxfAcadVersion::Ac1009 {
        b.push(u8::try_from(c).map_err(|_| io::Error::other("code"))?)
    } else {
        b.extend_from_slice(&c.to_le_bytes())
    }
    Ok(())
}
fn string(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16, s: &[u8]) -> io::Result<()> {
    code(b, v, c)?;
    b.extend_from_slice(s);
    b.push(0);
    Ok(())
}
fn i16v(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16, x: i16) -> io::Result<()> {
    code(b, v, c)?;
    b.extend_from_slice(&x.to_le_bytes());
    Ok(())
}
fn open_ascii<'a>(s: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut o = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        s,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut o,
    )
}
fn open_binary<'a>(s: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut o = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        s,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut o,
    )
}
fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

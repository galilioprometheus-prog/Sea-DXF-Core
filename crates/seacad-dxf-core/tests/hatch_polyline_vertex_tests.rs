use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchPolylineVertexCardState,
    DxfHatchPolylineVertexCountRelation, DxfHatchPolylineVertexDirectory,
    DxfHatchPolylineVertexGroupingState, DxfHatchPolylineVertexRole, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};
use std::{error::Error, io};

#[test]
fn every_dialect_groups_ascii_binary_vertices() -> Result<(), Box<dyn Error>> {
    for v in DxfAcadVersion::SUPPORTED {
        let a = ascii(v, "20\n9\n42\n8\n10\n1\n20\n2\n42\n0.5\n10\n3\n20\n4\n", 2);
        let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
        let ad = open_a(&s)?.hatch_polyline_vertex_directory(&DxfCancellationToken::default())?;
        let b = binary(v)?;
        let s = DxfMemorySource::new(&b, DxfResourceProfile::Safe)?;
        let bd = open_b(&s)?.hatch_polyline_vertex_directory(&DxfCancellationToken::default())?;
        for d in [&ad, &bd] {
            let DxfHatchPolylineVertexGroupingState::Grouped(g) = d.paths()[0].state() else {
                return Err(io::Error::other("grouped").into());
            };
            assert_eq!(g.vertex_count(), 2);
            assert_eq!(g.orphan_count(), 2);
            assert_eq!(
                g.count_relation(),
                DxfHatchPolylineVertexCountRelation::Matched { count: 2 }
            );
            let vs = d
                .vertices_for_path(0)
                .ok_or_else(|| io::Error::other("vertices"))?;
            assert_eq!(vs[0].y_state(), DxfHatchPolylineVertexCardState::Unique);
            assert_eq!(vs[0].bulge_state(), DxfHatchPolylineVertexCardState::Unique);
            assert_eq!(vs[1].bulge_state(), DxfHatchPolylineVertexCardState::Absent);
            assert_eq!(
                d.orphans_for_path(0)
                    .ok_or_else(|| io::Error::other("orphans"))?
                    .iter()
                    .map(|m| m.role())
                    .collect::<Vec<_>>(),
                vec![
                    DxfHatchPolylineVertexRole::Y,
                    DxfHatchPolylineVertexRole::Bulge
                ]
            );
        }
    }
    Ok(())
}

#[test]
fn duplicate_missing_and_count_mismatch_remain_explicit() -> Result<(), Box<dyn Error>> {
    let a = ascii(
        DxfAcadVersion::Ac1032,
        "10\n1\n20\n2\n20\n3\n42\n1\n42\n2\n10\n4\n",
        3,
    );
    let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
    let d = open_a(&s)?.hatch_polyline_vertex_directory(&DxfCancellationToken::default())?;
    let v = d
        .vertices_for_path(0)
        .ok_or_else(|| io::Error::other("vertices"))?;
    assert_eq!(
        v[0].y_state(),
        DxfHatchPolylineVertexCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        v[0].bulge_state(),
        DxfHatchPolylineVertexCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(v[1].y_state(), DxfHatchPolylineVertexCardState::Absent);
    let DxfHatchPolylineVertexGroupingState::Grouped(g) = d.paths()[0].state() else {
        return Err(io::Error::other("grouped").into());
    };
    assert_eq!(
        g.count_relation(),
        DxfHatchPolylineVertexCountRelation::Mismatched {
            declared: 3,
            observed: 2
        }
    );
    Ok(())
}

#[test]
fn edge_and_invalid_header_publish_no_vertices() -> Result<(), Box<dyn Error>> {
    for (flag, payload) in [(0, ""), (2, "72\n.\n73\n0\n93\n0\n")] {
        let mut a = ascii(DxfAcadVersion::Ac1032, payload, 0);
        a = String::from_utf8(a)?
            .replace("92\n2\n", &format!("92\n{flag}\n"))
            .into_bytes();
        let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
        let d = open_a(&s)?.hatch_polyline_vertex_directory(&DxfCancellationToken::default())?;
        assert!(matches!(
            d.paths()[0].state(),
            DxfHatchPolylineVertexGroupingState::NotPolyline
                | DxfHatchPolylineVertexGroupingState::HeaderUnavailable(_)
        ));
        assert!(d.vertices_for_path(0).is_none());
    }
    Ok(())
}

#[test]
fn cancellation_bounds_identity_and_traits_hold() -> Result<(), Box<dyn Error>> {
    let a = ascii(DxfAcadVersion::Ac1032, "10\n1\n20\n2\n", 1);
    let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
    let doc = open_a(&s)?;
    let c = DxfCancellationToken::default();
    c.cancel();
    assert!(matches!(
        doc.hatch_polyline_vertex_directory(&c),
        Err(DxfError::Cancelled)
    ));
    let d = doc.hatch_polyline_vertex_directory(&DxfCancellationToken::default())?;
    assert_eq!(d.path(0), Some(d.paths()[0]));
    assert_eq!(d.path(u64::MAX), None);
    assert!(d.vertices_for_path(u64::MAX).is_none());
    assert!(d.members_for_vertex(u64::MAX).is_none());
    assert!(d.orphans_for_path(u64::MAX).is_none());
    assert_eq!(d.source_id(), d.header_directory().source_id());
    send_sync::<DxfHatchPolylineVertexDirectory>();
    Ok(())
}

fn ascii(v: DxfAcadVersion, p: &str, n: u32) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n2\n72\n1\n73\n1\n93\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",v.code(),n,p).into_bytes()
}
fn binary(v: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut b = DXF_BINARY_SENTINEL.to_vec();
    for (c, x) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, v.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        ps(&mut b, v, c, x)?
    }
    pi32(&mut b, v, 91, 1)?;
    pi32(&mut b, v, 92, 2)?;
    pi16(&mut b, v, 72, 1)?;
    pi16(&mut b, v, 73, 1)?;
    pi32(&mut b, v, 93, 2)?;
    pd(&mut b, v, 20, 9.)?;
    pd(&mut b, v, 42, 8.)?;
    pd(&mut b, v, 10, 1.)?;
    pd(&mut b, v, 20, 2.)?;
    pd(&mut b, v, 42, 0.5)?;
    pd(&mut b, v, 10, 3.)?;
    pd(&mut b, v, 20, 4.)?;
    pi32(&mut b, v, 97, 0)?;
    pi16(&mut b, v, 75, 0)?;
    ps(&mut b, v, 0, b"ENDSEC")?;
    ps(&mut b, v, 0, b"EOF")?;
    Ok(b)
}
fn pc(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16) -> io::Result<()> {
    if v == DxfAcadVersion::Ac1009 {
        b.push(u8::try_from(c).map_err(|_| io::Error::other("code"))?)
    } else {
        b.extend_from_slice(&c.to_le_bytes())
    }
    Ok(())
}
fn ps(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16, x: &[u8]) -> io::Result<()> {
    pc(b, v, c)?;
    b.extend_from_slice(x);
    b.push(0);
    Ok(())
}
fn pi16(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16, x: i16) -> io::Result<()> {
    pc(b, v, c)?;
    b.extend_from_slice(&x.to_le_bytes());
    Ok(())
}
fn pi32(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16, x: i32) -> io::Result<()> {
    pc(b, v, c)?;
    b.extend_from_slice(&x.to_le_bytes());
    Ok(())
}
fn pd(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16, x: f64) -> io::Result<()> {
    pc(b, v, c)?;
    b.extend_from_slice(&x.to_le_bytes());
    Ok(())
}
fn open_a<'a>(s: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut o = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        s,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut o,
    )
}
fn open_b<'a>(s: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut o = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        s,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut o,
    )
}
fn send_sync<T: Send + Sync>() {}

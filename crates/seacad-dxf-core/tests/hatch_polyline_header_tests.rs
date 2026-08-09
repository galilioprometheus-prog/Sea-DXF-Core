use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchPolylineHeaderDirectory, DxfHatchPolylineHeaderIssue,
    DxfHatchPolylineHeaderState, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};
use std::{error::Error, io};

#[test]
fn every_dialect_has_ascii_binary_header_parity() -> Result<(), Box<dyn Error>> {
    for v in DxfAcadVersion::SUPPORTED {
        let a = ascii(v, "72\n1\n73\n0\n93\n2\n");
        let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
        let ad = open_a(&s)?.hatch_polyline_header_directory(&DxfCancellationToken::default())?;
        let b = binary(v)?;
        let s = DxfMemorySource::new(&b, DxfResourceProfile::Safe)?;
        let bd = open_b(&s)?.hatch_polyline_header_directory(&DxfCancellationToken::default())?;
        for d in [&ad, &bd] {
            let DxfHatchPolylineHeaderState::Explicit(h) = d.entries()[0].state() else {
                return Err(io::Error::other("explicit").into());
            };
            assert!(h.has_bulge().value());
            assert!(!h.is_closed().value());
            assert_eq!(h.vertex_count().value(), 2);
        }
    }
    Ok(())
}

#[test]
fn edges_are_not_polyline() -> Result<(), Box<dyn Error>> {
    let a = ascii(DxfAcadVersion::Ac1032, "");
    let a = String::from_utf8(a)?
        .replace("92\n2\n", "92\n0\n")
        .into_bytes();
    let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
    let d = open_a(&s)?.hatch_polyline_header_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        d.entries()[0].state(),
        DxfHatchPolylineHeaderState::NotPolyline
    );
    Ok(())
}

#[test]
fn absent_multiple_invalid_and_domains_fail_closed() -> Result<(), Box<dyn Error>> {
    for (payload, check) in [
        ("73\n0\n93\n1\n", 72),
        ("72\n0\n72\n1\n73\n0\n93\n1\n", 72),
        ("72\n.\n73\n0\n93\n1\n", 0),
        ("72\n2\n73\n0\n93\n-1\n", 0),
    ] {
        let a = ascii(DxfAcadVersion::Ac1032, payload);
        let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
        let d = open_a(&s)?.hatch_polyline_header_directory(&DxfCancellationToken::default())?;
        let DxfHatchPolylineHeaderState::Unavailable(i) = d.entries()[0].state() else {
            return Err(io::Error::other("unavailable").into());
        };
        if check != 0 {
            assert!(
                matches!(i,DxfHatchPolylineHeaderIssue::FieldAbsent{group_code} if group_code==check)
                    | matches!(i,DxfHatchPolylineHeaderIssue::FieldMultiple{group_code,..} if group_code==check)
            );
        }
    }
    Ok(())
}

#[test]
fn cancellation_bounds_and_traits_hold() -> Result<(), Box<dyn Error>> {
    let a = ascii(DxfAcadVersion::Ac1032, "72\n1\n73\n1\n93\n0\n");
    let s = DxfMemorySource::new(&a, DxfResourceProfile::Safe)?;
    let doc = open_a(&s)?;
    let c = DxfCancellationToken::default();
    c.cancel();
    assert!(matches!(
        doc.hatch_polyline_header_directory(&c),
        Err(DxfError::Cancelled)
    ));
    let d = doc.hatch_polyline_header_directory(&DxfCancellationToken::default())?;
    assert_eq!(d.entry(0), Some(d.entries()[0]));
    assert_eq!(d.entry(u64::MAX), None);
    assert!(d.entries_for_subclass(u64::MAX).is_empty());
    assert!(d.entries_for_raw_record(u64::MAX).is_empty());
    assert_eq!(d.source_id(), d.flag_directory().source_id());
    send_sync::<DxfHatchPolylineHeaderDirectory>();
    Ok(())
}

fn ascii(v: DxfAcadVersion, p: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n2\n{}75\n0\n0\nENDSEC\n0\nEOF\n",v.code(),p).into_bytes()
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
    pi16(&mut b, v, 73, 0)?;
    pi32(&mut b, v, 93, 2)?;
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

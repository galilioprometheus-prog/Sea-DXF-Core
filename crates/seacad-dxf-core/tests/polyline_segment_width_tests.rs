use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineEffectiveWidthOrigin,
    DxfPolylineSegmentEffectiveWidths, DxfPolylineSegmentWidthDirectory,
    DxfPolylineSegmentWidthIssue, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type WidthEvidence = (
    u64,
    DxfPolylineEffectiveWidthOrigin,
    u64,
    DxfPolylineEffectiveWidthOrigin,
);

#[test]
fn every_dialect_has_ascii_binary_effective_width_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let a = ascii.polyline_segment_width_directory(&DxfCancellationToken::default())?;
        let b = binary.polyline_segment_width_directory(&DxfCancellationToken::default())?;
        assert_eq!(evidence(&a)?, evidence(&b)?);
        assert_eq!(
            evidence(&a)?,
            vec![
                (
                    5.0_f64.to_bits(),
                    DxfPolylineEffectiveWidthOrigin::ParentDefault,
                    6.0_f64.to_bits(),
                    DxfPolylineEffectiveWidthOrigin::ParentDefault
                ),
                (
                    0.0_f64.to_bits(),
                    DxfPolylineEffectiveWidthOrigin::Vertex,
                    2.0_f64.to_bits(),
                    DxfPolylineEffectiveWidthOrigin::Vertex
                ),
            ]
        );
        let unsupported = b
            .widths_for_segment(2)?
            .ok_or(io::Error::other("3D widths"))?;
        assert_eq!(
            unsupported.start(),
            Err(DxfPolylineSegmentWidthIssue::UnsupportedForThreeDimensional)
        );
        assert_eq!(
            unsupported.end(),
            Err(DxfPolylineSegmentWidthIssue::UnsupportedForThreeDimensional)
        );
    }
    Ok(())
}

#[test]
fn invalid_vertex_and_parent_components_fail_independently() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n0\n20\n0\n30\n0\n70\n0\n40\n5\n41\n.\n0\nVERTEX\n70\n0\n10\n0\n20\n0\n30\n0\n40\n.\n0\nVERTEX\n70\n0\n10\n1\n20\n0\n30\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_segment_width_directory(&DxfCancellationToken::default())?;
    let widths = directory
        .widths_for_segment(0)?
        .ok_or(io::Error::other("widths"))?;
    assert_eq!(
        widths.start(),
        Err(DxfPolylineSegmentWidthIssue::VertexValueUnavailable)
    );
    assert_eq!(
        widths.end(),
        Err(DxfPolylineSegmentWidthIssue::ParentDefaultUnavailable)
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
        document.polyline_segment_width_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.polyline_segment_width_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.segment_semantic_directory().source_id()
    );
    assert!(directory.widths_for_segment(u64::MAX)?.is_none());
    assert_copy::<DxfPolylineSegmentEffectiveWidths>();
    assert_send_sync::<DxfPolylineSegmentWidthDirectory>();
    Ok(())
}

fn evidence(
    directory: &DxfPolylineSegmentWidthDirectory,
) -> Result<Vec<WidthEvidence>, Box<dyn Error>> {
    (0_u64..2)
        .map(|ordinal| {
            let widths = directory
                .widths_for_segment(ordinal)?
                .ok_or(io::Error::other("widths"))?;
            let start = widths.start().map_err(|_| io::Error::other("start"))?;
            let end = widths.end().map_err(|_| io::Error::other("end"))?;
            Ok((
                start.value().to_bits(),
                start.origin(),
                end.value().to_bits(),
                end.origin(),
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n0\n20\n0\n30\n0\n70\n0\n40\n5\n41\n6\n0\nVERTEX\n70\n0\n10\n0\n20\n0\n30\n0\n0\nVERTEX\n70\n0\n10\n1\n20\n0\n30\n0\n40\n0\n41\n2\n0\nVERTEX\n70\n0\n10\n2\n20\n0\n30\n0\n0\nSEQEND\n0\nPOLYLINE\n10\n0\n20\n0\n30\n0\n70\n8\n0\nVERTEX\n70\n32\n10\n0\n20\n0\n30\n0\n0\nVERTEX\n70\n32\n10\n1\n20\n1\n30\n1\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n").into_bytes()
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
    ] {
        string(&mut b, version, c, v)?;
    }
    string(&mut b, version, 0, b"POLYLINE")?;
    for (c, v) in [(10, 0.0), (20, 0.0), (30, 0.0), (40, 5.0), (41, 6.0)] {
        double(&mut b, version, c, v)?;
    }
    i16v(&mut b, version, 70, 0)?;
    for (i, p) in [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
        .into_iter()
        .enumerate()
    {
        string(&mut b, version, 0, b"VERTEX")?;
        i16v(&mut b, version, 70, 0)?;
        for (c, v) in [(10, p[0]), (20, p[1]), (30, p[2])] {
            double(&mut b, version, c, v)?;
        }
        if i == 1 {
            double(&mut b, version, 40, 0.0)?;
            double(&mut b, version, 41, 2.0)?;
        }
    }
    string(&mut b, version, 0, b"SEQEND")?;
    string(&mut b, version, 0, b"POLYLINE")?;
    for (c, v) in [(10, 0.0), (20, 0.0), (30, 0.0)] {
        double(&mut b, version, c, v)?;
    }
    i16v(&mut b, version, 70, 8)?;
    for p in [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]] {
        string(&mut b, version, 0, b"VERTEX")?;
        i16v(&mut b, version, 70, 32)?;
        for (c, v) in [(10, p[0]), (20, p[1]), (30, p[2])] {
            double(&mut b, version, c, v)?;
        }
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
fn double(b: &mut Vec<u8>, v: DxfAcadVersion, c: i16, x: f64) -> io::Result<()> {
    code(b, v, c)?;
    b.extend_from_slice(&x.to_bits().to_le_bytes());
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

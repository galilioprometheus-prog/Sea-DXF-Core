use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchPolylineBulgeDirectory, DxfHatchPolylineBulgeIssue, DxfHatchPolylineVertexCardState,
    DxfHatchPolylineVertexNumericIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};
use std::{error::Error, io};

#[test]
fn every_dialect_keeps_explicit_bulges_and_defaults_absence() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii(version, 2, 1, "10\n1\n20\n2\n42\n0.5\n10\n3\n20\n4\n", 2);
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory =
            open_ascii(&source)?.hatch_polyline_bulge_directory(&DxfCancellationToken::default())?;
        let binary = binary(
            version,
            2,
            1,
            &[(1.0, 2.0, Some(0.5_f64.to_bits())), (3.0, 4.0, None)],
        )?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_polyline_bulge_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            let [first, second] = directory.entries() else {
                return Err(io::Error::other("two bulge entries").into());
            };
            assert!(first.has_bulge().value());
            assert_eq!(first.bulge().state(), DxfSemanticValueState::Explicit);
            assert_eq!(first.bulge_value().map(|v| v.to_f64()), Some(0.5));
            assert!(first.bulge().raw_provenance().is_some());
            assert_eq!(second.bulge().state(), DxfSemanticValueState::Defaulted);
            assert_eq!(second.bulge_value().map(|v| v.to_bits()), Some(0));
            assert_eq!(second.bulge().raw_provenance(), None);
        }
    }
    Ok(())
}

#[test]
fn disabled_header_rejects_present_fields_without_losing_numeric_evidence()
-> Result<(), Box<dyn Error>> {
    let ascii = ascii(
        DxfAcadVersion::Ac1032,
        2,
        0,
        "10\n1\n20\n2\n42\n0.5\n10\n3\n20\n4\n42\n1\n42\n2\n10\n5\n20\n6\n",
        3,
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_polyline_bulge_directory(&DxfCancellationToken::default())?;
    let [unique, multiple, absent] = directory.entries() else {
        return Err(io::Error::other("three bulge entries").into());
    };
    assert!(!unique.has_bulge().value());
    assert_eq!(
        unique.bulge().invalid_issue(),
        Some(&DxfHatchPolylineBulgeIssue::PresentWhenHeaderDisallows {
            state: DxfHatchPolylineVertexCardState::Unique
        })
    );
    assert!(unique.bulge().raw_provenance().is_some());
    assert_eq!(
        multiple.bulge().invalid_issue(),
        Some(&DxfHatchPolylineBulgeIssue::PresentWhenHeaderDisallows {
            state: DxfHatchPolylineVertexCardState::Multiple {
                occurrence_count: 2
            }
        })
    );
    assert_eq!(multiple.bulge().raw_provenance(), None);
    assert_eq!(absent.bulge().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(absent.bulge_value().map(|v| v.to_bits()), Some(0));
    assert_eq!(
        unique.numeric().components().bulge().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        multiple.numeric().components().bulge().invalid_issue(),
        Some(&DxfHatchPolylineVertexNumericIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    Ok(())
}

#[test]
fn enabled_header_preserves_ascii_and_binary_numeric_failures() -> Result<(), Box<dyn Error>> {
    let ascii = ascii(DxfAcadVersion::Ac1032, 2, 1, "10\n1\n20\n2\n42\nbad\n", 1);
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_polyline_bulge_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].bulge().invalid_issue(),
        Some(DxfHatchPolylineBulgeIssue::Numeric(
            DxfHatchPolylineVertexNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            )
        ))
    ));
    assert!(directory.entries()[0].bulge().raw_provenance().is_some());

    let binary = binary(
        DxfAcadVersion::Ac1032,
        2,
        1,
        &[(1.0, 2.0, Some(f64::INFINITY.to_bits()))],
    )?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory =
        open_binary(&source)?.hatch_polyline_bulge_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].bulge().invalid_issue(),
        Some(DxfHatchPolylineBulgeIssue::Numeric(
            DxfHatchPolylineVertexNumericIssue::NonFiniteDouble(_)
        ))
    ));
    assert!(directory.entries()[0].bulge().raw_provenance().is_some());
    Ok(())
}

#[test]
fn unavailable_paths_cancellation_bounds_identity_traits_and_redaction_hold()
-> Result<(), Box<dyn Error>> {
    for (path_flag, payload) in [(0, ""), (2, "72\n.\n73\n0\n93\n0\n")] {
        let ascii = ascii(DxfAcadVersion::Ac1032, path_flag, 1, payload, 0);
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let directory =
            open_ascii(&source)?.hatch_polyline_bulge_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert!(directory.entries_for_path(0).is_none());
    }

    let ascii = ascii(
        DxfAcadVersion::Ac1032,
        2,
        1,
        "10\n1\n20\n2\n42\n12345.625\n",
        1,
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_polyline_bulge_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.hatch_polyline_bulge_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.numeric_directory().source_id()
    );
    assert!(!format!("{:?}", directory.entries()[0].bulge()).contains("12345.625"));
    send_sync::<DxfHatchPolylineBulgeDirectory>();
    Ok(())
}

fn ascii(
    version: DxfAcadVersion,
    path_flag: i32,
    has_bulge: i16,
    payload: &str,
    count: u32,
) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n{}\n72\n{}\n73\n1\n93\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        path_flag,
        has_bulge,
        count,
        payload
    )
    .into_bytes()
}

fn binary(
    version: DxfAcadVersion,
    path_flag: i32,
    has_bulge: i16,
    vertices: &[(f64, f64, Option<u64>)],
) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, path_flag)?;
    push_i16(&mut bytes, version, 72, has_bulge)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i32(
        &mut bytes,
        version,
        93,
        i32::try_from(vertices.len()).map_err(|_| io::Error::other("vertex count"))?,
    )?;
    for (x, y, bulge) in vertices.iter().copied() {
        push_double_bits(&mut bytes, version, 10, x.to_bits())?;
        push_double_bits(&mut bytes, version, 20, y.to_bits())?;
        if let Some(bits) = bulge {
            push_double_bits(&mut bytes, version, 42, bits)?;
        }
    }
    push_i32(&mut bytes, version, 97, 0)?;
    push_i16(&mut bytes, version, 75, 0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
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

fn push_i32(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i32) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_double_bits(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    bits: u64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&bits.to_le_bytes());
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

fn send_sync<T: Send + Sync>() {}

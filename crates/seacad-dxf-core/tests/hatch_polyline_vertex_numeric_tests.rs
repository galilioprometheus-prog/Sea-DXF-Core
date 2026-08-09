use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchPolylineVertexNumericDirectory, DxfHatchPolylineVertexNumericIssue, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};
use std::{error::Error, io};

#[test]
fn every_dialect_decodes_finite_ascii_binary_components() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii(version, 2, "10\n-0\n20\n2.5\n42\n-0.25\n", 1);
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_polyline_vertex_numeric_directory(&DxfCancellationToken::default())?;
        let binary = binary(version, (-0.0_f64).to_bits(), 2.5, Some(-0.25))?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_polyline_vertex_numeric_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            let [entry] = directory.entries() else {
                return Err(io::Error::other("one numeric vertex").into());
            };
            let components = entry.components();
            assert_eq!(components.x().state(), DxfSemanticValueState::Explicit);
            assert_eq!(components.y().state(), DxfSemanticValueState::Explicit);
            assert_eq!(components.bulge().state(), DxfSemanticValueState::Explicit);
            assert_eq!(number(components.x())?.to_bits(), (-0.0_f64).to_bits());
            assert_eq!(number(components.y())?.to_f64(), 2.5);
            assert_eq!(number(components.bulge())?.to_f64(), -0.25);
            for value in [components.x(), components.y(), components.bulge()] {
                assert_eq!(value.field_provenance().schema_namespace(), "entity.hatch");
                assert!(value.raw_provenance().is_some());
            }
        }
    }
    Ok(())
}

#[test]
fn absent_multiple_and_malformed_values_remain_distinct() -> Result<(), Box<dyn Error>> {
    let ascii = ascii(
        DxfAcadVersion::Ac1032,
        2,
        "10\nbad\n20\n2\n20\n3\n42\n4\n42\n5\n10\n6\n",
        2,
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_polyline_vertex_numeric_directory(&DxfCancellationToken::default())?;
    let [first, second] = directory.entries() else {
        return Err(io::Error::other("two numeric vertices").into());
    };
    assert!(matches!(
        first.components().x().invalid_issue(),
        Some(DxfHatchPolylineVertexNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    for value in [first.components().y(), first.components().bulge()] {
        assert_eq!(value.state(), DxfSemanticValueState::Invalid);
        assert_eq!(
            value.invalid_issue(),
            Some(&DxfHatchPolylineVertexNumericIssue::MultipleValues {
                occurrence_count: 2
            })
        );
        assert_eq!(value.raw_provenance(), None);
    }
    assert_eq!(
        second.components().x().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        second.components().y().state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(
        second.components().bulge().state(),
        DxfSemanticValueState::Absent
    );
    Ok(())
}

#[test]
fn binary_nonfinite_and_unavailable_paths_publish_no_usable_value() -> Result<(), Box<dyn Error>> {
    let binary = binary(
        DxfAcadVersion::Ac1032,
        0x7ff8_0000_0000_0042,
        2.0,
        Some(f64::INFINITY),
    )?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_polyline_vertex_numeric_directory(&DxfCancellationToken::default())?;
    let entry = directory
        .entry(0)
        .ok_or_else(|| io::Error::other("numeric entry"))?;
    for value in [entry.components().x(), entry.components().bulge()] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfHatchPolylineVertexNumericIssue::NonFiniteDouble(_))
        ));
        assert!(value.raw_provenance().is_some());
    }

    for (flag, payload) in [(0, ""), (2, "72\n.\n73\n0\n93\n0\n")] {
        let ascii = ascii(DxfAcadVersion::Ac1032, flag, payload, 0);
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let directory = open_ascii(&source)?
            .hatch_polyline_vertex_numeric_directory(&DxfCancellationToken::default())?;
        assert!(directory.entries().is_empty());
        assert!(directory.entries_for_path(0).is_none());
    }
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii(DxfAcadVersion::Ac1032, 2, "10\n12345.625\n20\n2\n", 1);
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_polyline_vertex_numeric_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.hatch_polyline_vertex_numeric_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.vertex_directory().source_id()
    );
    let debug = format!("{:?}", directory.entries()[0].components().x());
    assert!(!debug.contains("12345.625"));
    send_sync::<DxfHatchPolylineVertexNumericDirectory>();
    Ok(())
}

fn number(
    value: &seacad_dxf_core::DxfHatchPolylineVertexNumericValue,
) -> Result<DxfDouble, io::Error> {
    value
        .value()
        .copied()
        .ok_or_else(|| io::Error::other("explicit number"))
}

fn ascii(version: DxfAcadVersion, flag: i32, payload: &str, count: u32) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n{}\n72\n1\n73\n1\n93\n{}\n{}97\n0\n75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        flag,
        count,
        payload
    )
    .into_bytes()
}

fn binary(version: DxfAcadVersion, x_bits: u64, y: f64, bulge: Option<f64>) -> io::Result<Vec<u8>> {
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
    push_i32(&mut bytes, version, 92, 2)?;
    push_i16(&mut bytes, version, 72, 1)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i32(&mut bytes, version, 93, 1)?;
    push_double_bits(&mut bytes, version, 10, x_bits)?;
    push_double(&mut bytes, version, 20, y)?;
    if let Some(value) = bulge {
        push_double(&mut bytes, version, 42, value)?;
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

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_double_bits(bytes, version, code, value.to_bits())
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

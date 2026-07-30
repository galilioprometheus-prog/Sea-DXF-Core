use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPlanarFaceSemanticDirectory,
    DxfPlanarFaceSemantics, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct VisibilitySignature {
    raw_flags: Option<i16>,
    bits: Option<u16>,
    invisible: Option<[bool; 4]>,
    unknown_bits: Option<u16>,
}

#[test]
fn every_dialect_has_ascii_binary_edge_visibility_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.planar_face_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.planar_face_semantic_directory(&DxfCancellationToken::default())?;

        let expected = [
            VisibilitySignature {
                raw_flags: Some(0),
                bits: Some(0),
                invisible: Some([false; 4]),
                unknown_bits: Some(0),
            },
            VisibilitySignature {
                raw_flags: Some(5),
                bits: Some(5),
                invisible: Some([true, false, true, false]),
                unknown_bits: Some(0),
            },
            VisibilitySignature {
                raw_flags: Some(-32759),
                bits: Some(0x8009),
                invisible: Some([true, false, false, true]),
                unknown_bits: Some(0x8000),
            },
            VisibilitySignature {
                raw_flags: None,
                bits: None,
                invisible: None,
                unknown_bits: None,
            },
        ];
        assert_eq!(signatures(&ascii_directory)?, expected);
        assert_eq!(signatures(&binary_directory)?, expected);
        assert_eq!(semantics(&ascii_directory, 0)?.invisible_edge(4), None);
        assert_eq!(
            semantics(&binary_directory, 0)?.invisible_edge(usize::MAX),
            None
        );
    }
    Ok(())
}

#[test]
fn invalid_and_duplicate_flags_keep_visibility_unavailable() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\n3DFACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n70\n32768\n\
0\n3DFACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n70\n1\n70\n2\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.planar_face_semantic_directory(&DxfCancellationToken::default())?;

    for index in 0..2 {
        let value = semantics(&directory, index)?;
        assert_eq!(value.invisible_edge_flags_value(), None);
        assert_eq!(value.invisible_edge_flags_bits(), None);
        assert_eq!(value.invisible_edges(), None);
        assert_eq!(value.unknown_invisible_edge_flag_bits(), None);
    }
    Ok(())
}

fn signatures(
    directory: &DxfPlanarFaceSemanticDirectory,
) -> Result<Vec<VisibilitySignature>, Box<dyn Error>> {
    directory
        .records()
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let semantic = semantics(directory, index)?;
            Ok(VisibilitySignature {
                raw_flags: semantic.invisible_edge_flags_value(),
                bits: semantic.invisible_edge_flags_bits(),
                invisible: semantic.invisible_edges(),
                unknown_bits: semantic.unknown_invisible_edge_flag_bits(),
            })
        })
        .collect()
}

fn semantics(
    directory: &DxfPlanarFaceSemanticDirectory,
    index: usize,
) -> Result<DxfPlanarFaceSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or(io::Error::other("record"))?;
    directory
        .semantics_for_record(record)?
        .ok_or_else(|| io::Error::other("semantics").into())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
{}\
{}\
{}\
0\nSOLID\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
0\nENDSEC\n0\nEOF\n",
        ascii_face(None),
        ascii_face(Some(5)),
        ascii_face(Some(-32759))
    )
    .into_bytes()
}

fn ascii_face(flags: Option<i16>) -> String {
    let flags = flags.map_or_else(String::new, |value| format!("70\n{value}\n"));
    format!("0\n3DFACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n{flags}")
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_binary_face(&mut bytes, version, None)?;
    push_binary_face(&mut bytes, version, Some(5))?;
    push_binary_face(&mut bytes, version, Some(-32759))?;
    push_string(&mut bytes, version, 0, b"SOLID")?;
    push_corners(&mut bytes, version)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_face(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    flags: Option<i16>,
) -> io::Result<()> {
    push_string(bytes, version, 0, b"3DFACE")?;
    push_corners(bytes, version)?;
    if let Some(value) = flags {
        push_i16(bytes, version, 70, value)?;
    }
    Ok(())
}

fn push_corners(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    for (index, code) in [10_i16, 20, 30, 11, 21, 31, 12, 22, 32]
        .into_iter()
        .enumerate()
    {
        push_double(bytes, version, code, (index + 1) as f64)?;
    }
    Ok(())
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_i16(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: i16,
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, group_code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
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

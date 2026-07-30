use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfPlanarFaceDoubleSemanticValue, DxfPlanarFaceInt16SemanticValue, DxfPlanarFaceKind,
    DxfPlanarFaceSemanticDirectory, DxfPlanarFaceSemanticIssue, DxfPlanarFaceSemantics,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct ScalarSignature {
    state: DxfSemanticValueState,
    bits: Option<u64>,
}

#[derive(Debug, Eq, PartialEq)]
struct RecordSignature {
    kind: DxfPlanarFaceKind,
    corners: [[ScalarSignature; 3]; 4],
    thickness: Option<ScalarSignature>,
    extrusion: Option<[ScalarSignature; 3]>,
    flags: Option<(DxfSemanticValueState, Option<i16>)>,
}

#[test]
fn every_dialect_has_ascii_binary_semantic_and_default_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_semantics =
            ascii.planar_face_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_semantics =
            binary.planar_face_semantic_directory(&DxfCancellationToken::default())?;

        let ascii_signatures = signatures(&ascii_semantics)?;
        assert_eq!(ascii_signatures, signatures(&binary_semantics)?);
        assert_defaults(&ascii_semantics)?;
        assert_defaults(&binary_semantics)?;
    }
    Ok(())
}

#[test]
fn missing_invalid_multiple_and_partial_values_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\n3DFACE\n10\n1\n10\n2\n20\n3\n30\n4\n11\n5\n21\n6\n31\n7\n\
12\n.\n22\n9\n32\n10\n70\n32768\n\
0\nSOLID\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
13\n10\n39\n.\n\
0\nTRACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.planar_face_semantic_directory(&DxfCancellationToken::default())?;

    let face = semantics(&directory, 0)?;
    assert_eq!(
        face.corners()[0][0].invalid_issue(),
        Some(&DxfPlanarFaceSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        face.corners()[2][0].invalid_issue(),
        Some(&DxfPlanarFaceSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        face.corners()[3][0].invalid_issue(),
        Some(&DxfPlanarFaceSemanticIssue::UnavailableDefaultSource)
    );
    assert_eq!(
        face.corners()[3][1].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        face.invisible_edge_flags()
            .and_then(|value| value.invalid_issue()),
        Some(&DxfPlanarFaceSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );

    let solid = semantics(&directory, 1)?;
    assert_eq!(
        solid.corners()[3][0].state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        solid.corners()[3][1].invalid_issue(),
        Some(&DxfPlanarFaceSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(
        solid.thickness().and_then(|value| value.invalid_issue()),
        Some(&DxfPlanarFaceSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );

    let trace = semantics(&directory, 2)?;
    for component in &trace.corners()[3] {
        assert_eq!(
            component.invalid_issue(),
            Some(&DxfPlanarFaceSemanticIssue::MissingRequiredValue)
        );
    }
    assert_eq!(
        trace.thickness().map(|value| value.state()),
        Some(DxfSemanticValueState::Defaulted)
    );
    Ok(())
}

#[test]
fn cancellation_lookups_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.planar_face_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.planar_face_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert!(
        directory
            .semantics_for_record(directory.records()[0])?
            .is_some()
    );

    assert_copy::<DxfPlanarFaceSemantics>();
    assert_send_sync::<DxfPlanarFaceSemanticDirectory>();
    Ok(())
}

fn assert_defaults(directory: &DxfPlanarFaceSemanticDirectory) -> Result<(), Box<dyn Error>> {
    let face = semantics(directory, 0)?;
    assert_eq!(face.corner_value(2), face.corner_value(3));
    assert!(
        face.corners()[3]
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Defaulted)
    );
    assert_eq!(face.invisible_edge_flags_value(), Some(0));
    assert_eq!(face.thickness(), None);
    assert_eq!(face.extrusion(), None);
    assert_eq!(face.corner_value(usize::MAX), None);

    let solid = semantics(directory, 1)?;
    assert_eq!(solid.corner_value(2), solid.corner_value(3));
    assert_eq!(
        solid.thickness_value().map(|value| value.to_bits()),
        Some(0)
    );
    assert_eq!(
        solid.extrusion_value().map(bits3),
        Some([0.0_f64.to_bits(), 0.0_f64.to_bits(), 1.0_f64.to_bits()])
    );

    let trace = semantics(directory, 2)?;
    assert_eq!(
        trace.extrusion_value().map(bits3),
        Some([2.0_f64.to_bits(), 0.0_f64.to_bits(), 1.0_f64.to_bits()])
    );
    assert_eq!(trace.invisible_edge_flags(), None);
    Ok(())
}

fn signatures(
    directory: &DxfPlanarFaceSemanticDirectory,
) -> Result<Vec<RecordSignature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .map(|record| {
            directory
                .semantics_for_record(record)?
                .map(|semantic| RecordSignature {
                    kind: semantic.kind(),
                    corners: std::array::from_fn(|corner| {
                        std::array::from_fn(|axis| scalar(&semantic.corners()[corner][axis]))
                    }),
                    thickness: semantic.thickness().map(scalar),
                    extrusion: semantic
                        .extrusion()
                        .map(|values| std::array::from_fn(|axis| scalar(&values[axis]))),
                    flags: semantic.invisible_edge_flags().map(integer),
                })
                .ok_or_else(|| invalid_data("semantic record"))
        })
        .collect()
}

fn scalar(value: &DxfPlanarFaceDoubleSemanticValue) -> ScalarSignature {
    ScalarSignature {
        state: value.state(),
        bits: value.value().map(|value| value.to_bits()),
    }
}

fn integer(value: &DxfPlanarFaceInt16SemanticValue) -> (DxfSemanticValueState, Option<i16>) {
    (value.state(), value.value().copied())
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
0\n3DFACE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n12\n7\n22\n8\n32\n9\n\
0\nSOLID\n10\n11\n20\n12\n30\n13\n11\n14\n21\n15\n31\n16\n12\n17\n22\n18\n32\n19\n\
0\nTRACE\n10\n21\n20\n22\n30\n23\n11\n24\n21\n25\n31\n26\n12\n27\n22\n28\n32\n29\n\
13\n30\n23\n31\n33\n32\n39\n2.5\n210\n2\n\
0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
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
    push_string(&mut bytes, version, 0, b"3DFACE")?;
    push_corners(&mut bytes, version, 1.0, 3)?;
    push_string(&mut bytes, version, 0, b"SOLID")?;
    push_corners(&mut bytes, version, 11.0, 3)?;
    push_string(&mut bytes, version, 0, b"TRACE")?;
    push_corners(&mut bytes, version, 21.0, 4)?;
    push_double(&mut bytes, version, 39, 2.5)?;
    push_double(&mut bytes, version, 210, 2.0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_corners(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    first: f64,
    count: usize,
) -> io::Result<()> {
    let codes = [[10, 20, 30], [11, 21, 31], [12, 22, 32], [13, 23, 33]];
    for (index, corner) in codes.iter().take(count).enumerate() {
        for (axis, code) in corner.iter().enumerate() {
            push_double(bytes, version, *code, first + (index * 3 + axis) as f64)?;
        }
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

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, group_code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    Ok(())
}

fn bits3(values: [seacad_dxf_core::DxfDouble; 3]) -> [u64; 3] {
    values.map(seacad_dxf_core::DxfDouble::to_bits)
}

fn invalid_data(message: &'static str) -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::new(io::ErrorKind::InvalidData, message),
    )
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

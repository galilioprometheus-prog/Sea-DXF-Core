use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, DxfShapeNumericSemantics, DxfTextNumericSemantics,
    DxfTextShapeDoubleValue, DxfTextShapeScalarDirectory, DxfTextShapeScalarIssue,
    DxfTextSymbolKind, DxfTextSymbolRecordEntry, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum RecordSignature {
    Text {
        first: [ScalarSignature; 3],
        height: ScalarSignature,
        defaults: [ScalarSignature; 8],
        second: [ScalarSignature; 3],
    },
    Shape {
        insertion: [ScalarSignature; 3],
        size: ScalarSignature,
        defaults: [ScalarSignature; 8],
    },
}

#[derive(Debug, Eq, PartialEq)]
struct ScalarSignature {
    state: DxfSemanticValueState,
    bits: Option<u64>,
}

#[test]
fn every_dialect_has_ascii_binary_scalar_and_default_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.text_shape_scalar_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.text_shape_scalar_directory(&DxfCancellationToken::default())?;

        assert_defaults(&ascii_directory)?;
        assert_defaults(&binary_directory)?;
        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn missing_invalid_multiple_and_optional_states_keep_provenance() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n10\n1\n20\n2\n40\n.\n41\n2\n41\n3\n71\n.\n11\n9\n\
0\nSHAPE\n10\n4\n20\n5\n30\n6\n210\n2\n220\n3\n220\n4\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_shape_scalar_directory(&DxfCancellationToken::default())?;

    let text = text(&directory, 0)?;
    assert_invalid(
        &text.first_alignment()[2],
        DxfTextShapeScalarIssue::MissingRequiredValue,
        false,
    );
    assert!(matches!(
        text.text_height().invalid_issue(),
        Some(DxfTextShapeScalarIssue::InvalidAsciiNumber(_))
    ));
    assert!(text.text_height().raw_provenance().is_some());
    assert_invalid(
        text.width_factor(),
        DxfTextShapeScalarIssue::MultipleValues {
            occurrence_count: 2,
        },
        true,
    );
    assert!(matches!(
        text.generation_flags().invalid_issue(),
        Some(DxfTextShapeScalarIssue::InvalidAsciiNumber(_))
    ));
    assert_eq!(
        text.second_alignment()[0].state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        text.second_alignment()[1].state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(
        text.second_alignment()[2].state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(
        text.second_alignment()[1]
            .field_provenance()
            .schema_field_id(),
        "second_alignment_y"
    );

    let shape = shape(&directory, 1)?;
    assert_invalid(
        shape.size(),
        DxfTextShapeScalarIssue::MissingRequiredValue,
        false,
    );
    assert_invalid(
        &shape.extrusion()[1],
        DxfTextShapeScalarIssue::MultipleValues {
            occurrence_count: 2,
        },
        true,
    );
    assert_eq!(
        shape.extrusion()[0].value().map(|value| value.to_bits()),
        Some(2.0_f64.to_bits())
    );
    assert_eq!(
        shape.extrusion()[2].value().map(|value| value.to_bits()),
        Some(1.0_f64.to_bits())
    );
    assert_eq!(
        shape.extrusion()[2].state(),
        DxfSemanticValueState::Defaulted
    );
    Ok(())
}

#[test]
fn cancellation_family_scope_identity_lookups_and_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.text_shape_scalar_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.text_shape_scalar_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(directory.text_semantics_for_raw_record(u64::MAX)?, None);
    assert_eq!(directory.shape_semantics_for_raw_record(u64::MAX)?, None);

    let mtext_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n10\n1\n0\nTOLERANCE\n10\n2\n0\nENDSEC\n0\nEOF\n";
    let mtext_source = DxfMemorySource::new(mtext_bytes, DxfResourceProfile::Safe)?;
    let mtext_document = open_ascii(&mtext_source)?;
    let scoped = mtext_document.text_shape_scalar_directory(&DxfCancellationToken::default())?;
    for record in scoped.records() {
        assert_eq!(scoped.text_semantics_for_record(*record)?, None);
        assert_eq!(scoped.shape_semantics_for_record(*record)?, None);
    }

    assert_copy::<DxfTextNumericSemantics>();
    assert_copy::<DxfShapeNumericSemantics>();
    assert_send_sync::<DxfTextShapeScalarDirectory>();
    Ok(())
}

fn assert_defaults(directory: &DxfTextShapeScalarDirectory) -> Result<(), Box<dyn Error>> {
    let text = text(directory, 0)?;
    assert_eq!(
        bits3(text.first_alignment()),
        [1.0, 2.0, 3.0].map(f64::to_bits)
    );
    assert_eq!(
        text.text_height().value().map(|value| value.to_bits()),
        Some(4.0_f64.to_bits())
    );
    for value in [text.thickness(), text.rotation(), text.oblique_angle()] {
        assert_default(value, 0.0);
    }
    assert_default(text.width_factor(), 1.0);
    assert_eq!(text.generation_flags().value(), Some(&0));
    assert_eq!(text.horizontal_justification().value(), Some(&0));
    assert_eq!(text.vertical_justification().value(), Some(&0));
    assert!(
        text.second_alignment()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Absent)
    );
    assert_eq!(bits3(text.extrusion()), [0.0, 0.0, 1.0].map(f64::to_bits));
    assert!(
        text.extrusion()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Defaulted)
    );
    assert_eq!(
        text.record().kind(),
        seacad_dxf_core::DxfTextSymbolKind::Text
    );

    let shape = shape(directory, 1)?;
    assert_eq!(
        bits3(shape.insertion()),
        [10.0, 11.0, 12.0].map(f64::to_bits)
    );
    assert_eq!(
        shape.size().value().map(|value| value.to_bits()),
        Some(2.0_f64.to_bits())
    );
    for value in [shape.thickness(), shape.rotation(), shape.oblique_angle()] {
        assert_default(value, 0.0);
    }
    assert_default(shape.width_factor(), 1.0);
    assert_eq!(bits3(shape.extrusion()), [0.0, 0.0, 1.0].map(f64::to_bits));
    assert_eq!(
        shape.record().kind(),
        seacad_dxf_core::DxfTextSymbolKind::Shape
    );
    Ok(())
}

fn signatures(directory: &DxfTextShapeScalarDirectory) -> Result<Vec<RecordSignature>, DxfError> {
    let mut signatures = Vec::new();
    for record in directory.records().iter().copied() {
        match record.kind() {
            DxfTextSymbolKind::Text => {
                if let Some(value) = directory.text_semantics_for_record(record)? {
                    signatures.push(RecordSignature::Text {
                        first: value.first_alignment().each_ref().map(scalar),
                        height: scalar(value.text_height()),
                        defaults: [
                            scalar(value.thickness()),
                            scalar(value.rotation()),
                            scalar(value.width_factor()),
                            scalar(value.oblique_angle()),
                            integer(value.generation_flags()),
                            integer(value.horizontal_justification()),
                            integer(value.vertical_justification()),
                            scalar(&value.extrusion()[2]),
                        ],
                        second: value.second_alignment().each_ref().map(scalar),
                    });
                }
            }
            DxfTextSymbolKind::Shape => {
                if let Some(value) = directory.shape_semantics_for_record(record)? {
                    signatures.push(RecordSignature::Shape {
                        insertion: value.insertion().each_ref().map(scalar),
                        size: scalar(value.size()),
                        defaults: [
                            scalar(value.thickness()),
                            scalar(value.rotation()),
                            scalar(value.width_factor()),
                            scalar(value.oblique_angle()),
                            scalar(&value.extrusion()[0]),
                            scalar(&value.extrusion()[1]),
                            scalar(&value.extrusion()[2]),
                            scalar(value.size()),
                        ],
                    });
                }
            }
            DxfTextSymbolKind::MText | DxfTextSymbolKind::Tolerance => {}
            _ => {}
        }
    }
    Ok(signatures)
}

fn text(
    directory: &DxfTextShapeScalarDirectory,
    index: usize,
) -> Result<DxfTextNumericSemantics, Box<dyn Error>> {
    let record = record(directory, index)?;
    directory
        .text_semantics_for_record(record)?
        .ok_or_else(|| io::Error::other("TEXT semantics").into())
}

fn shape(
    directory: &DxfTextShapeScalarDirectory,
    index: usize,
) -> Result<DxfShapeNumericSemantics, Box<dyn Error>> {
    let record = record(directory, index)?;
    directory
        .shape_semantics_for_record(record)?
        .ok_or_else(|| io::Error::other("SHAPE semantics").into())
}

fn record(
    directory: &DxfTextShapeScalarDirectory,
    index: usize,
) -> Result<DxfTextSymbolRecordEntry, Box<dyn Error>> {
    directory
        .records()
        .get(index)
        .copied()
        .ok_or_else(|| io::Error::other("record").into())
}

fn scalar(value: &DxfTextShapeDoubleValue) -> ScalarSignature {
    ScalarSignature {
        state: value.state(),
        bits: value.value().map(|value| value.to_bits()),
    }
}

fn integer(value: &seacad_dxf_core::DxfTextShapeInt16Value) -> ScalarSignature {
    ScalarSignature {
        state: value.state(),
        bits: value.value().map(|value| *value as u16 as u64),
    }
}

fn assert_default(value: &DxfTextShapeDoubleValue, expected: f64) {
    assert_eq!(value.state(), DxfSemanticValueState::Defaulted);
    assert_eq!(
        value.value().map(|value| value.to_bits()),
        Some(expected.to_bits())
    );
    assert_eq!(value.raw_provenance(), None);
}

fn assert_invalid(value: &DxfTextShapeDoubleValue, issue: DxfTextShapeScalarIssue, has_raw: bool) {
    assert_eq!(value.state(), DxfSemanticValueState::Invalid);
    assert_eq!(value.invalid_issue(), Some(&issue));
    assert_eq!(value.raw_provenance().is_some(), has_raw);
}

fn bits3(values: &[DxfTextShapeDoubleValue; 3]) -> [u64; 3] {
    values.map(|value| {
        value
            .value()
            .map(|number| number.to_bits())
            .unwrap_or(u64::MAX)
    })
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n40\n4\n1\nA\n\
0\nSHAPE\n10\n10\n20\n11\n30\n12\n40\n2\n2\nBOLT\n\
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
    push_string(&mut bytes, version, 0, b"TEXT")?;
    push_vector(&mut bytes, version, [10, 20, 30], [1.0, 2.0, 3.0])?;
    push_double(&mut bytes, version, 40, 4.0)?;
    push_string(&mut bytes, version, 1, b"A")?;
    push_string(&mut bytes, version, 0, b"SHAPE")?;
    push_vector(&mut bytes, version, [10, 20, 30], [10.0, 11.0, 12.0])?;
    push_double(&mut bytes, version, 40, 2.0)?;
    push_string(&mut bytes, version, 2, b"BOLT")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_vector(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    codes: [i16; 3],
    values: [f64; 3],
) -> io::Result<()> {
    for (code, value) in codes.into_iter().zip(values) {
        push_double(bytes, version, code, value)?;
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

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("code"))?);
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

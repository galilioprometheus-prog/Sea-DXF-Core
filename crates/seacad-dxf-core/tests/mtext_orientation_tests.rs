use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextOrientationDirectory, DxfMTextOrientationInput,
    DxfMTextOrientationIssue, DxfMTextOrientationSemantics, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, DxfTextSymbolScalarIssue, NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum OrientationRow {
    RotationOnly,
    AxisOnly,
    RotationThenAxis,
    AxisThenRotation,
    Neither,
}

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    rotation_state: DxfSemanticValueState,
    rotation_bits: Option<u64>,
    input_state: DxfSemanticValueState,
    input: Option<DxfMTextOrientationInput>,
}

#[test]
fn every_dialect_has_ascii_binary_orientation_precedence_parity() -> Result<(), Box<dyn Error>> {
    let rows = [
        OrientationRow::RotationOnly,
        OrientationRow::AxisOnly,
        OrientationRow::RotationThenAxis,
        OrientationRow::AxisThenRotation,
        OrientationRow::Neither,
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.mtext_orientation_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.mtext_orientation_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
        assert_eq!(
            signatures(&ascii_directory)?,
            [
                signature(
                    DxfSemanticValueState::Explicit,
                    Some(0.5_f64.to_bits()),
                    DxfSemanticValueState::Explicit,
                    Some(DxfMTextOrientationInput::Rotation),
                ),
                signature(
                    DxfSemanticValueState::Absent,
                    None,
                    DxfSemanticValueState::Explicit,
                    Some(DxfMTextOrientationInput::XAxis),
                ),
                signature(
                    DxfSemanticValueState::Explicit,
                    Some(0.5_f64.to_bits()),
                    DxfSemanticValueState::Explicit,
                    Some(DxfMTextOrientationInput::XAxis),
                ),
                signature(
                    DxfSemanticValueState::Explicit,
                    Some(0.5_f64.to_bits()),
                    DxfSemanticValueState::Explicit,
                    Some(DxfMTextOrientationInput::Rotation),
                ),
                signature(
                    DxfSemanticValueState::Absent,
                    None,
                    DxfSemanticValueState::Absent,
                    None,
                ),
            ]
        );
    }
    Ok(())
}

#[test]
fn column_fields_keep_every_coexisting_group_50_role_ambiguous() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n50\n0.5\n75\n1\n\
0\nMTEXT\n11\n1\n21\n0\n31\n0\n75\n1\n50\n0.5\n\
0\nMTEXT\n50\n0.5\n50\n1.5\n76\n2\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_orientation_directory(&DxfCancellationToken::default())?;

    for (index, count) in [(0, 1), (1, 1), (2, 2)] {
        let value = semantics(&directory, index)?;
        assert_eq!(
            value.rotation().invalid_issue(),
            Some(&DxfMTextOrientationIssue::AmbiguousGroup50Role {
                occurrence_count: count,
            })
        );
        assert_eq!(
            value.effective_input().invalid_issue(),
            Some(&DxfMTextOrientationIssue::AmbiguousGroup50Role {
                occurrence_count: count,
            })
        );
        assert!(value.rotation().raw_provenance().is_some());
        assert!(value.effective_input().raw_provenance().is_some());
    }
    Ok(())
}

#[test]
fn invalid_and_duplicate_rotation_evidence_obeys_source_precedence_without_guessing()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n50\nbad\n11\n1\n21\n0\n31\n0\n\
0\nMTEXT\n11\n1\n21\n0\n31\n0\n50\nbad\n\
0\nMTEXT\n50\n0.5\n50\n1.5\n\
0\nMTEXT\n50\n0.5\n50\n1.5\n11\n1\n21\n0\n31\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_orientation_directory(&DxfCancellationToken::default())?;

    let invalid_before_axis = semantics(&directory, 0)?;
    assert!(matches!(
        invalid_before_axis.rotation().invalid_issue(),
        Some(DxfMTextOrientationIssue::Scalar(
            DxfTextSymbolScalarIssue::InvalidAsciiNumber(_)
        ))
    ));
    assert_eq!(
        invalid_before_axis.effective_input().value(),
        Some(&DxfMTextOrientationInput::XAxis)
    );

    let invalid_after_axis = semantics(&directory, 1)?;
    assert!(matches!(
        invalid_after_axis.effective_input().invalid_issue(),
        Some(DxfMTextOrientationIssue::Scalar(
            DxfTextSymbolScalarIssue::InvalidAsciiNumber(_)
        ))
    ));
    assert!(
        invalid_after_axis
            .effective_input()
            .raw_provenance()
            .is_some()
    );

    let duplicate = semantics(&directory, 2)?;
    assert_multiple(duplicate.rotation().invalid_issue());
    assert_multiple(duplicate.effective_input().invalid_issue());

    let duplicate_before_axis = semantics(&directory, 3)?;
    assert_multiple(duplicate_before_axis.rotation().invalid_issue());
    assert_eq!(
        duplicate_before_axis.effective_input().value(),
        Some(&DxfMTextOrientationInput::XAxis)
    );
    Ok(())
}

#[test]
fn cancellation_scope_identity_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextOrientationSemantics>();
    assert_copy::<DxfMTextOrientationInput>();
    assert_send_sync::<DxfMTextOrientationDirectory>();

    let bytes = ascii_fixture("AC1032", &[OrientationRow::RotationOnly]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_orientation_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n50\n0.5\n0\nTEXT\n40\n1\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_orientation_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.scalar_directory().source_id()
    );
    assert!(
        directory
            .semantics_for_record(directory.records()[0])?
            .is_some()
    );
    assert_eq!(
        directory.semantics_for_record(directory.records()[1])?,
        None
    );
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    Ok(())
}

fn signature(
    rotation_state: DxfSemanticValueState,
    rotation_bits: Option<u64>,
    input_state: DxfSemanticValueState,
    input: Option<DxfMTextOrientationInput>,
) -> Signature {
    Signature {
        rotation_state,
        rotation_bits,
        input_state,
        input,
    }
}

fn signatures(directory: &DxfMTextOrientationDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(value)) => Some(Ok(Signature {
                rotation_state: value.rotation().state(),
                rotation_bits: value.rotation().value().map(|number| number.to_bits()),
                input_state: value.effective_input().state(),
                input: value.effective_input().value().copied(),
            })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfMTextOrientationDirectory,
    index: usize,
) -> Result<DxfMTextOrientationSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or_else(invalid_test_data)?;
    directory
        .semantics_for_record(record)?
        .ok_or_else(|| invalid_test_data().into())
}

fn assert_multiple(issue: Option<&DxfMTextOrientationIssue>) {
    assert!(matches!(
        issue,
        Some(DxfMTextOrientationIssue::Scalar(
            DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2
            }
        ))
    ));
}

fn ascii_fixture(version: &str, rows: &[OrientationRow]) -> Vec<u8> {
    let mut entities = String::new();
    for row in rows {
        entities.push_str("0\nMTEXT\n");
        match row {
            OrientationRow::RotationOnly => entities.push_str("50\n0.5\n"),
            OrientationRow::AxisOnly => entities.push_str("11\n1\n21\n0\n31\n0\n"),
            OrientationRow::RotationThenAxis => {
                entities.push_str("50\n0.5\n11\n1\n21\n0\n31\n0\n");
            }
            OrientationRow::AxisThenRotation => {
                entities.push_str("11\n1\n21\n0\n31\n0\n50\n0.5\n");
            }
            OrientationRow::Neither => {}
        }
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, rows: &[OrientationRow]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    for row in rows {
        push_string(&mut bytes, version, 0, b"MTEXT")?;
        match row {
            OrientationRow::RotationOnly => push_double(&mut bytes, version, 50, 0.5)?,
            OrientationRow::AxisOnly => push_axis(&mut bytes, version)?,
            OrientationRow::RotationThenAxis => {
                push_double(&mut bytes, version, 50, 0.5)?;
                push_axis(&mut bytes, version)?;
            }
            OrientationRow::AxisThenRotation => {
                push_axis(&mut bytes, version)?;
                push_double(&mut bytes, version, 50, 0.5)?;
            }
            OrientationRow::Neither => {}
        }
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_axis(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    for (code, value) in [(11, 1.0), (21, 0.0), (31, 0.0)] {
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
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| invalid_test_data())?);
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

fn invalid_test_data() -> io::Error {
    io::Error::other("invalid test data")
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

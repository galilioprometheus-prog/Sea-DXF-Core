use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextOrientationInput, DxfMTextOrientationIssue,
    DxfMTextXAxisComponent, DxfMTextXAxisDirectionDirectory, DxfMTextXAxisDirectionIssue,
    DxfMTextXAxisDirectionSemantics, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValue, DxfSemanticValueState, DxfTextSymbolScalarIssue, NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum Row {
    RotationOnly,
    AxisOnly,
    RotationThenAxis,
    AxisThenRotation,
    Neither,
}

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    state: DxfSemanticValueState,
    input: Option<DxfMTextOrientationInput>,
    vector_bits: Option<[u64; 3]>,
    unit_bits: Option<[u64; 3]>,
}

#[test]
fn every_dialect_has_ascii_binary_effective_direction_parity() -> Result<(), Box<dyn Error>> {
    let rows = [
        Row::RotationOnly,
        Row::AxisOnly,
        Row::RotationThenAxis,
        Row::AxisThenRotation,
        Row::Neither,
    ];
    let angle = 0.5_f64;
    let rotation = [
        angle.cos().to_bits(),
        angle.sin().to_bits(),
        0.0_f64.to_bits(),
    ];
    let expected = [
        explicit(DxfMTextOrientationInput::Rotation, rotation, rotation),
        explicit(
            DxfMTextOrientationInput::XAxis,
            bits([3.0, 4.0, 0.0]),
            bits([0.6, 0.8, 0.0]),
        ),
        explicit(
            DxfMTextOrientationInput::XAxis,
            bits([3.0, 4.0, 0.0]),
            bits([0.6, 0.8, 0.0]),
        ),
        explicit(DxfMTextOrientationInput::Rotation, rotation, rotation),
        absent(),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        let ascii_directory = ascii.mtext_x_axis_direction_directory(&cancellation)?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.mtext_x_axis_direction_directory(&cancellation)?;

        assert_eq!(signatures(&ascii_directory)?, expected);
        assert_eq!(signatures(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn partial_invalid_zero_huge_and_ambiguous_inputs_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n11\n1\n\
0\nMTEXT\n11\nbad\n21\n0\n31\n0\n\
0\nMTEXT\n11\n0\n21\n0\n31\n0\n\
0\nMTEXT\n11\n1.7e308\n21\n1.7e308\n31\n1.7e308\n\
0\nMTEXT\n50\n0.5\n75\n1\n\
0\nMTEXT\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_x_axis_direction_directory(&DxfCancellationToken::default())?;

    assert_issue(
        semantics(&directory, 0)?,
        DxfMTextXAxisDirectionIssue::ComponentAbsent {
            component: DxfMTextXAxisComponent::Y,
        },
    );
    assert!(matches!(
        semantics(&directory, 1)?.direction().invalid_issue(),
        Some(DxfMTextXAxisDirectionIssue::ComponentInvalid {
            component: DxfMTextXAxisComponent::X,
            issue: DxfTextSymbolScalarIssue::InvalidAsciiNumber(_),
        })
    ));
    assert_issue(
        semantics(&directory, 2)?,
        DxfMTextXAxisDirectionIssue::ZeroLength,
    );
    assert_issue(
        semantics(&directory, 3)?,
        DxfMTextXAxisDirectionIssue::NonFiniteLength,
    );
    assert_issue(
        semantics(&directory, 4)?,
        DxfMTextXAxisDirectionIssue::Orientation(DxfMTextOrientationIssue::AmbiguousGroup50Role {
            occurrence_count: 1,
        }),
    );
    assert!(matches!(
        semantics(&directory, 5)?.direction(),
        DxfSemanticValue::Absent { .. }
    ));
    for index in 0..5 {
        assert!(
            semantics(&directory, index)?
                .direction()
                .raw_provenance()
                .is_some()
        );
    }
    Ok(())
}

#[test]
fn cancellation_scope_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextXAxisDirectionSemantics>();
    assert_send_sync::<DxfMTextXAxisDirectionDirectory>();

    let bytes = ascii_fixture("AC1032", &[Row::AxisOnly]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_x_axis_direction_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n11\n1\n21\n0\n31\n0\n\
0\nTEXT\n11\n1\n21\n0\n31\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_x_axis_direction_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.orientation_directory().source_id()
    );
    assert_eq!(directory.records().len(), 2);
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

fn explicit(
    input: DxfMTextOrientationInput,
    vector_bits: [u64; 3],
    unit_bits: [u64; 3],
) -> Signature {
    Signature {
        state: DxfSemanticValueState::Explicit,
        input: Some(input),
        vector_bits: Some(vector_bits),
        unit_bits: Some(unit_bits),
    }
}

fn absent() -> Signature {
    Signature {
        state: DxfSemanticValueState::Absent,
        input: None,
        vector_bits: None,
        unit_bits: None,
    }
}

fn bits(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
}

fn signatures(directory: &DxfMTextXAxisDirectionDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(semantics)) => {
                let direction = semantics.direction();
                Some(Ok(Signature {
                    state: direction.state(),
                    input: direction.value().map(|value| value.input()),
                    vector_bits: direction
                        .value()
                        .map(|value| value.vector().map(|number| number.to_bits())),
                    unit_bits: direction
                        .value()
                        .map(|value| value.unit_vector().map(|number| number.to_bits())),
                }))
            }
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfMTextXAxisDirectionDirectory,
    index: usize,
) -> Result<DxfMTextXAxisDirectionSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or_else(invalid_test_data)?;
    directory
        .semantics_for_record(record)?
        .ok_or_else(|| invalid_test_data().into())
}

fn assert_issue(semantics: DxfMTextXAxisDirectionSemantics, expected: DxfMTextXAxisDirectionIssue) {
    assert_eq!(semantics.direction().invalid_issue(), Some(&expected));
}

fn ascii_fixture(version: &str, rows: &[Row]) -> Vec<u8> {
    let mut entities = String::new();
    for row in rows {
        entities.push_str("0\nMTEXT\n");
        match row {
            Row::RotationOnly => entities.push_str("50\n0.5\n"),
            Row::AxisOnly => entities.push_str("11\n3\n21\n4\n31\n0\n"),
            Row::RotationThenAxis => {
                entities.push_str("50\n0.5\n11\n3\n21\n4\n31\n0\n");
            }
            Row::AxisThenRotation => {
                entities.push_str("11\n3\n21\n4\n31\n0\n50\n0.5\n");
            }
            Row::Neither => {}
        }
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, rows: &[Row]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for row in rows {
        push_string(&mut bytes, version, 0, b"MTEXT")?;
        match row {
            Row::RotationOnly => push_double(&mut bytes, version, 50, 0.5)?,
            Row::AxisOnly => push_axis(&mut bytes, version)?,
            Row::RotationThenAxis => {
                push_double(&mut bytes, version, 50, 0.5)?;
                push_axis(&mut bytes, version)?;
            }
            Row::AxisThenRotation => {
                push_axis(&mut bytes, version)?;
                push_double(&mut bytes, version, 50, 0.5)?;
            }
            Row::Neither => {}
        }
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_axis(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    for (code, value) in [(11, 3.0), (21, 4.0), (31, 0.0)] {
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

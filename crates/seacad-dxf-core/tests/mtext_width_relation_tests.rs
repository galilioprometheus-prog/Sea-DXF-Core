use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfMTextActualWidthRelation,
    DxfMTextNumericDomainDirectory, DxfMTextNumericDomainIssue, DxfMTextNumericDomainSemantics,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    DxfTextSymbolScalarIssue, NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
struct WidthRow {
    reference: f64,
    actual: Option<f64>,
}

#[test]
fn every_dialect_has_ascii_binary_actual_width_relation_parity() -> Result<(), Box<dyn Error>> {
    let rows = [
        WidthRow {
            reference: 5.0,
            actual: None,
        },
        WidthRow {
            reference: 5.0,
            actual: Some(4.0),
        },
        WidthRow {
            reference: 5.0,
            actual: Some(5.0),
        },
        WidthRow {
            reference: 5.0,
            actual: Some(6.0),
        },
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
        assert_eq!(
            signatures(&ascii_directory)?,
            [
                (DxfSemanticValueState::Absent, false),
                (DxfSemanticValueState::Explicit, true),
                (DxfSemanticValueState::Explicit, true),
                (DxfSemanticValueState::Invalid, false),
            ]
        );
    }
    Ok(())
}

#[test]
fn equality_is_accepted_and_adjacent_excess_preserves_both_exact_values()
-> Result<(), Box<dyn Error>> {
    let reference = 5.0_f64;
    let actual = f64::from_bits(reference.to_bits() + 1);
    let rows = [
        WidthRow {
            reference,
            actual: Some(reference),
        },
        WidthRow {
            reference,
            actual: Some(actual),
        },
    ];
    let bytes = ascii_fixture("AC1032", &rows);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;

    let equal = semantics(&directory, 0)?;
    assert_eq!(
        equal.actual_width_relation().value(),
        Some(&DxfMTextActualWidthRelation::WithinReference)
    );

    let excess = semantics(&directory, 1)?;
    assert_eq!(
        excess.actual_width_relation().invalid_issue(),
        Some(&DxfMTextNumericDomainIssue::ActualWidthExceedsReference {
            actual_width: DxfDouble::from_f64(actual),
            reference_width: DxfDouble::from_f64(reference),
        })
    );
    assert!(excess.actual_width_relation().raw_provenance().is_some());
    Ok(())
}

#[test]
fn invalid_and_duplicate_width_inputs_preserve_the_originating_scalar_evidence()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n41\n5\n42\nbad\n\
0\nMTEXT\n41\n5\n42\n4\n42\n5\n\
0\nMTEXT\n41\nbad\n42\n4\n\
0\nMTEXT\n41\n5\n41\n6\n42\n4\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;

    let invalid_actual = semantics(&directory, 0)?;
    assert!(matches!(
        invalid_actual.actual_width_relation().invalid_issue(),
        Some(DxfMTextNumericDomainIssue::Scalar(
            DxfTextSymbolScalarIssue::InvalidAsciiNumber(_)
        ))
    ));

    let multiple_actual = semantics(&directory, 1)?;
    assert_multiple(multiple_actual.actual_width_relation().invalid_issue());

    let invalid_reference = semantics(&directory, 2)?;
    assert!(matches!(
        invalid_reference.actual_width_relation().invalid_issue(),
        Some(DxfMTextNumericDomainIssue::Scalar(
            DxfTextSymbolScalarIssue::InvalidAsciiNumber(_)
        ))
    ));

    let multiple_reference = semantics(&directory, 3)?;
    assert_multiple(multiple_reference.actual_width_relation().invalid_issue());

    for index in 0..4 {
        assert!(
            semantics(&directory, index)?
                .actual_width_relation()
                .raw_provenance()
                .is_some()
        );
    }
    Ok(())
}

#[test]
fn cancellation_scope_identity_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextNumericDomainSemantics>();
    assert_copy::<DxfMTextActualWidthRelation>();
    assert_send_sync::<DxfMTextNumericDomainDirectory>();

    let bytes = ascii_fixture(
        "AC1032",
        &[WidthRow {
            reference: 5.0,
            actual: Some(4.0),
        }],
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_numeric_domain_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n41\n5\n42\n4\n0\nTEXT\n40\n1\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;
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

fn signatures(
    directory: &DxfMTextNumericDomainDirectory,
) -> Result<Vec<(DxfSemanticValueState, bool)>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(semantics)) => Some(Ok((
                semantics.actual_width_relation().state(),
                semantics.actual_width_relation().value()
                    == Some(&DxfMTextActualWidthRelation::WithinReference),
            ))),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfMTextNumericDomainDirectory,
    index: usize,
) -> Result<DxfMTextNumericDomainSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or_else(invalid_test_data)?;
    directory
        .semantics_for_record(record)?
        .ok_or_else(|| invalid_test_data().into())
}

fn assert_multiple(issue: Option<&DxfMTextNumericDomainIssue>) {
    assert!(matches!(
        issue,
        Some(DxfMTextNumericDomainIssue::Scalar(
            DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2
            }
        ))
    ));
}

fn ascii_fixture(version: &str, rows: &[WidthRow]) -> Vec<u8> {
    let mut entities = String::new();
    for row in rows {
        entities.push_str(&format!("0\nMTEXT\n41\n{}\n", row.reference));
        if let Some(actual) = row.actual {
            entities.push_str(&format!("42\n{actual}\n"));
        }
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, rows: &[WidthRow]) -> Result<Vec<u8>, io::Error> {
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
        push_double(&mut bytes, version, 41, row.reference)?;
        if let Some(actual) = row.actual {
            push_double(&mut bytes, version, 42, actual)?;
        }
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
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

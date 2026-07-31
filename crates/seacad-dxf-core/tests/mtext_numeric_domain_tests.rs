use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfMTextNumericDomainDirectory,
    DxfMTextNumericDomainIssue, DxfMTextNumericDomainSemantics, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, DxfTextSymbolScalarIssue, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_line_spacing_factor_parity() -> Result<(), Box<dyn Error>> {
    let values = [None, Some(0.25), Some(1.0), Some(4.0)];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &values);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, &values)?;
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
                (DxfSemanticValueState::Absent, None),
                (DxfSemanticValueState::Explicit, Some(0.25_f64.to_bits())),
                (DxfSemanticValueState::Explicit, Some(1.0_f64.to_bits())),
                (DxfSemanticValueState::Explicit, Some(4.0_f64.to_bits())),
            ]
        );
    }
    Ok(())
}

#[test]
fn exact_boundaries_are_accepted_and_neighbouring_values_are_rejected() -> Result<(), Box<dyn Error>>
{
    let below = f64::from_bits(0.25_f64.to_bits() - 1);
    let above = f64::from_bits(4.0_f64.to_bits() + 1);
    let values = [Some(0.25), Some(4.0), Some(below), Some(above), Some(0.0)];
    let bytes = ascii_fixture("AC1032", &values);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;

    for (index, expected) in [(0, 0.25_f64), (1, 4.0_f64)] {
        let semantics = semantics(&directory, index)?;
        let factor = semantics
            .line_spacing_factor()
            .value()
            .copied()
            .ok_or_else(invalid_test_data)?;
        assert_eq!(factor.to_f64().to_bits(), expected.to_bits());
        assert_eq!(factor.raw(), DxfDouble::from_f64(expected));
    }

    for (index, expected) in [(2, below), (3, above), (4, 0.0)] {
        let semantics = semantics(&directory, index)?;
        assert_eq!(
            semantics.line_spacing_factor().invalid_issue(),
            Some(&DxfMTextNumericDomainIssue::LineSpacingFactorOutOfRange {
                value: DxfDouble::from_f64(expected),
            })
        );
        assert!(semantics.line_spacing_factor().raw_provenance().is_some());
    }
    Ok(())
}

#[test]
fn invalid_duplicate_and_absent_values_preserve_typed_source_evidence() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n44\nbad\n\
0\nMTEXT\n44\n1\n44\n2\n\
0\nMTEXT\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_numeric_domain_directory(&DxfCancellationToken::default())?;

    let invalid = semantics(&directory, 0)?;
    assert!(matches!(
        invalid.line_spacing_factor().invalid_issue(),
        Some(DxfMTextNumericDomainIssue::Scalar(
            DxfTextSymbolScalarIssue::InvalidAsciiNumber(_)
        ))
    ));
    assert!(invalid.line_spacing_factor().raw_provenance().is_some());

    let duplicate = semantics(&directory, 1)?;
    assert_eq!(
        duplicate.line_spacing_factor().invalid_issue(),
        Some(&DxfMTextNumericDomainIssue::Scalar(
            DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2,
            }
        ))
    );
    assert!(duplicate.line_spacing_factor().raw_provenance().is_some());

    let absent = semantics(&directory, 2)?;
    assert_eq!(
        absent.line_spacing_factor().state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(absent.line_spacing_factor().raw_provenance(), None);
    Ok(())
}

#[test]
fn cancellation_scope_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextNumericDomainSemantics>();
    assert_send_sync::<DxfMTextNumericDomainDirectory>();

    let bytes = ascii_fixture("AC1032", &[Some(1.0)]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_numeric_domain_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n44\n1\n0\nTEXT\n40\n1\n0\nENDSEC\n0\nEOF\n";
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
) -> Result<Vec<(DxfSemanticValueState, Option<u64>)>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(semantics)) => Some(Ok((
                semantics.line_spacing_factor().state(),
                semantics
                    .line_spacing_factor()
                    .value()
                    .map(|factor| factor.raw().to_bits()),
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

fn ascii_fixture(version: &str, values: &[Option<f64>]) -> Vec<u8> {
    let mut entities = String::new();
    for value in values {
        entities.push_str("0\nMTEXT\n");
        if let Some(value) = value {
            entities.push_str(&format!("44\n{value}\n"));
        }
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, values: &[Option<f64>]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    for value in values {
        push_string(&mut bytes, version, 0, b"MTEXT")?;
        if let Some(value) = value {
            push_double(&mut bytes, version, 44, *value)?;
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

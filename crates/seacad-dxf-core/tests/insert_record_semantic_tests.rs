use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertRecordSemanticDirectory, DxfInsertRecordSemanticIssue, DxfInsertRecordSemantics,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_explicit_and_defaulted_semantic_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_record_semantic_directory(&DxfCancellationToken::default())?;
        assert_directory(&ascii_directory)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_record_semantic_directory(&DxfCancellationToken::default())?;
        assert_directory(&binary_directory)?;
    }
    Ok(())
}

#[test]
fn missing_required_values_do_not_hide_documented_defaults() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_record_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = first_semantics(&directory)?;

    assert_missing_required(semantics.block_name());
    for component in semantics.insertion_point() {
        assert_missing_required(component);
    }
    assert_eq!(
        states(semantics.scale_factors()),
        [DxfSemanticValueState::Defaulted; 3]
    );
    assert_eq!(
        semantics.rotation_degrees().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        states(semantics.array_counts()),
        [DxfSemanticValueState::Defaulted; 2]
    );
    assert_eq!(
        states(semantics.array_spacing()),
        [DxfSemanticValueState::Defaulted; 2]
    );
    assert_eq!(
        semantics.attributes_follow().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        states(semantics.extrusion()),
        [DxfSemanticValueState::Defaulted; 3]
    );
    assert_defaults(&semantics)?;
    Ok(())
}

#[test]
fn invalid_and_multiple_values_preserve_first_raw_provenance() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n2\nB\n10\nbad\n20\n1\n20\n2\n30\nbad\n41\nbad\n70\nbad\n71\n2\n71\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_record_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = first_semantics(&directory)?;

    assert_multiple(semantics.block_name(), 2);
    assert_invalid_number(&semantics.insertion_point()[0]);
    assert_multiple(&semantics.insertion_point()[1], 2);
    assert_invalid_number(&semantics.insertion_point()[2]);
    assert_invalid_number(&semantics.scale_factors()[0]);
    assert_invalid_number(&semantics.array_counts()[0]);
    assert_multiple(&semantics.array_counts()[1], 2);
    for semantic in [
        semantics.block_name().raw_provenance(),
        semantics.insertion_point()[0].raw_provenance(),
        semantics.insertion_point()[1].raw_provenance(),
        semantics.array_counts()[1].raw_provenance(),
    ] {
        assert!(semantic.is_some());
    }
    assert_eq!(
        states(&semantics.scale_factors()[1..]),
        [DxfSemanticValueState::Defaulted; 2]
    );
    Ok(())
}

#[test]
fn cancellation_lookups_provenance_and_public_traits_are_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_record_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.insert_record_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert!(directory.semantics_for_raw_record(u64::MAX)?.is_none());
    for record in directory.records() {
        let semantics = directory
            .semantics_for_entry(*record)?
            .ok_or_else(invalid_test_data)?;
        assert_eq!(semantics.record(), *record);
        for field in [
            semantics.block_name().field_provenance(),
            semantics.insertion_point()[0].field_provenance(),
            semantics.scale_factors()[0].field_provenance(),
            semantics.attributes_follow().field_provenance(),
        ] {
            assert_eq!(field.document_source_id(), directory.source_id());
            assert_eq!(field.schema_namespace(), "insert.record");
        }
    }
    assert_copy::<DxfInsertRecordSemantics>();
    assert_send_sync::<DxfInsertRecordSemanticDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfInsertRecordSemanticDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    let full = directory
        .semantics_for_entry(directory.records()[0])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(full.block_name().state(), DxfSemanticValueState::Explicit);
    assert_eq!(
        states(full.insertion_point()),
        [DxfSemanticValueState::Explicit; 3]
    );
    assert_eq!(
        states(full.scale_factors()),
        [DxfSemanticValueState::Explicit; 3]
    );
    assert_eq!(
        full.rotation_degrees().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        states(full.array_counts()),
        [DxfSemanticValueState::Explicit; 2]
    );
    assert_eq!(
        states(full.array_spacing()),
        [DxfSemanticValueState::Explicit; 2]
    );
    assert_eq!(
        full.attributes_follow().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        states(full.extrusion()),
        [DxfSemanticValueState::Explicit; 3]
    );
    assert_double_bits(full.insertion_point_value(), [1.0, 2.0, 3.0])?;
    assert_double_bits(full.scale_factor_values(), [4.0, 5.0, 6.0])?;
    assert_eq!(full.rotation_degrees_value(), Some(double(7.0)));
    assert_eq!(full.array_count_values(), Some([8, 9]));
    assert_double_bits(full.array_spacing_values(), [10.0, 11.0])?;
    assert_eq!(full.attributes_follow_value(), Some(1));
    assert_double_bits(full.extrusion_value(), [12.0, 13.0, 14.0])?;

    let defaulted = directory
        .semantics_for_entry(directory.records()[1])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        states(defaulted.insertion_point()),
        [DxfSemanticValueState::Explicit; 3]
    );
    assert_defaults(&defaulted)?;
    Ok(())
}

fn assert_defaults(semantics: &DxfInsertRecordSemantics) -> Result<(), Box<dyn Error>> {
    assert_double_bits(semantics.scale_factor_values(), [1.0, 1.0, 1.0])?;
    assert_eq!(semantics.rotation_degrees_value(), Some(double(0.0)));
    assert_eq!(semantics.array_count_values(), Some([1, 1]));
    assert_double_bits(semantics.array_spacing_values(), [0.0, 0.0])?;
    assert_eq!(semantics.attributes_follow_value(), Some(0));
    assert_double_bits(semantics.extrusion_value(), [0.0, 0.0, 1.0])
}

fn assert_missing_required<T>(
    semantic: &seacad_dxf_core::DxfSemanticValue<T, DxfInsertRecordSemanticIssue>,
) {
    assert_eq!(semantic.state(), DxfSemanticValueState::Invalid);
    assert_eq!(
        semantic.invalid_issue(),
        Some(&DxfInsertRecordSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(semantic.raw_provenance(), None);
}

fn assert_invalid_number<T>(
    semantic: &seacad_dxf_core::DxfSemanticValue<T, DxfInsertRecordSemanticIssue>,
) {
    assert!(matches!(
        semantic.invalid_issue(),
        Some(DxfInsertRecordSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
}

fn assert_multiple<T>(
    semantic: &seacad_dxf_core::DxfSemanticValue<T, DxfInsertRecordSemanticIssue>,
    occurrence_count: u32,
) {
    assert_eq!(
        semantic.invalid_issue(),
        Some(&DxfInsertRecordSemanticIssue::MultipleValues { occurrence_count })
    );
}

fn states<T, I>(values: &[seacad_dxf_core::DxfSemanticValue<T, I>]) -> Vec<DxfSemanticValueState> {
    values.iter().map(|value| value.state()).collect()
}

fn assert_double_bits<const N: usize>(
    actual: Option<[DxfDouble; N]>,
    expected: [f64; N],
) -> Result<(), Box<dyn Error>> {
    let actual = actual.ok_or_else(invalid_test_data)?;
    for (value, expected) in actual.into_iter().zip(expected) {
        assert_eq!(value.to_bits(), expected.to_bits());
    }
    Ok(())
}

fn double(value: f64) -> DxfDouble {
    DxfDouble::from_bits(value.to_bits())
}

fn first_semantics(
    directory: &DxfInsertRecordSemanticDirectory,
) -> Result<DxfInsertRecordSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .first()
        .copied()
        .ok_or_else(invalid_test_data)?;
    directory
        .semantics_for_entry(record)?
        .ok_or_else(|| invalid_test_data().into())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nRef\n10\n1\n20\n2\n30\n3\n41\n4\n42\n5\n43\n6\n50\n7\n70\n8\n71\n9\n44\n10\n45\n11\n66\n1\n210\n12\n220\n13\n230\n14\n0\nINSERT\n2\nRef\n10\n-1\n20\n-2\n30\n-3\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"Ref")?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (41, 4.0),
        (42, 5.0),
        (43, 6.0),
        (50, 7.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 70, 8)?;
    push_i16(&mut bytes, version, 71, 9)?;
    push_double(&mut bytes, version, 44, 10.0)?;
    push_double(&mut bytes, version, 45, 11.0)?;
    push_i16(&mut bytes, version, 66, 1)?;
    for (code, value) in [(210, 12.0), (220, 13.0), (230, 14.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"Ref")?;
    for (code, value) in [(10, -1.0), (20, -2.0), (30, -3.0)] {
        push_double(&mut bytes, version, code, value)?;
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
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

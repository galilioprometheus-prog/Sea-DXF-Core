use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfInsertAttributeIntegerSemanticDirectory, DxfInsertAttributeIntegerSemanticIssue,
    DxfInsertAttributeIntegerSemantics, DxfInsertAttributeValueCardState,
    DxfInsertAttributeValueRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

type IntegerSignature = (DxfSemanticValueState, Option<i16>);
type RecordSignature = [IntegerSignature; 5];

#[test]
fn every_supported_dialect_has_ascii_binary_integer_semantic_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_integer_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_integer_semantic_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn required_attribute_flags_and_documented_zero_defaults_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_integer_semantic_directory(&DxfCancellationToken::default())?;
    let empty = directory
        .semantics_for_entry(directory.records()[1])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        empty.attribute_flags().state(),
        DxfSemanticValueState::Invalid
    );
    assert_eq!(
        empty.attribute_flags().invalid_issue(),
        Some(&DxfInsertAttributeIntegerSemanticIssue::MissingRequiredValue)
    );
    for defaulted in [
        empty.field_length(),
        empty.text_generation_flags(),
        empty.horizontal_justification(),
        empty.vertical_justification(),
    ] {
        assert_eq!(defaulted.state(), DxfSemanticValueState::Defaulted);
        assert_eq!(defaulted.value(), Some(&0));
        assert_eq!(defaulted.raw_provenance(), None);
    }
    assert_eq!(empty.is_invisible(), None);
    assert_eq!(empty.is_backward(), Some(false));
    assert_eq!(empty.is_upside_down(), Some(false));
    Ok(())
}

#[test]
fn invalid_duplicates_and_neutral_group_280_remain_explicit_without_fallback()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\nbad\n73\n1\n73\n2\n71\nbad\n280\n0\n280\n1\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_integer_semantic_directory(&DxfCancellationToken::default())?;
    let record = directory.records()[0];
    let semantics = directory
        .semantics_for_raw_record(record.record().ordinal())?
        .ok_or_else(invalid_test_data)?;
    for invalid in [
        semantics.attribute_flags(),
        semantics.text_generation_flags(),
    ] {
        assert!(matches!(
            invalid.invalid_issue(),
            Some(DxfInsertAttributeIntegerSemanticIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { .. }
            ))
        ));
        assert!(invalid.raw_provenance().is_some());
        assert_ne!(invalid.state(), DxfSemanticValueState::Defaulted);
    }
    assert!(matches!(
        semantics.field_length().invalid_issue(),
        Some(DxfInsertAttributeIntegerSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    ));
    assert!(semantics.field_length().raw_provenance().is_some());
    let neutral = directory
        .card_directory()
        .card_for_role(
            record.record().ordinal(),
            DxfInsertAttributeValueRole::VersionOrLockPosition,
        )
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        neutral.state(),
        DxfInsertAttributeValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    Ok(())
}

#[test]
fn cancellation_lookups_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeIntegerSemantics>();
    assert_send_sync::<DxfInsertAttributeIntegerSemanticDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_integer_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.insert_attribute_integer_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert!(directory.semantics_for_raw_record(u64::MAX)?.is_none());
    assert!(
        directory
            .semantics_for_insert_sequence_attribute(u64::MAX, 0)?
            .is_none()
    );
    let first = directory.records()[0];
    let insert = first.sequence().insert().record().ordinal();
    assert_eq!(
        directory
            .semantics_for_insert_sequence_attribute(insert, 0)?
            .map(DxfInsertAttributeIntegerSemantics::record),
        Some(first)
    );
    assert!(
        directory
            .semantics_for_insert_sequence_attribute(insert, u64::MAX)?
            .is_none()
    );
    Ok(())
}

fn assert_directory(
    directory: &DxfInsertAttributeIntegerSemanticDirectory,
) -> Result<(), DxfError> {
    assert_eq!(directory.records().len(), 2);
    let full = directory
        .semantics_for_entry(directory.records()[0])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(full.attribute_flags_value(), Some(15));
    assert_eq!(full.field_length_value(), Some(10));
    assert_eq!(full.text_generation_flags_value(), Some(6));
    assert_eq!(full.horizontal_justification_value(), Some(5));
    assert_eq!(full.vertical_justification_value(), Some(3));
    assert_eq!(full.is_invisible(), Some(true));
    assert_eq!(full.is_constant(), Some(true));
    assert_eq!(full.requires_verification(), Some(true));
    assert_eq!(full.is_preset(), Some(true));
    assert_eq!(full.is_backward(), Some(true));
    assert_eq!(full.is_upside_down(), Some(true));
    Ok(())
}

fn signatures(
    directory: &DxfInsertAttributeIntegerSemanticDirectory,
) -> Result<Vec<RecordSignature>, DxfError> {
    directory
        .records()
        .iter()
        .map(|record| {
            let semantics = directory
                .semantics_for_entry(*record)?
                .ok_or_else(invalid_test_data)?;
            Ok([
                signature(semantics.attribute_flags()),
                signature(semantics.field_length()),
                signature(semantics.text_generation_flags()),
                signature(semantics.horizontal_justification()),
                signature(semantics.vertical_justification()),
            ])
        })
        .collect()
}

fn signature(value: &seacad_dxf_core::DxfInsertAttributeSemanticInteger) -> IntegerSignature {
    (value.state(), value.value().copied())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n15\n73\n10\n71\n6\n72\n5\n74\n3\n0\nATTRIB\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"B")?;
    for code in [10, 20, 30] {
        push_double(&mut bytes, version, code, 0.0)?;
    }
    push_i16(&mut bytes, version, 66, 1)?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    for (code, value) in [(70, 15), (73, 10), (71, 6), (72, 5), (74, 3)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
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

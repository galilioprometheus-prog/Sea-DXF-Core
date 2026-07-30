use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInsertAttributeHorizontalJustification,
    DxfInsertAttributeJustificationDirectory, DxfInsertAttributeJustificationIssue,
    DxfInsertAttributeJustificationSemantics, DxfInsertAttributeVerticalJustification,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_ascii_binary_justification_parity() -> Result<(), Box<dyn Error>> {
    let pairs = [(0, 0), (5, 0), (0, 3)];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &pairs);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_justification_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, &pairs)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_justification_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
        assert_eq!(
            signatures(&ascii_directory)?,
            [
                (
                    DxfSemanticValueState::Explicit,
                    Some(0),
                    DxfSemanticValueState::Explicit,
                    Some(0),
                    Some(false),
                ),
                (
                    DxfSemanticValueState::Explicit,
                    Some(5),
                    DxfSemanticValueState::Explicit,
                    Some(0),
                    Some(true),
                ),
                (
                    DxfSemanticValueState::Explicit,
                    Some(0),
                    DxfSemanticValueState::Explicit,
                    Some(3),
                    Some(true),
                ),
            ]
        );
    }
    Ok(())
}

#[test]
fn every_documented_horizontal_and_vertical_code_is_classified() -> Result<(), Box<dyn Error>> {
    let pairs = [
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (0, 1),
        (0, 2),
        (0, 3),
    ];
    let bytes = ascii_fixture("AC1032", &pairs);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_justification_directory(&DxfCancellationToken::default())?;
    let expected_horizontal = [
        DxfInsertAttributeHorizontalJustification::Left,
        DxfInsertAttributeHorizontalJustification::Center,
        DxfInsertAttributeHorizontalJustification::Right,
        DxfInsertAttributeHorizontalJustification::Aligned,
        DxfInsertAttributeHorizontalJustification::Middle,
        DxfInsertAttributeHorizontalJustification::Fit,
    ];
    for (record, expected) in directory
        .records()
        .iter()
        .take(expected_horizontal.len())
        .zip(expected_horizontal)
    {
        let semantics = directory
            .semantics_for_entry(*record)?
            .ok_or_else(invalid_test_data)?;
        assert_eq!(semantics.horizontal().value(), Some(&expected));
        assert_eq!(
            expected.code(),
            pairs[record.sequence_attribute_ordinal() as usize].0
        );
    }
    let expected_vertical = [
        DxfInsertAttributeVerticalJustification::Bottom,
        DxfInsertAttributeVerticalJustification::Middle,
        DxfInsertAttributeVerticalJustification::Top,
    ];
    for (record, expected) in directory.records()[6..].iter().zip(expected_vertical) {
        let semantics = directory
            .semantics_for_entry(*record)?
            .ok_or_else(invalid_test_data)?;
        assert_eq!(semantics.vertical().value(), Some(&expected));
        assert!(semantics.requires_alignment_point().unwrap_or(false));
    }
    Ok(())
}

#[test]
fn unsupported_and_unavailable_codes_fail_typed_with_provenance() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n0\n72\n6\n74\n-1\n0\nATTRIB\n70\n0\n72\nbad\n74\nbad\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_justification_directory(&DxfCancellationToken::default())?;
    let unsupported = directory
        .semantics_for_entry(directory.records()[0])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        unsupported.horizontal().invalid_issue(),
        Some(&DxfInsertAttributeJustificationIssue::UnsupportedHorizontalCode { code: 6 })
    );
    assert_eq!(
        unsupported.vertical().invalid_issue(),
        Some(&DxfInsertAttributeJustificationIssue::UnsupportedVerticalCode { code: -1 })
    );
    assert!(unsupported.horizontal().raw_provenance().is_some());
    assert!(unsupported.vertical().raw_provenance().is_some());
    assert_eq!(unsupported.requires_alignment_point(), None);

    let unavailable = directory
        .semantics_for_entry(directory.records()[1])?
        .ok_or_else(invalid_test_data)?;
    assert!(matches!(
        unavailable.horizontal().invalid_issue(),
        Some(DxfInsertAttributeJustificationIssue::Integer(_))
    ));
    assert!(matches!(
        unavailable.vertical().invalid_issue(),
        Some(DxfInsertAttributeJustificationIssue::Integer(_))
    ));
    assert_eq!(unavailable.uses_text_start_point(), None);
    Ok(())
}

#[test]
fn defaults_cancellation_lookups_source_identity_and_public_traits_hold()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeJustificationSemantics>();
    assert_send_sync::<DxfInsertAttributeJustificationDirectory>();

    let bytes = ascii_fixture("AC1032", &[]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_justification_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = ascii_fixture_with_empty_attribute("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_justification_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.integer_directory().source_id()
    );
    let record = directory.records()[0];
    let semantics = directory
        .semantics_for_entry(record)?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        semantics.horizontal().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        semantics.vertical().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(semantics.requires_alignment_point(), Some(false));
    assert_eq!(semantics.uses_text_start_point(), Some(true));
    assert!(directory.semantics_for_raw_record(u64::MAX)?.is_none());
    let insert = record.sequence().insert().record().ordinal();
    assert_eq!(
        directory
            .semantics_for_insert_sequence_attribute(insert, 0)?
            .map(DxfInsertAttributeJustificationSemantics::record),
        Some(record)
    );
    assert!(
        directory
            .semantics_for_insert_sequence_attribute(insert, u64::MAX)?
            .is_none()
    );
    Ok(())
}

type Signature = (
    DxfSemanticValueState,
    Option<i16>,
    DxfSemanticValueState,
    Option<i16>,
    Option<bool>,
);

fn signatures(
    directory: &DxfInsertAttributeJustificationDirectory,
) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .map(|record| {
            let semantics = directory
                .semantics_for_entry(*record)?
                .ok_or_else(invalid_test_data)?;
            Ok((
                semantics.horizontal().state(),
                semantics.horizontal().value().map(|value| value.code()),
                semantics.vertical().state(),
                semantics.vertical().value().map(|value| value.code()),
                semantics.requires_alignment_point(),
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str, pairs: &[(i16, i16)]) -> Vec<u8> {
    let mut attributes = String::new();
    for (horizontal, vertical) in pairs {
        attributes.push_str(&format!(
            "0\nATTRIB\n70\n0\n72\n{horizontal}\n74\n{vertical}\n"
        ));
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n{attributes}0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn ascii_fixture_with_empty_attribute(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, pairs: &[(i16, i16)]) -> io::Result<Vec<u8>> {
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
    for (horizontal, vertical) in pairs {
        push_string(&mut bytes, version, 0, b"ATTRIB")?;
        push_i16(&mut bytes, version, 70, 0)?;
        push_i16(&mut bytes, version, 72, *horizontal)?;
        push_i16(&mut bytes, version, 74, *vertical)?;
    }
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

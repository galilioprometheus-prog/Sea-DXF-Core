use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfBlockAttributeDefinitionValueData,
    DxfBlockAttributeDefinitionValueDirectory, DxfBlockAttributeDefinitionValueEntry,
    DxfBlockAttributeDefinitionValueIssue, DxfBlockAttributeDefinitionValueRole, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum ValueSignature {
    Text(Vec<u8>),
    Double(u64),
    Int16(i16),
    Invalid,
}

#[test]
fn every_supported_dialect_has_ascii_binary_classic_attdef_value_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.block_attribute_definition_value_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.block_attribute_definition_value_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory, version);
        assert_directory(&binary_directory, version);
        assert_eq!(
            signatures(DxfRawDocumentView::from(&ascii), &ascii_directory)?,
            signatures(DxfRawDocumentView::from(&binary), &binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn duplicates_invalid_numbers_and_subclass_boundaries_remain_explicit() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nB\n0\nATTDEF\n100\nCustomSubclass\n3\nignored\n100\nAcDbText\n10\n1\n10\n2\n40\nx\n102\n{APP\n1\nignored-app\n102\n}\n100\nAcDbAttributeDefinition\n3\nprompt\n280\n99999\n100\nAcDbXrecord\n70\n7\n3\nignored-extension\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.block_attribute_definition_value_directory(&DxfCancellationToken::default())?;
    let values = directory
        .values_for_raw_record(directory.records()[0].definition().record().ordinal())
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        values.iter().map(|value| value.role()).collect::<Vec<_>>(),
        [
            DxfBlockAttributeDefinitionValueRole::TextStartX,
            DxfBlockAttributeDefinitionValueRole::TextStartX,
            DxfBlockAttributeDefinitionValueRole::TextHeight,
            DxfBlockAttributeDefinitionValueRole::Prompt,
            DxfBlockAttributeDefinitionValueRole::VersionOrLockPosition,
        ]
    );
    assert!(matches!(
        values[2].value(),
        Err(DxfBlockAttributeDefinitionValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    assert!(values[4].value().is_err());
    assert_eq!(
        decode_text(
            DxfRawDocumentView::from(&document),
            values[3].value().map_err(|_| invalid_test_data())?
        )?,
        b"prompt"
    );
    Ok(())
}

#[test]
fn text_projection_rejects_source_identity_mismatch() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.block_attribute_definition_value_directory(&DxfCancellationToken::default())?;
    let text = directory
        .values()
        .iter()
        .find_map(|value| match value.value() {
            Ok(DxfBlockAttributeDefinitionValueData::Text(text)) => Some(text),
            Ok(_) | Err(_) => None,
        })
        .ok_or_else(invalid_test_data)?;

    let other_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nEOF\n";
    let other_source = DxfMemorySource::new(other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        text.decode_to_utf8_without_replacement(DxfRawDocumentView::from(&other), &mut [0_u8; 32]),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn cancellation_lookup_locality_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfBlockAttributeDefinitionValueEntry>();
    assert_send_sync::<DxfBlockAttributeDefinitionValueDirectory>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.block_attribute_definition_value_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.block_attribute_definition_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.definition_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.application_group_directory().source_id()
    );
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    for record in directory.records().iter().copied() {
        let ordinal = record.definition().record().ordinal();
        assert_eq!(directory.record_for_raw_ordinal(ordinal), Some(record));
        let values = directory
            .values_for_raw_record(ordinal)
            .ok_or_else(invalid_test_data)?;
        assert_eq!(values.len() as u64, record.value_range().len());
        for value in values {
            assert_eq!(
                directory.value_for_group(value.group().occurrence()),
                Some(*value)
            );
        }
    }
    Ok(())
}

fn assert_directory(
    directory: &DxfBlockAttributeDefinitionValueDirectory,
    version: DxfAcadVersion,
) {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(
        directory.records()[0]
            .definition()
            .attribute_definition_ordinal(),
        0
    );
    assert_eq!(
        directory.records()[1]
            .definition()
            .attribute_definition_ordinal(),
        1
    );
    assert_eq!(directory.records()[1].value_range().len(), 0);
    let expected = if version == DxfAcadVersion::Ac1009 {
        23
    } else {
        25
    };
    assert_eq!(directory.records()[0].value_range().len(), expected);
    assert_eq!(directory.values().len() as u64, expected);
}

fn signatures(
    document: DxfRawDocumentView<'_>,
    directory: &DxfBlockAttributeDefinitionValueDirectory,
) -> Result<Vec<(DxfBlockAttributeDefinitionValueRole, ValueSignature)>, DxfError> {
    directory
        .values()
        .iter()
        .map(|value| {
            let signature = match value.value() {
                Ok(DxfBlockAttributeDefinitionValueData::Text(text)) => {
                    let length = usize::try_from(text.value_span().len())
                        .map_err(|_| invalid_test_data())?;
                    let mut raw = vec![0_u8; length];
                    document.read_span(text.value_span(), &mut raw)?;
                    ValueSignature::Text(raw)
                }
                Ok(DxfBlockAttributeDefinitionValueData::Double(number)) => {
                    ValueSignature::Double(number.to_bits())
                }
                Ok(DxfBlockAttributeDefinitionValueData::Int16(number)) => {
                    ValueSignature::Int16(number)
                }
                Ok(_) | Err(_) => ValueSignature::Invalid,
            };
            Ok((value.role(), signature))
        })
        .collect()
}

fn decode_text(
    document: DxfRawDocumentView<'_>,
    value: DxfBlockAttributeDefinitionValueData,
) -> Result<Vec<u8>, DxfError> {
    let DxfBlockAttributeDefinitionValueData::Text(text) = value else {
        return Err(invalid_test_data());
    };
    let mut decoded = [0_u8; 32];
    let receipt = text.decode_to_utf8_without_replacement(document, &mut decoded)?;
    let written = receipt
        .decode_result()
        .ok_or_else(invalid_test_data)?
        .written();
    Ok(decoded[..written].to_vec())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let text_subclass = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "100\nAcDbEntity\n1\nignored-before\n100\nAcDbText\n"
    };
    let definition_subclass = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "100\nAcDbAttributeDefinition\n"
    };
    let version_lock = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "280\n0\n280\n1\n"
    };
    let extension = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "100\nAcDbXrecord\n70\n7\n3\nignored-extension\n"
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nB\n0\nATTDEF\n{}39\n1\n10\n2\n20\n3\n30\n4\n40\n5\n1\ndefault\n{}{}3\nprompt\n2\ntag\n70\n9\n73\n10\n50\n11\n41\n12\n51\n13\n7\nSTYLE\n71\n2\n72\n3\n74\n1\n11\n14\n21\n15\n31\n16\n210\n0\n220\n0\n230\n1\n102\n{{APP\n1\nignored-app\n102\n}}\n{}0\nATTDEF\n0\nENDBLK\n0\nATTDEF\n3\norphan\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nATTDEF\n3\noutside\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        text_subclass,
        definition_subclass,
        version_lock,
        extension
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 2, b"B")?;
    push_string(&mut bytes, version, 0, b"ATTDEF")?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 100, b"AcDbEntity")?;
        push_string(&mut bytes, version, 1, b"ignored-before")?;
        push_string(&mut bytes, version, 100, b"AcDbText")?;
    }
    for (code, value) in [(39, 1.0), (10, 2.0), (20, 3.0), (30, 4.0), (40, 5.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 1, b"default")?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 100, b"AcDbAttributeDefinition")?;
        push_i16(&mut bytes, version, 280, 0)?;
        push_i16(&mut bytes, version, 280, 1)?;
    }
    for (code, value) in [(3, b"prompt".as_slice()), (2, b"tag")] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(70, 9), (73, 10)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(50, 11.0), (41, 12.0), (51, 13.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 7, b"STYLE")?;
    for (code, value) in [(71, 2), (72, 3), (74, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for (code, value) in [
        (11, 14.0),
        (21, 15.0),
        (31, 16.0),
        (210, 0.0),
        (220, 0.0),
        (230, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 102, b"{APP")?;
    push_string(&mut bytes, version, 1, b"ignored-app")?;
    push_string(&mut bytes, version, 102, b"}")?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 100, b"AcDbXrecord")?;
        push_i16(&mut bytes, version, 70, 7)?;
        push_string(&mut bytes, version, 3, b"ignored-extension")?;
    }
    push_string(&mut bytes, version, 0, b"ATTDEF")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_string(&mut bytes, version, 0, b"ATTDEF")?;
    push_string(&mut bytes, version, 3, b"orphan")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"ATTDEF")?;
    push_string(&mut bytes, version, 3, b"outside")?;
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

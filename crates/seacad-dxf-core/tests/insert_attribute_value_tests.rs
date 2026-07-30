use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfInsertAttributeValueData, DxfInsertAttributeValueDirectory, DxfInsertAttributeValueEntry,
    DxfInsertAttributeValueIssue, DxfInsertAttributeValueRole, DxfMemorySource, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum ValueSignature {
    Text(Vec<u8>),
    Double(u64),
    Int16(i16),
    Invalid,
}

#[test]
fn every_supported_dialect_has_ascii_binary_classic_value_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_value_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_value_directory(&DxfCancellationToken::default())?;

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
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n100\nCustomSubclass\n2\nignored\n100\nAcDbText\n10\n1\n10\n2\n40\nx\n102\n{APP\n1\nignored-app\n102\n}\n100\nAcDbAttribute\n2\nreal\n280\n99999\n100\nAcDbXrecord\n70\n7\n2\nignored-extension\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_attribute_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 1);
    let values = directory
        .values_for_raw_record(directory.records()[0].record().ordinal())
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        values.iter().map(|value| value.role()).collect::<Vec<_>>(),
        [
            DxfInsertAttributeValueRole::TextStartX,
            DxfInsertAttributeValueRole::TextStartX,
            DxfInsertAttributeValueRole::TextHeight,
            DxfInsertAttributeValueRole::AttributeTag,
            DxfInsertAttributeValueRole::VersionOrLockPosition,
        ]
    );
    assert!(matches!(
        values[2].value(),
        Err(DxfInsertAttributeValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    assert!(values[4].value().is_err());
    let text = match values[3].value().map_err(|_| invalid_test_data())? {
        DxfInsertAttributeValueData::Text(text) => text,
        _ => return Err(invalid_test_data().into()),
    };
    let mut decoded = [0_u8; 16];
    let receipt =
        text.decode_to_utf8_without_replacement(DxfRawDocumentView::from(&document), &mut decoded)?;
    let written = receipt
        .decode_result()
        .ok_or_else(invalid_test_data)?
        .written();
    assert_eq!(&decoded[..written], b"real");
    Ok(())
}

#[test]
fn text_projection_rejects_source_identity_mismatch() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_attribute_value_directory(&DxfCancellationToken::default())?;
    let text = directory
        .values()
        .iter()
        .find_map(|value| match value.value() {
            Ok(DxfInsertAttributeValueData::Text(text)) => Some(text),
            Ok(DxfInsertAttributeValueData::Double(_) | DxfInsertAttributeValueData::Int16(_))
            | Ok(_)
            | Err(_) => None,
        })
        .ok_or_else(invalid_test_data)?;

    let other_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nEOF\n";
    let other_source = DxfMemorySource::new(other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        text.decode_to_utf8_without_replacement(DxfRawDocumentView::from(&other), &mut [0_u8; 32],),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn cancellation_lookup_locality_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeValueEntry>();
    assert_send_sync::<DxfInsertAttributeValueDirectory>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_value_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.insert_attribute_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.sequence_directory().source_id()
    );
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    for record in directory.records() {
        assert_eq!(
            directory.record_for_raw_ordinal(record.record().ordinal()),
            Some(*record)
        );
        let values = directory
            .values_for_raw_record(record.record().ordinal())
            .ok_or_else(invalid_test_data)?;
        assert_eq!(values.len() as u64, record.value_range().len());
        for value in values {
            assert_eq!(
                directory.value_for_group(value.group().occurrence()),
                Some(*value)
            );
            assert_eq!(
                record.sequence().insert().record(),
                directory.sequence_directory().entries()[0]
                    .insert()
                    .record()
            );
        }
    }
    Ok(())
}

fn assert_directory(directory: &DxfInsertAttributeValueDirectory, version: DxfAcadVersion) {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.records()[0].sequence_attribute_ordinal(), 0);
    assert_eq!(directory.records()[1].sequence_attribute_ordinal(), 1);
    assert_eq!(directory.records()[1].value_range().len(), 0);
    let expected = if version == DxfAcadVersion::Ac1009 {
        22
    } else {
        24
    };
    assert_eq!(directory.records()[0].value_range().len(), expected);
    assert_eq!(directory.values().len() as u64, expected);
    assert!(!directory
        .values()
        .iter()
        .any(|value| matches!(value.value(), Ok(DxfInsertAttributeValueData::Text(text)) if text.value_span().len() == b"ignored-extension".len() as u64)));
}

fn signatures(
    document: DxfRawDocumentView<'_>,
    directory: &DxfInsertAttributeValueDirectory,
) -> Result<Vec<(DxfInsertAttributeValueRole, ValueSignature)>, DxfError> {
    directory
        .values()
        .iter()
        .map(|value| {
            let signature = match value.value() {
                Ok(DxfInsertAttributeValueData::Text(text)) => {
                    let length = usize::try_from(text.value_span().len())
                        .map_err(|_| invalid_test_data())?;
                    let mut raw = vec![0_u8; length];
                    document.read_span(text.value_span(), &mut raw)?;
                    ValueSignature::Text(raw)
                }
                Ok(DxfInsertAttributeValueData::Double(number)) => {
                    ValueSignature::Double(number.to_bits())
                }
                Ok(DxfInsertAttributeValueData::Int16(number)) => ValueSignature::Int16(number),
                Ok(_) | Err(_) => ValueSignature::Invalid,
            };
            Ok((value.role(), signature))
        })
        .collect()
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let subclass_prefix = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "100\nAcDbEntity\n1\nignored-before\n100\nAcDbText\n".to_owned()
    };
    let attribute_marker = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "100\nAcDbAttribute\n".to_owned()
    };
    let version_lock = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "280\n0\n280\n1\n".to_owned()
    };
    let extension = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "100\nAcDbXrecord\n70\n7\n2\nignored-extension\n10\n999\n40\n999\n".to_owned()
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n{}39\n1\n10\n2\n20\n3\n30\n4\n40\n5\n1\nvalue\n{}{}2\ntag\n70\n9\n73\n10\n50\n11\n41\n12\n51\n13\n7\nSTYLE\n71\n2\n72\n3\n74\n1\n11\n14\n21\n15\n31\n16\n210\n0\n220\n0\n230\n1\n102\n{{APP\n1\nignored-app\n102\n}}\n{}0\nATTRIB\n0\nSEQEND\n0\nATTRIB\n1\norphan\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        subclass_prefix,
        attribute_marker,
        version_lock,
        extension,
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
    push_insert(&mut bytes, version)?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 100, b"AcDbEntity")?;
        push_string(&mut bytes, version, 1, b"ignored-before")?;
        push_string(&mut bytes, version, 100, b"AcDbText")?;
    }
    for (code, value) in [(39, 1.0), (10, 2.0), (20, 3.0), (30, 4.0), (40, 5.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 1, b"value")?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 100, b"AcDbAttribute")?;
        push_i16(&mut bytes, version, 280, 0)?;
        push_i16(&mut bytes, version, 280, 1)?;
    }
    push_string(&mut bytes, version, 2, b"tag")?;
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
        push_string(&mut bytes, version, 2, b"ignored-extension")?;
        push_double(&mut bytes, version, 10, 999.0)?;
        push_double(&mut bytes, version, 40, 999.0)?;
    }
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 1, b"orphan")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_insert(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"INSERT")?;
    push_string(bytes, version, 2, b"B")?;
    for code in [10, 20, 30] {
        push_double(bytes, version, code, 0.0)?;
    }
    push_i16(bytes, version, 66, 1)
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

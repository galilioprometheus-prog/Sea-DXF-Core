use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfInsertRecordTextValue,
    DxfInsertRecordValue, DxfInsertRecordValueData, DxfInsertRecordValueDirectory,
    DxfInsertRecordValueEntry, DxfInsertRecordValueIssue, DxfInsertRecordValueRole,
    DxfMemorySource, DxfRawDocumentView, DxfRawRecordSectionKind, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

const EXPECTED_ROLES: [DxfInsertRecordValueRole; 16] = [
    DxfInsertRecordValueRole::BlockName,
    DxfInsertRecordValueRole::InsertionPointX,
    DxfInsertRecordValueRole::InsertionPointY,
    DxfInsertRecordValueRole::InsertionPointZ,
    DxfInsertRecordValueRole::ScaleFactorX,
    DxfInsertRecordValueRole::ScaleFactorY,
    DxfInsertRecordValueRole::ScaleFactorZ,
    DxfInsertRecordValueRole::RotationAngle,
    DxfInsertRecordValueRole::ColumnCount,
    DxfInsertRecordValueRole::RowCount,
    DxfInsertRecordValueRole::ColumnSpacing,
    DxfInsertRecordValueRole::RowSpacing,
    DxfInsertRecordValueRole::AttributesFollow,
    DxfInsertRecordValueRole::ExtrusionX,
    DxfInsertRecordValueRole::ExtrusionY,
    DxfInsertRecordValueRole::ExtrusionZ,
];

#[test]
fn every_supported_dialect_has_ascii_binary_insert_value_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_record_value_directory(&DxfCancellationToken::default())?;
        assert_directory(&ascii_directory)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_record_value_directory(&DxfCancellationToken::default())?;
        assert_directory(&binary_directory)?;
    }
    Ok(())
}

#[test]
fn duplicates_invalid_numbers_and_application_groups_remain_distinct() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n10\nbad\n10\n1\n102\n{APP\n2\nHidden\n10\n9\n102\n}\n66\nbad\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_record_value_directory(&DxfCancellationToken::default())?;
    let entry = *directory.records().first().ok_or_else(invalid_test_data)?;
    let values = directory
        .values_for_raw_record(entry.record().ordinal())
        .ok_or_else(invalid_test_data)?;

    assert_eq!(values.len(), 4);
    assert_eq!(
        values.iter().map(|value| value.role()).collect::<Vec<_>>(),
        [
            DxfInsertRecordValueRole::BlockName,
            DxfInsertRecordValueRole::InsertionPointX,
            DxfInsertRecordValueRole::InsertionPointX,
            DxfInsertRecordValueRole::AttributesFollow,
        ]
    );
    assert!(matches!(
        values[1].value(),
        Err(DxfInsertRecordValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    assert!(matches!(
        values[2].value(),
        Ok(DxfInsertRecordValueData::Double(_))
    ));
    assert!(matches!(
        values[3].value(),
        Err(DxfInsertRecordValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    assert_eq!(directory.application_group_directory().groups().len(), 1);
    Ok(())
}

#[test]
fn marker_and_section_matching_are_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nINSERT\n2\nTable\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nINSERT\n2\nBlock\n0\ninsert\n2\nLower\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nEntity\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_record_value_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    assert_eq!(
        directory.records()[0].record().section_kind(),
        DxfRawRecordSectionKind::Blocks
    );
    assert_eq!(
        directory.records()[1].record().section_kind(),
        DxfRawRecordSectionKind::Entities
    );
    assert_eq!(directory.values().len(), 2);
    Ok(())
}

#[test]
fn text_receipts_lookups_cancellation_and_public_bounds_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertRecordTextValue>();
    assert_copy::<DxfInsertRecordValue>();
    assert_copy::<DxfInsertRecordValueEntry>();
    assert_send_sync::<DxfInsertRecordValueDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_record_value_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.insert_record_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    for entry in directory.records() {
        assert_eq!(
            directory.record_for_raw_ordinal(entry.record().ordinal()),
            Some(*entry)
        );
    }
    for value in directory.values() {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(*value)
        );
    }

    let text = match directory.values()[0].value() {
        Ok(DxfInsertRecordValueData::Text(text)) => text,
        other => return Err(io::Error::other(format!("unexpected value: {other:?}")).into()),
    };
    let mut decoded = [0_u8; 3];
    let receipt =
        text.decode_to_utf8_without_replacement(DxfRawDocumentView::from(&document), &mut decoded)?;
    let written = receipt
        .decode_result()
        .ok_or_else(invalid_test_data)?
        .written();
    assert_eq!(&decoded[..written], b"Ref");
    Ok(())
}

fn assert_directory(directory: &DxfInsertRecordValueDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.values().len(), 17);
    assert_eq!(
        directory.records()[0].record().section_kind(),
        DxfRawRecordSectionKind::Blocks
    );
    assert_eq!(
        directory.records()[1].record().section_kind(),
        DxfRawRecordSectionKind::Entities
    );
    let first = directory
        .values_for_raw_record(directory.records()[0].record().ordinal())
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        first.iter().map(|value| value.role()).collect::<Vec<_>>(),
        EXPECTED_ROLES
    );
    assert!(matches!(
        first[0].value(),
        Ok(DxfInsertRecordValueData::Text(_))
    ));
    for (index, value) in first.iter().enumerate().skip(1) {
        if matches!(index, 8 | 9 | 12) {
            assert!(matches!(
                value.value(),
                Ok(DxfInsertRecordValueData::Int16(_))
            ));
        } else {
            assert!(matches!(
                value.value(),
                Ok(DxfInsertRecordValueData::Double(_))
            ));
        }
    }
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nOwner\n3\nOwner\n0\nINSERT\n2\nRef\n10\n1\n20\n2\n30\n3\n41\n4\n42\n5\n43\n6\n50\n7\n70\n8\n71\n9\n44\n10\n45\n11\n66\n1\n210\n12\n220\n13\n230\n14\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nRef\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary_section_start(&mut bytes, version, b"HEADER")?;
    push_binary_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_binary_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;

    push_binary_section_start(&mut bytes, version, b"BLOCKS")?;
    push_binary_string(&mut bytes, version, 0, b"BLOCK")?;
    push_binary_string(&mut bytes, version, 2, b"Owner")?;
    push_binary_string(&mut bytes, version, 3, b"Owner")?;
    push_binary_string(&mut bytes, version, 0, b"INSERT")?;
    push_binary_string(&mut bytes, version, 2, b"Ref")?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (41, 4.0),
        (42, 5.0),
        (43, 6.0),
        (50, 7.0),
    ] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_i16(&mut bytes, version, 70, 8)?;
    push_binary_i16(&mut bytes, version, 71, 9)?;
    push_binary_double(&mut bytes, version, 44, 10.0)?;
    push_binary_double(&mut bytes, version, 45, 11.0)?;
    push_binary_i16(&mut bytes, version, 66, 1)?;
    for (code, value) in [(210, 12.0), (220, 13.0), (230, 14.0)] {
        push_binary_double(&mut bytes, version, code, value)?;
    }
    push_binary_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;

    push_binary_section_start(&mut bytes, version, b"ENTITIES")?;
    push_binary_string(&mut bytes, version, 0, b"INSERT")?;
    push_binary_string(&mut bytes, version, 2, b"Ref")?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_section_start(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
) -> io::Result<()> {
    push_binary_string(bytes, version, 0, b"SECTION")?;
    push_binary_string(bytes, version, 2, name)
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_binary_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_binary_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_binary_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_binary_i16(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: i16,
) -> io::Result<()> {
    push_binary_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_binary_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
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

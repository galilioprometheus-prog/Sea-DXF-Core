use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextEmbeddedColumnDirectory, DxfMTextEmbeddedColumnEntry,
    DxfMTextEmbeddedColumnRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfTextSymbolValueCardState, DxfTextSymbolValueData, DxfTextSymbolValueRole,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    role: DxfMTextEmbeddedColumnRole,
    bits: u64,
}

#[test]
fn every_dialect_has_ascii_binary_embedded_column_parity() -> Result<(), Box<dyn Error>> {
    let expected = [
        integer(DxfMTextEmbeddedColumnRole::Version, 1),
        double(DxfMTextEmbeddedColumnRole::SharedHeight, 50.0),
        integer(DxfMTextEmbeddedColumnRole::ColumnType, 2),
        integer(DxfMTextEmbeddedColumnRole::ColumnCount, 3),
        double(DxfMTextEmbeddedColumnRole::ColumnWidth, 20.0),
        double(DxfMTextEmbeddedColumnRole::ColumnGutter, 2.0),
        integer(DxfMTextEmbeddedColumnRole::ColumnAutoHeight, 0),
        integer(DxfMTextEmbeddedColumnRole::ColumnFlowReversed, 1),
        double(DxfMTextEmbeddedColumnRole::ColumnHeight, 10.0),
        double(DxfMTextEmbeddedColumnRole::ColumnHeight, 20.0),
        double(DxfMTextEmbeddedColumnRole::ColumnHeight, 30.0),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), "Embedded Object");
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_columns =
            ascii.mtext_embedded_column_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, b"Embedded Object")?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_columns =
            binary.mtext_embedded_column_directory(&DxfCancellationToken::default())?;

        assert_eq!(signatures(&ascii_columns)?, expected);
        assert_eq!(signatures(&binary_columns)?, expected);
        assert_eq!(ascii_columns.entries().len(), 1);
        assert_eq!(binary_columns.entries().len(), 1);
        assert_eq!(
            ascii_columns.entries()[0].marker().group_code().value(),
            101
        );
    }
    Ok(())
}

#[test]
fn embedded_groups_do_not_contaminate_main_mtext_cards() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032", "Embedded Object");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cards = document.text_symbol_card_directory(&DxfCancellationToken::default())?;
    let record = cards
        .evidence_directory()
        .records()
        .first()
        .copied()
        .ok_or_else(invalid_test_data)?;
    for role in [
        DxfTextSymbolValueRole::Attachment,
        DxfTextSymbolValueRole::DrawingDirection,
        DxfTextSymbolValueRole::LineSpacingFactor,
    ] {
        assert_eq!(
            cards
                .card_for_role(record.record().ordinal(), role)
                .ok_or_else(invalid_test_data)?
                .state(),
            DxfTextSymbolValueCardState::Unique
        );
    }
    let main_values = cards
        .evidence_directory()
        .values_for_raw_record(record.record().ordinal())
        .ok_or_else(invalid_test_data)?;
    assert_eq!(main_values.len(), 3);
    assert_eq!(
        main_values
            .iter()
            .map(|value| value.group().group_code().value())
            .collect::<Vec<_>>(),
        [71, 72, 44]
    );
    Ok(())
}

#[test]
fn marker_matching_is_exact_and_scope_identity_cancellation_are_bounded()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextEmbeddedColumnEntry>();
    assert_send_sync::<DxfMTextEmbeddedColumnDirectory>();

    let bytes = ascii_fixture("AC1032", "Not Embedded Object");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let columns = document.mtext_embedded_column_directory(&DxfCancellationToken::default())?;
    assert!(columns.entries().is_empty());
    assert!(columns.values().is_empty());
    assert_eq!(columns.entry_for_raw_record(u64::MAX), None);

    let bytes = ascii_fixture("AC1032", "Embedded Object");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_embedded_column_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let columns = document.mtext_embedded_column_directory(&DxfCancellationToken::default())?;
    assert_eq!(columns.source_id(), document.source_id());
    let entry = columns
        .entries()
        .first()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        columns
            .entry_for_raw_record(entry.record().ordinal())
            .ok_or_else(invalid_test_data)?,
        entry
    );
    assert_eq!(
        columns
            .values_for_entry(entry)
            .ok_or_else(invalid_test_data)?
            .len() as u64,
        entry.value_count()
    );
    Ok(())
}

fn signatures(
    directory: &DxfMTextEmbeddedColumnDirectory,
) -> Result<Vec<Signature>, Box<dyn Error>> {
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            let bits = match value.data() {
                DxfTextSymbolValueData::Double(Ok(number)) => number.to_bits(),
                DxfTextSymbolValueData::Int16(Ok(number)) => number as i64 as u64,
                _ => return Err(invalid_test_data().into()),
            };
            Ok(Signature {
                role: value.role(),
                bits,
            })
        })
        .collect()
}

fn integer(role: DxfMTextEmbeddedColumnRole, value: i16) -> Signature {
    Signature {
        role,
        bits: value as i64 as u64,
    }
}

fn double(role: DxfMTextEmbeddedColumnRole, value: f64) -> Signature {
    Signature {
        role,
        bits: value.to_bits(),
    }
}

fn ascii_fixture(version: &str, marker: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n71\n1\n72\n1\n44\n1.0\n\
101\n{marker}\n70\n1\n41\n50.0\n71\n2\n72\n3\n44\n20.0\n45\n2.0\n\
73\n0\n74\n1\n46\n10.0\n46\n20.0\n46\n30.0\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, marker: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"MTEXT")?;
    push_i16(&mut bytes, version, 71, 1)?;
    push_i16(&mut bytes, version, 72, 1)?;
    push_double(&mut bytes, version, 44, 1.0)?;
    push_string(&mut bytes, version, 101, marker)?;
    push_i16(&mut bytes, version, 70, 1)?;
    push_double(&mut bytes, version, 41, 50.0)?;
    push_i16(&mut bytes, version, 71, 2)?;
    push_i16(&mut bytes, version, 72, 3)?;
    push_double(&mut bytes, version, 44, 20.0)?;
    push_double(&mut bytes, version, 45, 2.0)?;
    push_i16(&mut bytes, version, 73, 0)?;
    push_i16(&mut bytes, version, 74, 1)?;
    for height in [10.0, 20.0, 30.0] {
        push_double(&mut bytes, version, 46, height)?;
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

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDimStyleField,
    DxfDimStyleFieldCardDirectory, DxfDimStyleFieldCardState, DxfDimStyleValueData,
    DxfDimStyleValueDirectory, DxfDimStyleValueIssue, DxfDimStyleWireKind, DxfError,
    DxfHandleParseIssue, DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver, dxf_dimstyle_fields,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValueSignature {
    Text(Vec<u8>),
    Double(u64),
    Int16(i16),
    Handle(u64),
}

#[test]
fn registry_is_complete_sorted_and_wire_typed() {
    let fields = dxf_dimstyle_fields();
    let expected_codes = [
        3, 4, 5, 6, 7, 40, 41, 42, 43, 44, 45, 46, 47, 48, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
        140, 141, 142, 143, 144, 145, 146, 147, 148, 170, 171, 172, 173, 174, 175, 176, 177, 178,
        179, 270, 271, 272, 273, 274, 275, 276, 277, 278, 279, 280, 281, 282, 283, 284, 285, 286,
        287, 288, 289, 340, 341, 342, 343, 344,
    ];
    assert_eq!(fields.len(), expected_codes.len());
    assert_eq!(
        fields
            .iter()
            .map(|field| field.group_code())
            .collect::<Vec<_>>(),
        expected_codes
    );
    assert_eq!(wire_count(fields, DxfDimStyleWireKind::Text), 5);
    assert_eq!(wire_count(fields, DxfDimStyleWireKind::Double), 18);
    assert_eq!(wire_count(fields, DxfDimStyleWireKind::Int16), 40);
    assert_eq!(wire_count(fields, DxfDimStyleWireKind::Handle), 5);
    assert!(fields.iter().all(|field| !field.name().is_empty()));
    assert_eq!(field(3).map(DxfDimStyleField::name), Some("DIMPOST"));
    assert_eq!(
        field(70).map(DxfDimStyleField::name),
        Some("STANDARD_FLAGS")
    );
    assert_eq!(field(344).map(DxfDimStyleField::name), Some("DIMBLK2"));
}

#[test]
fn every_dialect_has_ascii_binary_typed_value_and_card_parity() -> Result<(), Box<dyn Error>> {
    let expected = [
        ValueSignature::Text(b"<>".to_vec()),
        ValueSignature::Double(2.5_f64.to_bits()),
        ValueSignature::Double((-3.0_f64).to_bits()),
        ValueSignature::Int16(64),
        ValueSignature::Handle(0x1a),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let expected = expected
            .iter()
            .filter(|value| {
                version != DxfAcadVersion::Ac1009 || !matches!(value, ValueSignature::Handle(_))
            })
            .cloned()
            .collect::<Vec<_>>();
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;

        assert_document(DxfRawDocumentView::from(&ascii), &expected)?;
        assert_document(DxfRawDocumentView::from(&binary), &expected)?;
    }
    Ok(())
}

#[test]
fn malformed_values_are_typed_without_losing_source_evidence() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nDIMSTYLE\n0\nDIMSTYLE\n2\nBad\n\
40\nnot-double\n70\nnot-int16\n340\n1G\n0\nENDTAB\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.dimstyle_value_directory(&DxfCancellationToken::default())?;
    let values = directory.values();
    assert_eq!(values.len(), 3);
    assert!(matches!(
        values[0].value(),
        Err(DxfDimStyleValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    assert!(matches!(
        values[1].value(),
        Err(DxfDimStyleValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    assert_eq!(
        values[2].value(),
        Err(DxfDimStyleValueIssue::InvalidHandle(
            DxfHandleParseIssue::InvalidDigit { offset: 1 }
        ))
    );
    for value in values {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(*value)
        );
    }
    assert_eq!(directory.value_for_group(u64::MAX), None);
    Ok(())
}

#[test]
fn cancellation_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfDimStyleField>();
    assert_send_sync::<DxfDimStyleValueDirectory>();
    assert_send_sync::<DxfDimStyleFieldCardDirectory>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.dimstyle_value_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        document.dimstyle_field_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.dimstyle_field_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    Ok(())
}

fn assert_document(
    document: DxfRawDocumentView<'_>,
    expected: &[ValueSignature],
) -> Result<(), DxfError> {
    let cancellation = DxfCancellationToken::default();
    let evidence = document.dimstyle_value_directory(&cancellation)?;
    assert_eq!(evidence.records().len(), 2);
    let first = evidence.records()[0];
    let second = evidence.records()[1];
    assert_eq!(signatures(document, &evidence)?, expected);
    assert_eq!(first.value_range().len(), expected.len() as u64);
    assert!(second.value_range().is_empty());
    assert_eq!(
        evidence.record_for_raw_ordinal(first.table_entry().record().ordinal()),
        Some(first)
    );

    let cards = document.dimstyle_field_card_directory(&cancellation)?;
    assert_eq!(cards.cards().len(), dxf_dimstyle_fields().len() * 2);
    assert_eq!(cards.members().len(), expected.len());
    let raw = first.table_entry().record().ordinal();
    let double_card = cards
        .card_for_field(raw, field(40).ok_or_else(invalid_test_data)?)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        double_card.state(),
        DxfDimStyleFieldCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        cards
            .members_for_card(double_card.ordinal())
            .map(<[_]>::len),
        Some(2)
    );
    let decoy_card = cards
        .card_for_field(raw, field(41).ok_or_else(invalid_test_data)?)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(decoy_card.state(), DxfDimStyleFieldCardState::Absent);
    let minimal_raw = second.table_entry().record().ordinal();
    assert!(
        cards
            .cards_for_raw_ordinal(minimal_raw)
            .ok_or_else(invalid_test_data)?
            .iter()
            .all(|card| card.state() == DxfDimStyleFieldCardState::Absent)
    );
    Ok(())
}

fn signatures(
    document: DxfRawDocumentView<'_>,
    directory: &DxfDimStyleValueDirectory,
) -> Result<Vec<ValueSignature>, DxfError> {
    directory
        .values()
        .iter()
        .map(
            |value| match value.value().map_err(|_| invalid_test_data())? {
                DxfDimStyleValueData::Text(raw) => {
                    let mut bytes = vec![
                        0_u8;
                        usize::try_from(raw.value_span().len())
                            .map_err(|_| invalid_test_data())?
                    ];
                    document.read_span(raw.value_span(), &mut bytes)?;
                    Ok(ValueSignature::Text(bytes))
                }
                DxfDimStyleValueData::Double(value) => Ok(ValueSignature::Double(value.to_bits())),
                DxfDimStyleValueData::Int16(value) => Ok(ValueSignature::Int16(value)),
                DxfDimStyleValueData::Handle(value) => Ok(ValueSignature::Handle(value.value())),
                _ => Err(invalid_test_data()),
            },
        )
        .collect()
}

fn wire_count(fields: &[DxfDimStyleField], kind: DxfDimStyleWireKind) -> usize {
    fields
        .iter()
        .filter(|field| field.wire_kind() == kind)
        .count()
}

fn field(code: i16) -> Option<DxfDimStyleField> {
    dxf_dimstyle_fields()
        .binary_search_by_key(&code, |field| field.group_code())
        .ok()
        .and_then(|index| dxf_dimstyle_fields().get(index).copied())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let handle = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "340\n1A\n"
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nDIMSTYLE\n0\nDIMSTYLE\n2\nFull\n\
3\n<>\n40\n2.5\n40\n-3\n70\n64\n{handle}102\n{{APP\n41\n99\n102\n}}\n\
0\nDIMSTYLE\n2\nMinimal\n0\nENDTAB\n0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"TABLES")?;
    push_string(&mut bytes, version, 0, b"TABLE")?;
    push_string(&mut bytes, version, 2, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 0, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 2, b"Full")?;
    push_string(&mut bytes, version, 3, b"<>")?;
    push_double(&mut bytes, version, 40, 2.5)?;
    push_double(&mut bytes, version, 40, -3.0)?;
    push_i16(&mut bytes, version, 70, 64)?;
    if version != DxfAcadVersion::Ac1009 {
        push_string(&mut bytes, version, 340, b"1A")?;
    }
    push_string(&mut bytes, version, 102, b"{APP")?;
    push_double(&mut bytes, version, 41, 99.0)?;
    push_string(&mut bytes, version, 102, b"}")?;
    push_string(&mut bytes, version, 0, b"DIMSTYLE")?;
    push_string(&mut bytes, version, 2, b"Minimal")?;
    push_string(&mut bytes, version, 0, b"ENDTAB")?;
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

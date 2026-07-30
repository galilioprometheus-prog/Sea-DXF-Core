use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfTextSymbolCardDirectory, DxfTextSymbolCardMember, DxfTextSymbolKind, DxfTextSymbolValueCard,
    DxfTextSymbolValueCardState, DxfTextSymbolValueData, DxfTextSymbolValueRole,
    NoopDxfReadObserver,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValueSignature {
    Text,
    Double(u64),
    Int16(i16),
    Int32(i32),
}

type CardSignature = (
    DxfTextSymbolKind,
    DxfTextSymbolValueRole,
    DxfTextSymbolValueCardState,
    Vec<ValueSignature>,
);

#[test]
fn every_dialect_has_ascii_binary_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_cards = ascii.text_symbol_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_cards = binary.text_symbol_card_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_cards)?;
        assert_directory(&binary_cards)?;
        assert_eq!(signatures(&ascii_cards)?, signatures(&binary_cards)?);
    }
    Ok(())
}

#[test]
fn cardinality_lexical_validity_and_family_roles_are_independent() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n1\na\n1\nb\n40\n.\n90\n1\n\
0\nMTEXT\n3\nx\n3\ny\n50\n.\n\
0\nSHAPE\n2\nBOLT\n71\n7\n\
0\nTOLERANCE\n3\nISO\n40\n2\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_symbol_card_directory(&DxfCancellationToken::default())?;
    let records = directory.evidence_directory().records();

    let content = card(
        &directory,
        records[0].record().ordinal(),
        DxfTextSymbolValueRole::Content,
    )?;
    assert_eq!(
        content.state(),
        DxfTextSymbolValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        directory.card_for_role(
            records[0].record().ordinal(),
            DxfTextSymbolValueRole::BackgroundFill
        ),
        None
    );

    let height = card(
        &directory,
        records[0].record().ordinal(),
        DxfTextSymbolValueRole::TextHeight,
    )?;
    assert_eq!(height.state(), DxfTextSymbolValueCardState::Unique);
    let member = directory
        .members_for_card(height.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or(io::Error::other("height member"))?;
    assert!(matches!(
        directory
            .value_for_member(member)
            .ok_or(io::Error::other("height value"))?
            .data(),
        DxfTextSymbolValueData::Double(Err(_))
    ));

    let chunks = card(
        &directory,
        records[1].record().ordinal(),
        DxfTextSymbolValueRole::AdditionalContent,
    )?;
    assert_eq!(
        chunks.state(),
        DxfTextSymbolValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    let rotation = card(
        &directory,
        records[1].record().ordinal(),
        DxfTextSymbolValueRole::RotationOrColumnHeight,
    )?;
    assert_eq!(rotation.state(), DxfTextSymbolValueCardState::Unique);
    let rotation_member = directory
        .members_for_card(rotation.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or(io::Error::other("rotation member"))?;
    assert!(matches!(
        directory
            .value_for_member(rotation_member)
            .ok_or(io::Error::other("rotation value"))?
            .data(),
        DxfTextSymbolValueData::Double(Err(_))
    ));
    Ok(())
}

#[test]
fn cancellation_source_identity_lookups_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.text_symbol_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.text_symbol_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_record(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    assert_copy::<DxfTextSymbolValueCard>();
    assert_copy::<DxfTextSymbolCardMember>();
    assert_send_sync::<DxfTextSymbolCardDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfTextSymbolCardDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 75);
    assert_eq!(directory.members().len(), 10);
    let records = directory.evidence_directory().records();
    let expected = [
        (DxfTextSymbolKind::Text, 19),
        (DxfTextSymbolKind::MText, 33),
        (DxfTextSymbolKind::Shape, 12),
        (DxfTextSymbolKind::Tolerance, 11),
    ];
    for (record, (kind, count)) in records.iter().zip(expected) {
        assert_eq!(record.kind(), kind);
        assert_eq!(
            directory
                .cards_for_raw_record(record.record().ordinal())
                .ok_or(io::Error::other("record cards"))?
                .len(),
            count
        );
    }
    let text_content = card(
        directory,
        records[0].record().ordinal(),
        DxfTextSymbolValueRole::Content,
    )?;
    assert_eq!(
        text_content.state(),
        DxfTextSymbolValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    let mtext_rotation = card(
        directory,
        records[1].record().ordinal(),
        DxfTextSymbolValueRole::RotationOrColumnHeight,
    )?;
    assert_eq!(
        mtext_rotation.state(),
        DxfTextSymbolValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    Ok(())
}

fn card(
    directory: &DxfTextSymbolCardDirectory,
    raw: u64,
    role: DxfTextSymbolValueRole,
) -> Result<DxfTextSymbolValueCard, Box<dyn Error>> {
    directory
        .card_for_role(raw, role)
        .ok_or_else(|| io::Error::other("card").into())
}

fn signatures(directory: &DxfTextSymbolCardDirectory) -> Result<Vec<CardSignature>, io::Error> {
    directory
        .cards()
        .iter()
        .copied()
        .map(|card| {
            let values = directory
                .members_for_card(card.ordinal())
                .ok_or(io::Error::other("members"))?
                .iter()
                .map(|member| {
                    let value = directory
                        .value_for_member(*member)
                        .ok_or(io::Error::other("value"))?;
                    Ok(match value.data() {
                        DxfTextSymbolValueData::Text => ValueSignature::Text,
                        DxfTextSymbolValueData::Double(value) => ValueSignature::Double(
                            value.map_err(|_| io::Error::other("double"))?.to_bits(),
                        ),
                        DxfTextSymbolValueData::Int16(value) => {
                            ValueSignature::Int16(value.map_err(|_| io::Error::other("i16"))?)
                        }
                        DxfTextSymbolValueData::Int32(value) => {
                            ValueSignature::Int32(value.map_err(|_| io::Error::other("i32"))?)
                        }
                        _ => return Err(io::Error::other("unknown data")),
                    })
                })
                .collect::<Result<Vec<_>, io::Error>>()?;
            Ok((card.record().kind(), card.role(), card.state(), values))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n1\nA\n1\nB\n40\n2\n\
0\nMTEXT\n3\nfirst\n3\nsecond\n50\n0.25\n50\n0.5\n90\n1\n\
0\nSHAPE\n2\nBOLT\n\
0\nTOLERANCE\n3\nISO-25\n\
0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"TEXT")?;
    push_string(&mut bytes, version, 1, b"A")?;
    push_string(&mut bytes, version, 1, b"B")?;
    push_double(&mut bytes, version, 40, 2.0)?;
    push_string(&mut bytes, version, 0, b"MTEXT")?;
    push_string(&mut bytes, version, 3, b"first")?;
    push_string(&mut bytes, version, 3, b"second")?;
    push_double(&mut bytes, version, 50, 0.25)?;
    push_double(&mut bytes, version, 50, 0.5)?;
    push_i32(&mut bytes, version, 90, 1)?;
    push_string(&mut bytes, version, 0, b"SHAPE")?;
    push_string(&mut bytes, version, 2, b"BOLT")?;
    push_string(&mut bytes, version, 0, b"TOLERANCE")?;
    push_string(&mut bytes, version, 3, b"ISO-25")?;
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

fn push_i32(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i32) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("code"))?);
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

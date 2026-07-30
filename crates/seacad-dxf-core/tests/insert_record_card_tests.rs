use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInsertRecordCardDirectory, DxfInsertRecordCardMember,
    DxfInsertRecordCardMemberRange, DxfInsertRecordValueCard, DxfInsertRecordValueCardState,
    DxfInsertRecordValueData, DxfInsertRecordValueRole, DxfMemorySource, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

const ROLE_ORDER: [DxfInsertRecordValueRole; 16] = [
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

#[derive(Debug, Eq, PartialEq)]
enum ValueEvidence {
    Text(Vec<u8>),
    Double(u64),
    Int16(i16),
}

type CardEvidence = (
    u64,
    DxfInsertRecordValueRole,
    DxfInsertRecordValueCardState,
    Vec<ValueEvidence>,
);

#[test]
fn every_dialect_has_ascii_binary_insert_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_record_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_record_card_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            card_evidence(DxfRawDocumentView::from(&ascii), &ascii_directory)?,
            card_evidence(DxfRawDocumentView::from(&binary), &binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn cardinality_lexical_validity_and_record_locality_are_independent() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nA\n2\nB\n10\nbad\n10\nworse\n66\nbad\n102\n{APP\n41\n9\n102\n}\n0\nINSERT\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_record_card_directory(&DxfCancellationToken::default())?;
    let records = directory.evidence_directory().records();
    assert_eq!(records.len(), 2);
    assert_eq!(directory.cards().len(), 32);
    assert_eq!(directory.members().len(), 5);

    let first_ordinal = records[0].record().ordinal();
    for role in [
        DxfInsertRecordValueRole::BlockName,
        DxfInsertRecordValueRole::InsertionPointX,
    ] {
        let card = directory
            .card_for_role(first_ordinal, role)
            .ok_or(io::Error::other("multiple card"))?;
        assert_eq!(
            card.state(),
            DxfInsertRecordValueCardState::Multiple {
                occurrence_count: 2
            }
        );
        assert!(
            directory
                .members_for_card(card.ordinal())
                .ok_or(io::Error::other("multiple members"))?
                .windows(2)
                .all(|pair| pair[0].value_ordinal() < pair[1].value_ordinal())
        );
    }

    let point_x = directory
        .card_for_role(first_ordinal, DxfInsertRecordValueRole::InsertionPointX)
        .ok_or(io::Error::other("point x card"))?;
    for member in directory
        .members_for_card(point_x.ordinal())
        .ok_or(io::Error::other("point x members"))?
    {
        assert!(
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("point x value"))?
                .value()
                .is_err()
        );
    }

    let attributes = directory
        .card_for_role(first_ordinal, DxfInsertRecordValueRole::AttributesFollow)
        .ok_or(io::Error::other("attributes card"))?;
    assert_eq!(attributes.state(), DxfInsertRecordValueCardState::Unique);
    let [member] = directory
        .members_for_card(attributes.ordinal())
        .ok_or(io::Error::other("attributes member"))?
    else {
        return Err(io::Error::other("one attributes member").into());
    };
    assert!(
        directory
            .value_for_member(*member)
            .ok_or(io::Error::other("attributes value"))?
            .value()
            .is_err()
    );

    let scale_x = directory
        .card_for_role(first_ordinal, DxfInsertRecordValueRole::ScaleFactorX)
        .ok_or(io::Error::other("scale x card"))?;
    assert_eq!(scale_x.state(), DxfInsertRecordValueCardState::Absent);

    let second_ordinal = records[1].record().ordinal();
    assert!(
        directory
            .cards_for_raw_record(second_ordinal)
            .ok_or(io::Error::other("empty insert cards"))?
            .iter()
            .all(|card| card.state() == DxfInsertRecordValueCardState::Absent)
    );
    Ok(())
}

#[test]
fn cancellation_lookups_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_record_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.insert_record_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_record(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    for card in directory.cards() {
        assert_eq!(directory.card(card.ordinal()), Some(*card));
        for member in directory
            .members_for_card(card.ordinal())
            .ok_or(io::Error::other("card members"))?
        {
            assert!(directory.value_for_member(*member).is_some());
        }
    }
    assert_copy::<DxfInsertRecordValueCard>();
    assert_copy::<DxfInsertRecordCardMember>();
    assert_copy::<DxfInsertRecordCardMemberRange>();
    assert_send_sync::<DxfInsertRecordCardDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfInsertRecordCardDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 32);
    assert_eq!(directory.members().len(), 17);
    let records = directory.evidence_directory().records();
    assert_eq!(records.len(), 2);

    let first_cards = directory
        .cards_for_raw_record(records[0].record().ordinal())
        .ok_or(io::Error::other("first insert cards"))?;
    assert_eq!(
        first_cards
            .iter()
            .map(|card| card.role())
            .collect::<Vec<_>>(),
        ROLE_ORDER
    );
    assert!(
        first_cards
            .iter()
            .all(|card| card.state() == DxfInsertRecordValueCardState::Unique)
    );

    let second_cards = directory
        .cards_for_raw_record(records[1].record().ordinal())
        .ok_or(io::Error::other("second insert cards"))?;
    assert_eq!(
        second_cards
            .iter()
            .map(|card| card.role())
            .collect::<Vec<_>>(),
        ROLE_ORDER
    );
    assert_eq!(
        second_cards[0].state(),
        DxfInsertRecordValueCardState::Unique
    );
    assert!(
        second_cards[1..]
            .iter()
            .all(|card| card.state() == DxfInsertRecordValueCardState::Absent)
    );
    Ok(())
}

fn card_evidence(
    view: DxfRawDocumentView<'_>,
    directory: &DxfInsertRecordCardDirectory,
) -> Result<Vec<CardEvidence>, io::Error> {
    directory
        .cards()
        .iter()
        .copied()
        .map(|card| {
            let values = directory
                .members_for_card(card.ordinal())
                .ok_or(io::Error::other("card members"))?
                .iter()
                .copied()
                .map(|member| {
                    match directory
                        .value_for_member(member)
                        .ok_or(io::Error::other("card value"))?
                        .value()
                        .map_err(|_| io::Error::other("card data"))?
                    {
                        DxfInsertRecordValueData::Text(text) => {
                            let length = usize::try_from(text.value_span().len())
                                .map_err(io::Error::other)?;
                            let mut bytes = vec![0_u8; length];
                            view.read_span(text.value_span(), &mut bytes)
                                .map_err(io::Error::other)?;
                            Ok(ValueEvidence::Text(bytes))
                        }
                        DxfInsertRecordValueData::Double(value) => {
                            Ok(ValueEvidence::Double(value.to_bits()))
                        }
                        DxfInsertRecordValueData::Int16(value) => Ok(ValueEvidence::Int16(value)),
                        _ => Err(io::Error::other("unknown card data")),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                card.record().record().ordinal(),
                card.role(),
                card.state(),
                values,
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nOwner\n3\nOwner\n0\nINSERT\n2\nRef\n10\n1\n20\n2\n30\n3\n41\n4\n42\n5\n43\n6\n50\n7\n70\n8\n71\n9\n44\n10\n45\n11\n66\n1\n210\n12\n220\n13\n230\n14\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nRef\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section_start(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section_start(&mut bytes, version, b"BLOCKS")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 2, b"Owner")?;
    push_string(&mut bytes, version, 3, b"Owner")?;
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
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section_start(&mut bytes, version, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"Ref")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section_start(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

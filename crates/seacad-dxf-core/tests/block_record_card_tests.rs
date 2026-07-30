use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockDefinitionState, DxfBlockRecordCardDirectory, DxfBlockRecordCardMember,
    DxfBlockRecordCardMemberRange, DxfBlockRecordValueCard, DxfBlockRecordValueCardState,
    DxfBlockRecordValueData, DxfBlockRecordValueRole, DxfByteSource, DxfCancellationToken,
    DxfError, DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

const ROLE_ORDER: [DxfBlockRecordValueRole; 8] = [
    DxfBlockRecordValueRole::PrimaryName,
    DxfBlockRecordValueRole::Flags,
    DxfBlockRecordValueRole::BasePointX,
    DxfBlockRecordValueRole::BasePointY,
    DxfBlockRecordValueRole::BasePointZ,
    DxfBlockRecordValueRole::SecondaryName,
    DxfBlockRecordValueRole::XrefPath,
    DxfBlockRecordValueRole::Description,
];

#[derive(Debug, Eq, PartialEq)]
enum ValueEvidence {
    Text(Vec<u8>),
    Double(u64),
    Int16(i16),
}

type CardEvidence = (
    u64,
    DxfBlockRecordValueRole,
    DxfBlockRecordValueCardState,
    Vec<ValueEvidence>,
);

#[test]
fn every_dialect_has_ascii_binary_block_record_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.block_record_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.block_record_card_directory(&DxfCancellationToken::default())?;

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
fn cardinality_lexical_validity_and_record_locality_remain_independent()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n2\nB\n10\n.\n10\n1e-9999\n70\n32768\n102\n{APP\n4\nhidden\n102\n}\n0\nLINE\n4\nmember\n0\nENDBLK\n4\nboundary\n0\nBLOCK\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_record_card_directory(&DxfCancellationToken::default())?;
    let records = directory.evidence_directory().records();
    assert_eq!(records.len(), 2);
    assert_eq!(directory.members().len(), 5);

    let first_ordinal = records[0].definition().block_record().ordinal();
    for role in [
        DxfBlockRecordValueRole::PrimaryName,
        DxfBlockRecordValueRole::BasePointX,
    ] {
        let card = directory
            .card_for_role(first_ordinal, role)
            .ok_or(io::Error::other("multiple card"))?;
        assert_eq!(
            card.state(),
            DxfBlockRecordValueCardState::Multiple {
                occurrence_count: 2
            }
        );
    }

    let base_x = directory
        .card_for_role(first_ordinal, DxfBlockRecordValueRole::BasePointX)
        .ok_or(io::Error::other("base x card"))?;
    for member in directory
        .members_for_card(base_x.ordinal())
        .ok_or(io::Error::other("base x members"))?
    {
        assert!(
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("base x value"))?
                .value()
                .is_err()
        );
    }

    let flags = directory
        .card_for_role(first_ordinal, DxfBlockRecordValueRole::Flags)
        .ok_or(io::Error::other("flags card"))?;
    assert_eq!(flags.state(), DxfBlockRecordValueCardState::Unique);
    let [flag_member] = directory
        .members_for_card(flags.ordinal())
        .ok_or(io::Error::other("flags member"))?
    else {
        return Err(io::Error::other("one flags member").into());
    };
    assert!(
        directory
            .value_for_member(*flag_member)
            .ok_or(io::Error::other("flags value"))?
            .value()
            .is_err()
    );

    let description = directory
        .card_for_role(first_ordinal, DxfBlockRecordValueRole::Description)
        .ok_or(io::Error::other("description card"))?;
    assert_eq!(description.state(), DxfBlockRecordValueCardState::Absent);

    let second_ordinal = records[1].definition().block_record().ordinal();
    assert!(
        directory
            .cards_for_block_raw_ordinal(second_ordinal)
            .ok_or(io::Error::other("empty block cards"))?
            .iter()
            .all(|card| card.state() == DxfBlockRecordValueCardState::Absent)
    );
    Ok(())
}

#[test]
fn cards_remain_available_for_every_definition_state() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n70\n1\n0\nLINE\n0\nBLOCK\n70\n2\n0\nLINE\n0\nENDBLK\n0\nBLOCK\n70\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_record_card_directory(&DxfCancellationToken::default())?;
    let expected_states = [
        DxfBlockDefinitionState::Interrupted,
        DxfBlockDefinitionState::Closed,
        DxfBlockDefinitionState::Unclosed,
    ];
    let records = directory.evidence_directory().records();
    assert_eq!(records.len(), 3);
    for (record, state) in records.iter().zip(expected_states) {
        assert_eq!(record.definition().state(), state);
        let flags = directory
            .card_for_role(
                record.definition().block_record().ordinal(),
                DxfBlockRecordValueRole::Flags,
            )
            .ok_or(io::Error::other("flags card"))?;
        assert_eq!(flags.state(), DxfBlockRecordValueCardState::Unique);
    }
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
        document.block_record_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.block_record_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_block_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    assert_copy::<DxfBlockRecordValueCard>();
    assert_copy::<DxfBlockRecordCardMember>();
    assert_copy::<DxfBlockRecordCardMemberRange>();
    assert_send_sync::<DxfBlockRecordCardDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfBlockRecordCardDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 16);
    assert_eq!(directory.members().len(), 10);
    let records = directory.evidence_directory().records();
    assert_eq!(records.len(), 2);

    let first_cards = directory
        .cards_for_block_raw_ordinal(records[0].definition().block_record().ordinal())
        .ok_or(io::Error::other("first block cards"))?;
    assert_eq!(
        first_cards
            .iter()
            .map(|card| card.role())
            .collect::<Vec<_>>(),
        ROLE_ORDER
    );
    assert_eq!(
        first_cards[0].state(),
        DxfBlockRecordValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert!(
        first_cards[1..]
            .iter()
            .all(|card| card.state() == DxfBlockRecordValueCardState::Absent)
    );

    let second_cards = directory
        .cards_for_block_raw_ordinal(records[1].definition().block_record().ordinal())
        .ok_or(io::Error::other("second block cards"))?;
    assert_eq!(
        second_cards
            .iter()
            .map(|card| card.role())
            .collect::<Vec<_>>(),
        ROLE_ORDER
    );
    assert!(
        second_cards
            .iter()
            .all(|card| card.state() == DxfBlockRecordValueCardState::Unique)
    );
    Ok(())
}

fn card_evidence(
    view: DxfRawDocumentView<'_>,
    directory: &DxfBlockRecordCardDirectory,
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
                        DxfBlockRecordValueData::Text(text) => {
                            let length = usize::try_from(text.value_span().len())
                                .map_err(io::Error::other)?;
                            let mut bytes = vec![0_u8; length];
                            view.read_span(text.value_span(), &mut bytes)
                                .map_err(io::Error::other)?;
                            Ok(ValueEvidence::Text(bytes))
                        }
                        DxfBlockRecordValueData::Double(value) => {
                            Ok(ValueEvidence::Double(value.to_bits()))
                        }
                        DxfBlockRecordValueData::Int16(value) => Ok(ValueEvidence::Int16(value)),
                        _ => Err(io::Error::other("unknown card data")),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                card.record().definition().block_record().ordinal(),
                card.role(),
                card.state(),
                values,
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n2\nB\n0\nENDBLK\n0\nBLOCK\n2\nFULL\n70\n64\n10\n-0\n20\n1.25\n30\n-2.5\n3\nFULL\n1\nxref.dxf\n4\ndescription\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 2, b"BLOCKS")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 2, b"A")?;
    push_string(&mut bytes, version, 2, b"B")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 2, b"FULL")?;
    push_i16(&mut bytes, version, 70, 64)?;
    push_double(&mut bytes, version, 10, -0.0)?;
    push_double(&mut bytes, version, 20, 1.25)?;
    push_double(&mut bytes, version, 30, -2.5)?;
    push_string(&mut bytes, version, 3, b"FULL")?;
    push_string(&mut bytes, version, 1, b"xref.dxf")?;
    push_string(&mut bytes, version, 4, b"description")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInsertAttributeCardDirectory, DxfInsertAttributeCardMember,
    DxfInsertAttributeCardMemberRange, DxfInsertAttributeValueCard,
    DxfInsertAttributeValueCardState, DxfInsertAttributeValueRole, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

const ROLE_ORDER: [DxfInsertAttributeValueRole; 23] = [
    DxfInsertAttributeValueRole::Thickness,
    DxfInsertAttributeValueRole::TextStartX,
    DxfInsertAttributeValueRole::TextStartY,
    DxfInsertAttributeValueRole::TextStartZ,
    DxfInsertAttributeValueRole::TextHeight,
    DxfInsertAttributeValueRole::TextValue,
    DxfInsertAttributeValueRole::AttributeTag,
    DxfInsertAttributeValueRole::AttributeFlags,
    DxfInsertAttributeValueRole::FieldLength,
    DxfInsertAttributeValueRole::RotationAngle,
    DxfInsertAttributeValueRole::RelativeXScale,
    DxfInsertAttributeValueRole::ObliqueAngle,
    DxfInsertAttributeValueRole::TextStyleName,
    DxfInsertAttributeValueRole::TextGenerationFlags,
    DxfInsertAttributeValueRole::HorizontalJustification,
    DxfInsertAttributeValueRole::VerticalJustification,
    DxfInsertAttributeValueRole::AlignmentPointX,
    DxfInsertAttributeValueRole::AlignmentPointY,
    DxfInsertAttributeValueRole::AlignmentPointZ,
    DxfInsertAttributeValueRole::ExtrusionX,
    DxfInsertAttributeValueRole::ExtrusionY,
    DxfInsertAttributeValueRole::ExtrusionZ,
    DxfInsertAttributeValueRole::VersionOrLockPosition,
];

#[test]
fn every_supported_dialect_has_ascii_binary_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_card_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory, version)?;
        assert_directory(&binary_directory, version)?;
        assert_eq!(
            card_summary(&ascii_directory),
            card_summary(&binary_directory)
        );
    }
    Ok(())
}

#[test]
fn lexical_validity_cardinality_and_record_locality_are_independent() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n10\nbad\n10\nworse\n2\na\n2\nb\n40\nbad\n0\nATTRIB\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_attribute_card_directory(&DxfCancellationToken::default())?;
    let records = directory.evidence_directory().records();
    assert_eq!(records.len(), 2);
    assert_eq!(directory.cards().len(), 46);
    assert_eq!(directory.members().len(), 5);

    let first = records[0].record().ordinal();
    for role in [
        DxfInsertAttributeValueRole::TextStartX,
        DxfInsertAttributeValueRole::AttributeTag,
    ] {
        let card = directory
            .card_for_role(first, role)
            .ok_or_else(invalid_test_data)?;
        assert_eq!(
            card.state(),
            DxfInsertAttributeValueCardState::Multiple {
                occurrence_count: 2,
            }
        );
        let members = directory
            .members_for_card(card.ordinal())
            .ok_or_else(invalid_test_data)?;
        assert!(
            members
                .windows(2)
                .all(|pair| pair[0].value_ordinal() < pair[1].value_ordinal())
        );
    }
    let start_x = directory
        .card_for_role(first, DxfInsertAttributeValueRole::TextStartX)
        .ok_or_else(invalid_test_data)?;
    for member in directory
        .members_for_card(start_x.ordinal())
        .ok_or_else(invalid_test_data)?
    {
        assert!(
            directory
                .value_for_member(*member)
                .ok_or_else(invalid_test_data)?
                .value()
                .is_err()
        );
    }
    let height = directory
        .card_for_role(first, DxfInsertAttributeValueRole::TextHeight)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(height.state(), DxfInsertAttributeValueCardState::Unique);
    let second = records[1].record().ordinal();
    assert!(
        directory
            .cards_for_raw_record(second)
            .ok_or_else(invalid_test_data)?
            .iter()
            .all(|card| card.state() == DxfInsertAttributeValueCardState::Absent)
    );
    Ok(())
}

#[test]
fn cancellation_lookups_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeValueCard>();
    assert_copy::<DxfInsertAttributeCardMember>();
    assert_copy::<DxfInsertAttributeCardMemberRange>();
    assert_send_sync::<DxfInsertAttributeCardDirectory>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.insert_attribute_card_directory(&DxfCancellationToken::default())?;
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
            .ok_or_else(invalid_test_data)?
        {
            assert!(directory.value_for_member(*member).is_some());
        }
    }
    Ok(())
}

fn assert_directory(
    directory: &DxfInsertAttributeCardDirectory,
    version: DxfAcadVersion,
) -> Result<(), DxfError> {
    assert_eq!(directory.cards().len(), 46);
    assert_eq!(directory.evidence_directory().records().len(), 2);
    let first = directory.evidence_directory().records()[0]
        .record()
        .ordinal();
    let first_cards = directory
        .cards_for_raw_record(first)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        first_cards
            .iter()
            .map(|card| card.role())
            .collect::<Vec<_>>(),
        ROLE_ORDER
    );
    let expected_members = if version == DxfAcadVersion::Ac1009 {
        22
    } else {
        24
    };
    assert_eq!(directory.members().len(), expected_members);
    for card in &first_cards[..22] {
        assert_eq!(card.state(), DxfInsertAttributeValueCardState::Unique);
    }
    assert_eq!(
        first_cards[22].state(),
        if version == DxfAcadVersion::Ac1009 {
            DxfInsertAttributeValueCardState::Absent
        } else {
            DxfInsertAttributeValueCardState::Multiple {
                occurrence_count: 2,
            }
        }
    );
    let second = directory.evidence_directory().records()[1]
        .record()
        .ordinal();
    assert!(
        directory
            .cards_for_raw_record(second)
            .ok_or_else(invalid_test_data)?
            .iter()
            .all(|card| card.state() == DxfInsertAttributeValueCardState::Absent)
    );
    Ok(())
}

fn card_summary(
    directory: &DxfInsertAttributeCardDirectory,
) -> Vec<(
    DxfInsertAttributeValueRole,
    DxfInsertAttributeValueCardState,
    u64,
)> {
    directory
        .cards()
        .iter()
        .map(|card| (card.role(), card.state(), card.member_range().len()))
        .collect()
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let version_lock = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "280\n0\n280\n1\n".to_owned()
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n39\n1\n10\n2\n20\n3\n30\n4\n40\n5\n1\nvalue\n2\ntag\n70\n9\n73\n10\n50\n11\n41\n12\n51\n13\n7\nSTYLE\n71\n2\n72\n3\n74\n1\n11\n14\n21\n15\n31\n16\n210\n0\n220\n0\n230\n1\n{}0\nATTRIB\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        version_lock,
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
    for (code, value) in [(39, 1.0), (10, 2.0), (20, 3.0), (30, 4.0), (40, 5.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 1, b"value")?;
    push_string(&mut bytes, version, 2, b"tag")?;
    push_i16(&mut bytes, version, 70, 9)?;
    push_i16(&mut bytes, version, 73, 10)?;
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
    if version != DxfAcadVersion::Ac1009 {
        push_i16(&mut bytes, version, 280, 0)?;
        push_i16(&mut bytes, version, 280, 1)?;
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

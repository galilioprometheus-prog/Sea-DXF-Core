use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfLightweightPolylineInteger,
    DxfLightweightPolylineRecordCard, DxfLightweightPolylineRecordCardDirectory,
    DxfLightweightPolylineRecordCardEntry, DxfLightweightPolylineRecordCardMember,
    DxfLightweightPolylineRecordCardState, DxfLightweightPolylineRecordRole, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

const ROLE_ORDER: [DxfLightweightPolylineRecordRole; 8] = [
    DxfLightweightPolylineRecordRole::VertexCount,
    DxfLightweightPolylineRecordRole::Flags,
    DxfLightweightPolylineRecordRole::OcsElevation,
    DxfLightweightPolylineRecordRole::Thickness,
    DxfLightweightPolylineRecordRole::ConstantWidth,
    DxfLightweightPolylineRecordRole::ExtrusionX,
    DxfLightweightPolylineRecordRole::ExtrusionY,
    DxfLightweightPolylineRecordRole::ExtrusionZ,
];

#[derive(Debug, Eq, PartialEq)]
enum NumericEvidence {
    Floating(u64),
    Integer(DxfLightweightPolylineInteger),
}

type CardEvidence = (
    DxfLightweightPolylineRecordRole,
    DxfLightweightPolylineRecordCardState,
    Vec<NumericEvidence>,
);

#[test]
fn every_supported_dialect_has_ascii_binary_record_card_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.lightweight_polyline_record_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.lightweight_polyline_record_card_directory(&DxfCancellationToken::default())?;

        assert_valid_directory(&ascii_directory)?;
        assert_valid_directory(&binary_directory)?;
        assert_eq!(
            card_evidence(&ascii_directory)?,
            card_evidence(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn record_cardinality_is_independent_from_lexical_validity_and_vertex_fields()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n.\n90\n2\n70\n.\n38\n.\n39\n.\n39\n1\n210\n.\n10\n1\n20\n2\n40\n3\n41\n4\n42\n5\n91\n6\n0\nLWPOLYLINE\n10\n7\n20\n8\n40\n9\n41\n10\n42\n11\n91\n12\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.lightweight_polyline_record_card_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    let first = directory.records()[0];
    assert_state(
        &directory,
        first,
        DxfLightweightPolylineRecordRole::VertexCount,
        DxfLightweightPolylineRecordCardState::Multiple {
            occurrence_count: 2,
        },
    )?;
    assert_state(
        &directory,
        first,
        DxfLightweightPolylineRecordRole::Flags,
        DxfLightweightPolylineRecordCardState::Unique,
    )?;
    assert_state(
        &directory,
        first,
        DxfLightweightPolylineRecordRole::OcsElevation,
        DxfLightweightPolylineRecordCardState::Unique,
    )?;
    assert_state(
        &directory,
        first,
        DxfLightweightPolylineRecordRole::Thickness,
        DxfLightweightPolylineRecordCardState::Multiple {
            occurrence_count: 2,
        },
    )?;

    for role in [
        DxfLightweightPolylineRecordRole::ConstantWidth,
        DxfLightweightPolylineRecordRole::ExtrusionY,
        DxfLightweightPolylineRecordRole::ExtrusionZ,
    ] {
        assert_state(
            &directory,
            first,
            role,
            DxfLightweightPolylineRecordCardState::Absent,
        )?;
    }

    let count = card(
        &directory,
        first,
        DxfLightweightPolylineRecordRole::VertexCount,
    )?;
    let count_members = directory
        .members_for_card(count)
        .ok_or(io::Error::other("count members"))?;
    assert!(
        directory
            .integer_value_for_member(count_members[0])
            .ok_or(io::Error::other("invalid count evidence"))?
            .value()
            .is_err()
    );
    assert!(
        directory
            .floating_value_for_member(count_members[0])
            .is_none()
    );

    let elevation = card(
        &directory,
        first,
        DxfLightweightPolylineRecordRole::OcsElevation,
    )?;
    let elevation_member = directory
        .members_for_card(elevation)
        .and_then(|members| members.first())
        .copied()
        .ok_or(io::Error::other("elevation member"))?;
    assert!(
        directory
            .floating_value_for_member(elevation_member)
            .ok_or(io::Error::other("invalid elevation evidence"))?
            .value()
            .is_err()
    );
    assert!(
        directory
            .integer_value_for_member(elevation_member)
            .is_none()
    );

    let second = directory.records()[1];
    let second_cards = directory
        .cards_for_raw_record(second.record().ordinal())
        .ok_or(io::Error::other("second record cards"))?;
    assert_eq!(second_cards.len(), 8);
    assert!(
        second_cards
            .iter()
            .all(|card| card.state() == DxfLightweightPolylineRecordCardState::Absent)
    );
    assert_eq!(directory.members().len(), 7);
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.lightweight_polyline_record_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.lightweight_polyline_record_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.card_for_role(u64::MAX, DxfLightweightPolylineRecordRole::Flags),
        None
    );
    assert_copy::<DxfLightweightPolylineRecordCardEntry>();
    assert_copy::<DxfLightweightPolylineRecordCard>();
    assert_copy::<DxfLightweightPolylineRecordCardMember>();
    assert_send_sync::<DxfLightweightPolylineRecordCardDirectory>();
    Ok(())
}

fn assert_valid_directory(
    directory: &DxfLightweightPolylineRecordCardDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.cards().len(), 8);
    assert_eq!(directory.members().len(), 8);
    assert_eq!(
        directory.source_id(),
        directory.floating_evidence_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.integer_evidence_directory().source_id()
    );

    let record = directory.records()[0];
    assert_eq!(record.first_card_ordinal(), 0);
    let cards = directory
        .cards_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("record cards"))?;
    assert_eq!(
        cards.iter().map(|card| card.role()).collect::<Vec<_>>(),
        ROLE_ORDER
    );
    assert!(
        cards
            .iter()
            .all(|card| card.state() == DxfLightweightPolylineRecordCardState::Unique)
    );
    for card in cards.iter().copied() {
        let members = directory
            .members_for_card(card)
            .ok_or(io::Error::other("card members"))?;
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].role(), card.role());
    }
    Ok(())
}

fn assert_state(
    directory: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
    role: DxfLightweightPolylineRecordRole,
    expected: DxfLightweightPolylineRecordCardState,
) -> Result<(), io::Error> {
    assert_eq!(card(directory, record, role)?.state(), expected);
    Ok(())
}

fn card(
    directory: &DxfLightweightPolylineRecordCardDirectory,
    record: DxfLightweightPolylineRecordCardEntry,
    role: DxfLightweightPolylineRecordRole,
) -> Result<DxfLightweightPolylineRecordCard, io::Error> {
    directory
        .card_for_role(record.record().ordinal(), role)
        .ok_or(io::Error::other("record role card"))
}

fn card_evidence(
    directory: &DxfLightweightPolylineRecordCardDirectory,
) -> Result<Vec<CardEvidence>, io::Error> {
    directory
        .cards()
        .iter()
        .copied()
        .map(|card| {
            let mut values = Vec::new();
            for member in directory
                .members_for_card(card)
                .ok_or(io::Error::other("card members"))?
            {
                if let Some(value) = directory.floating_value_for_member(*member) {
                    values.push(NumericEvidence::Floating(
                        value
                            .value()
                            .map_err(|_| io::Error::other("floating value"))?
                            .to_bits(),
                    ));
                } else {
                    values.push(NumericEvidence::Integer(
                        directory
                            .integer_value_for_member(*member)
                            .ok_or(io::Error::other("integer member"))?
                            .value()
                            .map_err(|_| io::Error::other("integer value"))?,
                    ));
                }
            }
            Ok((card.role(), card.state(), values))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n70\n129\n38\n-0\n39\n0.5\n43\n1.5\n10\n1\n20\n2\n40\n0.25\n41\n0.5\n42\n-0\n91\n7\n210\n0\n220\n-0\n230\n1\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"LWPOLYLINE")?;
    push_i32(&mut bytes, version, 90, 2)?;
    push_i16(&mut bytes, version, 70, 129)?;
    for (code, value) in [
        (38, -0.0),
        (39, 0.5),
        (43, 1.5),
        (10, 1.0),
        (20, 2.0),
        (40, 0.25),
        (41, 0.5),
        (42, -0.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 7)?;
    for (code, value) in [(210, 0.0), (220, -0.0), (230, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
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

fn push_i32(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i32) -> io::Result<()> {
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

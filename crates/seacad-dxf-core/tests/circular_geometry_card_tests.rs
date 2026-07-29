use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfCircularGeometryCardDirectory, DxfCircularGeometryCardMember,
    DxfCircularGeometryKind, DxfCircularGeometryValueCard, DxfCircularGeometryValueCardState,
    DxfCircularGeometryValueRole, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

type CardEvidence = (
    DxfCircularGeometryKind,
    DxfCircularGeometryValueRole,
    DxfCircularGeometryValueCardState,
    Vec<u64>,
);

#[test]
fn every_dialect_has_ascii_binary_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_cards =
            ascii.circular_geometry_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_cards =
            binary.circular_geometry_card_directory(&DxfCancellationToken::default())?;

        assert_card_directory(&ascii_cards)?;
        assert_card_directory(&binary_cards)?;
        assert_eq!(card_evidence(&ascii_cards)?, card_evidence(&binary_cards)?);
    }
    Ok(())
}

#[test]
fn cardinality_and_lexical_validity_remain_independent() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n10\n.\n10\n1e-9999\n20\n2\n0\nARC\n50\n.\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.circular_geometry_card_directory(&DxfCancellationToken::default())?;
    let records = directory.evidence_directory().records();
    let circle = records[0];
    let arc = records[1];

    let circle_x = directory
        .card_for_role(
            circle.record().ordinal(),
            DxfCircularGeometryValueRole::OcsCenterX,
        )
        .ok_or(io::Error::other("circle x card"))?;
    assert_eq!(
        circle_x.state(),
        DxfCircularGeometryValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    for member in directory
        .members_for_card(circle_x.ordinal())
        .ok_or(io::Error::other("circle x members"))?
    {
        assert!(
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("circle x value"))?
                .value()
                .is_err()
        );
    }

    let circle_y = directory
        .card_for_role(
            circle.record().ordinal(),
            DxfCircularGeometryValueRole::OcsCenterY,
        )
        .ok_or(io::Error::other("circle y card"))?;
    assert_eq!(circle_y.state(), DxfCircularGeometryValueCardState::Unique);
    let circle_radius = directory
        .card_for_role(
            circle.record().ordinal(),
            DxfCircularGeometryValueRole::Radius,
        )
        .ok_or(io::Error::other("circle radius card"))?;
    assert_eq!(
        circle_radius.state(),
        DxfCircularGeometryValueCardState::Absent
    );
    assert!(
        directory
            .members_for_card(circle_radius.ordinal())
            .ok_or(io::Error::other("absent radius members"))?
            .is_empty()
    );

    let arc_start = directory
        .card_for_role(
            arc.record().ordinal(),
            DxfCircularGeometryValueRole::StartAngle,
        )
        .ok_or(io::Error::other("arc start card"))?;
    assert_eq!(arc_start.state(), DxfCircularGeometryValueCardState::Unique);
    assert!(
        directory
            .value_for_member(
                *directory
                    .members_for_card(arc_start.ordinal())
                    .and_then(|members| members.first())
                    .ok_or(io::Error::other("arc start member"))?
            )
            .ok_or(io::Error::other("arc start value"))?
            .value()
            .is_err()
    );
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_are_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.circular_geometry_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.circular_geometry_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_record(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    assert_copy::<DxfCircularGeometryValueCard>();
    assert_copy::<DxfCircularGeometryCardMember>();
    assert_send_sync::<DxfCircularGeometryCardDirectory>();
    Ok(())
}

fn assert_card_directory(
    directory: &DxfCircularGeometryCardDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 16);
    assert_eq!(directory.members().len(), 16);
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );

    let records = directory.evidence_directory().records();
    let circle = records[0];
    let arc = records[1];
    let circle_cards = directory
        .cards_for_raw_record(circle.record().ordinal())
        .ok_or(io::Error::other("circle cards"))?;
    let arc_cards = directory
        .cards_for_raw_record(arc.record().ordinal())
        .ok_or(io::Error::other("arc cards"))?;
    assert_eq!(circle_cards.len(), 7);
    assert_eq!(arc_cards.len(), 9);
    assert_eq!(
        circle_cards
            .iter()
            .map(|card| card.role())
            .collect::<Vec<_>>(),
        [
            DxfCircularGeometryValueRole::OcsCenterX,
            DxfCircularGeometryValueRole::OcsCenterY,
            DxfCircularGeometryValueRole::OcsCenterZ,
            DxfCircularGeometryValueRole::Radius,
            DxfCircularGeometryValueRole::ExtrusionX,
            DxfCircularGeometryValueRole::ExtrusionY,
            DxfCircularGeometryValueRole::ExtrusionZ,
        ]
    );
    assert_eq!(
        arc_cards.iter().map(|card| card.role()).collect::<Vec<_>>(),
        [
            DxfCircularGeometryValueRole::OcsCenterX,
            DxfCircularGeometryValueRole::OcsCenterY,
            DxfCircularGeometryValueRole::OcsCenterZ,
            DxfCircularGeometryValueRole::Radius,
            DxfCircularGeometryValueRole::StartAngle,
            DxfCircularGeometryValueRole::EndAngle,
            DxfCircularGeometryValueRole::ExtrusionX,
            DxfCircularGeometryValueRole::ExtrusionY,
            DxfCircularGeometryValueRole::ExtrusionZ,
        ]
    );

    let circle_x = directory
        .card_for_role(
            circle.record().ordinal(),
            DxfCircularGeometryValueRole::OcsCenterX,
        )
        .ok_or(io::Error::other("circle x card"))?;
    assert_eq!(
        circle_x.state(),
        DxfCircularGeometryValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    let circle_x_bits = directory
        .members_for_card(circle_x.ordinal())
        .ok_or(io::Error::other("circle x members"))?
        .iter()
        .map(|member| {
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("circle x member value"))?
                .value()
                .map(|value| value.to_bits())
                .map_err(|_| io::Error::other("circle x numeric value"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(circle_x_bits, [(-0.0_f64).to_bits(), 1.0_f64.to_bits()]);

    let circle_extrusion_y = directory
        .card_for_role(
            circle.record().ordinal(),
            DxfCircularGeometryValueRole::ExtrusionY,
        )
        .ok_or(io::Error::other("circle extrusion y card"))?;
    assert_eq!(
        circle_extrusion_y.state(),
        DxfCircularGeometryValueCardState::Absent
    );
    assert_eq!(
        directory.card_for_role(
            circle.record().ordinal(),
            DxfCircularGeometryValueRole::StartAngle
        ),
        None
    );
    assert!(
        arc_cards
            .iter()
            .all(|card| card.state() == DxfCircularGeometryValueCardState::Unique)
    );
    Ok(())
}

fn card_evidence(
    directory: &DxfCircularGeometryCardDirectory,
) -> Result<Vec<CardEvidence>, io::Error> {
    directory
        .cards()
        .iter()
        .copied()
        .map(|card| {
            let members = directory
                .members_for_card(card.ordinal())
                .ok_or(io::Error::other("card members"))?;
            let mut values = Vec::new();
            for member in members {
                values.push(
                    directory
                        .value_for_member(*member)
                        .ok_or(io::Error::other("card value"))?
                        .value()
                        .map_err(|_| io::Error::other("card numeric value"))?
                        .to_bits(),
                );
            }
            Ok((card.record().kind(), card.role(), card.state(), values))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n10\n-0\n10\n1\n20\n2\n30\n3\n40\n4\n210\n0\n230\n1\n0\nARC\n10\n1\n20\n2\n30\n3\n40\n4\n50\n0\n51\n270\n210\n0\n220\n1\n230\n0\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"CIRCLE")?;
    for (code, value) in [
        (10, -0.0),
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (40, 4.0),
        (210, 0.0),
        (230, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"ARC")?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (40, 4.0),
        (50, 0.0),
        (51, 270.0),
        (210, 0.0),
        (220, 1.0),
        (230, 0.0),
    ] {
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPlanarFaceCardDirectory,
    DxfPlanarFaceCardMember, DxfPlanarFaceKind, DxfPlanarFaceNumber, DxfPlanarFaceValueCard,
    DxfPlanarFaceValueCardState, DxfPlanarFaceValueRole, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NumberSignature {
    Double(u64),
    Int16(i16),
}

type CardSignature = (
    DxfPlanarFaceKind,
    DxfPlanarFaceValueRole,
    DxfPlanarFaceValueCardState,
    Vec<NumberSignature>,
);

#[test]
fn every_dialect_has_ascii_binary_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_cards = ascii.planar_face_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_cards = binary.planar_face_card_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_cards)?;
        assert_directory(&binary_cards)?;
        assert_eq!(
            card_signatures(&ascii_cards)?,
            card_signatures(&binary_cards)?
        );
    }
    Ok(())
}

#[test]
fn cardinality_lexical_validity_and_family_roles_are_independent() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\n3DFACE\n10\n.\n10\n1e-9999\n20\n2\n39\n4\n\
0\nSOLID\n10\n3\n70\n7\n\
0\nTRACE\n39\n.\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.planar_face_card_directory(&DxfCancellationToken::default())?;
    let records = directory.evidence_directory().records();
    let face = records[0];
    let solid = records[1];
    let trace = records[2];

    let face_x = directory
        .card_for_role(
            face.record().ordinal(),
            DxfPlanarFaceValueRole::FirstCornerX,
        )
        .ok_or(io::Error::other("3DFACE x card"))?;
    assert_eq!(
        face_x.state(),
        DxfPlanarFaceValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    for member in directory
        .members_for_card(face_x.ordinal())
        .ok_or(io::Error::other("3DFACE x members"))?
    {
        assert!(
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("3DFACE x value"))?
                .value()
                .is_err()
        );
    }
    assert_eq!(
        directory.card_for_role(face.record().ordinal(), DxfPlanarFaceValueRole::Thickness),
        None
    );
    assert_eq!(
        directory.card_for_role(
            solid.record().ordinal(),
            DxfPlanarFaceValueRole::InvisibleEdgeFlags
        ),
        None
    );

    let solid_thickness = directory
        .card_for_role(solid.record().ordinal(), DxfPlanarFaceValueRole::Thickness)
        .ok_or(io::Error::other("SOLID thickness card"))?;
    assert_eq!(solid_thickness.state(), DxfPlanarFaceValueCardState::Absent);
    assert!(
        directory
            .members_for_card(solid_thickness.ordinal())
            .ok_or(io::Error::other("SOLID thickness members"))?
            .is_empty()
    );

    let trace_thickness = directory
        .card_for_role(trace.record().ordinal(), DxfPlanarFaceValueRole::Thickness)
        .ok_or(io::Error::other("TRACE thickness card"))?;
    assert_eq!(trace_thickness.state(), DxfPlanarFaceValueCardState::Unique);
    let trace_member = directory
        .members_for_card(trace_thickness.ordinal())
        .and_then(|members| members.first())
        .copied()
        .ok_or(io::Error::other("TRACE thickness member"))?;
    assert!(
        directory
            .value_for_member(trace_member)
            .ok_or(io::Error::other("TRACE thickness value"))?
            .value()
            .is_err()
    );
    Ok(())
}

#[test]
fn cancellation_lookups_and_public_traits_are_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.planar_face_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.planar_face_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_record(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    assert_copy::<DxfPlanarFaceValueCard>();
    assert_copy::<DxfPlanarFaceCardMember>();
    assert_send_sync::<DxfPlanarFaceCardDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfPlanarFaceCardDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 45);
    assert_eq!(directory.members().len(), 9);
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );

    let records = directory.evidence_directory().records();
    let face_cards = directory
        .cards_for_raw_record(records[0].record().ordinal())
        .ok_or(io::Error::other("3DFACE cards"))?;
    let solid_cards = directory
        .cards_for_raw_record(records[1].record().ordinal())
        .ok_or(io::Error::other("SOLID cards"))?;
    let trace_cards = directory
        .cards_for_raw_record(records[2].record().ordinal())
        .ok_or(io::Error::other("TRACE cards"))?;
    assert_eq!(face_cards.len(), 13);
    assert_eq!(solid_cards.len(), 16);
    assert_eq!(trace_cards.len(), 16);
    assert_eq!(
        face_cards[12].role(),
        DxfPlanarFaceValueRole::InvisibleEdgeFlags
    );
    assert_eq!(solid_cards[12].role(), DxfPlanarFaceValueRole::Thickness);
    assert_eq!(solid_cards[15].role(), DxfPlanarFaceValueRole::ExtrusionZ);
    assert_eq!(
        face_cards[0].state(),
        DxfPlanarFaceValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(trace_cards[0].state(), DxfPlanarFaceValueCardState::Absent);
    Ok(())
}

fn card_signatures(
    directory: &DxfPlanarFaceCardDirectory,
) -> Result<Vec<CardSignature>, io::Error> {
    directory
        .cards()
        .iter()
        .copied()
        .map(|card| {
            let values = directory
                .members_for_card(card.ordinal())
                .ok_or(io::Error::other("card members"))?
                .iter()
                .map(|member| {
                    let value = directory
                        .value_for_member(*member)
                        .ok_or(io::Error::other("member value"))?
                        .value()
                        .map_err(|_| io::Error::other("numeric value"))?;
                    match value {
                        DxfPlanarFaceNumber::Double(value) => {
                            Ok(NumberSignature::Double(value.to_bits()))
                        }
                        DxfPlanarFaceNumber::Int16(value) => Ok(NumberSignature::Int16(value)),
                        _ => Err(io::Error::other("unknown numeric wire domain")),
                    }
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
0\n3DFACE\n10\n-0\n10\n1\n20\n2\n70\n9\n\
0\nSOLID\n10\n3\n39\n2.5\n210\n0\n\
0\nTRACE\n13\n4\n230\n1\n\
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
    push_string(&mut bytes, version, 0, b"3DFACE")?;
    push_double(&mut bytes, version, 10, -0.0)?;
    push_double(&mut bytes, version, 10, 1.0)?;
    push_double(&mut bytes, version, 20, 2.0)?;
    push_i16(&mut bytes, version, 70, 9)?;
    push_string(&mut bytes, version, 0, b"SOLID")?;
    push_double(&mut bytes, version, 10, 3.0)?;
    push_double(&mut bytes, version, 39, 2.5)?;
    push_double(&mut bytes, version, 210, 0.0)?;
    push_string(&mut bytes, version, 0, b"TRACE")?;
    push_double(&mut bytes, version, 13, 4.0)?;
    push_double(&mut bytes, version, 230, 1.0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_i16(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: i16,
) -> io::Result<()> {
    push_code(bytes, version, group_code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, group_code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
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

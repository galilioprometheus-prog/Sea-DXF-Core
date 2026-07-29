use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineSequenceState,
    DxfPolylineVertexCardDirectory, DxfPolylineVertexCardMember, DxfPolylineVertexCardMemberRange,
    DxfPolylineVertexNumber, DxfPolylineVertexValueCard, DxfPolylineVertexValueCardState,
    DxfPolylineVertexValueRole, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

const ROLE_ORDER: [DxfPolylineVertexValueRole; 13] = [
    DxfPolylineVertexValueRole::LocationX,
    DxfPolylineVertexValueRole::LocationY,
    DxfPolylineVertexValueRole::LocationZ,
    DxfPolylineVertexValueRole::StartWidth,
    DxfPolylineVertexValueRole::EndWidth,
    DxfPolylineVertexValueRole::Bulge,
    DxfPolylineVertexValueRole::CurveFitTangentDirection,
    DxfPolylineVertexValueRole::Flags,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex1,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex2,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex3,
    DxfPolylineVertexValueRole::PolyfaceVertexIndex4,
    DxfPolylineVertexValueRole::Identifier,
];

#[derive(Debug, Eq, PartialEq)]
enum NumberEvidence {
    Double(u64),
    Int16(i16),
    Int32(i32),
}

type CardEvidence = (
    u64,
    DxfPolylineVertexValueRole,
    DxfPolylineVertexValueCardState,
    Vec<NumberEvidence>,
);

#[test]
fn every_dialect_has_ascii_binary_vertex_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_vertex_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_vertex_card_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            card_evidence(&ascii_directory)?,
            card_evidence(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn cardinality_lexical_validity_and_record_locality_remain_independent()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n99\n0\nVERTEX\n10\n.\n10\n1e-9999\n70\n.\n91\n2147483648\n75\n7\n0\nSEQEND\n10\n100\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_vertex_card_directory(&DxfCancellationToken::default())?;
    let vertices = directory.evidence_directory().vertices();
    assert_eq!(vertices.len(), 2);
    assert_eq!(directory.members().len(), 4);

    let first_ordinal = vertices[0].vertex_record().ordinal();
    let location_x = directory
        .card_for_role(first_ordinal, DxfPolylineVertexValueRole::LocationX)
        .ok_or(io::Error::other("location x card"))?;
    assert_eq!(
        location_x.state(),
        DxfPolylineVertexValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    for member in directory
        .members_for_card(location_x.ordinal())
        .ok_or(io::Error::other("location x members"))?
    {
        assert!(
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("location x value"))?
                .value()
                .is_err()
        );
    }

    for role in [
        DxfPolylineVertexValueRole::Flags,
        DxfPolylineVertexValueRole::Identifier,
    ] {
        let card = directory
            .card_for_role(first_ordinal, role)
            .ok_or(io::Error::other("invalid unique card"))?;
        assert_eq!(card.state(), DxfPolylineVertexValueCardState::Unique);
        let [member] = directory
            .members_for_card(card.ordinal())
            .ok_or(io::Error::other("invalid unique member"))?
        else {
            return Err(io::Error::other("one invalid member").into());
        };
        assert!(
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("invalid unique value"))?
                .value()
                .is_err()
        );
    }

    let location_y = directory
        .card_for_role(first_ordinal, DxfPolylineVertexValueRole::LocationY)
        .ok_or(io::Error::other("location y card"))?;
    assert_eq!(location_y.state(), DxfPolylineVertexValueCardState::Absent);
    assert!(
        directory
            .members_for_card(location_y.ordinal())
            .ok_or(io::Error::other("location y members"))?
            .is_empty()
    );

    let second_cards = directory
        .cards_for_vertex_raw_ordinal(vertices[1].vertex_record().ordinal())
        .ok_or(io::Error::other("empty vertex cards"))?;
    assert!(
        second_cards
            .iter()
            .all(|card| card.state() == DxfPolylineVertexValueCardState::Absent)
    );
    Ok(())
}

#[test]
fn cards_remain_available_for_every_sequence_state_and_section() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n70\n1\n0\nLINE\n0\nPOLYLINE\n0\nVERTEX\n70\n2\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nPOLYLINE\n0\nVERTEX\n70\n4\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_vertex_card_directory(&DxfCancellationToken::default())?;
    let expected_states = [
        DxfPolylineSequenceState::Interrupted,
        DxfPolylineSequenceState::Unclosed,
        DxfPolylineSequenceState::Closed,
    ];
    let sequences = directory
        .evidence_directory()
        .sequence_directory()
        .sequences();
    assert_eq!(sequences.len(), 3);
    for (sequence, expected_state) in sequences.iter().zip(expected_states) {
        assert_eq!(sequence.state(), expected_state);
        let vertices = directory
            .evidence_directory()
            .vertices_for_polyline_raw_ordinal(sequence.polyline_record().ordinal())
            .ok_or(io::Error::other("sequence vertices"))?;
        let [vertex] = vertices else {
            return Err(io::Error::other("one sequence vertex").into());
        };
        let flags = directory
            .card_for_role(
                vertex.vertex_record().ordinal(),
                DxfPolylineVertexValueRole::Flags,
            )
            .ok_or(io::Error::other("flags card"))?;
        assert_eq!(flags.state(), DxfPolylineVertexValueCardState::Unique);
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
        document.polyline_vertex_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_vertex_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_vertex_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    assert_copy::<DxfPolylineVertexValueCard>();
    assert_copy::<DxfPolylineVertexCardMember>();
    assert_copy::<DxfPolylineVertexCardMemberRange>();
    assert_send_sync::<DxfPolylineVertexCardDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfPolylineVertexCardDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 26);
    assert_eq!(directory.members().len(), 15);
    let vertices = directory.evidence_directory().vertices();
    assert_eq!(vertices.len(), 2);

    let first_cards = directory
        .cards_for_vertex_raw_ordinal(vertices[0].vertex_record().ordinal())
        .ok_or(io::Error::other("first vertex cards"))?;
    assert_eq!(
        first_cards
            .iter()
            .map(|card| card.role())
            .collect::<Vec<_>>(),
        ROLE_ORDER
    );
    assert_eq!(
        first_cards[0].state(),
        DxfPolylineVertexValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert!(
        first_cards[1..]
            .iter()
            .all(|card| card.state() == DxfPolylineVertexValueCardState::Absent)
    );

    let second_cards = directory
        .cards_for_vertex_raw_ordinal(vertices[1].vertex_record().ordinal())
        .ok_or(io::Error::other("second vertex cards"))?;
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
            .all(|card| card.state() == DxfPolylineVertexValueCardState::Unique)
    );
    Ok(())
}

fn card_evidence(
    directory: &DxfPolylineVertexCardDirectory,
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
                        .map_err(|_| io::Error::other("card number"))?
                    {
                        DxfPolylineVertexNumber::Double(value) => {
                            Ok(NumberEvidence::Double(value.to_bits()))
                        }
                        DxfPolylineVertexNumber::Int16(value) => Ok(NumberEvidence::Int16(value)),
                        DxfPolylineVertexNumber::Int32(value) => Ok(NumberEvidence::Int32(value)),
                        _ => Err(io::Error::other("unknown card number")),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                card.vertex().sequence_vertex_ordinal(),
                card.role(),
                card.state(),
                values,
            ))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n-0\n10\n1\n0\nVERTEX\n10\n2\n20\n3\n30\n4\n40\n0.25\n41\n0.75\n42\n-1\n50\n45\n70\n192\n71\n1\n72\n-2\n73\n3\n74\n-4\n91\n2147483647\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"POLYLINE")?;
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    push_double(&mut bytes, version, 10, -0.0)?;
    push_double(&mut bytes, version, 10, 1.0)?;
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    for (code, value) in [
        (10, 2.0),
        (20, 3.0),
        (30, 4.0),
        (40, 0.25),
        (41, 0.75),
        (42, -1.0),
        (50, 45.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(70, 192), (71, 1), (72, -2), (73, 3), (74, -4)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, i32::MAX)?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
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

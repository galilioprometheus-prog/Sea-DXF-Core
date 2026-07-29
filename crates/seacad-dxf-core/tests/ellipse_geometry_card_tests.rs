use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEllipseGeometryCardDirectory, DxfEllipseGeometryCardMember,
    DxfEllipseGeometryValueCard, DxfEllipseGeometryValueCardState, DxfEllipseGeometryValueRole,
    DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type CardEvidence = (
    DxfEllipseGeometryValueRole,
    DxfEllipseGeometryValueCardState,
    Vec<u64>,
);

const ROLE_ORDER: [DxfEllipseGeometryValueRole; 12] = [
    DxfEllipseGeometryValueRole::WcsCenterX,
    DxfEllipseGeometryValueRole::WcsCenterY,
    DxfEllipseGeometryValueRole::WcsCenterZ,
    DxfEllipseGeometryValueRole::WcsMajorAxisEndpointX,
    DxfEllipseGeometryValueRole::WcsMajorAxisEndpointY,
    DxfEllipseGeometryValueRole::WcsMajorAxisEndpointZ,
    DxfEllipseGeometryValueRole::MinorToMajorAxisRatio,
    DxfEllipseGeometryValueRole::StartParameter,
    DxfEllipseGeometryValueRole::EndParameter,
    DxfEllipseGeometryValueRole::ExtrusionX,
    DxfEllipseGeometryValueRole::ExtrusionY,
    DxfEllipseGeometryValueRole::ExtrusionZ,
];

#[test]
fn every_dialect_has_ascii_binary_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_cards = ascii.ellipse_geometry_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_cards =
            binary.ellipse_geometry_card_directory(&DxfCancellationToken::default())?;

        assert_card_directory(&ascii_cards)?;
        assert_card_directory(&binary_cards)?;
        assert_eq!(card_evidence(&ascii_cards)?, card_evidence(&binary_cards)?);
    }
    Ok(())
}

#[test]
fn cardinality_and_lexical_validity_remain_independent() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nELLIPSE\n40\n.\n40\n1e-9999\n41\n.\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.ellipse_geometry_card_directory(&DxfCancellationToken::default())?;
    let record = directory.evidence_directory().records()[0];

    let ratio = directory
        .card_for_role(
            record.record().ordinal(),
            DxfEllipseGeometryValueRole::MinorToMajorAxisRatio,
        )
        .ok_or(io::Error::other("ratio card"))?;
    assert_eq!(
        ratio.state(),
        DxfEllipseGeometryValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    for member in directory
        .members_for_card(ratio.ordinal())
        .ok_or(io::Error::other("ratio members"))?
    {
        assert!(
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("ratio value"))?
                .value()
                .is_err()
        );
    }

    let start = directory
        .card_for_role(
            record.record().ordinal(),
            DxfEllipseGeometryValueRole::StartParameter,
        )
        .ok_or(io::Error::other("start card"))?;
    assert_eq!(start.state(), DxfEllipseGeometryValueCardState::Unique);
    assert!(
        directory
            .value_for_member(
                *directory
                    .members_for_card(start.ordinal())
                    .and_then(|members| members.first())
                    .ok_or(io::Error::other("start member"))?
            )
            .ok_or(io::Error::other("start value"))?
            .value()
            .is_err()
    );

    let end = directory
        .card_for_role(
            record.record().ordinal(),
            DxfEllipseGeometryValueRole::EndParameter,
        )
        .ok_or(io::Error::other("end card"))?;
    assert_eq!(end.state(), DxfEllipseGeometryValueCardState::Absent);
    assert!(
        directory
            .members_for_card(end.ordinal())
            .ok_or(io::Error::other("end members"))?
            .is_empty()
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
        document.ellipse_geometry_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.ellipse_geometry_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_record(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    assert_copy::<DxfEllipseGeometryValueCard>();
    assert_copy::<DxfEllipseGeometryCardMember>();
    assert_send_sync::<DxfEllipseGeometryCardDirectory>();
    Ok(())
}

fn assert_card_directory(
    directory: &DxfEllipseGeometryCardDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 12);
    assert_eq!(directory.members().len(), 12);
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );

    let record = directory.evidence_directory().records()[0];
    let cards = directory
        .cards_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("ellipse cards"))?;
    assert_eq!(cards.len(), 12);
    assert_eq!(
        cards.iter().map(|card| card.role()).collect::<Vec<_>>(),
        ROLE_ORDER
    );

    let center_x = directory
        .card_for_role(
            record.record().ordinal(),
            DxfEllipseGeometryValueRole::WcsCenterX,
        )
        .ok_or(io::Error::other("center x card"))?;
    assert_eq!(
        center_x.state(),
        DxfEllipseGeometryValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    let center_x_bits = directory
        .members_for_card(center_x.ordinal())
        .ok_or(io::Error::other("center x members"))?
        .iter()
        .map(|member| {
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("center x member value"))?
                .value()
                .map(|value| value.to_bits())
                .map_err(|_| io::Error::other("center x numeric value"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(center_x_bits, [(-0.0_f64).to_bits(), 1.0_f64.to_bits()]);

    let extrusion_y = directory
        .card_for_role(
            record.record().ordinal(),
            DxfEllipseGeometryValueRole::ExtrusionY,
        )
        .ok_or(io::Error::other("extrusion y card"))?;
    assert_eq!(
        extrusion_y.state(),
        DxfEllipseGeometryValueCardState::Absent
    );
    assert!(
        cards
            .iter()
            .filter(|card| card.role() != DxfEllipseGeometryValueRole::WcsCenterX)
            .filter(|card| card.role() != DxfEllipseGeometryValueRole::ExtrusionY)
            .all(|card| card.state() == DxfEllipseGeometryValueCardState::Unique)
    );
    Ok(())
}

fn card_evidence(
    directory: &DxfEllipseGeometryCardDirectory,
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
            Ok((card.role(), card.state(), values))
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nELLIPSE\n10\n-0\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n40\n0.5\n41\n0\n42\n6.283185307179586\n210\n0\n230\n1\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"ELLIPSE")?;
    for (code, value) in [
        (10, -0.0),
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (11, 4.0),
        (21, 5.0),
        (31, 6.0),
        (40, 0.5),
        (41, 0.0),
        (42, std::f64::consts::TAU),
        (210, 0.0),
        (230, 1.0),
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

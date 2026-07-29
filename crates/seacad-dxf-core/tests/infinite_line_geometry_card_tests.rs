use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInfiniteLineGeometryCardDirectory,
    DxfInfiniteLineGeometryCardMember, DxfInfiniteLineGeometryKind,
    DxfInfiniteLineGeometryValueCard, DxfInfiniteLineGeometryValueCardState,
    DxfInfiniteLineGeometryValueRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

type CardEvidence = (
    DxfInfiniteLineGeometryKind,
    DxfInfiniteLineGeometryValueRole,
    DxfInfiniteLineGeometryValueCardState,
    Vec<u64>,
);

const ROLE_ORDER: [DxfInfiniteLineGeometryValueRole; 6] = [
    DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointX,
    DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointY,
    DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointZ,
    DxfInfiniteLineGeometryValueRole::WcsUnitDirectionX,
    DxfInfiniteLineGeometryValueRole::WcsUnitDirectionY,
    DxfInfiniteLineGeometryValueRole::WcsUnitDirectionZ,
];

#[test]
fn every_dialect_has_ascii_binary_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_cards =
            ascii.infinite_line_geometry_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_cards =
            binary.infinite_line_geometry_card_directory(&DxfCancellationToken::default())?;

        assert_card_directory(&ascii_cards)?;
        assert_card_directory(&binary_cards)?;
        assert_eq!(card_evidence(&ascii_cards)?, card_evidence(&binary_cards)?);
    }
    Ok(())
}

#[test]
fn cardinality_and_lexical_validity_remain_independent() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nRAY\n10\n.\n10\n1e-9999\n11\n.\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.infinite_line_geometry_card_directory(&DxfCancellationToken::default())?;
    let record = directory.evidence_directory().records()[0];

    let point_x = directory
        .card_for_role(
            record.record().ordinal(),
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointX,
        )
        .ok_or(io::Error::other("point x card"))?;
    assert_eq!(
        point_x.state(),
        DxfInfiniteLineGeometryValueCardState::Multiple {
            occurrence_count: 2
        }
    );
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

    let direction_x = directory
        .card_for_role(
            record.record().ordinal(),
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionX,
        )
        .ok_or(io::Error::other("direction x card"))?;
    assert_eq!(
        direction_x.state(),
        DxfInfiniteLineGeometryValueCardState::Unique
    );
    assert!(
        directory
            .value_for_member(
                *directory
                    .members_for_card(direction_x.ordinal())
                    .and_then(|members| members.first())
                    .ok_or(io::Error::other("direction x member"))?
            )
            .ok_or(io::Error::other("direction x value"))?
            .value()
            .is_err()
    );

    let point_y = directory
        .card_for_role(
            record.record().ordinal(),
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointY,
        )
        .ok_or(io::Error::other("point y card"))?;
    assert_eq!(
        point_y.state(),
        DxfInfiniteLineGeometryValueCardState::Absent
    );
    assert!(
        directory
            .members_for_card(point_y.ordinal())
            .ok_or(io::Error::other("point y members"))?
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
        document.infinite_line_geometry_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.infinite_line_geometry_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.card(u64::MAX), None);
    assert_eq!(directory.cards_for_raw_record(u64::MAX), None);
    assert_eq!(directory.members_for_card(u64::MAX), None);
    assert_copy::<DxfInfiniteLineGeometryValueCard>();
    assert_copy::<DxfInfiniteLineGeometryCardMember>();
    assert_send_sync::<DxfInfiniteLineGeometryCardDirectory>();
    Ok(())
}

fn assert_card_directory(
    directory: &DxfInfiniteLineGeometryCardDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), 12);
    assert_eq!(directory.members().len(), 12);
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );

    let records = directory.evidence_directory().records();
    let ray = records[0];
    let xline = records[1];
    for record in [ray, xline] {
        let cards = directory
            .cards_for_raw_record(record.record().ordinal())
            .ok_or(io::Error::other("infinite-line cards"))?;
        assert_eq!(cards.len(), 6);
        assert_eq!(
            cards.iter().map(|card| card.role()).collect::<Vec<_>>(),
            ROLE_ORDER
        );
    }

    let ray_point_x = directory
        .card_for_role(
            ray.record().ordinal(),
            DxfInfiniteLineGeometryValueRole::WcsStartOrFirstPointX,
        )
        .ok_or(io::Error::other("ray point x card"))?;
    assert_eq!(
        ray_point_x.state(),
        DxfInfiniteLineGeometryValueCardState::Multiple {
            occurrence_count: 2
        }
    );
    let ray_point_x_bits = directory
        .members_for_card(ray_point_x.ordinal())
        .ok_or(io::Error::other("ray point x members"))?
        .iter()
        .map(|member| {
            directory
                .value_for_member(*member)
                .ok_or(io::Error::other("ray point x member value"))?
                .value()
                .map(|value| value.to_bits())
                .map_err(|_| io::Error::other("ray point x numeric value"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(ray_point_x_bits, [(-0.0_f64).to_bits(), 1.0_f64.to_bits()]);

    let ray_direction_y = directory
        .card_for_role(
            ray.record().ordinal(),
            DxfInfiniteLineGeometryValueRole::WcsUnitDirectionY,
        )
        .ok_or(io::Error::other("ray direction y card"))?;
    assert_eq!(
        ray_direction_y.state(),
        DxfInfiniteLineGeometryValueCardState::Absent
    );
    assert!(
        directory
            .cards_for_raw_record(xline.record().ordinal())
            .ok_or(io::Error::other("xline cards"))?
            .iter()
            .all(|card| card.state() == DxfInfiniteLineGeometryValueCardState::Unique)
    );
    Ok(())
}

fn card_evidence(
    directory: &DxfInfiniteLineGeometryCardDirectory,
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
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nRAY\n10\n-0\n10\n1\n20\n2\n30\n3\n11\n1\n31\n0\n0\nXLINE\n10\n4\n20\n5\n30\n6\n11\n0\n21\n-0\n31\n1\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"RAY")?;
    for (code, value) in [
        (10, -0.0),
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (11, 1.0),
        (31, 0.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"XLINE")?;
    for (code, value) in [
        (10, 4.0),
        (20, 5.0),
        (30, 6.0),
        (11, 0.0),
        (21, -0.0),
        (31, 1.0),
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

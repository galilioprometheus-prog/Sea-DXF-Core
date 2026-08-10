use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundarySplineEdgePointCard,
    DxfHatchBoundarySplineEdgePointCardDirectory, DxfHatchBoundarySplineEdgePointCardState,
    DxfHatchBoundarySplineEdgePointKind, DxfHatchBoundarySplineEdgePointMemberRole,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type CardEvidence = (
    DxfHatchBoundarySplineEdgePointKind,
    u64,
    DxfHatchBoundarySplineEdgePointMemberRole,
    DxfHatchBoundarySplineEdgePointCardState,
    Vec<i16>,
);

#[test]
fn every_dialect_publishes_exact_control_and_fit_component_cards() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(version, complete_payload());
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_spline_edge_point_card_directory(&DxfCancellationToken::default())?;

        let binary = binary_fixture(version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_spline_edge_point_card_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_complete_cards(directory)?;
        }
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn absent_and_multiple_cards_retain_all_members_without_selection() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n94\n1\n73\n1\n74\n0\n95\n0\n96\n2\n\
         20\n8\n42\n9\n10\n1\n20\n2\n42\n3\n20\n4\n42\n5\n10\n6\n\
         97\n2\n21\n7\n11\n8\n21\n9\n21\n10\n11\n11\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_point_card_directory(&DxfCancellationToken::default())?;
    let points = directory.point_tuple_directory();
    assert_eq!(points.entries().len(), 1);
    assert_eq!(points.orphans().len(), 3);
    assert_eq!(directory.cards().len(), 6);

    let control = points
        .tuples_for_edge(0, DxfHatchBoundarySplineEdgePointKind::ControlPoint)
        .ok_or_else(|| io::Error::other("control tuples"))?;
    let y = required_card(
        &directory,
        control[0].ordinal(),
        DxfHatchBoundarySplineEdgePointMemberRole::Y,
    )?;
    assert_eq!(
        y.state(),
        DxfHatchBoundarySplineEdgePointCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        codes(directory.members_for_card(y.ordinal()).unwrap_or_default()),
        [20, 20]
    );
    let weight = required_card(
        &directory,
        control[0].ordinal(),
        DxfHatchBoundarySplineEdgePointMemberRole::Weight,
    )?;
    assert_eq!(
        weight.state(),
        DxfHatchBoundarySplineEdgePointCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        codes(
            directory
                .members_for_card(weight.ordinal())
                .unwrap_or_default()
        ),
        [42, 42]
    );

    for role in [
        DxfHatchBoundarySplineEdgePointMemberRole::Y,
        DxfHatchBoundarySplineEdgePointMemberRole::Weight,
    ] {
        assert_eq!(
            required_card(&directory, control[1].ordinal(), role)?.state(),
            DxfHatchBoundarySplineEdgePointCardState::Absent
        );
    }
    let fit = points
        .tuples_for_edge(0, DxfHatchBoundarySplineEdgePointKind::FitPoint)
        .ok_or_else(|| io::Error::other("fit tuples"))?;
    assert_eq!(
        required_card(
            &directory,
            fit[0].ordinal(),
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
        )?
        .state(),
        DxfHatchBoundarySplineEdgePointCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        required_card(
            &directory,
            fit[1].ordinal(),
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
        )?
        .state(),
        DxfHatchBoundarySplineEdgePointCardState::Absent
    );
    assert!(
        directory
            .card_for_role(
                fit[0].ordinal(),
                DxfHatchBoundarySplineEdgePointMemberRole::Weight,
            )
            .is_none()
    );
    Ok(())
}

#[test]
fn empty_and_unavailable_sequences_keep_distinct_lower_states() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n2\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n97\n0\n\
         92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n0\n41\n9\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_point_card_directory(&DxfCancellationToken::default())?;
    assert!(directory.cards().is_empty());
    assert!(directory.members().is_empty());
    assert_eq!(directory.point_tuple_directory().entries().len(), 2);
    assert!(directory.cards_for_edge(0).is_some_and(<[_]>::is_empty));
    assert!(directory.cards_for_edge(1).is_none());
    assert!(
        directory.point_tuple_directory().entries()[0]
            .grouping()
            .is_ok()
    );
    assert!(
        directory.point_tuple_directory().entries()[1]
            .grouping()
            .is_err()
    );
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n94\n1\n73\n0\n74\n0\n95\n0\n96\n1\n\
         10\n12345.625\n20\n2\n42\n1\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_spline_edge_point_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document
        .hatch_boundary_spline_edge_point_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.point_tuple_directory().source_id()
    );
    assert_eq!(directory.card(0), Some(directory.cards()[0]));
    assert_eq!(directory.card(u64::MAX), None);
    assert!(directory.cards_for_tuple(u64::MAX).is_none());
    assert!(directory.cards_for_edge(u64::MAX).is_none());
    assert!(directory.members_for_card(u64::MAX).is_none());
    assert!(directory.tuple_for_card(u64::MAX).is_none());
    assert!(size_of::<DxfHatchBoundarySplineEdgePointCard>() <= 128);
    assert!(!format!("{:?}", directory.cards()[0]).contains("12345.625"));
    copy::<DxfHatchBoundarySplineEdgePointCardState>();
    copy::<DxfHatchBoundarySplineEdgePointCard>();
    send_sync::<DxfHatchBoundarySplineEdgePointCardDirectory>();
    Ok(())
}

fn assert_complete_cards(
    directory: &DxfHatchBoundarySplineEdgePointCardDirectory,
) -> Result<(), io::Error> {
    assert_eq!(directory.cards().len(), 6);
    assert_eq!(directory.members().len(), 5);
    let points = directory.point_tuple_directory();
    assert_eq!(points.tuples().len(), 4);
    let control = points
        .tuples_for_edge(0, DxfHatchBoundarySplineEdgePointKind::ControlPoint)
        .ok_or_else(|| io::Error::other("control tuples"))?;
    assert_state(
        directory,
        control[0].ordinal(),
        DxfHatchBoundarySplineEdgePointMemberRole::Y,
        DxfHatchBoundarySplineEdgePointCardState::Unique,
    )?;
    assert_state(
        directory,
        control[0].ordinal(),
        DxfHatchBoundarySplineEdgePointMemberRole::Weight,
        DxfHatchBoundarySplineEdgePointCardState::Unique,
    )?;
    assert_state(
        directory,
        control[1].ordinal(),
        DxfHatchBoundarySplineEdgePointMemberRole::Y,
        DxfHatchBoundarySplineEdgePointCardState::Unique,
    )?;
    assert_state(
        directory,
        control[1].ordinal(),
        DxfHatchBoundarySplineEdgePointMemberRole::Weight,
        DxfHatchBoundarySplineEdgePointCardState::Absent,
    )?;
    let fit = points
        .tuples_for_edge(0, DxfHatchBoundarySplineEdgePointKind::FitPoint)
        .ok_or_else(|| io::Error::other("fit tuples"))?;
    for tuple in fit {
        assert_state(
            directory,
            tuple.ordinal(),
            DxfHatchBoundarySplineEdgePointMemberRole::Y,
            DxfHatchBoundarySplineEdgePointCardState::Unique,
        )?;
        assert_eq!(
            directory
                .cards_for_tuple(tuple.ordinal())
                .ok_or_else(|| io::Error::other("fit cards"))?
                .len(),
            1
        );
    }
    for card in directory.cards().iter().copied() {
        assert_eq!(directory.card(card.ordinal()), Some(card));
        assert_eq!(
            directory.tuple_for_card(card.ordinal()),
            Some(card.point_tuple())
        );
        assert_eq!(
            directory
                .members_for_card(card.ordinal())
                .ok_or_else(|| io::Error::other("card members"))?
                .len() as u64,
            card.member_count()
        );
    }
    Ok(())
}

fn assert_state(
    directory: &DxfHatchBoundarySplineEdgePointCardDirectory,
    tuple_ordinal: u64,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
    expected: DxfHatchBoundarySplineEdgePointCardState,
) -> Result<(), io::Error> {
    assert_eq!(
        required_card(directory, tuple_ordinal, role)?.state(),
        expected
    );
    Ok(())
}

fn required_card(
    directory: &DxfHatchBoundarySplineEdgePointCardDirectory,
    tuple_ordinal: u64,
    role: DxfHatchBoundarySplineEdgePointMemberRole,
) -> Result<DxfHatchBoundarySplineEdgePointCard, io::Error> {
    directory
        .card_for_role(tuple_ordinal, role)
        .ok_or_else(|| io::Error::other("point component card"))
}

fn evidence(
    directory: &DxfHatchBoundarySplineEdgePointCardDirectory,
) -> Result<Vec<CardEvidence>, io::Error> {
    directory
        .cards()
        .iter()
        .copied()
        .map(|card| {
            Ok((
                card.point_tuple().kind(),
                card.point_tuple().point_index(),
                card.role(),
                card.state(),
                codes(
                    directory
                        .members_for_card(card.ordinal())
                        .ok_or_else(|| io::Error::other("card members"))?,
                ),
            ))
        })
        .collect()
}

fn codes(members: &[seacad_dxf_core::DxfHatchBoundarySplineEdgePointCardMember]) -> Vec<i16> {
    members
        .iter()
        .map(|member| member.field().group().group_code().value())
        .collect()
}

const fn complete_payload() -> &'static str {
    "91\n1\n92\n0\n93\n1\n72\n4\n94\n2\n73\n1\n74\n0\n95\n2\n96\n2\n\
     40\n0\n40\n1\n10\n0\n20\n0\n42\n1\n10\n1\n20\n1\n\
     97\n2\n11\n0.5\n21\n0.5\n11\n0.75\n21\n0.75\n97\n0\n"
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, 1)?;
    push_i16(&mut bytes, version, 72, 4)?;
    push_i32(&mut bytes, version, 94, 2)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i16(&mut bytes, version, 74, 0)?;
    push_i32(&mut bytes, version, 95, 2)?;
    push_i32(&mut bytes, version, 96, 2)?;
    for value in [0.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (x, y, weight) in [(0.0, 0.0, Some(1.0)), (1.0, 1.0, None)] {
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
        if let Some(weight) = weight {
            push_double(&mut bytes, version, 42, weight)?;
        }
    }
    push_i32(&mut bytes, version, 97, 2)?;
    for (x, y) in [(0.5, 0.5), (0.75, 0.75)] {
        push_double(&mut bytes, version, 11, x)?;
        push_double(&mut bytes, version, 21, y)?;
    }
    push_i32(&mut bytes, version, 97, 0)?;
    push_i16(&mut bytes, version, 75, 0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
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

fn copy<T: Copy>() {}

fn send_sync<T: Send + Sync>() {}

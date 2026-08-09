use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HATCH_BOUNDARY_SPLINE_EDGE_HEADER_ROLES, DxfAcadVersion,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchBoundaryEdgeCountRelation, DxfHatchBoundaryEdgePathState, DxfHatchBoundaryEdgeType,
    DxfHatchBoundarySplineEdgeHeaderCard, DxfHatchBoundarySplineEdgeHeaderCardDirectory,
    DxfHatchBoundarySplineEdgeHeaderCardState, DxfHatchBoundarySplineEdgeHeaderMember,
    DxfHatchBoundarySplineEdgeHeaderRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_parity_for_five_spline_header_cards() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n2\n\
             72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
             72\n4\n94\n3\n73\n1\n74\n0\n95\n4\n96\n2\n\
             40\n0\n40\n0\n40\n1\n40\n1\n\
             10\n0\n20\n0\n10\n1\n20\n1\n42\n1\n42\n1\n\
             97\n0\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_spline_edge_header_card_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_spline_edge_header_card_directory(&DxfCancellationToken::default())?;

        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.cards().len(), 5);
            assert_eq!(directory.members().len(), 5);
            assert_eq!(directory.cards_for_edge(0), Some(&[][..]));
            assert_eq!(directory.cards_for_edge(1), Some(directory.cards()));
            for (card, role) in directory
                .cards()
                .iter()
                .copied()
                .zip(DXF_HATCH_BOUNDARY_SPLINE_EDGE_HEADER_ROLES)
            {
                assert_eq!(card.edge_ordinal(), 1);
                assert_eq!(card.role(), role);
                assert_eq!(
                    card.state(),
                    DxfHatchBoundarySplineEdgeHeaderCardState::Unique
                );
                let [member] = directory
                    .members_for_card(card.ordinal())
                    .ok_or(io::Error::other("spline header card members"))?
                else {
                    return Err(io::Error::other("one spline header card member").into());
                };
                assert_eq!(
                    member.field().group().group_code().value(),
                    role.group_code()
                );
                assert_eq!(member.field().subclass_ordinal(), 0);
                assert_eq!(
                    directory
                        .edge_entry_for_card(card.ordinal())
                        .ok_or(io::Error::other("spline edge"))?
                        .edge_type(),
                    Ok(DxfHatchBoundaryEdgeType::Spline)
                );
            }
        }
    }
    Ok(())
}

#[test]
fn absent_unique_and_multiple_header_roles_remain_independent() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n\
         94\n2\n94\n3\n74\n0\n95\n4\n96\n2\n\
         40\n0\n40\n0\n40\n1\n40\n1\n\
         10\n0\n20\n0\n10\n1\n20\n1\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_header_card_directory(&DxfCancellationToken::default())?;
    for (role, expected) in [
        (
            DxfHatchBoundarySplineEdgeHeaderRole::Degree,
            DxfHatchBoundarySplineEdgeHeaderCardState::Multiple {
                occurrence_count: 2,
            },
        ),
        (
            DxfHatchBoundarySplineEdgeHeaderRole::Rational,
            DxfHatchBoundarySplineEdgeHeaderCardState::Absent,
        ),
        (
            DxfHatchBoundarySplineEdgeHeaderRole::Periodic,
            DxfHatchBoundarySplineEdgeHeaderCardState::Unique,
        ),
        (
            DxfHatchBoundarySplineEdgeHeaderRole::KnotCount,
            DxfHatchBoundarySplineEdgeHeaderCardState::Unique,
        ),
        (
            DxfHatchBoundarySplineEdgeHeaderRole::ControlPointCount,
            DxfHatchBoundarySplineEdgeHeaderCardState::Unique,
        ),
    ] {
        let card = directory
            .card_for_role(0, role)
            .ok_or(io::Error::other("spline header role card"))?;
        assert_eq!(card.state(), expected);
        let members = directory
            .members_for_card(card.ordinal())
            .ok_or(io::Error::other("spline header role members"))?;
        assert_eq!(members.len() as u64, card.member_range().len());
        assert!(
            members
                .iter()
                .all(|member| member.field().group().group_code().value() == role.group_code())
        );
        assert!(members.windows(2).all(|pair| {
            pair[0].field().group().occurrence() < pair[1].field().group().occurrence()
        }));
    }
    assert_eq!(directory.members().len(), 5);
    Ok(())
}

#[test]
fn mismatch_non_spline_invalid_empty_and_polyline_states_do_not_invent_cards()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n3\n\
         92\n0\n93\n0\n\
         72\n1\n10\n1\n20\n2\n11\n3\n21\n4\n\
         72\n2\n10\n5\n20\n6\n40\n7\n50\n0\n51\n90\n73\n1\n\
         72\n3\n10\n8\n20\n9\n11\n10\n21\n11\n40\n0.5\n50\n0\n51\n90\n73\n1\n\
         72\n4\n94\n3\n73\n0\n74\n0\n95\n0\n96\n0\n97\n0\n\
         72\n.\n94\n4\n73\n1\n74\n1\n95\n0\n96\n0\n97\n0\n\
         92\n0\n93\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let directory = open_ascii(&source)?
        .hatch_boundary_spline_edge_header_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.cards().len(), 5);
    assert_eq!(directory.cards_for_edge(0), Some(&[][..]));
    assert_eq!(directory.cards_for_edge(1), Some(&[][..]));
    assert_eq!(directory.cards_for_edge(2), Some(&[][..]));
    assert_eq!(directory.cards_for_edge(3), Some(directory.cards()));
    assert_eq!(directory.cards_for_edge(4), Some(&[][..]));
    assert!(directory.cards_for_edge(5).is_none());
    let edge_directory = directory.edge_type_directory().edge_directory();
    let DxfHatchBoundaryEdgePathState::Grouped(path) = edge_directory.paths()[0].state() else {
        return Err(io::Error::other("grouped mismatched edge path").into());
    };
    assert_eq!(
        path.count_relation(),
        DxfHatchBoundaryEdgeCountRelation::Mismatched {
            declared: 0,
            observed: 5
        }
    );
    assert_eq!(
        directory.edge_type_directory().entries_for_path(1),
        Some(&[][..])
    );
    assert!(
        directory
            .edge_type_directory()
            .entries_for_path(2)
            .is_none()
    );
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n4\n\
         94\n12345\n73\n1\n74\n0\n95\n4\n96\n2\n\
         40\n0\n40\n0\n40\n1\n40\n1\n\
         10\n0\n20\n0\n10\n1\n20\n1\n97\n0\n97\n0\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_spline_edge_header_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document
        .hatch_boundary_spline_edge_header_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.card(0), Some(directory.cards()[0]));
    assert_eq!(directory.card(u64::MAX), None);
    assert!(directory.cards_for_edge(u64::MAX).is_none());
    assert!(directory.members_for_card(u64::MAX).is_none());
    assert!(directory.edge_entry_for_card(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.edge_type_directory().source_id()
    );
    assert!(size_of::<DxfHatchBoundarySplineEdgeHeaderCard>() <= 48);
    assert!(size_of::<DxfHatchBoundarySplineEdgeHeaderMember>() <= 96);
    assert!(!format!("{:?}", directory.members()[0]).contains("12345"));
    copy::<DxfHatchBoundarySplineEdgeHeaderCard>();
    copy::<DxfHatchBoundarySplineEdgeHeaderMember>();
    send_sync::<DxfHatchBoundarySplineEdgeHeaderCardDirectory>();
    Ok(())
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
    push_i32(&mut bytes, version, 93, 2)?;
    push_i16(&mut bytes, version, 72, 1)?;
    for (code, value) in [(10, 1.0), (20, 2.0), (11, 3.0), (21, 4.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 72, 4)?;
    push_i32(&mut bytes, version, 94, 3)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i16(&mut bytes, version, 74, 0)?;
    push_i32(&mut bytes, version, 95, 4)?;
    push_i32(&mut bytes, version, 96, 2)?;
    for value in [0.0, 0.0, 1.0, 1.0] {
        push_double(&mut bytes, version, 40, value)?;
    }
    for (code, value) in [(10, 0.0), (20, 0.0), (10, 1.0), (20, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    for value in [1.0, 1.0] {
        push_double(&mut bytes, version, 42, value)?;
    }
    push_i32(&mut bytes, version, 97, 0)?;
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

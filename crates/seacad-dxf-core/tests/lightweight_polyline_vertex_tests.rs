use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfLightweightPolylineGroupedRecordEntry, DxfLightweightPolylineInteger,
    DxfLightweightPolylineIntegerIssue, DxfLightweightPolylineVertexCard,
    DxfLightweightPolylineVertexCardState, DxfLightweightPolylineVertexDirectory,
    DxfLightweightPolylineVertexEntry, DxfLightweightPolylineVertexMember,
    DxfLightweightPolylineVertexRole, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MemberEvidence {
    Floating(u64),
    Integer(DxfLightweightPolylineInteger),
}

type CardEvidence = (
    DxfLightweightPolylineVertexRole,
    DxfLightweightPolylineVertexCardState,
    Vec<MemberEvidence>,
);

#[test]
fn every_supported_dialect_has_ascii_binary_vertex_grouping_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.lightweight_polyline_vertex_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.lightweight_polyline_vertex_directory(&DxfCancellationToken::default())?;

        assert_valid_directory(&ascii_directory)?;
        assert_valid_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn pre_anchor_values_duplicates_absence_and_invalidity_remain_explicit()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n20\n.\n91\n5\n40\n1\n10\n1\n20\n2\n20\n3\n42\n.\n91\n2147483648\n38\n9\n10\n4\n91\n7\n91\n8\n0\nLWPOLYLINE\n38\n1\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.lightweight_polyline_vertex_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    let first = directory.records()[0];
    assert_eq!(first.vertex_count(), 2);
    assert_eq!(first.orphan_count(), 3);
    let orphans = directory
        .orphan_members_for_raw_record(first.record().ordinal())
        .ok_or(io::Error::other("orphan members"))?;
    assert_eq!(orphans.len(), 3);
    assert_eq!(
        [orphans[0].role(), orphans[1].role(), orphans[2].role()],
        [
            DxfLightweightPolylineVertexRole::OcsY,
            DxfLightweightPolylineVertexRole::Identifier,
            DxfLightweightPolylineVertexRole::StartWidth,
        ]
    );
    assert_eq!(
        directory
            .floating_value_for_member(orphans[0])
            .ok_or(io::Error::other("orphan y"))?
            .value(),
        Err(
            seacad_dxf_core::DxfLightweightPolylineNumericIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
            )
        )
    );
    assert!(directory.integer_value_for_member(orphans[0]).is_none());
    assert_eq!(
        directory
            .integer_value_for_member(orphans[1])
            .ok_or(io::Error::other("orphan identifier"))?
            .value(),
        Ok(DxfLightweightPolylineInteger::I32(5))
    );

    let vertices = directory
        .vertices_for_raw_record(first.record().ordinal())
        .ok_or(io::Error::other("grouped vertices"))?;
    assert_eq!(vertices.len(), 2);
    assert_states(
        vertices[0],
        [
            DxfLightweightPolylineVertexCardState::Unique,
            DxfLightweightPolylineVertexCardState::Multiple {
                occurrence_count: 2,
            },
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Unique,
            DxfLightweightPolylineVertexCardState::Unique,
        ],
    );
    assert_states(
        vertices[1],
        [
            DxfLightweightPolylineVertexCardState::Unique,
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Multiple {
                occurrence_count: 2,
            },
        ],
    );

    let bulge = directory
        .card_for_role(0, DxfLightweightPolylineVertexRole::Bulge)
        .ok_or(io::Error::other("bulge card"))?;
    let bulge_member = only_member(&directory, bulge)?;
    assert!(matches!(
        directory
            .floating_value_for_member(bulge_member)
            .ok_or(io::Error::other("bulge value"))?
            .value(),
        Err(seacad_dxf_core::DxfLightweightPolylineNumericIssue::InvalidAsciiNumber(_))
    ));
    let identifier = directory
        .card_for_role(0, DxfLightweightPolylineVertexRole::Identifier)
        .ok_or(io::Error::other("identifier card"))?;
    let identifier_member = only_member(&directory, identifier)?;
    assert_eq!(
        directory
            .integer_value_for_member(identifier_member)
            .ok_or(io::Error::other("identifier value"))?
            .value(),
        Err(DxfLightweightPolylineIntegerIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );

    let second = directory.records()[1];
    assert_eq!(second.vertex_count(), 0);
    assert_eq!(second.orphan_count(), 0);
    assert_eq!(
        directory.vertices_for_raw_record(second.record().ordinal()),
        Some([].as_slice())
    );
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
        document.lightweight_polyline_vertex_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.lightweight_polyline_vertex_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.vertices_for_raw_record(u64::MAX), None);
    assert_eq!(
        directory.card_for_role(u64::MAX, DxfLightweightPolylineVertexRole::OcsX),
        None
    );
    assert_copy::<DxfLightweightPolylineGroupedRecordEntry>();
    assert_copy::<DxfLightweightPolylineVertexEntry>();
    assert_copy::<DxfLightweightPolylineVertexCard>();
    assert_copy::<DxfLightweightPolylineVertexMember>();
    assert_send_sync::<DxfLightweightPolylineVertexDirectory>();
    Ok(())
}

fn assert_valid_directory(
    directory: &DxfLightweightPolylineVertexDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.vertices().len(), 2);
    let record = directory.records()[0];
    assert_eq!(record.vertex_count(), 2);
    assert_eq!(record.orphan_count(), 0);
    let vertices = directory
        .vertices_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("vertices"))?;
    assert_eq!(vertices[0].record_vertex_ordinal(), 0);
    assert_eq!(vertices[1].record_vertex_ordinal(), 1);
    assert!(vertices[0].anchor_group_occurrence() < vertices[1].anchor_group_occurrence());
    assert_states(
        vertices[0],
        [DxfLightweightPolylineVertexCardState::Unique; 6],
    );
    assert_states(
        vertices[1],
        [
            DxfLightweightPolylineVertexCardState::Unique,
            DxfLightweightPolylineVertexCardState::Unique,
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Absent,
            DxfLightweightPolylineVertexCardState::Unique,
            DxfLightweightPolylineVertexCardState::Unique,
        ],
    );
    for (vertex_ordinal, vertex) in directory.vertices().iter().enumerate() {
        for card in vertex.cards() {
            assert_eq!(card.vertex_ordinal(), vertex_ordinal as u64);
            let members = directory
                .members_for_card(*card)
                .ok_or(io::Error::other("card members"))?;
            assert_eq!(members.len() as u64, card.member_count());
            assert!(members.iter().all(|member| member.role() == card.role()));
        }
    }
    Ok(())
}

fn assert_states(
    vertex: DxfLightweightPolylineVertexEntry,
    expected: [DxfLightweightPolylineVertexCardState; 6],
) {
    assert_eq!(vertex.cards().map(|card| card.state()), expected);
}

fn only_member(
    directory: &DxfLightweightPolylineVertexDirectory,
    card: DxfLightweightPolylineVertexCard,
) -> Result<DxfLightweightPolylineVertexMember, io::Error> {
    let members = directory
        .members_for_card(card)
        .ok_or(io::Error::other("card members"))?;
    let [member] = members else {
        return Err(io::Error::other("unique card member"));
    };
    Ok(*member)
}

fn evidence(
    directory: &DxfLightweightPolylineVertexDirectory,
) -> Result<Vec<Vec<CardEvidence>>, io::Error> {
    directory
        .vertices()
        .iter()
        .map(|vertex| {
            vertex
                .cards()
                .iter()
                .copied()
                .map(|card| {
                    let values = directory
                        .members_for_card(card)
                        .ok_or(io::Error::other("card members"))?
                        .iter()
                        .copied()
                        .map(|member| {
                            if let Some(value) = directory.floating_value_for_member(member) {
                                return value
                                    .value()
                                    .map(DxfDouble::to_bits)
                                    .map(MemberEvidence::Floating)
                                    .map_err(|_| io::Error::other("floating member"));
                            }
                            directory
                                .integer_value_for_member(member)
                                .ok_or(io::Error::other("integer member"))?
                                .value()
                                .map(MemberEvidence::Integer)
                                .map_err(|_| io::Error::other("integer member"))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok((card.role(), card.state(), values))
                })
                .collect()
        })
        .collect()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n70\n129\n10\n1\n20\n2\n40\n0.25\n41\n0.5\n42\n-0\n91\n7\n10\n3\n20\n4\n42\n1\n91\n8\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [(10, 1.0), (20, 2.0), (40, 0.25), (41, 0.5), (42, -0.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 7)?;
    for (code, value) in [(10, 3.0), (20, 4.0), (42, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 8)?;
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

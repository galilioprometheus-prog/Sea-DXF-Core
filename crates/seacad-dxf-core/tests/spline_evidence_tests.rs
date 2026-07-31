use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_SPLINE_ROLES, DxfAcadVersion, DxfAsciiNumericIssue,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble,
    DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSplineCardDirectory,
    DxfSplineCardMember, DxfSplineCardState, DxfSplineDirectory, DxfSplineNumber,
    DxfSplineNumericIssue, DxfSplineRecordEntry, DxfSplineValue, DxfSplineValueCard,
    DxfSplineValueRole, NoopDxfReadObserver,
};

type Evidence = (DxfSplineValueRole, DxfSplineNumber);

#[test]
fn every_dialect_has_ascii_binary_spline_evidence_parity() -> Result<(), Box<dyn Error>> {
    let expected = expected_evidence();
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.spline_directory(&DxfCancellationToken::default())?;
        let ascii_cards = ascii.spline_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.spline_directory(&DxfCancellationToken::default())?;
        let binary_cards = binary.spline_card_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_card_directory(&ascii_cards)?;
        assert_card_directory(&binary_cards)?;
        assert_eq!(evidence(&ascii_directory)?, expected);
        assert_eq!(evidence(&binary_directory)?, expected);
        assert_eq!(card_evidence(&ascii_cards)?, card_evidence(&binary_cards)?);
    }
    Ok(())
}

#[test]
fn duplicates_invalid_numbers_and_application_decoys_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n70\n.\n42\n1e-9999\n71\n2\n71\n3\n\
102\n{APP\n40\n99\n102\n}\n40\n1\n0\nSPLINE\n8\nLayer\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 2);
    let first = directory.records()[0];
    let values = directory
        .values_for_raw_record(first.record().ordinal())
        .ok_or(io::Error::other("spline values"))?;
    assert_eq!(values.len(), 5);
    assert_eq!(
        values[0].value(),
        Err(DxfSplineNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert_eq!(
        values[1].value(),
        Err(DxfSplineNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(values[2].value(), Ok(DxfSplineNumber::Int16(2)));
    assert_eq!(values[3].value(), Ok(DxfSplineNumber::Int16(3)));
    assert_eq!(values[4].role(), DxfSplineValueRole::KnotValue);
    assert_eq!(
        directory.values_for_raw_record(directory.records()[1].record().ordinal()),
        Some([].as_slice())
    );

    let cards = document.spline_card_directory(&DxfCancellationToken::default())?;
    let raw = cards.evidence_directory().records()[0].record().ordinal();
    assert_eq!(
        cards
            .card_for_role(raw, DxfSplineValueRole::Flags)
            .ok_or(io::Error::other("flags card"))?
            .state(),
        DxfSplineCardState::Unique
    );
    let degree = cards
        .card_for_role(raw, DxfSplineValueRole::Degree)
        .ok_or(io::Error::other("degree card"))?;
    assert_eq!(
        degree.state(),
        DxfSplineCardState::Multiple {
            occurrence_count: 2
        }
    );
    assert_eq!(
        cards
            .card_for_role(raw, DxfSplineValueRole::NormalX)
            .ok_or(io::Error::other("normal x card"))?
            .state(),
        DxfSplineCardState::Absent
    );
    Ok(())
}

#[test]
fn matching_is_exact_and_limited_to_complete_entity_sections() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nSPLINE\n70\n1\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nSPLINE\n71\n2\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nspline\n72\n3\n0\nSPLINE \n73\n4\n0\nSPLINE\n0\nENDSEC\n\
0\nSECTION\n2\nOBJECTS\n0\nSPLINE\n74\n5\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.values().len(), 1);
    assert_eq!(directory.values()[0].role(), DxfSplineValueRole::Degree);
    assert!(directory.records()[1].value_range().is_empty());
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.spline_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        document.spline_card_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.spline_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_copy::<DxfSplineValue>();
    assert_copy::<DxfSplineRecordEntry>();
    assert_copy::<DxfSplineValueCard>();
    assert_copy::<DxfSplineCardMember>();
    assert_send_sync::<DxfSplineDirectory>();
    assert_send_sync::<DxfSplineCardDirectory>();
    Ok(())
}

fn assert_card_directory(directory: &DxfSplineCardDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), DXF_SPLINE_ROLES.len());
    assert_eq!(directory.members().len(), 28);
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    let record = directory.evidence_directory().records()[0];
    let cards = directory
        .cards_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("spline cards"))?;
    assert_eq!(
        cards.iter().map(|card| card.role()).collect::<Vec<_>>(),
        DXF_SPLINE_ROLES
    );
    for role in [
        DxfSplineValueRole::KnotValue,
        DxfSplineValueRole::ControlPointX,
        DxfSplineValueRole::ControlPointY,
        DxfSplineValueRole::ControlPointZ,
    ] {
        assert_eq!(
            directory
                .card_for_role(record.record().ordinal(), role)
                .ok_or(io::Error::other("multiple card"))?
                .state(),
            DxfSplineCardState::Multiple {
                occurrence_count: 2
            }
        );
    }
    assert!(
        cards
            .iter()
            .filter(|card| !matches!(
                card.role(),
                DxfSplineValueRole::KnotValue
                    | DxfSplineValueRole::ControlPointX
                    | DxfSplineValueRole::ControlPointY
                    | DxfSplineValueRole::ControlPointZ
            ))
            .all(|card| card.state() == DxfSplineCardState::Unique)
    );
    Ok(())
}

fn card_evidence(
    directory: &DxfSplineCardDirectory,
) -> Result<Vec<(DxfSplineValueRole, DxfSplineCardState, Vec<DxfSplineNumber>)>, io::Error> {
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
                    directory
                        .value_for_member(*member)
                        .ok_or(io::Error::other("member value"))?
                        .value()
                        .map_err(|_| io::Error::other("member number"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((card.role(), card.state(), values))
        })
        .collect()
}

fn assert_directory(directory: &DxfSplineDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert!(directory.raw_record_count() >= 1);
    let record = directory.records()[0];
    let values = directory
        .values_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("spline values"))?;
    assert_eq!(values.len() as u64, record.value_range().len());
    for value in values.iter().copied() {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(value)
        );
    }
    Ok(())
}

fn evidence(directory: &DxfSplineDirectory) -> Result<Vec<Evidence>, io::Error> {
    let record = directory
        .records()
        .first()
        .copied()
        .ok_or(io::Error::other("spline record"))?;
    directory
        .values_for_raw_record(record.record().ordinal())
        .ok_or(io::Error::other("spline values"))?
        .iter()
        .map(|value| {
            Ok((
                value.role(),
                value
                    .value()
                    .map_err(|_| io::Error::other("spline number"))?,
            ))
        })
        .collect()
}

fn expected_evidence() -> Vec<Evidence> {
    let mut values = Vec::new();
    for (role, value) in [
        (DxfSplineValueRole::Flags, 13),
        (DxfSplineValueRole::Degree, 3),
        (DxfSplineValueRole::KnotCount, 2),
        (DxfSplineValueRole::ControlPointCount, 2),
        (DxfSplineValueRole::FitPointCount, 1),
    ] {
        values.push((role, DxfSplineNumber::Int16(value)));
    }
    for (role, value) in double_values() {
        values.push((role, DxfSplineNumber::Double(DxfDouble::from_f64(value))));
    }
    values
}

fn double_values() -> [(DxfSplineValueRole, f64); 23] {
    use DxfSplineValueRole::*;
    [
        (KnotTolerance, 1.0e-7),
        (ControlPointTolerance, 2.0e-7),
        (FitTolerance, 3.0e-10),
        (StartTangentX, -0.0),
        (StartTangentY, 2.0),
        (StartTangentZ, 3.0),
        (EndTangentX, 4.0),
        (EndTangentY, 5.0),
        (EndTangentZ, 6.0),
        (KnotValue, 0.0),
        (KnotValue, 1.0),
        (ControlPointX, 1.0),
        (ControlPointY, 2.0),
        (ControlPointZ, 3.0),
        (ControlPointX, 4.0),
        (ControlPointY, 5.0),
        (ControlPointZ, 6.0),
        (FitPointX, 7.0),
        (FitPointY, 8.0),
        (FitPointZ, 9.0),
        (NormalX, 0.0),
        (NormalY, 0.0),
        (NormalZ, 1.0),
    ]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nSPLINE\n70\n13\n71\n3\n72\n2\n73\n2\n74\n1\n42\n0.0000001\n43\n0.0000002\n44\n0.0000000003\n12\n-0\n22\n2\n32\n3\n13\n4\n23\n5\n33\n6\n40\n0\n40\n1\n10\n1\n20\n2\n30\n3\n10\n4\n20\n5\n30\n6\n11\n7\n21\n8\n31\n9\n210\n0\n220\n0\n230\n1\n0\nENDSEC\n0\nEOF\n"
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
        (0, b"SPLINE"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(70, 13), (71, 3), (72, 2), (73, 2), (74, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for ((role, value), code) in double_values().into_iter().zip([
        42, 43, 44, 12, 22, 32, 13, 23, 33, 40, 40, 10, 20, 30, 10, 20, 30, 11, 21, 31, 210, 220,
        230,
    ]) {
        let _ = role;
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

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
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

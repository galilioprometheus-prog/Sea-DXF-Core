use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HELIX_ROLES, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfHelixCardDirectory,
    DxfHelixCardMember, DxfHelixCardState, DxfHelixNumber, DxfHelixValueCard, DxfHelixValueRole,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type CardEvidence = (DxfHelixValueRole, DxfHelixCardState, Vec<DxfHelixNumber>);

#[test]
fn every_dialect_has_ascii_binary_helix_card_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_cards = ascii.helix_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_cards = binary.helix_card_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_cards, version)?;
        assert_expected(&binary_cards, version)?;
        assert_eq!(card_evidence(&ascii_cards)?, card_evidence(&binary_cards)?);
    }
    Ok(())
}

#[test]
fn absent_duplicate_and_invalid_members_remain_unselected() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbHelix\n90\n.\n40\n1\n40\n2\n\
0\nHELIX\n100\nAcDbHelix\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cards = document.helix_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(cards.cards().len(), DXF_HELIX_ROLES.len() * 2);
    assert_eq!(cards.members().len(), 3);

    let first_raw = cards.evidence_directory().records()[0]
        .entity()
        .record()
        .ordinal();
    let major = cards
        .card_for_role(first_raw, DxfHelixValueRole::MajorVersion)
        .ok_or(io::Error::other("major card"))?;
    assert_eq!(major.state(), DxfHelixCardState::Unique);
    let [major_member] = cards
        .members_for_card(major.ordinal())
        .ok_or(io::Error::other("major member"))?
    else {
        return Err(io::Error::other("one major member").into());
    };
    assert!(
        cards
            .value_for_member(*major_member)
            .ok_or(io::Error::other("major value"))?
            .value()
            .is_err()
    );

    let radius = cards
        .card_for_role(first_raw, DxfHelixValueRole::Radius)
        .ok_or(io::Error::other("radius card"))?;
    assert_eq!(
        radius.state(),
        DxfHelixCardState::Multiple {
            occurrence_count: 2
        }
    );
    let radius_values = cards
        .members_for_card(radius.ordinal())
        .ok_or(io::Error::other("radius members"))?
        .iter()
        .map(|member| {
            cards
                .value_for_member(*member)
                .ok_or(io::Error::other("radius value"))?
                .value()
                .map_err(|_| io::Error::other("valid radius"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        radius_values,
        vec![
            DxfHelixNumber::Double(seacad_dxf_core::DxfDouble::from_f64(1.0)),
            DxfHelixNumber::Double(seacad_dxf_core::DxfDouble::from_f64(2.0))
        ]
    );
    assert_eq!(
        cards
            .card_for_role(first_raw, DxfHelixValueRole::Turns)
            .ok_or(io::Error::other("turns card"))?
            .state(),
        DxfHelixCardState::Absent
    );

    let second_raw = cards.evidence_directory().records()[1]
        .entity()
        .record()
        .ordinal();
    assert!(
        cards
            .cards_for_raw_record(second_raw)
            .ok_or(io::Error::other("empty record cards"))?
            .iter()
            .all(|card| card.state() == DxfHelixCardState::Absent)
    );
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_bounds_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.helix_card_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.helix_card_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let cards = document.helix_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(cards.card(u64::MAX), None);
    assert_eq!(cards.cards_for_raw_record(u64::MAX), None);
    assert_eq!(
        cards.card_for_role(u64::MAX, DxfHelixValueRole::Radius),
        None
    );
    assert_eq!(cards.members_for_card(u64::MAX), None);
    assert_eq!(cards.source_id(), cards.evidence_directory().source_id());
    assert_copy::<DxfHelixValueCard>();
    assert_copy::<DxfHelixCardMember>();
    assert_send_sync::<DxfHelixCardDirectory>();
    assert!(size_of::<DxfHelixValueCard>() <= 256);
    Ok(())
}

fn assert_expected(
    directory: &DxfHelixCardDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), DXF_HELIX_ROLES.len());
    let record = directory.evidence_directory().records()[0];
    let raw = record.entity().record().ordinal();
    let cards = directory
        .cards_for_raw_record(raw)
        .ok_or(io::Error::other("helix cards"))?;
    assert_eq!(
        cards.iter().map(|card| card.role()).collect::<Vec<_>>(),
        DXF_HELIX_ROLES
    );
    for card in cards {
        let omitted_in_ac1009 = version == DxfAcadVersion::Ac1009
            && matches!(
                card.role(),
                DxfHelixValueRole::Handedness | DxfHelixValueRole::ConstraintType
            );
        assert_eq!(
            card.state(),
            if omitted_in_ac1009 {
                DxfHelixCardState::Absent
            } else {
                DxfHelixCardState::Unique
            }
        );
    }
    assert_eq!(
        directory.members().len(),
        DXF_HELIX_ROLES.len() - usize::from(version == DxfAcadVersion::Ac1009) * 2
    );
    Ok(())
}

fn card_evidence(directory: &DxfHelixCardDirectory) -> Result<Vec<CardEvidence>, io::Error> {
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

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let modern_tail = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "280\n2\n290\n1\n"
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbHelix\n42\n-3\n90\n1\n10\n1\n\
41\n2.5\n91\n0\n20\n2\n40\n4\n30\n3\n11\n4\n21\n5\n31\n6\n\
12\n0\n22\n0\n32\n1\n{modern_tail}0\nENDSEC\n0\nEOF\n",
        version.code()
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
        (0, b"HELIX"),
        (100, b"AcDbHelix"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 42, -3.0)?;
    push_i32(&mut bytes, version, 90, 1)?;
    for (code, value) in [
        (10, 1.0),
        (41, 2.5),
        (20, 2.0),
        (40, 4.0),
        (30, 3.0),
        (11, 4.0),
        (21, 5.0),
        (31, 6.0),
        (12, 0.0),
        (22, 0.0),
        (32, 1.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 0)?;
    if version != DxfAcadVersion::Ac1009 {
        push_i16(&mut bytes, version, 280, 2)?;
        push_code(&mut bytes, version, 290)?;
        bytes.push(1);
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

struct CancellingSource<'a> {
    bytes: &'a [u8],
    token: DxfCancellationToken,
    armed: AtomicBool,
}

impl<'a> CancellingSource<'a> {
    fn new(bytes: &'a [u8], token: DxfCancellationToken) -> Self {
        Self {
            bytes,
            token,
            armed: AtomicBool::new(false),
        }
    }

    fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }
}

impl DxfByteSource for CancellingSource<'_> {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = self.bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        if self.armed.load(Ordering::Acquire) {
            self.token.cancel();
        }
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

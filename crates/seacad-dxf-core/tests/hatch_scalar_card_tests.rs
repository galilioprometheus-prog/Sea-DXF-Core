use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HATCH_SCALAR_ROLES, DxfAcadVersion, DxfAsciiNumericIssue,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchScalarCard, DxfHatchScalarCardDirectory, DxfHatchScalarCardMember,
    DxfHatchScalarCardState, DxfHatchScalarIssue, DxfHatchScalarRole, DxfHatchScalarValue,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type CardEvidence = (
    DxfHatchScalarRole,
    DxfHatchScalarCardState,
    Vec<Result<DxfHatchScalarValue, DxfHatchScalarIssue>>,
);

#[test]
fn every_dialect_has_ascii_binary_hatch_cardinality_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_cards = ascii.hatch_scalar_card_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_cards = binary.hatch_scalar_card_directory(&DxfCancellationToken::default())?;

        assert_expected(&ascii_cards, version)?;
        assert_expected(&binary_cards, version)?;
        assert_eq!(card_evidence(&ascii_cards)?, card_evidence(&binary_cards)?);
    }
    Ok(())
}

#[test]
fn duplicate_and_invalid_occurrences_do_not_change_card_rules() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n70\n.\n70\n1\n91\n.\n\
41\n2\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cards = document.hatch_scalar_card_directory(&DxfCancellationToken::default())?;
    let entry = cards.evidence_directory().entries()[0];
    let subclass = entry.subclass_ordinal();
    assert_eq!(cards.cards().len(), DXF_HATCH_SCALAR_ROLES.len());
    assert_eq!(cards.members().len(), 4);
    let solid = cards
        .card_for_role(subclass, DxfHatchScalarRole::SolidFillFlag)
        .ok_or(io::Error::other("solid card"))?;
    assert_eq!(
        solid.state(),
        DxfHatchScalarCardState::Multiple {
            occurrence_count: 2
        }
    );
    let solid_members = cards
        .members_for_card(solid.ordinal())
        .ok_or(io::Error::other("solid members"))?;
    assert_eq!(solid_members.len(), 2);
    assert_eq!(
        cards
            .occurrence_for_member(solid_members[0])
            .ok_or(io::Error::other("invalid solid"))?
            .value(),
        Err(DxfHatchScalarIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert_eq!(
        cards
            .occurrence_for_member(solid_members[1])
            .ok_or(io::Error::other("valid solid"))?
            .value(),
        Ok(DxfHatchScalarValue::Int16(1))
    );
    let boundary = cards
        .card_for_role(subclass, DxfHatchScalarRole::BoundaryPathCount)
        .ok_or(io::Error::other("boundary card"))?;
    assert_eq!(boundary.state(), DxfHatchScalarCardState::Unique);
    let [boundary_member] = cards
        .members_for_card(boundary.ordinal())
        .ok_or(io::Error::other("boundary member"))?
    else {
        return Err(io::Error::other("one boundary member").into());
    };
    assert!(
        cards
            .occurrence_for_member(*boundary_member)
            .ok_or(io::Error::other("boundary occurrence"))?
            .value()
            .is_err()
    );
    assert_eq!(
        cards
            .card_for_role(subclass, DxfHatchScalarRole::PatternName)
            .ok_or(io::Error::other("pattern name card"))?
            .state(),
        DxfHatchScalarCardState::Absent
    );
    Ok(())
}

#[test]
fn duplicate_subclasses_have_independent_fixed_card_sets() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n70\n1\n100\nAcDbHatch\n\
0\nHATCH\n100\nAcDbHatch\n91\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cards = document.hatch_scalar_card_directory(&DxfCancellationToken::default())?;
    let entries = cards.evidence_directory().entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(cards.cards().len(), DXF_HATCH_SCALAR_ROLES.len() * 3);
    let first_raw = entries[0].subclass().entity().record().ordinal();
    assert_eq!(cards.cards_for_raw_record(first_raw).len(), 50);
    assert_eq!(
        cards
            .card_for_role(
                entries[0].subclass_ordinal(),
                DxfHatchScalarRole::SolidFillFlag
            )
            .ok_or(io::Error::other("first solid"))?
            .state(),
        DxfHatchScalarCardState::Unique
    );
    assert!(
        cards
            .cards_for_subclass(entries[1].subclass_ordinal())
            .ok_or(io::Error::other("empty subclass cards"))?
            .iter()
            .all(|card| card.state() == DxfHatchScalarCardState::Absent)
    );
    assert_eq!(
        cards
            .card_for_role(
                entries[2].subclass_ordinal(),
                DxfHatchScalarRole::BoundaryPathCount
            )
            .ok_or(io::Error::other("third boundary"))?
            .state(),
        DxfHatchScalarCardState::Unique
    );
    Ok(())
}

#[test]
fn cancellation_lookups_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_scalar_card_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.hatch_scalar_card_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let cards = document.hatch_scalar_card_directory(&DxfCancellationToken::default())?;
    assert_eq!(cards.card(u64::MAX), None);
    assert_eq!(cards.cards_for_subclass(u64::MAX), None);
    assert!(cards.cards_for_raw_record(u64::MAX).is_empty());
    assert_eq!(
        cards.card_for_role(u64::MAX, DxfHatchScalarRole::PatternName),
        None
    );
    assert_eq!(cards.members_for_card(u64::MAX), None);
    assert_eq!(cards.source_id(), cards.evidence_directory().source_id());
    assert_copy::<DxfHatchScalarCard>();
    assert_copy::<DxfHatchScalarCardMember>();
    assert_send_sync::<DxfHatchScalarCardDirectory>();
    assert!(size_of::<DxfHatchScalarCard>() <= 256);
    Ok(())
}

fn assert_expected(
    directory: &DxfHatchScalarCardDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.cards().len(), DXF_HATCH_SCALAR_ROLES.len());
    let entry = directory.evidence_directory().entries()[0];
    let cards = directory
        .cards_for_subclass(entry.subclass_ordinal())
        .ok_or(io::Error::other("hatch cards"))?;
    assert_eq!(
        cards.iter().map(|card| card.role()).collect::<Vec<_>>(),
        DXF_HATCH_SCALAR_ROLES
    );
    for card in cards {
        let high_code = matches!(
            card.role(),
            DxfHatchScalarRole::GradientKind
                | DxfHatchScalarRole::GradientReserved
                | DxfHatchScalarRole::GradientColorMode
                | DxfHatchScalarRole::GradientColorCount
                | DxfHatchScalarRole::GradientRotation
                | DxfHatchScalarRole::GradientShift
                | DxfHatchScalarRole::GradientTint
                | DxfHatchScalarRole::GradientReservedValue
                | DxfHatchScalarRole::GradientName
        );
        assert_eq!(
            card.state(),
            if version == DxfAcadVersion::Ac1009 && high_code {
                DxfHatchScalarCardState::Absent
            } else {
                DxfHatchScalarCardState::Unique
            }
        );
    }
    assert_eq!(
        directory.members().len(),
        DXF_HATCH_SCALAR_ROLES.len() - usize::from(version == DxfAcadVersion::Ac1009) * 9
    );
    Ok(())
}

fn card_evidence(directory: &DxfHatchScalarCardDirectory) -> Result<Vec<CardEvidence>, io::Error> {
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
                    Ok(directory
                        .occurrence_for_member(*member)
                        .ok_or(io::Error::other("member occurrence"))?
                        .value())
                })
                .collect::<Result<Vec<_>, io::Error>>()?;
            Ok((card.role(), card.state(), values))
        })
        .collect()
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let modern = if version == DxfAcadVersion::Ac1009 {
        String::new()
    } else {
        "450\n1\n451\n0\n452\n0\n453\n2\n460\n0.5\n461\n0.25\n462\n0.75\n463\n1\n470\nLINEAR\n"
            .to_owned()
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
0\nHATCH\n100\nAcDbHatch\n30\n3\n210\n0\n220\n0\n230\n1\n2\nANSI31\n70\n0\n71\n1\n\
91\n1\n75\n0\n76\n1\n52\n0.25\n41\n2\n77\n0\n78\n0\n47\n0.01\n98\n1\n{modern}\
0\nENDSEC\n0\nEOF\n",
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
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(30, 3.0), (210, 0.0), (220, 0.0), (230, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 2, b"ANSI31")?;
    for (code, value) in [(70, 0), (71, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 1)?;
    for (code, value) in [(75, 0), (76, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 52, 0.25)?;
    push_double(&mut bytes, version, 41, 2.0)?;
    push_i16(&mut bytes, version, 77, 0)?;
    push_i16(&mut bytes, version, 78, 0)?;
    push_double(&mut bytes, version, 47, 0.01)?;
    push_i32(&mut bytes, version, 98, 1)?;
    if version != DxfAcadVersion::Ac1009 {
        for (code, value) in [(450, 1), (451, 0), (452, 0), (453, 2)] {
            push_i32(&mut bytes, version, code, value)?;
        }
        for (code, value) in [(460, 0.5), (461, 0.25), (462, 0.75), (463, 1.0)] {
            push_double(&mut bytes, version, code, value)?;
        }
        push_string(&mut bytes, version, 470, b"LINEAR")?;
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

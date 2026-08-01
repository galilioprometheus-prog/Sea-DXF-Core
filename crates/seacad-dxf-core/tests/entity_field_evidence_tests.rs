use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfApplicationGroupKind, DxfApplicationGroupState,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfEntityField,
    DxfEntityFieldCard, DxfEntityFieldCardMember, DxfEntityFieldCardState,
    DxfEntityFieldEvidenceDirectory, DxfEntityFieldOccurrence, DxfError, DxfMemorySource,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_common_field_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let groups = fixture_groups(version, "FUTURE");
        let ascii_bytes = ascii_fixture(version, &groups);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        assert_directory(DxfRawDocumentView::from(&ascii), version)?;

        let binary_bytes = binary_fixture(version, &groups)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_directory(DxfRawDocumentView::from(&binary), version)?;
    }
    Ok(())
}

#[test]
fn subclass_and_application_scopes_exclude_colliding_group_codes() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let groups = fixture_groups(version, "FUTURE");
    let bytes = ascii_fixture(version, &groups);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = view.entity_field_evidence_directory(&DxfCancellationToken::default())?;
    let line = semantic_entities(&directory)[0];
    let occurrences = directory.occurrences_for_entity(line)?;

    assert_eq!(occurrences.len(), 11);
    assert_eq!(
        occurrences
            .iter()
            .filter(|entry| entry.field() == DxfEntityField::OWNER)
            .count(),
        1
    );
    assert_eq!(
        occurrences
            .iter()
            .filter(|entry| entry.field() == DxfEntityField::EXTENSION_DICTIONARY)
            .count(),
        1
    );
    assert_eq!(
        exact_values(
            view,
            occurrences
                .iter()
                .filter(|entry| entry.field() == DxfEntityField::LAYER)
        )?,
        [b"L1".to_vec(), b"L2".to_vec()]
    );
    for excluded in [
        b"AA".as_slice(),
        b"FAMILY_LAYER",
        b"FAMILY_OWNER",
        b"OUTSIDE",
    ] {
        let raw_ordinal = occurrence_of(view, excluded)?;
        assert_eq!(directory.occurrence_for_group(raw_ordinal), None);
    }
    let extension = occurrence_of(view, b"BB")?;
    assert_eq!(
        directory
            .occurrence_for_group(extension)
            .map(DxfEntityFieldOccurrence::field),
        Some(DxfEntityField::EXTENSION_DICTIONARY)
    );
    let extension_occurrence = directory
        .occurrence_for_group(extension)
        .ok_or(io::Error::other("missing extension-dictionary occurrence"))?;
    assert_eq!(
        directory
            .application_group_for_occurrence(extension_occurrence)?
            .map(|group| (group.kind(), group.state())),
        Some((
            DxfApplicationGroupKind::AcadXDictionary,
            DxfApplicationGroupState::Closed
        ))
    );
    assert!(
        occurrences
            .iter()
            .filter(|entry| entry.field() != DxfEntityField::EXTENSION_DICTIONARY)
            .all(|entry| entry.application_group_start_control_ordinal().is_none())
    );
    Ok(())
}

#[test]
fn cards_distinguish_required_optional_duplicate_and_sequence_without_decoding()
-> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let groups = fixture_groups(version, "FUTURE");
    let bytes = ascii_fixture(version, &groups);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_field_evidence_directory(&DxfCancellationToken::default())?;
    let entities = semantic_entities(&directory);
    let line = entities[0];
    let future = entities[1];

    assert_card(
        &directory,
        line,
        DxfEntityField::LAYER,
        DxfEntityFieldCardState::Duplicate {
            occurrence_count: 2,
        },
        2,
    )?;
    assert_card(
        &directory,
        line,
        DxfEntityField::PROXY_GRAPHICS_DATA,
        DxfEntityFieldCardState::Sequence {
            occurrence_count: 2,
        },
        2,
    )?;
    assert_card(
        &directory,
        line,
        DxfEntityField::HANDLE,
        DxfEntityFieldCardState::Unique,
        1,
    )?;
    assert_card(
        &directory,
        future,
        DxfEntityField::LINETYPE,
        DxfEntityFieldCardState::AbsentOptional,
        0,
    )?;
    assert_card(
        &directory,
        future,
        DxfEntityField::LAYOUT,
        DxfEntityFieldCardState::AbsentRequired,
        0,
    )?;
    assert_eq!(directory.cards_for_entity(line)?.len(), 19);
    assert_eq!(directory.cards().len(), 38);
    assert_eq!(directory.members().len(), directory.occurrences().len());
    Ok(())
}

#[test]
fn cancellation_source_identity_and_public_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityFieldEvidenceDirectory>();
    assert_copy::<DxfEntityFieldOccurrence>();
    assert_copy::<DxfEntityFieldCard>();
    assert_copy::<DxfEntityFieldCardMember>();
    assert!(std::mem::size_of::<DxfEntityFieldOccurrence>() <= 192);
    assert!(std::mem::size_of::<DxfEntityFieldCard>() <= 160);
    assert!(std::mem::size_of::<DxfEntityFieldCardMember>() <= 8);

    let version = DxfAcadVersion::Ac1032;
    let groups = fixture_groups(version, "FUTURE");
    let bytes = ascii_fixture(version, &groups);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_field_evidence_directory(&DxfCancellationToken::default())?;
    let debug = format!("{directory:?}");
    assert!(!debug.contains("FAMILY_LAYER"));
    assert!(!debug.contains("BYLAYER"));
    assert!(!debug.contains("L1"));

    let other_groups = fixture_groups(version, "OTHER_FUTURE");
    let other_bytes = ascii_fixture(version, &other_groups);
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other_directory =
        other_document.entity_field_evidence_directory(&DxfCancellationToken::default())?;
    let foreign = semantic_entities(&directory)[0];
    assert!(matches!(
        other_directory.cards_for_entity(foreign),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.entity_field_evidence_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.entity_field_evidence_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_field_evidence_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), view.source_id());
    let entities = semantic_entities(&directory);
    assert_eq!(entities.len(), 2);
    assert_eq!(directory.cards().len(), 38);
    let expected_line_occurrences = if version == DxfAcadVersion::Ac1009 {
        7
    } else {
        11
    };
    assert_eq!(
        directory.occurrences_for_entity(entities[0])?.len(),
        expected_line_occurrences
    );
    assert_eq!(directory.occurrences_for_entity(entities[1])?.len(), 2);
    assert_eq!(
        card_state(&directory, entities[0], DxfEntityField::LAYER)?,
        DxfEntityFieldCardState::Duplicate {
            occurrence_count: 2
        }
    );
    assert_eq!(
        card_state(&directory, entities[0], DxfEntityField::PROXY_GRAPHICS_DATA)?,
        if version == DxfAcadVersion::Ac1009 {
            DxfEntityFieldCardState::AbsentOptional
        } else {
            DxfEntityFieldCardState::Sequence {
                occurrence_count: 2,
            }
        }
    );
    assert_eq!(
        card_state(&directory, entities[0], DxfEntityField::OWNER)?,
        if version == DxfAcadVersion::Ac1009 {
            DxfEntityFieldCardState::AbsentRequired
        } else {
            DxfEntityFieldCardState::Unique
        }
    );
    assert!(
        directory
            .entity_directory()
            .entities()
            .iter()
            .any(|entity| matches!(
                entity.classification(),
                seacad_dxf_core::DxfEntityClassification::WrongSection(_)
            ))
    );
    Ok(())
}

fn semantic_entities(
    directory: &DxfEntityFieldEvidenceDirectory,
) -> Vec<seacad_dxf_core::DxfEntityRef> {
    directory
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .filter(|entity| {
            !matches!(
                entity.classification(),
                seacad_dxf_core::DxfEntityClassification::WrongSection(_)
            )
        })
        .collect()
}

fn assert_card(
    directory: &DxfEntityFieldEvidenceDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
    expected_state: DxfEntityFieldCardState,
    expected_members: usize,
) -> Result<(), Box<dyn Error>> {
    let card = directory
        .card_for_field(entity, field)?
        .ok_or(io::Error::other("missing fixed common-field card"))?;
    assert_eq!(card.state(), expected_state);
    assert_eq!(directory.members_for_card(card)?.len(), expected_members);
    for member in directory.members_for_card(card)? {
        assert_eq!(
            directory
                .occurrence_for_member(*member)
                .map(DxfEntityFieldOccurrence::field),
            Some(field)
        );
    }
    Ok(())
}

fn card_state(
    directory: &DxfEntityFieldEvidenceDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<DxfEntityFieldCardState, Box<dyn Error>> {
    directory
        .card_for_field(entity, field)?
        .map(DxfEntityFieldCard::state)
        .ok_or_else(|| io::Error::other("missing fixed common-field card").into())
}

fn exact_values<'a>(
    view: DxfRawDocumentView<'_>,
    occurrences: impl Iterator<Item = &'a DxfEntityFieldOccurrence>,
) -> Result<Vec<Vec<u8>>, DxfError> {
    occurrences
        .map(|occurrence| exact_value(view, occurrence.group()))
        .collect()
}

fn exact_value(
    view: DxfRawDocumentView<'_>,
    group: seacad_dxf_core::DxfRawGroup,
) -> Result<Vec<u8>, DxfError> {
    let len = usize::try_from(group.value_payload_span().len()).map_err(|_| invalid_test_data())?;
    let mut value = vec![0_u8; len];
    view.read_span(group.value_payload_span(), &mut value)?;
    Ok(value)
}

fn occurrence_of(view: DxfRawDocumentView<'_>, value: &[u8]) -> Result<u64, Box<dyn Error>> {
    for raw_ordinal in 0..view.group_count() {
        let Some(group) = view.group(raw_ordinal) else {
            continue;
        };
        if exact_value(view, group)? == value {
            return Ok(raw_ordinal);
        }
    }
    Err(io::Error::other("missing exact fixture value").into())
}

#[derive(Clone, Copy)]
enum FixtureValue<'a> {
    Text(&'a str),
    Int16(i16),
    Int32(i32),
    Double(f64),
    Binary(&'a [u8]),
}

type FixtureGroup<'a> = (i16, FixtureValue<'a>);

fn fixture_groups(version: DxfAcadVersion, unknown: &'static str) -> Vec<FixtureGroup<'static>> {
    let mut groups = vec![
        (0, FixtureValue::Text("SECTION")),
        (2, FixtureValue::Text("TABLES")),
        (0, FixtureValue::Text("TABLE")),
        (5, FixtureValue::Text("BAD")),
        (0, FixtureValue::Text("ENDSEC")),
        (0, FixtureValue::Text("SECTION")),
        (2, FixtureValue::Text("ENTITIES")),
        (0, FixtureValue::Text("LINE")),
        (5, FixtureValue::Text("10")),
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (330, FixtureValue::Text("1F")),
            (102, FixtureValue::Text("{ACAD_REACTORS")),
            (330, FixtureValue::Text("AA")),
            (102, FixtureValue::Text("}")),
            (102, FixtureValue::Text("{ACAD_XDICTIONARY")),
            (360, FixtureValue::Text("BB")),
            (102, FixtureValue::Text("}")),
            (100, FixtureValue::Text("AcDbEntity")),
        ]);
    }
    groups.extend([
        (60, FixtureValue::Int16(0)),
        (8, FixtureValue::Text("L1")),
        (48, FixtureValue::Double(1.0)),
        (8, FixtureValue::Text("L2")),
        (6, FixtureValue::Text("BYLAYER")),
        (92, FixtureValue::Int32(3)),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (310, FixtureValue::Binary(&[1, 2])),
            (310, FixtureValue::Binary(&[3])),
            (100, FixtureValue::Text("AcDbLine")),
            (8, FixtureValue::Text("FAMILY_LAYER")),
            (330, FixtureValue::Text("FAMILY_OWNER")),
            (360, FixtureValue::Text("OUTSIDE")),
        ]);
    }
    groups.extend([
        (0, FixtureValue::Text(unknown)),
        (5, FixtureValue::Text("20")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, FixtureValue::Text("AcDbEntity")));
    }
    groups.extend([
        (8, FixtureValue::Text("U")),
        (0, FixtureValue::Text("ENDSEC")),
        (0, FixtureValue::Text("EOF")),
    ]);
    groups
}

fn ascii_fixture(version: DxfAcadVersion, groups: &[FixtureGroup<'_>]) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n",
        version.code()
    );
    for (code, value) in groups {
        let value = match value {
            FixtureValue::Text(value) => (*value).to_string(),
            FixtureValue::Int16(value) => value.to_string(),
            FixtureValue::Int32(value) => value.to_string(),
            FixtureValue::Double(value) => value.to_string(),
            FixtureValue::Binary(value) => value
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<String>(),
        };
        text.push_str(&format!("{code}\n{value}\n"));
    }
    text.into_bytes()
}

fn binary_fixture(
    version: DxfAcadVersion,
    groups: &[FixtureGroup<'_>],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, FixtureValue::Text("SECTION")),
        (2, FixtureValue::Text("HEADER")),
        (9, FixtureValue::Text("$ACADVER")),
        (1, FixtureValue::Text(version.code())),
        (0, FixtureValue::Text("ENDSEC")),
    ] {
        push_binary_group(&mut bytes, version, code, value)?;
    }
    for (code, value) in groups {
        push_binary_group(&mut bytes, version, *code, *value)?;
    }
    Ok(bytes)
}

fn push_binary_group(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: FixtureValue<'_>,
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    match value {
        FixtureValue::Text(value) => {
            bytes.extend_from_slice(value.as_bytes());
            bytes.push(0);
        }
        FixtureValue::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        FixtureValue::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        FixtureValue::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        FixtureValue::Binary(value) => {
            bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk length"))?);
            bytes.extend_from_slice(value);
        }
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
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

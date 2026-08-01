use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityAlias, DxfEntityClassification, DxfEntityDirectory,
    DxfEntityKnownClassification, DxfEntityRef, DxfEntitySubclassMarker, DxfEntityTopic, DxfError,
    DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
    dxf_entity_aliases, dxf_entity_topics,
};

#[test]
fn every_supported_version_has_ascii_binary_entity_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), "future_entity");
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        assert_directory(DxfRawDocumentView::from(&ascii))?;

        let binary_bytes = binary_fixture(version, "future_entity")?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_directory(DxfRawDocumentView::from(&binary))?;
    }
    Ok(())
}

#[test]
fn subclass_paths_are_exact_ordered_and_exclude_application_content() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii_fixture("AC1032", "future_entity");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = view.entity_directory(&DxfCancellationToken::default())?;
    let block_line = directory
        .entity_for_raw_ordinal(3)
        .ok_or(io::Error::other("missing block LINE"))?;

    let path = directory.subclass_path(block_line)?;
    assert_eq!(path.len(), 2);
    assert_eq!(
        exact_values(view, path)?,
        [b"AcDbLine".to_vec(), b"AcDbEntity".to_vec()]
    );
    assert!(path[0].group().occurrence() < path[1].group().occurrence());
    assert_eq!(path[0].record(), block_line.record());
    assert_eq!(block_line.subclass_range().len(), 2);
    assert!(!block_line.subclass_range().is_empty());

    let hidden = occurrence_of(view, 100, b"HiddenSubclass")?;
    assert!(
        directory
            .subclass_markers()
            .iter()
            .all(|marker| marker.group().occurrence() != hidden)
    );
    Ok(())
}

#[test]
fn classification_is_exact_placement_aware_and_block_controls_are_not_entities()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032", "line");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.entities().len(), 7);
    assert!(directory.entities().iter().all(|entity| {
        !matches!(
            entity.record().section_kind(),
            seacad_dxf_core::DxfRawRecordSectionKind::Blocks
        ) || entity.record().section_record_ordinal() == 1
    }));
    assert_eq!(
        directory
            .entity_for_raw_ordinal(7)
            .map(DxfEntityRef::classification),
        Some(DxfEntityClassification::Unknown)
    );
    assert_eq!(directory.entity_for_raw_ordinal(2), None);
    assert_eq!(directory.entity_for_raw_ordinal(4), None);
    assert_eq!(directory.entity_for_raw_ordinal(9), None);
    Ok(())
}

#[test]
fn lookups_source_identity_cancellation_and_public_bounds_are_enforced()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityDirectory>();
    assert_copy::<DxfEntityRef>();
    assert_copy::<DxfEntitySubclassMarker>();
    assert_send_sync::<DxfEntityRef>();
    assert_send_sync::<DxfEntitySubclassMarker>();
    let entity_size = std::mem::size_of::<DxfEntityRef>();
    assert!(
        entity_size <= 128,
        "entity metadata grew to {entity_size} bytes"
    );
    assert!(std::mem::size_of::<DxfEntitySubclassMarker>() <= 72);

    let bytes = ascii_fixture("AC1032", "future_entity");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_directory(&DxfCancellationToken::default())?;
    let entity = directory.entities()[2];
    assert_eq!(directory.raw_record_count(), 10);
    assert_eq!(
        directory.entity_for_group(entity.marker().occurrence()),
        Some(entity)
    );
    assert_eq!(
        directory.entity_for_group(entity.record().group_range().end() - 1),
        Some(entity)
    );
    assert_eq!(directory.entity_for_group(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("AcDbLine"));
    assert!(!debug.contains("HiddenSubclass"));
    assert!(!debug.contains("future_entity"));

    let other_bytes = ascii_fixture("AC1032", "different_future_entity");
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other_directory = other_document.entity_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        other_directory.subclass_path(entity),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.entity_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.entity_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

#[test]
fn interrupted_and_unclosed_sections_never_leak_partial_entities() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n100\nLeaked\n0\nSECTION\n2\nOBJECTS\n0\nLINE\n100\nIndexed\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nLINE\n100\nLeakedToo\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = view.entity_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.entities().len(), 1);
    assert!(matches!(
        directory.entities()[0].classification(),
        DxfEntityClassification::WrongSection(DxfEntityKnownClassification::Canonical(topic))
            if topic == DxfEntityTopic::LINE
    ));
    assert_eq!(
        exact_values(view, directory.subclass_markers())?,
        [b"Indexed".to_vec()]
    );
    Ok(())
}

#[test]
fn reviewed_names_fit_the_bounded_classifier_buffer() {
    assert!(
        dxf_entity_topics()
            .iter()
            .all(|descriptor| descriptor.dxf_name().len() <= 64)
    );
    assert!(
        dxf_entity_aliases()
            .iter()
            .all(|descriptor| descriptor.dxf_name().len() <= 64)
    );
}

fn assert_directory(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.raw_record_count(), 10);
    assert_eq!(directory.entities().len(), 7);
    let expected = [
        (
            0,
            DxfEntityClassification::WrongSection(DxfEntityKnownClassification::Canonical(
                DxfEntityTopic::LINE,
            )),
        ),
        (
            1,
            DxfEntityClassification::WrongSection(DxfEntityKnownClassification::Canonical(
                DxfEntityTopic::TABLE,
            )),
        ),
        (3, DxfEntityClassification::Canonical(DxfEntityTopic::LINE)),
        (5, DxfEntityClassification::Canonical(DxfEntityTopic::LINE)),
        (
            6,
            DxfEntityClassification::Alias(DxfEntityAlias::PDFUNDERLAY),
        ),
        (7, DxfEntityClassification::Unknown),
        (
            8,
            DxfEntityClassification::WrongSection(DxfEntityKnownClassification::Canonical(
                DxfEntityTopic::MLEADERSTYLE,
            )),
        ),
    ];
    for (entity, (raw_ordinal, classification)) in directory.entities().iter().zip(expected) {
        assert_eq!(entity.source_id(), view.source_id());
        assert_eq!(entity.record().ordinal(), raw_ordinal);
        assert_eq!(
            entity.marker().occurrence(),
            entity.record().marker_occurrence()
        );
        assert_eq!(entity.classification(), classification);
        assert_eq!(classification.topic(), entity.classification().topic());
        assert_eq!(directory.entity_for_raw_ordinal(raw_ordinal), Some(*entity));
    }
    assert_eq!(
        exact_value(view, directory.entities()[2].marker())?,
        b"LINE"
    );
    assert_eq!(
        exact_value(view, directory.entities()[4].marker())?,
        b"PDFUNDERLAY"
    );
    assert_eq!(
        exact_value(view, directory.entities()[5].marker())?,
        b"future_entity"
    );
    Ok(())
}

fn exact_values(
    view: DxfRawDocumentView<'_>,
    markers: &[DxfEntitySubclassMarker],
) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut values = Vec::new();
    for marker in markers {
        values.push(exact_value(view, marker.group())?);
    }
    Ok(values)
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

fn occurrence_of(
    view: DxfRawDocumentView<'_>,
    code: i16,
    value: &[u8],
) -> Result<u64, Box<dyn Error>> {
    for occurrence in 0..view.group_count() {
        let Some(group) = view.group(occurrence) else {
            continue;
        };
        if group.group_code().value() == code && exact_value(view, group)? == value {
            return Ok(occurrence);
        }
    }
    Err(io::Error::other("missing group").into())
}

fn ascii_fixture(version: &str, unknown: &str) -> Vec<u8> {
    let mut text = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n");
    push_ascii_section(&mut text, "CLASSES", &[(0, "LINE")]);
    push_ascii_section(&mut text, "TABLES", &[(0, "TABLE"), (2, "LAYER")]);
    push_ascii_section(
        &mut text,
        "BLOCKS",
        &[
            (0, "BLOCK"),
            (2, "B"),
            (0, "LINE"),
            (100, "AcDbLine"),
            (102, "{APP"),
            (100, "HiddenSubclass"),
            (102, "}"),
            (100, "AcDbEntity"),
            (0, "ENDBLK"),
        ],
    );
    push_ascii_section(
        &mut text,
        "ENTITIES",
        &[
            (0, "LINE"),
            (100, "AcDbLine"),
            (0, "PDFUNDERLAY"),
            (100, "AcDbUnderlayReference"),
            (0, unknown),
            (100, "FutureSubclass"),
        ],
    );
    push_ascii_section(
        &mut text,
        "OBJECTS",
        &[
            (0, "MLEADERSTYLE"),
            (100, "AcDbMLeaderStyle"),
            (0, "DICTIONARY"),
        ],
    );
    text.push_str("0\nEOF\n");
    text.into_bytes()
}

fn push_ascii_section(text: &mut String, name: &str, groups: &[(i16, &str)]) {
    text.push_str(&format!("0\nSECTION\n2\n{name}\n"));
    for (code, value) in groups {
        text.push_str(&format!("{code}\n{value}\n"));
    }
    text.push_str("0\nENDSEC\n");
}

fn binary_fixture(version: DxfAcadVersion, unknown: &str) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary_section(
        &mut bytes,
        version,
        "HEADER",
        &[(9, "$ACADVER"), (1, version.code())],
    )?;
    push_binary_section(&mut bytes, version, "CLASSES", &[(0, "LINE")])?;
    push_binary_section(&mut bytes, version, "TABLES", &[(0, "TABLE"), (2, "LAYER")])?;
    push_binary_section(
        &mut bytes,
        version,
        "BLOCKS",
        &[
            (0, "BLOCK"),
            (2, "B"),
            (0, "LINE"),
            (100, "AcDbLine"),
            (102, "{APP"),
            (100, "HiddenSubclass"),
            (102, "}"),
            (100, "AcDbEntity"),
            (0, "ENDBLK"),
        ],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "ENTITIES",
        &[
            (0, "LINE"),
            (100, "AcDbLine"),
            (0, "PDFUNDERLAY"),
            (100, "AcDbUnderlayReference"),
            (0, unknown),
            (100, "FutureSubclass"),
        ],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "OBJECTS",
        &[
            (0, "MLEADERSTYLE"),
            (100, "AcDbMLeaderStyle"),
            (0, "DICTIONARY"),
        ],
    )?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_section(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &str,
    groups: &[(i16, &str)],
) -> Result<(), io::Error> {
    push_binary_string(bytes, version, 0, b"SECTION")?;
    push_binary_string(bytes, version, 2, name.as_bytes())?;
    for (code, value) in groups {
        push_binary_string(bytes, version, *code, value.as_bytes())?;
    }
    push_binary_string(bytes, version, 0, b"ENDSEC")
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    bytes.extend_from_slice(value);
    bytes.push(0);
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

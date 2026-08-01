use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfEntityField,
    DxfEntityFieldSemanticDirectory, DxfEntityFieldSemanticEntry, DxfEntityFieldSemanticIssue,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfError, DxfMemorySource, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValue, DxfSemanticValueState,
    NoopDxfReadObserver,
};

type StateSignature = Vec<(DxfEntityField, u8, u32)>;

#[test]
fn every_dialect_has_ascii_binary_typed_common_field_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let groups = parity_groups(version);
        let ascii_bytes = ascii_fixture(version, &groups);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.entity_field_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, &groups)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.entity_field_semantic_directory(&DxfCancellationToken::default())?;

        assert_expected(DxfRawDocumentView::from(&ascii), &ascii_directory, version)?;
        assert_expected(
            DxfRawDocumentView::from(&binary),
            &binary_directory,
            version,
        )?;
        assert_eq!(
            state_signature(&ascii_directory)?,
            state_signature(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn defaults_duplicates_invalid_values_and_opaque_sequences_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nLINE\n5\nG\n100\nAcDbEntity\n8\nA\n8\nB\n48\n.\n\
310\n0102\n310\n03\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_field_semantic_directory(&DxfCancellationToken::default())?;
    let entity = only_entity(&directory)?;

    assert_issue(
        singleton(&directory, entity, DxfEntityField::HANDLE)?,
        DxfEntityFieldSemanticIssue::InvalidHandle(
            seacad_dxf_core::DxfHandleParseIssue::InvalidDigit { offset: 0 },
        ),
        true,
    )?;
    assert_issue(
        singleton(&directory, entity, DxfEntityField::LAYER)?,
        DxfEntityFieldSemanticIssue::MultipleValues {
            occurrence_count: 2,
        },
        false,
    )?;
    assert_issue(
        singleton(&directory, entity, DxfEntityField::LINETYPE_SCALE)?,
        DxfEntityFieldSemanticIssue::InvalidAsciiNumber(DxfAsciiNumericIssue::InvalidSyntax {
            token_offset: 1,
        }),
        true,
    )?;
    assert_eq!(
        singleton(&directory, entity, DxfEntityField::LINETYPE)?.value(),
        Some(&DxfEntityFieldValue::SchemaExactText("BYLAYER"))
    );
    assert_eq!(
        singleton(&directory, entity, DxfEntityField::MATERIAL)?.value(),
        Some(&DxfEntityFieldValue::ByLayer)
    );
    assert_eq!(
        singleton(&directory, entity, DxfEntityField::VISIBILITY)?.value(),
        Some(&DxfEntityFieldValue::Int16(0))
    );
    assert_issue(
        singleton(&directory, entity, DxfEntityField::LAYOUT)?,
        DxfEntityFieldSemanticIssue::MissingRequired,
        false,
    )?;
    assert_eq!(
        singleton(&directory, entity, DxfEntityField::COLOR_NAME)?.state(),
        DxfSemanticValueState::Absent
    );
    let proxy = entry(&directory, entity, DxfEntityField::PROXY_GRAPHICS_DATA)?;
    assert_eq!(
        proxy.semantics(),
        DxfEntityFieldSemantics::OpaqueSequence {
            occurrence_count: 2
        }
    );
    assert_eq!(
        directory
            .members_for_opaque_sequence(proxy)?
            .ok_or(io::Error::other("proxy members"))?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn binary_non_finite_double_fails_with_exact_raw_provenance() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let groups = [
        (0, FixtureValue::Text("SECTION")),
        (2, FixtureValue::Text("ENTITIES")),
        (0, FixtureValue::Text("LINE")),
        (5, FixtureValue::Text("10")),
        (100, FixtureValue::Text("AcDbEntity")),
        (48, FixtureValue::Double(f64::NAN)),
        (0, FixtureValue::Text("ENDSEC")),
        (0, FixtureValue::Text("EOF")),
    ];
    let bytes = binary_fixture(version, &groups)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.entity_field_semantic_directory(&DxfCancellationToken::default())?;
    let value = singleton(
        &directory,
        only_entity(&directory)?,
        DxfEntityField::LINETYPE_SCALE,
    )?;
    let DxfSemanticValue::Invalid {
        issue: DxfEntityFieldSemanticIssue::NonFiniteDouble(number),
        raw: Some(raw),
        ..
    } = value
    else {
        return Err(io::Error::other("non-finite common double").into());
    };
    assert!(number.to_f64().is_nan());
    assert!(!raw.value_span().is_empty());
    Ok(())
}

#[test]
fn cancellation_source_identity_text_decode_and_public_bounds_fail_closed()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityFieldSemanticDirectory>();
    assert_copy::<DxfEntityFieldSemanticEntry>();
    assert_copy::<DxfEntityFieldSemantics>();
    let entry_size = std::mem::size_of::<DxfEntityFieldSemanticEntry>();
    assert!(entry_size <= 320, "semantic entry size: {entry_size}");

    let version = DxfAcadVersion::Ac1032;
    let groups = parity_groups(version);
    let bytes = ascii_fixture(version, &groups);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_field_semantic_directory(&DxfCancellationToken::default())?;
    let entity = only_entity(&directory)?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entries_for_entity(entity)?.len(), 19);

    let layer = singleton(&directory, entity, DxfEntityField::LAYER)?;
    let Some(DxfEntityFieldValue::ExactText(text)) = layer.value().copied() else {
        return Err(io::Error::other("exact layer text").into());
    };
    let mut decoded = [0_u8; 5];
    let receipt =
        text.decode_to_utf8_without_replacement(DxfRawDocumentView::from(&document), &mut decoded)?;
    let written = receipt
        .decode_result()
        .ok_or(io::Error::other("available layer decoder"))?
        .written();
    assert_eq!(&decoded[..written], b"Layer");

    let mut other_groups = parity_groups(version);
    let insert_at = other_groups
        .len()
        .checked_sub(2)
        .ok_or(io::Error::other("other fixture boundary"))?;
    other_groups.insert(insert_at, (999, FixtureValue::Text("OTHER")));
    let other_bytes = ascii_fixture(version, &other_groups);
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let other_directory = other.entity_field_semantic_directory(&DxfCancellationToken::default())?;
    let foreign = only_entity(&other_directory)?;
    assert!(matches!(
        directory.entries_for_entity(foreign),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(matches!(
        text.decode_to_utf8_without_replacement(DxfRawDocumentView::from(&other), &mut decoded),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.entity_field_semantic_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.entity_field_semantic_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_expected(
    view: DxfRawDocumentView<'_>,
    directory: &DxfEntityFieldSemanticDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.entries().len(), 19);
    let entity = only_entity(directory)?;
    assert_eq!(
        singleton(directory, entity, DxfEntityField::HANDLE)?.value(),
        Some(&DxfEntityFieldValue::Handle(
            seacad_dxf_core::DxfHandle::from_u64(0x10)
        ))
    );
    assert_eq!(
        singleton(directory, entity, DxfEntityField::COLOR)?.value(),
        Some(&DxfEntityFieldValue::Int16(3))
    );
    assert_eq!(
        singleton(directory, entity, DxfEntityField::LINETYPE_SCALE)?.value(),
        Some(&DxfEntityFieldValue::Double(
            seacad_dxf_core::DxfDouble::from_f64(2.5)
        ))
    );
    assert_eq!(
        singleton(directory, entity, DxfEntityField::VISIBILITY)?.value(),
        Some(&DxfEntityFieldValue::Int16(1))
    );
    let layer = singleton(directory, entity, DxfEntityField::LAYER)?;
    let Some(DxfEntityFieldValue::ExactText(text)) = layer.value().copied() else {
        return Err(io::Error::other("layer text").into());
    };
    let mut exact = [0_u8; 5];
    view.read_span(text.value_span(), &mut exact)?;
    assert_eq!(&exact, b"Layer");
    assert_eq!(
        singleton(directory, entity, DxfEntityField::PAPER_SPACE)?.state(),
        DxfSemanticValueState::Defaulted
    );
    if version != DxfAcadVersion::Ac1009 {
        for (field, handle) in [
            (DxfEntityField::OWNER, 0x1f),
            (DxfEntityField::EXTENSION_DICTIONARY, 0x2f),
            (DxfEntityField::MATERIAL, 0x30),
            (DxfEntityField::PLOT_STYLE, 0x40),
        ] {
            assert_eq!(
                singleton(directory, entity, field)?.value(),
                Some(&DxfEntityFieldValue::Handle(
                    seacad_dxf_core::DxfHandle::from_u64(handle)
                ))
            );
        }
        for (field, value) in [
            (DxfEntityField::PROXY_GRAPHICS_SIZE, 3),
            (DxfEntityField::TRUE_COLOR, 0x112233),
            (DxfEntityField::TRANSPARENCY, 0x02000000),
        ] {
            assert_eq!(
                singleton(directory, entity, field)?.value(),
                Some(&DxfEntityFieldValue::Int32(value))
            );
        }
        assert_eq!(
            singleton(directory, entity, DxfEntityField::LINEWEIGHT)?.value(),
            Some(&DxfEntityFieldValue::Int16(25))
        );
        assert_eq!(
            singleton(directory, entity, DxfEntityField::SHADOW)?.value(),
            Some(&DxfEntityFieldValue::Int16(3))
        );
        assert_eq!(
            singleton(directory, entity, DxfEntityField::LAYOUT)?.state(),
            DxfSemanticValueState::Explicit
        );
        assert_eq!(
            singleton(directory, entity, DxfEntityField::COLOR_NAME)?.state(),
            DxfSemanticValueState::Explicit
        );
    }
    let proxy = entry(directory, entity, DxfEntityField::PROXY_GRAPHICS_DATA)?;
    assert_eq!(
        proxy.semantics(),
        DxfEntityFieldSemantics::OpaqueSequence {
            occurrence_count: if version == DxfAcadVersion::Ac1009 {
                0
            } else {
                2
            }
        }
    );
    Ok(())
}

fn state_signature(
    directory: &DxfEntityFieldSemanticDirectory,
) -> Result<StateSignature, Box<dyn Error>> {
    let mut signature = Vec::new();
    for entry in directory.entries().iter().copied() {
        let (state, count) = match entry.semantics() {
            DxfEntityFieldSemantics::Singleton(value) => (value.state() as u8, 0),
            DxfEntityFieldSemantics::OpaqueSequence { occurrence_count } => {
                (u8::MAX, occurrence_count)
            }
            _ => return Err(io::Error::other("unknown common semantics").into()),
        };
        signature.push((entry.field(), state, count));
    }
    Ok(signature)
}

fn assert_issue(
    value: DxfEntityFieldSemanticValue,
    expected: DxfEntityFieldSemanticIssue,
    has_raw: bool,
) -> Result<(), Box<dyn Error>> {
    let DxfSemanticValue::Invalid { issue, raw, .. } = value else {
        return Err(io::Error::other("invalid common singleton").into());
    };
    assert_eq!(issue, expected);
    assert_eq!(raw.is_some(), has_raw);
    Ok(())
}

type DxfEntityFieldSemanticValue = seacad_dxf_core::DxfEntityFieldSemanticValue;

fn singleton(
    directory: &DxfEntityFieldSemanticDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<DxfEntityFieldSemanticValue, Box<dyn Error>> {
    match entry(directory, entity, field)?.semantics() {
        DxfEntityFieldSemantics::Singleton(value) => Ok(value),
        _ => Err(io::Error::other("common singleton").into()),
    }
}

fn entry(
    directory: &DxfEntityFieldSemanticDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<DxfEntityFieldSemanticEntry, Box<dyn Error>> {
    directory
        .entry_for_field(entity, field)?
        .ok_or_else(|| io::Error::other("common semantic entry").into())
}

fn only_entity(
    directory: &DxfEntityFieldSemanticDirectory,
) -> Result<seacad_dxf_core::DxfEntityRef, Box<dyn Error>> {
    let [entity] = directory.evidence_directory().entity_directory().entities() else {
        return Err(io::Error::other("one semantic entity").into());
    };
    Ok(*entity)
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

fn parity_groups(version: DxfAcadVersion) -> Vec<FixtureGroup<'static>> {
    let mut groups = vec![
        (0, FixtureValue::Text("SECTION")),
        (2, FixtureValue::Text("ENTITIES")),
        (0, FixtureValue::Text("LINE")),
        (5, FixtureValue::Text("10")),
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (330, FixtureValue::Text("1F")),
            (102, FixtureValue::Text("{ACAD_XDICTIONARY")),
            (360, FixtureValue::Text("2F")),
            (102, FixtureValue::Text("}")),
            (100, FixtureValue::Text("AcDbEntity")),
            (410, FixtureValue::Text("Model")),
        ]);
    }
    groups.extend([
        (8, FixtureValue::Text("Layer")),
        (6, FixtureValue::Text("DASHED")),
        (62, FixtureValue::Int16(3)),
        (48, FixtureValue::Double(2.5)),
        (60, FixtureValue::Int16(1)),
        (92, FixtureValue::Int32(3)),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (347, FixtureValue::Text("30")),
            (370, FixtureValue::Int16(25)),
            (310, FixtureValue::Binary(&[1, 2])),
            (310, FixtureValue::Binary(&[3])),
            (420, FixtureValue::Int32(0x112233)),
            (430, FixtureValue::Text("Named")),
            (440, FixtureValue::Int32(0x02000000)),
            (390, FixtureValue::Text("40")),
            (284, FixtureValue::Int16(3)),
        ]);
    }
    groups.extend([
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonLayoutIssue, DxfEntityCommonLayoutSemanticValue,
    DxfEntityCommonSymbolIssue, DxfEntityCommonSymbolSemanticValue, DxfEntityCommonSymbolValue,
    DxfEntityCommonTextDirectory, DxfEntityCommonTextSemantics, DxfEntityField,
    DxfEntityFieldSemanticIssue, DxfEntityFieldValue, DxfError, DxfLayoutObjectNameState,
    DxfMemorySource, DxfNamedSymbolTableKind, DxfRawDocumentFormat, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValue, DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_common_text_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = valid_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_text = ascii.entity_common_text_directory(&token())?;

        let binary_bytes = valid_fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_text = binary.entity_common_text_directory(&token())?;

        assert_valid(&ascii_text, version)?;
        assert_valid(&binary_text, version)?;
        assert_eq!(signature(&ascii_text)?, signature(&binary_text)?);
    }
    Ok(())
}

#[test]
fn missing_and_ambiguous_exact_symbol_names_remain_distinct() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let groups = negative_name_groups();
        let bytes = encode(format, DxfAcadVersion::Ac1032, &groups)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        match format {
            DxfRawDocumentFormat::Ascii => {
                let document = open_ascii(&source)?;
                assert_name_failures(&document.entity_common_text_directory(&token())?)?;
            }
            DxfRawDocumentFormat::Binary => {
                let document = open_binary(&source)?;
                assert_name_failures(&document.entity_common_text_directory(&token())?)?;
            }
            _ => return Err(io::Error::other("test format").into()),
        }
    }
    Ok(())
}

#[test]
fn layout_names_are_subclass_aware_and_keep_cardinality() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let groups = layout_cardinality_groups();
        let bytes = encode(format, DxfAcadVersion::Ac1032, &groups)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let states: Vec<_> = match format {
            DxfRawDocumentFormat::Ascii => open_ascii(&source)?
                .layout_object_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.name())
                .collect(),
            DxfRawDocumentFormat::Binary => open_binary(&source)?
                .layout_object_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.name())
                .collect(),
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(states.len(), 4);
        assert!(matches!(states[0], DxfLayoutObjectNameState::Unique(_)));
        assert_eq!(states[1], DxfLayoutObjectNameState::Missing);
        assert!(matches!(
            states[2],
            DxfLayoutObjectNameState::Duplicate {
                occurrence_count: 2,
                ..
            }
        ));
        assert!(matches!(states[3], DxfLayoutObjectNameState::Unique(_)));
    }
    Ok(())
}

#[test]
fn field_failures_bylayer_default_and_unreviewed_text_stay_exact() -> Result<(), Box<dyn Error>> {
    let groups = field_failure_groups();
    let bytes = encode(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, &groups)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let texts = document.entity_common_text_directory(&token())?;
    let entity = only_entity(&texts)?;

    assert!(matches!(
        symbol(&texts, entity, DxfEntityField::LAYER)?,
        DxfSemanticValue::Invalid {
            issue: DxfEntityCommonSymbolIssue::Field(DxfEntityFieldSemanticIssue::MultipleValues {
                occurrence_count: 2
            }),
            raw: None,
            ..
        }
    ));
    assert_eq!(
        symbol(&texts, entity, DxfEntityField::LINETYPE)?.state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        symbol(&texts, entity, DxfEntityField::LINETYPE)?.value(),
        Some(&DxfEntityCommonSymbolValue::ByLayer)
    );
    assert_eq!(
        layout(&texts, entity)?.state(),
        DxfSemanticValueState::Explicit
    );
    assert!(matches!(
        entry(&texts, entity, DxfEntityField::COLOR_NAME)?.semantics(),
        DxfEntityCommonTextSemantics::ExactUnreviewed(DxfSemanticValue::Explicit {
            value: DxfEntityFieldValue::ExactText(_),
            ..
        })
    ));
    assert!(!format!("{texts:?}").contains("SECRET"));
    Ok(())
}

#[test]
fn cancellation_source_bound_lookup_and_public_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityCommonTextDirectory>();
    assert_copy::<seacad_dxf_core::DxfEntityCommonTextEntry>();
    assert_copy::<DxfEntityCommonTextSemantics>();
    let entry_size = std::mem::size_of::<seacad_dxf_core::DxfEntityCommonTextEntry>();
    assert!(entry_size <= 640, "text entry size: {entry_size}");

    let bytes = valid_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_common_text_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let texts = document.entity_common_text_directory(&token())?;
    let entity = only_entity(&texts)?;
    assert_eq!(texts.source_id(), document.source_id());
    assert_eq!(texts.source_directory().source_id(), document.source_id());
    assert_eq!(
        texts.named_symbol_table_directory().source_id(),
        document.source_id()
    );
    assert_eq!(
        texts.layout_object_directory().source_id(),
        document.source_id()
    );
    assert_eq!(texts.entries().len(), 4);
    assert_eq!(texts.entries_for_entity(entity)?.len(), 4);
    assert_eq!(texts.entry(u64::MAX), None);
    assert_eq!(texts.entry_for_field(entity, DxfEntityField::COLOR)?, None);

    let other_bytes = valid_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let other_texts = other.entity_common_text_directory(&token())?;
    assert!(matches!(
        texts.entries_for_entity(only_entity(&other_texts)?),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn assert_valid(
    texts: &DxfEntityCommonTextDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(texts.entries().len(), 4);
    let entity = only_entity(texts)?;
    for (field, kind) in [
        (DxfEntityField::LAYER, DxfNamedSymbolTableKind::Layer),
        (DxfEntityField::LINETYPE, DxfNamedSymbolTableKind::Linetype),
    ] {
        let value = symbol(texts, entity, field)?;
        let Some(DxfEntityCommonSymbolValue::ExactMatch { text, target }) = value.value() else {
            return Err(io::Error::other("exact symbol match").into());
        };
        assert_eq!(target.kind(), kind);
        assert_eq!(text.source_id(), texts.source_id());
        assert_eq!(value.state(), DxfSemanticValueState::Explicit);
    }
    let color_name = exact(texts, entity, DxfEntityField::COLOR_NAME)?;
    if version == DxfAcadVersion::Ac1009 {
        assert!(matches!(
            layout(texts, entity)?,
            DxfSemanticValue::Invalid {
                issue: DxfEntityCommonLayoutIssue::Field(
                    DxfEntityFieldSemanticIssue::MissingRequired
                ),
                ..
            }
        ));
        assert_eq!(color_name.state(), DxfSemanticValueState::Absent);
    } else {
        let layout = layout(texts, entity)?;
        assert_eq!(layout.state(), DxfSemanticValueState::Explicit);
        assert!(layout.value().is_some());
        assert_eq!(color_name.state(), DxfSemanticValueState::Explicit);
    }
    Ok(())
}

fn assert_name_failures(texts: &DxfEntityCommonTextDirectory) -> Result<(), Box<dyn Error>> {
    let entity = only_entity(texts)?;
    let layer = symbol(texts, entity, DxfEntityField::LAYER)?;
    assert_eq!(
        layer.invalid_issue(),
        Some(&DxfEntityCommonSymbolIssue::Ambiguous { target_count: 2 })
    );
    assert!(layer.raw_provenance().is_some());
    let linetype = symbol(texts, entity, DxfEntityField::LINETYPE)?;
    assert_eq!(
        linetype.invalid_issue(),
        Some(&DxfEntityCommonSymbolIssue::Missing)
    );
    assert!(linetype.raw_provenance().is_some());
    let layout = layout(texts, entity)?;
    assert_eq!(
        layout.invalid_issue(),
        Some(&DxfEntityCommonLayoutIssue::Ambiguous { target_count: 2 })
    );
    assert!(layout.raw_provenance().is_some());
    Ok(())
}

type Signature = Vec<(DxfEntityField, u8, Option<DxfNamedSymbolTableKind>)>;

fn signature(texts: &DxfEntityCommonTextDirectory) -> Result<Signature, Box<dyn Error>> {
    let mut result = Vec::new();
    for entry in texts.entries().iter().copied() {
        let (state, kind) = match entry.semantics() {
            DxfEntityCommonTextSemantics::Symbol(value) => {
                let kind = match value.value() {
                    Some(DxfEntityCommonSymbolValue::ExactMatch { target, .. }) => {
                        Some(target.kind())
                    }
                    _ => None,
                };
                (value.state() as u8, kind)
            }
            DxfEntityCommonTextSemantics::Layout(value) => (value.state() as u8, None),
            DxfEntityCommonTextSemantics::ExactUnreviewed(value) => (value.state() as u8, None),
            _ => return Err(io::Error::other("unknown common text semantics").into()),
        };
        result.push((entry.field(), state, kind));
    }
    Ok(result)
}

fn layout(
    texts: &DxfEntityCommonTextDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
) -> Result<DxfEntityCommonLayoutSemanticValue, Box<dyn Error>> {
    match entry(texts, entity, DxfEntityField::LAYOUT)?.semantics() {
        DxfEntityCommonTextSemantics::Layout(value) => Ok(value),
        _ => Err(io::Error::other("layout semantics").into()),
    }
}

fn symbol(
    texts: &DxfEntityCommonTextDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<DxfEntityCommonSymbolSemanticValue, Box<dyn Error>> {
    match entry(texts, entity, field)?.semantics() {
        DxfEntityCommonTextSemantics::Symbol(value) => Ok(value),
        _ => Err(io::Error::other("symbol semantics").into()),
    }
}

fn exact(
    texts: &DxfEntityCommonTextDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<seacad_dxf_core::DxfEntityFieldSemanticValue, Box<dyn Error>> {
    match entry(texts, entity, field)?.semantics() {
        DxfEntityCommonTextSemantics::ExactUnreviewed(value) => Ok(value),
        _ => Err(io::Error::other("exact semantics").into()),
    }
}

fn entry(
    texts: &DxfEntityCommonTextDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<seacad_dxf_core::DxfEntityCommonTextEntry, Box<dyn Error>> {
    texts
        .entry_for_field(entity, field)?
        .ok_or_else(|| io::Error::other("common text entry").into())
}

fn only_entity(
    texts: &DxfEntityCommonTextDirectory,
) -> Result<seacad_dxf_core::DxfEntityRef, Box<dyn Error>> {
    let Some(entity) = texts.entries().first().map(|entry| entry.entity()) else {
        return Err(io::Error::other("one entity").into());
    };
    if texts
        .entries()
        .iter()
        .any(|entry| entry.entity().record().ordinal() != entity.record().ordinal())
    {
        return Err(io::Error::other("one entity").into());
    }
    Ok(entity)
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
}

fn valid_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version);
    groups.extend(symbol_table(b"LAYER", &[(b"LayerA", b"A")]));
    groups.extend(symbol_table(b"LTYPE", &[(b"Dash", b"B")]));
    groups.extend(layout_objects(&[b"SECRET_LAYOUT"]));
    groups.extend(entity_start());
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (100, Value::Text(b"AcDbEntity")),
            (410, Value::Text(b"SECRET_LAYOUT")),
            (430, Value::Text(b"SECRET_COLOR")),
        ]);
    }
    groups.extend([(8, Value::Text(b"LayerA")), (6, Value::Text(b"Dash"))]);
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
}

fn negative_name_groups() -> Vec<(i16, Value<'static>)> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend(symbol_table(
        b"LAYER",
        &[(b"LayerA", b"A"), (b"LayerA", b"B")],
    ));
    groups.extend(symbol_table(b"LTYPE", &[(b"Other", b"C")]));
    groups.extend(layout_objects(&[b"SECRET_LAYOUT", b"SECRET_LAYOUT"]));
    groups.extend(entity_start());
    groups.extend([
        (100, Value::Text(b"AcDbEntity")),
        (410, Value::Text(b"SECRET_LAYOUT")),
        (8, Value::Text(b"LayerA")),
        (6, Value::Text(b"Missing")),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    groups
}

fn field_failure_groups() -> Vec<(i16, Value<'static>)> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend(symbol_table(b"LAYER", &[(b"LayerA", b"A")]));
    groups.extend(symbol_table(b"LTYPE", &[(b"Dash", b"B")]));
    groups.extend(layout_objects(&[b"SECRET_LAYOUT"]));
    groups.extend(entity_start());
    groups.extend([
        (100, Value::Text(b"AcDbEntity")),
        (410, Value::Text(b"SECRET_LAYOUT")),
        (430, Value::Text(b"SECRET_COLOR")),
        (8, Value::Text(b"LayerA")),
        (8, Value::Text(b"LayerA")),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    groups
}

fn header(version: DxfAcadVersion) -> Vec<(i16, Value<'static>)> {
    vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
    ]
}

fn symbol_table(
    kind: &'static [u8],
    entries: &[(&'static [u8], &'static [u8])],
) -> Vec<(i16, Value<'static>)> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(kind)),
    ];
    for (name, handle) in entries {
        groups.extend([
            (0, Value::Text(kind)),
            (2, Value::Text(name)),
            (5, Value::Text(handle)),
        ]);
    }
    groups.extend([(0, Value::Text(b"ENDTAB")), (0, Value::Text(b"ENDSEC"))]);
    groups
}

fn layout_objects(names: &[&'static [u8]]) -> Vec<(i16, Value<'static>)> {
    let mut groups = vec![(0, Value::Text(b"SECTION")), (2, Value::Text(b"OBJECTS"))];
    for name in names {
        groups.extend([
            (0, Value::Text(b"LAYOUT")),
            (100, Value::Text(b"AcDbPlotSettings")),
            (1, Value::Text(b"PAGE_SETUP")),
            (100, Value::Text(b"AcDbLayout")),
            (1, Value::Text(name)),
        ]);
    }
    groups.extend([(0, Value::Text(b"ENDSEC"))]);
    groups
}

fn layout_cardinality_groups() -> Vec<(i16, Value<'static>)> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"OBJECTS")),
        (0, Value::Text(b"LAYOUT")),
        (100, Value::Text(b"AcDbPlotSettings")),
        (1, Value::Text(b"NOT_THE_LAYOUT_NAME")),
        (100, Value::Text(b"AcDbLayout")),
        (1, Value::Text(b"Model")),
        (0, Value::Text(b"LAYOUT")),
        (100, Value::Text(b"AcDbLayout")),
        (0, Value::Text(b"LAYOUT")),
        (100, Value::Text(b"AcDbLayout")),
        (1, Value::Text(b"First")),
        (1, Value::Text(b"Second")),
        (0, Value::Text(b"LAYOUT")),
        (100, Value::Text(b"AcDbLayout")),
        (102, Value::Text(b"{APP")),
        (1, Value::Text(b"APPLICATION_DATA")),
        (102, Value::Text(b"}")),
        (1, Value::Text(b"Paper")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LAYOUT")),
        (100, Value::Text(b"AcDbLayout")),
        (1, Value::Text(b"WRONG_SECTION")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    groups
}

fn entity_start() -> Vec<(i16, Value<'static>)> {
    vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ]
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, Value::Text(value)) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(value);
        bytes.extend_from_slice(b"\r\n");
    }
    bytes
}

fn binary_groups(
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, Value::Text(value)) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        bytes.extend_from_slice(value);
        bytes.push(0);
    }
    Ok(bytes)
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut observer)
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut observer)
}

fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonFieldPatch, DxfEntityCommonSymbolEditIssue,
    DxfEntityCommonSymbolEditOutcome, DxfEntityCommonSymbolValue, DxfEntityCommonTextSemantics,
    DxfEntityEditIssue, DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityEditVerificationOutcome,
    DxfEntityField, DxfEntityPatch, DxfError, DxfMemorySource, DxfNamedSymbolTableKind,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfRawRecordSectionKind, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_valid_symbol_edit_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = fixture(DxfRawDocumentFormat::Ascii, version, false)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = fixture(DxfRawDocumentFormat::Binary, version, false)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;

        for (field, name, kind) in valid_cases() {
            let ascii_value = valid(ascii.classify_entity_common_symbol_edit(
                field,
                DxfEntityEditValue::ExactRawText(name),
                &token(),
            )?)?;
            let binary_value = valid(binary.classify_entity_common_symbol_edit(
                field,
                DxfEntityEditValue::ExactRawText(name),
                &token(),
            )?)?;
            assert_eq!(ascii_value.field(), field);
            assert_eq!(ascii_value.kind(), kind);
            assert_eq!(ascii_value.target().kind(), kind);
            assert_eq!(binary_value.field(), field);
            assert_eq!(binary_value.kind(), kind);
            assert_eq!(binary_value.target().kind(), kind);
        }
        assert!(matches!(
            ascii.classify_entity_common_symbol_edit(
                DxfEntityField::COLOR,
                DxfEntityEditValue::Int16(7),
                &token(),
            )?,
            DxfEntityCommonSymbolEditOutcome::NotSymbol {
                field: DxfEntityField::COLOR
            }
        ));
    }
    Ok(())
}

#[test]
fn missing_ambiguous_unreviewed_and_wrong_kind_fail_typed() -> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityCommonSymbolEditIssue>();
    assert_copy_send_sync::<DxfEntityCommonSymbolEditOutcome>();
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, true)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        assert!(matches!(
            invalid(view.classify_entity_common_symbol_edit(
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(b"NewLayer"),
                &token(),
            )?)?,
            DxfEntityCommonSymbolEditIssue::Ambiguous {
                field: DxfEntityField::LAYER,
                kind: DxfNamedSymbolTableKind::Layer,
                target_count: 2,
            }
        ));
        assert_eq!(
            invalid(view.classify_entity_common_symbol_edit(
                DxfEntityField::LINETYPE,
                DxfEntityEditValue::ExactRawText(b"Missing"),
                &token(),
            )?)?,
            DxfEntityCommonSymbolEditIssue::Missing {
                field: DxfEntityField::LINETYPE,
                kind: DxfNamedSymbolTableKind::Linetype,
            }
        );
        assert!(matches!(
            invalid(view.classify_entity_common_symbol_edit(
                DxfEntityField::LAYER,
                DxfEntityEditValue::Int16(1),
                &token(),
            )?)?,
            DxfEntityCommonSymbolEditIssue::ValueKindMismatch { .. }
        ));
        assert_eq!(
            invalid(view.classify_entity_common_symbol_edit(
                DxfEntityField::LAYOUT,
                DxfEntityEditValue::ExactRawText(b"Model"),
                &token(),
            )?)?,
            DxfEntityCommonSymbolEditIssue::LayoutResolutionRequired {
                field: DxfEntityField::LAYOUT,
            }
        );
        assert_eq!(
            invalid(view.classify_entity_common_symbol_edit(
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(b"Book$Color"),
                &token(),
            )?)?,
            DxfEntityCommonSymbolEditIssue::ColorBookResolutionRequired {
                field: DxfEntityField::COLOR_NAME,
            }
        );
    }
    Ok(())
}

#[test]
fn every_dialect_session_materializes_symbols_and_exact_inverse() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, false)?;
            verify_session(&bytes, format, version)?;
        }
    }
    Ok(())
}

#[test]
fn rejected_symbols_never_queue_and_cancellation_precedes_scan() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, true)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        let evidence = view.entity_field_evidence_directory(&token())?;
        let key = evidence
            .entity_directory()
            .entities()
            .iter()
            .find(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities)
            .ok_or(io::Error::other("entity"))?
            .key();
        let cancellation = token();
        let mut session =
            view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
        for (field, value) in [
            (
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(b"NewLayer"),
            ),
            (
                DxfEntityField::LINETYPE,
                DxfEntityEditValue::ExactRawText(b"Missing"),
            ),
            (
                DxfEntityField::LAYOUT,
                DxfEntityEditValue::ExactRawText(b"Model"),
            ),
            (
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(b"Book$Color"),
            ),
        ] {
            assert!(matches!(
                session.update(key, set(field, value))?,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Symbol(_))
            ));
        }
        assert_eq!(session.queued_edit_count(), 0);

        let cancelled = token();
        cancelled.cancel();
        assert!(matches!(
            view.classify_entity_common_symbol_edit(
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(b"NewLayer"),
                &cancelled,
            ),
            Err(DxfError::Cancelled)
        ));
    }
    Ok(())
}

fn verify_session(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_document(&source, format)?;
    let view = document.view();
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = evidence
        .entity_directory()
        .entities()
        .iter()
        .find(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities)
        .ok_or(io::Error::other("entity"))?
        .key();
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    for (field, name, _) in valid_cases() {
        let outcome = session.update(key, set(field, DxfEntityEditValue::ExactRawText(name)))?;
        assert!(
            matches!(outcome, DxfEntityEditOutcome::Applied(_)),
            "{version:?} {format:?} {field:?}: {outcome:?}"
        );
    }
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 2);
    let output = materialize(bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let post_document = open_document(&output_source, format)?;
    let post = post_document.view();
    assert_symbol_targets(post)?;
    let DxfEntityEditVerificationOutcome::Verified(journal) =
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
    else {
        return Err(io::Error::other("verified symbol edits").into());
    };
    assert_eq!(journal.receipt().edit_count(), 2);
    assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
    Ok(())
}

fn assert_symbol_targets(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_common_text_directory(&token())?;
    let entity = directory
        .entries()
        .iter()
        .map(|entry| entry.entity())
        .find(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities)
        .ok_or(io::Error::other("entity"))?;
    for (field, _, kind) in valid_cases() {
        let entry = directory
            .entry_for_field(entity, field)?
            .ok_or(io::Error::other("symbol field"))?;
        let DxfEntityCommonTextSemantics::Symbol(value) = entry.semantics() else {
            return Err(io::Error::other("symbol semantics").into());
        };
        assert_eq!(value.state(), DxfSemanticValueState::Explicit);
        assert!(matches!(
            value.value(),
            Some(DxfEntityCommonSymbolValue::ExactMatch { target, .. })
                if target.kind() == kind
        ));
    }
    Ok(())
}

fn valid_cases() -> [(DxfEntityField, &'static [u8], DxfNamedSymbolTableKind); 2] {
    [
        (
            DxfEntityField::LAYER,
            b"NewLayer",
            DxfNamedSymbolTableKind::Layer,
        ),
        (
            DxfEntityField::LINETYPE,
            b"NewType",
            DxfNamedSymbolTableKind::Linetype,
        ),
    ]
}

fn valid(
    outcome: DxfEntityCommonSymbolEditOutcome,
) -> Result<seacad_dxf_core::DxfEntityCommonSymbolEditValue, io::Error> {
    match outcome {
        DxfEntityCommonSymbolEditOutcome::Valid(value) => Ok(value),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn invalid(
    outcome: DxfEntityCommonSymbolEditOutcome,
) -> Result<DxfEntityCommonSymbolEditIssue, io::Error> {
    match outcome {
        DxfEntityCommonSymbolEditOutcome::Invalid(issue) => Ok(issue),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn set(field: DxfEntityField, value: DxfEntityEditValue<'_>) -> DxfEntityPatch<'_> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit { field, value })
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    duplicate_layer: bool,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version);
    let mut layers = vec![(b"OLD".as_slice(), b"A".as_slice()), (b"NewLayer", b"B")];
    if duplicate_layer {
        layers.push((b"NewLayer", b"C"));
    }
    groups.extend(symbol_table(b"LAYER", &layers));
    groups.extend(symbol_table(
        b"LTYPE",
        &[(b"OLDTYPE", b"D"), (b"NewType", b"E")],
    ));
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    groups.extend([(8, Value::Text(b"OLD")), (6, Value::Text(b"OLDTYPE"))]);
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
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

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("format")),
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

fn materialize(bytes: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::with_capacity(
        usize::try_from(plan.projected_len()).map_err(|_| io::Error::other("length"))?,
    );
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start =
            usize::try_from(patch.source_span().start()).map_err(|_| io::Error::other("start"))?;
        let end =
            usize::try_from(patch.source_span().end()).map_err(|_| io::Error::other("end"))?;
        output.extend_from_slice(bytes.get(cursor..start).ok_or(io::Error::other("source"))?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or(io::Error::other("replacement"))?,
        );
        cursor = end;
    }
    output.extend_from_slice(bytes.get(cursor..).ok_or(io::Error::other("tail"))?);
    Ok(output)
}

enum OpenedDocument<'a> {
    Ascii(DxfAsciiRawDocument<'a>),
    Binary(DxfBinaryRawDocument<'a>),
}

impl OpenedDocument<'_> {
    fn view(&self) -> DxfRawDocumentView<'_> {
        match self {
            Self::Ascii(document) => DxfRawDocumentView::from(document),
            Self::Binary(document) => DxfRawDocumentView::from(document),
        }
    }
}

fn open_document<'a>(
    source: &'a dyn DxfByteSource,
    format: DxfRawDocumentFormat,
) -> Result<OpenedDocument<'a>, Box<dyn Error>> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(OpenedDocument::Ascii(open_ascii(source)?)),
        DxfRawDocumentFormat::Binary => Ok(OpenedDocument::Binary(open_binary(source)?)),
        _ => Err(io::Error::other("format").into()),
    }
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

fn assert_copy_send_sync<T: Copy + Send + Sync>() {}

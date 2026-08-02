use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonFieldPatch, DxfEntityCommonLayoutEditIssue,
    DxfEntityCommonLayoutEditOutcome, DxfEntityCommonTextSemantics, DxfEntityEditIssue,
    DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityEditVerificationOutcome, DxfEntityField,
    DxfEntityPatch, DxfError, DxfLayoutObjectNameState, DxfMemorySource, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfRawRecordSectionKind, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_layout_classifier_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, LayoutFixture::Valid)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let value = valid(document.view().classify_entity_common_layout_edit(
                DxfEntityField::LAYOUT,
                DxfEntityEditValue::ExactRawText(b"Paper"),
                &token(),
            )?)?;
            assert_eq!(value.field(), DxfEntityField::LAYOUT);
            assert_eq!(
                value.target().record().section_kind(),
                DxfRawRecordSectionKind::Objects
            );
            assert!(matches!(
                value.target().name(),
                DxfLayoutObjectNameState::Unique(_)
            ));
            assert_eq!(
                document.view().classify_entity_common_layout_edit(
                    DxfEntityField::COLOR,
                    DxfEntityEditValue::Int16(7),
                    &token(),
                )?,
                DxfEntityCommonLayoutEditOutcome::NotLayout {
                    field: DxfEntityField::COLOR
                }
            );
        }
    }
    Ok(())
}

#[test]
fn missing_ambiguous_wrong_value_and_case_fail_typed() -> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityCommonLayoutEditIssue>();
    assert_copy_send_sync::<DxfEntityCommonLayoutEditOutcome>();
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, LayoutFixture::Duplicate)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        assert_eq!(
            invalid(view.classify_entity_common_layout_edit(
                DxfEntityField::LAYOUT,
                DxfEntityEditValue::ExactRawText(b"Paper"),
                &token(),
            )?)?,
            DxfEntityCommonLayoutEditIssue::Ambiguous {
                field: DxfEntityField::LAYOUT,
                target_count: 2,
            }
        );
        for name in [
            b"Missing".as_slice(),
            b"paper".as_slice(),
            b"Malformed".as_slice(),
        ] {
            assert_eq!(
                invalid(view.classify_entity_common_layout_edit(
                    DxfEntityField::LAYOUT,
                    DxfEntityEditValue::ExactRawText(name),
                    &token(),
                )?)?,
                DxfEntityCommonLayoutEditIssue::Missing {
                    field: DxfEntityField::LAYOUT
                }
            );
        }
        assert!(matches!(
            invalid(view.classify_entity_common_layout_edit(
                DxfEntityField::LAYOUT,
                DxfEntityEditValue::Int16(1),
                &token(),
            )?)?,
            DxfEntityCommonLayoutEditIssue::ValueKindMismatch { .. }
        ));
    }
    Ok(())
}

#[test]
fn modern_sessions_materialize_verify_and_restore_layout_edits() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED
        .into_iter()
        .filter(|version| *version != DxfAcadVersion::Ac1009)
    {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, LayoutFixture::Valid)?;
            verify_session(&bytes, format)?;
        }
    }
    Ok(())
}

#[test]
fn rejected_layouts_never_queue_and_ac1009_binary_stays_unencodable() -> Result<(), Box<dyn Error>>
{
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, LayoutFixture::Duplicate)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        let evidence = view.entity_field_evidence_directory(&token())?;
        let key = entity_key(&evidence)?;
        let cancellation = token();
        let mut session =
            view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
        for name in [b"Paper".as_slice(), b"Missing".as_slice()] {
            assert!(matches!(
                session.update(key, set_layout(name))?,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Layout(_))
            ));
        }
        assert_eq!(session.queued_edit_count(), 0);

        let cancelled = token();
        cancelled.cancel();
        assert!(matches!(
            view.classify_entity_common_layout_edit(
                DxfEntityField::LAYOUT,
                DxfEntityEditValue::ExactRawText(b"Paper"),
                &cancelled,
            ),
            Err(DxfError::Cancelled)
        ));
    }

    let bytes = fixture(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        LayoutFixture::Valid,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = entity_key(&evidence)?;
    let cancellation = token();
    let mut session =
        document.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(key, set_layout(b"Paper"))?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Insertion(_))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    Ok(())
}

fn verify_session(bytes: &[u8], format: DxfRawDocumentFormat) -> Result<(), Box<dyn Error>> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_document(&source, format)?;
    let view = document.view();
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = entity_key(&evidence)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(key, set_layout(b"Paper"))?,
        DxfEntityEditOutcome::Applied(_)
    ));
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 1);
    let output = materialize(bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let post_document = open_document(&output_source, format)?;
    let post = post_document.view();
    let texts = post.entity_common_text_directory(&token())?;
    let entity = texts
        .entries()
        .iter()
        .map(|entry| entry.entity())
        .find(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities)
        .ok_or(io::Error::other("post entity"))?;
    let entry = texts
        .entry_for_field(entity, DxfEntityField::LAYOUT)?
        .ok_or(io::Error::other("layout semantic"))?;
    let DxfEntityCommonTextSemantics::Layout(value) = entry.semantics() else {
        return Err(io::Error::other("layout semantic").into());
    };
    assert_eq!(value.state(), DxfSemanticValueState::Explicit);
    assert!(value.value().is_some());
    let DxfEntityEditVerificationOutcome::Verified(journal) =
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
    else {
        return Err(io::Error::other("verified layout edit").into());
    };
    assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
    Ok(())
}

fn entity_key(
    evidence: &seacad_dxf_core::DxfEntityFieldEvidenceDirectory,
) -> Result<seacad_dxf_core::DxfEntityKey, io::Error> {
    evidence
        .entity_directory()
        .entities()
        .iter()
        .find(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities)
        .map(|entity| entity.key())
        .ok_or(io::Error::other("entity"))
}

fn set_layout(name: &[u8]) -> DxfEntityPatch<'_> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
        field: DxfEntityField::LAYOUT,
        value: DxfEntityEditValue::ExactRawText(name),
    })
}

fn valid(
    outcome: DxfEntityCommonLayoutEditOutcome,
) -> Result<seacad_dxf_core::DxfEntityCommonLayoutEditValue, io::Error> {
    match outcome {
        DxfEntityCommonLayoutEditOutcome::Valid(value) => Ok(value),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn invalid(
    outcome: DxfEntityCommonLayoutEditOutcome,
) -> Result<DxfEntityCommonLayoutEditIssue, io::Error> {
    match outcome {
        DxfEntityCommonLayoutEditOutcome::Invalid(issue) => Ok(issue),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

#[derive(Clone, Copy)]
enum LayoutFixture {
    Valid,
    Duplicate,
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    fixture: LayoutFixture,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"OBJECTS")),
    ];
    groups.extend(layout(b"Model", false));
    groups.extend(layout(b"Paper", false));
    if matches!(fixture, LayoutFixture::Duplicate) {
        groups.extend(layout(b"Paper", false));
    }
    groups.extend(layout(b"Malformed", true));
    groups.extend([
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    groups.push((8, Value::Text(b"0")));
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
}

fn layout(name: &'static [u8], duplicate: bool) -> Vec<(i16, Value<'static>)> {
    let mut groups = vec![
        (0, Value::Text(b"LAYOUT")),
        (100, Value::Text(b"AcDbPlotSettings")),
        (1, Value::Text(b"PAGE_SETUP")),
        (100, Value::Text(b"AcDbLayout")),
        (1, Value::Text(name)),
    ];
    if duplicate {
        groups.push((1, Value::Text(name)));
    }
    groups
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => {
            let mut bytes = Vec::new();
            for (code, Value::Text(value)) in groups {
                bytes.extend_from_slice(code.to_string().as_bytes());
                bytes.extend_from_slice(b"\r\n");
                bytes.extend_from_slice(value);
                bytes.extend_from_slice(b"\r\n");
            }
            Ok(bytes)
        }
        DxfRawDocumentFormat::Binary => {
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
        _ => Err(io::Error::other("format")),
    }
}

fn materialize(bytes: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start = usize::try_from(patch.source_span().start()).map_err(io::Error::other)?;
        let end = usize::try_from(patch.source_span().end()).map_err(io::Error::other)?;
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

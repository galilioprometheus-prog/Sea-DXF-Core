use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonColorBookEditIssue, DxfEntityCommonColorBookEditOutcome,
    DxfEntityCommonFieldPatch, DxfEntityCommonTextSemantics, DxfEntityEditIssue,
    DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityEditVerificationOutcome, DxfEntityField,
    DxfEntityPatch, DxfError, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValue, DxfSemanticValueState,
    DxfTransactionPlan, NoopDxfReadObserver,
};

const PROPOSED: &[u8] = b"RAL CLASSIC$RAL 1004";

#[test]
fn every_dialect_has_ascii_binary_color_book_admission_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let mut signature = None;
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, Relation::Valid)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let entity = only_entity(view)?;
            let outcome = view.classify_entity_common_color_book_edit(
                entity,
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(PROPOSED),
                &token(),
            )?;
            let state = match outcome {
                DxfEntityCommonColorBookEditOutcome::Valid(value) => {
                    assert_ne!(version, DxfAcadVersion::Ac1009);
                    assert_eq!(value.field(), DxfEntityField::COLOR_NAME);
                    assert_eq!(value.book_name(PROPOSED), Some(b"RAL CLASSIC".as_slice()));
                    assert_eq!(value.color_name(PROPOSED), Some(b"RAL 1004".as_slice()));
                    assert_eq!(value.book_name(b"short"), None);
                    assert_eq!(value.indexed_color().raw(), 40);
                    assert_eq!(value.true_color().raw(), 16_235_019);
                    1_u8
                }
                DxfEntityCommonColorBookEditOutcome::Invalid(
                    DxfEntityCommonColorBookEditIssue::RelatedColor {
                        related_field: DxfEntityField::TRUE_COLOR,
                        semantics: DxfSemanticValue::Absent { .. },
                        ..
                    },
                ) => {
                    assert_eq!(version, DxfAcadVersion::Ac1009);
                    0
                }
                other => return Err(io::Error::other(format!("admission: {other:?}")).into()),
            };
            assert_eq!(*signature.get_or_insert(state), state);
        }
    }
    Ok(())
}

#[test]
fn syntax_relation_kind_source_and_cancellation_fail_typed() -> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityCommonColorBookEditIssue>();
    assert_copy_send_sync::<DxfEntityCommonColorBookEditOutcome>();
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, Relation::Valid)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        let entity = only_entity(view)?;
        for (proposed, expected) in [
            (b"RAL".as_slice(), 0_u8),
            (b"$RAL".as_slice(), 1),
            (b"RAL$".as_slice(), 2),
            (b"RAL$A$B".as_slice(), 3),
        ] {
            let issue = invalid(view.classify_entity_common_color_book_edit(
                entity,
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(proposed),
                &token(),
            )?)?;
            let observed = match issue {
                DxfEntityCommonColorBookEditIssue::MissingSeparator { .. } => 0,
                DxfEntityCommonColorBookEditIssue::EmptyBookName { .. } => 1,
                DxfEntityCommonColorBookEditIssue::EmptyColorName { .. } => 2,
                DxfEntityCommonColorBookEditIssue::MultipleSeparators {
                    separator_count: 2,
                    ..
                } => 3,
                _ => return Err(io::Error::other("syntax issue").into()),
            };
            assert_eq!(observed, expected);
        }
        assert!(matches!(
            invalid(view.classify_entity_common_color_book_edit(
                entity,
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::Int16(7),
                &token(),
            )?)?,
            DxfEntityCommonColorBookEditIssue::ValueKindMismatch { .. }
        ));
        assert!(matches!(
            view.classify_entity_common_color_book_edit(
                entity,
                DxfEntityField::COLOR,
                DxfEntityEditValue::Int16(7),
                &token(),
            )?,
            DxfEntityCommonColorBookEditOutcome::NotColorBook { .. }
        ));

        let missing_bytes = fixture(format, DxfAcadVersion::Ac1032, Relation::MissingTrueColor)?;
        let missing_source = DxfMemorySource::new(&missing_bytes, DxfResourceProfile::Safe)?;
        let missing = open_document(&missing_source, format)?;
        assert!(matches!(
            invalid(missing.view().classify_entity_common_color_book_edit(
                only_entity(missing.view())?,
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(PROPOSED),
                &token(),
            )?)?,
            DxfEntityCommonColorBookEditIssue::RelatedColor {
                related_field: DxfEntityField::TRUE_COLOR,
                semantics: DxfSemanticValue::Absent { .. },
                ..
            }
        ));
        for (relation, related_field) in [
            (Relation::InvalidTrueColor, DxfEntityField::TRUE_COLOR),
            (Relation::DuplicateIndexedColor, DxfEntityField::COLOR),
        ] {
            let invalid_bytes = fixture(format, DxfAcadVersion::Ac1032, relation)?;
            let invalid_source = DxfMemorySource::new(&invalid_bytes, DxfResourceProfile::Safe)?;
            let invalid_document = open_document(&invalid_source, format)?;
            assert!(matches!(
                invalid(invalid_document.view().classify_entity_common_color_book_edit(
                    only_entity(invalid_document.view())?,
                    DxfEntityField::COLOR_NAME,
                    DxfEntityEditValue::ExactRawText(PROPOSED),
                    &token(),
                )?)?,
                DxfEntityCommonColorBookEditIssue::RelatedColor {
                    related_field: observed,
                    semantics: DxfSemanticValue::Invalid { .. },
                    ..
                } if observed == related_field
            ));
        }

        let other_bytes = fixture(format, DxfAcadVersion::Ac1027, Relation::Valid)?;
        let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
        let other = open_document(&other_source, format)?;
        assert!(matches!(
            view.classify_entity_common_color_book_edit(
                only_entity(other.view())?,
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(PROPOSED),
                &token(),
            ),
            Err(DxfError::SourceIdentityMismatch { .. })
        ));
        let cancelled = token();
        cancelled.cancel();
        assert!(matches!(
            view.classify_entity_common_color_book_edit(
                entity,
                DxfEntityField::COLOR_NAME,
                DxfEntityEditValue::ExactRawText(PROPOSED),
                &cancelled,
            ),
            Err(DxfError::Cancelled)
        ));
    }
    Ok(())
}

#[test]
fn modern_sessions_verify_color_book_semantics_and_exact_inverse() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for relation in [Relation::Valid, Relation::ValidWithoutName] {
                let bytes = fixture(format, version, relation)?;
                let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
                let document = open_document(&source, format)?;
                let view = document.view();
                let evidence = view.entity_field_evidence_directory(&token())?;
                let entity = only_entity(view)?;
                let key = entity.key();
                let cancellation = token();
                let mut session =
                    view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
                let outcome = session.update(key, set(PROPOSED))?;
                if version == DxfAcadVersion::Ac1009 {
                    assert!(matches!(
                        outcome,
                        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::ColorBook(_))
                    ));
                    assert_eq!(session.queued_edit_count(), 0);
                    continue;
                }
                assert!(matches!(outcome, DxfEntityEditOutcome::Applied(_)));
                let plan = session.finish_verifiable()?;
                let output = materialize(&bytes, plan.transaction())?;
                let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                let post_document = open_document(&output_source, format)?;
                let post = post_document.view();
                let texts = post.entity_common_text_directory(&token())?;
                let post_entity = only_entity(post)?;
                let entry = texts
                    .entry_for_field(post_entity, DxfEntityField::COLOR_NAME)?
                    .ok_or(io::Error::other("color-name entry"))?;
                let DxfEntityCommonTextSemantics::ColorBook(value) = entry.semantics() else {
                    return Err(io::Error::other("color-book semantics").into());
                };
                assert_eq!(value.state(), DxfSemanticValueState::Explicit);
                assert_eq!(
                    value.value().map(|value| value.true_color().raw()),
                    Some(16_235_019)
                );
                let DxfEntityEditVerificationOutcome::Verified(journal) =
                    plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
                else {
                    return Err(io::Error::other("verified color-book edit").into());
                };
                assert_eq!(journal.receipt().edit_count(), 1);
                assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
            }
        }
    }
    Ok(())
}

fn invalid(
    outcome: DxfEntityCommonColorBookEditOutcome,
) -> Result<DxfEntityCommonColorBookEditIssue, io::Error> {
    match outcome {
        DxfEntityCommonColorBookEditOutcome::Invalid(issue) => Ok(issue),
        other => Err(io::Error::other(format!("expected invalid: {other:?}"))),
    }
}

fn set(value: &[u8]) -> DxfEntityPatch<'_> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
        field: DxfEntityField::COLOR_NAME,
        value: DxfEntityEditValue::ExactRawText(value),
    })
}

#[derive(Clone, Copy)]
enum Relation {
    Valid,
    ValidWithoutName,
    MissingTrueColor,
    InvalidTrueColor,
    DuplicateIndexedColor,
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Int32(i32),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    relation: Relation,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbEntity")));
        if !matches!(relation, Relation::ValidWithoutName) {
            groups.push((430, Value::Text(b"RAL CLASSIC$RAL 1003")));
        }
        groups.push((62, Value::Int16(40)));
        if !matches!(relation, Relation::MissingTrueColor) {
            groups.push((
                420,
                Value::Int32(if matches!(relation, Relation::InvalidTrueColor) {
                    0x01_00_00_00
                } else {
                    16_235_019
                }),
            ));
        }
        if matches!(relation, Relation::DuplicateIndexedColor) {
            groups.push((62, Value::Int16(41)));
        }
        groups.push((100, Value::Text(b"AcDbLine")));
    } else {
        groups.push((62, Value::Int16(40)));
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = if format == DxfRawDocumentFormat::Binary {
        DXF_BINARY_SENTINEL.to_vec()
    } else {
        Vec::new()
    };
    for (code, value) in groups {
        match format {
            DxfRawDocumentFormat::Ascii => {
                bytes.extend_from_slice(code.to_string().as_bytes());
                bytes.extend_from_slice(b"\r\n");
                match value {
                    Value::Text(value) => bytes.extend_from_slice(value),
                    Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
                    Value::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
                }
                bytes.extend_from_slice(b"\r\n");
            }
            DxfRawDocumentFormat::Binary => {
                if version == DxfAcadVersion::Ac1009 {
                    bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("group code"))?);
                } else {
                    bytes.extend_from_slice(&code.to_le_bytes());
                }
                match value {
                    Value::Text(value) => {
                        bytes.extend_from_slice(value);
                        bytes.push(0);
                    }
                    Value::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                    Value::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                }
            }
            _ => return Err(io::Error::other("format")),
        }
    }
    Ok(bytes)
}

fn only_entity(view: DxfRawDocumentView<'_>) -> Result<seacad_dxf_core::DxfEntityRef, DxfError> {
    let directory = view.entity_directory(&token())?;
    let [entity] = directory.entities() else {
        return Err(DxfError::from_io(
            seacad_dxf_core::DxfIoOperation::Read,
            &io::Error::other("one entity"),
        ));
    };
    Ok(*entity)
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

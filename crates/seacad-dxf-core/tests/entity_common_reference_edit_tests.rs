use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonFieldPatch, DxfEntityCommonReferenceEditIssue,
    DxfEntityCommonReferenceEditOutcome, DxfEntityCommonReferenceTargetKind,
    DxfEntityCommonReferenceTargetSemantics, DxfEntityCommonReferenceValue, DxfEntityEditIssue,
    DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityEditVerificationOutcome, DxfEntityField,
    DxfEntityFieldInsertionIssue, DxfEntityGroupEncodeIssue, DxfEntityPatch, DxfError, DxfHandle,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfRawRecordSectionKind,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, DxfTransactionPlan,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_valid_reference_edit_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = classifier_fixture(DxfRawDocumentFormat::Ascii, version, false)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let binary_bytes = classifier_fixture(DxfRawDocumentFormat::Binary, version, false)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;

        for (field, handle, kind) in reviewed_cases() {
            let ascii_value = valid(ascii.classify_entity_common_reference_edit(
                field,
                DxfEntityEditValue::Handle(handle),
                &token(),
            )?)?;
            let binary_value = valid(binary.classify_entity_common_reference_edit(
                field,
                DxfEntityEditValue::Handle(handle),
                &token(),
            )?)?;
            assert_eq!(ascii_value.field(), field);
            assert_eq!(ascii_value.handle(), handle);
            assert_eq!(ascii_value.expected_target_kind(), kind);
            assert_eq!(ascii_value.target().handle(), handle);
            assert_eq!(binary_value.field(), field);
            assert_eq!(binary_value.handle(), handle);
            assert_eq!(binary_value.expected_target_kind(), kind);
            assert_eq!(binary_value.target().handle(), handle);
        }
        assert!(matches!(
            ascii.classify_entity_common_reference_edit(
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(b"0"),
                &token(),
            )?,
            DxfEntityCommonReferenceEditOutcome::NotReference {
                field: DxfEntityField::LAYER
            }
        ));
    }
    Ok(())
}

#[test]
fn invalid_targets_and_specialized_operations_fail_typed() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = classifier_fixture(format, DxfAcadVersion::Ac1032, true)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        let cases = [
            (
                DxfEntityField::HANDLE,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x20)),
                DxfEntityCommonReferenceEditIssue::HandleRemapRequired {
                    field: DxfEntityField::HANDLE,
                },
            ),
            (
                DxfEntityField::OWNER,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x1f)),
                DxfEntityCommonReferenceEditIssue::OwnerPlacementRequired {
                    field: DxfEntityField::OWNER,
                },
            ),
            (
                DxfEntityField::MATERIAL,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0)),
                DxfEntityCommonReferenceEditIssue::Null {
                    field: DxfEntityField::MATERIAL,
                },
            ),
            (
                DxfEntityField::MATERIAL,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x99)),
                DxfEntityCommonReferenceEditIssue::Missing {
                    field: DxfEntityField::MATERIAL,
                    handle: DxfHandle::from_u64(0x99),
                },
            ),
            (
                DxfEntityField::EXTENSION_DICTIONARY,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x2f)),
                DxfEntityCommonReferenceEditIssue::Ambiguous {
                    field: DxfEntityField::EXTENSION_DICTIONARY,
                    handle: DxfHandle::from_u64(0x2f),
                    target_count: 2,
                },
            ),
        ];
        for (field, value, expected) in cases {
            assert_eq!(
                invalid(view.classify_entity_common_reference_edit(field, value, &token(),)?)?,
                expected
            );
        }
        assert!(matches!(
            invalid(view.classify_entity_common_reference_edit(
                DxfEntityField::MATERIAL,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x1f)),
                &token(),
            )?)?,
            DxfEntityCommonReferenceEditIssue::IncompatibleTarget {
                field: DxfEntityField::MATERIAL,
                expected: DxfEntityCommonReferenceTargetKind::Material,
                target,
            } if target.handle() == DxfHandle::from_u64(0x1f)
        ));
        assert!(matches!(
            invalid(view.classify_entity_common_reference_edit(
                DxfEntityField::PLOT_STYLE,
                DxfEntityEditValue::Int16(4),
                &token(),
            )?)?,
            DxfEntityCommonReferenceEditIssue::ValueKindMismatch { .. }
        ));

        let wrong_section_bytes = wrong_section_fixture(format)?;
        let wrong_section_source =
            DxfMemorySource::new(&wrong_section_bytes, DxfResourceProfile::Safe)?;
        let wrong_section_document = open_document(&wrong_section_source, format)?;
        assert!(matches!(
            invalid(wrong_section_document.view().classify_entity_common_reference_edit(
                DxfEntityField::MATERIAL,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x3f)),
                &token(),
            )?)?,
            DxfEntityCommonReferenceEditIssue::IncompatibleTarget { target, .. }
                if target.record().section_kind() == DxfRawRecordSectionKind::Tables
        ));
    }
    Ok(())
}

#[test]
fn modern_sessions_materialize_valid_reference_updates_and_exact_inverse()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        if version == DxfAcadVersion::Ac1009 {
            continue;
        }
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = session_fixture(format, version)?;
            verify_session(&bytes, format)?;
        }
    }
    Ok(())
}

#[test]
fn session_rejections_never_queue_and_ac1009_stays_wire_typed() -> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityCommonReferenceEditIssue>();
    assert_copy_send_sync::<DxfEntityCommonReferenceEditOutcome>();

    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = classifier_fixture(format, DxfAcadVersion::Ac1032, false)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        let evidence = view.entity_field_evidence_directory(&token())?;
        let key = evidence.entity_directory().entities()[0].key();
        let cancellation = token();
        let mut session =
            view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
        for (field, value) in [
            (
                DxfEntityField::HANDLE,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x20)),
            ),
            (
                DxfEntityField::OWNER,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x1f)),
            ),
            (
                DxfEntityField::MATERIAL,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x1f)),
            ),
        ] {
            assert!(matches!(
                session.update(key, set(field, value))?,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Reference(_))
            ));
        }
        assert_eq!(session.queued_edit_count(), 0);

        let cancelled = token();
        cancelled.cancel();
        assert!(matches!(
            view.classify_entity_common_reference_edit(
                DxfEntityField::MATERIAL,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x3f)),
                &cancelled,
            ),
            Err(DxfError::Cancelled)
        ));

        let bytes = classifier_fixture(format, DxfAcadVersion::Ac1009, false)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_document(&source, format)?;
        let view = document.view();
        let evidence = view.entity_field_evidence_directory(&token())?;
        let key = evidence.entity_directory().entities()[0].key();
        let cancellation = token();
        let mut session =
            view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
        let outcome = session.update(
            key,
            set(
                DxfEntityField::MATERIAL,
                DxfEntityEditValue::Handle(DxfHandle::from_u64(0x3f)),
            ),
        )?;
        match format {
            DxfRawDocumentFormat::Ascii => {
                assert!(matches!(outcome, DxfEntityEditOutcome::Applied(_)));
                assert_eq!(session.queued_edit_count(), 1);
            }
            DxfRawDocumentFormat::Binary => {
                assert!(matches!(
                    outcome,
                    DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Insertion(
                        DxfEntityFieldInsertionIssue::Encoding(
                            DxfEntityGroupEncodeIssue::GroupCodeUnavailableInDialect {
                                group_code: 347,
                                version: DxfAcadVersion::Ac1009,
                            }
                        )
                    ))
                ));
                assert_eq!(session.queued_edit_count(), 0);
            }
            _ => return Err(io::Error::other("format").into()),
        }
    }
    Ok(())
}

fn verify_session(bytes: &[u8], format: DxfRawDocumentFormat) -> Result<(), Box<dyn Error>> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_document(&source, format)?;
    let view = document.view();
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = evidence.entity_directory().entities()[0].key();
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    for (field, handle, _) in reviewed_cases() {
        assert!(matches!(
            session.update(key, set(field, DxfEntityEditValue::Handle(handle)))?,
            DxfEntityEditOutcome::Applied(_)
        ));
    }
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.transaction().patches().len(), 3);
    let output = materialize(bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let post_document = open_document(&output_source, format)?;
    let post = post_document.view();
    assert_reviewed_handles(post)?;
    let DxfEntityEditVerificationOutcome::Verified(journal) =
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
    else {
        return Err(io::Error::other("verified reference edits").into());
    };
    assert_eq!(journal.receipt().edit_count(), 3);
    assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
    Ok(())
}

fn assert_reviewed_handles(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_common_reference_target_directory(&token())?;
    let entity = directory
        .source_directory()
        .source_directory()
        .evidence_directory()
        .entity_directory()
        .entities()[0];
    for (field, handle, _) in reviewed_cases() {
        let entry = directory
            .entry_for_field(entity, field)?
            .ok_or(io::Error::other("target field"))?;
        let DxfEntityCommonReferenceTargetSemantics::Reviewed(value) = entry.semantics() else {
            return Err(io::Error::other("reviewed target").into());
        };
        assert_eq!(value.state(), DxfSemanticValueState::Explicit);
        assert!(matches!(
            value.value(),
            Some(DxfEntityCommonReferenceValue::Resolved { target, .. })
                if target.handle() == handle
        ));
    }
    Ok(())
}

fn reviewed_cases() -> [(
    DxfEntityField,
    DxfHandle,
    DxfEntityCommonReferenceTargetKind,
); 3] {
    [
        (
            DxfEntityField::EXTENSION_DICTIONARY,
            DxfHandle::from_u64(0x2f),
            DxfEntityCommonReferenceTargetKind::ExtensionDictionary,
        ),
        (
            DxfEntityField::MATERIAL,
            DxfHandle::from_u64(0x3f),
            DxfEntityCommonReferenceTargetKind::Material,
        ),
        (
            DxfEntityField::PLOT_STYLE,
            DxfHandle::from_u64(0x4f),
            DxfEntityCommonReferenceTargetKind::PlotStyle,
        ),
    ]
}

fn valid(
    outcome: DxfEntityCommonReferenceEditOutcome,
) -> Result<seacad_dxf_core::DxfEntityCommonReferenceEditValue, io::Error> {
    match outcome {
        DxfEntityCommonReferenceEditOutcome::Valid(value) => Ok(value),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn invalid(
    outcome: DxfEntityCommonReferenceEditOutcome,
) -> Result<DxfEntityCommonReferenceEditIssue, io::Error> {
    match outcome {
        DxfEntityCommonReferenceEditOutcome::Invalid(issue) => Ok(issue),
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

fn classifier_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    ambiguous: bool,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version);
    groups.extend(object_records(&[
        (b"DICTIONARY", b"1F"),
        (b"DICTIONARY", b"2F"),
        (b"MATERIAL", b"3F"),
        (b"ACDBPLACEHOLDER", b"4F"),
    ]));
    if ambiguous {
        groups.extend(object_records(&[(b"DICTIONARY", b"2F")]));
    }
    groups.extend(simple_entity());
    encode(format, version, &groups)
}

fn session_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version);
    groups.extend(object_records(&[
        (b"DICTIONARY", b"1F"),
        (b"DICTIONARY", b"2F"),
        (b"MATERIAL", b"3F"),
        (b"ACDBPLACEHOLDER", b"4F"),
        (b"DICTIONARY", b"5F"),
        (b"MATERIAL", b"6F"),
        (b"ACDBPLACEHOLDER", b"7F"),
    ]));
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (330, Value::Text(b"1F")),
        (102, Value::Text(b"{ACAD_XDICTIONARY")),
        (360, Value::Text(b"5F")),
        (102, Value::Text(b"}")),
        (100, Value::Text(b"AcDbEntity")),
        (390, Value::Text(b"7F")),
        (347, Value::Text(b"6F")),
        (8, Value::Text(b"0")),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    encode(format, version, &groups)
}

fn wrong_section_fixture(format: DxfRawDocumentFormat) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"MATERIAL")),
        (5, Value::Text(b"3F")),
        (0, Value::Text(b"ENDSEC")),
    ]);
    groups.extend(simple_entity());
    encode(format, DxfAcadVersion::Ac1032, &groups)
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

fn object_records(records: &[(&'static [u8], &'static [u8])]) -> Vec<(i16, Value<'static>)> {
    let mut groups = vec![(0, Value::Text(b"SECTION")), (2, Value::Text(b"OBJECTS"))];
    for (marker, handle) in records {
        groups.extend([(0, Value::Text(marker)), (5, Value::Text(handle))]);
    }
    groups.push((0, Value::Text(b"ENDSEC")));
    groups
}

fn simple_entity() -> [(i16, Value<'static>); 7] {
    [
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (8, Value::Text(b"0")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
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

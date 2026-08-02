use std::{error::Error, io};

use seacad_dxf_core::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfByteSource, DxfCancellationToken, DxfEntityCommonFieldPatch, DxfEntityField,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityPatch, DxfError, DxfHandle,
    DxfHandleAssignmentPlanOutcome, DxfHandleIdentityState, DxfHandseedValue, DxfMemorySource,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResource, DxfResourceProfile,
    DxfSemanticValueState, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_composes_ascii_binary_plans_and_exact_inverse() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, b"AAAA", b"BBBB")?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let [first, second] = payloads(view)?;
            let first_plan = replacement_plan(view, first, b"A", DxfResourceProfile::Safe)?;
            let second_plan = replacement_plan(view, second, b"LONGER", DxfResourceProfile::Safe)?;
            let composed = view.compose_transaction_plans(
                &[&second_plan, &first_plan],
                DxfResourceProfile::Safe,
                &token(),
            )?;
            assert_eq!(composed.patches().len(), 2);
            assert_eq!(composed.patches()[0].source_span(), first);
            assert_eq!(composed.patches()[1].source_span(), second);
            assert_eq!(composed.projected_len(), view.source_len() - 1);

            let output = materialize(&bytes, &composed)?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let post_document = open_document(&output_source, format)?;
            let post = post_document.view();
            let inverse = composed.materialize_inverse_plan(
                view,
                post,
                DxfResourceProfile::Safe,
                &token(),
            )?;
            assert_eq!(materialize(&output, &inverse)?, bytes);
        }
    }
    Ok(())
}

#[test]
fn empty_boundary_and_source_order_compositions_are_deterministic() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        b"AAAA",
        b"BBBB",
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let [first, _] = payloads(view)?;

    let empty = view.compose_transaction_plans(&[], DxfResourceProfile::Safe, &token())?;
    assert!(empty.patches().is_empty());
    assert_eq!(empty.projected_len(), view.source_len());

    let replacement = replacement_plan(view, first, b"EDIT", DxfResourceProfile::Safe)?;
    let insertion = replacement_plan(
        view,
        point(first.start())?,
        b"PREFIX",
        DxfResourceProfile::Safe,
    )?;
    let composed = view.compose_transaction_plans(
        &[&replacement, &insertion],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(composed.patches().len(), 2);
    assert!(composed.patches()[0].source_span().is_empty());
    assert_eq!(composed.patches()[1].source_span(), first);
    assert_eq!(
        composed.replacement_bytes_for_patch_ordinal(0),
        Some(b"PREFIX".as_slice())
    );
    assert_eq!(
        composed.replacement_bytes_for_patch_ordinal(1),
        Some(b"EDIT".as_slice())
    );
    assert_eq!(composed.projected_len(), view.source_len() + 6);
    Ok(())
}

#[test]
fn overlap_duplicate_insertion_source_and_cancellation_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        b"AAAA",
        b"BBBB",
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let [first, _] = payloads(view)?;
    let whole = replacement_plan(view, first, b"X", DxfResourceProfile::Safe)?;
    let overlap = replacement_plan(
        view,
        ByteSpan::new(first.start() + 1, first.end()).ok_or_else(invalid_test_data)?,
        b"Y",
        DxfResourceProfile::Safe,
    )?;
    assert!(matches!(
        view.compose_transaction_plans(&[&whole, &overlap], DxfResourceProfile::Safe, &token()),
        Err(DxfError::TransactionPatchConflict { .. })
    ));

    let insertion = replacement_plan(
        view,
        point(first.start())?,
        b"ONE",
        DxfResourceProfile::Safe,
    )?;
    assert!(matches!(
        view.compose_transaction_plans(
            &[&insertion, &insertion],
            DxfResourceProfile::Safe,
            &token()
        ),
        Err(DxfError::TransactionPatchConflict { .. })
    ));

    let other_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1027,
        b"AAAA",
        b"BBBB",
    )?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        DxfRawDocumentView::from(&other).compose_transaction_plans(
            &[&whole],
            DxfResourceProfile::Safe,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        view.compose_transaction_plans(&[&whole], DxfResourceProfile::Safe, &cancelled),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

#[test]
fn composition_reapplies_resource_limits_and_redacts_payloads() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        b"AAAA",
        b"BBBB",
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Large)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let [first, _] = payloads(view)?;
    let oversized = vec![b'S'; 1024 * 1024 + 1];
    let plan = replacement_plan(view, first, &oversized, DxfResourceProfile::Large)?;
    assert!(matches!(
        view.compose_transaction_plans(&[&plan], DxfResourceProfile::Safe, &token()),
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::ValueBytes,
            ..
        })
    ));
    let composed = view.compose_transaction_plans(&[&plan], DxfResourceProfile::Large, &token())?;
    assert!(!format!("{composed:?}").contains("SSSS"));

    let empty = view.compose_transaction_plans(&[], DxfResourceProfile::Safe, &token())?;
    let excessive_count = usize::try_from(DxfResourceProfile::Safe.limits().max_records() + 1)
        .map_err(|_| invalid_test_data())?;
    let excessive = vec![&empty; excessive_count];
    assert!(matches!(
        view.compose_transaction_plans(&excessive, DxfResourceProfile::Safe, &token()),
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::Records,
            ..
        })
    ));
    Ok(())
}

#[test]
fn handle_assignment_and_entity_edit_compose_across_every_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = crud_fixture(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let entity = only_entity(view)?;

            let policy = view.handle_allocation_policy_directory(&token())?;
            let DxfHandleAssignmentPlanOutcome::Planned(assignment) = view
                .plan_handle_assignments(
                    &policy,
                    &[entity.record().ordinal()],
                    DxfResourceProfile::Safe,
                    &token(),
                )?
            else {
                return Err(io::Error::other("handle assignment").into());
            };

            let evidence = view.entity_field_evidence_directory(&token())?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            session.update(
                entity.key(),
                DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::ResetToDefault {
                    field: DxfEntityField::COLOR,
                }),
            )?;
            let field_edit = session.finish_verifiable()?;
            let composed = field_edit.compose_supplemental_transactions(
                view,
                &[assignment.transaction()],
                DxfResourceProfile::Safe,
                &token(),
            )?;
            assert_eq!(composed.transaction().patches().len(), 3);

            let output = materialize(&bytes, composed.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let post_document = open_document(&output_source, format)?;
            let post = post_document.view();
            let post_entity = only_entity(post)?;
            let identities = post.handle_identity_directory(&token())?;
            assert_eq!(
                identities
                    .entry(post_entity.record().ordinal())
                    .map(|entry| entry.state()),
                Some(DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(
                    0x10
                )))
            );
            assert!(matches!(
                post.handseed_report()
                    .primary_occurrence()
                    .map(|entry| entry.value()),
                Some(DxfHandseedValue::Parsed(handle))
                    if handle == DxfHandle::from_u64(0x11)
            ));
            assert_default_color(post, post_entity)?;

            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                composed.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("composed semantic verification").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

fn assert_default_color(
    view: DxfRawDocumentView<'_>,
    entity: seacad_dxf_core::DxfEntityRef,
) -> Result<(), Box<dyn Error>> {
    let semantics = view.entity_field_semantic_directory(&token())?;
    let entry = semantics
        .entry_for_field(entity, DxfEntityField::COLOR)?
        .ok_or(io::Error::other("color entry"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("color singleton").into());
    };
    assert_eq!(value.state(), DxfSemanticValueState::Defaulted);
    assert_eq!(value.value(), Some(&DxfEntityFieldValue::Int16(256)));
    Ok(())
}

fn only_entity(view: DxfRawDocumentView<'_>) -> Result<seacad_dxf_core::DxfEntityRef, DxfError> {
    let directory = view.entity_directory(&token())?;
    let [entity] = directory.entities() else {
        return Err(invalid_test_data());
    };
    Ok(*entity)
}

fn replacement_plan(
    view: DxfRawDocumentView<'_>,
    span: ByteSpan,
    replacement: &[u8],
    profile: DxfResourceProfile,
) -> Result<DxfTransactionPlan, DxfError> {
    let mut builder = view.transaction_plan_builder(profile)?;
    builder.replace_raw_span(span, replacement, &token())?;
    builder.finish(&token())
}

fn payloads(view: DxfRawDocumentView<'_>) -> Result<[ByteSpan; 2], DxfError> {
    let first = view
        .group(4)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let second = view
        .group(5)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    Ok([first, second])
}

fn point(offset: u64) -> Result<ByteSpan, DxfError> {
    ByteSpan::new(offset, offset).ok_or_else(invalid_test_data)
}

fn materialize(source: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, DxfError> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start =
            usize::try_from(patch.source_span().start()).map_err(|_| invalid_test_data())?;
        let end = usize::try_from(patch.source_span().end()).map_err(|_| invalid_test_data())?;
        output.extend_from_slice(source.get(cursor..start).ok_or_else(invalid_test_data)?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or_else(invalid_test_data)?,
        );
        cursor = end;
    }
    output.extend_from_slice(source.get(cursor..).ok_or_else(invalid_test_data)?);
    Ok(output)
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    first: &[u8],
    second: &[u8],
) -> Result<Vec<u8>, io::Error> {
    let groups = [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (1, first),
        (1, second),
        (0, b"ENDSEC"),
        (0, b"EOF"),
    ];
    match format {
        DxfRawDocumentFormat::Ascii => {
            let mut bytes = Vec::new();
            for (code, value) in groups {
                bytes.extend_from_slice(code.to_string().as_bytes());
                bytes.push(b'\n');
                bytes.extend_from_slice(value);
                bytes.push(b'\n');
            }
            Ok(bytes)
        }
        DxfRawDocumentFormat::Binary => {
            let mut bytes = DXF_BINARY_SENTINEL.to_vec();
            for (code, value) in groups {
                if version == DxfAcadVersion::Ac1009 {
                    bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
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

#[derive(Clone, Copy)]
enum CrudValue<'a> {
    Text(&'a [u8]),
    Int16(i16),
}

fn crud_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0_i16, CrudValue::Text(b"SECTION")),
        (2, CrudValue::Text(b"HEADER")),
        (9, CrudValue::Text(b"$ACADVER")),
        (1, CrudValue::Text(version.code().as_bytes())),
        (9, CrudValue::Text(b"$HANDSEED")),
        (5, CrudValue::Text(b"10")),
        (0, CrudValue::Text(b"ENDSEC")),
        (0, CrudValue::Text(b"SECTION")),
        (2, CrudValue::Text(b"ENTITIES")),
        (0, CrudValue::Text(b"LINE")),
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, CrudValue::Text(b"AcDbEntity")));
    }
    groups.push((62, CrudValue::Int16(7)));
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, CrudValue::Text(b"AcDbLine")));
    }
    groups.extend([
        (0, CrudValue::Text(b"ENDSEC")),
        (0, CrudValue::Text(b"EOF")),
    ]);

    let mut bytes = if format == DxfRawDocumentFormat::Binary {
        DXF_BINARY_SENTINEL.to_vec()
    } else {
        Vec::new()
    };
    for (code, value) in groups {
        match format {
            DxfRawDocumentFormat::Ascii => {
                bytes.extend_from_slice(code.to_string().as_bytes());
                bytes.push(b'\n');
                match value {
                    CrudValue::Text(value) => bytes.extend_from_slice(value),
                    CrudValue::Int16(value) => {
                        bytes.extend_from_slice(value.to_string().as_bytes());
                    }
                }
                bytes.push(b'\n');
            }
            DxfRawDocumentFormat::Binary => {
                if version == DxfAcadVersion::Ac1009 {
                    bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
                } else {
                    bytes.extend_from_slice(&code.to_le_bytes());
                }
                match value {
                    CrudValue::Text(value) => {
                        bytes.extend_from_slice(value);
                        bytes.push(0);
                    }
                    CrudValue::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                }
            }
            _ => return Err(io::Error::other("format")),
        }
    }
    Ok(bytes)
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityCommonFieldPatch, DxfEntityEditPlan,
    DxfEntityEditValue, DxfEntityEditVerificationIssue, DxfEntityEditVerificationJournal,
    DxfEntityEditVerificationOutcome, DxfEntityField, DxfEntityPatch, DxfError, DxfHandle,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_verifies_ascii_binary_semantics_and_exact_inverse() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, false)?;
            verify_round_trip(&bytes, format)?;
        }
    }
    Ok(())
}

#[test]
fn every_singleton_value_domain_has_an_exact_semantic_postcondition() -> Result<(), Box<dyn Error>>
{
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, true)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        match format {
            DxfRawDocumentFormat::Ascii => {
                let document = open_ascii(&source)?;
                verify_value_domains(&bytes, DxfRawDocumentView::from(&document))?;
            }
            DxfRawDocumentFormat::Binary => {
                let document = open_binary(&source)?;
                verify_value_domains(&bytes, DxfRawDocumentView::from(&document))?;
            }
            _ => return Err(io::Error::other("format").into()),
        }
    }
    Ok(())
}

#[test]
fn semantic_mismatch_is_typed_before_unrelated_raw_mismatch() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, false)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let plan = standard_plan(view, b"NEW")?;
    let correct = materialize(&bytes, plan.transaction())?;

    let mut wrong_semantic = correct.clone();
    replace_once(&mut wrong_semantic, b"NEW", b"BAD")?;
    let wrong_source = DxfMemorySource::new(&wrong_semantic, DxfResourceProfile::Safe)?;
    let wrong = open_ascii(&wrong_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&wrong),
            DxfResourceProfile::Safe,
            &token()
        )?,
        DxfEntityEditVerificationOutcome::Unavailable(
            DxfEntityEditVerificationIssue::ValueMismatch { field, .. }
        ) if field == DxfEntityField::LAYER
    ));

    let mut wrong_raw = correct;
    replace_once(&mut wrong_raw, b"10\r\n0\r\n", b"10\r\n1\r\n")?;
    let wrong_source = DxfMemorySource::new(&wrong_raw, DxfResourceProfile::Safe)?;
    let wrong = open_ascii(&wrong_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&wrong),
            DxfResourceProfile::Safe,
            &token()
        ),
        Err(DxfError::TransactionPostImageMismatch { .. })
    ));
    Ok(())
}

#[test]
fn source_envelope_cancellation_traits_and_debug_redaction_fail_closed()
-> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityEditVerificationIssue>();
    assert_send_sync::<DxfEntityEditPlan>();
    assert_send_sync::<DxfEntityEditVerificationJournal>();

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, false)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let plan = standard_plan(view, b"SECRET")?;
    assert_eq!(plan.source_id(), view.source_id());
    assert_eq!(plan.edit_count(), 3);
    let debug = format!("{plan:?}");
    assert!(debug.contains("edit_count"));
    assert!(!debug.contains("SECRET"));
    let correct = materialize(&bytes, plan.transaction())?;
    let post_source = DxfMemorySource::new(&correct, DxfResourceProfile::Safe)?;
    let post = open_ascii(&post_source)?;

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027, false)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        plan.verify_post_image(
            DxfRawDocumentView::from(&other),
            DxfRawDocumentView::from(&post),
            DxfResourceProfile::Safe,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&post),
            DxfResourceProfile::Safe,
            &cancellation
        ),
        Err(DxfError::Cancelled)
    ));

    let mut longer = correct;
    let offset = find_once(&longer, b"SECRET")? + b"SECRET".len();
    longer.insert(offset, b'X');
    let longer_source = DxfMemorySource::new(&longer, DxfResourceProfile::Safe)?;
    let longer_post = open_ascii(&longer_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&longer_post),
            DxfResourceProfile::Safe,
            &token()
        ),
        Err(DxfError::TransactionPostImageLengthMismatch { .. })
    ));

    let disposable = standard_plan(view, b"DISCARD")?;
    assert_eq!(disposable.into_transaction().source_id(), view.source_id());
    Ok(())
}

fn verify_round_trip(bytes: &[u8], format: DxfRawDocumentFormat) -> Result<(), Box<dyn Error>> {
    let original = bytes.to_vec();
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    match format {
        DxfRawDocumentFormat::Ascii => {
            let document = open_ascii(&source)?;
            let view = DxfRawDocumentView::from(&document);
            let plan = standard_plan(view, b"NEW")?;
            let output = materialize(bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let post = open_ascii(&output_source)?;
            assert_verified(bytes, &output, view, DxfRawDocumentView::from(&post), &plan)?;
        }
        DxfRawDocumentFormat::Binary => {
            let document = open_binary(&source)?;
            let view = DxfRawDocumentView::from(&document);
            let plan = standard_plan(view, b"NEW")?;
            let output = materialize(bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let post = open_binary(&output_source)?;
            assert_verified(bytes, &output, view, DxfRawDocumentView::from(&post), &plan)?;
        }
        _ => return Err(io::Error::other("format").into()),
    }
    assert_eq!(bytes, original);
    Ok(())
}

fn assert_verified(
    source: &[u8],
    output: &[u8],
    source_view: DxfRawDocumentView<'_>,
    post_view: DxfRawDocumentView<'_>,
    plan: &DxfEntityEditPlan,
) -> Result<(), Box<dyn Error>> {
    let journal = verified(plan.verify_post_image(
        source_view,
        post_view,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    assert_eq!(journal.receipt().source_id(), source_view.source_id());
    assert_eq!(journal.receipt().post_image_id(), post_view.source_id());
    assert_eq!(journal.receipt().edit_count(), 3);
    assert_eq!(journal.inverse_plan().source_id(), post_view.source_id());
    let (receipt, inverse) = journal.into_parts();
    assert_eq!(receipt.edit_count(), 3);
    assert_eq!(materialize(output, &inverse)?, source);
    Ok(())
}

fn verify_value_domains(bytes: &[u8], view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = evidence.entity_directory().entities()[0].key();
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    for (field, value) in [
        (
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"DOMAIN"),
        ),
        (DxfEntityField::PAPER_SPACE, DxfEntityEditValue::Int16(1)),
        (
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(2.5)),
        ),
        (
            DxfEntityField::PROXY_GRAPHICS_SIZE,
            DxfEntityEditValue::Int32(9),
        ),
        (
            DxfEntityField::MATERIAL,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(0x2a)),
        ),
    ] {
        applied(session.update(key, set(field, value))?)?;
    }
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 5);
    let output = materialize(bytes, plan.transaction())?;
    let source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    match view.format() {
        DxfRawDocumentFormat::Ascii => {
            let post = open_ascii(&source)?;
            verified(plan.verify_post_image(
                view,
                DxfRawDocumentView::from(&post),
                DxfResourceProfile::Safe,
                &token(),
            )?)?;
        }
        DxfRawDocumentFormat::Binary => {
            let post = open_binary(&source)?;
            verified(plan.verify_post_image(
                view,
                DxfRawDocumentView::from(&post),
                DxfResourceProfile::Safe,
                &token(),
            )?)?;
        }
        _ => return Err(io::Error::other("format").into()),
    }
    Ok(())
}

fn standard_plan(
    view: DxfRawDocumentView<'_>,
    layer: &[u8],
) -> Result<DxfEntityEditPlan, Box<dyn Error>> {
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = evidence.entity_directory().entities()[0].key();
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    applied(session.update(
        key,
        set(
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(layer),
        ),
    )?)?;
    applied(session.update(key, reset(DxfEntityField::COLOR))?)?;
    applied(session.update(
        key,
        set(
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"DASHED"),
        ),
    )?)?;
    Ok(session.finish_verifiable()?)
}

fn set(field: DxfEntityField, value: DxfEntityEditValue<'_>) -> DxfEntityPatch<'_> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit { field, value })
}

const fn reset(field: DxfEntityField) -> DxfEntityPatch<'static> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::ResetToDefault { field })
}

fn applied(outcome: seacad_dxf_core::DxfEntityEditOutcome) -> Result<(), io::Error> {
    match outcome {
        seacad_dxf_core::DxfEntityEditOutcome::Applied(_) => Ok(()),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn verified(
    outcome: DxfEntityEditVerificationOutcome,
) -> Result<DxfEntityEditVerificationJournal, io::Error> {
    match outcome {
        DxfEntityEditVerificationOutcome::Verified(journal) => Ok(journal),
        other => Err(io::Error::other(format!(
            "unexpected verification outcome: {other:?}"
        ))),
    }
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

fn replace_once(bytes: &mut [u8], needle: &[u8], replacement: &[u8]) -> Result<(), io::Error> {
    if needle.len() != replacement.len() {
        return Err(io::Error::other("replacement length"));
    }
    let start = find_once(bytes, needle)?;
    bytes
        .get_mut(start..start + needle.len())
        .ok_or(io::Error::other("replacement"))?
        .copy_from_slice(replacement);
    Ok(())
}

fn find_once(bytes: &[u8], needle: &[u8]) -> Result<usize, io::Error> {
    let mut matches = bytes
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(index, _)| index);
    let first = matches.next().ok_or(io::Error::other("needle"))?;
    if matches.next().is_some() {
        return Err(io::Error::other("needle duplicate"));
    }
    Ok(first)
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Int32(i32),
    Double(f64),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    extended: bool,
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
        groups.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    groups.extend([
        (8, Value::Text(b"OLD")),
        (67, Value::Int16(0)),
        (62, Value::Int16(7)),
        (48, Value::Double(1.0)),
        (92, Value::Int32(0)),
    ]);
    if extended {
        groups.push((347, Value::Text(b"20")));
    }
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([
        (10, Value::Double(0.0)),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.extend_from_slice(b"\r\n");
    }
    bytes
}

fn binary_groups(
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
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
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }
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

fn assert_copy_send_sync<T: Copy + Send + Sync>() {}
fn assert_send_sync<T: Send + Sync>() {}

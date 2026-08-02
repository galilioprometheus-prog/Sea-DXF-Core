use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonFieldPatch, DxfEntityCommonSymbolEditIssue,
    DxfEntityEditDisposition, DxfEntityEditIssue, DxfEntityEditOutcome, DxfEntityEditReceipt,
    DxfEntityEditSession, DxfEntityEditValue, DxfEntityEditValueKind, DxfEntityField,
    DxfEntityFieldEvidenceDirectory, DxfEntityFieldReplacementIssue, DxfEntityFieldResetIssue,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityKey, DxfEntityPatch, DxfError,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfRawRecordSectionKind,
    DxfReadOptions, DxfResource, DxfResourceProfile, DxfSemanticValueState, DxfTransactionPlan,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_batches_ascii_binary_updates_and_restores_exact_inverse()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, false)?;
            verify_batch(&bytes, format)?;
        }
    }
    Ok(())
}

#[test]
fn same_anchor_insertions_follow_writer_order_and_publish_receipts() -> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityEditIssue>();
    assert_copy_send_sync::<DxfEntityEditOutcome>();
    assert_send_sync::<DxfEntityEditSession<'_, '_, '_>>();

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, false)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let keys = keys(&evidence)?;
    let cancellation = token();
    let mut session =
        document.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert_eq!(session.source_id(), keys[0].source_id());

    let implicit = applied(session.update(keys[1], reset(DxfEntityField::VISIBILITY))?)?;
    assert_receipt(
        implicit,
        keys[1],
        DxfEntityField::VISIBILITY,
        DxfEntityEditDisposition::AlreadyImplicit,
        0,
    );
    let color = applied(session.update(
        keys[1],
        set(DxfEntityField::COLOR, DxfEntityEditValue::Int16(3)),
    )?)?;
    assert_receipt(
        color,
        keys[1],
        DxfEntityField::COLOR,
        DxfEntityEditDisposition::Inserted,
        1,
    );
    let linetype = applied(session.update(
        keys[1],
        set(
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"DASHED"),
        ),
    )?)?;
    assert_receipt(
        linetype,
        keys[1],
        DxfEntityField::LINETYPE,
        DxfEntityEditDisposition::Inserted,
        2,
    );
    let layer = applied(session.update(
        keys[1],
        set(
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"L2"),
        ),
    )?)?;
    assert_receipt(
        layer,
        keys[1],
        DxfEntityField::LAYER,
        DxfEntityEditDisposition::Inserted,
        3,
    );
    assert_eq!(session.queued_edit_count(), 3);
    let debug = format!("{session:?}");
    assert!(debug.contains("queued_edit_count"));
    assert!(!debug.contains("DASHED"));

    let plan = session.finish()?;
    assert_eq!(plan.patches().len(), 1);
    assert!(plan.patches()[0].source_span().is_empty());
    assert_eq!(
        plan.replacement_bytes_for_patch_ordinal(0),
        Some(b"8\r\nL2\r\n6\r\nDASHED\r\n62\r\n3\r\n".as_slice())
    );
    Ok(())
}

#[test]
fn duplicate_structural_and_encoding_failures_never_enter_the_batch() -> Result<(), Box<dyn Error>>
{
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, true)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let keys = keys(&evidence)?;
    let cancellation = token();
    let mut session =
        document.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;

    assert_issue(
        session.update(
            keys[0],
            set(
                DxfEntityField::LINETYPE,
                DxfEntityEditValue::ExactRawText(b"NEW"),
            ),
        )?,
        DxfEntityEditIssue::Replacement(DxfEntityFieldReplacementIssue::DuplicateSingleton {
            occurrence_count: 2,
        }),
    )?;
    assert_issue(
        session.update(
            keys[0],
            set(
                DxfEntityField::PROXY_GRAPHICS_DATA,
                DxfEntityEditValue::BinaryChunk(&[9]),
            ),
        )?,
        DxfEntityEditIssue::Replacement(
            DxfEntityFieldReplacementIssue::SequenceOperationRequired {
                occurrence_count: 2,
            },
        ),
    )?;
    assert_issue(
        session.update(keys[0], reset(DxfEntityField::HANDLE))?,
        DxfEntityEditIssue::Reset(DxfEntityFieldResetIssue::RequiredField),
    )?;
    assert_issue(
        session.update(
            keys[1],
            set(DxfEntityField::LINETYPE, DxfEntityEditValue::Int16(1)),
        )?,
        DxfEntityEditIssue::Symbol(DxfEntityCommonSymbolEditIssue::ValueKindMismatch {
            field: DxfEntityField::LINETYPE,
            expected: DxfEntityEditValueKind::ExactRawText,
            observed: DxfEntityEditValueKind::Int16,
        }),
    )?;
    assert_eq!(session.queued_edit_count(), 0);

    applied(session.update(
        keys[0],
        set(
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"QUEUED"),
        ),
    )?)?;
    assert_issue(
        session.update(keys[0], reset(DxfEntityField::LAYER))?,
        DxfEntityEditIssue::DuplicateFieldEdit {
            key: keys[0],
            field: DxfEntityField::LAYER,
        },
    )?;
    assert_eq!(session.queued_edit_count(), 1);
    assert_eq!(session.finish()?.patches().len(), 1);
    Ok(())
}

#[test]
fn source_limits_cancellation_and_debug_redaction_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, false)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let keys = keys(&evidence)?;

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027, false)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        other.entity_edit_session(&evidence, DxfResourceProfile::Safe, &token()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled),
        Err(DxfError::Cancelled)
    ));

    let cancellation = token();
    let mut session =
        document.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    let oversized = vec![b'X'; 1024 * 1024 + 1];
    assert!(matches!(
        session.update(
            keys[0],
            set(
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(&oversized)
            )
        ),
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::ValueBytes,
            ..
        })
    ));
    assert_eq!(session.queued_edit_count(), 0);
    applied(session.update(
        keys[0],
        set(
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"SECRET"),
        ),
    )?)?;
    let debug = format!("{session:?}");
    assert!(!debug.contains("SECRET"));
    cancellation.cancel();
    assert!(matches!(
        session.update(keys[0], reset(DxfEntityField::COLOR)),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(session.finish(), Err(DxfError::Cancelled)));
    Ok(())
}

fn verify_batch(bytes: &[u8], format: DxfRawDocumentFormat) -> Result<(), Box<dyn Error>> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    match format {
        DxfRawDocumentFormat::Ascii => {
            let document = open_ascii(&source)?;
            verify_view(bytes, DxfRawDocumentView::from(&document))?;
        }
        DxfRawDocumentFormat::Binary => {
            let document = open_binary(&source)?;
            verify_view(bytes, DxfRawDocumentView::from(&document))?;
        }
        _ => return Err(io::Error::other("format").into()),
    }
    Ok(())
}

fn verify_view(bytes: &[u8], view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let original = bytes.to_vec();
    let evidence = view.entity_field_evidence_directory(&token())?;
    let keys = keys(&evidence)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert_eq!(
        applied(session.update(
            keys[0],
            set(
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(b"NEW")
            )
        )?)?
        .disposition(),
        DxfEntityEditDisposition::Replaced
    );
    assert_eq!(
        applied(session.update(keys[0], reset(DxfEntityField::COLOR))?)?.disposition(),
        DxfEntityEditDisposition::Reset
    );
    applied(session.update(
        keys[1],
        set(DxfEntityField::COLOR, DxfEntityEditValue::Int16(3)),
    )?)?;
    applied(session.update(
        keys[1],
        set(
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"DASHED"),
        ),
    )?)?;
    applied(session.update(
        keys[1],
        set(
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"L2"),
        ),
    )?)?;
    let plan = session.finish()?;
    assert_eq!(plan.patches().len(), 3);
    let output = materialize(bytes, &plan)?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    match view.format() {
        DxfRawDocumentFormat::Ascii => {
            let post = open_ascii(&output_source)?;
            verify_semantics(DxfRawDocumentView::from(&post))?;
            verify_inverse(bytes, &output, view, DxfRawDocumentView::from(&post), &plan)?;
        }
        DxfRawDocumentFormat::Binary => {
            let post = open_binary(&output_source)?;
            verify_semantics(DxfRawDocumentView::from(&post))?;
            verify_inverse(bytes, &output, view, DxfRawDocumentView::from(&post), &plan)?;
        }
        _ => return Err(io::Error::other("format").into()),
    }
    assert_eq!(bytes, original);
    Ok(())
}

fn verify_semantics(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    assert_text(view, 0, DxfEntityField::LAYER, b"NEW")?;
    assert_default(
        view,
        0,
        DxfEntityField::COLOR,
        DxfEntityFieldValue::Int16(256),
    )?;
    assert_text(view, 1, DxfEntityField::LAYER, b"L2")?;
    assert_text(view, 1, DxfEntityField::LINETYPE, b"DASHED")?;
    assert_explicit(
        view,
        1,
        DxfEntityField::COLOR,
        DxfEntityFieldValue::Int16(3),
    )?;
    Ok(())
}

fn assert_text(
    view: DxfRawDocumentView<'_>,
    entity_index: usize,
    field: DxfEntityField,
    expected: &[u8],
) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_field_semantic_directory(&token())?;
    let entity = directory
        .evidence_directory()
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .filter(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities)
        .nth(entity_index)
        .ok_or(io::Error::other("entity"))?;
    let entry = directory
        .entry_for_field(entity, field)?
        .ok_or(io::Error::other("field"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("singleton").into());
    };
    let Some(DxfEntityFieldValue::ExactText(text)) = value.value().copied() else {
        return Err(io::Error::other("text").into());
    };
    let mut observed = vec![0_u8; expected.len()];
    view.read_span(text.value_span(), &mut observed)?;
    assert_eq!(observed, expected);
    Ok(())
}

fn assert_default(
    view: DxfRawDocumentView<'_>,
    entity_index: usize,
    field: DxfEntityField,
    expected: DxfEntityFieldValue,
) -> Result<(), Box<dyn Error>> {
    let value = singleton(view, entity_index, field)?;
    assert_eq!(value.state(), DxfSemanticValueState::Defaulted);
    assert_eq!(value.value(), Some(&expected));
    Ok(())
}

fn assert_explicit(
    view: DxfRawDocumentView<'_>,
    entity_index: usize,
    field: DxfEntityField,
    expected: DxfEntityFieldValue,
) -> Result<(), Box<dyn Error>> {
    let value = singleton(view, entity_index, field)?;
    assert_eq!(value.state(), DxfSemanticValueState::Explicit);
    assert_eq!(value.value(), Some(&expected));
    Ok(())
}

fn singleton(
    view: DxfRawDocumentView<'_>,
    entity_index: usize,
    field: DxfEntityField,
) -> Result<seacad_dxf_core::DxfEntityFieldSemanticValue, Box<dyn Error>> {
    let directory = view.entity_field_semantic_directory(&token())?;
    let entity = directory
        .evidence_directory()
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .filter(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities)
        .nth(entity_index)
        .ok_or(io::Error::other("entity"))?;
    let entry = directory
        .entry_for_field(entity, field)?
        .ok_or(io::Error::other("field"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("singleton").into());
    };
    Ok(value)
}

fn verify_inverse(
    original: &[u8],
    output: &[u8],
    source_view: DxfRawDocumentView<'_>,
    post_view: DxfRawDocumentView<'_>,
    plan: &DxfTransactionPlan,
) -> Result<(), Box<dyn Error>> {
    let inverse =
        plan.materialize_inverse_plan(source_view, post_view, DxfResourceProfile::Safe, &token())?;
    assert_eq!(materialize(output, &inverse)?, original);
    Ok(())
}

fn set(field: DxfEntityField, value: DxfEntityEditValue<'_>) -> DxfEntityPatch<'_> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit { field, value })
}

const fn reset(field: DxfEntityField) -> DxfEntityPatch<'static> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::ResetToDefault { field })
}

fn applied(outcome: DxfEntityEditOutcome) -> Result<DxfEntityEditReceipt, io::Error> {
    match outcome {
        DxfEntityEditOutcome::Applied(receipt) => Ok(receipt),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn assert_issue(
    outcome: DxfEntityEditOutcome,
    expected: DxfEntityEditIssue,
) -> Result<(), io::Error> {
    match outcome {
        DxfEntityEditOutcome::Unavailable(issue) if issue == expected => Ok(()),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn assert_receipt(
    receipt: DxfEntityEditReceipt,
    key: DxfEntityKey,
    field: DxfEntityField,
    disposition: DxfEntityEditDisposition,
    count: u64,
) {
    assert_eq!(receipt.key(), key);
    assert_eq!(receipt.field(), field);
    assert_eq!(receipt.disposition(), disposition);
    assert_eq!(receipt.queued_edit_count(), count);
}

fn keys(evidence: &DxfEntityFieldEvidenceDirectory) -> Result<[DxfEntityKey; 2], io::Error> {
    let mut entities = evidence
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .filter(|entity| entity.record().section_kind() == DxfRawRecordSectionKind::Entities);
    let first = entities.next().ok_or(io::Error::other("first"))?;
    let second = entities.next().ok_or(io::Error::other("second"))?;
    Ok([first.key(), second.key()])
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

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Double(f64),
    Binary(&'a [u8]),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    conflicts: bool,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
    ];
    groups.extend(symbol_table(
        b"LAYER",
        &[b"OLD", b"NEW", b"L2", b"QUEUED", b"SECRET"],
    ));
    groups.extend(symbol_table(b"LTYPE", &[b"DASHED", b"NEW"]));
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    groups.extend([(8, Value::Text(b"OLD")), (62, Value::Int16(7))]);
    if conflicts {
        groups.extend([
            (6, Value::Text(b"DASHED")),
            (6, Value::Text(b"SECOND")),
            (310, Value::Binary(&[1])),
            (310, Value::Binary(&[2])),
        ]);
    }
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([
        (10, Value::Double(0.0)),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"20")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (330, Value::Text(b"1F")),
            (100, Value::Text(b"AcDbEntity")),
            (100, Value::Text(b"AcDbLine")),
        ]);
    }
    groups.extend([
        (10, Value::Double(1.0)),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn symbol_table(kind: &'static [u8], names: &[&'static [u8]]) -> Vec<(i16, Value<'static>)> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(kind)),
    ];
    for (index, name) in names.iter().enumerate() {
        let handle = match index {
            0 => b"A".as_slice(),
            1 => b"B".as_slice(),
            2 => b"C".as_slice(),
            3 => b"D".as_slice(),
            _ => b"E".as_slice(),
        };
        groups.extend([
            (0, Value::Text(kind)),
            (2, Value::Text(name)),
            (5, Value::Text(handle)),
        ]);
    }
    groups.extend([(0, Value::Text(b"ENDTAB")), (0, Value::Text(b"ENDSEC"))]);
    groups
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Binary(value) => {
                for byte in *value {
                    bytes.extend_from_slice(format!("{byte:02X}").as_bytes());
                }
            }
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
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Binary(value) => {
                bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk"))?);
                bytes.extend_from_slice(value);
            }
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

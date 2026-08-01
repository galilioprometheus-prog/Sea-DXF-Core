use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityEditValue, DxfEntityField,
    DxfEntityFieldEvidenceDirectory, DxfEntityFieldInsertionAnchorIssue,
    DxfEntityFieldInsertionIssue, DxfEntityFieldInsertionOutcome, DxfEntityFieldInsertionPlan,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityGroupEncodeIssue, DxfEntityKey,
    DxfError, DxfHandle, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_inserts_strict_ascii_binary_and_materializes_exact_inverse()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = fixture(
            DxfRawDocumentFormat::Ascii,
            version,
            FixtureShape::Absent,
            b"\r\n",
        )?;
        verify_linetype_insertion(&ascii, DxfRawDocumentFormat::Ascii)?;
        let binary = fixture(
            DxfRawDocumentFormat::Binary,
            version,
            FixtureShape::Absent,
            b"\n",
        )?;
        verify_linetype_insertion(&binary, DxfRawDocumentFormat::Binary)?;
    }
    Ok(())
}

#[test]
fn insertion_preserves_ascii_endings_and_encodes_singleton_wire_domains()
-> Result<(), Box<dyn Error>> {
    for ending in [b"\n".as_slice(), b"\r", b"\r\n"] {
        let bytes = fixture(
            DxfRawDocumentFormat::Ascii,
            DxfAcadVersion::Ac1032,
            FixtureShape::Absent,
            ending,
        )?;
        let plan = plan_ascii_field(
            &bytes,
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"DASHED"),
        )?;
        let mut expected = b"6".to_vec();
        expected.extend_from_slice(ending);
        expected.extend_from_slice(b"DASHED");
        expected.extend_from_slice(ending);
        assert_eq!(
            plan.transaction().replacement_bytes_for_patch_ordinal(0),
            Some(expected.as_slice())
        );
        let output = materialize(&bytes, plan.transaction())?;
        let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
        open_ascii(&output_source)?;
    }

    let cases = [
        (
            DxfEntityField::HANDLE,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(0xab)),
            b"5\r\nAB\r\n".as_slice(),
        ),
        (
            DxfEntityField::OWNER,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(0x1f)),
            b"330\r\n1F\r\n",
        ),
        (
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"NEW"),
            b"8\r\nNEW\r\n",
        ),
        (
            DxfEntityField::MATERIAL,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(0xcd)),
            b"347\r\nCD\r\n",
        ),
        (
            DxfEntityField::COLOR,
            DxfEntityEditValue::Int16(-7),
            b"62\r\n-7\r\n",
        ),
        (
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(2.5)),
            b"48\r\n2.5\r\n",
        ),
        (
            DxfEntityField::PROXY_GRAPHICS_SIZE,
            DxfEntityEditValue::Int32(9),
            b"92\r\n9\r\n",
        ),
        (
            DxfEntityField::TRUE_COLOR,
            DxfEntityEditValue::Int32(0x12_34_56),
            b"420\r\n1193046\r\n",
        ),
    ];
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::AllSingletonsAbsent,
        b"\r\n",
    )?;
    for (field, value, expected) in cases {
        let plan = plan_ascii_field(&bytes, field, value)?;
        assert_eq!(plan.field(), field);
        assert_eq!(plan.key(), plan.anchor().key());
        assert_eq!(plan.source_id(), plan.anchor().source_id());
        assert_eq!(plan.transaction().patches().len(), 1);
        assert!(plan.transaction().patches()[0].source_span().is_empty());
        assert_eq!(
            plan.transaction().replacement_bytes_for_patch_ordinal(0),
            Some(expected)
        );
        let output = materialize(&bytes, plan.transaction())?;
        let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
        open_ascii(&output_source)?;
    }
    Ok(())
}

#[test]
fn anchor_and_encoding_failures_remain_typed_without_a_transaction() -> Result<(), Box<dyn Error>> {
    let conflicts = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::Conflicts,
        b"\n",
    )?;
    let source = DxfMemorySource::new(&conflicts, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_key(&evidence)?;
    assert_issue(
        plan_ascii(
            &document,
            &evidence,
            key,
            DxfEntityField::LAYER,
            DxfEntityEditValue::Int32(1),
        )?,
        DxfEntityFieldInsertionIssue::Anchor(
            DxfEntityFieldInsertionAnchorIssue::FieldAlreadyPresent {
                occurrence_count: 2,
            },
        ),
    )?;
    assert_issue(
        plan_ascii(
            &document,
            &evidence,
            key,
            DxfEntityField::PROXY_GRAPHICS_DATA,
            DxfEntityEditValue::BinaryChunk(&[1]),
        )?,
        DxfEntityFieldInsertionIssue::Anchor(
            DxfEntityFieldInsertionAnchorIssue::SequenceOperationRequired {
                occurrence_count: 2,
            },
        ),
    )?;
    assert_issue(
        plan_ascii(
            &document,
            &evidence,
            key,
            DxfEntityField::EXTENSION_DICTIONARY,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(2)),
        )?,
        DxfEntityFieldInsertionIssue::Anchor(
            DxfEntityFieldInsertionAnchorIssue::NestedStructureOperationRequired {
                occurrence_count: 0,
            },
        ),
    )?;
    assert_issue(
        plan_ascii(
            &document,
            &evidence,
            key,
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::Int16(1),
        )?,
        DxfEntityFieldInsertionIssue::Encoding(DxfEntityGroupEncodeIssue::WireTypeMismatch {
            expected: seacad_dxf_core::DxfEntityFieldWireType::ExactText,
            observed: seacad_dxf_core::DxfEntityEditValueKind::Int16,
        }),
    )?;

    let ac1009 = fixture(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        FixtureShape::AllSingletonsAbsent,
        b"\n",
    )?;
    let source = DxfMemorySource::new(&ac1009, DxfResourceProfile::Safe)?;
    let binary = open_binary(&source)?;
    let evidence = binary.entity_field_evidence_directory(&token())?;
    let key = first_key(&evidence)?;
    assert_issue(
        binary.plan_entity_field_insertion(
            &evidence,
            key,
            DxfEntityField::TRUE_COLOR,
            DxfEntityEditValue::Int32(1),
            DxfResourceProfile::Safe,
            &token(),
        )?,
        DxfEntityFieldInsertionIssue::Encoding(
            DxfEntityGroupEncodeIssue::GroupCodeUnavailableInDialect {
                group_code: 420,
                version: DxfAcadVersion::Ac1009,
            },
        ),
    )?;
    Ok(())
}

#[test]
fn source_identity_cancellation_traits_and_debug_redaction_fail_closed()
-> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityFieldInsertionIssue>();
    assert_send_sync::<DxfEntityFieldInsertionPlan>();

    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::Absent,
        b"\n",
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_key(&evidence)?;

    let other_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::AllSingletonsAbsent,
        b"\n",
    )?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        other.plan_entity_field_insertion(
            &evidence,
            key,
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"SECRET"),
            DxfResourceProfile::Safe,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.plan_entity_field_insertion(
            &evidence,
            key,
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"SECRET"),
            DxfResourceProfile::Safe,
            &cancellation
        ),
        Err(DxfError::Cancelled)
    ));
    let plan = planned(plan_ascii(
        &document,
        &evidence,
        key,
        DxfEntityField::LINETYPE,
        DxfEntityEditValue::ExactRawText(b"SECRET"),
    )?)?;
    let debug = format!("{plan:?}");
    assert!(!debug.contains("SECRET"));
    assert!(debug.contains("patch_count"));
    Ok(())
}

fn verify_linetype_insertion(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
) -> Result<(), Box<dyn Error>> {
    let original = bytes.to_vec();
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    match format {
        DxfRawDocumentFormat::Ascii => {
            let document = open_ascii(&source)?;
            let evidence = document.entity_field_evidence_directory(&token())?;
            let plan = planned(plan_ascii(
                &document,
                &evidence,
                first_key(&evidence)?,
                DxfEntityField::LINETYPE,
                DxfEntityEditValue::ExactRawText(b"DASHED"),
            )?)?;
            assert_eq!(
                plan.transaction().replacement_bytes_for_patch_ordinal(0),
                Some(b"6\r\nDASHED\r\n".as_slice())
            );
            verify_post_image(
                bytes,
                DxfRawDocumentView::from(&document),
                plan.transaction(),
                format,
            )?;
        }
        DxfRawDocumentFormat::Binary => {
            let document = open_binary(&source)?;
            let evidence = document.entity_field_evidence_directory(&token())?;
            let plan = planned(document.plan_entity_field_insertion(
                &evidence,
                first_key(&evidence)?,
                DxfEntityField::LINETYPE,
                DxfEntityEditValue::ExactRawText(b"DASHED"),
                DxfResourceProfile::Safe,
                &token(),
            )?)?;
            verify_post_image(
                bytes,
                DxfRawDocumentView::from(&document),
                plan.transaction(),
                format,
            )?;
        }
        _ => return Err(io::Error::other("unsupported format").into()),
    }
    assert_eq!(bytes, original);
    Ok(())
}

fn verify_post_image(
    original: &[u8],
    source_view: DxfRawDocumentView<'_>,
    plan: &DxfTransactionPlan,
    format: DxfRawDocumentFormat,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(plan.patches().len(), 1);
    assert!(plan.patches()[0].source_span().is_empty());
    assert_eq!(
        plan.inverse_bytes_for_patch_ordinal(0),
        Some(b"".as_slice())
    );
    let output = materialize(original, plan)?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    match format {
        DxfRawDocumentFormat::Ascii => {
            let post = open_ascii(&output_source)?;
            assert_linetype(DxfRawDocumentView::from(&post), b"DASHED")?;
            let inverse = plan.materialize_inverse_plan(
                source_view,
                DxfRawDocumentView::from(&post),
                DxfResourceProfile::Safe,
                &token(),
            )?;
            assert_eq!(materialize(&output, &inverse)?, original);
        }
        DxfRawDocumentFormat::Binary => {
            let post = open_binary(&output_source)?;
            assert_linetype(DxfRawDocumentView::from(&post), b"DASHED")?;
            let inverse = plan.materialize_inverse_plan(
                source_view,
                DxfRawDocumentView::from(&post),
                DxfResourceProfile::Safe,
                &token(),
            )?;
            assert_eq!(materialize(&output, &inverse)?, original);
        }
        _ => return Err(io::Error::other("unsupported format").into()),
    }
    Ok(())
}

fn assert_linetype(view: DxfRawDocumentView<'_>, expected: &[u8]) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_field_semantic_directory(&token())?;
    let entity = directory.evidence_directory().entity_directory().entities()[0];
    let entry = directory
        .entry_for_field(entity, DxfEntityField::LINETYPE)?
        .ok_or(io::Error::other("linetype"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("singleton").into());
    };
    let Some(DxfEntityFieldValue::ExactText(text)) = value.value().copied() else {
        return Err(io::Error::other("text").into());
    };
    let mut bytes = vec![0_u8; expected.len()];
    view.read_span(text.value_span(), &mut bytes)?;
    assert_eq!(bytes, expected);
    Ok(())
}

#[derive(Clone, Copy)]
enum FixtureShape {
    Absent,
    AllSingletonsAbsent,
    Conflicts,
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Binary(&'a [u8]),
    Double(f64),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    shape: FixtureShape,
    ending: &[u8],
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
    ];
    if !matches!(shape, FixtureShape::AllSingletonsAbsent) {
        groups.push((5, Value::Text(b"10")));
    }
    if version != DxfAcadVersion::Ac1009 {
        if !matches!(shape, FixtureShape::AllSingletonsAbsent) {
            groups.push((330, Value::Text(b"1F")));
        }
        groups.push((100, Value::Text(b"AcDbEntity")));
    }
    if !matches!(shape, FixtureShape::AllSingletonsAbsent) {
        groups.push((8, Value::Text(b"0")));
    }
    if matches!(shape, FixtureShape::Conflicts) {
        groups.extend([
            (8, Value::Text(b"SECOND")),
            (310, Value::Binary(&[1])),
            (310, Value::Binary(&[2])),
        ]);
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
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups, ending)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("unsupported format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)], ending: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(ending);
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Binary(value) => {
                for byte in *value {
                    bytes.extend_from_slice(format!("{byte:02X}").as_bytes());
                }
            }
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.extend_from_slice(ending);
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
            Value::Binary(value) => {
                bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk"))?);
                bytes.extend_from_slice(value);
            }
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }
    }
    Ok(bytes)
}

fn plan_ascii_field(
    bytes: &[u8],
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> Result<DxfEntityFieldInsertionPlan, Box<dyn Error>> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    Ok(planned(plan_ascii(
        &document,
        &evidence,
        first_key(&evidence)?,
        field,
        value,
    )?)?)
}

fn plan_ascii(
    document: &DxfAsciiRawDocument<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> Result<DxfEntityFieldInsertionOutcome, DxfError> {
    document.plan_entity_field_insertion(
        evidence,
        key,
        field,
        value,
        DxfResourceProfile::Safe,
        &token(),
    )
}

fn planned(
    outcome: DxfEntityFieldInsertionOutcome,
) -> Result<DxfEntityFieldInsertionPlan, io::Error> {
    match outcome {
        DxfEntityFieldInsertionOutcome::Planned(plan) => Ok(plan),
        other => Err(io::Error::other(format!(
            "unexpected insertion outcome: {other:?}"
        ))),
    }
}

fn assert_issue(
    outcome: DxfEntityFieldInsertionOutcome,
    expected: DxfEntityFieldInsertionIssue,
) -> Result<(), io::Error> {
    match outcome {
        DxfEntityFieldInsertionOutcome::Unavailable(issue) if issue == expected => Ok(()),
        other => Err(io::Error::other(format!(
            "unexpected insertion outcome: {other:?}"
        ))),
    }
}

fn first_key(evidence: &DxfEntityFieldEvidenceDirectory) -> Result<DxfEntityKey, io::Error> {
    evidence
        .entity_directory()
        .entities()
        .first()
        .copied()
        .map(|entity| entity.key())
        .ok_or_else(|| io::Error::other("entity"))
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

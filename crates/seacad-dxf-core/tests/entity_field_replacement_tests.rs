use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityEditValue, DxfEntityField,
    DxfEntityFieldReplacementIssue, DxfEntityFieldReplacementOutcome,
    DxfEntityFieldReplacementPlan, DxfEntityFieldSemantics, DxfEntityFieldValue,
    DxfEntityGroupEncodeIssue, DxfEntityKey, DxfError, DxfHandle, DxfMemorySource,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_replaces_strict_ascii_binary_and_materializes_exact_inverse()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = fixture(DxfRawDocumentFormat::Ascii, version, FixtureShape::Unique)?;
        verify_layer_replacement(&ascii_bytes, DxfRawDocumentFormat::Ascii)?;

        let binary_bytes = fixture(DxfRawDocumentFormat::Binary, version, FixtureShape::Unique)?;
        verify_layer_replacement(&binary_bytes, DxfRawDocumentFormat::Binary)?;
    }
    Ok(())
}

#[test]
fn replacement_targets_exact_group_and_supports_singleton_wire_domains()
-> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let bytes = fixture(DxfRawDocumentFormat::Ascii, version, FixtureShape::Unique)?;
    for (field, value, expected) in [
        (
            DxfEntityField::HANDLE,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(0xab)),
            b"5\nAB\n".as_slice(),
        ),
        (
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"NEW"),
            b"8\nNEW\n",
        ),
        (
            DxfEntityField::COLOR,
            DxfEntityEditValue::Int16(-7),
            b"62\n-7\n",
        ),
        (
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(2.5)),
            b"48\n2.5\n",
        ),
        (
            DxfEntityField::PROXY_GRAPHICS_SIZE,
            DxfEntityEditValue::Int32(9),
            b"92\n9\n",
        ),
    ] {
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let evidence = document.entity_field_evidence_directory(&token())?;
        let key = first_semantic_key(&evidence)?;
        let plan = planned(document.plan_entity_field_replacement(
            &evidence,
            key,
            field,
            value,
            DxfResourceProfile::Safe,
            &token(),
        )?)?;
        assert_eq!(plan.key(), key);
        assert_eq!(plan.source_id(), key.source_id());
        assert_eq!(plan.field(), field);
        assert_eq!(plan.transaction().patches().len(), 1);
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
fn absent_duplicate_sequence_wrong_section_dialect_and_encoding_fail_typed()
-> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        version,
        FixtureShape::DuplicateAndWrongSection,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let entities = evidence.entity_directory().entities();
    let editable = entities
        .iter()
        .copied()
        .find(|entity| {
            entity.record().section_kind() == seacad_dxf_core::DxfRawRecordSectionKind::Entities
        })
        .ok_or(io::Error::other("editable entity"))?;
    let wrong = entities
        .iter()
        .copied()
        .find(|entity| {
            entity.record().section_kind() == seacad_dxf_core::DxfRawRecordSectionKind::Objects
        })
        .ok_or(io::Error::other("wrong-section entity"))?;

    assert_issue(
        plan(
            &document,
            &evidence,
            editable.key(),
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"X"),
        )?,
        DxfEntityFieldReplacementIssue::DuplicateSingleton {
            occurrence_count: 2,
        },
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            editable.key(),
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"X"),
        )?,
        DxfEntityFieldReplacementIssue::FieldAbsent { required: false },
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            editable.key(),
            DxfEntityField::LAYOUT,
            DxfEntityEditValue::ExactRawText(b"Model"),
        )?,
        DxfEntityFieldReplacementIssue::FieldAbsent { required: true },
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            editable.key(),
            DxfEntityField::PROXY_GRAPHICS_DATA,
            DxfEntityEditValue::BinaryChunk(&[9]),
        )?,
        DxfEntityFieldReplacementIssue::SequenceOperationRequired {
            occurrence_count: 2,
        },
    )?;
    assert!(matches!(
        plan(
            &document,
            &evidence,
            wrong.key(),
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"X")
        )?,
        DxfEntityFieldReplacementOutcome::Unavailable(
            DxfEntityFieldReplacementIssue::WrongSection { .. }
        )
    ));
    assert_issue(
        plan(
            &document,
            &evidence,
            editable.key(),
            DxfEntityField::COLOR,
            DxfEntityEditValue::Int32(1),
        )?,
        DxfEntityFieldReplacementIssue::Encoding(DxfEntityGroupEncodeIssue::WireTypeMismatch {
            expected: seacad_dxf_core::DxfEntityFieldWireType::Int16,
            observed: seacad_dxf_core::DxfEntityEditValueKind::Int32,
        }),
    )?;

    let no_version = b"0\nSECTION\n2\nENTITIES\n0\nLINE\n8\nOLD\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(no_version, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_semantic_key(&evidence)?;
    assert!(matches!(
        plan(
            &document,
            &evidence,
            key,
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"X")
        )?,
        DxfEntityFieldReplacementOutcome::Unavailable(
            DxfEntityFieldReplacementIssue::DialectUnavailable { .. }
        )
    ));

    let ac1009 = fixture(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        FixtureShape::Unique,
    )?;
    let source = DxfMemorySource::new(&ac1009, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_semantic_key(&evidence)?;
    assert_issue(
        plan_binary(
            &document,
            &evidence,
            key,
            DxfEntityField::TRUE_COLOR,
            DxfEntityEditValue::Int32(1),
        )?,
        DxfEntityFieldReplacementIssue::FieldAbsent { required: false },
    )?;
    Ok(())
}

#[test]
fn source_identity_cancellation_traits_and_debug_redaction_fail_closed()
-> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityKey>();
    assert_copy_send_sync::<DxfEntityFieldReplacementIssue>();
    assert_send_sync::<DxfEntityFieldReplacementPlan>();
    assert!(std::mem::size_of::<DxfEntityKey>() <= 48);

    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::Unique,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_semantic_key(&evidence)?;
    assert_eq!(
        evidence
            .entity_directory()
            .entity_for_key(key)?
            .map(|entity| entity.key()),
        Some(key)
    );

    let other_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::DuplicateAndWrongSection,
    )?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        other.plan_entity_field_replacement(
            &evidence,
            key,
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"SECRET"),
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.plan_entity_field_replacement(
            &evidence,
            key,
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"SECRET"),
            DxfResourceProfile::Safe,
            &cancellation,
        ),
        Err(DxfError::Cancelled)
    ));

    let plan = planned(plan(
        &document,
        &evidence,
        key,
        DxfEntityField::LAYER,
        DxfEntityEditValue::ExactRawText(b"SECRET"),
    )?)?;
    let debug = format!("{plan:?}");
    assert!(!debug.contains("SECRET"));
    assert!(debug.contains("patch_count"));
    Ok(())
}

fn verify_layer_replacement(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
) -> Result<(), Box<dyn Error>> {
    let original = bytes.to_vec();
    match format {
        DxfRawDocumentFormat::Ascii => {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            let document = open_ascii(&source)?;
            let evidence = document.entity_field_evidence_directory(&token())?;
            let key = first_semantic_key(&evidence)?;
            let plan = planned(plan(
                &document,
                &evidence,
                key,
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(b"NEW"),
            )?)?;
            verify_post_image(
                bytes,
                DxfRawDocumentView::from(&document),
                plan.transaction(),
                format,
            )?;
        }
        DxfRawDocumentFormat::Binary => {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            let document = open_binary(&source)?;
            let evidence = document.entity_field_evidence_directory(&token())?;
            let key = first_semantic_key(&evidence)?;
            let plan = planned(plan_binary(
                &document,
                &evidence,
                key,
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(b"NEW"),
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
    let output = materialize(original, plan)?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    match format {
        DxfRawDocumentFormat::Ascii => {
            let post = open_ascii(&output_source)?;
            assert_layer(DxfRawDocumentView::from(&post), b"NEW")?;
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
            assert_layer(DxfRawDocumentView::from(&post), b"NEW")?;
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

fn assert_layer(view: DxfRawDocumentView<'_>, expected: &[u8]) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_field_semantic_directory(&token())?;
    let entity = directory.evidence_directory().entity_directory().entities()[0];
    let entry = directory
        .entry_for_field(entity, DxfEntityField::LAYER)?
        .ok_or(io::Error::other("layer semantics"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("layer singleton").into());
    };
    let Some(DxfEntityFieldValue::ExactText(text)) = value.value().copied() else {
        return Err(io::Error::other("layer exact text").into());
    };
    let mut bytes = vec![0_u8; expected.len()];
    view.read_span(text.value_span(), &mut bytes)?;
    assert_eq!(bytes, expected);
    Ok(())
}

#[derive(Clone, Copy)]
enum FixtureShape {
    Unique,
    DuplicateAndWrongSection,
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Int32(i32),
    Double(f64),
    Binary(&'a [u8]),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    shape: FixtureShape,
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
        (100, Value::Text(b"AcDbEntity")),
        (8, Value::Text(b"OLD")),
    ];
    if matches!(shape, FixtureShape::DuplicateAndWrongSection) {
        groups.push((8, Value::Text(b"SECOND")));
    }
    groups.extend([
        (62, Value::Int16(7)),
        (48, Value::Double(1.5)),
        (60, Value::Int16(0)),
        (92, Value::Int32(3)),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([(310, Value::Binary(&[1, 2])), (310, Value::Binary(&[3]))]);
    }
    groups.push((0, Value::Text(b"ENDSEC")));
    if matches!(shape, FixtureShape::DuplicateAndWrongSection) {
        groups.extend([
            (0, Value::Text(b"SECTION")),
            (2, Value::Text(b"OBJECTS")),
            (0, Value::Text(b"LINE")),
            (8, Value::Text(b"WRONG")),
            (0, Value::Text(b"ENDSEC")),
        ]);
    }
    groups.push((0, Value::Text(b"EOF")));
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("unsupported format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Binary(value) => {
                for byte in *value {
                    bytes.extend_from_slice(format!("{byte:02X}").as_bytes());
                }
            }
        }
        bytes.push(b'\n');
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
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 group code"))?);
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
            Value::Binary(value) => {
                bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk"))?);
                bytes.extend_from_slice(value);
            }
        }
    }
    Ok(bytes)
}

fn plan(
    document: &DxfAsciiRawDocument<'_>,
    evidence: &seacad_dxf_core::DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> Result<DxfEntityFieldReplacementOutcome, DxfError> {
    document.plan_entity_field_replacement(
        evidence,
        key,
        field,
        value,
        DxfResourceProfile::Safe,
        &token(),
    )
}

fn plan_binary(
    document: &DxfBinaryRawDocument<'_>,
    evidence: &seacad_dxf_core::DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> Result<DxfEntityFieldReplacementOutcome, DxfError> {
    document.plan_entity_field_replacement(
        evidence,
        key,
        field,
        value,
        DxfResourceProfile::Safe,
        &token(),
    )
}

fn planned(
    outcome: DxfEntityFieldReplacementOutcome,
) -> Result<DxfEntityFieldReplacementPlan, io::Error> {
    match outcome {
        DxfEntityFieldReplacementOutcome::Planned(plan) => Ok(plan),
        other => Err(io::Error::other(format!(
            "unexpected replacement outcome: {other:?}"
        ))),
    }
}

fn assert_issue(
    outcome: DxfEntityFieldReplacementOutcome,
    expected: DxfEntityFieldReplacementIssue,
) -> Result<(), io::Error> {
    match outcome {
        DxfEntityFieldReplacementOutcome::Unavailable(issue) if issue == expected => Ok(()),
        other => Err(io::Error::other(format!(
            "unexpected replacement outcome: {other:?}"
        ))),
    }
}

fn first_semantic_key(
    evidence: &seacad_dxf_core::DxfEntityFieldEvidenceDirectory,
) -> Result<DxfEntityKey, io::Error> {
    evidence
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .find(|entity| {
            !matches!(
                entity.classification(),
                seacad_dxf_core::DxfEntityClassification::WrongSection(_)
            )
        })
        .map(|entity| entity.key())
        .ok_or_else(|| io::Error::other("semantic entity"))
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
        output.extend_from_slice(
            bytes
                .get(cursor..start)
                .ok_or(io::Error::other("source range"))?,
        );
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

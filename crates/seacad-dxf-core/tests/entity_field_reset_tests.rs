use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityField, DxfEntityFieldResetIssue, DxfEntityFieldResetOutcome,
    DxfEntityFieldResetPlan, DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityKey, DxfError,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_resets_strict_ascii_binary_and_materializes_exact_inverse()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = fixture(DxfRawDocumentFormat::Ascii, version, FixtureShape::Unique)?;
        verify_linetype_reset(&ascii, DxfRawDocumentFormat::Ascii)?;

        let binary = fixture(DxfRawDocumentFormat::Binary, version, FixtureShape::Unique)?;
        verify_linetype_reset(&binary, DxfRawDocumentFormat::Binary)?;
    }
    Ok(())
}

#[test]
fn reset_deletes_exact_unique_optional_singletons_across_wire_domains() -> Result<(), Box<dyn Error>>
{
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::Unique,
    )?;
    for field in [
        DxfEntityField::PAPER_SPACE,
        DxfEntityField::LINETYPE,
        DxfEntityField::MATERIAL,
        DxfEntityField::COLOR,
        DxfEntityField::LINETYPE_SCALE,
        DxfEntityField::VISIBILITY,
        DxfEntityField::PROXY_GRAPHICS_SIZE,
        DxfEntityField::TRUE_COLOR,
        DxfEntityField::COLOR_NAME,
        DxfEntityField::TRANSPARENCY,
    ] {
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let evidence = document.entity_field_evidence_directory(&token())?;
        let key = first_semantic_key(&evidence)?;
        let plan = planned(document.plan_entity_field_reset_to_default(
            &evidence,
            key,
            field,
            DxfResourceProfile::Safe,
            &token(),
        )?)?;
        assert_eq!(plan.source_id(), key.source_id());
        assert_eq!(plan.key(), key);
        assert_eq!(plan.field(), field);
        assert_eq!(plan.transaction().patches().len(), 1);
        assert_eq!(
            plan.transaction().replacement_bytes_for_patch_ordinal(0),
            Some(b"".as_slice())
        );
        let output = materialize(&bytes, plan.transaction())?;
        let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
        open_ascii(&output_source)?;
    }
    Ok(())
}

#[test]
fn absent_required_duplicate_sequence_nested_wrong_section_and_dialect_fail_typed()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::Conflicts,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let editable = evidence
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .find(|entity| {
            entity.record().section_kind() == seacad_dxf_core::DxfRawRecordSectionKind::Entities
        })
        .ok_or(io::Error::other("editable entity"))?;
    let wrong = evidence
        .entity_directory()
        .entities()
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
            DxfEntityField::LINETYPE,
        )?,
        DxfEntityFieldResetIssue::DuplicateSingleton {
            occurrence_count: 2,
        },
    )?;
    assert_issue(
        plan(&document, &evidence, editable.key(), DxfEntityField::HANDLE)?,
        DxfEntityFieldResetIssue::RequiredField,
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            editable.key(),
            DxfEntityField::PROXY_GRAPHICS_DATA,
        )?,
        DxfEntityFieldResetIssue::SequenceOperationRequired {
            occurrence_count: 2,
        },
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            editable.key(),
            DxfEntityField::EXTENSION_DICTIONARY,
        )?,
        DxfEntityFieldResetIssue::NestedStructureOperationRequired {
            occurrence_count: 1,
        },
    )?;
    assert!(matches!(
        plan(&document, &evidence, wrong.key(), DxfEntityField::LINETYPE)?,
        DxfEntityFieldResetOutcome::Unavailable(DxfEntityFieldResetIssue::WrongSection { .. })
    ));
    let absent = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nLINE\n5\n10\n100\nAcDbEntity\n8\nLayer0\n0\nENDSEC\n0\nEOF\n";
    let absent_source = DxfMemorySource::new(absent, DxfResourceProfile::Safe)?;
    let absent_document = open_ascii(&absent_source)?;
    let absent_evidence = absent_document.entity_field_evidence_directory(&token())?;
    let absent_key = first_semantic_key(&absent_evidence)?;
    assert!(matches!(
        plan(
            &absent_document,
            &absent_evidence,
            absent_key,
            DxfEntityField::LINETYPE
        )?,
        DxfEntityFieldResetOutcome::AlreadyImplicit { key, field }
            if key == absent_key && field == DxfEntityField::LINETYPE
    ));
    assert_issue(
        plan(
            &absent_document,
            &absent_evidence,
            absent_key,
            DxfEntityField::PROXY_GRAPHICS_DATA,
        )?,
        DxfEntityFieldResetIssue::SequenceOperationRequired {
            occurrence_count: 0,
        },
    )?;

    let no_version = b"0\nSECTION\n2\nENTITIES\n0\nLINE\n6\nDASHED\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(no_version, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_semantic_key(&evidence)?;
    assert!(matches!(
        plan(&document, &evidence, key, DxfEntityField::LINETYPE)?,
        DxfEntityFieldResetOutcome::Unavailable(
            DxfEntityFieldResetIssue::DialectUnavailable { .. }
        )
    ));
    Ok(())
}

#[test]
fn source_identity_cancellation_traits_and_debug_metadata_fail_closed() -> Result<(), Box<dyn Error>>
{
    assert_copy_send_sync::<DxfEntityFieldResetIssue>();
    assert_send_sync::<DxfEntityFieldResetPlan>();

    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        FixtureShape::Unique,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_semantic_key(&evidence)?;

    let other_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1027,
        FixtureShape::Unique,
    )?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        other.plan_entity_field_reset_to_default(
            &evidence,
            key,
            DxfEntityField::LINETYPE,
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.plan_entity_field_reset_to_default(
            &evidence,
            key,
            DxfEntityField::LINETYPE,
            DxfResourceProfile::Safe,
            &cancellation,
        ),
        Err(DxfError::Cancelled)
    ));

    let plan = planned(plan(&document, &evidence, key, DxfEntityField::LINETYPE)?)?;
    let debug = format!("{plan:?}");
    assert!(debug.contains("field"));
    assert!(debug.contains("transaction"));
    Ok(())
}

fn verify_linetype_reset(bytes: &[u8], format: DxfRawDocumentFormat) -> Result<(), Box<dyn Error>> {
    let original = bytes.to_vec();
    match format {
        DxfRawDocumentFormat::Ascii => {
            let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
            let document = open_ascii(&source)?;
            let evidence = document.entity_field_evidence_directory(&token())?;
            let key = first_semantic_key(&evidence)?;
            let plan = planned(plan(&document, &evidence, key, DxfEntityField::LINETYPE)?)?;
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
            let plan = planned(document.plan_entity_field_reset_to_default(
                &evidence,
                key,
                DxfEntityField::LINETYPE,
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
    let output = materialize(original, plan)?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    match format {
        DxfRawDocumentFormat::Ascii => {
            let post = open_ascii(&output_source)?;
            assert_default_linetype(DxfRawDocumentView::from(&post))?;
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
            assert_default_linetype(DxfRawDocumentView::from(&post))?;
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

fn assert_default_linetype(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_field_semantic_directory(&token())?;
    let entity = directory.evidence_directory().entity_directory().entities()[0];
    let entry = directory
        .entry_for_field(entity, DxfEntityField::LINETYPE)?
        .ok_or(io::Error::other("linetype semantics"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("linetype singleton").into());
    };
    assert_eq!(value.state(), DxfSemanticValueState::Defaulted);
    assert_eq!(
        value.value(),
        Some(&DxfEntityFieldValue::SchemaExactText("BYLAYER"))
    );
    Ok(())
}

fn plan(
    document: &DxfAsciiRawDocument<'_>,
    evidence: &seacad_dxf_core::DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    field: DxfEntityField,
) -> Result<DxfEntityFieldResetOutcome, DxfError> {
    document.plan_entity_field_reset_to_default(
        evidence,
        key,
        field,
        DxfResourceProfile::Safe,
        &token(),
    )
}

fn planned(outcome: DxfEntityFieldResetOutcome) -> Result<DxfEntityFieldResetPlan, io::Error> {
    match outcome {
        DxfEntityFieldResetOutcome::Planned(plan) => Ok(plan),
        other => Err(io::Error::other(format!(
            "unexpected reset outcome: {other:?}"
        ))),
    }
}

fn assert_issue(
    outcome: DxfEntityFieldResetOutcome,
    expected: DxfEntityFieldResetIssue,
) -> Result<(), io::Error> {
    match outcome {
        DxfEntityFieldResetOutcome::Unavailable(issue) if issue == expected => Ok(()),
        other => Err(io::Error::other(format!(
            "unexpected reset outcome: {other:?}"
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

#[derive(Clone, Copy)]
enum FixtureShape {
    Unique,
    Conflicts,
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
        (8, Value::Text(b"Layer0")),
        (67, Value::Int16(1)),
        (6, Value::Text(b"DASHED")),
        (62, Value::Int16(7)),
        (48, Value::Double(1.5)),
        (60, Value::Int16(1)),
        (92, Value::Int32(3)),
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (347, Value::Text(b"20")),
            (420, Value::Int32(0x010203)),
            (430, Value::Text(b"BOOK$NAME")),
            (440, Value::Int32(0x02000000)),
        ]);
    }
    if matches!(shape, FixtureShape::Conflicts) {
        groups.extend([
            (6, Value::Text(b"SECOND")),
            (310, Value::Binary(&[1, 2])),
            (310, Value::Binary(&[3])),
            (102, Value::Text(b"{ACAD_XDICTIONARY")),
            (360, Value::Text(b"A")),
            (102, Value::Text(b"}")),
        ]);
    }
    groups.extend([(4, Value::Text(b"KEEP")), (0, Value::Text(b"ENDSEC"))]);
    if matches!(shape, FixtureShape::Conflicts) {
        groups.extend([
            (0, Value::Text(b"SECTION")),
            (2, Value::Text(b"OBJECTS")),
            (0, Value::Text(b"LINE")),
            (6, Value::Text(b"WRONG")),
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

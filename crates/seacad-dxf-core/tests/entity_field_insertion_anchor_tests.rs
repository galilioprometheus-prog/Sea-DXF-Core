use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfApplicationGroupState, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfEntityClassification,
    DxfEntityField, DxfEntityFieldEvidenceDirectory, DxfEntityFieldInsertionAnchor,
    DxfEntityFieldInsertionAnchorIssue, DxfEntityFieldInsertionAnchorOutcome, DxfEntityKey,
    DxfError, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_canonical_anchor_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = canonical_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_neighbors = verify_canonical_anchor(&ascii, DxfRawDocumentFormat::Ascii)?;
        let binary = canonical_fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_neighbors = verify_canonical_anchor(&binary, DxfRawDocumentFormat::Binary)?;
        assert_eq!(ascii_neighbors, binary_neighbors);
    }
    Ok(())
}

#[test]
fn preamble_anchors_stay_outside_application_group_envelopes() -> Result<(), Box<dyn Error>> {
    let groups = document_groups(
        DxfAcadVersion::Ac1032,
        &[
            (0, Value::Text(b"LINE")),
            (102, Value::Text(b"{ACAD_REACTORS")),
            (330, Value::Text(b"AA")),
            (102, Value::Text(b"}")),
            (102, Value::Text(b"{ACAD_XDICTIONARY")),
            (360, Value::Text(b"BB")),
            (102, Value::Text(b"}")),
            (100, Value::Text(b"AcDbEntity")),
            (8, Value::Text(b"Layer0")),
            (62, Value::Int16(7)),
            (100, Value::Text(b"AcDbLine")),
            (10, Value::Double(0.0)),
        ],
        None,
    );
    let bytes = ascii_groups(&groups);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let key = first_key(&evidence)?;

    let handle = planned(document.plan_entity_field_insertion_anchor(
        &evidence,
        key,
        DxfEntityField::HANDLE,
        &token(),
    )?)?;
    assert_eq!(
        group_code(&document, handle.following_group_occurrence())?,
        102
    );
    assert_eq!(
        handle.byte_offset(),
        group(&document, handle.following_group_occurrence())?
            .full_span()
            .start()
    );

    let owner = planned(document.plan_entity_field_insertion_anchor(
        &evidence,
        key,
        DxfEntityField::OWNER,
        &token(),
    )?)?;
    assert_eq!(
        group_code(&document, owner.preceding_group_occurrence())?,
        102
    );
    assert_eq!(
        group_code(&document, owner.following_group_occurrence())?,
        100
    );
    assert_issue(
        document.plan_entity_field_insertion_anchor(
            &evidence,
            key,
            DxfEntityField::EXTENSION_DICTIONARY,
            &token(),
        )?,
        DxfEntityFieldInsertionAnchorIssue::FieldAlreadyPresent {
            occurrence_count: 1,
        },
    )?;
    Ok(())
}

#[test]
fn present_nested_sequence_subclass_order_section_and_dialect_fail_typed()
-> Result<(), Box<dyn Error>> {
    let body = [
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (100, Value::Text(b"AcDbEntity")),
        (8, Value::Text(b"A")),
        (8, Value::Text(b"B")),
        (310, Value::Binary(&[1])),
        (310, Value::Binary(&[2])),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"11")),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"12")),
        (100, Value::Text(b"AcDbEntity")),
        (8, Value::Text(b"0")),
        (100, Value::Text(b"AcDbEntity")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"13")),
        (100, Value::Text(b"AcDbEntity")),
        (62, Value::Int16(7)),
        (8, Value::Text(b"0")),
        (100, Value::Text(b"AcDbLine")),
    ];
    let groups = document_groups(
        DxfAcadVersion::Ac1032,
        &body,
        Some(&[(0, Value::Text(b"LINE"))]),
    );
    let bytes = ascii_groups(&groups);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let entities = evidence.entity_directory().entities().to_vec();

    assert_issue(
        plan(
            &document,
            &evidence,
            entities[0].key(),
            DxfEntityField::LAYER,
        )?,
        DxfEntityFieldInsertionAnchorIssue::FieldAlreadyPresent {
            occurrence_count: 2,
        },
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            entities[0].key(),
            DxfEntityField::PROXY_GRAPHICS_DATA,
        )?,
        DxfEntityFieldInsertionAnchorIssue::SequenceOperationRequired {
            occurrence_count: 2,
        },
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            entities[0].key(),
            DxfEntityField::EXTENSION_DICTIONARY,
        )?,
        DxfEntityFieldInsertionAnchorIssue::NestedStructureOperationRequired {
            occurrence_count: 0,
        },
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            entities[1].key(),
            DxfEntityField::LINETYPE,
        )?,
        DxfEntityFieldInsertionAnchorIssue::MissingAcDbEntitySubclass,
    )?;
    assert_issue(
        plan(
            &document,
            &evidence,
            entities[2].key(),
            DxfEntityField::LINETYPE,
        )?,
        DxfEntityFieldInsertionAnchorIssue::DuplicateAcDbEntitySubclass {
            occurrence_count: 2,
        },
    )?;
    assert!(matches!(
        plan(
            &document,
            &evidence,
            entities[3].key(),
            DxfEntityField::LINETYPE
        )?,
        DxfEntityFieldInsertionAnchorOutcome::Unavailable(
            DxfEntityFieldInsertionAnchorIssue::ConflictingCommonFieldOrder {
                lower_group_occurrence,
                higher_group_occurrence
            }
        ) if lower_group_occurrence > higher_group_occurrence
    ));
    let wrong = entities
        .iter()
        .copied()
        .find(|entity| {
            matches!(
                entity.classification(),
                DxfEntityClassification::WrongSection(_)
            )
        })
        .ok_or(io::Error::other("wrong section"))?;
    assert!(matches!(
        plan(&document, &evidence, wrong.key(), DxfEntityField::LAYER)?,
        DxfEntityFieldInsertionAnchorOutcome::Unavailable(
            DxfEntityFieldInsertionAnchorIssue::WrongSection { .. }
        )
    ));

    let no_version = b"0\nSECTION\n2\nENTITIES\n0\nLINE\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(no_version, DxfResourceProfile::Safe)?;
    let no_version_document = open_ascii(&source)?;
    let no_version_evidence = no_version_document.entity_field_evidence_directory(&token())?;
    assert!(matches!(
        plan(
            &no_version_document,
            &no_version_evidence,
            first_key(&no_version_evidence)?,
            DxfEntityField::HANDLE
        )?,
        DxfEntityFieldInsertionAnchorOutcome::Unavailable(
            DxfEntityFieldInsertionAnchorIssue::DialectUnavailable { .. }
        )
    ));
    Ok(())
}

#[test]
fn legacy_unclosed_groups_blocks_unknowns_source_identity_cancellation_and_traits_are_explicit()
-> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityFieldInsertionAnchor>();
    assert_copy_send_sync::<DxfEntityFieldInsertionAnchorIssue>();
    assert_copy_send_sync::<DxfEntityFieldInsertionAnchorOutcome>();

    let legacy = document_groups(
        DxfAcadVersion::Ac1009,
        &[
            (0, Value::Text(b"LINE")),
            (5, Value::Text(b"10")),
            (102, Value::Text(b"{ACAD_REACTORS")),
            (330, Value::Text(b"AA")),
        ],
        None,
    );
    let legacy_bytes = ascii_groups(&legacy);
    let legacy_source = DxfMemorySource::new(&legacy_bytes, DxfResourceProfile::Safe)?;
    let legacy_document = open_ascii(&legacy_source)?;
    let legacy_evidence = legacy_document.entity_field_evidence_directory(&token())?;
    assert!(matches!(
        plan(
            &legacy_document,
            &legacy_evidence,
            first_key(&legacy_evidence)?,
            DxfEntityField::LAYER
        )?,
        DxfEntityFieldInsertionAnchorOutcome::Unavailable(
            DxfEntityFieldInsertionAnchorIssue::ApplicationGroupUnavailable {
                state: DxfApplicationGroupState::Unclosed,
                ..
            }
        )
    ));

    let blocks = blocks_unknown_fixture();
    let source = DxfMemorySource::new(&blocks, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let evidence = document.entity_field_evidence_directory(&token())?;
    let entity = evidence.entity_directory().entities()[0];
    assert_eq!(entity.classification(), DxfEntityClassification::Unknown);
    let anchor = planned(plan(
        &document,
        &evidence,
        entity.key(),
        DxfEntityField::LINETYPE,
    )?)?;
    assert_eq!(
        group_code(&document, anchor.preceding_group_occurrence())?,
        999
    );
    assert_eq!(
        group_code(&document, anchor.following_group_occurrence())?,
        62
    );

    let other = canonical_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let other_source = DxfMemorySource::new(&other, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    assert!(matches!(
        other_document.plan_entity_field_insertion_anchor(
            &evidence,
            entity.key(),
            DxfEntityField::LINETYPE,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.plan_entity_field_insertion_anchor(
            &evidence,
            entity.key(),
            DxfEntityField::LINETYPE,
            &cancelled
        ),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn verify_canonical_anchor(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
) -> Result<(Option<u64>, Option<u64>), Box<dyn Error>> {
    let original = bytes.to_vec();
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let neighbors = match format {
        DxfRawDocumentFormat::Ascii => {
            let document = open_ascii(&source)?;
            let evidence = document.entity_field_evidence_directory(&token())?;
            verify_canonical_view(DxfRawDocumentView::from(&document), &evidence)?
        }
        DxfRawDocumentFormat::Binary => {
            let document = open_binary(&source)?;
            let evidence = document.entity_field_evidence_directory(&token())?;
            verify_canonical_view(DxfRawDocumentView::from(&document), &evidence)?
        }
        _ => return Err(io::Error::other("unsupported format").into()),
    };
    assert_eq!(bytes, original);
    Ok(neighbors)
}

fn verify_canonical_view(
    view: DxfRawDocumentView<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
) -> Result<(Option<u64>, Option<u64>), Box<dyn Error>> {
    let anchor = planned(view.plan_entity_field_insertion_anchor(
        evidence,
        first_key(evidence)?,
        DxfEntityField::LINETYPE,
        &token(),
    )?)?;
    assert_eq!(
        group_code_view(view, anchor.preceding_group_occurrence())?,
        8
    );
    assert_eq!(
        group_code_view(view, anchor.following_group_occurrence())?,
        62
    );
    assert_eq!(
        anchor.byte_offset(),
        group_view(view, anchor.following_group_occurrence())?
            .full_span()
            .start()
    );
    Ok((
        anchor.preceding_group_occurrence(),
        anchor.following_group_occurrence(),
    ))
}

fn canonical_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut body = vec![(0, Value::Text(b"LINE")), (5, Value::Text(b"10"))];
    if version != DxfAcadVersion::Ac1009 {
        body.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    body.extend([(8, Value::Text(b"0")), (62, Value::Int16(7))]);
    if version != DxfAcadVersion::Ac1009 {
        body.push((100, Value::Text(b"AcDbLine")));
    }
    body.push((10, Value::Double(0.0)));
    let groups = document_groups(version, &body, None);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("unsupported format")),
    }
}

fn blocks_unknown_fixture() -> Vec<u8> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"BLOCKS")),
        (0, Value::Text(b"BLOCK")),
        (2, Value::Text(b"B")),
        (0, Value::Text(b"FUTURE_ENTITY")),
        (5, Value::Text(b"20")),
        (330, Value::Text(b"1F")),
        (100, Value::Text(b"AcDbEntity")),
        (8, Value::Text(b"0")),
        (999, Value::Text(b"opaque")),
        (62, Value::Int16(7)),
        (100, Value::Text(b"AcDbFuture")),
        (1, Value::Text(b"payload")),
        (0, Value::Text(b"ENDBLK")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    ascii_groups(&groups)
}

fn document_groups<'a>(
    version: DxfAcadVersion,
    entities: &[(i16, Value<'a>)],
    objects: Option<&[(i16, Value<'a>)]>,
) -> Vec<(i16, Value<'a>)> {
    let mut groups = header(version);
    groups.extend([(0, Value::Text(b"SECTION")), (2, Value::Text(b"ENTITIES"))]);
    groups.extend_from_slice(entities);
    groups.push((0, Value::Text(b"ENDSEC")));
    if let Some(objects) = objects {
        groups.extend([(0, Value::Text(b"SECTION")), (2, Value::Text(b"OBJECTS"))]);
        groups.extend_from_slice(objects);
        groups.push((0, Value::Text(b"ENDSEC")));
    }
    groups.push((0, Value::Text(b"EOF")));
    groups
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

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Double(f64),
    Binary(&'a [u8]),
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
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

fn plan(
    document: &DxfAsciiRawDocument<'_>,
    evidence: &DxfEntityFieldEvidenceDirectory,
    key: DxfEntityKey,
    field: DxfEntityField,
) -> Result<DxfEntityFieldInsertionAnchorOutcome, DxfError> {
    document.plan_entity_field_insertion_anchor(evidence, key, field, &token())
}

fn planned(
    outcome: DxfEntityFieldInsertionAnchorOutcome,
) -> Result<DxfEntityFieldInsertionAnchor, io::Error> {
    match outcome {
        DxfEntityFieldInsertionAnchorOutcome::Planned(anchor) => Ok(anchor),
        other => Err(io::Error::other(format!(
            "unexpected anchor outcome: {other:?}"
        ))),
    }
}

fn assert_issue(
    outcome: DxfEntityFieldInsertionAnchorOutcome,
    expected: DxfEntityFieldInsertionAnchorIssue,
) -> Result<(), io::Error> {
    match outcome {
        DxfEntityFieldInsertionAnchorOutcome::Unavailable(issue) if issue == expected => Ok(()),
        other => Err(io::Error::other(format!(
            "unexpected anchor outcome: {other:?}"
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

fn group_code(
    document: &DxfAsciiRawDocument<'_>,
    occurrence: Option<u64>,
) -> Result<i16, io::Error> {
    group(document, occurrence).map(|raw| raw.group_code().value())
}

fn group(
    document: &DxfAsciiRawDocument<'_>,
    occurrence: Option<u64>,
) -> Result<seacad_dxf_core::DxfRawGroup, io::Error> {
    group_view(DxfRawDocumentView::from(document), occurrence)
}

fn group_code_view(
    document: DxfRawDocumentView<'_>,
    occurrence: Option<u64>,
) -> Result<i16, io::Error> {
    group_view(document, occurrence).map(|raw| raw.group_code().value())
}

fn group_view(
    document: DxfRawDocumentView<'_>,
    occurrence: Option<u64>,
) -> Result<seacad_dxf_core::DxfRawGroup, io::Error> {
    document
        .group(occurrence.ok_or(io::Error::other("group occurrence"))?)
        .ok_or_else(|| io::Error::other("group"))
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

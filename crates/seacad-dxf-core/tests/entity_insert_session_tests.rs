use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityCommonFieldPatch, DxfEntityDraft, DxfEntityEditIssue,
    DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityField, DxfEntityInsertIssue,
    DxfEntityInsertOutcome, DxfEntityInsertOwnerIssue, DxfEntityLineweight, DxfEntityPatch,
    DxfEntityTopic, DxfError, DxfHandle, DxfHandleIdentityLookup, DxfMemorySource, DxfPointDraft,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    DxfTransactionPlan, NoopDxfReadObserver,
};

const LOCATION: [DxfDouble; 3] = [
    DxfDouble::from_bits(1.25_f64.to_bits()),
    DxfDouble::from_bits((-2.5_f64).to_bits()),
    DxfDouble::from_bits(3.75_f64.to_bits()),
];

#[test]
fn session_inserts_and_verifies_point_across_every_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let placement = only_placement(view)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let draft = point_draft(version).with_owner(handle(0x10));
            assert_eq!(draft.owner(), Some(handle(0x10)));
            let DxfEntityInsertOutcome::Applied(receipt) = session.insert(placement, draft)? else {
                return Err(io::Error::other("session POINT insertion").into());
            };
            assert_eq!(receipt.handle(), handle(0x40));
            assert_eq!(
                receipt.name().canonical_topic(),
                Some(DxfEntityTopic::POINT)
            );
            assert_eq!(receipt.placement(), placement.target());
            assert_eq!(session.queued_edit_count(), 1);
            let debug = format!("{session:?}");
            assert!(!debug.contains("Layer0"));
            assert!(!debug.contains("Model"));

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            let output = materialize(&bytes, plan.transaction())?;
            assert_eq!(bytes, fixture(format, version)?);
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            assert!(matches!(
                post.handle_identity_directory(&token())?
                    .lookup(handle(0x40)),
                DxfHandleIdentityLookup::Unique(_)
            ));
            let outcome = plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?;
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) = outcome
            else {
                return Err(io::Error::other("verified session POINT insertion").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn session_finish_returns_the_atomic_insert_transaction() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let placement = only_placement(view)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.insert(
            placement,
            point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10))
        )?,
        DxfEntityInsertOutcome::Applied(_)
    ));
    let transaction = session.finish()?;
    assert_eq!(transaction.patches().len(), 2);
    let output = materialize(&bytes, &transaction)?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_ascii(&output_source)?;
    assert!(matches!(
        output_document
            .handle_identity_directory(&token())?
            .lookup(handle(0x40)),
        DxfHandleIdentityLookup::Unique(_)
    ));
    let inverse = transaction.materialize_inverse_plan(
        view,
        DxfRawDocumentView::from(&output_document),
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(materialize(&output, &inverse)?, bytes);
    Ok(())
}

#[test]
fn session_insert_failures_do_not_queue_or_mix_operations() -> Result<(), Box<dyn Error>> {
    let bytes = fixture_with_point_ascii();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let placement = only_placement(view)?;

    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.insert(placement, point_draft(DxfAcadVersion::Ac1032))?,
        DxfEntityInsertOutcome::Unavailable(DxfEntityInsertIssue::OwnerRequired)
    ));
    assert_eq!(session.queued_edit_count(), 0);

    assert!(matches!(
        session.insert(
            placement,
            point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x77))
        )?,
        DxfEntityInsertOutcome::Unavailable(DxfEntityInsertIssue::Owner(
            DxfEntityInsertOwnerIssue::OwnerMissing { .. }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 0);

    let DxfEntityInsertOutcome::Applied(_) = session.insert(
        placement,
        point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10)),
    )?
    else {
        return Err(io::Error::other("first session insert").into());
    };
    assert!(matches!(
        session.insert(
            placement,
            point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10))
        )?,
        DxfEntityInsertOutcome::Unavailable(DxfEntityInsertIssue::InsertAlreadyQueued)
    ));
    let key = existing_point_key(&evidence)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
                field: DxfEntityField::LAYER,
                value: DxfEntityEditValue::ExactRawText(b"Layer1")
            })
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::InsertPending)
    ));
    assert_eq!(session.queued_edit_count(), 1);

    let cancellation = token();
    let mut update_first =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        update_first.update(
            key,
            DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
                field: DxfEntityField::LAYER,
                value: DxfEntityEditValue::ExactRawText(b"Layer1")
            })
        )?,
        DxfEntityEditOutcome::Applied(_)
    ));
    assert!(matches!(
        update_first.insert(
            placement,
            point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10))
        )?,
        DxfEntityInsertOutcome::Unavailable(DxfEntityInsertIssue::UpdatePending {
            queued_update_count: 1
        })
    ));
    assert_eq!(update_first.queued_edit_count(), 1);
    Ok(())
}

#[test]
fn session_insert_rejects_foreign_placement_cancellation_and_record_error()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;

    let other_bytes = fixture_with_handseed_ascii(0x50);
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let foreign = only_placement(DxfRawDocumentView::from(&other_document))?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.insert(
            foreign,
            point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10))
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert_eq!(session.queued_edit_count(), 0);

    assert!(matches!(
        session.insert(
            only_placement(view)?,
            DxfEntityDraft::point(
                DxfPointDraft::new(b"Missing", LOCATION)
                    .with_layout(b"Model")
                    .with_lineweight(DxfEntityLineweight::BY_LAYER)
            )
            .with_owner(handle(0x10))
        )?,
        DxfEntityInsertOutcome::Unavailable(DxfEntityInsertIssue::Record(_))
    ));
    assert_eq!(session.queued_edit_count(), 0);

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.insert(
            only_placement(view)?,
            point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10))
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    assert_copy::<DxfEntityInsertIssue>();
    assert_copy::<DxfEntityInsertOutcome>();
    Ok(())
}

fn point_draft(version: DxfAcadVersion) -> DxfEntityDraft<'static> {
    let point = DxfPointDraft::new(b"Layer0", LOCATION);
    let point = if version >= DxfAcadVersion::Ac1015 {
        point
            .with_layout(b"Model")
            .with_lineweight(DxfEntityLineweight::BY_LAYER)
    } else {
        point
    };
    DxfEntityDraft::point(point)
}

fn existing_point_key(
    evidence: &seacad_dxf_core::DxfEntityFieldEvidenceDirectory,
) -> Result<seacad_dxf_core::DxfEntityKey, io::Error> {
    evidence
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .find(|entity| entity.classification().topic() == Some(DxfEntityTopic::POINT))
        .map(|entity| entity.key())
        .ok_or_else(|| io::Error::other("existing POINT key"))
}

fn only_placement(
    view: DxfRawDocumentView<'_>,
) -> Result<seacad_dxf_core::DxfEntityPlacement, Box<dyn Error>> {
    let directory = view.entity_placement_directory(&token())?;
    let [assessment] = directory.assessments() else {
        return Err(io::Error::other("one entity placement").into());
    };
    assessment
        .placement()
        .ok_or_else(|| io::Error::other("ready entity placement").into())
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0_i16, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version.code()),
        (9, "$HANDSEED"),
        (5, "40"),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "TABLES"),
        (0, "TABLE"),
        (2, "BLOCK_RECORD"),
        (0, "BLOCK_RECORD"),
        (5, "10"),
        (2, "*Model_Space"),
        (0, "ENDTAB"),
        (0, "TABLE"),
        (2, "LAYER"),
        (0, "LAYER"),
        (5, "11"),
        (2, "Layer0"),
        (0, "ENDTAB"),
        (0, "ENDSEC"),
    ];
    if version >= DxfAcadVersion::Ac1015 {
        groups.extend([
            (0, "SECTION"),
            (2, "OBJECTS"),
            (0, "LAYOUT"),
            (100, "AcDbPlotSettings"),
            (1, "PAGE_SETUP"),
            (100, "AcDbLayout"),
            (1, "Model"),
            (0, "ENDSEC"),
        ]);
    }
    groups.extend([(0, "SECTION"), (2, "ENTITIES"), (0, "ENDSEC"), (0, "EOF")]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_document(&groups)),
        DxfRawDocumentFormat::Binary => binary_document(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn fixture_with_handseed_ascii(handseed: u64) -> Vec<u8> {
    let handseed = format!("{handseed:X}");
    ascii_document(&[
        (0, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, "AC1032"),
        (9, "$HANDSEED"),
        (5, handseed.as_str()),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "TABLES"),
        (0, "TABLE"),
        (2, "BLOCK_RECORD"),
        (0, "BLOCK_RECORD"),
        (5, "10"),
        (2, "*Model_Space"),
        (0, "ENDTAB"),
        (0, "TABLE"),
        (2, "LAYER"),
        (0, "LAYER"),
        (5, "11"),
        (2, "Layer0"),
        (0, "ENDTAB"),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "OBJECTS"),
        (0, "LAYOUT"),
        (100, "AcDbPlotSettings"),
        (1, "PAGE_SETUP"),
        (100, "AcDbLayout"),
        (1, "Model"),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "ENTITIES"),
        (0, "ENDSEC"),
        (0, "EOF"),
    ])
}

fn fixture_with_point_ascii() -> Vec<u8> {
    let mut bytes = fixture_with_handseed_ascii(0x40);
    let marker = b"0\nENDSEC\n0\nEOF\n";
    let Some(offset) = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
    else {
        return bytes;
    };
    let point = b"0\nPOINT\n5\n20\n330\n10\n100\nAcDbEntity\n410\nModel\n8\nLayer0\n370\n-1\n100\nAcDbPoint\n10\n0\n20\n0\n30\n0\n";
    bytes.splice(offset..offset, point.iter().copied());
    let layer_end = b"0\nLAYER\n5\n11\n2\nLayer0\n0\nENDTAB\n";
    let Some(layer_start) = bytes
        .windows(layer_end.len())
        .position(|window| window == layer_end)
    else {
        return bytes;
    };
    let layer_offset = layer_start + layer_end.len() - b"0\nENDTAB\n".len();
    let layer1 = b"0\nLAYER\n5\n12\n2\nLayer1\n";
    bytes.splice(layer_offset..layer_offset, layer1.iter().copied());
    bytes
}

fn ascii_document(groups: &[(i16, &str)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

fn binary_document(version: DxfAcadVersion, groups: &[(i16, &str)]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
    }
    Ok(bytes)
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

const fn handle(value: u64) -> DxfHandle {
    DxfHandle::from_u64(value)
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

fn assert_copy<T: Copy>() {}

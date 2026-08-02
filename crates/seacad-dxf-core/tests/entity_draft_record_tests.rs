use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityCommonLayoutEditIssue,
    DxfEntityCommonSymbolEditIssue, DxfEntityDraft, DxfEntityDraftApplicabilityIssue,
    DxfEntityDraftApplicabilityPlan, DxfEntityDraftIdentityIssue, DxfEntityDraftName,
    DxfEntityDraftRecordIssue, DxfEntityDraftRecordPlan, DxfEntityEditVerificationIssue,
    DxfEntityEditVerificationOutcome, DxfEntityEditWriteOutcome, DxfEntityField,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityGroupEncodeIssue, DxfEntityLineweight,
    DxfEntityNameClassification, DxfEntityPlacementOwnerBinding, DxfEntityPlacementOwnerOutcome,
    DxfEntityPlacementTarget, DxfEntityTopic, DxfError, DxfHandle, DxfHandleIdentityLookup,
    DxfHandleReservationPlan, DxfHandleReservationPlanOutcome, DxfMemorySource,
    DxfNamedSymbolTableKind, DxfPointDraft, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, DxfTransactionPlan,
    NoopDxfReadObserver,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

const LOCATION: [DxfDouble; 3] = [
    DxfDouble::from_bits(1.25_f64.to_bits()),
    DxfDouble::from_bits((-2.5_f64).to_bits()),
    DxfDouble::from_bits(3.75_f64.to_bits()),
];

#[test]
fn point_record_is_canonical_and_composes_across_every_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, 0x40)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let plan = encode_point_plan(view, b"Layer0", LOCATION)?;

            assert_eq!(plan.source_id(), view.source_id());
            assert_eq!(
                plan.name(),
                DxfEntityDraftName::canonical(DxfEntityTopic::POINT)
            );
            assert_eq!(plan.version(), version);
            assert_eq!(plan.handle(), handle(0x40));
            assert_eq!(plan.bytes(), expected_point(format, version, 0x40, 0x10)?);
            assert_eq!(plan.transaction().patches().len(), 1);
            let debug = format!("{plan:?}");
            assert!(!debug.contains("Layer0"));
            assert!(!debug.contains("Model"));

            let insert = view.plan_entity_draft_insert(plan, DxfResourceProfile::Safe, &token())?;
            assert_eq!(insert.edit_count(), 1);
            assert_eq!(insert.transaction().patches().len(), 2);
            let output = materialize(&bytes, insert.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();

            assert!(matches!(
                post.handle_identity_directory(&token())?
                    .lookup(handle(0x40)),
                DxfHandleIdentityLookup::Unique(_)
            ));
            let semantics = post.basic_geometry_semantic_directory(&token())?;
            let [entry] = semantics.entries() else {
                return Err(io::Error::other("one POINT semantic entry").into());
            };
            let point = semantics
                .point_for_entry(*entry)?
                .ok_or_else(|| io::Error::other("POINT semantics"))?;
            assert_eq!(point.location_value(), Some(LOCATION));
            if version >= DxfAcadVersion::Ac1015 {
                assert_modern_common_fields(post)?;
            }

            let DxfEntityEditVerificationOutcome::Verified(journal) =
                insert.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("POINT insert verification").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            let inverse = journal.into_parts().1;
            assert_eq!(materialize(&output, &inverse)?, bytes);
        }
    }
    Ok(())
}

fn assert_modern_common_fields(post: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let fields = post.entity_field_semantic_directory(&token())?;
    let entity = fields
        .evidence_directory()
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .find(|entity| entity.classification().topic() == Some(DxfEntityTopic::POINT))
        .ok_or_else(|| io::Error::other("post POINT entity"))?;
    for field in [DxfEntityField::LAYOUT, DxfEntityField::LAYER] {
        let entry = fields
            .entry_for_field(entity, field)?
            .ok_or_else(|| io::Error::other("common text field"))?;
        let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
            return Err(io::Error::other("common text singleton").into());
        };
        assert_eq!(value.state(), DxfSemanticValueState::Explicit);
        assert!(matches!(
            value.value(),
            Some(DxfEntityFieldValue::ExactText(_))
        ));
    }
    let entry = fields
        .entry_for_field(entity, DxfEntityField::LINEWEIGHT)?
        .ok_or_else(|| io::Error::other("lineweight field"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("lineweight singleton").into());
    };
    assert_eq!(value.state(), DxfSemanticValueState::Explicit);
    assert_eq!(value.value(), Some(&DxfEntityFieldValue::Int16(-1)));
    Ok(())
}

#[test]
fn point_insert_verification_rejects_identity_family_common_and_geometry_tampering()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, 0x40)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let record = encode_point_plan(view, b"Layer0", LOCATION)?;
    let insert = view.plan_entity_draft_insert(record, DxfResourceProfile::Safe, &token())?;
    let output = materialize(&bytes, insert.transaction())?;

    let missing = replace_once(&output, b"0\nPOINT\n5\n40\n", b"0\nPOINT\n5\n41\n")?;
    assert!(matches!(
        verification_issue(&insert, view, &missing)?,
        DxfEntityEditVerificationIssue::MissingInsertedEntity { handle: observed }
            if observed == handle(0x40)
    ));

    let ambiguous = replace_once(
        &output,
        b"0\nLAYER\n5\n11\n2\nLayer0\n",
        b"0\nLAYER\n5\n40\n2\nLayer0\n",
    )?;
    assert!(matches!(
        verification_issue(&insert, view, &ambiguous)?,
        DxfEntityEditVerificationIssue::AmbiguousInsertedEntity {
            handle: observed,
            candidate_count: 2
        } if observed == handle(0x40)
    ));

    let wrong_family = replace_once(&output, b"0\nPOINT\n5\n40\n", b"0\nLINE \n5\n40\n")?;
    assert!(matches!(
        verification_issue(&insert, view, &wrong_family)?,
        DxfEntityEditVerificationIssue::InsertedEntityClassificationMismatch {
            handle: observed,
            ..
        } if observed == handle(0x40)
    ));

    let wrong_layer = replace_once(&output, b"8\nLayer0\n370\n", b"8\nLayer1\n370\n")?;
    assert!(matches!(
        verification_issue(&insert, view, &wrong_layer)?,
        DxfEntityEditVerificationIssue::ValueMismatch {
            field: DxfEntityField::LAYER,
            ..
        }
    ));

    let wrong_location = replace_once(&output, b"10\n1.25\n20\n", b"10\n1.26\n20\n")?;
    assert!(matches!(
        verification_issue(&insert, view, &wrong_location)?,
        DxfEntityEditVerificationIssue::InsertedPointLocationMismatch { handle: observed }
            if observed == handle(0x40)
    ));
    Ok(())
}

#[test]
fn point_insert_uses_create_new_verified_write_pipeline_for_every_dialect()
-> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, 0x40)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let record = encode_point_plan(view, b"Layer0", LOCATION)?;
            let insert =
                view.plan_entity_draft_insert(record, DxfResourceProfile::Safe, &token())?;
            let suffix = match format {
                DxfRawDocumentFormat::Ascii => "ascii",
                DxfRawDocumentFormat::Binary => "binary",
                _ => return Err(io::Error::other("format").into()),
            };
            let output = directory
                .path()
                .join(format!("{}-{suffix}.dxf", version.code()));
            let mut observer = NoopDxfReadObserver;
            let DxfEntityEditWriteOutcome::Written(journal) = insert
                .write_reparse_verify_and_journal_to_new_file(
                    view,
                    &output,
                    DxfResourceProfile::Safe,
                    &token(),
                    &mut observer,
                )?
            else {
                return Err(io::Error::other("verified POINT write").into());
            };
            assert_eq!(journal.verification_receipt().edit_count(), 1);
            let output_bytes = fs::read(&output)?;
            assert_eq!(materialize(&output_bytes, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_record_uses_placement_specific_modern_common_envelope() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED
        .into_iter()
        .filter(|version| *version >= DxfAcadVersion::Ac1012)
    {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = block_fixture(format, version, 0x40)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let applicability = admitted_block_plan(view)?;
            let mut point = DxfPointDraft::new(b"Layer0", LOCATION);
            if version >= DxfAcadVersion::Ac1015 {
                point = point.with_lineweight(DxfEntityLineweight::BY_LAYER);
            }
            let plan = match view.encode_entity_draft_record(
                applicability,
                DxfEntityDraft::point(point),
                DxfResourceProfile::Safe,
                &token(),
            )? {
                Ok(plan) => plan,
                Err(issue) => return Err(io::Error::other(format!("POINT: {issue:?}")).into()),
            };
            assert_eq!(
                plan.bytes(),
                expected_block_point(format, version, 0x40, 0x10)?
            );

            let insert = view.plan_entity_draft_insert(plan, DxfResourceProfile::Safe, &token())?;
            let output = materialize(&bytes, insert.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let semantics = post.basic_geometry_semantic_directory(&token())?;
            let [entry] = semantics.entries() else {
                return Err(io::Error::other("one block POINT semantic entry").into());
            };
            let point = semantics
                .point_for_entry(*entry)?
                .ok_or_else(|| io::Error::other("block POINT semantics"))?;
            assert_eq!(point.location_value(), Some(LOCATION));
            let DxfEntityEditVerificationOutcome::Verified(journal) =
                insert.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("block POINT insert verification").into());
            };
            let inverse = journal.into_parts().1;
            assert_eq!(materialize(&output, &inverse)?, bytes);

            if version >= DxfAcadVersion::Ac1015 {
                let applicability = admitted_block_plan(view)?;
                assert!(matches!(
                    view.encode_entity_draft_record(
                        applicability,
                        DxfEntityDraft::point(
                            DxfPointDraft::new(b"Layer0", LOCATION)
                                .with_layout(b"Model")
                                .with_lineweight(DxfEntityLineweight::BY_LAYER)
                        ),
                        DxfResourceProfile::Safe,
                        &token()
                    )?,
                    Err(DxfEntityDraftRecordIssue::LayoutNotApplicable {
                        target: DxfEntityPlacementTarget::BlockDefinition { .. },
                        ..
                    })
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn point_record_fails_closed_on_common_applicability_and_references() -> Result<(), Box<dyn Error>>
{
    let modern_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, 0x40)?;
    let modern_source = DxfMemorySource::new(&modern_bytes, DxfResourceProfile::Safe)?;
    let modern = open_ascii(&modern_source)?;
    let view = DxfRawDocumentView::from(&modern);

    let applicability = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(DxfPointDraft::new(b"Layer0", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LayoutRequired {
            target: DxfEntityPlacementTarget::EntitiesSection { .. }
        })
    ));

    let applicability = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(DxfPointDraft::new(b"Layer0", LOCATION).with_layout(b"Model")),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LineweightRequired {
            version: DxfAcadVersion::Ac1032
        })
    ));

    let applicability = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(
                DxfPointDraft::new(b"Layer0", LOCATION)
                    .with_layout(b"Missing")
                    .with_lineweight(DxfEntityLineweight::BY_LAYER)
            ),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LayoutReference(
            DxfEntityCommonLayoutEditIssue::Missing { .. }
        ))
    ));

    let applicability = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Missing", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LayerReference(
            DxfEntityCommonSymbolEditIssue::Missing {
                kind: DxfNamedSymbolTableKind::Layer,
                ..
            }
        ))
    ));

    let duplicate_layout_bytes = reference_fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        0x40,
        1,
        2,
    )?;
    let duplicate_layout_source =
        DxfMemorySource::new(&duplicate_layout_bytes, DxfResourceProfile::Safe)?;
    let duplicate_layout = open_ascii(&duplicate_layout_source)?;
    let duplicate_layout_view = DxfRawDocumentView::from(&duplicate_layout);
    let applicability = admitted_plan(
        duplicate_layout_view,
        DxfEntityDraftName::canonical(DxfEntityTopic::POINT),
    )?;
    assert!(matches!(
        duplicate_layout_view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Layer0", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LayoutReference(
            DxfEntityCommonLayoutEditIssue::Ambiguous {
                target_count: 2,
                ..
            }
        ))
    ));

    let duplicate_layer_bytes = reference_fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        0x40,
        2,
        1,
    )?;
    let duplicate_layer_source =
        DxfMemorySource::new(&duplicate_layer_bytes, DxfResourceProfile::Safe)?;
    let duplicate_layer = open_ascii(&duplicate_layer_source)?;
    let duplicate_layer_view = DxfRawDocumentView::from(&duplicate_layer);
    let applicability = admitted_plan(
        duplicate_layer_view,
        DxfEntityDraftName::canonical(DxfEntityTopic::POINT),
    )?;
    assert!(matches!(
        duplicate_layer_view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Layer0", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LayerReference(
            DxfEntityCommonSymbolEditIssue::Ambiguous {
                kind: DxfNamedSymbolTableKind::Layer,
                target_count: 2,
                ..
            }
        ))
    ));

    let old_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1014, 0x40)?;
    let old_source = DxfMemorySource::new(&old_bytes, DxfResourceProfile::Safe)?;
    let old = open_ascii(&old_source)?;
    let old_view = DxfRawDocumentView::from(&old);
    let applicability = admitted_plan(
        old_view,
        DxfEntityDraftName::canonical(DxfEntityTopic::POINT),
    )?;
    assert!(matches!(
        old_view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(DxfPointDraft::new(b"Layer0", LOCATION).with_layout(b"Model")),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LayoutNotApplicable {
            version: DxfAcadVersion::Ac1014,
            ..
        })
    ));
    let applicability = admitted_plan(
        old_view,
        DxfEntityDraftName::canonical(DxfEntityTopic::POINT),
    )?;
    assert!(matches!(
        old_view.encode_entity_draft_record(
            applicability,
            DxfEntityDraft::point(
                DxfPointDraft::new(b"Layer0", LOCATION)
                    .with_lineweight(DxfEntityLineweight::BY_LAYER)
            ),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::LineweightNotApplicable {
            version: DxfAcadVersion::Ac1014
        })
    ));
    Ok(())
}

#[test]
fn point_draft_rejects_name_layer_numeric_source_and_cancellation() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, 0x40)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);

    let wrong = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::LINE))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            wrong,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Layer0", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::NameMismatch { admitted, draft })
            if admitted == DxfEntityNameClassification::Canonical(DxfEntityTopic::LINE)
                && draft == DxfEntityNameClassification::Canonical(DxfEntityTopic::POINT)
    ));

    let wrong_owner = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            wrong_owner,
            DxfEntityDraft::point(point_draft(
                DxfAcadVersion::Ac1032,
                b"Layer0",
                LOCATION
            ))
            .with_owner(handle(0x11)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::OwnerMismatch {
            admitted,
            requested
        }) if admitted == handle(0x10) && requested == handle(0x11)
    ));

    let empty = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            empty,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::EmptyLayerName)
    ));

    let invalid_text = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        view.encode_entity_draft_record(
            invalid_text,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Bad\nLayer", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::GroupEncode {
            group_code: 8,
            issue: DxfEntityGroupEncodeIssue::ForbiddenTextByte {
                offset: 3,
                byte: b'\n'
            }
        })
    ));

    let invalid_number = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    let non_finite = [DxfDouble::from_f64(f64::NAN), LOCATION[1], LOCATION[2]];
    assert!(matches!(
        view.encode_entity_draft_record(
            invalid_number,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Layer0", non_finite)),
            DxfResourceProfile::Safe,
            &token()
        )?,
        Err(DxfEntityDraftRecordIssue::GroupEncode {
            group_code: 10,
            issue: DxfEntityGroupEncodeIssue::NonFiniteDouble(_)
        })
    ));

    let cancelled_plan = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        view.encode_entity_draft_record(
            cancelled_plan,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Layer0", LOCATION)),
            DxfResourceProfile::Safe,
            &cancelled
        ),
        Err(DxfError::Cancelled)
    ));

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, 0x50)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let foreign = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    assert!(matches!(
        other.encode_entity_draft_record(
            foreign,
            DxfEntityDraft::point(point_draft(DxfAcadVersion::Ac1032, b"Layer0", LOCATION)),
            DxfResourceProfile::Safe,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled_record = encode_point_plan(view, b"Layer0", LOCATION)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        view.plan_entity_draft_insert(cancelled_record, DxfResourceProfile::Safe, &cancelled),
        Err(DxfError::Cancelled)
    ));

    let foreign_record = encode_point_plan(view, b"Layer0", LOCATION)?;
    assert!(matches!(
        other.plan_entity_draft_insert(foreign_record, DxfResourceProfile::Safe, &token()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert_send_sync::<DxfEntityDraftRecordPlan>();
    assert_copy::<DxfEntityDraft<'static>>();
    assert_copy::<DxfPointDraft<'static>>();
    assert_copy::<DxfEntityDraftRecordIssue>();
    Ok(())
}

fn encode_point_plan<'a>(
    view: DxfRawDocumentView<'a>,
    layer: &'a [u8],
    location: [DxfDouble; 3],
) -> Result<DxfEntityDraftRecordPlan, Box<dyn Error>> {
    let applicability = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    let version = applicability.version();
    match view.encode_entity_draft_record(
        applicability,
        DxfEntityDraft::point(point_draft(version, layer, location)),
        DxfResourceProfile::Safe,
        &token(),
    )? {
        Ok(plan) => Ok(plan),
        Err(_) => Err(io::Error::other("POINT record plan").into()),
    }
}

fn point_draft(
    version: DxfAcadVersion,
    layer: &[u8],
    location: [DxfDouble; 3],
) -> DxfPointDraft<'_> {
    let draft = DxfPointDraft::new(layer, location);
    if version >= DxfAcadVersion::Ac1015 {
        draft
            .with_layout(b"Model")
            .with_lineweight(DxfEntityLineweight::BY_LAYER)
    } else {
        draft
    }
}

fn admitted_plan(
    view: DxfRawDocumentView<'_>,
    name: DxfEntityDraftName,
) -> Result<DxfEntityDraftApplicabilityPlan, Box<dyn Error>> {
    let binding = owner_binding(view)?;
    let policy = view.handle_allocation_policy_directory(&token())?;
    let reservation = reserve(view, &policy)?;
    let identity = match view.prepare_entity_draft_identity(name, binding, reservation, &token())? {
        Ok(plan) => plan,
        Err(DxfEntityDraftIdentityIssue::ReservationCardinality { .. }) => {
            return Err(io::Error::other("draft identity").into());
        }
        _ => return Err(io::Error::other("unknown draft identity issue").into()),
    };
    match view.prepare_entity_draft_applicability(identity, &token())? {
        Ok(plan) => Ok(plan),
        Err(
            DxfEntityDraftApplicabilityIssue::VersionUnavailable { .. }
            | DxfEntityDraftApplicabilityIssue::NotApplicable { .. }
            | DxfEntityDraftApplicabilityIssue::NotYetReviewed { .. },
        ) => Err(io::Error::other("draft applicability").into()),
        _ => Err(io::Error::other("unknown draft applicability issue").into()),
    }
}

fn admitted_block_plan(
    view: DxfRawDocumentView<'_>,
) -> Result<DxfEntityDraftApplicabilityPlan, Box<dyn Error>> {
    let plan = admitted_plan(view, DxfEntityDraftName::canonical(DxfEntityTopic::POINT))?;
    if !matches!(
        plan.identity().placement().target(),
        DxfEntityPlacementTarget::BlockDefinition { .. }
    ) {
        return Err(io::Error::other("BLOCK placement").into());
    }
    Ok(plan)
}

fn owner_binding(
    view: DxfRawDocumentView<'_>,
) -> Result<DxfEntityPlacementOwnerBinding, Box<dyn Error>> {
    let directory = view.entity_placement_owner_directory(&token())?;
    let [assessment] = directory.placement_directory().assessments() else {
        return Err(io::Error::other("one placement").into());
    };
    let placement = assessment
        .placement()
        .ok_or_else(|| io::Error::other("ready placement"))?;
    match directory.bind(placement, handle(0x10), &token())? {
        DxfEntityPlacementOwnerOutcome::Bound(binding) => Ok(binding),
        DxfEntityPlacementOwnerOutcome::Rejected(_) => {
            Err(io::Error::other("owner binding").into())
        }
        _ => Err(io::Error::other("unknown owner outcome").into()),
    }
}

fn reserve(
    view: DxfRawDocumentView<'_>,
    policy: &seacad_dxf_core::DxfHandleAllocationPolicyDirectory,
) -> Result<DxfHandleReservationPlan, Box<dyn Error>> {
    match view.plan_handle_reservation(policy, 1, DxfResourceProfile::Safe, &token())? {
        DxfHandleReservationPlanOutcome::Planned(plan) => Ok(plan),
        DxfHandleReservationPlanOutcome::PolicyUnavailable { .. }
        | DxfHandleReservationPlanOutcome::Exhausted { .. } => {
            Err(io::Error::other("reservation").into())
        }
        _ => Err(io::Error::other("unknown reservation outcome").into()),
    }
}

fn expected_point(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handle_value: u64,
    owner_value: u64,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        ExpectedGroup::Text(0, b"POINT"),
        ExpectedGroup::Handle(5, handle_value),
    ];
    if version >= DxfAcadVersion::Ac1012 {
        groups.extend([
            ExpectedGroup::Handle(330, owner_value),
            ExpectedGroup::Text(100, b"AcDbEntity"),
        ]);
    }
    if version >= DxfAcadVersion::Ac1015 {
        groups.push(ExpectedGroup::Text(410, b"Model"));
    }
    groups.push(ExpectedGroup::Text(8, b"Layer0"));
    if version >= DxfAcadVersion::Ac1015 {
        groups.push(ExpectedGroup::Int16(370, -1));
    }
    if version >= DxfAcadVersion::Ac1012 {
        groups.push(ExpectedGroup::Text(100, b"AcDbPoint"));
    }
    groups.extend([
        ExpectedGroup::Double(10, LOCATION[0]),
        ExpectedGroup::Double(20, LOCATION[1]),
        ExpectedGroup::Double(30, LOCATION[2]),
    ]);
    encode_expected(format, version, &groups)
}

fn expected_block_point(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handle_value: u64,
    owner_value: u64,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        ExpectedGroup::Text(0, b"POINT"),
        ExpectedGroup::Handle(5, handle_value),
        ExpectedGroup::Handle(330, owner_value),
        ExpectedGroup::Text(100, b"AcDbEntity"),
        ExpectedGroup::Text(8, b"Layer0"),
    ];
    if version >= DxfAcadVersion::Ac1015 {
        groups.push(ExpectedGroup::Int16(370, -1));
    }
    groups.extend([
        ExpectedGroup::Text(100, b"AcDbPoint"),
        ExpectedGroup::Double(10, LOCATION[0]),
        ExpectedGroup::Double(20, LOCATION[1]),
        ExpectedGroup::Double(30, LOCATION[2]),
    ]);
    encode_expected(format, version, &groups)
}

enum ExpectedGroup<'a> {
    Text(i16, &'a [u8]),
    Handle(i16, u64),
    Int16(i16, i16),
    Double(i16, DxfDouble),
}

fn encode_expected(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[ExpectedGroup<'_>],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::new();
    for group in groups {
        let code = match group {
            ExpectedGroup::Text(code, _)
            | ExpectedGroup::Handle(code, _)
            | ExpectedGroup::Int16(code, _)
            | ExpectedGroup::Double(code, _) => *code,
        };
        match format {
            DxfRawDocumentFormat::Ascii => {
                bytes.extend_from_slice(code.to_string().as_bytes());
                bytes.push(b'\n');
                match group {
                    ExpectedGroup::Text(_, value) => bytes.extend_from_slice(value),
                    ExpectedGroup::Handle(_, value) => {
                        bytes.extend_from_slice(format!("{value:X}").as_bytes());
                    }
                    ExpectedGroup::Int16(_, value) => {
                        bytes.extend_from_slice(value.to_string().as_bytes());
                    }
                    ExpectedGroup::Double(_, value) => {
                        bytes.extend_from_slice(value.to_f64().to_string().as_bytes());
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
                match group {
                    ExpectedGroup::Text(_, value) => {
                        bytes.extend_from_slice(value);
                        bytes.push(0);
                    }
                    ExpectedGroup::Handle(_, value) => {
                        bytes.extend_from_slice(format!("{value:X}").as_bytes());
                        bytes.push(0);
                    }
                    ExpectedGroup::Int16(_, value) => {
                        bytes.extend_from_slice(&value.to_le_bytes());
                    }
                    ExpectedGroup::Double(_, value) => {
                        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
                    }
                }
            }
            _ => return Err(io::Error::other("format")),
        }
    }
    Ok(bytes)
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handseed: u64,
) -> Result<Vec<u8>, io::Error> {
    reference_fixture(format, version, handseed, 1, 1)
}

fn reference_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handseed: u64,
    layer_count: u8,
    layout_count: u8,
) -> Result<Vec<u8>, io::Error> {
    let handseed = format!("{handseed:X}");
    let mut groups = vec![
        (0_i16, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version.code()),
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
    ];
    for handle in ["11", "12"].into_iter().take(usize::from(layer_count)) {
        groups.extend([(0, "LAYER"), (5, handle), (2, "Layer0")]);
    }
    groups.extend([(0, "ENDTAB"), (0, "ENDSEC")]);
    if version >= DxfAcadVersion::Ac1015 {
        groups.extend([(0, "SECTION"), (2, "OBJECTS")]);
        for _ in 0..layout_count {
            groups.extend([
                (0, "LAYOUT"),
                (100, "AcDbPlotSettings"),
                (1, "PAGE_SETUP"),
                (100, "AcDbLayout"),
                (1, "Model"),
            ]);
        }
        groups.push((0, "ENDSEC"));
    }
    groups.extend([(0, "SECTION"), (2, "ENTITIES"), (0, "ENDSEC"), (0, "EOF")]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_document(&groups)),
        DxfRawDocumentFormat::Binary => binary_document(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn block_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handseed: u64,
) -> Result<Vec<u8>, io::Error> {
    let handseed = format!("{handseed:X}");
    let groups = [
        (0_i16, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version.code()),
        (9, "$HANDSEED"),
        (5, handseed.as_str()),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "TABLES"),
        (0, "TABLE"),
        (2, "BLOCK_RECORD"),
        (0, "BLOCK_RECORD"),
        (5, "10"),
        (2, "B"),
        (0, "ENDTAB"),
        (0, "TABLE"),
        (2, "LAYER"),
        (0, "LAYER"),
        (5, "11"),
        (2, "Layer0"),
        (0, "ENDTAB"),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "BLOCKS"),
        (0, "BLOCK"),
        (5, "20"),
        (330, "10"),
        (100, "AcDbEntity"),
        (8, "0"),
        (100, "AcDbBlockBegin"),
        (2, "B"),
        (0, "ENDBLK"),
        (5, "21"),
        (330, "10"),
        (100, "AcDbEntity"),
        (8, "0"),
        (100, "AcDbBlockEnd"),
        (0, "ENDSEC"),
        (0, "EOF"),
    ];
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_document(&groups)),
        DxfRawDocumentFormat::Binary => binary_document(version, &groups),
        _ => Err(io::Error::other("format")),
    }
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

fn verification_issue(
    plan: &seacad_dxf_core::DxfEntityEditPlan,
    source: DxfRawDocumentView<'_>,
    post_bytes: &[u8],
) -> Result<DxfEntityEditVerificationIssue, Box<dyn Error>> {
    let post_source = DxfMemorySource::new(post_bytes, DxfResourceProfile::Safe)?;
    let post_document = open_ascii(&post_source)?;
    match plan.verify_post_image(
        source,
        DxfRawDocumentView::from(&post_document),
        DxfResourceProfile::Safe,
        &token(),
    )? {
        DxfEntityEditVerificationOutcome::Unavailable(issue) => Ok(issue),
        DxfEntityEditVerificationOutcome::Verified(_) => {
            Err(io::Error::other("expected unavailable insert verification").into())
        }
        _ => Err(io::Error::other("unknown insert verification outcome").into()),
    }
}

fn replace_once(source: &[u8], from: &[u8], to: &[u8]) -> Result<Vec<u8>, io::Error> {
    if from.len() != to.len() {
        return Err(io::Error::other("same-length test replacement"));
    }
    let Some(offset) = source.windows(from.len()).position(|window| window == from) else {
        return Err(io::Error::other("test replacement pattern"));
    };
    let mut output = source.to_vec();
    let end = offset
        .checked_add(to.len())
        .ok_or_else(|| io::Error::other("test replacement range"))?;
    output
        .get_mut(offset..end)
        .ok_or_else(|| io::Error::other("test replacement span"))?
        .copy_from_slice(to);
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

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Result<Self, io::Error> {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "seacad-point-draft-insert-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

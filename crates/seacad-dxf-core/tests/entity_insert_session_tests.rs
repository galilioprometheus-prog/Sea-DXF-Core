use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityCloneIssue, DxfEntityCloneOutcome,
    DxfEntityCommonColorBookEditIssue, DxfEntityCommonFieldPatch,
    DxfEntityCommonReferenceEditIssue, DxfEntityDeleteOutcome, DxfEntityDraft,
    DxfEntityDraftRecordIssue, DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityField,
    DxfEntityFieldSemantics, DxfEntityFieldValue, DxfEntityInsertIssue, DxfEntityInsertOutcome,
    DxfEntityInsertOwnerIssue, DxfEntityLineweight, DxfEntityPatch, DxfEntityProxyGraphicsState,
    DxfEntityTopic, DxfError, DxfHandle, DxfHandleIdentityLookup, DxfHandseedValue,
    DxfMemorySource, DxfPointDraft, DxfPointPatch, DxfPointPatchKind, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    DxfTransactionPlan, NoopDxfReadObserver,
};

const LOCATION: [DxfDouble; 3] = [
    DxfDouble::from_bits(1.25_f64.to_bits()),
    DxfDouble::from_bits((-2.5_f64).to_bits()),
    DxfDouble::from_bits(3.75_f64.to_bits()),
];
const THICKNESS: DxfDouble = DxfDouble::from_bits(2.25_f64.to_bits());
const EXTRUSION: [DxfDouble; 3] = [
    DxfDouble::from_bits(0.25_f64.to_bits()),
    DxfDouble::from_bits((-0.5_f64).to_bits()),
    DxfDouble::from_bits(1.0_f64.to_bits()),
];
const UCS_X_AXIS_ANGLE: DxfDouble = DxfDouble::from_bits(37.5_f64.to_bits());

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
            for (index, expected_handle) in [0x40, 0x41, 0x42].into_iter().enumerate() {
                let draft = point_draft(version).with_owner(handle(0x10));
                assert_eq!(draft.owner(), Some(handle(0x10)));
                let DxfEntityInsertOutcome::Applied(receipt) = session.insert(placement, draft)?
                else {
                    return Err(io::Error::other("session POINT insertion").into());
                };
                assert_eq!(receipt.handle(), handle(expected_handle));
                assert_eq!(
                    receipt.name().canonical_topic(),
                    Some(DxfEntityTopic::POINT)
                );
                assert_eq!(receipt.placement(), placement.target());
                assert_eq!(session.queued_edit_count(), index as u64 + 1);
            }
            let debug = format!("{session:?}");
            assert!(!debug.contains("Layer0"));
            assert!(!debug.contains("Model"));

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 3);
            assert_eq!(plan.transaction().patches().len(), 2);
            let output = materialize(&bytes, plan.transaction())?;
            assert_eq!(bytes, fixture(format, version)?);
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            assert!(matches!(
                post.handseed_report()
                    .primary_occurrence()
                    .map(|occurrence| occurrence.value()),
                Some(DxfHandseedValue::Parsed(value)) if value == handle(0x43)
            ));
            let identities = post.handle_identity_directory(&token())?;
            let mut previous_ordinal = None;
            for expected_handle in [0x40, 0x41, 0x42] {
                let DxfHandleIdentityLookup::Unique(identity) =
                    identities.lookup(handle(expected_handle))
                else {
                    return Err(io::Error::other("unique inserted handle").into());
                };
                if let Some(previous) = previous_ordinal {
                    assert!(previous < identity.record().ordinal());
                }
                previous_ordinal = Some(identity.record().ordinal());
            }
            let outcome = plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?;
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) = outcome
            else {
                return Err(io::Error::other("verified session POINT insertion").into());
            };
            assert_eq!(journal.receipt().edit_count(), 3);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn session_mixes_point_updates_and_inserts_across_every_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for insert_first in [false, true] {
                let bytes = fixture_with_existing_point(format, version)?;
                let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
                let document = open_document(&source, format)?;
                let view = document.view();
                let evidence = view.entity_field_evidence_directory(&token())?;
                let placement = only_placement(view)?;
                let key = existing_point_key(&evidence)?;
                let cancellation = token();
                let mut session =
                    view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
                if insert_first {
                    assert!(matches!(
                        session.insert(
                            placement,
                            point_draft(version).with_owner(handle(0x10))
                        )?,
                        DxfEntityInsertOutcome::Applied(receipt)
                            if receipt.handle() == handle(0x40)
                    ));
                }
                assert!(matches!(
                    session.update(
                        key,
                        DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(
                            UCS_X_AXIS_ANGLE
                        ))
                    )?,
                    DxfEntityEditOutcome::PointApplied(_)
                ));
                assert!(matches!(
                    session.update(
                        key,
                        DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
                            field: DxfEntityField::VISIBILITY,
                            value: DxfEntityEditValue::Int16(1),
                        })
                    )?,
                    DxfEntityEditOutcome::Applied(_)
                ));
                if !insert_first {
                    assert!(matches!(
                        session.insert(
                            placement,
                            point_draft(version).with_owner(handle(0x10))
                        )?,
                        DxfEntityInsertOutcome::Applied(receipt)
                            if receipt.handle() == handle(0x40)
                    ));
                }
                assert_eq!(session.queued_edit_count(), 3);

                let plan = session.finish_verifiable()?;
                assert_eq!(plan.edit_count(), 3);
                assert_eq!(plan.transaction().patches().len(), 4);
                let output = materialize(&bytes, plan.transaction())?;
                let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                let output_document = open_document(&output_source, format)?;
                let post = output_document.view();
                let point = post
                    .basic_geometry_semantic_directory(&token())?
                    .point_for_raw_record(key.raw_record_ordinal())?
                    .ok_or_else(|| io::Error::other("mixed updated POINT"))?;
                assert_eq!(point.ucs_x_axis_angle_value(), Some(UCS_X_AXIS_ANGLE));
                assert_eq!(
                    point.ucs_x_axis_angle().state(),
                    DxfSemanticValueState::Explicit
                );
                assert!(matches!(
                    post.handle_identity_directory(&token())?
                        .lookup(handle(0x40)),
                    DxfHandleIdentityLookup::Unique(_)
                ));
                let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                    plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
                else {
                    return Err(io::Error::other("verified mixed POINT session").into());
                };
                assert_eq!(journal.receipt().edit_count(), 3);
                assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
            }
        }
    }
    Ok(())
}

#[test]
fn session_mixes_point_delete_and_insert_in_both_orders_across_every_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for delete_first in [false, true] {
                let bytes = fixture_with_existing_point(format, version)?;
                let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
                let document = open_document(&source, format)?;
                let view = document.view();
                let evidence = view.entity_field_evidence_directory(&token())?;
                let placement = only_placement(view)?;
                let key = existing_point_key(&evidence)?;
                let cancellation = token();
                let mut session =
                    view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
                if delete_first {
                    assert!(matches!(
                        session.delete(key)?,
                        DxfEntityDeleteOutcome::Applied(_)
                            | DxfEntityDeleteOutcome::HandlelessApplied(_)
                    ));
                }
                assert!(matches!(
                    session.insert(
                        placement,
                        point_draft(version).with_owner(handle(0x10)),
                    )?,
                    DxfEntityInsertOutcome::Applied(receipt)
                        if receipt.handle() == handle(0x40)
                ));
                if !delete_first {
                    assert!(matches!(
                        session.delete(key)?,
                        DxfEntityDeleteOutcome::Applied(_)
                            | DxfEntityDeleteOutcome::HandlelessApplied(_)
                    ));
                }
                assert_eq!(session.queued_edit_count(), 2);
                let plan = session.finish_verifiable()?;
                assert_eq!(plan.edit_count(), 2);
                let output = materialize(&bytes, plan.transaction())?;
                let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                let output_document = open_document(&output_source, format)?;
                let post = output_document.view();
                let DxfHandleIdentityLookup::Unique(inserted) = post
                    .handle_identity_directory(&token())?
                    .lookup(handle(0x40))
                else {
                    return Err(io::Error::other("mixed delete/insert identity").into());
                };
                assert!(
                    post.entity_directory(&token())?
                        .entity_for_raw_ordinal(inserted.record().ordinal())
                        .is_some_and(|entity| {
                            entity.classification().topic() == Some(DxfEntityTopic::POINT)
                        })
                );
                let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                    plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
                else {
                    return Err(io::Error::other("verified mixed delete/insert").into());
                };
                assert_eq!(journal.receipt().edit_count(), 2);
                assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
            }
        }
    }
    Ok(())
}

#[test]
fn mixed_insert_before_update_target_adjusts_raw_record_ordinal_across_every_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture_with_earlier_empty_entities(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = existing_point_key(&evidence)?;
            let placements = view.entity_placement_directory(&token())?;
            let [first, second] = placements.assessments() else {
                return Err(io::Error::other("two entity placements").into());
            };
            let first = first
                .placement()
                .ok_or_else(|| io::Error::other("first ready placement"))?;
            if second.placement().is_none() {
                return Err(io::Error::other("second ready placement").into());
            }
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            assert!(matches!(
                session.insert(first, point_draft(version).with_owner(handle(0x10)))?,
                DxfEntityInsertOutcome::Applied(_)
            ));
            assert!(matches!(
                session.update(
                    key,
                    DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UCS_X_AXIS_ANGLE))
                )?,
                DxfEntityEditOutcome::PointApplied(_)
            ));
            assert!(matches!(
                session.update(
                    key,
                    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
                        field: DxfEntityField::VISIBILITY,
                        value: DxfEntityEditValue::Int16(1),
                    })
                )?,
                DxfEntityEditOutcome::Applied(_)
            ));

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 3);
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified shifted mixed session").into());
            };
            assert_eq!(journal.receipt().edit_count(), 3);
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
fn session_insert_failures_preserve_already_queued_operations() -> Result<(), Box<dyn Error>> {
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
            DxfEntityDraft::point(
                DxfPointDraft::new(b"Missing", LOCATION)
                    .with_layout(b"Model")
                    .with_lineweight(DxfEntityLineweight::BY_LAYER)
            )
            .with_owner(handle(0x10))
        )?,
        DxfEntityInsertOutcome::Unavailable(DxfEntityInsertIssue::Record(_))
    ));
    assert_eq!(session.queued_edit_count(), 1);
    let DxfEntityInsertOutcome::Applied(second) = session.insert(
        placement,
        point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10)),
    )?
    else {
        return Err(io::Error::other("second session insert").into());
    };
    assert_eq!(second.handle(), handle(0x41));
    let key = existing_point_key(&evidence)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
                field: DxfEntityField::LAYER,
                value: DxfEntityEditValue::ExactRawText(b"Layer1")
            })
        )?,
        DxfEntityEditOutcome::Applied(_)
    ));
    assert_eq!(session.queued_edit_count(), 3);

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
        DxfEntityInsertOutcome::Applied(_)
    ));
    assert_eq!(update_first.queued_edit_count(), 2);
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

    let exhausted_bytes = fixture_with_handseed_ascii(u64::MAX);
    let exhausted_source = DxfMemorySource::new(&exhausted_bytes, DxfResourceProfile::Safe)?;
    let exhausted_document = open_ascii(&exhausted_source)?;
    let exhausted_view = DxfRawDocumentView::from(&exhausted_document);
    let exhausted_evidence = exhausted_view.entity_field_evidence_directory(&token())?;
    let exhausted_cancellation = token();
    let mut exhausted_session = exhausted_view.entity_edit_session(
        &exhausted_evidence,
        DxfResourceProfile::Safe,
        &exhausted_cancellation,
    )?;
    assert!(matches!(
        exhausted_session.insert(
            only_placement(exhausted_view)?,
            point_draft(DxfAcadVersion::Ac1032).with_owner(handle(0x10))
        )?,
        DxfEntityInsertOutcome::Unavailable(DxfEntityInsertIssue::HandleExhausted {
            handseed,
            requested_count: 1
        }) if handseed == handle(u64::MAX)
    ));
    assert_eq!(exhausted_session.queued_edit_count(), 0);
    assert_copy::<DxfEntityInsertIssue>();
    assert_copy::<DxfEntityInsertOutcome>();
    Ok(())
}

#[test]
fn session_clones_canonical_point_across_every_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture_with_existing_point(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = existing_point_key(&evidence)?;
            let placement = only_placement(view)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let outcome = session.clone_entity(key, placement, handle(0x10))?;
            let DxfEntityCloneOutcome::Applied(receipt) = outcome else {
                return Err(io::Error::other(format!("canonical POINT clone: {outcome:?}")).into());
            };
            assert_eq!(receipt.handle(), handle(0x40));
            assert_eq!(receipt.placement(), placement.target());
            assert_eq!(session.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let identities = post.handle_identity_directory(&token())?;
            let DxfHandleIdentityLookup::Unique(clone) = identities.lookup(handle(0x40)) else {
                return Err(io::Error::other("unique cloned POINT identity").into());
            };
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let cloned = geometry
                .point_for_raw_record(clone.record().ordinal())?
                .ok_or_else(|| io::Error::other("cloned POINT semantics"))?;
            assert_eq!(cloned.location_value(), Some([DxfDouble::from_f64(0.0); 3]));
            assert_eq!(cloned.thickness_value(), Some(DxfDouble::from_f64(0.0)));
            assert_eq!(cloned.thickness().state(), DxfSemanticValueState::Defaulted);
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified canonical POINT clone").into());
            };
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn session_clone_rejects_unsupported_partial_and_pending_source_state() -> Result<(), Box<dyn Error>>
{
    let version = DxfAcadVersion::Ac1032;
    let base = fixture_with_existing_point(DxfRawDocumentFormat::Ascii, version)?;
    for (extra, expected_code) in [
        (b"6\nDASHED\n".as_slice(), Some(6)),
        (b"60\n1\n".as_slice(), Some(60)),
        (b"210\n1\n".as_slice(), None),
    ] {
        let mut bytes = base.clone();
        let endsec = b"0\nENDSEC\n";
        let offset = bytes
            .windows(endsec.len())
            .rposition(|window| window == endsec)
            .ok_or_else(|| io::Error::other("ENTITIES ENDSEC"))?;
        bytes.splice(offset..offset, extra.iter().copied());
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let view = DxfRawDocumentView::from(&document);
        let evidence = view.entity_field_evidence_directory(&token())?;
        let key = existing_point_key(&evidence)?;
        let cancellation = token();
        let mut session =
            view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
        let outcome = session.clone_entity(key, only_placement(view)?, handle(0x10))?;
        assert!(
            match expected_code {
                Some(group_code) => matches!(
                    outcome,
                    DxfEntityCloneOutcome::Unavailable(
                        DxfEntityCloneIssue::UnsupportedSourceGroup {
                            key: observed,
                            group_code: observed_code,
                            ..
                        }
                    ) if observed == key && observed_code == group_code
                ),
                None => matches!(
                    outcome,
                    DxfEntityCloneOutcome::Unavailable(
                        DxfEntityCloneIssue::PointFieldUnavailable {
                            key: observed,
                            kind: DxfPointPatchKind::Extrusion,
                        }
                    ) if observed == key
                ),
            },
            "unexpected clone outcome: {outcome:?}"
        );
        assert_eq!(session.queued_edit_count(), 0);
    }

    let mut invalid_scalar = base.clone();
    let point_subclass = b"100\nAcDbPoint\n";
    let offset = invalid_scalar
        .windows(point_subclass.len())
        .rposition(|window| window == point_subclass)
        .ok_or_else(|| io::Error::other("POINT subclass"))?;
    invalid_scalar.splice(offset..offset, b"60\n2\n".iter().copied());
    let invalid_source = DxfMemorySource::new(&invalid_scalar, DxfResourceProfile::Safe)?;
    let invalid_document = open_ascii(&invalid_source)?;
    let invalid_view = DxfRawDocumentView::from(&invalid_document);
    let invalid_evidence = invalid_view.entity_field_evidence_directory(&token())?;
    let invalid_key = existing_point_key(&invalid_evidence)?;
    let invalid_cancellation = token();
    let mut invalid_session = invalid_view.entity_edit_session(
        &invalid_evidence,
        DxfResourceProfile::Safe,
        &invalid_cancellation,
    )?;
    let invalid_outcome =
        invalid_session.clone_entity(invalid_key, only_placement(invalid_view)?, handle(0x10))?;
    assert!(
        matches!(
        invalid_outcome,
        DxfEntityCloneOutcome::Unavailable(DxfEntityCloneIssue::CommonFieldUnavailable {
            key,
            field: DxfEntityField::VISIBILITY,
            ..
        }) if key == invalid_key
        ),
        "unexpected invalid scalar clone outcome: {invalid_outcome:?}"
    );
    assert_eq!(invalid_session.queued_edit_count(), 0);

    let source = DxfMemorySource::new(&base, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = existing_point_key(&evidence)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UCS_X_AXIS_ANGLE))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    assert!(matches!(
        session.clone_entity(key, only_placement(view)?, handle(0x10))?,
        DxfEntityCloneOutcome::Unavailable(DxfEntityCloneIssue::SourceUpdatePending {
            key: observed
        }) if observed == key
    ));
    assert_eq!(session.queued_edit_count(), 1);
    Ok(())
}

#[test]
fn session_clone_preserves_reviewed_scalar_common_fields_across_every_dialect()
-> Result<(), Box<dyn Error>> {
    const SCALE: DxfDouble = DxfDouble::from_bits(0.5_f64.to_bits());
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture_with_existing_point_common_scalars(format, version, SCALE)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = existing_point_key(&evidence)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            assert!(matches!(
                session.clone_entity(key, only_placement(view)?, handle(0x10))?,
                DxfEntityCloneOutcome::Applied(receipt) if receipt.handle() == handle(0x40)
            ));
            let plan = session.finish_verifiable()?;
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let identities = post.handle_identity_directory(&token())?;
            let DxfHandleIdentityLookup::Unique(clone) = identities.lookup(handle(0x40)) else {
                return Err(io::Error::other("scalar-common clone identity").into());
            };
            let semantics = post.entity_field_semantic_directory(&token())?;
            let entity = semantics
                .evidence_directory()
                .entity_directory()
                .entity_for_raw_ordinal(clone.record().ordinal())
                .ok_or_else(|| io::Error::other("scalar-common clone entity"))?;
            for (field, expected) in [
                (DxfEntityField::PAPER_SPACE, DxfEntityFieldValue::Int16(1)),
                (DxfEntityField::COLOR, DxfEntityFieldValue::Int16(-7)),
                (
                    DxfEntityField::LINETYPE_SCALE,
                    DxfEntityFieldValue::Double(SCALE),
                ),
                (DxfEntityField::VISIBILITY, DxfEntityFieldValue::Int16(1)),
            ] {
                assert_explicit_scalar(&semantics, entity, field, expected)?;
            }
            if version >= DxfAcadVersion::Ac1012 {
                for (field, expected) in [
                    (
                        DxfEntityField::TRUE_COLOR,
                        DxfEntityFieldValue::Int32(0x12_34_56),
                    ),
                    (
                        DxfEntityField::TRANSPARENCY,
                        DxfEntityFieldValue::Int32(0x0200_007f),
                    ),
                    (DxfEntityField::SHADOW, DxfEntityFieldValue::Int16(3)),
                ] {
                    assert_explicit_scalar(&semantics, entity, field, expected)?;
                }
            }
            assert!(matches!(
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
                seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
            ));
        }
    }
    Ok(())
}

#[test]
fn session_clone_preserves_valid_linetype_across_every_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture_with_existing_point_linetype(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = existing_point_key(&evidence)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            assert!(matches!(
                session.clone_entity(key, only_placement(view)?, handle(0x10))?,
                DxfEntityCloneOutcome::Applied(receipt) if receipt.handle() == handle(0x40)
            ));
            let plan = session.finish_verifiable()?;
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let identities = post.handle_identity_directory(&token())?;
            let DxfHandleIdentityLookup::Unique(clone) = identities.lookup(handle(0x40)) else {
                return Err(io::Error::other("linetype clone identity").into());
            };
            assert_exact_text_field(
                post,
                clone.record().ordinal(),
                DxfEntityField::LINETYPE,
                b"DASHED",
            )?;
            assert!(matches!(
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
                seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
            ));
        }
    }
    Ok(())
}

#[test]
fn session_clone_preserves_valid_object_references_across_modern_dialects()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED
        .into_iter()
        .filter(|version| *version >= DxfAcadVersion::Ac1012)
    {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture_with_existing_point_references(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = existing_point_key(&evidence)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            assert!(matches!(
                session.clone_entity(key, only_placement(view)?, handle(0x10))?,
                DxfEntityCloneOutcome::Applied(receipt) if receipt.handle() == handle(0x40)
            ));
            let plan = session.finish_verifiable()?;
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let identities = post.handle_identity_directory(&token())?;
            let DxfHandleIdentityLookup::Unique(clone) = identities.lookup(handle(0x40)) else {
                return Err(io::Error::other("reference clone identity").into());
            };
            let semantics = post.entity_field_semantic_directory(&token())?;
            let entity = semantics
                .evidence_directory()
                .entity_directory()
                .entity_for_raw_ordinal(clone.record().ordinal())
                .ok_or_else(|| io::Error::other("reference clone entity"))?;
            for (field, target) in [
                (DxfEntityField::MATERIAL, handle(0x13)),
                (DxfEntityField::PLOT_STYLE, handle(0x14)),
            ] {
                assert_explicit_scalar(
                    &semantics,
                    entity,
                    field,
                    DxfEntityFieldValue::Handle(target),
                )?;
            }
            assert!(matches!(
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
                seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
            ));
        }
    }
    Ok(())
}

#[test]
fn session_clone_rejects_incompatible_object_reference_without_queueing()
-> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let bytes = fixture_with_existing_point_reference_values(
        DxfRawDocumentFormat::Ascii,
        version,
        b"14",
        b"14",
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = existing_point_key(&evidence)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    let outcome = session.clone_entity(key, only_placement(view)?, handle(0x10))?;
    assert!(matches!(
        outcome,
        DxfEntityCloneOutcome::Unavailable(DxfEntityCloneIssue::Insert(
            DxfEntityInsertIssue::Record(DxfEntityDraftRecordIssue::Reference(
                DxfEntityCommonReferenceEditIssue::IncompatibleTarget {
                    field: DxfEntityField::MATERIAL,
                    ..
                }
            ))
        ))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn session_clone_preserves_valid_color_book_tuple_across_modern_dialects()
-> Result<(), Box<dyn Error>> {
    const COLOR_NAME: &[u8] = b"RAL CLASSIC$RAL 1003";
    for version in DxfAcadVersion::SUPPORTED
        .into_iter()
        .filter(|version| *version >= DxfAcadVersion::Ac1012)
    {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture_with_existing_point_color_book(format, version, COLOR_NAME)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = existing_point_key(&evidence)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            assert!(matches!(
                session.clone_entity(key, only_placement(view)?, handle(0x10))?,
                DxfEntityCloneOutcome::Applied(receipt) if receipt.handle() == handle(0x40)
            ));
            let plan = session.finish_verifiable()?;
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let identities = post.handle_identity_directory(&token())?;
            let DxfHandleIdentityLookup::Unique(clone) = identities.lookup(handle(0x40)) else {
                return Err(io::Error::other("color-book clone identity").into());
            };
            let semantics = post.entity_field_semantic_directory(&token())?;
            let entity = semantics
                .evidence_directory()
                .entity_directory()
                .entity_for_raw_ordinal(clone.record().ordinal())
                .ok_or_else(|| io::Error::other("color-book clone entity"))?;
            assert_explicit_scalar(
                &semantics,
                entity,
                DxfEntityField::COLOR,
                DxfEntityFieldValue::Int16(40),
            )?;
            assert_explicit_scalar(
                &semantics,
                entity,
                DxfEntityField::TRUE_COLOR,
                DxfEntityFieldValue::Int32(16_235_019),
            )?;
            assert_exact_text_field(
                post,
                clone.record().ordinal(),
                DxfEntityField::COLOR_NAME,
                COLOR_NAME,
            )?;
            assert!(matches!(
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
                seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
            ));
        }
    }
    Ok(())
}

#[test]
fn session_clone_rejects_invalid_color_book_tuple_without_queueing() -> Result<(), Box<dyn Error>> {
    let bytes = fixture_with_existing_point_color_book(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        b"MISSING_SEPARATOR",
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = existing_point_key(&evidence)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.clone_entity(key, only_placement(view)?, handle(0x10))?,
        DxfEntityCloneOutcome::Unavailable(DxfEntityCloneIssue::Insert(
            DxfEntityInsertIssue::Record(DxfEntityDraftRecordIssue::ColorBook(
                DxfEntityCommonColorBookEditIssue::MissingSeparator { .. }
            ))
        ))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn session_clone_preserves_proxy_graphics_across_modern_dialects() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED
        .into_iter()
        .filter(|version| *version >= DxfAcadVersion::Ac1012)
    {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture_with_existing_point_proxy_graphics(format, version, 3)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = existing_point_key(&evidence)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            assert!(matches!(
                session.clone_entity(key, only_placement(view)?, handle(0x10))?,
                DxfEntityCloneOutcome::Applied(receipt) if receipt.handle() == handle(0x40)
            ));
            let plan = session.finish_verifiable()?;
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let identities = post.handle_identity_directory(&token())?;
            let DxfHandleIdentityLookup::Unique(clone) = identities.lookup(handle(0x40)) else {
                return Err(io::Error::other("proxy clone identity").into());
            };
            let entities = post.entity_directory(&token())?;
            let entity = entities
                .entity_for_raw_ordinal(clone.record().ordinal())
                .ok_or_else(|| io::Error::other("proxy clone entity"))?;
            let proxy = post.entity_proxy_graphics_directory(&token())?;
            assert!(matches!(
                proxy.entry_for_entity(entity)?.map(|entry| entry.state()),
                Some(DxfEntityProxyGraphicsState::Matched {
                    declared_bytes: 3,
                    payload_bytes: 3,
                    chunk_count: 1,
                })
            ));
            assert!(matches!(
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
                seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
            ));
        }
    }
    Ok(())
}

#[test]
fn session_clone_rejects_proxy_graphics_size_mismatch_without_queueing()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture_with_existing_point_proxy_graphics(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        4,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = existing_point_key(&evidence)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.clone_entity(key, only_placement(view)?, handle(0x10))?,
        DxfEntityCloneOutcome::Unavailable(DxfEntityCloneIssue::ProxyGraphicsUnavailable {
            key: observed,
            state: DxfEntityProxyGraphicsState::CountMismatch {
                declared_bytes: 4,
                payload_bytes: 3,
                chunk_count: 2,
            },
        }) if observed == key
    ));
    assert_eq!(session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn session_clone_preserves_every_explicit_point_payload_field() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = fixture_with_existing_point(DxfRawDocumentFormat::Ascii, version)?;
    let endsec = b"0\nENDSEC\n";
    let offset = bytes
        .windows(endsec.len())
        .rposition(|window| window == endsec)
        .ok_or_else(|| io::Error::other("ENTITIES ENDSEC"))?;
    let explicit = b"39\n2.25\n210\n0.25\n220\n-0.5\n230\n1\n50\n37.5\n";
    bytes.splice(offset..offset, explicit.iter().copied());
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = existing_point_key(&evidence)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.clone_entity(key, only_placement(view)?, handle(0x10))?,
        DxfEntityCloneOutcome::Applied(receipt) if receipt.handle() == handle(0x40)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_ascii(&output_source)?;
    let post = DxfRawDocumentView::from(&output_document);
    let identities = post.handle_identity_directory(&token())?;
    let DxfHandleIdentityLookup::Unique(clone) = identities.lookup(handle(0x40)) else {
        return Err(io::Error::other("explicit clone identity").into());
    };
    let geometry = post.basic_geometry_semantic_directory(&token())?;
    let point = geometry
        .point_for_raw_record(clone.record().ordinal())?
        .ok_or_else(|| io::Error::other("explicit clone semantics"))?;
    assert_eq!(point.thickness_value(), Some(THICKNESS));
    assert_eq!(point.extrusion_value(), Some(EXTRUSION));
    assert_eq!(point.ucs_x_axis_angle_value(), Some(UCS_X_AXIS_ANGLE));
    assert!(
        point
            .extrusion()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Explicit)
    );
    assert!(matches!(
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
    ));
    Ok(())
}

fn point_draft(version: DxfAcadVersion) -> DxfEntityDraft<'static> {
    let point = DxfPointDraft::new(b"Layer0", LOCATION)
        .with_thickness(THICKNESS)
        .with_extrusion(EXTRUSION)
        .with_ucs_x_axis_angle(UCS_X_AXIS_ANGLE);
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
        (0, "TABLE"),
        (2, "LTYPE"),
        (0, "LTYPE"),
        (5, "12"),
        (2, "DASHED"),
        (0, "ENDTAB"),
        (0, "ENDSEC"),
    ];
    if version >= DxfAcadVersion::Ac1012 {
        groups.extend([
            (0, "SECTION"),
            (2, "OBJECTS"),
            (0, "MATERIAL"),
            (5, "13"),
            (0, "ACDBPLACEHOLDER"),
            (5, "14"),
        ]);
        if version >= DxfAcadVersion::Ac1015 {
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

fn fixture_with_existing_point(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = fixture(format, version)?;
    let endsec = encoded_string_group(format, version, 0, b"ENDSEC")?;
    let offset = bytes
        .windows(endsec.len())
        .rposition(|window| window == endsec)
        .ok_or_else(|| io::Error::other("ENTITIES ENDSEC"))?;
    let point = encoded_existing_point(format, version)?;
    bytes.splice(offset..offset, point);
    Ok(bytes)
}

fn fixture_with_existing_point_common_scalars(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    scale: DxfDouble,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = fixture_with_existing_point(format, version)?;
    let marker = if version >= DxfAcadVersion::Ac1012 {
        encoded_string_group(format, version, 100, b"AcDbPoint")?
    } else {
        encoded_double_group(format, version, 10, 0.0)?
    };
    let offset = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| io::Error::other("POINT common-field boundary"))?;
    let mut common = Vec::new();
    common.extend_from_slice(&encoded_i16_group(format, version, 67, 1)?);
    common.extend_from_slice(&encoded_i16_group(format, version, 62, -7)?);
    common.extend_from_slice(&encoded_double_group(format, version, 48, scale.to_f64())?);
    common.extend_from_slice(&encoded_i16_group(format, version, 60, 1)?);
    if version >= DxfAcadVersion::Ac1012 {
        common.extend_from_slice(&encoded_i32_group(format, version, 420, 0x12_34_56)?);
        common.extend_from_slice(&encoded_i32_group(format, version, 440, 0x0200_007f)?);
        common.extend_from_slice(&encoded_i16_group(format, version, 284, 3)?);
    }
    bytes.splice(offset..offset, common);
    Ok(bytes)
}

fn fixture_with_existing_point_linetype(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = fixture_with_existing_point(format, version)?;
    let marker = if version >= DxfAcadVersion::Ac1012 {
        encoded_string_group(format, version, 100, b"AcDbPoint")?
    } else {
        encoded_double_group(format, version, 10, 0.0)?
    };
    let offset = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| io::Error::other("POINT common-field boundary"))?;
    bytes.splice(
        offset..offset,
        encoded_string_group(format, version, 6, b"DASHED")?,
    );
    Ok(bytes)
}

fn fixture_with_existing_point_references(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    fixture_with_existing_point_reference_values(format, version, b"13", b"14")
}

fn fixture_with_existing_point_reference_values(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    material: &[u8],
    plot_style: &[u8],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = fixture_with_existing_point(format, version)?;
    let marker = encoded_string_group(format, version, 100, b"AcDbPoint")?;
    let offset = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| io::Error::other("POINT common-field boundary"))?;
    let mut references = encoded_string_group(format, version, 347, material)?;
    references.extend_from_slice(&encoded_string_group(format, version, 390, plot_style)?);
    bytes.splice(offset..offset, references);
    Ok(bytes)
}

fn fixture_with_existing_point_color_book(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    color_name: &[u8],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = fixture_with_existing_point(format, version)?;
    let marker = encoded_string_group(format, version, 100, b"AcDbPoint")?;
    let offset = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| io::Error::other("POINT common-field boundary"))?;
    let mut tuple = encoded_i16_group(format, version, 62, 40)?;
    tuple.extend_from_slice(&encoded_i32_group(format, version, 420, 16_235_019)?);
    tuple.extend_from_slice(&encoded_string_group(format, version, 430, color_name)?);
    bytes.splice(offset..offset, tuple);
    Ok(bytes)
}

fn fixture_with_existing_point_proxy_graphics(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    declared_bytes: i32,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = fixture_with_existing_point(format, version)?;
    let marker = encoded_string_group(format, version, 100, b"AcDbPoint")?;
    let offset = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| io::Error::other("POINT common-field boundary"))?;
    let mut proxy = encoded_i32_group(format, version, 92, declared_bytes)?;
    proxy.extend_from_slice(&encoded_binary_chunk_group(format, version, 310, &[1, 2])?);
    proxy.extend_from_slice(&encoded_binary_chunk_group(format, version, 310, &[3])?);
    bytes.splice(offset..offset, proxy);
    Ok(bytes)
}

fn fixture_with_earlier_empty_entities(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = fixture_with_existing_point(format, version)?;
    let mut opening = encoded_string_group(format, version, 0, b"SECTION")?;
    opening.extend_from_slice(&encoded_string_group(format, version, 2, b"ENTITIES")?);
    let offset = bytes
        .windows(opening.len())
        .rposition(|window| window == opening)
        .ok_or_else(|| io::Error::other("ENTITIES opening"))?;
    let mut empty = opening;
    empty.extend_from_slice(&encoded_string_group(format, version, 0, b"ENDSEC")?);
    bytes.splice(offset..offset, empty);
    Ok(bytes)
}

fn encoded_existing_point(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = encoded_string_group(format, version, 0, b"POINT")?;
    if version >= DxfAcadVersion::Ac1012 {
        for (code, value) in [
            (5_i16, b"20".as_slice()),
            (330, b"10".as_slice()),
            (100, b"AcDbEntity".as_slice()),
        ] {
            bytes.extend_from_slice(&encoded_string_group(format, version, code, value)?);
        }
        if version >= DxfAcadVersion::Ac1015 {
            bytes.extend_from_slice(&encoded_string_group(format, version, 410, b"Model")?);
        }
        bytes.extend_from_slice(&encoded_string_group(format, version, 8, b"Layer0")?);
        if version >= DxfAcadVersion::Ac1015 {
            bytes.extend_from_slice(&encoded_i16_group(format, version, 370, -1)?);
        }
        bytes.extend_from_slice(&encoded_string_group(format, version, 100, b"AcDbPoint")?);
    } else {
        bytes.extend_from_slice(&encoded_string_group(format, version, 8, b"Layer0")?);
    }
    for (code, value) in [(10_i16, 0.0_f64), (20, 0.0), (30, 0.0)] {
        bytes.extend_from_slice(&encoded_double_group(format, version, code, value)?);
    }
    Ok(bytes)
}

fn encoded_string_group(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::new();
    match format {
        DxfRawDocumentFormat::Ascii => {
            bytes.extend_from_slice(code.to_string().as_bytes());
            bytes.push(b'\n');
            bytes.extend_from_slice(value);
            bytes.push(b'\n');
        }
        DxfRawDocumentFormat::Binary => {
            push_binary_code(&mut bytes, version, code)?;
            bytes.extend_from_slice(value);
            bytes.push(0);
        }
        _ => return Err(io::Error::other("format")),
    }
    Ok(bytes)
}

fn encoded_binary_chunk_group(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::new();
    match format {
        DxfRawDocumentFormat::Ascii => {
            bytes.extend_from_slice(code.to_string().as_bytes());
            bytes.push(b'\n');
            for byte in value {
                bytes.extend_from_slice(format!("{byte:02X}").as_bytes());
            }
            bytes.push(b'\n');
        }
        DxfRawDocumentFormat::Binary => {
            push_binary_code(&mut bytes, version, code)?;
            bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk"))?);
            bytes.extend_from_slice(value);
        }
        _ => return Err(io::Error::other("format")),
    }
    Ok(bytes)
}

fn encoded_double_group(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::new();
    match format {
        DxfRawDocumentFormat::Ascii => {
            bytes.extend_from_slice(code.to_string().as_bytes());
            bytes.push(b'\n');
            bytes.extend_from_slice(value.to_string().as_bytes());
            bytes.push(b'\n');
        }
        DxfRawDocumentFormat::Binary => {
            push_binary_code(&mut bytes, version, code)?;
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        _ => return Err(io::Error::other("format")),
    }
    Ok(bytes)
}

fn encoded_i16_group(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code: i16,
    value: i16,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::new();
    match format {
        DxfRawDocumentFormat::Ascii => {
            bytes.extend_from_slice(code.to_string().as_bytes());
            bytes.push(b'\n');
            bytes.extend_from_slice(value.to_string().as_bytes());
            bytes.push(b'\n');
        }
        DxfRawDocumentFormat::Binary => {
            push_binary_code(&mut bytes, version, code)?;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        _ => return Err(io::Error::other("format")),
    }
    Ok(bytes)
}

fn encoded_i32_group(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code: i16,
    value: i32,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::new();
    match format {
        DxfRawDocumentFormat::Ascii => {
            bytes.extend_from_slice(code.to_string().as_bytes());
            bytes.push(b'\n');
            bytes.extend_from_slice(value.to_string().as_bytes());
            bytes.push(b'\n');
        }
        DxfRawDocumentFormat::Binary => {
            push_binary_code(&mut bytes, version, code)?;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        _ => return Err(io::Error::other("format")),
    }
    Ok(bytes)
}

fn assert_explicit_scalar(
    semantics: &seacad_dxf_core::DxfEntityFieldSemanticDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
    expected: DxfEntityFieldValue,
) -> Result<(), Box<dyn Error>> {
    let entry = semantics
        .entry_for_field(entity, field)?
        .ok_or_else(|| io::Error::other("scalar semantic entry"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("scalar singleton semantics").into());
    };
    assert_eq!(value.state(), DxfSemanticValueState::Explicit);
    assert_eq!(value.value(), Some(&expected));
    Ok(())
}

fn assert_exact_text_field(
    view: DxfRawDocumentView<'_>,
    raw_record_ordinal: u64,
    field: DxfEntityField,
    expected: &[u8],
) -> Result<(), Box<dyn Error>> {
    let semantics = view.entity_field_semantic_directory(&token())?;
    let entity = semantics
        .evidence_directory()
        .entity_directory()
        .entity_for_raw_ordinal(raw_record_ordinal)
        .ok_or_else(|| io::Error::other("text-field entity"))?;
    let entry = semantics
        .entry_for_field(entity, field)?
        .ok_or_else(|| io::Error::other("text-field entry"))?;
    let DxfEntityFieldSemantics::Singleton(value) = entry.semantics() else {
        return Err(io::Error::other("text-field singleton").into());
    };
    let Some(DxfEntityFieldValue::ExactText(text)) = value.value().copied() else {
        return Err(io::Error::other("text-field exact value").into());
    };
    assert_eq!(value.state(), DxfSemanticValueState::Explicit);
    let mut observed = vec![0_u8; expected.len()];
    view.read_span(text.value_span(), &mut observed)?;
    assert_eq!(observed, expected);
    Ok(())
}

fn push_binary_code(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("AC1009 code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
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

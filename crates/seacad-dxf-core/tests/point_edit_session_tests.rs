use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBasicGeometryComponentRole,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble,
    DxfEntityCommonFieldPatch, DxfEntityDraft, DxfEntityEditDisposition, DxfEntityEditIssue,
    DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityField, DxfEntityFieldEvidenceDirectory,
    DxfEntityPatch, DxfEntityTopic, DxfError, DxfMemorySource, DxfPointDraft, DxfPointEditIssue,
    DxfPointPatch, DxfPointPatchKind, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, DxfTransactionPlan, NoopDxfReadObserver,
};

const UPDATED: [DxfDouble; 3] = [
    DxfDouble::from_bits(4.0_f64.to_bits()),
    DxfDouble::from_bits(5.0_f64.to_bits()),
    DxfDouble::from_bits(6.0_f64.to_bits()),
];
const UPDATED_THICKNESS: DxfDouble = DxfDouble::from_bits(4.25_f64.to_bits());
const UPDATED_EXTRUSION: [DxfDouble; 3] = [
    DxfDouble::from_bits(0.25_f64.to_bits()),
    DxfDouble::from_bits((-0.5_f64).to_bits()),
    DxfDouble::from_bits(2.0_f64.to_bits()),
];
const DEFAULT_EXTRUSION: [DxfDouble; 3] = [
    DxfDouble::from_bits(0.0_f64.to_bits()),
    DxfDouble::from_bits(0.0_f64.to_bits()),
    DxfDouble::from_bits(1.0_f64.to_bits()),
];
const UPDATED_ANGLE: DxfDouble = DxfDouble::from_bits(45.0_f64.to_bits());

#[test]
fn point_location_update_is_atomic_across_every_ascii_binary_dialect() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::Complete)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED)),
            )?
            else {
                return Err(io::Error::other("POINT location update").into());
            };
            assert_eq!(receipt.key(), key);
            assert_eq!(receipt.kind(), DxfPointPatchKind::Location);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Replaced);
            assert_eq!(receipt.queued_edit_count(), 1);
            assert_eq!(session.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            assert_eq!(plan.transaction().patches().len(), 3);
            let output = materialize(&bytes, plan.transaction())?;
            assert_eq!(bytes, fixture(format, version, PointShape::Complete)?);
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("updated POINT semantics"))?;
            assert_eq!(point.location_value(), Some(UPDATED));
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT update").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_location_rejects_duplicate_missing_wrong_family_and_nonfinite()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let point_key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let line_key = key_for_topic(&evidence, DxfEntityTopic::LINE)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            line_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::WrongClassification { key, .. }
        )) if key == line_key
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location([
                DxfDouble::from_f64(f64::NAN),
                UPDATED[1],
                UPDATED[2],
            ]))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(DxfPointEditIssue::Encoding {
            group_code: 10,
            ..
        }))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::Location,
                ..
            }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 1);

    for (shape, expected_role, duplicate) in [
        (
            PointShape::MissingY,
            seacad_dxf_core::DxfBasicGeometryComponentRole::WcsLocationOrStartY,
            false,
        ),
        (
            PointShape::DuplicateX,
            seacad_dxf_core::DxfBasicGeometryComponentRole::WcsLocationOrStartX,
            true,
        ),
    ] {
        let malformed = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, shape)?;
        let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
        let malformed_document = open_ascii(&malformed_source)?;
        let malformed_view = DxfRawDocumentView::from(&malformed_document);
        let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
        let key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
        let malformed_cancellation = token();
        let mut malformed_session = malformed_view.entity_edit_session(
            &malformed_evidence,
            DxfResourceProfile::Safe,
            &malformed_cancellation,
        )?;
        let outcome = malformed_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED)),
        )?;
        assert!(if duplicate {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::DuplicateLocationComponent { role, occurrence_count: 2 }
                )) if role == expected_role
            )
        } else {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::MissingLocationComponent { role }
                )) if role == expected_role
            )
        });
        assert_eq!(malformed_session.queued_edit_count(), 0);
    }
    Ok(())
}

#[test]
fn point_location_composes_with_common_update_and_blocks_insert_mixing()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let DxfEntityEditOutcome::Applied(common) = session.update(
        key,
        DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
            field: DxfEntityField::VISIBILITY,
            value: DxfEntityEditValue::Int16(1),
        }),
    )?
    else {
        return Err(io::Error::other("common update after POINT").into());
    };
    assert_eq!(common.queued_edit_count(), 2);

    let placement = view.entity_placement_directory(&token())?.assessments()[0]
        .placement()
        .ok_or_else(|| io::Error::other("placement"))?;
    assert!(matches!(
        session.insert(
            placement,
            DxfEntityDraft::point(DxfPointDraft::new(b"0", UPDATED))
        )?,
        seacad_dxf_core::DxfEntityInsertOutcome::Unavailable(
            seacad_dxf_core::DxfEntityInsertIssue::UpdatePending {
                queued_update_count: 2
            }
        )
    ));
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 2);
    assert_eq!(plan.transaction().patches().len(), 4);
    let output = materialize(&bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_ascii(&output_source)?;
    let post = DxfRawDocumentView::from(&output_document);
    assert!(matches!(
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
    ));
    Ok(())
}

#[test]
fn point_location_verifier_and_cancellation_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"10\n4\n", b"10\n7\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointLocationMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_thickness_update_is_atomic_across_every_ascii_binary_dialect() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::Complete)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS)),
            )?
            else {
                return Err(io::Error::other("POINT thickness update").into());
            };
            assert_eq!(receipt.key(), key);
            assert_eq!(receipt.kind(), DxfPointPatchKind::Thickness);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Replaced);
            assert_eq!(receipt.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            assert_eq!(plan.transaction().patches().len(), 1);
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("updated POINT semantics"))?;
            assert_eq!(point.thickness_value(), Some(UPDATED_THICKNESS));
            assert_eq!(point.thickness().state(), DxfSemanticValueState::Explicit);
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT thickness update").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_thickness_inserts_absent_group_across_every_ascii_binary_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::MissingThickness)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS)),
            )?
            else {
                return Err(io::Error::other("POINT thickness insertion").into());
            };
            assert_eq!(receipt.kind(), DxfPointPatchKind::Thickness);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Inserted);
            let plan = session.finish_verifiable()?;
            let [patch] = plan.transaction().patches() else {
                return Err(io::Error::other("one POINT thickness insertion patch").into());
            };
            assert!(patch.source_span().is_empty());
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("inserted POINT thickness semantics"))?;
            assert_eq!(point.thickness_value(), Some(UPDATED_THICKNESS));
            assert_eq!(point.thickness().state(), DxfSemanticValueState::Explicit);
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT thickness insertion").into());
            };
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_thickness_insertion_preserves_ascii_line_endings_and_requires_location_anchor()
-> Result<(), Box<dyn Error>> {
    let bytes = String::from_utf8(ascii_fixture(
        DxfAcadVersion::Ac1032,
        PointShape::MissingThickness,
    ))?
    .replace('\n', "\r\n")
    .into_bytes();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    assert!(
        output
            .windows(b"20\r\n2\r\n39\r\n4.25\r\n50\r\n30\r\n".len())
            .any(|window| window == b"20\r\n2\r\n39\r\n4.25\r\n50\r\n30\r\n")
    );

    let missing_thickness = String::from_utf8(ascii_fixture(
        DxfAcadVersion::Ac1032,
        PointShape::MissingThickness,
    ))?;
    for (malformed, expected_role, duplicate) in [
        (
            missing_thickness.replace("20\n2\n", "").into_bytes(),
            DxfBasicGeometryComponentRole::WcsLocationOrStartY,
            false,
        ),
        (
            missing_thickness
                .replace("20\n2\n", "20\n2\n10\n9\n")
                .into_bytes(),
            DxfBasicGeometryComponentRole::WcsLocationOrStartX,
            true,
        ),
    ] {
        let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
        let malformed_document = open_ascii(&malformed_source)?;
        let malformed_view = DxfRawDocumentView::from(&malformed_document);
        let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
        let malformed_key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
        let malformed_cancellation = token();
        let mut malformed_session = malformed_view.entity_edit_session(
            &malformed_evidence,
            DxfResourceProfile::Safe,
            &malformed_cancellation,
        )?;
        let outcome = malformed_session.update(
            malformed_key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS)),
        )?;
        assert!(if duplicate {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::DuplicateLocationComponent {
                        role,
                        occurrence_count: 2
                    }
                )) if role == expected_role
            )
        } else {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::MissingLocationComponent { role }
                )) if role == expected_role
            )
        });
        assert_eq!(malformed_session.queued_edit_count(), 0);
    }
    Ok(())
}

#[test]
fn point_thickness_rejects_duplicate_wrong_family_and_nonfinite() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let point_key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let line_key = key_for_topic(&evidence, DxfEntityTopic::LINE)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            line_key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::WrongClassification { key, .. }
        )) if key == line_key
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(DxfDouble::from_f64(f64::NAN)))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(DxfPointEditIssue::Encoding {
            group_code: 39,
            ..
        }))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::Thickness,
                ..
            }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 1);

    let malformed = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::DuplicateThickness,
    )?;
    let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
    let malformed_document = open_ascii(&malformed_source)?;
    let malformed_view = DxfRawDocumentView::from(&malformed_document);
    let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
    let malformed_cancellation = token();
    let mut malformed_session = malformed_view.entity_edit_session(
        &malformed_evidence,
        DxfResourceProfile::Safe,
        &malformed_cancellation,
    )?;
    let outcome = malformed_session.update(
        key,
        DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS)),
    )?;
    assert!(matches!(
        outcome,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateThickness {
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(malformed_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_location_and_thickness_compose_as_distinct_family_patches() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::MissingThickness,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
        key,
        DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS)),
    )?
    else {
        return Err(io::Error::other("POINT thickness composition").into());
    };
    assert_eq!(receipt.queued_edit_count(), 2);

    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 2);
    assert_eq!(plan.transaction().patches().len(), 4);
    assert_eq!(
        plan.transaction()
            .patches()
            .iter()
            .filter(|patch| patch.source_span().is_empty())
            .count(),
        1
    );
    let output = materialize(&bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_ascii(&output_source)?;
    let post = DxfRawDocumentView::from(&output_document);
    assert!(matches!(
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
    ));
    Ok(())
}

#[test]
fn point_thickness_verifier_and_cancellation_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"39\n4.25\n", b"39\n7.25\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointThicknessMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS))
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_thickness_reset_is_atomic_across_every_ascii_binary_dialect() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::Complete)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) =
                session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_thickness()))?
            else {
                return Err(io::Error::other("POINT thickness reset").into());
            };
            assert_eq!(receipt.key(), key);
            assert_eq!(receipt.kind(), DxfPointPatchKind::Thickness);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Reset);
            assert_eq!(receipt.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            let [patch] = plan.transaction().patches() else {
                return Err(io::Error::other("one POINT thickness reset patch").into());
            };
            assert!(!patch.source_span().is_empty());
            assert_eq!(
                plan.transaction()
                    .replacement_bytes_for_patch_ordinal(patch.ordinal()),
                Some(&[][..])
            );
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("reset POINT thickness semantics"))?;
            assert_eq!(point.thickness_value(), Some(DxfDouble::from_f64(0.0)));
            assert_eq!(point.thickness().state(), DxfSemanticValueState::Defaulted);
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT thickness reset").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_thickness_reset_absent_is_noop_and_does_not_reserve_patch_kind()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::MissingThickness,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    let DxfEntityEditOutcome::PointApplied(reset) =
        session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_thickness()))?
    else {
        return Err(io::Error::other("implicit POINT thickness reset").into());
    };
    assert_eq!(
        reset.disposition(),
        DxfEntityEditDisposition::AlreadyImplicit
    );
    assert_eq!(reset.queued_edit_count(), 0);
    assert_eq!(session.queued_edit_count(), 0);

    let DxfEntityEditOutcome::PointApplied(set) = session.update(
        key,
        DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS)),
    )?
    else {
        return Err(io::Error::other("set after no-op reset").into());
    };
    assert_eq!(set.disposition(), DxfEntityEditDisposition::Inserted);
    assert_eq!(set.queued_edit_count(), 1);

    let noop_cancellation = token();
    let mut noop_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &noop_cancellation)?;
    assert!(matches!(
        noop_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::reset_thickness())
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::AlreadyImplicit
    ));
    let noop_plan = noop_session.finish_verifiable()?;
    assert_eq!(noop_plan.edit_count(), 0);
    assert!(noop_plan.transaction().patches().is_empty());
    assert_eq!(materialize(&bytes, noop_plan.transaction())?, bytes);
    Ok(())
}

#[test]
fn point_thickness_reset_rejects_duplicate_card_and_conflicting_patch() -> Result<(), Box<dyn Error>>
{
    let malformed = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::DuplicateThickness,
    )?;
    let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
    let malformed_document = open_ascii(&malformed_source)?;
    let malformed_view = DxfRawDocumentView::from(&malformed_document);
    let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
    let malformed_key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
    let malformed_cancellation = token();
    let mut malformed_session = malformed_view.entity_edit_session(
        &malformed_evidence,
        DxfResourceProfile::Safe,
        &malformed_cancellation,
    )?;
    assert!(matches!(
        malformed_session.update(
            malformed_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_thickness())
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateThickness {
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(malformed_session.queued_edit_count(), 0);

    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_thickness()))?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_thickness(UPDATED_THICKNESS))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::Thickness,
                ..
            }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 1);
    Ok(())
}

#[test]
fn point_thickness_reset_verifier_and_cancellation_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_thickness()))?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"50\n30\n", b"39\n00\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointThicknessMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_thickness())),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_extrusion_update_is_atomic_across_every_ascii_binary_dialect() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::ExplicitExtrusion)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION)),
            )?
            else {
                return Err(io::Error::other("POINT extrusion update").into());
            };
            assert_eq!(receipt.key(), key);
            assert_eq!(receipt.kind(), DxfPointPatchKind::Extrusion);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Replaced);
            assert_eq!(receipt.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            assert_eq!(plan.transaction().patches().len(), 3);
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("updated POINT extrusion semantics"))?;
            assert_eq!(point.extrusion_value(), Some(UPDATED_EXTRUSION));
            assert!(
                point
                    .extrusion()
                    .iter()
                    .all(|value| value.state() == DxfSemanticValueState::Explicit)
            );
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT extrusion update").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_extrusion_inserts_defaulted_tuple_across_every_ascii_binary_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::Complete)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION)),
            )?
            else {
                return Err(io::Error::other("POINT extrusion insertion").into());
            };
            assert_eq!(receipt.kind(), DxfPointPatchKind::Extrusion);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Inserted);
            assert_eq!(receipt.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            let [patch] = plan.transaction().patches() else {
                return Err(io::Error::other("one POINT extrusion insertion patch").into());
            };
            assert!(patch.source_span().is_empty());
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("inserted POINT extrusion semantics"))?;
            assert_eq!(point.extrusion_value(), Some(UPDATED_EXTRUSION));
            assert!(
                point
                    .extrusion()
                    .iter()
                    .all(|value| value.state() == DxfSemanticValueState::Explicit)
            );
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT extrusion insertion").into());
            };
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_extrusion_completes_every_partial_tuple_across_all_dialects() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for mask in 1_u8..0b111 {
                let bytes = fixture(format, version, PointShape::PartialExtrusion(mask))?;
                let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
                let document = open_document(&source, format)?;
                let view = document.view();
                let evidence = view.entity_field_evidence_directory(&token())?;
                let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
                let cancellation = token();
                let mut session =
                    view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
                let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                    key,
                    DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION)),
                )?
                else {
                    return Err(io::Error::other("partial POINT extrusion update").into());
                };
                assert_eq!(receipt.kind(), DxfPointPatchKind::Extrusion);
                assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Composite);
                assert_eq!(receipt.queued_edit_count(), 1);

                let plan = session.finish_verifiable()?;
                let expected_patch_count = if matches!(mask, 0b001 | 0b100) { 2 } else { 3 };
                assert_eq!(plan.edit_count(), 1);
                assert_eq!(plan.transaction().patches().len(), expected_patch_count);
                let expected_insertion_count = if mask == 0b010 { 2 } else { 1 };
                assert_eq!(
                    plan.transaction()
                        .patches()
                        .iter()
                        .filter(|patch| patch.source_span().is_empty())
                        .count(),
                    expected_insertion_count
                );
                let output = materialize(&bytes, plan.transaction())?;
                let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                let output_document = open_document(&output_source, format)?;
                let post = output_document.view();
                let geometry = post.basic_geometry_semantic_directory(&token())?;
                let point = geometry
                    .point_for_raw_record(key.raw_record_ordinal())?
                    .ok_or_else(|| io::Error::other("completed POINT extrusion semantics"))?;
                assert_eq!(point.extrusion_value(), Some(UPDATED_EXTRUSION));
                assert!(
                    point
                        .extrusion()
                        .iter()
                        .all(|value| value.state() == DxfSemanticValueState::Explicit)
                );
                let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                    plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
                else {
                    return Err(io::Error::other("verified partial POINT extrusion").into());
                };
                assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
            }
        }
    }
    Ok(())
}

#[test]
fn point_extrusion_reset_is_atomic_for_every_explicit_mask_across_all_dialects()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for mask in 1_u8..=0b111 {
                let shape = if mask == 0b111 {
                    PointShape::ExplicitExtrusion
                } else {
                    PointShape::PartialExtrusion(mask)
                };
                let bytes = fixture(format, version, shape)?;
                let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
                let document = open_document(&source, format)?;
                let view = document.view();
                let evidence = view.entity_field_evidence_directory(&token())?;
                let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
                let cancellation = token();
                let mut session =
                    view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
                let DxfEntityEditOutcome::PointApplied(receipt) =
                    session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_extrusion()))?
                else {
                    return Err(io::Error::other("POINT extrusion reset").into());
                };
                assert_eq!(receipt.key(), key);
                assert_eq!(receipt.kind(), DxfPointPatchKind::Extrusion);
                assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Reset);
                assert_eq!(receipt.queued_edit_count(), 1);

                let plan = session.finish_verifiable()?;
                assert_eq!(plan.edit_count(), 1);
                assert_eq!(
                    plan.transaction().patches().len(),
                    mask.count_ones() as usize
                );
                for patch in plan.transaction().patches() {
                    assert!(!patch.source_span().is_empty());
                    assert_eq!(
                        plan.transaction()
                            .replacement_bytes_for_patch_ordinal(patch.ordinal()),
                        Some(&[][..])
                    );
                }
                let output = materialize(&bytes, plan.transaction())?;
                let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                let output_document = open_document(&output_source, format)?;
                let post = output_document.view();
                let geometry = post.basic_geometry_semantic_directory(&token())?;
                let point = geometry
                    .point_for_raw_record(key.raw_record_ordinal())?
                    .ok_or_else(|| io::Error::other("reset POINT extrusion semantics"))?;
                assert_eq!(point.extrusion_value(), Some(DEFAULT_EXTRUSION));
                assert!(
                    point
                        .extrusion()
                        .iter()
                        .all(|value| value.state() == DxfSemanticValueState::Defaulted)
                );
                let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                    plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
                else {
                    return Err(io::Error::other("verified POINT extrusion reset").into());
                };
                assert_eq!(journal.receipt().edit_count(), 1);
                assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
            }
        }
    }
    Ok(())
}

#[test]
fn point_extrusion_reset_absent_is_noop_and_does_not_reserve_patch_kind()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    let DxfEntityEditOutcome::PointApplied(reset) =
        session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_extrusion()))?
    else {
        return Err(io::Error::other("implicit POINT extrusion reset").into());
    };
    assert_eq!(
        reset.disposition(),
        DxfEntityEditDisposition::AlreadyImplicit
    );
    assert_eq!(reset.queued_edit_count(), 0);
    assert_eq!(session.queued_edit_count(), 0);

    let DxfEntityEditOutcome::PointApplied(set) = session.update(
        key,
        DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION)),
    )?
    else {
        return Err(io::Error::other("set after no-op extrusion reset").into());
    };
    assert_eq!(set.disposition(), DxfEntityEditDisposition::Inserted);
    assert_eq!(set.queued_edit_count(), 1);

    let noop_cancellation = token();
    let mut noop_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &noop_cancellation)?;
    assert!(matches!(
        noop_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::reset_extrusion())
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::AlreadyImplicit
    ));
    let noop_plan = noop_session.finish_verifiable()?;
    assert_eq!(noop_plan.edit_count(), 0);
    assert!(noop_plan.transaction().patches().is_empty());
    assert_eq!(materialize(&bytes, noop_plan.transaction())?, bytes);
    Ok(())
}

#[test]
fn point_extrusion_reset_rejects_wrong_duplicate_and_conflicting_requests()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::ExplicitExtrusion,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let point_key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let line_key = key_for_topic(&evidence, DxfEntityTopic::LINE)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            line_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_extrusion())
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::WrongClassification { key, .. }
        )) if key == line_key
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_extrusion())
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::Reset
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::Extrusion,
                ..
            }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 1);

    let malformed = String::from_utf8(bytes)?
        .replace("210\n1\n", "210\n1\n210\n4\n")
        .into_bytes();
    let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
    let malformed_document = open_ascii(&malformed_source)?;
    let malformed_view = DxfRawDocumentView::from(&malformed_document);
    let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
    let malformed_key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
    let malformed_cancellation = token();
    let mut malformed_session = malformed_view.entity_edit_session(
        &malformed_evidence,
        DxfResourceProfile::Safe,
        &malformed_cancellation,
    )?;
    assert!(matches!(
        malformed_session.update(
            malformed_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_extrusion())
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateExtrusionComponent {
                role: DxfBasicGeometryComponentRole::ExtrusionX,
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(malformed_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_extrusion_reset_verifier_and_cancellation_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::ExplicitExtrusion,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_extrusion()))?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"50\n30\n", b"210\n0\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointExtrusionMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(key, DxfEntityPatch::Point(DxfPointPatch::reset_extrusion())),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_replaces_explicit_group_across_every_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::Complete)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE)),
            )?
            else {
                return Err(io::Error::other("POINT angle replacement").into());
            };
            assert_eq!(receipt.key(), key);
            assert_eq!(receipt.kind(), DxfPointPatchKind::UcsXAxisAngle);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Replaced);
            assert_eq!(receipt.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            let [patch] = plan.transaction().patches() else {
                return Err(io::Error::other("one POINT angle replacement patch").into());
            };
            assert!(!patch.source_span().is_empty());
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("updated POINT angle semantics"))?;
            assert_eq!(point.ucs_x_axis_angle_value(), Some(UPDATED_ANGLE));
            assert_eq!(
                point.ucs_x_axis_angle().state(),
                DxfSemanticValueState::Explicit
            );
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT angle replacement").into());
            };
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_inserts_after_every_extrusion_mask_across_every_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for mask in 0_u8..=0b111 {
                let bytes = fixture(format, version, PointShape::MissingAngleWithExtrusion(mask))?;
                let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
                let document = open_document(&source, format)?;
                let view = document.view();
                let evidence = view.entity_field_evidence_directory(&token())?;
                let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
                let cancellation = token();
                let mut session =
                    view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
                let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                    key,
                    DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE)),
                )?
                else {
                    return Err(io::Error::other("POINT angle insertion").into());
                };
                assert_eq!(receipt.kind(), DxfPointPatchKind::UcsXAxisAngle);
                assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Inserted);
                assert_eq!(receipt.queued_edit_count(), 1);

                let plan = session.finish_verifiable()?;
                assert_eq!(plan.edit_count(), 1);
                let [patch] = plan.transaction().patches() else {
                    return Err(io::Error::other("one POINT angle insertion patch").into());
                };
                assert!(patch.source_span().is_empty());
                let output = materialize(&bytes, plan.transaction())?;
                let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                let output_document = open_document(&output_source, format)?;
                let post = output_document.view();
                let geometry = post.basic_geometry_semantic_directory(&token())?;
                let point = geometry
                    .point_for_raw_record(key.raw_record_ordinal())?
                    .ok_or_else(|| io::Error::other("inserted POINT angle semantics"))?;
                assert_eq!(point.ucs_x_axis_angle_value(), Some(UPDATED_ANGLE));
                assert_eq!(
                    point.ucs_x_axis_angle().state(),
                    DxfSemanticValueState::Explicit
                );
                let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                    plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
                else {
                    return Err(io::Error::other("verified POINT angle insertion").into());
                };
                assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
            }
        }
    }
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_preserves_crlf_and_rejects_ambiguous_evidence()
-> Result<(), Box<dyn Error>> {
    for shape in [
        PointShape::Complete,
        PointShape::MissingAngleWithExtrusion(0b101),
    ] {
        let bytes = String::from_utf8(ascii_fixture(DxfAcadVersion::Ac1032, shape))?
            .replace('\n', "\r\n")
            .into_bytes();
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let view = DxfRawDocumentView::from(&document);
        let evidence = view.entity_field_evidence_directory(&token())?;
        let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
        let cancellation = token();
        let mut session =
            view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
        assert!(matches!(
            session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
            )?,
            DxfEntityEditOutcome::PointApplied(_)
        ));
        let plan = session.finish_verifiable()?;
        let output = materialize(&bytes, plan.transaction())?;
        assert!(
            output
                .windows(b"50\r\n45\r\n0\r\nLINE\r\n".len())
                .any(|window| window == b"50\r\n45\r\n0\r\nLINE\r\n")
        );
    }

    let duplicate_angle =
        String::from_utf8(ascii_fixture(DxfAcadVersion::Ac1032, PointShape::Complete))?
            .replace("50\n30\n", "50\n30\n50\n31\n")
            .into_bytes();
    let duplicate_source = DxfMemorySource::new(&duplicate_angle, DxfResourceProfile::Safe)?;
    let duplicate_document = open_ascii(&duplicate_source)?;
    let duplicate_view = DxfRawDocumentView::from(&duplicate_document);
    let duplicate_evidence = duplicate_view.entity_field_evidence_directory(&token())?;
    let duplicate_key = key_for_topic(&duplicate_evidence, DxfEntityTopic::POINT)?;
    let duplicate_cancellation = token();
    let mut duplicate_session = duplicate_view.entity_edit_session(
        &duplicate_evidence,
        DxfResourceProfile::Safe,
        &duplicate_cancellation,
    )?;
    assert!(matches!(
        duplicate_session.update(
            duplicate_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateUcsXAxisAngle {
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(duplicate_session.queued_edit_count(), 0);

    let duplicate_extrusion = String::from_utf8(ascii_fixture(
        DxfAcadVersion::Ac1032,
        PointShape::MissingAngleWithExtrusion(0b001),
    ))?
    .replace("210\n1\n", "210\n1\n210\n4\n")
    .into_bytes();
    let duplicate_source = DxfMemorySource::new(&duplicate_extrusion, DxfResourceProfile::Safe)?;
    let duplicate_document = open_ascii(&duplicate_source)?;
    let duplicate_view = DxfRawDocumentView::from(&duplicate_document);
    let duplicate_evidence = duplicate_view.entity_field_evidence_directory(&token())?;
    let duplicate_key = key_for_topic(&duplicate_evidence, DxfEntityTopic::POINT)?;
    let duplicate_cancellation = token();
    let mut duplicate_session = duplicate_view.entity_edit_session(
        &duplicate_evidence,
        DxfResourceProfile::Safe,
        &duplicate_cancellation,
    )?;
    assert!(matches!(
        duplicate_session.update(
            duplicate_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateExtrusionComponent {
                role: DxfBasicGeometryComponentRole::ExtrusionX,
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(duplicate_session.queued_edit_count(), 0);

    let duplicate_thickness = String::from_utf8(ascii_fixture(
        DxfAcadVersion::Ac1032,
        PointShape::DuplicateThickness,
    ))?
    .replace("50\n30\n", "")
    .into_bytes();
    let duplicate_source = DxfMemorySource::new(&duplicate_thickness, DxfResourceProfile::Safe)?;
    let duplicate_document = open_ascii(&duplicate_source)?;
    let duplicate_view = DxfRawDocumentView::from(&duplicate_document);
    let duplicate_evidence = duplicate_view.entity_field_evidence_directory(&token())?;
    let duplicate_key = key_for_topic(&duplicate_evidence, DxfEntityTopic::POINT)?;
    let duplicate_cancellation = token();
    let mut duplicate_session = duplicate_view.entity_edit_session(
        &duplicate_evidence,
        DxfResourceProfile::Safe,
        &duplicate_cancellation,
    )?;
    assert!(matches!(
        duplicate_session.update(
            duplicate_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateThickness {
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(duplicate_session.queued_edit_count(), 0);

    let missing_location = String::from_utf8(ascii_fixture(
        DxfAcadVersion::Ac1032,
        PointShape::MissingThickness,
    ))?
    .replace("20\n2\n", "")
    .replace("50\n30\n", "")
    .into_bytes();
    let missing_source = DxfMemorySource::new(&missing_location, DxfResourceProfile::Safe)?;
    let missing_document = open_ascii(&missing_source)?;
    let missing_view = DxfRawDocumentView::from(&missing_document);
    let missing_evidence = missing_view.entity_field_evidence_directory(&token())?;
    let missing_key = key_for_topic(&missing_evidence, DxfEntityTopic::POINT)?;
    let missing_cancellation = token();
    let mut missing_session = missing_view.entity_edit_session(
        &missing_evidence,
        DxfResourceProfile::Safe,
        &missing_cancellation,
    )?;
    assert!(matches!(
        missing_session.update(
            missing_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::MissingLocationComponent {
                role: DxfBasicGeometryComponentRole::WcsLocationOrStartY
            }
        ))
    ));
    assert_eq!(missing_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_composes_with_other_point_fields() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::ExplicitExtrusion,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    for patch in [
        DxfPointPatch::set_location(UPDATED),
        DxfPointPatch::set_thickness(UPDATED_THICKNESS),
        DxfPointPatch::set_extrusion(UPDATED_EXTRUSION),
        DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE),
    ] {
        assert!(matches!(
            session.update(key, DxfEntityPatch::Point(patch))?,
            DxfEntityEditOutcome::PointApplied(_)
        ));
    }
    assert_eq!(session.queued_edit_count(), 4);
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 4);
    assert_eq!(plan.transaction().patches().len(), 8);
    let output = materialize(&bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_ascii(&output_source)?;
    let post = DxfRawDocumentView::from(&output_document);
    let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
    else {
        return Err(io::Error::other("verified composed POINT angle edit").into());
    };
    assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_rejects_invalid_requests_and_verifies_exactly()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let point_key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let line_key = key_for_topic(&evidence, DxfEntityTopic::LINE)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            line_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::WrongClassification { key, .. }
        )) if key == line_key
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(DxfDouble::from_f64(
                f64::NAN
            )))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(DxfPointEditIssue::Encoding {
            group_code: 50,
            ..
        }))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::UcsXAxisAngle,
                ..
            }
        ))
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"50\n45\n", b"50\n46\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointUcsXAxisAngleMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == point_key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_reset_is_atomic_across_every_ascii_binary_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::Complete)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle()),
            )?
            else {
                return Err(io::Error::other("POINT UCS X-axis angle reset").into());
            };
            assert_eq!(receipt.key(), key);
            assert_eq!(receipt.kind(), DxfPointPatchKind::UcsXAxisAngle);
            assert_eq!(receipt.disposition(), DxfEntityEditDisposition::Reset);
            assert_eq!(receipt.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            let [patch] = plan.transaction().patches() else {
                return Err(io::Error::other("one POINT angle reset patch").into());
            };
            assert!(!patch.source_span().is_empty());
            assert_eq!(
                plan.transaction()
                    .replacement_bytes_for_patch_ordinal(patch.ordinal()),
                Some(&[][..])
            );
            let output = materialize(&bytes, plan.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("reset POINT angle semantics"))?;
            assert_eq!(
                point.ucs_x_axis_angle_value(),
                Some(DxfDouble::from_f64(0.0))
            );
            assert_eq!(
                point.ucs_x_axis_angle().state(),
                DxfSemanticValueState::Defaulted
            );
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT angle reset").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_reset_absent_is_noop_and_does_not_reserve_patch_kind()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::MissingAngleWithExtrusion(0b101),
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    let DxfEntityEditOutcome::PointApplied(reset) = session.update(
        key,
        DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle()),
    )?
    else {
        return Err(io::Error::other("implicit POINT angle reset").into());
    };
    assert_eq!(
        reset.disposition(),
        DxfEntityEditDisposition::AlreadyImplicit
    );
    assert_eq!(reset.queued_edit_count(), 0);
    assert_eq!(session.queued_edit_count(), 0);

    let DxfEntityEditOutcome::PointApplied(set) = session.update(
        key,
        DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE)),
    )?
    else {
        return Err(io::Error::other("set after no-op angle reset").into());
    };
    assert_eq!(set.disposition(), DxfEntityEditDisposition::Inserted);
    assert_eq!(set.queued_edit_count(), 1);

    let noop_cancellation = token();
    let mut noop_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &noop_cancellation)?;
    assert!(matches!(
        noop_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle())
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::AlreadyImplicit
    ));
    let noop_plan = noop_session.finish_verifiable()?;
    assert_eq!(noop_plan.edit_count(), 0);
    assert!(noop_plan.transaction().patches().is_empty());
    assert_eq!(materialize(&bytes, noop_plan.transaction())?, bytes);
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_reset_rejects_ambiguous_wrong_and_conflicting_requests()
-> Result<(), Box<dyn Error>> {
    let malformed = String::from_utf8(ascii_fixture(DxfAcadVersion::Ac1032, PointShape::Complete))?
        .replace("50\n30\n", "50\n30\n50\n31\n")
        .into_bytes();
    let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
    let malformed_document = open_ascii(&malformed_source)?;
    let malformed_view = DxfRawDocumentView::from(&malformed_document);
    let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
    let malformed_key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
    let malformed_cancellation = token();
    let mut malformed_session = malformed_view.entity_edit_session(
        &malformed_evidence,
        DxfResourceProfile::Safe,
        &malformed_cancellation,
    )?;
    assert!(matches!(
        malformed_session.update(
            malformed_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle())
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateUcsXAxisAngle {
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(malformed_session.queued_edit_count(), 0);

    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let point_key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let line_key = key_for_topic(&evidence, DxfEntityTopic::LINE)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            line_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle())
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::WrongClassification { key, .. }
        )) if key == line_key
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle())
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::Reset
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_ucs_x_axis_angle(UPDATED_ANGLE))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::UcsXAxisAngle,
                ..
            }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 1);
    Ok(())
}

#[test]
fn point_ucs_x_axis_angle_reset_verifier_cancellation_and_crlf_fail_closed()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle())
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"0\nLINE\n", b"50\n0.0\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointUcsXAxisAngleMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle())
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);

    let crlf = String::from_utf8(bytes)?.replace('\n', "\r\n").into_bytes();
    let crlf_source = DxfMemorySource::new(&crlf, DxfResourceProfile::Safe)?;
    let crlf_document = open_ascii(&crlf_source)?;
    let crlf_view = DxfRawDocumentView::from(&crlf_document);
    let crlf_evidence = crlf_view.entity_field_evidence_directory(&token())?;
    let crlf_key = key_for_topic(&crlf_evidence, DxfEntityTopic::POINT)?;
    let crlf_cancellation = token();
    let mut crlf_session = crlf_view.entity_edit_session(
        &crlf_evidence,
        DxfResourceProfile::Safe,
        &crlf_cancellation,
    )?;
    assert!(matches!(
        crlf_session.update(
            crlf_key,
            DxfEntityPatch::Point(DxfPointPatch::reset_ucs_x_axis_angle())
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let crlf_plan = crlf_session.finish_verifiable()?;
    let crlf_output = materialize(&crlf, crlf_plan.transaction())?;
    assert!(
        !crlf_output
            .windows(b"50\r\n30\r\n".len())
            .any(|window| window == b"50\r\n30\r\n")
    );
    let crlf_output_source = DxfMemorySource::new(&crlf_output, DxfResourceProfile::Safe)?;
    let crlf_output_document = open_ascii(&crlf_output_source)?;
    let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(crlf_journal) = crlf_plan
        .verify_post_image(
            crlf_view,
            DxfRawDocumentView::from(&crlf_output_document),
            DxfResourceProfile::Safe,
            &token(),
        )?
    else {
        return Err(io::Error::other("verified CRLF angle reset").into());
    };
    assert_eq!(
        materialize(&crlf_output, crlf_journal.inverse_plan())?,
        crlf
    );
    Ok(())
}

#[test]
fn point_extrusion_partial_completion_preserves_crlf_and_rejects_source_reordering()
-> Result<(), Box<dyn Error>> {
    let bytes = String::from_utf8(ascii_fixture(
        DxfAcadVersion::Ac1032,
        PointShape::PartialExtrusion(0b010),
    ))?
    .replace('\n', "\r\n")
    .into_bytes();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::Composite
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    assert!(
        output
            .windows(b"210\r\n0.25\r\n220\r\n-0.5\r\n230\r\n2\r\n50\r\n30\r\n".len())
            .any(|window| { window == b"210\r\n0.25\r\n220\r\n-0.5\r\n230\r\n2\r\n50\r\n30\r\n" }),
        "{}",
        String::from_utf8_lossy(&output)
    );

    let canonical = String::from_utf8(ascii_fixture(
        DxfAcadVersion::Ac1032,
        PointShape::PartialExtrusion(0b101),
    ))?;
    let reordered = canonical
        .replace("210\n1\n230\n3\n", "230\n3\n210\n1\n")
        .into_bytes();
    let reordered_source = DxfMemorySource::new(&reordered, DxfResourceProfile::Safe)?;
    let reordered_document = open_ascii(&reordered_source)?;
    let reordered_view = DxfRawDocumentView::from(&reordered_document);
    let reordered_evidence = reordered_view.entity_field_evidence_directory(&token())?;
    let reordered_key = key_for_topic(&reordered_evidence, DxfEntityTopic::POINT)?;
    let reordered_cancellation = token();
    let mut reordered_session = reordered_view.entity_edit_session(
        &reordered_evidence,
        DxfResourceProfile::Safe,
        &reordered_cancellation,
    )?;
    assert!(matches!(
        reordered_session.update(
            reordered_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::NonCanonicalExtrusionOrder {
                preceding: DxfBasicGeometryComponentRole::ExtrusionX,
                following: DxfBasicGeometryComponentRole::ExtrusionZ,
            }
        ))
    ));
    assert_eq!(reordered_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_extrusion_insertion_preserves_crlf_and_requires_an_unambiguous_predecessor()
-> Result<(), Box<dyn Error>> {
    let bytes = String::from_utf8(ascii_fixture(DxfAcadVersion::Ac1032, PointShape::Complete))?
        .replace('\n', "\r\n")
        .into_bytes();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::Inserted
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    assert!(
        output
            .windows(b"39\r\n2.5\r\n210\r\n0.25\r\n220\r\n-0.5\r\n230\r\n2\r\n50\r\n30\r\n".len())
            .any(|window| {
                window == b"39\r\n2.5\r\n210\r\n0.25\r\n220\r\n-0.5\r\n230\r\n2\r\n50\r\n30\r\n"
            })
    );

    let without_thickness = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::MissingThickness,
    )?;
    let without_source = DxfMemorySource::new(&without_thickness, DxfResourceProfile::Safe)?;
    let without_document = open_ascii(&without_source)?;
    let without_view = DxfRawDocumentView::from(&without_document);
    let without_evidence = without_view.entity_field_evidence_directory(&token())?;
    let without_key = key_for_topic(&without_evidence, DxfEntityTopic::POINT)?;
    let without_cancellation = token();
    let mut without_session = without_view.entity_edit_session(
        &without_evidence,
        DxfResourceProfile::Safe,
        &without_cancellation,
    )?;
    assert!(matches!(
        without_session.update(
            without_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::PointApplied(receipt)
            if receipt.disposition() == DxfEntityEditDisposition::Inserted
    ));
    let without_plan = without_session.finish()?;
    let without_output = materialize(&without_thickness, &without_plan)?;
    assert!(
        without_output
            .windows(b"20\n2\n210\n0.25\n220\n-0.5\n230\n2\n50\n30\n".len())
            .any(|window| window == b"20\n2\n210\n0.25\n220\n-0.5\n230\n2\n50\n30\n")
    );

    let duplicate_thickness = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::DuplicateThickness,
    )?;
    let duplicate_source = DxfMemorySource::new(&duplicate_thickness, DxfResourceProfile::Safe)?;
    let duplicate_document = open_ascii(&duplicate_source)?;
    let duplicate_view = DxfRawDocumentView::from(&duplicate_document);
    let duplicate_evidence = duplicate_view.entity_field_evidence_directory(&token())?;
    let duplicate_key = key_for_topic(&duplicate_evidence, DxfEntityTopic::POINT)?;
    let duplicate_cancellation = token();
    let mut duplicate_session = duplicate_view.entity_edit_session(
        &duplicate_evidence,
        DxfResourceProfile::Safe,
        &duplicate_cancellation,
    )?;
    assert!(matches!(
        duplicate_session.update(
            duplicate_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateThickness {
                occurrence_count: 2
            }
        ))
    ));

    let missing_thickness = String::from_utf8(without_thickness)?;
    for (malformed, expected_role, duplicate) in [
        (
            missing_thickness.replace("20\n2\n", "").into_bytes(),
            DxfBasicGeometryComponentRole::WcsLocationOrStartY,
            false,
        ),
        (
            missing_thickness
                .replace("10\n1\n", "10\n1\n10\n4\n")
                .into_bytes(),
            DxfBasicGeometryComponentRole::WcsLocationOrStartX,
            true,
        ),
    ] {
        let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
        let malformed_document = open_ascii(&malformed_source)?;
        let malformed_view = DxfRawDocumentView::from(&malformed_document);
        let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
        let malformed_key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
        let malformed_cancellation = token();
        let mut malformed_session = malformed_view.entity_edit_session(
            &malformed_evidence,
            DxfResourceProfile::Safe,
            &malformed_cancellation,
        )?;
        let outcome = malformed_session.update(
            malformed_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION)),
        )?;
        assert!(if duplicate {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::DuplicateLocationComponent {
                        role,
                        occurrence_count: 2
                    }
                )) if role == expected_role
            )
        } else {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::MissingLocationComponent { role }
                )) if role == expected_role
            )
        });
        assert_eq!(malformed_session.queued_edit_count(), 0);
    }
    Ok(())
}

#[test]
fn point_extrusion_rejects_wrong_zero_nonfinite_and_duplicate() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::ExplicitExtrusion,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let point_key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let line_key = key_for_topic(&evidence, DxfEntityTopic::LINE)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            line_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::WrongClassification { key, .. }
        )) if key == line_key
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion([
                DxfDouble::from_f64(0.0),
                DxfDouble::from_f64(-0.0),
                DxfDouble::from_f64(0.0),
            ]))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::ZeroExtrusion
        ))
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion([
                DxfDouble::from_f64(f64::NAN),
                UPDATED_EXTRUSION[1],
                UPDATED_EXTRUSION[2],
            ]))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(DxfPointEditIssue::Encoding {
            group_code: 210,
            ..
        }))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::Extrusion,
                ..
            }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 1);

    let explicit = String::from_utf8(bytes)?;
    let malformed = explicit
        .replace("210\n1\n", "210\n1\n210\n4\n")
        .into_bytes();
    let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
    let malformed_document = open_ascii(&malformed_source)?;
    let malformed_view = DxfRawDocumentView::from(&malformed_document);
    let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
    let malformed_cancellation = token();
    let mut malformed_session = malformed_view.entity_edit_session(
        &malformed_evidence,
        DxfResourceProfile::Safe,
        &malformed_cancellation,
    )?;
    let outcome = malformed_session.update(
        key,
        DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION)),
    )?;
    assert!(matches!(
        outcome,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicateExtrusionComponent {
                role: DxfBasicGeometryComponentRole::ExtrusionX,
                occurrence_count: 2
            }
        ))
    ));
    assert_eq!(malformed_session.queued_edit_count(), 0);
    Ok(())
}

#[test]
fn point_extrusion_composes_with_location_and_thickness() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::ExplicitExtrusion,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    for patch in [
        DxfPointPatch::set_location(UPDATED),
        DxfPointPatch::set_thickness(UPDATED_THICKNESS),
        DxfPointPatch::set_extrusion(UPDATED_EXTRUSION),
    ] {
        assert!(matches!(
            session.update(key, DxfEntityPatch::Point(patch))?,
            DxfEntityEditOutcome::PointApplied(_)
        ));
    }
    assert_eq!(session.queued_edit_count(), 3);
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 3);
    assert_eq!(plan.transaction().patches().len(), 7);
    let output = materialize(&bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_ascii(&output_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&output_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
    ));
    Ok(())
}

#[test]
fn point_extrusion_verifier_and_cancellation_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::ExplicitExtrusion,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"210\n0.25\n", b"210\n0.35\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointExtrusionMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_extrusion(UPDATED_EXTRUSION))
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    Ok(())
}

#[derive(Clone, Copy)]
enum PointShape {
    Complete,
    MissingY,
    DuplicateX,
    MissingThickness,
    DuplicateThickness,
    ExplicitExtrusion,
    PartialExtrusion(u8),
    MissingAngleWithExtrusion(u8),
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    shape: PointShape,
) -> Result<Vec<u8>, io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_fixture(version, shape)),
        DxfRawDocumentFormat::Binary => binary_fixture(version, shape),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_fixture(version: DxfAcadVersion, shape: PointShape) -> Vec<u8> {
    let point = match shape {
        PointShape::Complete
        | PointShape::MissingThickness
        | PointShape::DuplicateThickness
        | PointShape::ExplicitExtrusion
        | PointShape::PartialExtrusion(_)
        | PointShape::MissingAngleWithExtrusion(_) => "30\n3\n10\n1\n20\n2\n",
        PointShape::MissingY => "30\n3\n10\n1\n",
        PointShape::DuplicateX => "30\n3\n10\n1\n20\n2\n10\n9\n",
    };
    let thickness = match shape {
        PointShape::MissingThickness => "",
        PointShape::DuplicateThickness => "39\n2.5\n39\n3.5\n",
        _ => "39\n2.5\n",
    };
    let extrusion = match shape {
        PointShape::ExplicitExtrusion => "210\n1\n220\n2\n230\n3\n".to_owned(),
        PointShape::PartialExtrusion(mask) | PointShape::MissingAngleWithExtrusion(mask) => {
            let mut extrusion = String::new();
            for (bit, group) in [
                (0b001, "210\n1\n"),
                (0b010, "220\n2\n"),
                (0b100, "230\n3\n"),
            ] {
                if mask & bit != 0 {
                    extrusion.push_str(group);
                }
            }
            extrusion
        }
        _ => String::new(),
    };
    let angle = if matches!(shape, PointShape::MissingAngleWithExtrusion(_)) {
        ""
    } else {
        "50\n30\n"
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOINT\n60\n0\n{}{}{}{}0\nLINE\n10\n7\n20\n8\n30\n9\n11\n10\n21\n11\n31\n12\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        point,
        thickness,
        extrusion,
        angle,
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, shape: PointShape) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"POINT")?;
    push_i16(&mut bytes, version, 60, 0)?;
    for (code, value) in [(30, 3.0), (10, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    if !matches!(shape, PointShape::MissingY) {
        push_double(&mut bytes, version, 20, 2.0)?;
    }
    if matches!(shape, PointShape::DuplicateX) {
        push_double(&mut bytes, version, 10, 9.0)?;
    }
    if !matches!(shape, PointShape::MissingThickness) {
        push_double(&mut bytes, version, 39, 2.5)?;
    }
    if matches!(shape, PointShape::DuplicateThickness) {
        push_double(&mut bytes, version, 39, 3.5)?;
    }
    if matches!(
        shape,
        PointShape::ExplicitExtrusion
            | PointShape::PartialExtrusion(_)
            | PointShape::MissingAngleWithExtrusion(_)
    ) {
        let mask = match shape {
            PointShape::PartialExtrusion(mask) | PointShape::MissingAngleWithExtrusion(mask) => {
                mask
            }
            _ => 0b111,
        };
        for (bit, code, value) in [(0b001, 210, 1.0), (0b010, 220, 2.0), (0b100, 230, 3.0)] {
            if mask & bit == 0 {
                continue;
            }
            push_double(&mut bytes, version, code, value)?;
        }
    }
    if !matches!(shape, PointShape::MissingAngleWithExtrusion(_)) {
        push_double(&mut bytes, version, 50, 30.0)?;
    }
    push_string(&mut bytes, version, 0, b"LINE")?;
    for (code, value) in [
        (10, 7.0),
        (20, 8.0),
        (30, 9.0),
        (11, 10.0),
        (21, 11.0),
        (31, 12.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn key_for_topic(
    evidence: &DxfEntityFieldEvidenceDirectory,
    topic: DxfEntityTopic,
) -> Result<seacad_dxf_core::DxfEntityKey, io::Error> {
    evidence
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .find(|entity| entity.classification().topic() == Some(topic))
        .map(|entity| entity.key())
        .ok_or_else(|| io::Error::other("entity key"))
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn materialize(source: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, DxfError> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start = usize::try_from(patch.source_span().start()).map_err(|_| test_error())?;
        let end = usize::try_from(patch.source_span().end()).map_err(|_| test_error())?;
        output.extend_from_slice(source.get(cursor..start).ok_or_else(test_error)?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or_else(test_error)?,
        );
        cursor = end;
    }
    output.extend_from_slice(source.get(cursor..).ok_or_else(test_error)?);
    Ok(output)
}

fn replace_once(source: &[u8], needle: &[u8], replacement: &[u8]) -> Result<Vec<u8>, io::Error> {
    if needle.len() != replacement.len() {
        return Err(io::Error::other("same-length replacement"));
    }
    let matches: Vec<_> = source
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == needle).then_some(offset))
        .collect();
    let [offset] = matches.as_slice() else {
        return Err(io::Error::other("one replacement target"));
    };
    let mut output = source.to_vec();
    output[*offset..*offset + replacement.len()].copy_from_slice(replacement);
    Ok(output)
}

fn open_document<'a>(
    source: &'a dyn DxfByteSource,
    format: DxfRawDocumentFormat,
) -> Result<OpenedDocument<'a>, DxfError> {
    Ok(match format {
        DxfRawDocumentFormat::Ascii => OpenedDocument::Ascii(open_ascii(source)?),
        DxfRawDocumentFormat::Binary => OpenedDocument::Binary(open_binary(source)?),
        _ => return Err(test_error()),
    })
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

fn test_error() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::other("test data"),
    )
}

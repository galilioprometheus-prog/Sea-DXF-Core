use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityCloneIssue, DxfEntityDraft,
    DxfEntityDraftApplicabilityIssue, DxfEntityDraftApplicabilityPlan, DxfEntityDraftIdentityIssue,
    DxfEntityDraftName, DxfEntityDraftRecordPlan, DxfEntityLineweight,
    DxfEntityPlacementOwnerBinding, DxfEntityPlacementOwnerOutcome, DxfEntityTopic,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataDraftInsertPlan,
    DxfEntityXDataDraftRecordIssue, DxfEntityXDataDraftRecordPlan,
    DxfEntityXDataDraftVerificationJournal, DxfEntityXDataDraftVerificationOutcome,
    DxfEntityXDataDraftVerificationReceipt, DxfEntityXDataDraftWriteJournal,
    DxfEntityXDataDraftWriteOutcome, DxfEntityXDataEncodedEntityDestinationDirectory,
    DxfEntityXDataEncodedEntityDestinationEntry, DxfEntityXDataEncodedEntityDestinationState,
    DxfError, DxfHandleReservationPlan, DxfHandleReservationPlanOutcome, DxfMemorySource,
    DxfPointCloneDestinationBindings, DxfPointCloneDraftProjectionIssue,
    DxfPointCloneSourceEvidence, DxfPointCloneXDataDraftIssue, DxfPointCloneXDataDraftPlan,
    DxfPointCloneXDataInsertPlan, DxfPointCloneXDataSource, DxfPointCloneXDataVerificationJournal,
    DxfPointCloneXDataVerificationOutcome, DxfPointCloneXDataWriteJournal,
    DxfPointCloneXDataWriteOutcome, DxfPointDraft, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadControl, DxfReadObserver, DxfReadOptions, DxfReadProgress, DxfResourceProfile,
    DxfTransactionPlan, NoopDxfReadObserver,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

const LOCATION: [DxfDouble; 3] = [
    DxfDouble::from_bits(1.0_f64.to_bits()),
    DxfDouble::from_bits(2.0_f64.to_bits()),
    DxfDouble::from_bits(3.0_f64.to_bits()),
];

#[test]
fn insert_plan_retains_xdata_expectation_and_atomic_transaction() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataDraftInsertPlan>();
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x40)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;
    let payload = directory
        .encoded_bytes_for_entry(entry)
        .ok_or(io::Error::other("payload"))?;
    let composed = planned(directory.compose_entity_draft_record(
        entry,
        draft_record(destination.view(), version)?,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    let insert = destination.view().plan_entity_xdata_draft_insert(
        composed,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(insert.source_id(), source.view().source_id());
    assert_eq!(insert.destination_id(), destination.view().source_id());
    assert_eq!(insert.source_entity(), directory.entity_for_entry(entry)?);
    assert_eq!(insert.encoded_entry(), entry);
    assert_eq!(insert.encoded_state(), entry.state());
    assert_eq!(insert.expected_xdata_bytes(), payload);
    let transaction = insert.edit_plan().transaction();
    assert_eq!(transaction.patches().len(), 2);
    let insertion = transaction
        .patches()
        .last()
        .ok_or(io::Error::other("insertion patch"))?;
    let replacement = transaction
        .replacement_bytes_for_patch_ordinal(insertion.ordinal())
        .ok_or(io::Error::other("insertion bytes"))?;
    assert!(replacement.ends_with(payload));
    let debug = format!("{insert:?}");
    assert!(!debug.contains("SECRET_DRAFT_XDATA"));
    assert_eq!(insert.into_edit_plan().transaction().patches().len(), 2);
    Ok(())
}

#[test]
fn insert_planning_is_cancellable_and_destination_bound() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x40)?;
    let other_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x50)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let other_storage = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let other = open_document(&other_storage, DxfRawDocumentFormat::Ascii)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        destination.view().plan_entity_xdata_draft_insert(
            planned(directory.compose_entity_draft_record(
                entry,
                draft_record(destination.view(), version)?,
                DxfResourceProfile::Safe,
                &token(),
            )?)?,
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        other.view().plan_entity_xdata_draft_insert(
            planned(directory.compose_entity_draft_record(
                entry,
                draft_record(destination.view(), version)?,
                DxfResourceProfile::Safe,
                &token(),
            )?)?,
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn post_image_verification_is_strict_cancellable_bound_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataDraftVerificationJournal>();
    assert_copy::<DxfEntityXDataDraftVerificationReceipt>();
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x40)?;
    let other_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x50)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let other_storage = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let other = open_document(&other_storage, DxfRawDocumentFormat::Ascii)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;
    let composed = planned(directory.compose_entity_draft_record(
        entry,
        draft_record(destination.view(), version)?,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    let insert = destination.view().plan_entity_xdata_draft_insert(
        composed,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    let output = materialize(&destination_bytes, insert.edit_plan().transaction())?;
    let output_storage = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let post = open_document(&output_storage, DxfRawDocumentFormat::Ascii)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        insert.verify_post_image(
            destination.view(),
            post.view(),
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        insert.verify_post_image(
            other.view(),
            post.view(),
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let tampered = replace_once(&output, b"SECRET_DRAFT_XDATA", b"SECRET_DRAFT_XDATB")?;
    let tampered_storage = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_post = open_document(&tampered_storage, DxfRawDocumentFormat::Ascii)?;
    assert!(
        insert
            .verify_post_image(
                destination.view(),
                tampered_post.view(),
                DxfResourceProfile::Safe,
                &token(),
            )
            .is_err()
    );
    let debug = format!("{insert:?}");
    assert!(!debug.contains("SECRET_DRAFT_XDATA"));
    Ok(())
}

#[test]
fn write_pipeline_rejects_existing_cancelled_and_tampered_destinations()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataDraftWriteJournal>();
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x40)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;
    let insert = build_insert(&directory, entry, destination.view(), version)?;
    let temporary = TestDirectory::new()?;

    let existing = temporary.path().join("existing.dxf");
    fs::write(&existing, b"KEEP")?;
    let mut observer = NoopDxfReadObserver;
    assert!(
        insert
            .write_reparse_verify_and_journal_to_new_file(
                destination.view(),
                &existing,
                DxfResourceProfile::Safe,
                &token(),
                &mut observer,
            )
            .is_err()
    );
    assert_eq!(fs::read(&existing)?, b"KEEP");

    let cancelled_path = temporary.path().join("cancelled.dxf");
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        insert.write_reparse_verify_and_journal_to_new_file(
            destination.view(),
            &cancelled_path,
            DxfResourceProfile::Safe,
            &cancelled,
            &mut observer,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!cancelled_path.exists());

    let tampered_path = temporary.path().join("tampered.dxf");
    let mut tamper =
        FinalTamperObserver::new(&tampered_path, b"SECRET_DRAFT_XDATA", b"SECRET_DRAFT_XDATB");
    assert!(
        insert
            .write_reparse_verify_and_journal_to_new_file(
                destination.view(),
                &tampered_path,
                DxfResourceProfile::Safe,
                &token(),
                &mut tamper,
            )
            .is_err()
    );
    tamper.finish()?;
    assert!(!tampered_path.exists());
    Ok(())
}

#[test]
fn ready_xdata_appends_to_drafts_for_every_format_and_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                assert_format_pair(source_format, version, destination_format, version, 0x40)?;
            }
        }
    }
    assert_format_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        0x40,
    )?;
    assert_format_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        0x40,
    )?;
    Ok(())
}

#[test]
fn point_clone_projects_and_composes_owned_xdata_for_every_format_and_dialect()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfPointCloneXDataDraftPlan>();
    assert_send_sync::<DxfPointCloneXDataInsertPlan>();
    assert_send_sync::<DxfPointCloneXDataVerificationJournal>();
    assert_send_sync::<DxfPointCloneXDataWriteJournal>();
    assert_copy::<DxfPointCloneSourceEvidence>();
    assert_copy::<DxfPointCloneXDataDraftIssue>();
    assert_copy::<DxfPointCloneXDataSource<'static>>();
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                assert_point_clone_format_pair(
                    source_format,
                    version,
                    destination_format,
                    version,
                )?;
            }
        }
    }
    assert_point_clone_format_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
    )?;
    Ok(())
}

#[test]
fn point_clone_xdata_composition_is_fail_closed_and_keeps_standalone_projection_strict()
-> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x40)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;
    let entity = directory.entity_for_entry(entry)?;
    let bindings = DxfPointCloneDestinationBindings::new(b"Layer0").with_layout(b"Model");

    assert!(matches!(
        destination.view().project_point_clone_draft_from(
            source.view(),
            entity.key(),
            admitted_plan(destination.view())?,
            bindings,
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfPointCloneDraftProjectionIssue::Source(
            DxfEntityCloneIssue::UnsupportedSourceGroup {
                group_code: 1001,
                ..
            }
        ))
    ));

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        destination.view().project_point_clone_xdata_draft_from(
            source.view(),
            DxfPointCloneXDataSource::new(&directory, entry)?,
            admitted_plan(destination.view())?,
            bindings,
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));

    let other_source_bytes = replace_once(&source_bytes, b"5\n30\n", b"5\n31\n")?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_document(&other_source_storage, DxfRawDocumentFormat::Ascii)?;
    assert!(matches!(
        destination.view().project_point_clone_xdata_draft_from(
            other_source.view(),
            DxfPointCloneXDataSource::new(&directory, entry)?,
            admitted_plan(destination.view())?,
            bindings,
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled_insert = token();
    cancelled_insert.cancel();
    assert!(matches!(
        destination.view().plan_point_clone_xdata_insert(
            point_clone_xdata_draft(
                &directory,
                entry,
                source.view(),
                destination.view(),
                version,
            )?,
            DxfResourceProfile::Safe,
            &cancelled_insert,
        ),
        Err(DxfError::Cancelled)
    ));

    let other_destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x50)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_document(&other_destination_storage, DxfRawDocumentFormat::Ascii)?;
    assert!(matches!(
        other_destination.view().plan_point_clone_xdata_insert(
            point_clone_xdata_draft(
                &directory,
                entry,
                source.view(),
                destination.view(),
                version,
            )?,
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let insert = destination.view().plan_point_clone_xdata_insert(
        point_clone_xdata_draft(
            &directory,
            entry,
            source.view(),
            destination.view(),
            version,
        )?,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    let temporary = TestDirectory::new()?;
    let existing = temporary.path().join("existing.dxf");
    fs::write(&existing, b"KEEP")?;
    let mut observer = NoopDxfReadObserver;
    assert!(
        insert
            .write_reparse_verify_and_journal_to_new_file(
                destination.view(),
                &existing,
                DxfResourceProfile::Safe,
                &token(),
                &mut observer,
            )
            .is_err()
    );
    assert_eq!(fs::read(&existing)?, b"KEEP");
    let cancelled_path = temporary.path().join("cancelled.dxf");
    let cancelled_write = token();
    cancelled_write.cancel();
    assert!(matches!(
        insert.write_reparse_verify_and_journal_to_new_file(
            destination.view(),
            &cancelled_path,
            DxfResourceProfile::Safe,
            &cancelled_write,
            &mut observer,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!cancelled_path.exists());
    let tampered_path = temporary.path().join("point-clone-tampered.dxf");
    let mut tamper =
        FinalTamperObserver::new(&tampered_path, b"SECRET_DRAFT_XDATA", b"SECRET_DRAFT_XDATB");
    assert!(
        insert
            .write_reparse_verify_and_journal_to_new_file(
                destination.view(),
                &tampered_path,
                DxfResourceProfile::Safe,
                &token(),
                &mut tamper,
            )
            .is_err()
    );
    tamper.finish()?;
    assert!(!tampered_path.exists());

    let orphan_bytes = replace_once(&source_bytes, b"1001\nAPP_READY\n", b"1000\nAPP_READY\n")?;
    let orphan_storage = DxfMemorySource::new(&orphan_bytes, DxfResourceProfile::Safe)?;
    let orphan = open_document(&orphan_storage, DxfRawDocumentFormat::Ascii)?;
    let orphan_directory = build_directory(orphan.view(), destination.view())?;
    let orphan_entity = point_entity(orphan.view())?;
    let orphan_entry = orphan_directory
        .entry_for_entity(orphan_entity)?
        .ok_or(io::Error::other("orphan POINT entry"))?;
    assert!(matches!(
        destination.view().project_point_clone_xdata_draft_from(
            orphan.view(),
            DxfPointCloneXDataSource::new(&orphan_directory, orphan_entry)?,
            admitted_plan(destination.view())?,
            bindings,
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfPointCloneXDataDraftIssue::XData(
            DxfEntityXDataDraftRecordIssue::EncodedPayloadUnavailable { state }
        )) if state == orphan_entry.state()
    ));
    Ok(())
}

#[test]
fn zero_xdata_is_an_exact_noop_and_unavailable_payloads_fail_closed() -> Result<(), Box<dyn Error>>
{
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Binary, version, 0x40)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Binary)?;
    let directory = build_directory(source.view(), destination.view())?;

    let zero = find_entry(&directory, |state| {
        matches!(
            state,
            DxfEntityXDataEncodedEntityDestinationState::Ready {
                application_count: 0,
                member_count: 0,
                encoded_byte_count: 0,
            }
        )
    })?;
    let record = draft_record(destination.view(), version)?;
    let original = record.bytes().to_vec();
    let plan = planned(directory.compose_entity_draft_record(
        zero,
        record,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    assert_eq!(plan.bytes(), original);
    let insert = destination.view().plan_entity_xdata_draft_insert(
        plan,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(insert.expected_xdata_bytes(), &[]);
    let output = materialize(&destination_bytes, insert.edit_plan().transaction())?;
    let output_storage = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let post = open_document(&output_storage, DxfRawDocumentFormat::Binary)?;
    let DxfEntityXDataDraftVerificationOutcome::Verified(journal) = insert.verify_post_image(
        destination.view(),
        post.view(),
        DxfResourceProfile::Safe,
        &token(),
    )?
    else {
        return Err(io::Error::other("zero-XDATA verification").into());
    };
    assert_eq!(journal.receipt().application_count(), 0);
    assert_eq!(journal.receipt().xdata_byte_count(), 0);
    assert_eq!(
        materialize(&output, journal.inverse_plan())?,
        destination_bytes
    );
    let temporary = TestDirectory::new()?;
    let written_path = temporary.path().join("zero-xdata.dxf");
    let mut observer = NoopDxfReadObserver;
    let DxfEntityXDataDraftWriteOutcome::Written(written) = insert
        .write_reparse_verify_and_journal_to_new_file(
            destination.view(),
            &written_path,
            DxfResourceProfile::Safe,
            &token(),
            &mut observer,
        )?
    else {
        return Err(io::Error::other("zero-XDATA write verification").into());
    };
    assert_eq!(written.verification_receipt().xdata_byte_count(), 0);
    assert_eq!(fs::read(&written_path)?, output);

    let unavailable = find_entry(&directory, |state| {
        matches!(
            state,
            DxfEntityXDataEncodedEntityDestinationState::Unavailable {
                application_count: 0,
                member_count: 0,
                ..
            }
        )
    })?;
    let record = draft_record(destination.view(), version)?;
    assert!(matches!(
        directory.compose_entity_draft_record(
            unavailable,
            record,
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfEntityXDataDraftRecordIssue::EncodedPayloadUnavailable { state })
            if state == unavailable.state()
    ));
    Ok(())
}

#[test]
fn draft_composition_is_cancellable_dual_source_bound_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataDraftRecordPlan>();
    assert_copy::<DxfEntityXDataDraftRecordIssue>();
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x40)?;
    let other_destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x50)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let other_storage = DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let other_destination = open_document(&other_storage, DxfRawDocumentFormat::Ascii)?;
    let directory = build_directory(source.view(), destination.view())?;
    let other_directory = build_directory(source.view(), other_destination.view())?;
    let ready = ready_entry(&directory)?;
    let foreign = ready_entry(&other_directory)?;

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        directory.compose_entity_draft_record(
            ready,
            draft_record(destination.view(), version)?,
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(
        directory
            .compose_entity_draft_record(
                foreign,
                draft_record(destination.view(), version)?,
                DxfResourceProfile::Safe,
                &token(),
            )
            .is_err()
    );
    assert!(matches!(
        directory.compose_entity_draft_record(
            ready,
            draft_record(other_destination.view(), version)?,
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let record = draft_record(destination.view(), version)?;
    let base_len = record.bytes().len();
    let plan = planned(directory.compose_entity_draft_record(
        ready,
        record,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    let payload = directory
        .encoded_bytes_for_entry(ready)
        .ok_or(io::Error::other("ready payload"))?;
    assert_eq!(plan.source_id(), source.view().source_id());
    assert_eq!(plan.destination_id(), destination.view().source_id());
    assert_eq!(plan.encoded_entry(), ready);
    assert_eq!(plan.encoded_state(), ready.state());
    assert_eq!(plan.source_entity(), directory.entity_for_entry(ready)?);
    assert_eq!(plan.bytes().get(base_len..), Some(payload));
    let debug = format!("{plan:?}");
    assert!(!debug.contains("SECRET_DRAFT_XDATA"));
    assert!(!debug.contains("SECRET_ORPHAN_XDATA"));
    assert_eq!(
        plan.into_draft_record().bytes().get(base_len..),
        Some(payload)
    );
    Ok(())
}

fn assert_format_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
    handseed: u64,
) -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(source_format, source_version)?;
    let destination_bytes = destination_fixture(destination_format, destination_version, handseed)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, source_format)?;
    let destination = open_document(&destination_storage, destination_format)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;
    let payload = directory
        .encoded_bytes_for_entry(entry)
        .ok_or(io::Error::other("encoded payload"))?
        .to_vec();
    let record = draft_record(destination.view(), destination_version)?;
    let base = record.bytes().to_vec();
    let plan = planned(directory.compose_entity_draft_record(
        entry,
        record,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    assert_eq!(plan.bytes().get(..base.len()), Some(base.as_slice()));
    assert_eq!(plan.bytes().get(base.len()..), Some(payload.as_slice()));
    let insert = destination.view().plan_entity_xdata_draft_insert(
        plan,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(insert.expected_xdata_bytes(), payload);
    let output = materialize(&destination_bytes, insert.edit_plan().transaction())?;
    let output_storage = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let post = open_document(&output_storage, destination_format)?;
    let DxfEntityXDataDraftVerificationOutcome::Verified(journal) = insert.verify_post_image(
        destination.view(),
        post.view(),
        DxfResourceProfile::Safe,
        &token(),
    )?
    else {
        return Err(io::Error::other("XDATA verification").into());
    };
    let receipt = journal.receipt();
    assert_eq!(receipt.source_id(), source.view().source_id());
    assert_eq!(receipt.destination_id(), destination.view().source_id());
    assert_eq!(receipt.post_image_id(), post.view().source_id());
    assert_eq!(receipt.destination_handle(), insert.destination_handle());
    assert_eq!(receipt.application_count(), 1);
    assert_eq!(receipt.xdata_byte_count(), payload.len() as u64);
    assert_eq!(
        materialize(&output, journal.inverse_plan())?,
        destination_bytes
    );
    let temporary = TestDirectory::new()?;
    let written_path = temporary.path().join("xdata-draft.dxf");
    let mut observer = NoopDxfReadObserver;
    let DxfEntityXDataDraftWriteOutcome::Written(written) = insert
        .write_reparse_verify_and_journal_to_new_file(
            destination.view(),
            &written_path,
            DxfResourceProfile::Safe,
            &token(),
            &mut observer,
        )?
    else {
        return Err(io::Error::other("XDATA write verification").into());
    };
    assert_eq!(fs::read(&written_path)?, output);
    assert_eq!(written.write_receipt().output_id(), receipt.post_image_id());
    assert_eq!(
        written.verification_receipt().xdata_byte_count(),
        payload.len() as u64
    );
    assert_eq!(
        materialize(&output, written.inverse_plan())?,
        destination_bytes
    );
    Ok(())
}

fn assert_point_clone_format_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(source_format, source_version)?;
    let destination_bytes = destination_fixture(destination_format, destination_version, 0x40)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, source_format)?;
    let destination = open_document(&destination_storage, destination_format)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;
    let source_entity = directory.entity_for_entry(entry)?;
    let payload = directory
        .encoded_bytes_for_entry(entry)
        .ok_or(io::Error::other("POINT XDATA payload"))?
        .to_vec();
    let plan = point_clone_xdata_draft(
        &directory,
        entry,
        source.view(),
        destination.view(),
        destination_version,
    )?;
    assert_eq!(plan.source_id(), source.view().source_id());
    assert_eq!(plan.source_key(), source_entity.key());
    assert_eq!(plan.source_entity(), source_entity);
    assert_eq!(plan.source_version(), source_version);
    assert_eq!(plan.destination_id(), destination.view().source_id());
    assert_eq!(plan.encoded_entry(), entry);
    assert_eq!(plan.encoded_state(), entry.state());
    assert!(plan.bytes().ends_with(&payload));
    assert_eq!(plan.xdata_draft_record().source_entity(), source_entity);
    let debug = format!("{plan:?}");
    assert!(!debug.contains("SECRET_DRAFT_XDATA"));
    assert!(!debug.contains("Layer0"));
    assert!(!debug.contains("Model"));

    let source_evidence = plan.source_key();
    let insert = destination.view().plan_point_clone_xdata_insert(
        plan,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(
        insert.source_evidence().source_id(),
        source.view().source_id()
    );
    assert_eq!(insert.source_evidence().source_key(), source_evidence);
    assert_eq!(insert.source_evidence().source_version(), source_version);
    assert_eq!(insert.xdata_insert_plan().source_entity(), source_entity);
    assert_eq!(insert.expected_xdata_bytes(), payload);
    let output = materialize(&destination_bytes, insert.edit_plan().transaction())?;
    let output_storage = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let post = open_document(&output_storage, destination_format)?;
    let DxfPointCloneXDataVerificationOutcome::Verified(journal) = insert.verify_post_image(
        destination.view(),
        post.view(),
        DxfResourceProfile::Safe,
        &token(),
    )?
    else {
        return Err(io::Error::other("POINT XDATA projection verification").into());
    };
    assert_eq!(journal.source_evidence(), insert.source_evidence());
    assert_eq!(journal.receipt().source_id(), source.view().source_id());
    assert_eq!(journal.receipt().application_count(), 1);
    assert_eq!(journal.receipt().xdata_byte_count(), payload.len() as u64);
    assert_eq!(
        materialize(&output, journal.inverse_plan())?,
        destination_bytes
    );
    let temporary = TestDirectory::new()?;
    let written_path = temporary.path().join("point-clone-xdata.dxf");
    let mut observer = NoopDxfReadObserver;
    let DxfPointCloneXDataWriteOutcome::Written(written) = insert
        .write_reparse_verify_and_journal_to_new_file(
            destination.view(),
            &written_path,
            DxfResourceProfile::Safe,
            &token(),
            &mut observer,
        )?
    else {
        return Err(io::Error::other("POINT XDATA projection write").into());
    };
    assert_eq!(written.source_evidence(), insert.source_evidence());
    assert_eq!(fs::read(&written_path)?, output);
    assert_eq!(
        written.xdata_journal().verification_receipt().source_id(),
        source.view().source_id()
    );
    assert_eq!(
        materialize(&output, written.inverse_plan())?,
        destination_bytes
    );
    Ok(())
}

fn point_entity(view: DxfRawDocumentView<'_>) -> Result<seacad_dxf_core::DxfEntityRef, io::Error> {
    let directory = view
        .entity_directory(&token())
        .map_err(|error| io::Error::other(error.to_string()))?;
    directory
        .entities()
        .iter()
        .copied()
        .find(|entity| entity.classification().topic() == Some(DxfEntityTopic::POINT))
        .ok_or_else(|| io::Error::other("POINT entity"))
}

fn point_clone_xdata_draft(
    directory: &DxfEntityXDataEncodedEntityDestinationDirectory,
    entry: DxfEntityXDataEncodedEntityDestinationEntry,
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    destination_version: DxfAcadVersion,
) -> Result<DxfPointCloneXDataDraftPlan, Box<dyn Error>> {
    let mut bindings = DxfPointCloneDestinationBindings::new(b"Layer0");
    if destination_version >= DxfAcadVersion::Ac1015 {
        bindings = bindings.with_layout(b"Model");
    }
    match destination.project_point_clone_xdata_draft_from(
        source,
        DxfPointCloneXDataSource::new(directory, entry)?,
        admitted_plan(destination)?,
        bindings,
        DxfResourceProfile::Safe,
        &token(),
    )? {
        Ok(plan) => Ok(plan),
        Err(issue) => Err(io::Error::other(format!("POINT XDATA draft: {issue:?}")).into()),
    }
}

fn build_directory(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataEncodedEntityDestinationDirectory, Box<dyn Error>> {
    Ok(source.entity_xdata_encoded_entity_destination_directory(
        destination,
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?)
}

fn build_insert(
    directory: &DxfEntityXDataEncodedEntityDestinationDirectory,
    entry: DxfEntityXDataEncodedEntityDestinationEntry,
    destination: DxfRawDocumentView<'_>,
    version: DxfAcadVersion,
) -> Result<DxfEntityXDataDraftInsertPlan, Box<dyn Error>> {
    let composed = planned(directory.compose_entity_draft_record(
        entry,
        draft_record(destination, version)?,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    Ok(destination.plan_entity_xdata_draft_insert(composed, DxfResourceProfile::Safe, &token())?)
}

fn ready_entry(
    directory: &DxfEntityXDataEncodedEntityDestinationDirectory,
) -> Result<DxfEntityXDataEncodedEntityDestinationEntry, io::Error> {
    find_entry(directory, |state| {
        matches!(
            state,
            DxfEntityXDataEncodedEntityDestinationState::Ready {
                application_count: 1,
                member_count: 2,
                ..
            }
        )
    })
}

fn find_entry(
    directory: &DxfEntityXDataEncodedEntityDestinationDirectory,
    predicate: impl Fn(DxfEntityXDataEncodedEntityDestinationState) -> bool,
) -> Result<DxfEntityXDataEncodedEntityDestinationEntry, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .find(|entry| predicate(entry.state()))
        .ok_or_else(|| io::Error::other("encoded entity entry"))
}

fn planned(
    outcome: Result<DxfEntityXDataDraftRecordPlan, DxfEntityXDataDraftRecordIssue>,
) -> Result<DxfEntityXDataDraftRecordPlan, io::Error> {
    outcome.map_err(|issue| io::Error::other(format!("{issue:?}")))
}

fn draft_record(
    view: DxfRawDocumentView<'_>,
    version: DxfAcadVersion,
) -> Result<DxfEntityDraftRecordPlan, Box<dyn Error>> {
    let applicability = admitted_plan(view)?;
    let draft = DxfPointDraft::new(b"Layer0", LOCATION);
    let draft = if version >= DxfAcadVersion::Ac1015 {
        draft
            .with_layout(b"Model")
            .with_lineweight(DxfEntityLineweight::BY_LAYER)
    } else {
        draft
    };
    match view.encode_entity_draft_record(
        applicability,
        DxfEntityDraft::point(draft),
        DxfResourceProfile::Safe,
        &token(),
    )? {
        Ok(plan) => Ok(plan),
        Err(issue) => Err(io::Error::other(format!("{issue:?}")).into()),
    }
}

fn admitted_plan(
    view: DxfRawDocumentView<'_>,
) -> Result<DxfEntityDraftApplicabilityPlan, Box<dyn Error>> {
    let binding = owner_binding(view)?;
    let policy = view.handle_allocation_policy_directory(&token())?;
    let reservation = reserve(view, &policy)?;
    let name = DxfEntityDraftName::canonical(DxfEntityTopic::POINT);
    let identity = match view.prepare_entity_draft_identity(name, binding, reservation, &token())? {
        Ok(plan) => plan,
        Err(DxfEntityDraftIdentityIssue::ReservationCardinality { .. }) => {
            return Err(io::Error::other("draft identity").into());
        }
        _ => return Err(io::Error::other("draft identity issue").into()),
    };
    match view.prepare_entity_draft_applicability(identity, &token())? {
        Ok(plan) => Ok(plan),
        Err(
            DxfEntityDraftApplicabilityIssue::VersionUnavailable { .. }
            | DxfEntityDraftApplicabilityIssue::NotApplicable { .. }
            | DxfEntityDraftApplicabilityIssue::NotYetReviewed { .. },
        ) => Err(io::Error::other("draft applicability").into()),
        _ => Err(io::Error::other("draft applicability issue").into()),
    }
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
        .ok_or(io::Error::other("ready placement"))?;
    match directory.bind(
        placement,
        seacad_dxf_core::DxfHandle::from_u64(0x10),
        &token(),
    )? {
        DxfEntityPlacementOwnerOutcome::Bound(binding) => Ok(binding),
        DxfEntityPlacementOwnerOutcome::Rejected(_) => {
            Err(io::Error::other("owner binding").into())
        }
        _ => Err(io::Error::other("owner outcome").into()),
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
        _ => Err(io::Error::other("reservation outcome").into()),
    }
}

fn source_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version, 0x30);
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"TABLES"),
        group(0, b"TABLE"),
        group(2, b"BLOCK_RECORD"),
        group(0, b"BLOCK_RECORD"),
        group(5, b"10"),
        group(2, b"*Model_Space"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"LAYER"),
        group(0, b"LAYER"),
        group(5, b"11"),
        group(2, b"Layer0"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"APPID"),
        group(0, b"APPID"),
        group(2, b"APP_READY"),
        group(0, b"ENDTAB"),
        group(0, b"ENDSEC"),
    ]);
    if version >= DxfAcadVersion::Ac1015 {
        groups.extend([
            group(0, b"SECTION"),
            group(2, b"OBJECTS"),
            group(0, b"LAYOUT"),
            group(100, b"AcDbPlotSettings"),
            group(1, b"PAGE_SETUP"),
            group(100, b"AcDbLayout"),
            group(1, b"Model"),
            group(0, b"ENDSEC"),
        ]);
    }
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"ENTITIES"),
        group(0, b"POINT"),
        group(5, b"20"),
    ]);
    if version >= DxfAcadVersion::Ac1012 {
        groups.extend([group(330, b"10"), group(100, b"AcDbEntity")]);
    }
    if version >= DxfAcadVersion::Ac1015 {
        groups.push(group(410, b"Model"));
    }
    groups.push(group(8, b"Layer0"));
    if version >= DxfAcadVersion::Ac1015 {
        groups.push(group(370, b"-1"));
    }
    if version >= DxfAcadVersion::Ac1012 {
        groups.push(group(100, b"AcDbPoint"));
    }
    groups.extend([
        group(10, b"1"),
        group(20, b"2"),
        group(30, b"3"),
        group(1001, b"APP_READY"),
        group(1000, b"SECRET_DRAFT_XDATA"),
        group(0, b"LINE"),
        group(1000, b"SECRET_ORPHAN_XDATA"),
        group(0, b"ENDSEC"),
        group(0, b"EOF"),
    ]);
    encode_fixture(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handseed: u64,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version, handseed);
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"TABLES"),
        group(0, b"TABLE"),
        group(2, b"BLOCK_RECORD"),
        group(0, b"BLOCK_RECORD"),
        group(5, b"10"),
        group(2, b"*Model_Space"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"LAYER"),
        group(0, b"LAYER"),
        group(5, b"11"),
        group(2, b"Layer0"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"APPID"),
        group(0, b"APPID"),
        group(2, b"APP_READY"),
        group(0, b"ENDTAB"),
        group(0, b"ENDSEC"),
    ]);
    if version >= DxfAcadVersion::Ac1015 {
        groups.extend([
            group(0, b"SECTION"),
            group(2, b"OBJECTS"),
            group(0, b"LAYOUT"),
            group(100, b"AcDbPlotSettings"),
            group(1, b"PAGE_SETUP"),
            group(100, b"AcDbLayout"),
            group(1, b"Model"),
            group(0, b"ENDSEC"),
        ]);
    }
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"ENTITIES"),
        group(0, b"ENDSEC"),
        group(0, b"EOF"),
    ]);
    encode_fixture(format, version, &groups)
}

fn header(version: DxfAcadVersion, handseed: u64) -> Vec<(i16, Vec<u8>)> {
    vec![
        group(0, b"SECTION"),
        group(2, b"HEADER"),
        group(9, b"$ACADVER"),
        group(1, version.code().as_bytes()),
        group(9, b"$DWGCODEPAGE"),
        group(3, b"ANSI_1252"),
        group(9, b"$HANDSEED"),
        (5, format!("{handseed:X}").into_bytes()),
        group(0, b"ENDSEC"),
    ]
}

fn group(code: i16, value: &[u8]) -> (i16, Vec<u8>) {
    (code, value.to_vec())
}

fn encode_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Vec<u8>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = if format == DxfRawDocumentFormat::Binary {
        DXF_BINARY_SENTINEL.to_vec()
    } else {
        Vec::new()
    };
    for (code, value) in groups {
        match format {
            DxfRawDocumentFormat::Ascii => {
                bytes.extend_from_slice(code.to_string().as_bytes());
                bytes.push(b'\n');
                bytes.extend_from_slice(value);
                bytes.push(b'\n');
            }
            DxfRawDocumentFormat::Binary => {
                if version == DxfAcadVersion::Ac1009 && (0..=254).contains(code) {
                    bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("code"))?);
                } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(code) {
                    bytes.push(u8::MAX);
                    bytes.extend_from_slice(&code.to_le_bytes());
                } else {
                    bytes.extend_from_slice(&code.to_le_bytes());
                }
                if matches!(*code, 10 | 20 | 30) {
                    let text =
                        std::str::from_utf8(value).map_err(|_| io::Error::other("double text"))?;
                    let number = text
                        .parse::<f64>()
                        .map_err(|_| io::Error::other("double value"))?;
                    bytes.extend_from_slice(&number.to_bits().to_le_bytes());
                } else if *code == 370 {
                    let text =
                        std::str::from_utf8(value).map_err(|_| io::Error::other("int16 text"))?;
                    let number = text
                        .parse::<i16>()
                        .map_err(|_| io::Error::other("int16 value"))?;
                    bytes.extend_from_slice(&number.to_le_bytes());
                } else {
                    bytes.extend_from_slice(value);
                    bytes.push(0);
                }
            }
            _ => return Err(io::Error::other("format")),
        }
    }
    Ok(bytes)
}

fn materialize(source: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start = usize::try_from(patch.source_span().start())
            .map_err(|_| io::Error::other("patch start"))?;
        let end = usize::try_from(patch.source_span().end())
            .map_err(|_| io::Error::other("patch end"))?;
        output.extend_from_slice(
            source
                .get(cursor..start)
                .ok_or(io::Error::other("source prefix"))?,
        );
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or(io::Error::other("replacement"))?,
        );
        cursor = end;
    }
    output.extend_from_slice(
        source
            .get(cursor..)
            .ok_or(io::Error::other("source suffix"))?,
    );
    Ok(output)
}

fn replace_once(source: &[u8], from: &[u8], to: &[u8]) -> Result<Vec<u8>, io::Error> {
    if from.len() != to.len() {
        return Err(io::Error::other("replacement length"));
    }
    let offset = source
        .windows(from.len())
        .position(|window| window == from)
        .ok_or(io::Error::other("replacement pattern"))?;
    let end = offset
        .checked_add(to.len())
        .ok_or(io::Error::other("replacement range"))?;
    let mut output = source.to_vec();
    output
        .get_mut(offset..end)
        .ok_or(io::Error::other("replacement slice"))?
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
    let mut observer = NoopDxfReadObserver;
    match format {
        DxfRawDocumentFormat::Ascii => Ok(OpenedDocument::Ascii(DxfAsciiRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &token(),
            &mut observer,
        )?)),
        DxfRawDocumentFormat::Binary => Ok(OpenedDocument::Binary(DxfBinaryRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &token(),
            &mut observer,
        )?)),
        _ => Err(io::Error::other("format").into()),
    }
}

fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
}

struct FinalTamperObserver<'a> {
    path: &'a Path,
    from: &'a [u8],
    to: &'a [u8],
    attempted: bool,
    error: Option<io::Error>,
}

impl<'a> FinalTamperObserver<'a> {
    const fn new(path: &'a Path, from: &'a [u8], to: &'a [u8]) -> Self {
        Self {
            path,
            from,
            to,
            attempted: false,
            error: None,
        }
    }

    fn finish(&mut self) -> Result<(), io::Error> {
        if !self.attempted {
            return Err(io::Error::other("tamper observer was not reached"));
        }
        match self.error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl DxfReadObserver for FinalTamperObserver<'_> {
    fn on_progress(&mut self, progress: DxfReadProgress) -> DxfReadControl {
        if !self.attempted && progress.is_complete() {
            self.attempted = true;
            if let Err(error) = tamper_file_once(self.path, self.from, self.to) {
                self.error = Some(error);
            }
        }
        DxfReadControl::Continue
    }
}

fn tamper_file_once(path: &Path, from: &[u8], to: &[u8]) -> Result<(), io::Error> {
    if from.len() != to.len() {
        return Err(io::Error::other("tamper length"));
    }
    let mut bytes = fs::read(path)?;
    let start = bytes
        .windows(from.len())
        .rposition(|value| value == from)
        .ok_or(io::Error::other("tamper source"))?;
    let end = start
        .checked_add(to.len())
        .ok_or(io::Error::other("tamper range"))?;
    bytes
        .get_mut(start..end)
        .ok_or(io::Error::other("tamper slice"))?
        .copy_from_slice(to);
    fs::write(path, bytes)
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Result<Self, io::Error> {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "seacad-xdata-draft-write-{}-{id}",
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

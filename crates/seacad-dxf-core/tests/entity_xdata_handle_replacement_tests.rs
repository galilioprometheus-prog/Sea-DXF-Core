use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataHandleDestinationState, DxfEntityXDataHandleRemap,
    DxfEntityXDataHandleRemapState, DxfEntityXDataHandleReplacementDirectory,
    DxfEntityXDataHandleReplacementEntry, DxfEntityXDataHandleReplacementPatch,
    DxfEntityXDataHandleReplacementSetDirectory, DxfEntityXDataHandleReplacementSetState,
    DxfEntityXDataHandleReplacementState, DxfEntityXDataHandleReplacementTransactionIssue,
    DxfEntityXDataHandleReplacementTransactionOutcome,
    DxfEntityXDataHandleReplacementTransactionPlan,
    DxfEntityXDataHandleReplacementVerificationIssue,
    DxfEntityXDataHandleReplacementVerificationOutcome,
    DxfEntityXDataHandleReplacementWriteJournal, DxfEntityXDataHandleReplacementWriteOutcome,
    DxfError, DxfHandle, DxfHandleParseIssue, DxfIoOperation, DxfMemorySource,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfReadControl, DxfReadObserver, DxfReadOptions,
    DxfReadProgress, DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[test]
fn every_supported_same_format_write_verifies_and_restores() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataHandleReplacementWriteJournal>();
    assert_send_sync::<DxfEntityXDataHandleReplacementWriteOutcome>();
    let directory = TestDirectory::new()?;
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let source_bytes = source_fixture(format, version)?;
            let destination_bytes = destination_fixture(format, version)?;
            let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
            let destination_storage =
                DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
            let source = open(&source_storage, format)?;
            let destination = open(&destination_storage, format)?;
            let sets = source
                .view()
                .entity_xdata_handle_replacement_set_directory(
                    destination.view(),
                    &mappings()?,
                    DxfResourceProfile::Safe,
                    &token(),
                )?;
            let plan = planned(source.view(), &sets)?;
            let kind = match format {
                DxfRawDocumentFormat::Ascii => "ascii",
                DxfRawDocumentFormat::Binary => "binary",
                _ => return Err(io::Error::other("unsupported test format").into()),
            };
            let output = directory
                .path()
                .join(format!("{}-{kind}.dxf", version.code()));
            let mut progress = Vec::new();
            let mut observer = |value: DxfReadProgress| {
                progress.push((value.processed_bytes(), value.total_bytes()));
                DxfReadControl::Continue
            };
            let journal = written(plan.write_reparse_verify_and_journal_to_new_file(
                &sets,
                source.view(),
                &output,
                DxfResourceProfile::Safe,
                &token(),
                &mut observer,
            )?)?;
            let output_bytes = fs::read(&output)?;
            let output_storage = DxfMemorySource::new(&output_bytes, DxfResourceProfile::Safe)?;
            let output_document = open(&output_storage, format)?;
            let write = journal.write_receipt();
            let verification = journal.verification_receipt();
            assert_eq!(write.source_id(), source.view().source_id());
            assert_eq!(write.output_id(), output_document.view().source_id());
            assert_eq!(write.bytes_written(), plan.transaction().projected_len());
            assert_eq!(write.patch_count(), 1);
            assert_eq!(verification.source_id(), write.source_id());
            assert_eq!(
                verification.destination_id(),
                destination.view().source_id()
            );
            assert_eq!(verification.post_image_id(), write.output_id());
            assert_eq!(verification.replacement_count(), 1);
            assert_eq!(journal.inverse_plan().source_id(), write.output_id());
            assert_eq!(
                apply_plan(&output_bytes, journal.inverse_plan())?,
                source_bytes
            );
            assert!(!progress.is_empty());
            assert!(progress.windows(2).all(|pair| pair[0].0 <= pair[1].0));
            assert!(!format!("{journal:?}").contains("SECRET_REPLACEMENT_SOURCE"));
            let (write_part, verification_part, inverse) = journal.into_parts();
            assert_eq!(write_part, write);
            assert_eq!(verification_part, verification);
            assert_eq!(apply_plan(&output_bytes, &inverse)?, source_bytes);
        }
    }
    Ok(())
}

#[test]
fn write_preconditions_preserve_existing_and_leave_no_partial_output() -> Result<(), Box<dyn Error>>
{
    let directory = TestDirectory::new()?;
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let sets = source
        .view()
        .entity_xdata_handle_replacement_set_directory(
            destination.view(),
            &mappings()?,
            DxfResourceProfile::Safe,
            &token(),
        )?;
    let plan = planned(source.view(), &sets)?;
    let mut noop = NoopDxfReadObserver;

    let other_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_storage = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other = open(&other_storage, DxfRawDocumentFormat::Ascii)?;
    let other_destination = open(&other_destination_storage, DxfRawDocumentFormat::Ascii)?;
    let other_sets = other.view().entity_xdata_handle_replacement_set_directory(
        other_destination.view(),
        &mappings()?,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    let foreign = directory.path().join("foreign-evidence.dxf");
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            &other_sets,
            source.view(),
            &foreign,
            DxfResourceProfile::Safe,
            &token(),
            &mut noop,
        )?,
        DxfEntityXDataHandleReplacementWriteOutcome::Unavailable(
            DxfEntityXDataHandleReplacementVerificationIssue::EvidenceMismatch
        )
    ));
    assert!(!foreign.exists());

    let existing = directory.path().join("existing.dxf");
    fs::write(&existing, b"KEEP")?;
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            &sets,
            source.view(),
            &existing,
            DxfResourceProfile::Safe,
            &token(),
            &mut noop,
        ),
        Err(DxfError::Io {
            operation: DxfIoOperation::Create,
            kind: io::ErrorKind::AlreadyExists,
            ..
        })
    ));
    assert_eq!(fs::read(&existing)?, b"KEEP");

    let mismatch = directory.path().join("source-mismatch.dxf");
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            &sets,
            other.view(),
            &mismatch,
            DxfResourceProfile::Safe,
            &token(),
            &mut noop,
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(!mismatch.exists());

    let cancelled = directory.path().join("observer-cancelled.dxf");
    let mut cancel_after_start = |progress: DxfReadProgress| {
        if progress.processed_bytes() == 0 {
            DxfReadControl::Continue
        } else {
            DxfReadControl::Cancel
        }
    };
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            &sets,
            source.view(),
            &cancelled,
            DxfResourceProfile::Safe,
            &token(),
            &mut cancel_after_start,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!cancelled.exists());

    let pre_cancelled = directory.path().join("pre-cancelled.dxf");
    let cancelled_token = token();
    cancelled_token.cancel();
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            &sets,
            source.view(),
            &pre_cancelled,
            DxfResourceProfile::Safe,
            &cancelled_token,
            &mut noop,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!pre_cancelled.exists());
    Ok(())
}

#[test]
fn post_write_reparse_and_raw_failures_remove_the_destination() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let sets = source
        .view()
        .entity_xdata_handle_replacement_set_directory(
            destination.view(),
            &mappings()?,
            DxfResourceProfile::Safe,
            &token(),
        )?;
    let plan = planned(source.view(), &sets)?;

    let invalid = directory.path().join("strict-reparse-failure.dxf");
    let mut invalid_observer = FinalTamperObserver::new(&invalid, b"EOF", b"EOG");
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            &sets,
            source.view(),
            &invalid,
            DxfResourceProfile::Safe,
            &token(),
            &mut invalid_observer,
        ),
        Err(DxfError::MissingAsciiEof { .. })
    ));
    invalid_observer.finish()?;
    assert!(!invalid.exists());

    let tampered = directory.path().join("raw-mismatch.dxf");
    let mut tamper_observer = FinalTamperObserver::new(
        &tampered,
        b"SECRET_REPLACEMENT_SOURCE",
        b"BROKEN_REPLACEMENT_SOURCE",
    );
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            &sets,
            source.view(),
            &tampered,
            DxfResourceProfile::Safe,
            &token(),
            &mut tamper_observer,
        ),
        Err(DxfError::TransactionPostImageMismatch { .. })
    ));
    tamper_observer.finish()?;
    assert!(!tampered.exists());
    Ok(())
}

#[test]
fn every_supported_destination_dialect_encodes_one_exact_complete_group()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                let source_bytes = source_fixture(source_format, version)?;
                let destination_bytes = destination_fixture(destination_format, version)?;
                let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
                let destination_storage =
                    DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
                let source = open(&source_storage, source_format)?;
                let destination = open(&destination_storage, destination_format)?;
                let directory = source.view().entity_xdata_handle_replacement_directory(
                    destination.view(),
                    &mappings()?,
                    DxfResourceProfile::Safe,
                    &token(),
                )?;
                assert_eq!(directory.destination_format(), destination_format);
                assert_directory(&directory, version, destination_format)?;
                let sets = source
                    .view()
                    .entity_xdata_handle_replacement_set_directory(
                        destination.view(),
                        &mappings()?,
                        DxfResourceProfile::Safe,
                        &token(),
                    )?;
                assert_sets(&sets)?;
                assert_transaction(
                    &source_bytes,
                    source.view(),
                    &sets,
                    version,
                    source_format,
                    destination_format,
                )?;
            }
        }
    }
    Ok(())
}

#[test]
fn unavailable_destinations_publish_no_replacement_bytes() -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Binary)?;
    let directory = source.view().entity_xdata_handle_replacement_directory(
        destination.view(),
        &mappings()?,
        DxfResourceProfile::Safe,
        &token(),
    )?;

    for (index, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        assert!(directory.destination_for_entry(entry).is_some());
        assert_eq!(
            directory.replacement_bytes_for_entry(entry).is_some(),
            index == 0
        );
        assert_eq!(directory.patch_for_entry(entry).is_some(), index == 0);
    }
    let patch = directory
        .patch_for_entry(directory.entries()[0])
        .ok_or(io::Error::other("ready patch"))?;
    assert_eq!(patch.source_id(), directory.source_id());
    assert_eq!(patch.destination_id(), directory.destination_id());
    assert_eq!(patch.replacement_ordinal(), 0);
    assert_eq!(patch.source_group().group_code().value(), 1005);
    assert_eq!(patch.target(), handle(u64::MAX));
    let source_len = usize::try_from(patch.source_group().full_span().len())?;
    let mut source_group = vec![0; source_len];
    source
        .view()
        .read_span(patch.source_group().full_span(), &mut source_group)?;
    assert_eq!(source_group, b"1005\n1\n");
    assert_eq!(
        directory.replacement_bytes_for_patch(patch),
        directory.replacement_bytes_for_entry(directory.entries()[0])
    );
    assert_eq!(directory.entry(u64::MAX), None);
    Ok(())
}

#[test]
fn directory_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataHandleReplacementDirectory>();
    assert_send_sync::<DxfEntityXDataHandleReplacementSetDirectory>();
    assert_copy::<DxfEntityXDataHandleReplacementEntry>();
    assert_copy::<DxfEntityXDataHandleReplacementPatch>();
    assert!(size_of::<DxfEntityXDataHandleReplacementEntry>() <= 160);
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        source.view().entity_xdata_handle_replacement_directory(
            destination.view(),
            &[],
            DxfResourceProfile::Safe,
            &cancelled
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        source.view().entity_xdata_handle_replacement_set_directory(
            destination.view(),
            &[],
            DxfResourceProfile::Safe,
            &cancelled
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = source.view().entity_xdata_handle_replacement_directory(
        destination.view(),
        &mappings()?,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open(&other_source_storage, DxfRawDocumentFormat::Binary)?;
    let other = other_source
        .view()
        .entity_xdata_handle_replacement_directory(
            destination.view(),
            &mappings()?,
            DxfResourceProfile::Safe,
            &token(),
        )?;
    assert_eq!(directory.destination_for_entry(other.entries()[0]), None);
    assert_eq!(
        directory.replacement_bytes_for_entry(other.entries()[0]),
        None
    );
    let other_patch = other
        .patch_for_entry(other.entries()[0])
        .ok_or(io::Error::other("other ready patch"))?;
    assert_eq!(directory.replacement_bytes_for_patch(other_patch), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_REPLACEMENT_SOURCE"));
    assert!(!debug.contains("1005\\nFFFFFFFFFFFFFFFF"));
    Ok(())
}

fn assert_directory(
    directory: &DxfEntityXDataHandleReplacementDirectory,
    version: DxfAcadVersion,
    format: DxfRawDocumentFormat,
) -> Result<(), io::Error> {
    let expected_unavailable = [
        DxfEntityXDataHandleDestinationState::Missing {
            target: handle(0x33),
        },
        DxfEntityXDataHandleDestinationState::Ambiguous {
            target: handle(0x44),
            target_count: 2,
        },
        DxfEntityXDataHandleDestinationState::RemapUnavailable(
            DxfEntityXDataHandleRemapState::SourceInvalid(DxfHandleParseIssue::InvalidDigit {
                offset: 1,
            }),
        ),
    ];
    let entries = directory.entries();
    assert_eq!(entries.len(), 4);
    let ready = entries[0];
    let expected = expected_group(version, format);
    assert_eq!(
        ready.state(),
        DxfEntityXDataHandleReplacementState::Ready {
            target: handle(u64::MAX),
            encoded_byte_count: u32::try_from(expected.len())
                .map_err(|_| io::Error::other("encoded byte count"))?,
        }
    );
    assert_eq!(
        directory.replacement_bytes_for_entry(ready),
        Some(expected.as_slice())
    );
    let patch = directory
        .patch_for_entry(ready)
        .ok_or(io::Error::other("matrix patch"))?;
    assert_eq!(patch.source_id(), directory.source_id());
    assert_eq!(patch.destination_id(), directory.destination_id());
    assert_eq!(patch.source_group().group_code().value(), 1005);
    assert_eq!(patch.target(), handle(u64::MAX));
    assert_eq!(
        directory.replacement_bytes_for_patch(patch),
        Some(expected.as_slice())
    );
    for (offset, state) in expected_unavailable.into_iter().enumerate() {
        let entry = entries[offset + 1];
        assert_eq!(entry.ordinal(), (offset + 1) as u64);
        assert_eq!(entry.destination_ordinal(), (offset + 1) as u64);
        assert_eq!(
            entry.state(),
            DxfEntityXDataHandleReplacementState::DestinationUnavailable(state)
        );
        assert_eq!(directory.replacement_bytes_for_entry(entry), None);
    }
    Ok(())
}

fn assert_sets(directory: &DxfEntityXDataHandleReplacementSetDirectory) -> Result<(), io::Error> {
    let entries = directory.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(
        entries[0].state(),
        DxfEntityXDataHandleReplacementSetState::Ready {
            replacement_count: 1
        }
    );
    assert_eq!(
        entries[1].state(),
        DxfEntityXDataHandleReplacementSetState::Unavailable {
            replacement_count: 1,
            unavailable_count: 1,
            first_unavailable_ordinal: 1,
        }
    );
    assert_eq!(
        entries[2].state(),
        DxfEntityXDataHandleReplacementSetState::Unavailable {
            replacement_count: 2,
            unavailable_count: 2,
            first_unavailable_ordinal: 2,
        }
    );
    for (ordinal, entry) in entries.iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(entry.destination_id(), directory.destination_id());
        let replacements = directory
            .replacements_for_entry(entry)
            .ok_or(io::Error::other("set replacements"))?;
        for replacement in replacements.iter().copied() {
            assert_eq!(
                directory
                    .patch_for_replacement(entry, replacement)
                    .is_some(),
                matches!(
                    replacement.state(),
                    DxfEntityXDataHandleReplacementState::Ready { .. }
                )
            );
        }
    }
    assert_eq!(directory.entry(u64::MAX), None);
    Ok(())
}

fn assert_transaction(
    source_bytes: &[u8],
    source: DxfRawDocumentView<'_>,
    sets: &DxfEntityXDataHandleReplacementSetDirectory,
    version: DxfAcadVersion,
    source_format: DxfRawDocumentFormat,
    destination_format: DxfRawDocumentFormat,
) -> Result<(), Box<dyn Error>> {
    let unavailable = source.plan_entity_xdata_handle_replacement_set(
        sets,
        sets.entries()[1],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert!(matches!(
        unavailable,
        DxfEntityXDataHandleReplacementTransactionOutcome::Unavailable(
            DxfEntityXDataHandleReplacementTransactionIssue::SetUnavailable(
                DxfEntityXDataHandleReplacementSetState::Unavailable { .. }
            )
        )
    ));
    let outcome = source.plan_entity_xdata_handle_replacement_set(
        sets,
        sets.entries()[0],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    if source_format != destination_format {
        assert!(matches!(
            outcome,
            DxfEntityXDataHandleReplacementTransactionOutcome::Unavailable(
                DxfEntityXDataHandleReplacementTransactionIssue::FormatMismatch {
                    source,
                    destination,
                }
            ) if source == source_format && destination == destination_format
        ));
        return Ok(());
    }
    let DxfEntityXDataHandleReplacementTransactionOutcome::Planned(plan) = outcome else {
        return Err(io::Error::other("same-dialect transaction unavailable").into());
    };
    assert_eq!(plan.source_id(), sets.source_id());
    assert_eq!(plan.destination_id(), sets.destination_id());
    assert_eq!(plan.set(), sets.entries()[0]);
    plan.transaction().validate_source_precondition(source)?;
    assert_eq!(plan.transaction().patches().len(), 1);
    let replacement = expected_group(version, destination_format);
    assert_eq!(
        plan.transaction().replacement_bytes_for_patch_ordinal(0),
        Some(replacement.as_slice())
    );
    let inverse = expected_original_group(version, source_format);
    assert_eq!(
        plan.transaction().inverse_bytes_for_patch_ordinal(0),
        Some(inverse.as_slice())
    );
    let post_bytes = apply_plan(source_bytes, plan.transaction())?;
    let post_storage = DxfMemorySource::new(&post_bytes, DxfResourceProfile::Safe)?;
    let post = open(&post_storage, source_format)?;
    let verified = plan.verify_post_image(
        sets,
        source,
        post.view(),
        DxfResourceProfile::Safe,
        &token(),
    )?;
    let DxfEntityXDataHandleReplacementVerificationOutcome::Verified(journal) = verified else {
        return Err(io::Error::other("post-image verification unavailable").into());
    };
    assert_eq!(journal.receipt().source_id(), sets.source_id());
    assert_eq!(journal.receipt().destination_id(), sets.destination_id());
    assert_eq!(journal.receipt().post_image_id(), post.view().source_id());
    assert_eq!(journal.receipt().replacement_count(), 1);
    assert_eq!(
        apply_plan(&post_bytes, journal.inverse_plan())?,
        source_bytes
    );
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        plan.verify_post_image(
            sets,
            source,
            post.view(),
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let mut tampered = post_bytes.clone();
    let handle_start = tampered
        .windows(16)
        .position(|window| window == b"FFFFFFFFFFFFFFFF")
        .ok_or(io::Error::other("replacement handle"))?;
    tampered[handle_start] = b'E';
    let tampered_storage = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open(&tampered_storage, source_format)?;
    assert!(matches!(
        plan.verify_post_image(
            sets,
            source,
            tampered_document.view(),
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::TransactionPostImageMismatch { .. })
    ));
    Ok(())
}

fn planned(
    source: DxfRawDocumentView<'_>,
    sets: &DxfEntityXDataHandleReplacementSetDirectory,
) -> Result<Box<DxfEntityXDataHandleReplacementTransactionPlan>, Box<dyn Error>> {
    match source.plan_entity_xdata_handle_replacement_set(
        sets,
        sets.entries()[0],
        DxfResourceProfile::Safe,
        &token(),
    )? {
        DxfEntityXDataHandleReplacementTransactionOutcome::Planned(plan) => Ok(plan),
        other => Err(io::Error::other(format!("unexpected plan outcome: {other:?}")).into()),
    }
}

fn written(
    outcome: DxfEntityXDataHandleReplacementWriteOutcome,
) -> Result<DxfEntityXDataHandleReplacementWriteJournal, io::Error> {
    match outcome {
        DxfEntityXDataHandleReplacementWriteOutcome::Written(journal) => Ok(journal),
        other => Err(io::Error::other(format!(
            "unexpected write outcome: {other:?}"
        ))),
    }
}

fn apply_plan(source: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, DxfError> {
    let capacity = usize::try_from(plan.projected_len()).map_err(|_| DxfError::Cancelled)?;
    let mut output = Vec::with_capacity(capacity);
    let mut cursor = 0_usize;
    for patch in plan.patches().iter().copied() {
        let start =
            usize::try_from(patch.source_span().start()).map_err(|_| DxfError::Cancelled)?;
        let end = usize::try_from(patch.source_span().end()).map_err(|_| DxfError::Cancelled)?;
        output.extend_from_slice(source.get(cursor..start).ok_or(DxfError::Cancelled)?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or(DxfError::Cancelled)?,
        );
        cursor = end;
    }
    output.extend_from_slice(source.get(cursor..).ok_or(DxfError::Cancelled)?);
    if output.len() != capacity {
        return Err(DxfError::Cancelled);
    }
    Ok(output)
}

fn expected_group(version: DxfAcadVersion, format: DxfRawDocumentFormat) -> Vec<u8> {
    let mut bytes = Vec::new();
    match format {
        DxfRawDocumentFormat::Ascii => bytes.extend_from_slice(b"1005\n"),
        DxfRawDocumentFormat::Binary if version == DxfAcadVersion::Ac1009 => {
            bytes.extend_from_slice(&[u8::MAX, 0xED, 0x03]);
        }
        DxfRawDocumentFormat::Binary => bytes.extend_from_slice(&[0xED, 0x03]),
        _ => {}
    }
    bytes.extend_from_slice(b"FFFFFFFFFFFFFFFF");
    match format {
        DxfRawDocumentFormat::Ascii => bytes.push(b'\n'),
        DxfRawDocumentFormat::Binary => bytes.push(0),
        _ => {}
    }
    bytes
}

fn expected_original_group(version: DxfAcadVersion, format: DxfRawDocumentFormat) -> Vec<u8> {
    let mut bytes = Vec::new();
    match format {
        DxfRawDocumentFormat::Ascii => bytes.extend_from_slice(b"1005\n1\n"),
        DxfRawDocumentFormat::Binary if version == DxfAcadVersion::Ac1009 => {
            bytes.extend_from_slice(&[u8::MAX, 0xED, 0x03, b'1', 0]);
        }
        DxfRawDocumentFormat::Binary => bytes.extend_from_slice(&[0xED, 0x03, b'1', 0]),
        _ => {}
    }
    bytes
}

fn mappings() -> Result<[DxfEntityXDataHandleRemap; 3], io::Error> {
    Ok([remap(1, u64::MAX)?, remap(2, 0x33)?, remap(3, 0x44)?])
}

fn remap(source: u64, target: u64) -> Result<DxfEntityXDataHandleRemap, io::Error> {
    DxfEntityXDataHandleRemap::new(handle(source), handle(target))
        .map_err(|issue| io::Error::other(format!("{issue:?}")))
}

const fn handle(value: u64) -> DxfHandle {
    DxfHandle::from_u64(value)
}

#[derive(Clone, Copy)]
struct Group<'a>(i16, &'a [u8]);

fn source_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    encode(
        format,
        version,
        &document_groups(
            version,
            &[
                Group(0, b"POINT"),
                Group(5, b"1"),
                Group(1001, b"APP"),
                Group(1000, b"SECRET_REPLACEMENT_SOURCE"),
                Group(1005, b"1"),
                Group(0, b"POINT"),
                Group(5, b"2"),
                Group(1001, b"APP"),
                Group(1005, b"2"),
                Group(0, b"POINT"),
                Group(5, b"3"),
                Group(1001, b"APP"),
                Group(1005, b"3"),
                Group(1005, b"0x1"),
            ],
        ),
    )
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    encode(
        format,
        version,
        &document_groups(
            version,
            &[
                Group(0, b"POINT"),
                Group(5, b"FFFFFFFFFFFFFFFF"),
                Group(0, b"POINT"),
                Group(5, b"44"),
                Group(0, b"POINT"),
                Group(5, b"044"),
            ],
        ),
    )
}

fn document_groups<'a>(version: DxfAcadVersion, body: &[Group<'a>]) -> Vec<Group<'a>> {
    let mut groups = vec![
        Group(0, b"SECTION"),
        Group(2, b"HEADER"),
        Group(9, b"$ACADVER"),
        Group(1, version.code().as_bytes()),
        Group(0, b"ENDSEC"),
        Group(0, b"SECTION"),
        Group(2, b"ENTITIES"),
    ];
    groups.extend_from_slice(body);
    groups.extend([Group(0, b"ENDSEC"), Group(0, b"EOF")]);
    groups
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[Group<'_>],
) -> io::Result<Vec<u8>> {
    let mut bytes = if format == DxfRawDocumentFormat::Binary {
        DXF_BINARY_SENTINEL.to_vec()
    } else {
        Vec::new()
    };
    for Group(code, value) in groups {
        if format == DxfRawDocumentFormat::Ascii {
            bytes.extend_from_slice(code.to_string().as_bytes());
            bytes.push(b'\n');
            bytes.extend_from_slice(value);
            bytes.push(b'\n');
        } else {
            push_code(&mut bytes, version, *code)?;
            bytes.extend_from_slice(value);
            bytes.push(0);
        }
    }
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 && (0..=254).contains(&code) {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(&code) {
        bytes.push(u8::MAX);
        bytes.extend_from_slice(&code.to_le_bytes());
    } else if version == DxfAcadVersion::Ac1009 {
        return Err(io::Error::other("group code"));
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

enum Opened<'a> {
    Ascii(DxfAsciiRawDocument<'a>),
    Binary(DxfBinaryRawDocument<'a>),
}

impl Opened<'_> {
    fn view(&self) -> DxfRawDocumentView<'_> {
        match self {
            Self::Ascii(document) => DxfRawDocumentView::from_ascii(document),
            Self::Binary(document) => DxfRawDocumentView::from_binary(document),
        }
    }
}

fn open<'a>(
    source: &'a dyn DxfByteSource,
    format: DxfRawDocumentFormat,
) -> Result<Opened<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    match format {
        DxfRawDocumentFormat::Ascii => Ok(Opened::Ascii(DxfAsciiRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &token(),
            &mut observer,
        )?)),
        DxfRawDocumentFormat::Binary => Ok(Opened::Binary(DxfBinaryRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &token(),
            &mut observer,
        )?)),
        _ => Err(DxfError::Cancelled),
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
        if !self.attempted && progress.processed_bytes() == progress.total_bytes() {
            self.attempted = true;
            if let Err(error) = tamper_once(self.path, self.from, self.to) {
                self.error = Some(error);
            }
        }
        DxfReadControl::Continue
    }
}

fn tamper_once(path: &Path, from: &[u8], to: &[u8]) -> Result<(), io::Error> {
    if from.len() != to.len() {
        return Err(io::Error::other("tamper length"));
    }
    let mut bytes = fs::read(path)?;
    let start = bytes
        .windows(from.len())
        .rposition(|value| value == from)
        .ok_or(io::Error::other("tamper source"))?;
    bytes
        .get_mut(start..start + from.len())
        .ok_or(io::Error::other("tamper range"))?
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
            "seacad-xdata-handle-replacement-write-{}-{id}",
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

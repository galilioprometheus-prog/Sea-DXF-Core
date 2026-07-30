use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfByteSource, DxfCancellationToken, DxfError, DxfHandleAssignmentPlanOutcome, DxfIoOperation,
    DxfMemorySource, DxfRawDocumentView, DxfReadControl, DxfReadOptions, DxfResourceProfile,
    DxfTransactionPlan, DxfTransactionWriteJournal, NoopDxfReadObserver,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[test]
fn every_supported_version_writes_verified_ascii_binary_handle_plans() -> Result<(), Box<dyn Error>>
{
    let directory = TestDirectory::new()?;
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), b"PAYLOAD");
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_plan = handle_plan(DxfRawDocumentView::from(&ascii))?;
        let ascii_output = directory
            .path()
            .join(format!("{}-ascii.dxf", version.code()));
        let mut ascii_progress = Vec::new();
        let mut ascii_observer = |progress: seacad_dxf_core::DxfReadProgress| {
            ascii_progress.push((progress.processed_bytes(), progress.total_bytes()));
            DxfReadControl::Continue
        };
        let ascii_journal = ascii_plan.write_reparse_and_journal_to_new_file(
            DxfRawDocumentView::from(&ascii),
            &ascii_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut ascii_observer,
        )?;
        assert_journal_round_trip(
            &ascii_output,
            &directory
                .path()
                .join(format!("{}-ascii-restored.dxf", version.code())),
            DxfRawDocumentView::from(&ascii),
            &ascii_plan,
            ascii_journal,
            &ascii_progress,
            false,
        )?;

        let binary_bytes = binary_fixture(version, b"PAYLOAD")?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_plan = handle_plan(DxfRawDocumentView::from(&binary))?;
        let binary_output = directory
            .path()
            .join(format!("{}-binary.dxf", version.code()));
        let mut binary_progress = Vec::new();
        let mut binary_observer = |progress: seacad_dxf_core::DxfReadProgress| {
            binary_progress.push((progress.processed_bytes(), progress.total_bytes()));
            DxfReadControl::Continue
        };
        let binary_journal = binary_plan.write_reparse_and_journal_to_new_file(
            DxfRawDocumentView::from(&binary),
            &binary_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut binary_observer,
        )?;
        assert_journal_round_trip(
            &binary_output,
            &directory
                .path()
                .join(format!("{}-binary-restored.dxf", version.code())),
            DxfRawDocumentView::from(&binary),
            &binary_plan,
            binary_journal,
            &binary_progress,
            true,
        )?;
    }
    Ok(())
}

#[test]
fn empty_plan_is_verified_and_existing_or_mismatched_destinations_are_untouched()
-> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let bytes = ascii_fixture("AC1032", b"PAYLOAD");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let plan = DxfRawDocumentView::from(&document)
        .transaction_plan_builder(DxfResourceProfile::Safe)?
        .finish(&DxfCancellationToken::default())?;

    let output = directory.path().join("empty-plan.dxf");
    let mut noop = NoopDxfReadObserver;
    let receipt = plan.write_to_new_file(
        DxfRawDocumentView::from(&document),
        &output,
        &DxfCancellationToken::default(),
        &mut noop,
    )?;
    assert_eq!(fs::read(&output)?, bytes);
    assert_eq!(receipt.source_id(), document.source_id());
    assert_eq!(receipt.output_id(), document.source_id());
    assert_eq!(receipt.patch_count(), 0);

    let existing = directory.path().join("existing.dxf");
    fs::write(&existing, b"KEEP")?;
    assert!(matches!(
        plan.write_to_new_file(
            DxfRawDocumentView::from(&document),
            &existing,
            &DxfCancellationToken::default(),
            &mut noop,
        ),
        Err(DxfError::Io {
            operation: DxfIoOperation::Create,
            kind: io::ErrorKind::AlreadyExists,
            ..
        })
    ));
    assert_eq!(fs::read(existing)?, b"KEEP");

    let other_bytes = ascii_fixture("AC1027", b"PAYLOAD");
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let mismatch = directory.path().join("mismatch.dxf");
    assert!(matches!(
        plan.write_to_new_file(
            DxfRawDocumentView::from(&other),
            &mismatch,
            &DxfCancellationToken::default(),
            &mut noop,
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(!mismatch.exists());
    Ok(())
}

#[test]
fn chunked_write_cancellation_removes_the_incomplete_output() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let payload = vec![b'A'; 140_000];
    let bytes = ascii_fixture("AC1032", &payload);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let span = view
        .group(12)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(
        span,
        &vec![b'B'; payload.len()],
        &DxfCancellationToken::default(),
    )?;
    let plan = builder.finish(&DxfCancellationToken::default())?;
    let output = directory.path().join("cancelled.dxf");
    let mut observer = |progress: seacad_dxf_core::DxfReadProgress| {
        if progress.processed_bytes() >= 64 * 1024 {
            DxfReadControl::Cancel
        } else {
            DxfReadControl::Continue
        }
    };
    assert!(matches!(
        plan.write_to_new_file(
            view,
            &output,
            &DxfCancellationToken::default(),
            &mut observer,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!output.exists());
    Ok(())
}

#[test]
fn strict_reparse_failure_removes_the_verified_raw_output() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfTransactionWriteJournal>();

    let directory = TestDirectory::new()?;
    let bytes = ascii_fixture("AC1032", b"PAYLOAD");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let eof = view
        .group(view.group_count() - 1)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(eof, b"EOG", &DxfCancellationToken::default())?;
    let plan = builder.finish(&DxfCancellationToken::default())?;
    let output = directory.path().join("invalid-post-image.dxf");
    let mut noop = NoopDxfReadObserver;

    assert!(matches!(
        plan.write_reparse_and_journal_to_new_file(
            view,
            &output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut noop,
        ),
        Err(DxfError::MissingAsciiEof { .. })
    ));
    assert!(!output.exists());
    Ok(())
}

fn handle_plan(view: DxfRawDocumentView<'_>) -> Result<DxfTransactionPlan, DxfError> {
    let policy = view.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    match view.plan_handle_assignments(
        &policy,
        &[1],
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )? {
        DxfHandleAssignmentPlanOutcome::Planned(plan) => Ok(plan.into_transaction()),
        _ => Err(invalid_test_data()),
    }
}

fn assert_journal_round_trip(
    output: &Path,
    restored_output: &Path,
    source_view: DxfRawDocumentView<'_>,
    plan: &DxfTransactionPlan,
    journal: DxfTransactionWriteJournal,
    progress: &[(u64, u64)],
    binary: bool,
) -> Result<(), Box<dyn Error>> {
    let receipt = journal.receipt();
    let bytes = fs::read(output)?;
    assert_eq!(bytes.len() as u64, plan.projected_len());
    assert_eq!(receipt.source_id(), plan.source_id());
    assert_eq!(receipt.bytes_written(), plan.projected_len());
    assert_eq!(receipt.patch_count(), plan.patches().len() as u64);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let output_id = if binary {
        open_binary(&source)?.source_id()
    } else {
        open_ascii(&source)?.source_id()
    };
    assert_eq!(receipt.output_id(), output_id);
    assert_eq!(journal.inverse_plan().source_id(), receipt.output_id());
    assert_eq!(
        journal.inverse_plan().projected_len(),
        source_view.source_len()
    );
    assert_eq!(
        progress.first().copied(),
        Some((0, plan.source_len() + plan.projected_len()))
    );
    assert_eq!(
        progress.last().copied(),
        Some((
            plan.source_len() + plan.projected_len(),
            plan.source_len() + plan.projected_len()
        ))
    );
    assert!(progress.windows(2).all(|pair| pair[0].0 < pair[1].0));

    let (_, inverse) = journal.into_parts();
    let post_source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let mut noop = NoopDxfReadObserver;
    let restored_journal = if binary {
        let post_image = open_binary(&post_source)?;
        inverse.write_reparse_and_journal_to_new_file(
            DxfRawDocumentView::from(&post_image),
            restored_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut noop,
        )?
    } else {
        let post_image = open_ascii(&post_source)?;
        inverse.write_reparse_and_journal_to_new_file(
            DxfRawDocumentView::from(&post_image),
            restored_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut noop,
        )?
    };
    let source_len =
        usize::try_from(source_view.source_len()).map_err(|_| io::Error::other("source length"))?;
    let mut expected_source = vec![0_u8; source_len];
    let source_span = ByteSpan::new(0, source_view.source_len())
        .ok_or_else(|| io::Error::other("source span"))?;
    source_view.read_span(source_span, &mut expected_source)?;
    assert_eq!(fs::read(restored_output)?, expected_source);
    assert_eq!(
        restored_journal.receipt().output_id(),
        source_view.source_id()
    );
    assert_eq!(
        restored_journal.inverse_plan().source_id(),
        source_view.source_id()
    );
    assert_eq!(
        restored_journal.inverse_plan().projected_len(),
        plan.projected_len()
    );
    Ok(())
}

fn ascii_fixture(version: &str, payload: &[u8]) -> Vec<u8> {
    let mut bytes = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n9\n$HANDSEED\n5\n10\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nEXISTING\n5\nF\n0\nTARGET\n1\n"
    )
    .into_bytes();
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(b"\n0\nENDSEC\n0\nEOF\n");
    bytes
}

fn binary_fixture(version: DxfAcadVersion, payload: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (9, b"$HANDSEED"),
        (5, b"10"),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"OBJECTS"),
        (0, b"EXISTING"),
        (5, b"F"),
        (0, b"TARGET"),
        (1, payload),
        (0, b"ENDSEC"),
        (0, b"EOF"),
    ] {
        push_binary_string(&mut bytes, version, code, value)?;
    }
    Ok(bytes)
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        DxfIoOperation::Read,
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
            "seacad-transaction-write-{}-{id}",
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

fn assert_send_sync<T: Send + Sync>() {}

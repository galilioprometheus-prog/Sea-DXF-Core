use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonFieldPatch, DxfEntityEditOutcome, DxfEntityEditPlan,
    DxfEntityEditValue, DxfEntityEditVerificationIssue, DxfEntityEditWriteJournal,
    DxfEntityEditWriteOutcome, DxfEntityField, DxfEntityPatch, DxfError, DxfIoOperation,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadControl, DxfReadObserver,
    DxfReadOptions, DxfReadProgress, DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[test]
fn every_dialect_writes_verifies_and_restores_ascii_binary() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version)?;
            let name = match format {
                DxfRawDocumentFormat::Ascii => "ascii",
                DxfRawDocumentFormat::Binary => "binary",
                _ => return Err(io::Error::other("unsupported test format").into()),
            };
            let output = directory
                .path()
                .join(format!("{}-{name}.dxf", version.code()));
            verify_write(&bytes, format, &output)?;
        }
    }
    Ok(())
}

#[test]
fn semantic_failure_is_typed_and_removes_the_destination() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let output = directory.path().join("semantic-mismatch.dxf");
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let plan = edit_plan(view)?;
    let mut observer = FinalTamperObserver::new(&output, b"NEW", b"BAD");

    let outcome = plan.write_reparse_verify_and_journal_to_new_file(
        view,
        &output,
        DxfResourceProfile::Safe,
        &token(),
        &mut observer,
    )?;
    observer.finish()?;
    assert!(matches!(
        outcome,
        DxfEntityEditWriteOutcome::Unavailable(DxfEntityEditVerificationIssue::ValueMismatch {
            field: DxfEntityField::LAYER,
            ..
        })
    ));
    assert!(!output.exists());
    Ok(())
}

#[test]
fn unrelated_raw_mismatch_removes_the_destination() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let output = directory.path().join("raw-mismatch.dxf");
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let plan = edit_plan(view)?;
    let mut observer = FinalTamperObserver::new(&output, b"KEEP", b"EVIL");

    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            view,
            &output,
            DxfResourceProfile::Safe,
            &token(),
            &mut observer,
        ),
        Err(DxfError::TransactionPostImageMismatch { .. })
    ));
    observer.finish()?;
    assert!(!output.exists());
    Ok(())
}

#[test]
fn strict_reparse_failure_removes_the_destination() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let output = directory.path().join("strict-reparse-failure.dxf");
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let plan = edit_plan(view)?;
    let mut observer = FinalTamperObserver::new(&output, b"EOF", b"EOG");

    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            view,
            &output,
            DxfResourceProfile::Safe,
            &token(),
            &mut observer,
        ),
        Err(DxfError::MissingAsciiEof { .. })
    ));
    observer.finish()?;
    assert!(!output.exists());
    Ok(())
}

#[test]
fn source_existing_destination_and_cancellation_fail_without_partial_output()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityEditWriteJournal>();
    assert_send_sync::<DxfEntityEditWriteOutcome>();

    let directory = TestDirectory::new()?;
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let plan = edit_plan(view)?;
    let mut noop = NoopDxfReadObserver;

    let existing = directory.path().join("existing.dxf");
    fs::write(&existing, b"KEEP")?;
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            view,
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

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let mismatch = directory.path().join("source-mismatch.dxf");
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            DxfRawDocumentView::from(&other),
            &mismatch,
            DxfResourceProfile::Safe,
            &token(),
            &mut noop,
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(!mismatch.exists());

    let cancelled = directory.path().join("cancelled.dxf");
    let mut cancel_after_start = |progress: DxfReadProgress| {
        if progress.processed_bytes() == 0 {
            DxfReadControl::Continue
        } else {
            DxfReadControl::Cancel
        }
    };
    assert!(matches!(
        plan.write_reparse_verify_and_journal_to_new_file(
            view,
            &cancelled,
            DxfResourceProfile::Safe,
            &token(),
            &mut cancel_after_start,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!cancelled.exists());
    Ok(())
}

fn verify_write(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
    output: &Path,
) -> Result<(), Box<dyn Error>> {
    let original = bytes.to_vec();
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let mut progress = Vec::new();
    let mut observer = |value: DxfReadProgress| {
        progress.push((value.processed_bytes(), value.total_bytes()));
        DxfReadControl::Continue
    };
    let journal = match format {
        DxfRawDocumentFormat::Ascii => {
            let document = open_ascii(&source)?;
            let view = DxfRawDocumentView::from(&document);
            let plan = edit_plan(view)?;
            written(plan.write_reparse_verify_and_journal_to_new_file(
                view,
                output,
                DxfResourceProfile::Safe,
                &token(),
                &mut observer,
            )?)?
        }
        DxfRawDocumentFormat::Binary => {
            let document = open_binary(&source)?;
            let view = DxfRawDocumentView::from(&document);
            let plan = edit_plan(view)?;
            written(plan.write_reparse_verify_and_journal_to_new_file(
                view,
                output,
                DxfResourceProfile::Safe,
                &token(),
                &mut observer,
            )?)?
        }
        _ => return Err(io::Error::other("unsupported test format").into()),
    };
    assert_eq!(bytes, original);
    assert!(!progress.is_empty());
    assert!(progress.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    verify_journal(bytes, format, output, journal)
}

fn verify_journal(
    source: &[u8],
    format: DxfRawDocumentFormat,
    output: &Path,
    journal: DxfEntityEditWriteJournal,
) -> Result<(), Box<dyn Error>> {
    let output_bytes = fs::read(output)?;
    let output_source = DxfMemorySource::new(&output_bytes, DxfResourceProfile::Safe)?;
    let output_id = match format {
        DxfRawDocumentFormat::Ascii => open_ascii(&output_source)?.source_id(),
        DxfRawDocumentFormat::Binary => open_binary(&output_source)?.source_id(),
        _ => return Err(io::Error::other("unsupported test format").into()),
    };
    let write = journal.write_receipt();
    let verification = journal.verification_receipt();
    let debug = format!("{journal:?}");
    assert!(!debug.contains("DASHED"));
    assert_eq!(write.output_id(), output_id);
    assert_eq!(verification.source_id(), write.source_id());
    assert_eq!(verification.post_image_id(), write.output_id());
    assert_eq!(verification.edit_count(), 3);
    assert_eq!(journal.inverse_plan().source_id(), output_id);
    assert_eq!(materialize(&output_bytes, journal.inverse_plan())?, source);

    let (write_part, verification_part, inverse) = journal.into_parts();
    assert_eq!(write_part, write);
    assert_eq!(verification_part, verification);
    assert_eq!(materialize(&output_bytes, &inverse)?, source);
    Ok(())
}

fn edit_plan(view: DxfRawDocumentView<'_>) -> Result<DxfEntityEditPlan, Box<dyn Error>> {
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = evidence.entity_directory().entities()[0].key();
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    applied(session.update(
        key,
        set(
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"NEW"),
        ),
    )?)?;
    applied(session.update(key, reset(DxfEntityField::COLOR))?)?;
    applied(session.update(
        key,
        set(
            DxfEntityField::LINETYPE,
            DxfEntityEditValue::ExactRawText(b"DASHED"),
        ),
    )?)?;
    Ok(session.finish_verifiable()?)
}

fn written(outcome: DxfEntityEditWriteOutcome) -> Result<DxfEntityEditWriteJournal, io::Error> {
    match outcome {
        DxfEntityEditWriteOutcome::Written(journal) => Ok(journal),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn applied(outcome: DxfEntityEditOutcome) -> Result<(), io::Error> {
    match outcome {
        DxfEntityEditOutcome::Applied(_) => Ok(()),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn set(field: DxfEntityField, value: DxfEntityEditValue<'_>) -> DxfEntityPatch<'_> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit { field, value })
}

const fn reset(field: DxfEntityField) -> DxfEntityPatch<'static> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::ResetToDefault { field })
}

fn materialize(bytes: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start = usize::try_from(patch.source_span().start())
            .map_err(|_| io::Error::other("patch start"))?;
        let end = usize::try_from(patch.source_span().end())
            .map_err(|_| io::Error::other("patch end"))?;
        output.extend_from_slice(bytes.get(cursor..start).ok_or(io::Error::other("source"))?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or(io::Error::other("replacement"))?,
        );
        cursor = end;
    }
    output.extend_from_slice(bytes.get(cursor..).ok_or(io::Error::other("tail"))?);
    Ok(output)
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
    let mut matches = bytes
        .windows(from.len())
        .enumerate()
        .filter_map(
            |(index, value)| {
                if value == from { Some(index) } else { None }
            },
        );
    let start = matches.next().ok_or(io::Error::other("tamper source"))?;
    if matches.next().is_some() {
        return Err(io::Error::other("ambiguous tamper source"));
    }
    bytes
        .get_mut(start..start + from.len())
        .ok_or(io::Error::other("tamper range"))?
        .copy_from_slice(to);
    fs::write(path, bytes)
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Double(f64),
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
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
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    groups.extend([
        (8, Value::Text(b"OLD")),
        (62, Value::Int16(7)),
        (1, Value::Text(b"KEEP")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([
        (10, Value::Double(0.0)),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("unsupported test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.extend_from_slice(b"\r\n");
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

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Result<Self, io::Error> {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "seacad-entity-edit-write-{}-{id}",
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

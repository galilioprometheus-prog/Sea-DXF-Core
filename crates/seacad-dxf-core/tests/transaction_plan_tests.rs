use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadOptions, DxfResource, DxfResourceProfile, DxfTransactionPlan,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_ascii_binary_transaction_plan_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_plan = build_version_plan(DxfRawDocumentView::from(&ascii), version.code())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_plan = build_version_plan(DxfRawDocumentView::from(&binary), version.code())?;

        assert_eq!(ascii_plan.format(), DxfRawDocumentFormat::Ascii);
        assert_eq!(binary_plan.format(), DxfRawDocumentFormat::Binary);
        ascii_plan.validate_source_precondition(DxfRawDocumentView::from(&ascii))?;
        binary_plan.validate_source_precondition(DxfRawDocumentView::from(&binary))?;
        for plan in [&ascii_plan, &binary_plan] {
            assert_eq!(plan.patches().len(), 2);
            assert_eq!(plan.projected_len(), plan.source_len() + 1);
            assert_eq!(
                plan.replacement_bytes_for_patch_ordinal(0),
                Some(b"X".as_slice())
            );
            assert_eq!(
                plan.replacement_bytes_for_patch_ordinal(1),
                Some(b"AC1032".as_slice())
            );
            assert_eq!(plan.inverse_bytes_for_patch_ordinal(0), Some([].as_slice()));
            assert_eq!(
                plan.inverse_bytes_for_patch_ordinal(1),
                Some(version.code().as_bytes())
            );
            assert!(plan.replacement_bytes_for_patch_ordinal(2).is_none());
            assert!(plan.inverse_bytes_for_patch_ordinal(2).is_none());
        }
    }
    Ok(())
}

#[test]
fn source_order_inverse_capture_and_boundary_insertions_are_deterministic()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let first = view
        .group(0)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let last = view
        .group(5)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(last, b"", &DxfCancellationToken::default())?;
    builder.replace_raw_span(first, b"START", &DxfCancellationToken::default())?;
    let at_first_end = ByteSpan::new(first.end(), first.end()).ok_or_else(invalid_test_data)?;
    builder.replace_raw_span(at_first_end, b"+", &DxfCancellationToken::default())?;
    let plan = builder.finish(&DxfCancellationToken::default())?;
    assert_eq!(
        plan.patches()
            .iter()
            .map(|patch| patch.source_span())
            .collect::<Vec<_>>(),
        [first, at_first_end, last]
    );
    assert_eq!(
        plan.inverse_bytes_for_patch_ordinal(0),
        Some(b"SECTION".as_slice())
    );
    assert_eq!(plan.inverse_bytes_for_patch_ordinal(1), Some([].as_slice()));
    assert_eq!(
        plan.inverse_bytes_for_patch_ordinal(2),
        Some(b"EOF".as_slice())
    );
    assert_eq!(
        plan.replacement_bytes_for_patch_ordinal(0),
        Some(b"START".as_slice())
    );
    assert_eq!(
        plan.replacement_bytes_for_patch_ordinal(1),
        Some(b"+".as_slice())
    );
    assert_eq!(
        plan.replacement_bytes_for_patch_ordinal(2),
        Some([].as_slice())
    );
    assert_eq!(plan.projected_len(), view.source_len() - 4);
    plan.validate_source_precondition(view)?;
    Ok(())
}

#[test]
fn conflicts_bounds_limits_and_cancellation_leave_builder_unchanged() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let span = view
        .group(3)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(span, b"AC1027", &DxfCancellationToken::default())?;
    let expected_count = builder.patch_count();
    let expected_len = builder.projected_len();

    let overlap = ByteSpan::new(span.start() + 1, span.end()).ok_or_else(invalid_test_data)?;
    assert!(matches!(
        builder.replace_raw_span(overlap, b"x", &DxfCancellationToken::default()),
        Err(DxfError::TransactionPatchConflict { .. })
    ));
    let inside = ByteSpan::new(span.start() + 1, span.start() + 1).ok_or_else(invalid_test_data)?;
    assert!(matches!(
        builder.replace_raw_span(inside, b"x", &DxfCancellationToken::default()),
        Err(DxfError::TransactionPatchConflict { .. })
    ));
    let outside =
        ByteSpan::new(view.source_len(), view.source_len() + 1).ok_or_else(invalid_test_data)?;
    assert!(matches!(
        builder.replace_raw_span(outside, b"x", &DxfCancellationToken::default()),
        Err(DxfError::TransactionSpanOutOfBounds { .. })
    ));
    let oversized = vec![0_u8; 1024 * 1024 + 1];
    let end = ByteSpan::new(view.source_len(), view.source_len()).ok_or_else(invalid_test_data)?;
    assert!(matches!(
        builder.replace_raw_span(end, &oversized, &DxfCancellationToken::default()),
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::ValueBytes,
            ..
        })
    ));
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        builder.replace_raw_span(end, b"x", &cancelled),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(builder.patch_count(), expected_count);
    assert_eq!(builder.projected_len(), expected_len);
    assert!(matches!(
        builder.finish(&cancelled),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

#[test]
fn duplicate_insertions_source_mismatch_noop_and_debug_redaction_hold() -> Result<(), Box<dyn Error>>
{
    assert_copy::<seacad_dxf_core::DxfTransactionByteRange>();
    assert_copy::<seacad_dxf_core::DxfTransactionPatch>();
    assert_send_sync::<DxfTransactionPlan>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let point = ByteSpan::new(0, 0).ok_or_else(invalid_test_data)?;
    let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(point, b"SECRET", &DxfCancellationToken::default())?;
    assert!(matches!(
        builder.replace_raw_span(point, b"second", &DxfCancellationToken::default()),
        Err(DxfError::TransactionPatchConflict { .. })
    ));
    let noop = ByteSpan::new(view.source_len(), view.source_len()).ok_or_else(invalid_test_data)?;
    builder.replace_raw_span(noop, b"", &DxfCancellationToken::default())?;
    assert_eq!(builder.patch_count(), 1);
    let plan = builder.finish(&DxfCancellationToken::default())?;
    let debug = format!("{plan:?}");
    assert!(!debug.contains("SECRET"));

    let other_bytes = ascii_fixture("AC1027");
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        plan.validate_source_precondition(DxfRawDocumentView::from(&other)),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn inverse_capture_io_failure_does_not_admit_a_patch() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = ToggleSource::new(&bytes);
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let span = view
        .group(3)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    let expected_len = builder.projected_len();
    source.fail_reads();
    assert!(matches!(
        builder.replace_raw_span(span, b"AC1027", &DxfCancellationToken::default()),
        Err(DxfError::Io {
            operation: seacad_dxf_core::DxfIoOperation::Read,
            ..
        })
    ));
    assert_eq!(builder.patch_count(), 0);
    assert_eq!(builder.projected_len(), expected_len);
    Ok(())
}

fn build_version_plan(
    view: DxfRawDocumentView<'_>,
    original: &str,
) -> Result<DxfTransactionPlan, DxfError> {
    let version = view
        .group(3)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(version, b"AC1032", &DxfCancellationToken::default())?;
    assert_eq!(original.len(), 6);
    let insertion =
        ByteSpan::new(version.start(), version.start()).ok_or_else(invalid_test_data)?;
    builder.replace_raw_span(insertion, b"X", &DxfCancellationToken::default())?;
    builder.finish(&DxfCancellationToken::default())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nEOF\n").into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
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
) -> io::Result<()> {
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
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

struct ToggleSource<'a> {
    bytes: &'a [u8],
    fail: AtomicBool,
}

impl<'a> ToggleSource<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            fail: AtomicBool::new(false),
        }
    }

    fn fail_reads(&self) {
        self.fail.store(true, Ordering::Relaxed);
    }
}

impl DxfByteSource for ToggleSource<'_> {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        if self.fail.load(Ordering::Relaxed) {
            return Err(DxfError::from_io(
                seacad_dxf_core::DxfIoOperation::Read,
                &io::Error::other("synthetic read failure"),
            ));
        }
        if destination.is_empty() || offset >= self.len() {
            return Ok(0);
        }
        let start = usize::try_from(offset).map_err(|_| invalid_test_data())?;
        let count = (self.bytes.len() - start).min(destination.len());
        destination[..count].copy_from_slice(&self.bytes[start..start + count]);
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

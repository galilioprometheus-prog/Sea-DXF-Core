use std::{error::Error, io};

use seacad_dxf_core::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_round_trips_through_an_executable_inverse_plan()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), b"ABCDEFGH");
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_view = DxfRawDocumentView::from(&ascii);
        let ascii_forward = plan_for_payload(ascii_view)?;
        let ascii_post_bytes = apply_plan(&ascii_bytes, &ascii_forward)?;
        let ascii_post_source = DxfMemorySource::new(&ascii_post_bytes, DxfResourceProfile::Safe)?;
        let ascii_post = open_ascii(&ascii_post_source)?;
        assert_inverse_round_trip(
            &ascii_bytes,
            ascii_view,
            &ascii_forward,
            &ascii_post_bytes,
            DxfRawDocumentView::from(&ascii_post),
        )?;

        let binary_bytes = binary_fixture(version, b"ABCDEFGH")?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_view = DxfRawDocumentView::from(&binary);
        let binary_forward = plan_for_payload(binary_view)?;
        let binary_post_bytes = apply_plan(&binary_bytes, &binary_forward)?;
        let binary_post_source =
            DxfMemorySource::new(&binary_post_bytes, DxfResourceProfile::Safe)?;
        let binary_post = open_binary(&binary_post_source)?;
        assert_inverse_round_trip(
            &binary_bytes,
            binary_view,
            &binary_forward,
            &binary_post_bytes,
            DxfRawDocumentView::from(&binary_post),
        )?;
    }
    Ok(())
}

#[test]
fn adjacent_deletions_coalesce_and_redo_restores_the_exact_post_image() -> Result<(), Box<dyn Error>>
{
    let source_bytes = ascii_fixture("AC1032", b"ABCDEFGH");
    let source = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let source_document = open_ascii(&source)?;
    let source_view = DxfRawDocumentView::from(&source_document);
    let payload = source_view
        .group(4)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = source_view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(
        point(payload.start())?,
        b"X",
        &DxfCancellationToken::default(),
    )?;
    builder.replace_raw_span(
        subspan(payload, 2, 1)?,
        b"",
        &DxfCancellationToken::default(),
    )?;
    builder.replace_raw_span(
        subspan(payload, 3, 1)?,
        b"",
        &DxfCancellationToken::default(),
    )?;
    builder.replace_raw_span(
        subspan(payload, 5, 1)?,
        b"YY",
        &DxfCancellationToken::default(),
    )?;
    builder.replace_raw_span(
        point(payload.end())?,
        b"Z",
        &DxfCancellationToken::default(),
    )?;
    let forward = builder.finish(&DxfCancellationToken::default())?;
    assert_eq!(forward.patches().len(), 5);
    let post_bytes = apply_plan(&source_bytes, &forward)?;
    assert!(post_bytes.windows(9).any(|window| window == b"XABEYYGHZ"));
    let post_source = DxfMemorySource::new(&post_bytes, DxfResourceProfile::Safe)?;
    let post_document = open_ascii(&post_source)?;
    let post_view = DxfRawDocumentView::from(&post_document);

    let inverse = forward.materialize_inverse_plan(
        source_view,
        post_view,
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?;
    assert_eq!(inverse.patches().len(), 4);
    assert_eq!(inverse.source_id(), post_view.source_id());
    assert_eq!(inverse.projected_len(), source_view.source_len());
    assert!(
        inverse
            .patches()
            .iter()
            .any(|patch| patch.source_span().is_empty()
                && inverse.replacement_bytes_for_patch_ordinal(patch.ordinal())
                    == Some(b"CD".as_slice()))
    );
    let restored_bytes = apply_plan(&post_bytes, &inverse)?;
    assert_eq!(restored_bytes, source_bytes);

    let redo = inverse.materialize_inverse_plan(
        post_view,
        source_view,
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?;
    assert_eq!(apply_plan(&source_bytes, &redo)?, post_bytes);
    Ok(())
}

#[test]
fn length_content_source_and_cancellation_failures_remain_typed() -> Result<(), Box<dyn Error>> {
    let source_bytes = ascii_fixture("AC1032", b"ABCDEFGH");
    let source = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let source_document = open_ascii(&source)?;
    let source_view = DxfRawDocumentView::from(&source_document);
    let payload = source_view
        .group(4)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = source_view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(payload, b"GOODGOOD", &DxfCancellationToken::default())?;
    let plan = builder.finish(&DxfCancellationToken::default())?;

    let wrong_content_bytes = ascii_fixture("AC1032", b"EVILEVIL");
    let wrong_content_source =
        DxfMemorySource::new(&wrong_content_bytes, DxfResourceProfile::Safe)?;
    let wrong_content = open_ascii(&wrong_content_source)?;
    assert!(matches!(
        plan.materialize_inverse_plan(
            source_view,
            DxfRawDocumentView::from(&wrong_content),
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        ),
        Err(DxfError::TransactionPostImageMismatch { .. })
    ));

    let wrong_length_bytes = ascii_fixture("AC1032", b"TOO-LONG!");
    let wrong_length_source = DxfMemorySource::new(&wrong_length_bytes, DxfResourceProfile::Safe)?;
    let wrong_length = open_ascii(&wrong_length_source)?;
    assert!(matches!(
        plan.materialize_inverse_plan(
            source_view,
            DxfRawDocumentView::from(&wrong_length),
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        ),
        Err(DxfError::TransactionPostImageLengthMismatch { .. })
    ));

    let other_source_bytes = ascii_fixture("AC1027", b"ABCDEFGH");
    let other_source = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        plan.materialize_inverse_plan(
            DxfRawDocumentView::from(&other),
            DxfRawDocumentView::from(&wrong_content),
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        plan.materialize_inverse_plan(
            source_view,
            DxfRawDocumentView::from(&wrong_content),
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

#[test]
fn verification_crosses_fixed_chunks_and_reports_the_exact_changed_byte()
-> Result<(), Box<dyn Error>> {
    let payload = vec![b'A'; 9_000];
    let source_bytes = ascii_fixture("AC1032", &payload);
    let source = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let source_document = open_ascii(&source)?;
    let source_view = DxfRawDocumentView::from(&source_document);
    let payload_span = source_view
        .group(4)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = source_view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(
        subspan(payload_span, 8_999, 1)?,
        b"Z",
        &DxfCancellationToken::default(),
    )?;
    let plan = builder.finish(&DxfCancellationToken::default())?;
    let correct_bytes = apply_plan(&source_bytes, &plan)?;
    let correct_source = DxfMemorySource::new(&correct_bytes, DxfResourceProfile::Safe)?;
    let correct = open_ascii(&correct_source)?;
    plan.materialize_inverse_plan(
        source_view,
        DxfRawDocumentView::from(&correct),
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?;

    let mut wrong_bytes = correct_bytes;
    let changed_offset =
        usize::try_from(payload_span.start() + 5_000).map_err(|_| invalid_test_data())?;
    wrong_bytes[changed_offset] = b'B';
    let wrong_source = DxfMemorySource::new(&wrong_bytes, DxfResourceProfile::Safe)?;
    let wrong = open_ascii(&wrong_source)?;
    match plan.materialize_inverse_plan(
        source_view,
        DxfRawDocumentView::from(&wrong),
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    ) {
        Err(DxfError::TransactionPostImageMismatch { span }) => {
            assert_eq!(span.start(), changed_offset as u64);
            assert_eq!(span.len(), 1);
        }
        other => return Err(format!("unexpected result: {other:?}").into()),
    }
    Ok(())
}

#[test]
fn same_length_physical_format_mismatch_is_typed() -> Result<(), Box<dyn Error>> {
    let source_bytes = ascii_fixture("AC1032", &vec![b'A'; 256]);
    let source = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let source_document = open_ascii(&source)?;
    let source_view = DxfRawDocumentView::from(&source_document);
    let builder = source_view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    let plan = builder.finish(&DxfCancellationToken::default())?;

    let binary_base = binary_fixture(DxfAcadVersion::Ac1032, b"")?;
    let payload_len = source_bytes
        .len()
        .checked_sub(binary_base.len())
        .ok_or_else(invalid_test_data)?;
    let binary_bytes = binary_fixture(DxfAcadVersion::Ac1032, &vec![b'B'; payload_len])?;
    assert_eq!(binary_bytes.len(), source_bytes.len());
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    assert!(matches!(
        plan.materialize_inverse_plan(
            source_view,
            DxfRawDocumentView::from(&binary),
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        ),
        Err(DxfError::TransactionPostImageMismatch { span }) if span.is_empty()
    ));
    Ok(())
}

#[test]
fn coalesced_inverse_and_redo_remain_closed_above_one_value_limit() -> Result<(), Box<dyn Error>> {
    let large_value = vec![b'A'; 600_000];
    let source_bytes = ascii_two_value_fixture("AC1032", &large_value, &large_value);
    let source = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let source_document = open_ascii(&source)?;
    let source_view = DxfRawDocumentView::from(&source_document);
    let first = source_view
        .group(4)
        .ok_or_else(invalid_test_data)?
        .full_span();
    let second = source_view
        .group(5)
        .ok_or_else(invalid_test_data)?
        .full_span();
    assert_eq!(first.end(), second.start());
    let mut builder = source_view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(first, b"", &DxfCancellationToken::default())?;
    builder.replace_raw_span(second, b"", &DxfCancellationToken::default())?;
    let forward = builder.finish(&DxfCancellationToken::default())?;
    let post_bytes = apply_plan(&source_bytes, &forward)?;
    let post_source = DxfMemorySource::new(&post_bytes, DxfResourceProfile::Safe)?;
    let post_document = open_ascii(&post_source)?;
    let post_view = DxfRawDocumentView::from(&post_document);

    let inverse = forward.materialize_inverse_plan(
        source_view,
        post_view,
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?;
    assert_eq!(inverse.patches().len(), 1);
    assert!(
        inverse
            .replacement_bytes_for_patch_ordinal(0)
            .ok_or_else(invalid_test_data)?
            .len()
            > 1024 * 1024
    );
    assert_eq!(apply_plan(&post_bytes, &inverse)?, source_bytes);

    let redo = inverse.materialize_inverse_plan(
        post_view,
        source_view,
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?;
    assert_eq!(apply_plan(&source_bytes, &redo)?, post_bytes);
    Ok(())
}

fn plan_for_payload(source_view: DxfRawDocumentView<'_>) -> Result<DxfTransactionPlan, DxfError> {
    let payload = source_view
        .group(4)
        .ok_or_else(invalid_test_data)?
        .value_payload_span();
    let mut builder = source_view.transaction_plan_builder(DxfResourceProfile::Safe)?;
    builder.replace_raw_span(payload, b"EDITED!!", &DxfCancellationToken::default())?;
    builder.finish(&DxfCancellationToken::default())
}

fn assert_inverse_round_trip(
    source_bytes: &[u8],
    source_view: DxfRawDocumentView<'_>,
    forward: &DxfTransactionPlan,
    post_bytes: &[u8],
    post_view: DxfRawDocumentView<'_>,
) -> Result<(), Box<dyn Error>> {
    let inverse = forward.materialize_inverse_plan(
        source_view,
        post_view,
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?;
    assert_eq!(apply_plan(post_bytes, &inverse)?, source_bytes);
    Ok(())
}

fn apply_plan(source: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, DxfError> {
    let capacity = usize::try_from(plan.projected_len()).map_err(|_| invalid_test_data())?;
    let mut output = Vec::with_capacity(capacity);
    let mut cursor = 0_usize;
    for patch in plan.patches().iter().copied() {
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
    if output.len() != capacity {
        return Err(invalid_test_data());
    }
    Ok(output)
}

fn point(offset: u64) -> Result<ByteSpan, DxfError> {
    ByteSpan::new(offset, offset).ok_or_else(invalid_test_data)
}

fn subspan(parent: ByteSpan, relative_start: u64, len: u64) -> Result<ByteSpan, DxfError> {
    let start = parent
        .start()
        .checked_add(relative_start)
        .ok_or_else(invalid_test_data)?;
    let span = ByteSpan::from_start_and_len(start, len).ok_or_else(invalid_test_data)?;
    if span.end() > parent.end() {
        return Err(invalid_test_data());
    }
    Ok(span)
}

fn ascii_fixture(version: &str, payload: &[u8]) -> Vec<u8> {
    let mut bytes = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n1\n").into_bytes();
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(b"\n0\nENDSEC\n0\nEOF\n");
    bytes
}

fn ascii_two_value_fixture(version: &str, first: &[u8], second: &[u8]) -> Vec<u8> {
    let mut bytes = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n1\n").into_bytes();
    bytes.extend_from_slice(first);
    bytes.extend_from_slice(b"\n1\n");
    bytes.extend_from_slice(second);
    bytes.extend_from_slice(b"\n0\nENDSEC\n0\nEOF\n");
    bytes
}

fn binary_fixture(version: DxfAcadVersion, payload: &[u8]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
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

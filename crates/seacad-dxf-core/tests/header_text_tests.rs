use std::{error::Error, io, num::NonZeroU64};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfError, DxfHeaderTextDirectory, DxfHeaderTextEntry, DxfHeaderTextIssue,
    DxfHeaderTextValue, DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, DxfTextDecodeStatus, NoopDxfReadObserver,
};

const TEXT_FIELDS: &[(u64, &str, &str, i16)] = &[
    (1, "acadver", "$ACADVER", 1),
    (7, "dwgcodepage", "$DWGCODEPAGE", 3),
    (187, "celtype", "$CELTYPE", 6),
    (188, "clayer", "$CLAYER", 8),
    (189, "cmlstyle", "$CMLSTYLE", 2),
    (190, "dimapost", "$DIMAPOST", 1),
    (191, "dimblk", "$DIMBLK", 1),
    (192, "dimblk1", "$DIMBLK1", 1),
    (193, "dimblk2", "$DIMBLK2", 1),
    (194, "dimldrblk", "$DIMLDRBLK", 1),
    (195, "dimpost", "$DIMPOST", 1),
    (196, "dimstyle", "$DIMSTYLE", 2),
    (197, "dimtxsty", "$DIMTXSTY", 7),
];

#[test]
fn ascii_and_binary_preserve_exact_text_and_schema_order() -> Result<(), Box<dyn Error>> {
    let ascii_bytes = ascii_fixture(standard_text_body());
    let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
    let ascii = open_ascii(&ascii_source)?;
    let ascii_directory = ascii.header_text_directory(&DxfCancellationToken::default())?;
    assert_directory_shape(&ascii_directory, ascii.source_id())?;
    assert_decoded(
        explicit(&ascii_directory, "clayer")?,
        DxfRawDocumentView::from(&ascii),
        b" Layer/../MiXeD ",
    )?;

    let binary_bytes = binary_fixture()?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    let binary_directory = binary.header_text_directory(&DxfCancellationToken::default())?;
    assert_directory_shape(&binary_directory, binary.source_id())?;
    assert_decoded(
        explicit(&binary_directory, "clayer")?,
        DxfRawDocumentView::from(&binary),
        b" Layer/../MiXeD ",
    )?;
    Ok(())
}

#[test]
fn structural_failures_remain_typed_and_source_anchored() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            "9\n$CELTYPE\n7\nWrong family\n",
            DxfHeaderTextIssue::InvalidGroupCode(code(7)?),
        ),
        (
            "9\n$CELTYPE\n9\n$CLAYER\n8\nLayer\n",
            DxfHeaderTextIssue::MissingValue,
        ),
        (
            "9\n$CELTYPE\n6\nA\n6\nB\n",
            DxfHeaderTextIssue::MultipleValueGroups {
                group_count: NonZeroU64::new(2).ok_or(io::Error::other("count"))?,
            },
        ),
        (
            "9\n$CELTYPE\n6\nA\n9\n$CELTYPE\n6\nB\n",
            DxfHeaderTextIssue::MultipleVariables {
                occurrence_count: NonZeroU64::new(2).ok_or(io::Error::other("count"))?,
            },
        ),
    ];
    for (body, expected) in cases {
        let bytes = ascii_fixture(body);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let directory = document.header_text_directory(&DxfCancellationToken::default())?;
        let value = directory
            .entry("celtype")
            .ok_or(io::Error::other("missing CELTYPE entry"))?
            .value();
        assert_eq!(value.state(), DxfSemanticValueState::Invalid);
        assert_eq!(value.invalid_issue(), Some(&expected));
        assert!(value.raw_provenance().is_some());
    }
    Ok(())
}

#[test]
fn absence_cancellation_and_source_identity_fail_closed() -> Result<(), Box<dyn Error>> {
    let first_bytes = ascii_fixture("9\n$CLAYER\n8\nFirst\n");
    let first_source = DxfMemorySource::new(&first_bytes, DxfResourceProfile::Safe)?;
    let first = open_ascii(&first_source)?;
    let first_directory = first.header_text_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        first_directory
            .entry("celtype")
            .ok_or(io::Error::other("missing CELTYPE entry"))?
            .value()
            .state(),
        DxfSemanticValueState::Absent
    );

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        first.header_text_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let second_bytes = ascii_fixture("9\n$CLAYER\n8\nSecond\n");
    let second_source = DxfMemorySource::new(&second_bytes, DxfResourceProfile::Safe)?;
    let second = open_ascii(&second_source)?;
    let value = explicit(&first_directory, "clayer")?;
    let mut destination = [0_u8; 16];
    assert!(matches!(
        value.decode_to_utf8_without_replacement(
            DxfRawDocumentView::from(&second),
            &mut destination,
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn public_metadata_is_copy_send_sync_and_debug_redacted() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfHeaderTextEntry>();
    assert_copy::<DxfHeaderTextValue>();
    assert_send_sync::<DxfHeaderTextDirectory>();
    let bytes = ascii_fixture("9\n$CLAYER\n8\nsecret-layer\n");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.header_text_directory(&DxfCancellationToken::default())?;
    let debug = format!("{directory:?}");
    assert!(debug.contains("text_field_count"));
    assert!(!debug.contains("secret-layer"));
    Ok(())
}

fn assert_directory_shape(
    directory: &DxfHeaderTextDirectory,
    source_id: seacad_dxf_core::DxfSourceId,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), source_id);
    assert_eq!(directory.schema_version(), "dxf.v1");
    assert_eq!(directory.entries().len(), TEXT_FIELDS.len());
    for (entry, (ordinal, id, name, group_code)) in
        directory.entries().iter().zip(TEXT_FIELDS.iter().copied())
    {
        assert_eq!(entry.schema_ordinal(), ordinal);
        assert_eq!(entry.schema_field_id(), id);
        assert_eq!(entry.dxf_name(), name);
        assert_eq!(entry.group_code(), group_code);
        assert_eq!(entry.value().state(), DxfSemanticValueState::Explicit);
        assert_eq!(directory.entry(id), Some(entry));
        assert_eq!(directory.entry_at_schema_ordinal(ordinal), Some(entry));
    }
    assert!(directory.entry_at_schema_ordinal(0).is_none());
    assert!(directory.entry_at_schema_ordinal(10).is_none());
    Ok(())
}

fn explicit(directory: &DxfHeaderTextDirectory, id: &str) -> Result<DxfHeaderTextValue, io::Error> {
    directory
        .entry(id)
        .and_then(|entry| entry.value().value())
        .copied()
        .ok_or(io::Error::other("missing explicit text"))
}

fn assert_decoded(
    value: DxfHeaderTextValue,
    document: DxfRawDocumentView<'_>,
    expected: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut destination = vec![0_u8; expected.len()];
    let receipt = value.decode_to_utf8_without_replacement(document, &mut destination)?;
    let result = receipt
        .decode_result()
        .ok_or(io::Error::other("encoding unavailable"))?;
    assert_eq!(result.status(), DxfTextDecodeStatus::Complete);
    assert_eq!(result.written(), expected.len());
    assert_eq!(destination, expected);
    assert_eq!(receipt.value_span(), value.value_span());
    Ok(())
}

fn standard_text_body() -> &'static str {
    "9\n$DWGCODEPAGE\n3\nUTF-8\n9\n$CELTYPE\n6\nDash Dot\n9\n$CLAYER\n8\n Layer/../MiXeD \n9\n$CMLSTYLE\n2\nMLine Style\n9\n$DIMAPOST\n1\n[] mm\n9\n$DIMBLK\n1\n_ARCHTICK\n9\n$DIMBLK1\n1\nArrow One\n9\n$DIMBLK2\n1\nArrow Two\n9\n$DIMLDRBLK\n1\nLeader Arrow\n9\n$DIMPOST\n1\n<> units\n9\n$DIMSTYLE\n2\nDim Style\n9\n$DIMTXSTY\n7\nText Style\n"
}

fn ascii_fixture(body: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n{body}0\nENDSEC\n0\nEOF\n").into_bytes()
}

fn binary_fixture() -> Result<Vec<u8>, io::Error> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (group_code, value) in [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, b"AC1032"),
    ] {
        push_binary_string(&mut bytes, group_code, value);
    }
    for (name, group_code, value) in [
        (b"$DWGCODEPAGE".as_slice(), 3_i16, b"UTF-8".as_slice()),
        (b"$CELTYPE", 6, b"Dash Dot"),
        (b"$CLAYER", 8, b" Layer/../MiXeD "),
        (b"$CMLSTYLE", 2, b"MLine Style"),
        (b"$DIMAPOST", 1, b"[] mm"),
        (b"$DIMBLK", 1, b"_ARCHTICK"),
        (b"$DIMBLK1", 1, b"Arrow One"),
        (b"$DIMBLK2", 1, b"Arrow Two"),
        (b"$DIMLDRBLK", 1, b"Leader Arrow"),
        (b"$DIMPOST", 1, b"<> units"),
        (b"$DIMSTYLE", 2, b"Dim Style"),
        (b"$DIMTXSTY", 7, b"Text Style"),
    ] {
        push_binary_string(&mut bytes, 9, name);
        push_binary_string(&mut bytes, group_code, value);
    }
    for (group_code, value) in [(0_i16, b"ENDSEC".as_slice()), (0, b"EOF")] {
        push_binary_string(&mut bytes, group_code, value);
    }
    assert_eq!(version.code(), "AC1032");
    Ok(bytes)
}

fn push_binary_string(bytes: &mut Vec<u8>, group_code: i16, value: &[u8]) {
    bytes.extend_from_slice(&group_code.to_le_bytes());
    bytes.extend_from_slice(value);
    bytes.push(0);
}

fn open_ascii<'a>(source: &'a DxfMemorySource<'_>) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a DxfMemorySource<'_>) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn code(value: i16) -> Result<seacad_dxf_core::DxfGroupCode, io::Error> {
    seacad_dxf_core::DxfGroupCode::new(value).ok_or(io::Error::other("group code"))
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

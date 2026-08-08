use std::{error::Error, io};

use seacad_dxf_core::{
    ByteSpan, DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfError, DxfLegacyCodePage, DxfMemorySource, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadOptions, DxfResource, DxfResourceProfile, DxfTextDecodeStatus,
    DxfTextEncodeStatus, DxfTextEncodingResolution, DxfTextTranscodeIssue, DxfTextTranscodePlan,
    NoopDxfReadObserver,
};

#[test]
fn utf8_and_windows1252_transcode_both_directions_for_all_format_pairs()
-> Result<(), Box<dyn Error>> {
    for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            assert_pair(
                source_format,
                DxfAcadVersion::Ac1021,
                None,
                "é".as_bytes(),
                destination_format,
                DxfAcadVersion::Ac1018,
                Some(b"ANSI_1252"),
                b"\xE9",
            )?;
            assert_pair(
                source_format,
                DxfAcadVersion::Ac1018,
                Some(b"ANSI_1252"),
                b"\xE9",
                destination_format,
                DxfAcadVersion::Ac1021,
                None,
                "é".as_bytes(),
            )?;
        }
    }
    Ok(())
}

#[test]
fn same_and_cross_legacy_transcodes_are_round_trip_exact() -> Result<(), Box<dyn Error>> {
    assert_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_932"),
        b"\x82\xA0",
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_932"),
        b"\x82\xA0",
    )?;
    assert_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1252"),
        b"ASCII",
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1251"),
        b"ASCII",
    )?;
    assert_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1021,
        None,
        b"",
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1258"),
        b"",
    )?;
    Ok(())
}

#[test]
fn unmappable_unavailable_and_malformed_text_fail_typed_without_replacement()
-> Result<(), Box<dyn Error>> {
    let modern_chinese = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1021,
        None,
        "你".as_bytes(),
    )?;
    let legacy_1252 = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1252"),
        b"",
    )?;
    let modern_source = DxfMemorySource::new(&modern_chinese, DxfResourceProfile::Safe)?;
    let legacy_storage = DxfMemorySource::new(&legacy_1252, DxfResourceProfile::Safe)?;
    let modern = open(&modern_source, DxfRawDocumentFormat::Ascii)?;
    let legacy = open(&legacy_storage, DxfRawDocumentFormat::Ascii)?;
    let span = value_span(modern.view())?;
    assert!(matches!(
        modern.view().transcode_text_span_to(
            span,
            legacy.view(),
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfTextTranscodeIssue::DestinationEncode {
            status: DxfTextEncodeStatus::Unmappable { utf8_len: 3, .. },
        })
    ));

    let modern_korean = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1021,
        None,
        "가".as_bytes(),
    )?;
    let johab = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1361"),
        b"",
    )?;
    let korean_storage = DxfMemorySource::new(&modern_korean, DxfResourceProfile::Safe)?;
    let johab_storage = DxfMemorySource::new(&johab, DxfResourceProfile::Safe)?;
    let korean = open(&korean_storage, DxfRawDocumentFormat::Ascii)?;
    let johab_document = open(&johab_storage, DxfRawDocumentFormat::Ascii)?;
    assert!(matches!(
        korean.view().transcode_text_span_to(
            value_span(korean.view())?,
            johab_document.view(),
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfTextTranscodeIssue::DestinationEncode {
            status: DxfTextEncodeStatus::Unavailable,
        })
    ));

    let malformed = fixture(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_932"),
        b"\x82",
    )?;
    let modern_empty = fixture(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1021,
        None,
        b"",
    )?;
    let malformed_storage = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
    let modern_empty_storage = DxfMemorySource::new(&modern_empty, DxfResourceProfile::Safe)?;
    let malformed_document = open(&malformed_storage, DxfRawDocumentFormat::Binary)?;
    let modern_destination = open(&modern_empty_storage, DxfRawDocumentFormat::Binary)?;
    assert!(matches!(
        malformed_document.view().transcode_text_span_to(
            value_span(malformed_document.view())?,
            modern_destination.view(),
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfTextTranscodeIssue::SourceDecode {
            status: DxfTextDecodeStatus::Malformed { .. },
        })
    ));
    Ok(())
}

#[test]
fn encoding_resolution_cancellation_identity_and_debug_are_explicit() -> Result<(), Box<dyn Error>>
{
    let source_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1021,
        None,
        b"SECRET_TEXT_VALUE",
    )?;
    let indeterminate_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1018,
        None,
        b"",
    )?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&indeterminate_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    assert!(matches!(
        source.view().transcode_text_span_to(
            value_span(source.view())?,
            destination.view(),
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfTextTranscodeIssue::DestinationEncodingUnavailable {
            resolution: DxfTextEncodingResolution::Indeterminate,
        })
    ));

    let unsupported_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1362"),
        b"A",
    )?;
    let modern_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1021,
        None,
        b"",
    )?;
    let unsupported_storage = DxfMemorySource::new(&unsupported_bytes, DxfResourceProfile::Safe)?;
    let modern_storage = DxfMemorySource::new(&modern_bytes, DxfResourceProfile::Safe)?;
    let unsupported = open(&unsupported_storage, DxfRawDocumentFormat::Ascii)?;
    let modern = open(&modern_storage, DxfRawDocumentFormat::Ascii)?;
    assert!(matches!(
        unsupported.view().transcode_text_span_to(
            value_span(unsupported.view())?,
            modern.view(),
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfTextTranscodeIssue::SourceEncodingUnavailable {
            resolution: DxfTextEncodingResolution::UnsupportedLegacy { .. },
        })
    ));

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        source.view().transcode_text_span_to(
            value_span(source.view())?,
            source.view(),
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));

    let over_limit = ByteSpan::from_start_and_len(
        0,
        DxfResourceProfile::Safe
            .limits()
            .max_value_bytes()
            .saturating_add(1),
    )
    .ok_or_else(|| io::Error::other("bounded hostile span"))?;
    assert!(matches!(
        source.view().transcode_text_span_to(
            over_limit,
            source.view(),
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::ValueBytes,
            ..
        })
    ));

    let plan = planned(source.view().transcode_text_span_to(
        value_span(source.view())?,
        source.view(),
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    assert_eq!(plan.source_id(), source.view().source_id());
    assert_eq!(plan.destination_id(), source.view().source_id());
    assert_eq!(plan.encoded_bytes(), b"SECRET_TEXT_VALUE");
    assert_eq!(plan.source_byte_count(), 17);
    assert_eq!(plan.utf8_byte_count(), 17);
    let debug = format!("{plan:?}");
    assert!(!debug.contains("SECRET_TEXT_VALUE"));
    assert_send_sync::<DxfTextTranscodePlan>();
    assert_copy::<DxfTextTranscodeIssue>();
    assert_copy::<DxfTextEncodeStatus>();
    assert_copy::<DxfLegacyCodePage>();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn assert_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    source_codepage: Option<&[u8]>,
    source_value: &[u8],
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
    destination_codepage: Option<&[u8]>,
    expected: &[u8],
) -> Result<(), Box<dyn Error>> {
    let source_bytes = fixture(source_format, source_version, source_codepage, source_value)?;
    let destination_bytes = fixture(
        destination_format,
        destination_version,
        destination_codepage,
        b"",
    )?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, source_format)?;
    let destination = open(&destination_storage, destination_format)?;
    let span = value_span(source.view())?;
    let plan = planned(source.view().transcode_text_span_to(
        span,
        destination.view(),
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    assert_eq!(plan.source_id(), source.view().source_id());
    assert_eq!(plan.destination_id(), destination.view().source_id());
    assert_eq!(plan.source_span(), span);
    assert_eq!(plan.source_byte_count(), source_value.len() as u64);
    assert_eq!(plan.encoded_bytes(), expected);
    assert_eq!(
        plan.source_encoding(),
        source.view().text_encoding_report().resolution()
    );
    assert_eq!(
        plan.destination_encoding(),
        destination.view().text_encoding_report().resolution()
    );
    Ok(())
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    codepage: Option<&[u8]>,
    value: &[u8],
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        group(0, b"SECTION"),
        group(2, b"HEADER"),
        group(9, b"$ACADVER"),
        group(1, version.code().as_bytes()),
    ];
    if let Some(codepage) = codepage {
        groups.extend([group(9, b"$DWGCODEPAGE"), group(3, codepage)]);
    }
    groups.extend([
        group(0, b"ENDSEC"),
        group(0, b"SECTION"),
        group(2, b"ENTITIES"),
        group(0, b"TEXT"),
        group(1, value),
        group(0, b"ENDSEC"),
        group(0, b"EOF"),
    ]);
    encode_fixture(format, version, &groups)
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
                    bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("group code"))?);
                } else {
                    bytes.extend_from_slice(&code.to_le_bytes());
                }
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            _ => return Err(io::Error::other("format")),
        }
    }
    Ok(bytes)
}

fn value_span(view: DxfRawDocumentView<'_>) -> Result<ByteSpan, io::Error> {
    (0..view.group_count())
        .rev()
        .filter_map(|ordinal| view.group(ordinal))
        .find(|group| group.group_code().value() == 1)
        .map(|group| group.value_payload_span())
        .ok_or_else(|| io::Error::other("text value span"))
}

fn planned(
    outcome: Result<DxfTextTranscodePlan, DxfTextTranscodeIssue>,
) -> Result<DxfTextTranscodePlan, io::Error> {
    outcome.map_err(|issue| io::Error::other(format!("{issue:?}")))
}

fn open<'a>(
    source: &'a DxfMemorySource<'_>,
    format: DxfRawDocumentFormat,
) -> Result<OpenedDocument<'a>, Box<dyn Error>> {
    let cancellation = token();
    let mut observer = NoopDxfReadObserver;
    match format {
        DxfRawDocumentFormat::Ascii => Ok(OpenedDocument::Ascii(DxfAsciiRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &cancellation,
            &mut observer,
        )?)),
        DxfRawDocumentFormat::Binary => Ok(OpenedDocument::Binary(DxfBinaryRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &cancellation,
            &mut observer,
        )?)),
        _ => Err(io::Error::other("format").into()),
    }
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

fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
}

fn assert_send_sync<T: Send + Sync>() {}

fn assert_copy<T: Copy>() {}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_XDATA_STRING_MAX_BYTES, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfCancellationToken, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataDestinationEncodeIssue, DxfEntityXDataEncodedApplicationDestinationState,
    DxfEntityXDataEncodedDestinationDirectory, DxfEntityXDataEncodedDestinationState,
    DxfEntityXDataEncodedEntityDestinationState, DxfMemorySource, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, DxfTextDecodeStatus,
    DxfTextEncodeStatus, DxfTextTranscodeIssue, NoopDxfReadObserver,
};

#[test]
fn xdata_strings_transcode_both_directions_for_all_format_pairs() -> Result<(), Box<dyn Error>> {
    for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            assert_transcoded_pair(
                source_format,
                DxfAcadVersion::Ac1018,
                Some(b"ANSI_1252"),
                b"\xE9",
                destination_format,
                DxfAcadVersion::Ac1021,
                None,
                "é".as_bytes(),
            )?;
            assert_transcoded_pair(
                source_format,
                DxfAcadVersion::Ac1021,
                None,
                "é".as_bytes(),
                destination_format,
                DxfAcadVersion::Ac1018,
                Some(b"ANSI_1252"),
                b"\xE9",
            )?;
        }
    }
    Ok(())
}

#[test]
fn transcode_failures_propagate_without_partial_application_or_entity_bytes()
-> Result<(), Box<dyn Error>> {
    assert_unavailable(
        DxfAcadVersion::Ac1021,
        None,
        "你".as_bytes(),
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1252"),
        |issue| {
            matches!(
                issue,
                DxfEntityXDataDestinationEncodeIssue::TextTranscode(
                    DxfTextTranscodeIssue::DestinationEncode {
                        status: DxfTextEncodeStatus::Unmappable { .. }
                    }
                )
            )
        },
    )?;
    assert_unavailable(
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_932"),
        b"\x82",
        DxfAcadVersion::Ac1021,
        None,
        |issue| {
            matches!(
                issue,
                DxfEntityXDataDestinationEncodeIssue::TextTranscode(
                    DxfTextTranscodeIssue::SourceDecode {
                        status: DxfTextDecodeStatus::Malformed { .. }
                    }
                )
            )
        },
    )?;
    assert_unavailable(
        DxfAcadVersion::Ac1021,
        None,
        "가".as_bytes(),
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1361"),
        |issue| {
            matches!(
                issue,
                DxfEntityXDataDestinationEncodeIssue::TextTranscode(
                    DxfTextTranscodeIssue::DestinationEncode {
                        status: DxfTextEncodeStatus::Unavailable
                    }
                )
            )
        },
    )?;
    assert_unavailable(
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1362"),
        b"\x80",
        DxfAcadVersion::Ac1021,
        None,
        |issue| {
            matches!(
                issue,
                DxfEntityXDataDestinationEncodeIssue::TextTranscode(
                    DxfTextTranscodeIssue::SourceEncodingUnavailable { .. }
                )
            )
        },
    )?;
    assert_unavailable(
        DxfAcadVersion::Ac1021,
        None,
        "é".as_bytes(),
        DxfAcadVersion::Ac1018,
        None,
        |issue| {
            matches!(
                issue,
                DxfEntityXDataDestinationEncodeIssue::TextTranscode(
                    DxfTextTranscodeIssue::DestinationEncodingUnavailable { .. }
                )
            )
        },
    )?;
    Ok(())
}

#[test]
fn transcoded_xdata_string_limit_is_enforced_before_group_encoding() -> Result<(), Box<dyn Error>> {
    let source_text = vec![0xE9; 128];
    assert_unavailable(
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1252"),
        &source_text,
        DxfAcadVersion::Ac1021,
        None,
        |issue| {
            matches!(
                issue,
                DxfEntityXDataDestinationEncodeIssue::TranscodedTextTooLong {
                    limit: DXF_XDATA_STRING_MAX_BYTES,
                    observed: 256,
                }
            )
        },
    )
}

#[test]
fn ascii_xdata_remains_portable_without_inventing_transcode_evidence() -> Result<(), Box<dyn Error>>
{
    let source_bytes = source_fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1021,
        None,
        b"PORTABLE_ASCII",
    )?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1018, None)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Binary)?;
    let directory = encoded(source.view(), destination.view())?;
    let text = directory.entries()[1];
    assert!(matches!(
        text.state(),
        DxfEntityXDataEncodedDestinationState::Ready { .. }
    ));
    assert_eq!(directory.text_transcode_for_entry(text), None);
    assert_eq!(
        group_payload(
            directory
                .encoded_bytes_for_entry(text)
                .ok_or_else(|| io::Error::other("ASCII encoded group"))?,
            destination.view().format(),
            DxfAcadVersion::Ac1018,
        )?,
        b"PORTABLE_ASCII"
    );
    Ok(())
}

#[test]
fn xdata_transcode_receipts_are_copy_bound_and_non_disclosing() -> Result<(), Box<dyn Error>> {
    assert_copy::<seacad_dxf_core::DxfTextTranscodeReceipt>();
    assert_send_sync::<seacad_dxf_core::DxfTextTranscodeReceipt>();
    let source_bytes = source_fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1018,
        Some(b"ANSI_1252"),
        b"SECRET_\xE9",
    )?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1021, None)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Binary)?;
    let directory = encoded(source.view(), destination.view())?;
    let receipt = directory
        .text_transcode_for_entry(directory.entries()[1])
        .ok_or_else(|| io::Error::other("redacted receipt"))?;
    assert_eq!(receipt.source_id(), source.view().source_id());
    assert_eq!(receipt.destination_id(), destination.view().source_id());
    let debug = format!("{directory:?} {receipt:?}");
    assert!(!debug.contains("SECRET"));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn assert_transcoded_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    source_code_page: Option<&[u8]>,
    source_text: &[u8],
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
    destination_code_page: Option<&[u8]>,
    expected_text: &[u8],
) -> Result<(), Box<dyn Error>> {
    let source_bytes =
        source_fixture(source_format, source_version, source_code_page, source_text)?;
    let destination_bytes = destination_fixture(
        destination_format,
        destination_version,
        destination_code_page,
    )?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, source_format)?;
    let destination = open(&destination_storage, destination_format)?;
    let directory = encoded(source.view(), destination.view())?;
    assert_eq!(directory.entries().len(), 5);
    let text = directory.entries()[1];
    assert!(matches!(
        text.state(),
        DxfEntityXDataEncodedDestinationState::Ready { .. }
    ));
    let receipt = directory
        .text_transcode_for_entry(text)
        .ok_or_else(|| io::Error::other("text transcode receipt"))?;
    let logical = directory
        .logical_for_entry(text)
        .ok_or_else(|| io::Error::other("logical text"))?;
    let typed = directory
        .logical_destination_directory()
        .typed_for_entry(logical)
        .ok_or_else(|| io::Error::other("typed text"))?;
    assert_eq!(receipt.source_id(), source.view().source_id());
    assert_eq!(receipt.destination_id(), destination.view().source_id());
    assert_eq!(
        receipt.source_span(),
        typed.occurrence().group().value_payload_span()
    );
    assert_eq!(
        receipt.source_encoding(),
        source.view().text_encoding_report().resolution()
    );
    assert_eq!(
        receipt.destination_encoding(),
        destination.view().text_encoding_report().resolution()
    );
    assert_eq!(receipt.source_byte_count(), source_text.len() as u64);
    assert_eq!(receipt.utf8_byte_count(), "é".len() as u64);
    assert_eq!(receipt.encoded_byte_count(), expected_text.len() as u64);
    let group = directory
        .encoded_bytes_for_entry(text)
        .ok_or_else(|| io::Error::other("encoded text group"))?;
    assert_eq!(
        group_payload(group, destination_format, destination_version)?,
        expected_text
    );
    for entry in [
        directory.entries()[0],
        directory.entries()[2],
        directory.entries()[3],
        directory.entries()[4],
    ] {
        assert_eq!(directory.text_transcode_for_entry(entry), None);
    }

    let applications = source
        .view()
        .entity_xdata_encoded_application_destination_directory(
            destination.view(),
            DxfEntityXDataCoordinateTransform::identity(),
            &[],
            DxfResourceProfile::Safe,
            &token(),
        )?;
    assert!(matches!(
        applications.entries()[0].state(),
        DxfEntityXDataEncodedApplicationDestinationState::Ready {
            member_count: 5,
            ..
        }
    ));
    let member = applications.encoded_entries_for_entry(applications.entries()[0])?[1];
    assert_eq!(
        applications
            .encoded_destination_directory()
            .text_transcode_for_entry(member),
        Some(receipt)
    );

    let entities = source
        .view()
        .entity_xdata_encoded_entity_destination_directory(
            destination.view(),
            DxfEntityXDataCoordinateTransform::identity(),
            &[],
            DxfResourceProfile::Safe,
            &token(),
        )?;
    let entity = entities
        .entries()
        .iter()
        .copied()
        .find(|entry| {
            matches!(
                entry.state(),
                DxfEntityXDataEncodedEntityDestinationState::Ready {
                    application_count: 1,
                    member_count: 5,
                    ..
                }
            )
        })
        .ok_or_else(|| io::Error::other("ready transcoded entity"))?;
    let application = entities.applications_for_entry(entity)?[0];
    let nested = entities
        .encoded_application_destination_directory()
        .encoded_entries_for_entry(application)?[1];
    assert_eq!(
        entities
            .encoded_application_destination_directory()
            .encoded_destination_directory()
            .text_transcode_for_entry(nested),
        Some(receipt)
    );
    Ok(())
}

fn assert_unavailable(
    source_version: DxfAcadVersion,
    source_code_page: Option<&[u8]>,
    source_text: &[u8],
    destination_version: DxfAcadVersion,
    destination_code_page: Option<&[u8]>,
    matches_issue: impl FnOnce(DxfEntityXDataDestinationEncodeIssue) -> bool,
) -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(
        DxfRawDocumentFormat::Ascii,
        source_version,
        source_code_page,
        source_text,
    )?;
    let destination_bytes = destination_fixture(
        DxfRawDocumentFormat::Binary,
        destination_version,
        destination_code_page,
    )?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open(&destination_storage, DxfRawDocumentFormat::Binary)?;
    let directory = encoded(source.view(), destination.view())?;
    let text = directory.entries()[1];
    let DxfEntityXDataEncodedDestinationState::EncodingUnavailable(issue) = text.state() else {
        return Err(io::Error::other("expected unavailable text encoding").into());
    };
    if !matches_issue(issue) {
        return Err(io::Error::other(format!("unexpected issue: {issue:?}")).into());
    }
    assert_eq!(directory.encoded_bytes_for_entry(text), None);
    assert_eq!(directory.text_transcode_for_entry(text), None);

    let applications = source
        .view()
        .entity_xdata_encoded_application_destination_directory(
            destination.view(),
            DxfEntityXDataCoordinateTransform::identity(),
            &[],
            DxfResourceProfile::Safe,
            &token(),
        )?;
    assert!(matches!(
        applications.entries()[0].state(),
        DxfEntityXDataEncodedApplicationDestinationState::Unavailable {
            unavailable_member_count: 1,
            ..
        }
    ));
    assert_eq!(
        applications.encoded_bytes_for_entry(applications.entries()[0]),
        None
    );

    let entities = source
        .view()
        .entity_xdata_encoded_entity_destination_directory(
            destination.view(),
            DxfEntityXDataCoordinateTransform::identity(),
            &[],
            DxfResourceProfile::Safe,
            &token(),
        )?;
    let entity = entities
        .entries()
        .iter()
        .copied()
        .find(|entry| {
            matches!(
                entry.state(),
                DxfEntityXDataEncodedEntityDestinationState::Unavailable {
                    unavailable_member_count: 1,
                    ..
                }
            )
        })
        .ok_or_else(|| io::Error::other("unavailable transcoded entity"))?;
    assert_eq!(entities.encoded_bytes_for_entry(entity), None);
    Ok(())
}

fn encoded(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataEncodedDestinationDirectory, seacad_dxf_core::DxfError> {
    source.entity_xdata_encoded_destination_directory(
        destination,
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )
}

fn source_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code_page: Option<&[u8]>,
    text: &[u8],
) -> io::Result<Vec<u8>> {
    let mut groups = header(version, code_page);
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"TABLES"),
        group(0, b"TABLE"),
        group(2, b"APPID"),
        group(0, b"APPID"),
        group(2, b"APP_READY"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"LAYER"),
        group(0, b"LAYER"),
        group(2, b"LAYER_READY"),
        group(0, b"ENDTAB"),
        group(0, b"ENDSEC"),
        group(0, b"SECTION"),
        group(2, b"ENTITIES"),
        group(0, b"POINT"),
        group(1001, b"APP_READY"),
        group(1000, text),
        group(1002, b"{"),
        group(1003, b"LAYER_READY"),
        group(1002, b"}"),
        group(0, b"ENDSEC"),
        group(0, b"EOF"),
    ]);
    encode_fixture(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code_page: Option<&[u8]>,
) -> io::Result<Vec<u8>> {
    let mut groups = header(version, code_page);
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"TABLES"),
        group(0, b"TABLE"),
        group(2, b"APPID"),
        group(0, b"APPID"),
        group(2, b"APP_READY"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"LAYER"),
        group(0, b"LAYER"),
        group(2, b"LAYER_READY"),
        group(0, b"ENDTAB"),
        group(0, b"ENDSEC"),
        group(0, b"EOF"),
    ]);
    encode_fixture(format, version, &groups)
}

fn header(version: DxfAcadVersion, code_page: Option<&[u8]>) -> Vec<(i16, Vec<u8>)> {
    let mut groups = vec![
        group(0, b"SECTION"),
        group(2, b"HEADER"),
        group(9, b"$ACADVER"),
        group(1, version.code().as_bytes()),
    ];
    if let Some(code_page) = code_page {
        groups.extend([group(9, b"$DWGCODEPAGE"), group(3, code_page)]);
    }
    groups.push(group(0, b"ENDSEC"));
    groups
}

fn group(code: i16, value: &[u8]) -> (i16, Vec<u8>) {
    (code, value.to_vec())
}

fn encode_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Vec<u8>)],
) -> io::Result<Vec<u8>> {
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
                push_binary_code(&mut bytes, version, *code)?;
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            _ => return Err(io::Error::other("format")),
        }
    }
    Ok(bytes)
}

fn push_binary_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 && (0..=254).contains(&code) {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(&code) {
        bytes.push(u8::MAX);
        bytes.extend_from_slice(&code.to_le_bytes());
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn group_payload(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<&[u8], io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => bytes
            .strip_prefix(b"1000\n")
            .and_then(|bytes| bytes.strip_suffix(b"\n"))
            .ok_or_else(|| io::Error::other("ASCII group payload")),
        DxfRawDocumentFormat::Binary => {
            let prefix = if version == DxfAcadVersion::Ac1009 {
                3
            } else {
                2
            };
            bytes
                .get(prefix..)
                .and_then(|bytes| bytes.strip_suffix(&[0]))
                .ok_or_else(|| io::Error::other("Binary group payload"))
        }
        _ => Err(io::Error::other("format")),
    }
}

fn open<'a>(
    source: &'a DxfMemorySource<'_>,
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

fn assert_copy<T: Copy>() {}

fn assert_send_sync<T: Send + Sync>() {}

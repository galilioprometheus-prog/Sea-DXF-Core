use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataHandleDestinationState, DxfEntityXDataHandleRemap,
    DxfEntityXDataHandleRemapState, DxfEntityXDataHandleReplacementDirectory,
    DxfEntityXDataHandleReplacementEntry, DxfEntityXDataHandleReplacementState, DxfError,
    DxfHandle, DxfHandleParseIssue, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

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
    }
    assert_eq!(directory.entry(u64::MAX), None);
    Ok(())
}

#[test]
fn directory_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataHandleReplacementDirectory>();
    assert_copy::<DxfEntityXDataHandleReplacementEntry>();
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
                Group(0, b"POINT"),
                Group(5, b"2"),
                Group(0, b"POINT"),
                Group(5, b"3"),
                Group(1001, b"APP"),
                Group(1000, b"SECRET_REPLACEMENT_SOURCE"),
                Group(1005, b"1"),
                Group(1005, b"2"),
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataHandleDestinationDirectory,
    DxfEntityXDataHandleDestinationEntry, DxfEntityXDataHandleDestinationState,
    DxfEntityXDataHandleRemap, DxfEntityXDataHandleRemapState, DxfError, DxfHandle,
    DxfHandleParseIssue, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_destination_parity() -> Result<(), Box<dyn Error>> {
    let mappings = mappings()?;
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let binary_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, version)?;
        let ascii_destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let binary_destination_bytes = destination_fixture(DxfRawDocumentFormat::Binary, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_source_bytes, DxfResourceProfile::Safe)?;
        let binary_source = DxfMemorySource::new(&binary_source_bytes, DxfResourceProfile::Safe)?;
        let ascii_destination =
            DxfMemorySource::new(&ascii_destination_bytes, DxfResourceProfile::Safe)?;
        let binary_destination =
            DxfMemorySource::new(&binary_destination_bytes, DxfResourceProfile::Safe)?;
        let ascii_source = open_ascii(&ascii_source)?;
        let binary_source = open_binary(&binary_source)?;
        let ascii_destination = open_ascii(&ascii_destination)?;
        let binary_destination = open_binary(&binary_destination)?;

        let directories = [
            DxfRawDocumentView::from(&ascii_source).entity_xdata_handle_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &mappings,
                &token(),
            )?,
            DxfRawDocumentView::from(&ascii_source).entity_xdata_handle_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                &mappings,
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_handle_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &mappings,
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_handle_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                &mappings,
                &token(),
            )?,
        ];
        let expected = states(&directories[0]);
        for directory in &directories {
            assert_directory(directory)?;
            assert_eq!(states(directory), expected);
        }
    }
    Ok(())
}

#[test]
fn only_unique_destination_identity_is_publishable() -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_handle_destination_directory(
        DxfRawDocumentView::from(&destination),
        &mappings()?,
        &token(),
    )?;

    let unique = directory.entries()[1];
    assert_eq!(
        unique.state(),
        DxfEntityXDataHandleDestinationState::Unique {
            target: handle(0x200)
        }
    );
    let destination_target = directory
        .destination_target_for_entry(unique)
        .ok_or(io::Error::other("destination target"))?;
    assert_eq!(destination_target.handle(), handle(0x200));
    assert_eq!(
        destination_target.candidate().source_id(),
        directory.destination_id()
    );
    assert_eq!(unique.source_id(), directory.source_id());
    assert_eq!(unique.destination_id(), directory.destination_id());

    for (index, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(
            directory.destination_target_for_entry(entry).is_some(),
            index == 1
        );
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        assert!(directory.remap_for_entry(entry).is_some());
    }
    Ok(())
}

#[test]
fn directory_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataHandleDestinationDirectory>();
    assert_copy::<DxfEntityXDataHandleDestinationEntry>();
    assert!(
        size_of::<DxfEntityXDataHandleDestinationEntry>() <= 128,
        "{}",
        size_of::<DxfEntityXDataHandleDestinationEntry>()
    );

    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_ascii(&destination_storage)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        source.entity_xdata_handle_destination_directory(
            DxfRawDocumentView::from(&destination),
            &[],
            &cancelled
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = source.entity_xdata_handle_destination_directory(
        DxfRawDocumentView::from(&destination),
        &mappings()?,
        &token(),
    )?;
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_DESTINATION_SOURCE"));
    assert!(!debug.contains("SECRET_DESTINATION_TARGET"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let other = source.entity_xdata_handle_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        &mappings()?,
        &token(),
    )?;
    assert_eq!(directory.remap_for_entry(other.entries()[0]), None);
    Ok(())
}

fn assert_directory(directory: &DxfEntityXDataHandleDestinationDirectory) -> Result<(), io::Error> {
    let expected = [
        DxfEntityXDataHandleDestinationState::RemapUnavailable(
            DxfEntityXDataHandleRemapState::AmbiguousMapping {
                source: handle(0xA),
                candidate_count: 2,
            },
        ),
        DxfEntityXDataHandleDestinationState::Unique {
            target: handle(0x200),
        },
        DxfEntityXDataHandleDestinationState::Missing {
            target: handle(0x300),
        },
        DxfEntityXDataHandleDestinationState::Ambiguous {
            target: handle(0x400),
            target_count: 2,
        },
        DxfEntityXDataHandleDestinationState::RemapUnavailable(
            DxfEntityXDataHandleRemapState::SourceMissing,
        ),
        DxfEntityXDataHandleDestinationState::RemapUnavailable(
            DxfEntityXDataHandleRemapState::SourceNull,
        ),
        DxfEntityXDataHandleDestinationState::RemapUnavailable(
            DxfEntityXDataHandleRemapState::SourceInvalid(DxfHandleParseIssue::InvalidDigit {
                offset: 1,
            }),
        ),
        DxfEntityXDataHandleDestinationState::RemapUnavailable(
            DxfEntityXDataHandleRemapState::SourceAmbiguous { target_count: 2 },
        ),
    ];
    assert_eq!(directory.entries().len(), expected.len());
    for (ordinal, (entry, state)) in directory
        .entries()
        .iter()
        .copied()
        .zip(expected)
        .enumerate()
    {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.remap_ordinal(), ordinal as u64);
        assert_eq!(entry.state(), state);
        directory
            .remap_for_entry(entry)
            .ok_or(io::Error::other("remap"))?;
    }
    Ok(())
}

fn states(
    directory: &DxfEntityXDataHandleDestinationDirectory,
) -> Vec<DxfEntityXDataHandleDestinationState> {
    directory
        .entries()
        .iter()
        .map(|entry| entry.state())
        .collect()
}

fn mappings() -> Result<[DxfEntityXDataHandleRemap; 5], io::Error> {
    Ok([
        remap(0xE, 0x200)?,
        remap(0xA, 0x101)?,
        remap(0x9, 0x400)?,
        remap(0xA, 0x100)?,
        remap(0x8, 0x300)?,
    ])
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
    let groups = document_groups(
        version,
        &[
            Group(0, b"POINT"),
            Group(5, b"A"),
            Group(0, b"POINT"),
            Group(5, b"E"),
            Group(0, b"POINT"),
            Group(5, b"8"),
            Group(0, b"POINT"),
            Group(5, b"9"),
            Group(0, b"POINT"),
            Group(5, b"B"),
            Group(0, b"POINT"),
            Group(5, b"b"),
            Group(0, b"POINT"),
            Group(5, b"C"),
            Group(1001, b"APP"),
            Group(1000, b"SECRET_DESTINATION_SOURCE"),
            Group(1005, b"A"),
            Group(1005, b"E"),
            Group(1005, b"8"),
            Group(1005, b"9"),
            Group(1005, b"F"),
            Group(1005, b"0"),
            Group(1005, b"0x1"),
            Group(1005, b"B"),
        ],
    );
    encode(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let groups = document_groups(
        version,
        &[
            Group(0, b"POINT"),
            Group(5, b"200"),
            Group(1000, b"SECRET_DESTINATION_TARGET"),
            Group(0, b"POINT"),
            Group(5, b"400"),
            Group(0, b"POINT"),
            Group(5, b"0400"),
        ],
    );
    encode(format, version, &groups)
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
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[Group<'_>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for Group(code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(value);
        bytes.push(b'\n');
    }
    bytes
}

fn binary_groups(version: DxfAcadVersion, groups: &[Group<'_>]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for Group(code, value) in groups {
        push_code(&mut bytes, version, *code)?;
        bytes.extend_from_slice(value);
        bytes.push(0);
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

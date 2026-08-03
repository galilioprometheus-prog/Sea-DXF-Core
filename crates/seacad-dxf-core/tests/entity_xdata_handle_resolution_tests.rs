use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataHandleResolutionDirectory,
    DxfEntityXDataHandleResolutionEntry, DxfEntityXDataOccurrenceKind, DxfError,
    DxfHandleParseIssue, DxfHandleResolutionState, DxfMemorySource, DxfRawDocumentFormat,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_xdata_handle_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?.entity_xdata_handle_resolution_directory(&token())?;

        let binary_bytes = fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary =
            open_binary(&binary_source)?.entity_xdata_handle_resolution_directory(&token())?;

        assert_directory(&ascii)?;
        assert_directory(&binary)?;
        assert_eq!(evidence(&ascii)?, evidence(&binary)?);
    }
    Ok(())
}

#[test]
fn application_orphan_and_group_102_scopes_remain_distinct() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => {
                open_ascii(&source)?.entity_xdata_handle_resolution_directory(&token())?
            }
            DxfRawDocumentFormat::Binary => {
                open_binary(&source)?.entity_xdata_handle_resolution_directory(&token())?
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(directory.entries().len(), 7);
        assert_eq!(directory.handle_resolution_directory().entries().len(), 8);

        for (index, entry) in directory.entries().iter().copied().enumerate() {
            let typed = directory
                .typed_for_entry(entry)
                .ok_or(io::Error::other("typed entry"))?;
            assert_eq!(directory.entry_for_typed(typed), Some(entry));
            assert_eq!(
                directory
                    .generic_for_entry(entry)
                    .ok_or(io::Error::other("generic entry"))?
                    .state(),
                entry.state()
            );
            let application_value = index < 5;
            assert_eq!(
                matches!(
                    typed.occurrence().kind(),
                    DxfEntityXDataOccurrenceKind::ApplicationValue { .. }
                ),
                application_value
            );
        }

        let unique = directory.entries()[0];
        let target = unique.target().ok_or(io::Error::other("unique target"))?;
        assert_eq!(target.handle().value(), 0xA);
        let candidate_occurrence = target.candidate().group().occurrence();
        let target_range = target.record().group_range();
        assert!(
            target_range.start() <= candidate_occurrence
                && candidate_occurrence < target_range.end()
        );
        for entry in &directory.entries()[1..5] {
            assert_eq!(entry.target(), None);
        }
        assert!(directory.entries()[5].target().is_some());
        assert!(directory.entries()[6].target().is_some());
    }
    Ok(())
}

#[test]
fn directory_is_cancellable_source_bound_bounded_and_non_disclosing() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityXDataHandleResolutionDirectory>();
    assert_copy::<DxfEntityXDataHandleResolutionEntry>();
    assert!(size_of::<DxfEntityXDataHandleResolutionEntry>() <= 176);

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_xdata_handle_resolution_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let directory = document.entity_xdata_handle_resolution_directory(&token())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(!format!("{directory:?}").contains("SECRET_HANDLE_PAYLOAD"));

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?.entity_xdata_handle_resolution_directory(&token())?;
    let foreign = other.entries()[0];
    let foreign_typed = other
        .typed_for_entry(foreign)
        .ok_or(io::Error::other("foreign typed"))?;
    assert_eq!(directory.entry_for_typed(foreign_typed), None);
    assert_eq!(directory.typed_for_entry(foreign), None);
    assert_eq!(directory.generic_for_entry(foreign), None);
    Ok(())
}

fn assert_directory(directory: &DxfEntityXDataHandleResolutionDirectory) -> Result<(), io::Error> {
    let expected = [
        DxfHandleResolutionState::Unique,
        DxfHandleResolutionState::Missing,
        DxfHandleResolutionState::Null,
        DxfHandleResolutionState::Invalid(DxfHandleParseIssue::InvalidDigit { offset: 1 }),
        DxfHandleResolutionState::Ambiguous { target_count: 2 },
        DxfHandleResolutionState::Unique,
        DxfHandleResolutionState::Unique,
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
        assert_eq!(entry.state(), state);
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        assert_eq!(
            entry.target().is_some(),
            state == DxfHandleResolutionState::Unique
        );
        directory
            .typed_for_entry(entry)
            .ok_or(io::Error::other("typed entry"))?;
    }
    Ok(())
}

fn evidence(
    directory: &DxfEntityXDataHandleResolutionDirectory,
) -> Result<
    Vec<(
        DxfHandleResolutionState,
        Option<u64>,
        DxfEntityXDataOccurrenceKind,
    )>,
    io::Error,
> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| {
            let typed = directory
                .typed_for_entry(entry)
                .ok_or(io::Error::other("typed entry"))?;
            Ok((
                entry.state(),
                entry.target().map(|target| target.record().ordinal()),
                typed.occurrence().kind(),
            ))
        })
        .collect()
}

#[derive(Clone, Copy)]
struct Group<'a>(i16, &'a [u8]);

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let groups = [
        Group(0, b"SECTION"),
        Group(2, b"HEADER"),
        Group(9, b"$ACADVER"),
        Group(1, version.code().as_bytes()),
        Group(0, b"ENDSEC"),
        Group(0, b"SECTION"),
        Group(2, b"ENTITIES"),
        Group(0, b"POINT"),
        Group(5, b"A"),
        Group(0, b"POINT"),
        Group(5, b"B"),
        Group(0, b"POINT"),
        Group(5, b"b"),
        Group(0, b"POINT"),
        Group(5, b"C"),
        Group(1001, b"APP"),
        Group(1000, b"SECRET_HANDLE_PAYLOAD"),
        Group(1005, b"A"),
        Group(1005, b"F"),
        Group(1005, b"0"),
        Group(1005, b"0x1"),
        Group(1005, b"B"),
        Group(102, b"{FAKE"),
        Group(1005, b"A"),
        Group(102, b"}"),
        Group(8, b"0"),
        Group(1005, b"A"),
        Group(0, b"LINE"),
        Group(5, b"D"),
        Group(1005, b"A"),
        Group(0, b"ENDSEC"),
        Group(0, b"EOF"),
    ];
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
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

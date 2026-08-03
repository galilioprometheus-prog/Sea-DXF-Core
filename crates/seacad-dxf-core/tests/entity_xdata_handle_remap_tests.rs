use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataHandleRemap, DxfEntityXDataHandleRemapDirectory,
    DxfEntityXDataHandleRemapEntry, DxfEntityXDataHandleRemapInputIssue,
    DxfEntityXDataHandleRemapState, DxfError, DxfHandle, DxfHandleParseIssue, DxfMemorySource,
    DxfRawDocumentFormat, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_remap_parity() -> Result<(), Box<dyn Error>> {
    let mappings = mappings()?;
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii =
            open_ascii(&ascii_source)?.entity_xdata_handle_remap_directory(&mappings, &token())?;

        let binary_bytes = fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?
            .entity_xdata_handle_remap_directory(&mappings, &token())?;

        assert_directory(&ascii)?;
        assert_directory(&binary)?;
        assert_eq!(states(&ascii), states(&binary));
    }
    Ok(())
}

#[test]
fn mapping_candidates_are_validated_sorted_and_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        DxfEntityXDataHandleRemap::new(handle(0), handle(1)),
        Err(DxfEntityXDataHandleRemapInputIssue::NullSource)
    );
    assert_eq!(
        DxfEntityXDataHandleRemap::new(handle(1), handle(0)),
        Err(DxfEntityXDataHandleRemapInputIssue::NullTarget)
    );
    let mappings = mappings()?;
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.entity_xdata_handle_remap_directory(&mappings, &token())?;

    assert_eq!(
        directory.mappings(),
        [remap(0xA, 0x100)?, remap(0xA, 0x101)?, remap(0xE, 0x200)?,]
    );
    let first = directory.entries()[0];
    assert!(matches!(
        first.state(),
        DxfEntityXDataHandleRemapState::AmbiguousMapping {
            source,
            candidate_count: 2
        } if source == handle(0xA)
    ));
    assert_eq!(
        directory
            .source_target_for_entry(first)
            .ok_or(io::Error::other("source target"))?
            .handle(),
        handle(0xA)
    );
    let mapped = directory.entries()[1];
    assert_eq!(
        mapped.state(),
        DxfEntityXDataHandleRemapState::Mapped {
            source: handle(0xE),
            target: handle(0x200),
        }
    );
    assert!(directory.source_target_for_entry(mapped).is_some());
    assert!(
        directory
            .source_target_for_entry(directory.entries()[3])
            .is_none()
    );
    Ok(())
}

#[test]
fn directory_is_cancellable_source_bound_bounded_and_non_disclosing() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityXDataHandleRemapDirectory>();
    assert_copy::<DxfEntityXDataHandleRemap>();
    assert_copy::<DxfEntityXDataHandleRemapEntry>();
    assert!(size_of::<DxfEntityXDataHandleRemapEntry>() <= 96);

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_xdata_handle_remap_directory(&[], &cancelled),
        Err(DxfError::Cancelled)
    ));

    let directory = document.entity_xdata_handle_remap_directory(&mappings()?, &token())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(!format!("{directory:?}").contains("SECRET_REMAP_PAYLOAD"));
    for entry in directory.entries().iter().copied() {
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        assert!(directory.resolution_for_entry(entry).is_some());
    }

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other =
        open_ascii(&other_source)?.entity_xdata_handle_remap_directory(&mappings()?, &token())?;
    let foreign = other.entries()[0];
    assert_eq!(directory.resolution_for_entry(foreign), None);
    assert_eq!(directory.source_target_for_entry(foreign), None);
    Ok(())
}

fn assert_directory(directory: &DxfEntityXDataHandleRemapDirectory) -> Result<(), io::Error> {
    let expected = [
        DxfEntityXDataHandleRemapState::AmbiguousMapping {
            source: handle(0xA),
            candidate_count: 2,
        },
        DxfEntityXDataHandleRemapState::Mapped {
            source: handle(0xE),
            target: handle(0x200),
        },
        DxfEntityXDataHandleRemapState::Unmapped {
            source: handle(0x7),
        },
        DxfEntityXDataHandleRemapState::SourceMissing,
        DxfEntityXDataHandleRemapState::SourceNull,
        DxfEntityXDataHandleRemapState::SourceInvalid(DxfHandleParseIssue::InvalidDigit {
            offset: 1,
        }),
        DxfEntityXDataHandleRemapState::SourceAmbiguous { target_count: 2 },
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
        assert_eq!(entry.resolution_ordinal(), ordinal as u64);
        directory
            .resolution_for_entry(entry)
            .ok_or(io::Error::other("resolution"))?;
    }
    Ok(())
}

fn states(directory: &DxfEntityXDataHandleRemapDirectory) -> Vec<DxfEntityXDataHandleRemapState> {
    directory
        .entries()
        .iter()
        .map(|entry| entry.state())
        .collect()
}

fn mappings() -> Result<[DxfEntityXDataHandleRemap; 3], io::Error> {
    Ok([remap(0xE, 0x200)?, remap(0xA, 0x101)?, remap(0xA, 0x100)?])
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
        Group(5, b"E"),
        Group(0, b"POINT"),
        Group(5, b"7"),
        Group(0, b"POINT"),
        Group(5, b"B"),
        Group(0, b"POINT"),
        Group(5, b"b"),
        Group(0, b"POINT"),
        Group(5, b"C"),
        Group(1001, b"APP"),
        Group(1000, b"SECRET_REMAP_PAYLOAD"),
        Group(1005, b"A"),
        Group(1005, b"E"),
        Group(1005, b"7"),
        Group(1005, b"F"),
        Group(1005, b"0"),
        Group(1005, b"0x1"),
        Group(1005, b"B"),
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

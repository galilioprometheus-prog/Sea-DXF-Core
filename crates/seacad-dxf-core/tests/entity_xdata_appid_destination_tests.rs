use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataAppIdDestinationDirectory,
    DxfEntityXDataAppIdDestinationEntry, DxfEntityXDataAppIdDestinationState, DxfError,
    DxfMemorySource, DxfNamedSymbolTableKind, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_destination_appid_parity()
-> Result<(), Box<dyn Error>> {
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
            DxfRawDocumentView::from(&ascii_source).entity_xdata_appid_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&ascii_source).entity_xdata_appid_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_appid_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_appid_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                &token(),
            )?,
        ];
        for directory in &directories {
            assert_directory(directory)?;
        }
    }

    assert_cross_dialect_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
    )?;
    assert_cross_dialect_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
    )?;
    Ok(())
}

#[test]
fn malformed_destination_appid_tables_publish_only_missing() -> Result<(), Box<dyn Error>> {
    let source_bytes =
        malformed_source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;

    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let destination_bytes = malformed_destination_fixture(format, DxfAcadVersion::Ac1032)?;
        let destination_storage =
            DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
        let states: Vec<_> = match format {
            DxfRawDocumentFormat::Ascii => {
                let destination = open_ascii(&destination_storage)?;
                source
                    .entity_xdata_appid_destination_directory(
                        DxfRawDocumentView::from(&destination),
                        &token(),
                    )?
                    .entries()
                    .iter()
                    .map(|entry| entry.state())
                    .collect()
            }
            DxfRawDocumentFormat::Binary => {
                let destination = open_binary(&destination_storage)?;
                source
                    .entity_xdata_appid_destination_directory(
                        DxfRawDocumentView::from(&destination),
                        &token(),
                    )?
                    .entries()
                    .iter()
                    .map(|entry| entry.state())
                    .collect()
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(states.len(), 4);
        assert!(
            states
                .iter()
                .all(|state| { *state == DxfEntityXDataAppIdDestinationState::DestinationMissing })
        );
    }
    Ok(())
}

#[test]
fn directory_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataAppIdDestinationDirectory>();
    assert_copy::<DxfEntityXDataAppIdDestinationEntry>();
    assert_copy::<DxfEntityXDataAppIdDestinationState>();
    let entry_size = size_of::<DxfEntityXDataAppIdDestinationEntry>();
    assert!(
        entry_size <= 128,
        "destination APPID entry is {entry_size} bytes"
    );

    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        source.entity_xdata_appid_destination_directory(
            DxfRawDocumentView::from(&destination),
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));

    let directory = source.entity_xdata_appid_destination_directory(
        DxfRawDocumentView::from(&destination),
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(
        directory.source_resolution_directory().source_id(),
        source.source_id()
    );
    assert_eq!(
        directory.destination_symbol_table_directory().source_id(),
        destination.source_id()
    );
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(
        directory.entries_for_entity(
            directory.source_resolution_directory().entries()[0]
                .application()
                .entity(),
        )?,
        directory.entries()
    );
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_APP_OK"));
    assert!(!debug.contains("SECRET_SOURCE_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_APPID"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let other_destination_directory = source.entity_xdata_appid_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        &token(),
    )?;
    let foreign_destination_entry = other_destination_directory.entries()[0];
    assert_eq!(
        directory.source_resolution_for_entry(foreign_destination_entry),
        None
    );
    assert_eq!(
        directory.destination_target_for_entry(foreign_destination_entry),
        None
    );

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_appid_destination_directory(
            DxfRawDocumentView::from(&destination),
            &token(),
        )?;
    let foreign_source_entry = other_source_directory.entries()[0];
    assert_eq!(
        directory.source_resolution_for_entry(foreign_source_entry),
        None
    );
    assert_eq!(
        directory.destination_target_for_entry(foreign_source_entry),
        None
    );
    assert!(matches!(
        directory.entry_for_application(
            other_source_directory
                .source_resolution_directory()
                .entries()[0]
                .application(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn assert_cross_dialect_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(source_format, source_version)?;
    let destination_bytes = destination_fixture(destination_format, destination_version)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    match (source_format, destination_format) {
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            let directory = source.entity_xdata_appid_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?;
            assert_directory(&directory)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            let directory = source.entity_xdata_appid_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?;
            assert_directory(&directory)?;
        }
        _ => return Err(io::Error::other("cross-dialect test format pair").into()),
    }
    Ok(())
}

fn assert_directory(directory: &DxfEntityXDataAppIdDestinationDirectory) -> Result<(), io::Error> {
    let expected = [0_u8, 1, 2, 3, 4, 1, 0, 0];
    assert_eq!(directory.entries().len(), expected.len());
    for (ordinal, (entry, expected_state)) in directory
        .entries()
        .iter()
        .copied()
        .zip(expected)
        .enumerate()
    {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.source_resolution_ordinal(), ordinal as u64);
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(entry.destination_id(), directory.destination_id());
        let resolution = directory
            .source_resolution_for_entry(entry)
            .ok_or(io::Error::other("source resolution"))?;
        assert_eq!(resolution.application().ordinal(), ordinal as u64);
        assert_eq!(
            directory
                .entry_for_application(resolution.application())
                .map_err(|error| io::Error::other(format!("{error:?}")))?,
            entry
        );
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));

        let observed = match entry.state() {
            DxfEntityXDataAppIdDestinationState::DestinationUnique { target } => {
                assert_eq!(target.kind(), DxfNamedSymbolTableKind::AppId);
                assert_eq!(
                    directory
                        .destination_symbol_table_directory()
                        .entry_for_raw_ordinal(target.record().ordinal()),
                    Some(target)
                );
                assert_eq!(directory.destination_target_for_entry(entry), Some(target));
                0
            }
            DxfEntityXDataAppIdDestinationState::DestinationMissing => 1,
            DxfEntityXDataAppIdDestinationState::DestinationAmbiguous { target_count: 2 } => 2,
            DxfEntityXDataAppIdDestinationState::SourceMissing => 3,
            DxfEntityXDataAppIdDestinationState::SourceAmbiguous { target_count: 2 } => 4,
            _ => return Err(io::Error::other("destination APPID state")),
        };
        assert_eq!(observed, expected_state);
        if observed != 0 {
            assert_eq!(directory.destination_target_for_entry(entry), None);
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Group<'a>(i16, &'a [u8]);

fn source_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let long_appid = long_appid();
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"SECRET_APP_OK"),
            Group(0, b"APPID"),
            Group(2, b"DEST_MISSING"),
            Group(0, b"APPID"),
            Group(2, b"DEST_DUP"),
            Group(0, b"APPID"),
            Group(2, b"SOURCE_DUP"),
            Group(0, b"APPID"),
            Group(2, b"SOURCE_DUP"),
            Group(0, b"APPID"),
            Group(2, b"NearCase"),
            Group(0, b"APPID"),
            Group(2, long_appid.as_slice()),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(1000, b"ORPHAN"),
            Group(1001, b"SECRET_APP_OK"),
            Group(1000, b"SECRET_SOURCE_PAYLOAD"),
            Group(1001, b"DEST_MISSING"),
            Group(1000, b"A"),
            Group(1001, b"DEST_DUP"),
            Group(1000, b"B"),
            Group(1001, b"SOURCE_MISSING"),
            Group(1000, b"C"),
            Group(1001, b"SOURCE_DUP"),
            Group(1000, b"D"),
            Group(1001, b"NearCase"),
            Group(1000, b"E"),
            Group(1001, b"SECRET_APP_OK"),
            Group(1000, b"F"),
            Group(1001, long_appid.as_slice()),
            Group(1000, b"G"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let long_appid = long_appid();
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"SECRET_APP_OK"),
            Group(1000, b"SECRET_DESTINATION_APPID"),
            Group(0, b"APPID"),
            Group(2, b"DEST_DUP"),
            Group(0, b"APPID"),
            Group(2, b"DEST_DUP"),
            Group(0, b"APPID"),
            Group(2, b"SOURCE_MISSING"),
            Group(0, b"APPID"),
            Group(2, b"SOURCE_DUP"),
            Group(0, b"APPID"),
            Group(2, b"nearcase"),
            Group(0, b"APPID"),
            Group(2, long_appid.as_slice()),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn long_appid() -> Vec<u8> {
    let mut name = Vec::with_capacity(4_097);
    name.extend_from_slice(b"LONG_");
    name.resize(4_097, b'X');
    name
}

fn malformed_source_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"UNCLOSED"),
            Group(0, b"APPID"),
            Group(2, b"LOWER_TABLE"),
            Group(0, b"APPID"),
            Group(2, b"WRONG_TABLE"),
            Group(0, b"APPID"),
            Group(2, b"MULTI_NAME"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(1001, b"UNCLOSED"),
            Group(1001, b"LOWER_TABLE"),
            Group(1001, b"WRONG_TABLE"),
            Group(1001, b"MULTI_NAME"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn malformed_destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"UNCLOSED"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"appid"),
            Group(0, b"APPID"),
            Group(2, b"LOWER_TABLE"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"APPID"),
            Group(2, b"WRONG_TABLE"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"MULTI_NAME"),
            Group(2, b"DECOY"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
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
    ];
    groups.extend_from_slice(body);
    groups.push(Group(0, b"EOF"));
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

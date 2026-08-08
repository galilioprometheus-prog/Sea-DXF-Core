use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataCoordinateDestinationState,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataEntityDestinationState,
    DxfEntityXDataHandleComposedDestinationDirectory, DxfEntityXDataHandleComposedDestinationEntry,
    DxfEntityXDataHandleComposedDestinationState, DxfEntityXDataHandleDestinationState,
    DxfEntityXDataHandleRemap, DxfEntityXDataHandleRemapState, DxfError, DxfHandle,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_handle_composition_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        assert_all_format_pairs(version, version)?;
    }
    assert_format_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
    )?;
    assert_format_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
    )?;
    Ok(())
}

#[test]
fn coordinate_and_handle_failures_remain_independent() -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_handle_composed_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?;

    assert!(matches!(
        directory.entries()[5].state(),
        DxfEntityXDataHandleComposedDestinationState::Ready {
            transformed_tuple_count: 1,
            destination_handle_count: 1,
            ..
        }
    ));
    let expected_handle_states = [
        DxfEntityXDataHandleDestinationState::Missing {
            target: handle(0x300),
        },
        DxfEntityXDataHandleDestinationState::Ambiguous {
            target: handle(0x400),
            target_count: 2,
        },
        DxfEntityXDataHandleDestinationState::RemapUnavailable(
            DxfEntityXDataHandleRemapState::Unmapped {
                source: handle(0xD),
            },
        ),
    ];
    for (entry, expected) in directory.entries()[6..9]
        .iter()
        .copied()
        .zip(expected_handle_states)
    {
        assert!(matches!(
            entry.state(),
            DxfEntityXDataHandleComposedDestinationState::Unavailable {
                coordinate_state: DxfEntityXDataCoordinateDestinationState::Ready { .. },
                handle_count: 1,
                unavailable_handle_count: 1,
            }
        ));
        assert_eq!(
            directory.handle_entries_for_entry(entry)?[0].state(),
            expected
        );
    }

    let coordinate_failure = directory.entries()[9];
    assert!(matches!(
        coordinate_failure.state(),
        DxfEntityXDataHandleComposedDestinationState::Unavailable {
            coordinate_state: DxfEntityXDataCoordinateDestinationState::Unavailable {
                entity_state: DxfEntityXDataEntityDestinationState::Unavailable { .. },
                tuple_count: 1,
                unavailable_tuple_count: 1,
            },
            handle_count: 1,
            unavailable_handle_count: 0,
        }
    ));
    assert!(matches!(
        directory.entries()[10].state(),
        DxfEntityXDataHandleComposedDestinationState::Ready {
            transformed_tuple_count: 1,
            destination_handle_count: 0,
            ..
        }
    ));
    assert!(matches!(
        directory.entries()[11].state(),
        DxfEntityXDataHandleComposedDestinationState::Unavailable {
            coordinate_state: DxfEntityXDataCoordinateDestinationState::Unavailable {
                entity_state: DxfEntityXDataEntityDestinationState::Unavailable { .. },
                ..
            },
            handle_count: 1,
            unavailable_handle_count: 0,
        }
    ));
    Ok(())
}

#[test]
fn composition_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataHandleComposedDestinationDirectory>();
    assert_copy::<DxfEntityXDataHandleComposedDestinationEntry>();
    assert_copy::<DxfEntityXDataHandleComposedDestinationState>();
    let entry_size = size_of::<DxfEntityXDataHandleComposedDestinationEntry>();
    assert!(
        entry_size <= 192,
        "handle composed destination entry is {entry_size} bytes"
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
        source.entity_xdata_handle_composed_destination_directory(
            DxfRawDocumentView::from(&destination),
            DxfEntityXDataCoordinateTransform::identity(),
            &[],
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = source.entity_xdata_handle_composed_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_HANDLE_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let foreign_destination = source.entity_xdata_handle_composed_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?;
    let foreign_entry = foreign_destination.entries()[5];
    assert_eq!(
        directory.coordinate_destination_for_entry(foreign_entry),
        None
    );
    assert!(directory.entity_for_entry(foreign_entry).is_err());
    assert!(directory.handle_entries_for_entry(foreign_entry).is_err());

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_handle_composed_destination_directory(
            DxfRawDocumentView::from(&destination),
            DxfEntityXDataCoordinateTransform::identity(),
            &mappings()?,
            &token(),
        )?;
    let foreign_entity =
        other_source_directory.entity_for_entry(other_source_directory.entries()[5])?;
    assert!(matches!(
        directory.entry_for_entity(foreign_entity),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(matches!(
        directory
            .handle_destination_directory()
            .entries_for_entity(foreign_entity),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn assert_all_format_pairs(
    source_version: DxfAcadVersion,
    destination_version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            assert_format_pair(
                source_format,
                source_version,
                destination_format,
                destination_version,
            )?;
        }
    }
    Ok(())
}

fn assert_format_pair(
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
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Ascii) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(&build_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_directory(&build_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(&build_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Binary) => {
            let source = open_binary(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_directory(&build_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        _ => return Err(io::Error::other("unsupported format pair").into()),
    }
    Ok(())
}

fn build_directory(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataHandleComposedDestinationDirectory, Box<dyn Error>> {
    Ok(source.entity_xdata_handle_composed_destination_directory(
        destination,
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?)
}

fn assert_directory(
    directory: &DxfEntityXDataHandleComposedDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 12);
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.coordinate_destination_ordinal(), ordinal as u64);
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(entry.destination_id(), directory.destination_id());
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let coordinate = directory
            .coordinate_destination_for_entry(entry)
            .ok_or(io::Error::other("coordinate destination"))?;
        let entity = directory.entity_for_entry(entry)?;
        assert_eq!(directory.entry_for_entity(entity)?, Some(entry));
        let handles = directory.handle_entries_for_entry(entry)?;
        assert_eq!(handles.len(), usize::from(matches!(ordinal, 5..=9 | 11)));
        assert_eq!(coordinate.ordinal(), ordinal as u64);
        match ordinal {
            0..=4 => assert!(matches!(
                entry.state(),
                DxfEntityXDataHandleComposedDestinationState::Ready {
                    application_count: 0,
                    transformed_tuple_count: 0,
                    destination_handle_count: 0,
                    ..
                }
            )),
            5 => assert!(matches!(
                entry.state(),
                DxfEntityXDataHandleComposedDestinationState::Ready {
                    application_count: 1,
                    transformed_tuple_count: 1,
                    destination_handle_count: 1,
                    ..
                }
            )),
            6..=8 => assert!(matches!(
                entry.state(),
                DxfEntityXDataHandleComposedDestinationState::Unavailable {
                    coordinate_state: DxfEntityXDataCoordinateDestinationState::Ready { .. },
                    handle_count: 1,
                    unavailable_handle_count: 1,
                }
            )),
            9 => assert!(matches!(
                entry.state(),
                DxfEntityXDataHandleComposedDestinationState::Unavailable {
                    coordinate_state: DxfEntityXDataCoordinateDestinationState::Unavailable { .. },
                    handle_count: 1,
                    unavailable_handle_count: 0,
                }
            )),
            10 => assert!(matches!(
                entry.state(),
                DxfEntityXDataHandleComposedDestinationState::Ready {
                    application_count: 1,
                    transformed_tuple_count: 1,
                    destination_handle_count: 0,
                    ..
                }
            )),
            11 => assert!(matches!(
                entry.state(),
                DxfEntityXDataHandleComposedDestinationState::Unavailable {
                    coordinate_state: DxfEntityXDataCoordinateDestinationState::Unavailable { .. },
                    handle_count: 1,
                    unavailable_handle_count: 0,
                }
            )),
            _ => return Err(io::Error::other("composition ordinal").into()),
        }
    }
    Ok(())
}

fn mappings() -> Result<[DxfEntityXDataHandleRemap; 3], io::Error> {
    Ok([remap(0xA, 0x200)?, remap(0xB, 0x300)?, remap(0xC, 0x400)?])
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
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"APP_READY"),
            Group(0, b"APPID"),
            Group(2, b"APP_DESTINATION_BAD"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(5, b"A"),
            Group(0, b"POINT"),
            Group(5, b"B"),
            Group(0, b"POINT"),
            Group(5, b"C"),
            Group(0, b"POINT"),
            Group(5, b"D"),
            Group(0, b"LINE"),
            Group(5, b"E"),
            Group(1001, b"APP_READY"),
            Group(1010, b"1"),
            Group(1020, b"2"),
            Group(1030, b"3"),
            Group(1000, b"SECRET_HANDLE_PAYLOAD"),
            Group(1005, b"A"),
            Group(0, b"CIRCLE"),
            Group(5, b"F"),
            Group(1001, b"APP_READY"),
            Group(1005, b"B"),
            Group(0, b"ARC"),
            Group(5, b"10"),
            Group(1001, b"APP_READY"),
            Group(1005, b"C"),
            Group(0, b"TEXT"),
            Group(5, b"11"),
            Group(1001, b"APP_READY"),
            Group(1005, b"D"),
            Group(0, b"SHAPE"),
            Group(5, b"12"),
            Group(1001, b"APP_READY"),
            Group(1011, b"1"),
            Group(1021, b"2"),
            Group(1005, b"A"),
            Group(0, b"ELLIPSE"),
            Group(5, b"13"),
            Group(1001, b"APP_READY"),
            Group(1012, b"1"),
            Group(1022, b"2"),
            Group(1032, b"3"),
            Group(0, b"SPLINE"),
            Group(5, b"14"),
            Group(1001, b"APP_DESTINATION_BAD"),
            Group(1005, b"A"),
            Group(0, b"ENDSEC"),
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
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"APP_READY"),
            Group(1000, b"SECRET_DESTINATION_PAYLOAD"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(5, b"200"),
            Group(0, b"POINT"),
            Group(5, b"400"),
            Group(0, b"POINT"),
            Group(5, b"0400"),
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
        _ => Err(io::Error::other("format")),
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
        if (1010..=1059).contains(code) {
            let text = std::str::from_utf8(value).map_err(|_| io::Error::other("double text"))?;
            let value = text
                .parse::<f64>()
                .map_err(|_| io::Error::other("double value"))?;
            bytes.extend_from_slice(&value.to_le_bytes());
        } else {
            bytes.extend_from_slice(value);
            bytes.push(0);
        }
    }
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 && (0..=254).contains(&code) {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("code"))?);
    } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(&code) {
        bytes.push(u8::MAX);
        bytes.extend_from_slice(&code.to_le_bytes());
    } else if version == DxfAcadVersion::Ac1009 {
        return Err(io::Error::other("code"));
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

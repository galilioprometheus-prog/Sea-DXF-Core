use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataCoordinateDestinationState,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataHandleComposedDestinationState,
    DxfEntityXDataHandleRemap, DxfEntityXDataPayloadDestinationDirectory,
    DxfEntityXDataPayloadDestinationEntry, DxfEntityXDataPayloadDestinationState, DxfError,
    DxfHandle, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_payload_readiness_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                assert_format_pair(source_format, version, destination_format, version)?;
            }
        }
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
fn orphan_and_composed_failures_remain_independent() -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_payload_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?;

    assert!(matches!(
        directory.entries()[3].state(),
        DxfEntityXDataPayloadDestinationState::Unavailable {
            composition_state: DxfEntityXDataHandleComposedDestinationState::Unavailable { .. },
            payload_value_count: 1,
            orphan_value_count: 1,
        }
    ));
    assert!(matches!(
        directory.entries()[4].state(),
        DxfEntityXDataPayloadDestinationState::Unavailable {
            composition_state: DxfEntityXDataHandleComposedDestinationState::Unavailable {
                unavailable_handle_count: 1,
                ..
            },
            payload_value_count: 1,
            orphan_value_count: 0,
        }
    ));
    assert!(matches!(
        directory.entries()[5].state(),
        DxfEntityXDataPayloadDestinationState::Unavailable {
            composition_state: DxfEntityXDataHandleComposedDestinationState::Unavailable {
                coordinate_state: DxfEntityXDataCoordinateDestinationState::Unavailable { .. },
                unavailable_handle_count: 0,
                ..
            },
            payload_value_count: 3,
            orphan_value_count: 0,
        }
    ));
    assert_eq!(
        directory
            .applications_for_entry(directory.entries()[3])?
            .len(),
        0
    );
    assert_eq!(
        directory
            .typed_entries_for_entry(directory.entries()[3])?
            .len(),
        1
    );
    Ok(())
}

#[test]
fn payload_readiness_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataPayloadDestinationDirectory>();
    assert_copy::<DxfEntityXDataPayloadDestinationEntry>();
    assert_copy::<DxfEntityXDataPayloadDestinationState>();
    let entry_size = size_of::<DxfEntityXDataPayloadDestinationEntry>();
    assert!(
        entry_size <= 192,
        "payload destination entry is {entry_size} bytes"
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
        source.entity_xdata_payload_destination_directory(
            DxfRawDocumentView::from(&destination),
            DxfEntityXDataCoordinateTransform::identity(),
            &[],
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = source.entity_xdata_payload_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_APPLICATION_PAYLOAD"));
    assert!(!debug.contains("SECRET_ORPHAN_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let foreign_destination = source.entity_xdata_payload_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?;
    let foreign_entry = foreign_destination.entries()[2];
    assert_eq!(directory.composition_for_entry(foreign_entry), None);
    assert!(directory.entity_for_entry(foreign_entry).is_err());
    assert!(directory.typed_entries_for_entry(foreign_entry).is_err());
    assert!(directory.applications_for_entry(foreign_entry).is_err());

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_payload_destination_directory(
            DxfRawDocumentView::from(&destination),
            DxfEntityXDataCoordinateTransform::identity(),
            &mappings()?,
            &token(),
        )?;
    let foreign_entity =
        other_source_directory.entity_for_entry(other_source_directory.entries()[2])?;
    assert!(matches!(
        directory.entry_for_entity(foreign_entity),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
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
        _ => return Err(io::Error::other("format pair").into()),
    }
    Ok(())
}

fn build_directory(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataPayloadDestinationDirectory, Box<dyn Error>> {
    Ok(source.entity_xdata_payload_destination_directory(
        destination,
        DxfEntityXDataCoordinateTransform::identity(),
        &mappings()?,
        &token(),
    )?)
}

fn assert_directory(
    directory: &DxfEntityXDataPayloadDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 7);
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.composition_ordinal(), ordinal as u64);
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(entry.destination_id(), directory.destination_id());
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let composition = directory
            .composition_for_entry(entry)
            .ok_or(io::Error::other("composition"))?;
        let entity = directory.entity_for_entry(entry)?;
        assert_eq!(directory.entry_for_entity(entity)?, Some(entry));
        let typed = directory.typed_entries_for_entry(entry)?;
        let applications = directory.applications_for_entry(entry)?;
        assert_eq!(composition.ordinal(), ordinal as u64);
        match ordinal {
            0 | 1 => assert_eq!(
                entry.state(),
                DxfEntityXDataPayloadDestinationState::Ready {
                    application_count: 0,
                    payload_value_count: 0,
                    used_bytes: 0,
                    remaining_bytes: 16_383,
                    transformed_tuple_count: 0,
                    destination_handle_count: 0,
                }
            ),
            2 => assert!(matches!(
                entry.state(),
                DxfEntityXDataPayloadDestinationState::Ready {
                    application_count: 1,
                    payload_value_count: 5,
                    transformed_tuple_count: 1,
                    destination_handle_count: 1,
                    ..
                }
            )),
            3 => assert!(matches!(
                entry.state(),
                DxfEntityXDataPayloadDestinationState::Unavailable {
                    composition_state: DxfEntityXDataHandleComposedDestinationState::Unavailable { .. },
                    payload_value_count: 1,
                    orphan_value_count: 1,
                }
            )),
            4 | 5 => assert!(matches!(
                entry.state(),
                DxfEntityXDataPayloadDestinationState::Unavailable {
                    orphan_value_count: 0,
                    ..
                }
            )),
            6 => assert!(matches!(
                entry.state(),
                DxfEntityXDataPayloadDestinationState::Ready {
                    application_count: 1,
                    payload_value_count: 1,
                    transformed_tuple_count: 0,
                    destination_handle_count: 0,
                    ..
                }
            )),
            _ => return Err(io::Error::other("payload ordinal").into()),
        }
        assert_eq!(
            typed.len(),
            applications.len() + payload_count(entry.state())?
        );
    }
    Ok(())
}

fn payload_count(state: DxfEntityXDataPayloadDestinationState) -> Result<usize, io::Error> {
    let count = match state {
        DxfEntityXDataPayloadDestinationState::Ready {
            payload_value_count,
            ..
        }
        | DxfEntityXDataPayloadDestinationState::Unavailable {
            payload_value_count,
            ..
        } => payload_value_count,
        _ => return Err(io::Error::other("payload state")),
    };
    usize::try_from(count).map_err(|_| io::Error::other("payload count"))
}

fn mappings() -> Result<[DxfEntityXDataHandleRemap; 2], io::Error> {
    Ok([remap(0xA, 0x200)?, remap(0xB, 0x300)?])
}

fn remap(source: u64, target: u64) -> Result<DxfEntityXDataHandleRemap, io::Error> {
    DxfEntityXDataHandleRemap::new(DxfHandle::from_u64(source), DxfHandle::from_u64(target))
        .map_err(|issue| io::Error::other(format!("{issue:?}")))
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
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(5, b"A"),
            Group(0, b"LINE"),
            Group(5, b"B"),
            Group(1001, b"APP_READY"),
            Group(1010, b"1"),
            Group(1020, b"2"),
            Group(1030, b"3"),
            Group(1000, b"SECRET_APPLICATION_PAYLOAD"),
            Group(1005, b"A"),
            Group(0, b"CIRCLE"),
            Group(5, b"C"),
            Group(1000, b"SECRET_ORPHAN_PAYLOAD"),
            Group(0, b"ARC"),
            Group(5, b"D"),
            Group(1001, b"APP_READY"),
            Group(1005, b"B"),
            Group(0, b"TEXT"),
            Group(5, b"E"),
            Group(1001, b"APP_READY"),
            Group(1011, b"1"),
            Group(1021, b"2"),
            Group(1005, b"A"),
            Group(0, b"ELLIPSE"),
            Group(5, b"F"),
            Group(1001, b"APP_READY"),
            Group(1000, b"A"),
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_XDATA_ENTITY_CAPACITY_BYTES, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfEntityXDataCapacityIssueKind,
    DxfEntityXDataCapacityState, DxfEntityXDataEntityDestinationDirectory,
    DxfEntityXDataEntityDestinationEntry, DxfEntityXDataEntityDestinationState, DxfError,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_entity_capacity_readiness_parity()
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
            DxfRawDocumentView::from(&ascii_source).entity_xdata_entity_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&ascii_source).entity_xdata_entity_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_entity_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_entity_destination_directory(
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
fn destination_and_capacity_failures_remain_independent() -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_entity_destination_directory(
        DxfRawDocumentView::from(&destination),
        &token(),
    )?;

    let destination_only = directory.entries()[3];
    assert!(matches!(
        destination_only.state(),
        DxfEntityXDataEntityDestinationState::Unavailable {
            unavailable_application_count: 1,
            capacity: DxfEntityXDataCapacityState::WithinLimit { .. },
            ..
        }
    ));
    assert!(
        directory
            .capacity_issues_for_entry(destination_only)?
            .is_empty()
    );

    let structural = directory.entries()[4];
    assert!(matches!(
        structural.state(),
        DxfEntityXDataEntityDestinationState::Unavailable {
            unavailable_application_count: 1,
            capacity: DxfEntityXDataCapacityState::Indeterminate { .. },
            ..
        }
    ));
    assert!(
        directory
            .capacity_issues_for_entry(structural)?
            .iter()
            .any(|issue| matches!(
                issue.kind(),
                DxfEntityXDataCapacityIssueKind::InvalidApplicationStructure { .. }
            ))
    );

    let exceeded = directory.entries()[5];
    assert!(matches!(
        exceeded.state(),
        DxfEntityXDataEntityDestinationState::Unavailable {
            unavailable_application_count: 0,
            capacity: DxfEntityXDataCapacityState::Exceeded { .. },
            ..
        }
    ));
    assert!(directory.capacity_issues_for_entry(exceeded)?.is_empty());
    Ok(())
}

#[test]
fn readiness_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataEntityDestinationDirectory>();
    assert_copy::<DxfEntityXDataEntityDestinationEntry>();
    assert_copy::<DxfEntityXDataEntityDestinationState>();
    let entry_size = size_of::<DxfEntityXDataEntityDestinationEntry>();
    assert!(
        entry_size <= 128,
        "entity destination entry is {entry_size} bytes"
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
        source.entity_xdata_entity_destination_directory(
            DxfRawDocumentView::from(&destination),
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));

    let directory = source.entity_xdata_entity_destination_directory(
        DxfRawDocumentView::from(&destination),
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_APP_READY"));
    assert!(!debug.contains("SECRET_LAYER_READY"));
    assert!(!debug.contains("SECRET_CAPACITY_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let other_destination_directory = source.entity_xdata_entity_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        &token(),
    )?;
    let foreign_destination_entry = other_destination_directory.entries()[0];
    assert!(
        directory
            .entity_for_entry(foreign_destination_entry)
            .is_err()
    );
    assert_eq!(
        directory.capacity_entry_for_entry(foreign_destination_entry),
        None
    );
    assert!(
        directory
            .application_entries_for_entry(foreign_destination_entry)
            .is_err()
    );

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_entity_destination_directory(
            DxfRawDocumentView::from(&destination),
            &token(),
        )?;
    let foreign_entity =
        other_source_directory.entity_for_entry(other_source_directory.entries()[0])?;
    assert!(matches!(
        directory.entry_for_entity(foreign_entity),
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
            assert_directory(&source.entity_xdata_entity_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(&source.entity_xdata_entity_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?)?;
        }
        _ => return Err(io::Error::other("cross-dialect test format pair").into()),
    }
    Ok(())
}

fn assert_directory(
    directory: &DxfEntityXDataEntityDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 7);
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.capacity_ordinal(), ordinal as u64);
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(entry.destination_id(), directory.destination_id());
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let entity = directory.entity_for_entry(entry)?;
        assert_eq!(entity.record().ordinal(), entry.raw_record_ordinal());
        assert_eq!(directory.entry_for_entity(entity)?, Some(entry));
        let capacity = directory
            .capacity_entry_for_entry(entry)
            .ok_or(io::Error::other("capacity entry"))?;
        assert_eq!(capacity.ordinal(), ordinal as u64);
        let applications = directory.application_entries_for_entry(entry)?;
        match ordinal {
            0 | 1 | 6 => assert_eq!(
                entry.state(),
                DxfEntityXDataEntityDestinationState::Ready {
                    application_count: 0,
                    used_bytes: 0,
                    remaining_bytes: DXF_XDATA_ENTITY_CAPACITY_BYTES,
                }
            ),
            2 => assert!(matches!(
                entry.state(),
                DxfEntityXDataEntityDestinationState::Ready {
                    application_count: 1,
                    used_bytes,
                    remaining_bytes,
                } if used_bytes > 0
                    && used_bytes + remaining_bytes == DXF_XDATA_ENTITY_CAPACITY_BYTES
                    && applications.len() == 1
            )),
            3 => assert!(matches!(
                entry.state(),
                DxfEntityXDataEntityDestinationState::Unavailable {
                    application_count: 1,
                    unavailable_application_count: 1,
                    capacity: DxfEntityXDataCapacityState::WithinLimit { .. },
                }
            )),
            4 => assert!(matches!(
                entry.state(),
                DxfEntityXDataEntityDestinationState::Unavailable {
                    application_count: 1,
                    unavailable_application_count: 1,
                    capacity: DxfEntityXDataCapacityState::Indeterminate { .. },
                }
            )),
            5 => assert!(matches!(
                entry.state(),
                DxfEntityXDataEntityDestinationState::Unavailable {
                    application_count: 1,
                    unavailable_application_count: 0,
                    capacity: DxfEntityXDataCapacityState::Exceeded {
                        used_bytes,
                        excess_bytes,
                    },
                } if used_bytes > DXF_XDATA_ENTITY_CAPACITY_BYTES
                    && excess_bytes == used_bytes - DXF_XDATA_ENTITY_CAPACITY_BYTES
            )),
            _ => return Err(io::Error::other("entity destination ordinal").into()),
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Group<'a>(i16, &'a [u8]);

fn source_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let oversized = [b'X'; 255];
    let mut body = vec![
        Group(0, b"SECTION"),
        Group(2, b"TABLES"),
        Group(0, b"TABLE"),
        Group(2, b"APPID"),
        Group(0, b"APPID"),
        Group(2, b"SECRET_APP_READY"),
        Group(0, b"APPID"),
        Group(2, b"APP_DESTINATION_BAD"),
        Group(0, b"APPID"),
        Group(2, b"APP_STRUCTURE_BAD"),
        Group(0, b"APPID"),
        Group(2, b"APP_OVERSIZED"),
        Group(0, b"ENDTAB"),
        Group(0, b"TABLE"),
        Group(2, b"LAYER"),
        Group(0, b"LAYER"),
        Group(2, b"SECRET_LAYER_READY"),
        Group(0, b"ENDTAB"),
        Group(0, b"ENDSEC"),
        Group(0, b"SECTION"),
        Group(2, b"ENTITIES"),
        Group(0, b"POINT"),
        Group(1001, b"SECRET_APP_READY"),
        Group(1003, b"SECRET_LAYER_READY"),
        Group(1000, b"SECRET_CAPACITY_PAYLOAD"),
        Group(0, b"LINE"),
        Group(1001, b"APP_DESTINATION_BAD"),
        Group(1000, b"A"),
        Group(0, b"CIRCLE"),
        Group(1001, b"APP_STRUCTURE_BAD"),
        Group(1002, b"{"),
        Group(1000, b"B"),
        Group(0, b"ARC"),
        Group(1001, b"APP_OVERSIZED"),
    ];
    for _ in 0..70 {
        body.push(Group(1000, &oversized));
    }
    body.extend([
        Group(0, b"TEXT"),
        Group(1, b"NO_XDATA"),
        Group(0, b"ENDSEC"),
    ]);
    let groups = document_groups(version, &body);
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
            Group(2, b"SECRET_APP_READY"),
            Group(1000, b"SECRET_DESTINATION_PAYLOAD"),
            Group(0, b"APPID"),
            Group(2, b"APP_STRUCTURE_BAD"),
            Group(0, b"APPID"),
            Group(2, b"APP_OVERSIZED"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"LAYER"),
            Group(2, b"SECRET_LAYER_READY"),
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

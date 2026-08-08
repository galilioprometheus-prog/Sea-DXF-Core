use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityXDataCoordinateDestinationDirectory,
    DxfEntityXDataCoordinateDestinationEntry, DxfEntityXDataCoordinateDestinationState,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataEntityDestinationState,
    DxfEntityXDataTransformedPointIssue, DxfEntityXDataTransformedPointState, DxfError,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_coordinate_readiness_parity()
-> Result<(), Box<dyn Error>> {
    let transform = transform()?;
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
            DxfRawDocumentView::from(&ascii_source).entity_xdata_coordinate_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                transform,
                &token(),
            )?,
            DxfRawDocumentView::from(&ascii_source).entity_xdata_coordinate_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                transform,
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source)
                .entity_xdata_coordinate_destination_directory(
                    DxfRawDocumentView::from(&ascii_destination),
                    transform,
                    &token(),
                )?,
            DxfRawDocumentView::from(&binary_source)
                .entity_xdata_coordinate_destination_directory(
                    DxfRawDocumentView::from(&binary_destination),
                    transform,
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
        transform,
    )?;
    assert_cross_dialect_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        transform,
    )?;
    Ok(())
}

#[test]
fn entity_destination_and_transform_failures_remain_independent() -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_coordinate_destination_directory(
        DxfRawDocumentView::from(&destination),
        transform()?,
        &token(),
    )?;

    let transformed_only = directory.entries()[2];
    assert!(matches!(
        transformed_only.state(),
        DxfEntityXDataCoordinateDestinationState::Unavailable {
            entity_state: DxfEntityXDataEntityDestinationState::Ready { .. },
            tuple_count: 1,
            unavailable_tuple_count: 1,
        }
    ));
    assert!(matches!(
        directory.transformed_entries_for_entry(transformed_only)?[0].state(),
        DxfEntityXDataTransformedPointState::Unavailable(
            DxfEntityXDataTransformedPointIssue::NonFiniteDerivedPoint
        )
    ));

    let destination_only = directory.entries()[3];
    assert!(matches!(
        destination_only.state(),
        DxfEntityXDataCoordinateDestinationState::Unavailable {
            entity_state: DxfEntityXDataEntityDestinationState::Unavailable { .. },
            tuple_count: 0,
            unavailable_tuple_count: 0,
        }
    ));

    let partial = directory.entries()[5];
    assert!(matches!(
        directory.transformed_entries_for_entry(partial)?[0].state(),
        DxfEntityXDataTransformedPointState::Unavailable(
            DxfEntityXDataTransformedPointIssue::PartialTuple(components)
        ) if components.has_x() && components.has_y() && !components.has_z()
    ));
    Ok(())
}

#[test]
fn readiness_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataCoordinateDestinationDirectory>();
    assert_copy::<DxfEntityXDataCoordinateDestinationEntry>();
    assert_copy::<DxfEntityXDataCoordinateDestinationState>();
    let entry_size = size_of::<DxfEntityXDataCoordinateDestinationEntry>();
    assert!(
        entry_size <= 160,
        "coordinate destination entry is {entry_size} bytes"
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
        source.entity_xdata_coordinate_destination_directory(
            DxfRawDocumentView::from(&destination),
            transform()?,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = source.entity_xdata_coordinate_destination_directory(
        DxfRawDocumentView::from(&destination),
        transform()?,
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_APP_READY"));
    assert!(!debug.contains("SECRET_COORDINATE_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let other = source.entity_xdata_coordinate_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        transform()?,
        &token(),
    )?;
    assert_eq!(
        directory.entity_destination_for_entry(other.entries()[2]),
        None
    );
    assert!(
        directory
            .transformed_entries_for_entry(other.entries()[2])
            .is_err()
    );

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_coordinate_destination_directory(
            DxfRawDocumentView::from(&destination),
            transform()?,
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

fn assert_cross_dialect_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
    transform: DxfEntityXDataCoordinateTransform,
) -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(source_format, source_version)?;
    let destination_bytes = destination_fixture(destination_format, destination_version)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    match (source_format, destination_format) {
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_directory(&source.entity_xdata_coordinate_destination_directory(
                DxfRawDocumentView::from(&destination),
                transform,
                &token(),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(&source.entity_xdata_coordinate_destination_directory(
                DxfRawDocumentView::from(&destination),
                transform,
                &token(),
            )?)?;
        }
        _ => return Err(io::Error::other("cross-dialect pair").into()),
    }
    Ok(())
}

fn assert_directory(
    directory: &DxfEntityXDataCoordinateDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 6);
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.entity_destination_ordinal(), ordinal as u64);
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let entity_destination = directory
            .entity_destination_for_entry(entry)
            .ok_or(io::Error::other("entity destination"))?;
        let entity = directory.entity_for_entry(entry)?;
        assert_eq!(directory.entry_for_entity(entity)?, Some(entry));
        let transformed = directory.transformed_entries_for_entry(entry)?;
        match ordinal {
            0 | 4 => assert!(matches!(
                entry.state(),
                DxfEntityXDataCoordinateDestinationState::Ready {
                    application_count: 0,
                    transformed_tuple_count: 0,
                    ..
                }
            )),
            1 => assert!(matches!(
                entry.state(),
                DxfEntityXDataCoordinateDestinationState::Ready {
                    application_count: 1,
                    transformed_tuple_count: 1,
                    ..
                }
            )),
            2 => assert!(matches!(
                entry.state(),
                DxfEntityXDataCoordinateDestinationState::Unavailable {
                    entity_state: DxfEntityXDataEntityDestinationState::Ready { .. },
                    tuple_count: 1,
                    unavailable_tuple_count: 1,
                }
            )),
            3 => assert!(matches!(
                entry.state(),
                DxfEntityXDataCoordinateDestinationState::Unavailable {
                    tuple_count: 0,
                    unavailable_tuple_count: 0,
                    ..
                }
            )),
            5 => assert!(matches!(
                entry.state(),
                DxfEntityXDataCoordinateDestinationState::Unavailable {
                    tuple_count: 1,
                    unavailable_tuple_count: 1,
                    ..
                }
            )),
            _ => return Err(io::Error::other("coordinate ordinal").into()),
        }
        assert_eq!(
            transformed.len(),
            usize::from(!matches!(ordinal, 0 | 3 | 4))
        );
        assert_eq!(entity_destination.ordinal(), ordinal as u64);
    }
    Ok(())
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
            Group(2, b"SECRET_APP_READY"),
            Group(0, b"APPID"),
            Group(2, b"APP_DESTINATION_BAD"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(1001, b"SECRET_APP_READY"),
            Group(1010, b"1"),
            Group(1020, b"2"),
            Group(1030, b"3"),
            Group(1000, b"SECRET_COORDINATE_PAYLOAD"),
            Group(0, b"LINE"),
            Group(1001, b"SECRET_APP_READY"),
            Group(1011, b"1.7976931348623157e308"),
            Group(1021, b"2"),
            Group(1031, b"2"),
            Group(0, b"CIRCLE"),
            Group(1001, b"APP_DESTINATION_BAD"),
            Group(1000, b"A"),
            Group(0, b"ARC"),
            Group(0, b"TEXT"),
            Group(1001, b"SECRET_APP_READY"),
            Group(1012, b"1"),
            Group(1022, b"2"),
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
            Group(2, b"SECRET_APP_READY"),
            Group(1000, b"SECRET_DESTINATION_PAYLOAD"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn transform() -> Result<DxfEntityXDataCoordinateTransform, Box<dyn Error>> {
    DxfEntityXDataCoordinateTransform::translation([
        DxfDouble::from_f64(f64::MAX),
        DxfDouble::from_f64(0.0),
        DxfDouble::from_f64(0.0),
    ])
    .map_err(|issue| io::Error::other(format!("{issue:?}")).into())
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
    let mut o = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut o)
}
fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut o = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(source, DxfReadOptions::strict(), &token(), &mut o)
}
fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
}
fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

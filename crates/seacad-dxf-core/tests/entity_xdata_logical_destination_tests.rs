use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityXDataAppIdDestinationState,
    DxfEntityXDataCoordinateTransform, DxfEntityXDataHandleDestinationState,
    DxfEntityXDataHandleRemap, DxfEntityXDataLayerDestinationState,
    DxfEntityXDataLogicalDestinationDirectory, DxfEntityXDataLogicalDestinationEntry,
    DxfEntityXDataLogicalDestinationIssue, DxfEntityXDataLogicalDestinationState,
    DxfEntityXDataLogicalDestinationValue, DxfEntityXDataTransformedPointIssue,
    DxfEntityXDataValueIssue, DxfError, DxfHandle, DxfMemorySource, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_logical_destination_parity()
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
fn specialized_and_source_exact_values_remain_independent() -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_logical_destination_directory(
        DxfRawDocumentView::from(&destination),
        transform()?,
        &mappings()?,
        &token(),
    )?;

    assert!(matches!(
        directory.entries()[0].state(),
        DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Application { .. }
        )
    ));
    for ordinal in [1_usize, 2, 4, 9, 10, 11, 12, 14] {
        assert!(matches!(
            directory.entries()[ordinal].state(),
            DxfEntityXDataLogicalDestinationState::Available(
                DxfEntityXDataLogicalDestinationValue::SourceExact(_)
            )
        ));
    }
    assert!(matches!(
        directory.entries()[3].state(),
        DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Layer { .. }
        )
    ));
    assert_eq!(
        directory.entries()[5].state(),
        DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Handle {
                target: DxfHandle::from_u64(0x200),
            }
        )
    );
    let expected_coordinates = [11.0, 22.0, 33.0];
    for (entry, expected) in directory.entries()[6..9].iter().zip(expected_coordinates) {
        assert!(matches!(
            entry.state(),
            DxfEntityXDataLogicalDestinationState::Available(
                DxfEntityXDataLogicalDestinationValue::TransformedDouble { value, .. }
            ) if value == DxfDouble::from_f64(expected)
        ));
    }
    assert!(matches!(
        directory.entries()[13].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Application(
                DxfEntityXDataAppIdDestinationState::DestinationMissing
            )
        )
    ));
    assert!(matches!(
        directory.entries()[16].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Layer(
                DxfEntityXDataLayerDestinationState::DestinationMissing
            )
        )
    ));
    assert_eq!(
        directory.entries()[18].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Handle(
                DxfEntityXDataHandleDestinationState::Missing {
                    target: DxfHandle::from_u64(0x300),
                }
            )
        )
    );
    for entry in &directory.entries()[20..22] {
        assert!(matches!(
            entry.state(),
            DxfEntityXDataLogicalDestinationState::Unavailable(
                DxfEntityXDataLogicalDestinationIssue::Coordinate(
                    DxfEntityXDataTransformedPointIssue::PartialTuple(components)
                )
            ) if components.has_x() && components.has_y() && !components.has_z()
        ));
    }
    assert_eq!(
        directory.entries()[22].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Orphan
        )
    );
    assert!(matches!(
        directory.entries()[24].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::InvalidSource(
                DxfEntityXDataValueIssue::InvalidControlString
            )
        )
    ));
    Ok(())
}

#[test]
fn logical_projection_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataLogicalDestinationDirectory>();
    assert_copy::<DxfEntityXDataLogicalDestinationEntry>();
    assert_copy::<DxfEntityXDataLogicalDestinationState>();
    let entry_size = size_of::<DxfEntityXDataLogicalDestinationEntry>();
    assert!(
        entry_size <= 192,
        "logical destination entry is {entry_size} bytes"
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
        source.entity_xdata_logical_destination_directory(
            DxfRawDocumentView::from(&destination),
            transform()?,
            &[],
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = source.entity_xdata_logical_destination_directory(
        DxfRawDocumentView::from(&destination),
        transform()?,
        &mappings()?,
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entries().len(), 25);
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_EXACT_PAYLOAD"));
    assert!(!debug.contains("SECRET_ORPHAN_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.typed_ordinal(), ordinal as u64);
        assert!(directory.typed_for_entry(entry).is_some());
        assert!(directory.payload_entry_for_entry(entry).is_ok());
    }

    let ready_payload = directory.payload_destination_directory().entries()[4];
    let ready_entity = directory
        .payload_destination_directory()
        .entity_for_entry(ready_payload)?;
    assert_eq!(directory.entries_for_entity(ready_entity)?.len(), 13);

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let foreign_destination = source.entity_xdata_logical_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        transform()?,
        &mappings()?,
        &token(),
    )?;
    assert_eq!(
        directory.typed_for_entry(foreign_destination.entries()[0]),
        None
    );
    assert!(
        directory
            .payload_entry_for_entry(foreign_destination.entries()[0])
            .is_err()
    );

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_logical_destination_directory(
            DxfRawDocumentView::from(&destination),
            transform()?,
            &mappings()?,
            &token(),
        )?;
    let foreign_payload = other_source_directory
        .payload_destination_directory()
        .entries()[4];
    let foreign_entity = other_source_directory
        .payload_destination_directory()
        .entity_for_entry(foreign_payload)?;
    assert!(matches!(
        directory.entries_for_entity(foreign_entity),
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
) -> Result<DxfEntityXDataLogicalDestinationDirectory, Box<dyn Error>> {
    Ok(source.entity_xdata_logical_destination_directory(
        destination,
        transform()?,
        &mappings()?,
        &token(),
    )?)
}

fn assert_directory(
    directory: &DxfEntityXDataLogicalDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 25);
    assert!(matches!(
        directory.entries()[0].state(),
        DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Application { .. }
        )
    ));
    assert!(matches!(
        directory.entries()[3].state(),
        DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Layer { .. }
        )
    ));
    assert!(matches!(
        directory.entries()[5].state(),
        DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::Handle { target }
        ) if target == DxfHandle::from_u64(0x200)
    ));
    assert!(matches!(
        directory.entries()[6].state(),
        DxfEntityXDataLogicalDestinationState::Available(
            DxfEntityXDataLogicalDestinationValue::TransformedDouble { value, .. }
        ) if value == DxfDouble::from_f64(11.0)
    ));
    assert!(matches!(
        directory.entries()[13].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Application(_)
        )
    ));
    assert!(matches!(
        directory.entries()[16].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Layer(_)
        )
    ));
    assert!(matches!(
        directory.entries()[18].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Handle(_)
        )
    ));
    assert!(matches!(
        directory.entries()[20].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Coordinate(_)
        )
    ));
    assert_eq!(
        directory.entries()[22].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::Orphan
        )
    );
    assert!(matches!(
        directory.entries()[24].state(),
        DxfEntityXDataLogicalDestinationState::Unavailable(
            DxfEntityXDataLogicalDestinationIssue::InvalidSource(_)
        )
    ));
    Ok(())
}

fn transform() -> Result<DxfEntityXDataCoordinateTransform, Box<dyn Error>> {
    DxfEntityXDataCoordinateTransform::translation([
        DxfDouble::from_f64(10.0),
        DxfDouble::from_f64(20.0),
        DxfDouble::from_f64(30.0),
    ])
    .map_err(|issue| io::Error::other(format!("{issue:?}")).into())
}

fn mappings() -> Result<[DxfEntityXDataHandleRemap; 2], io::Error> {
    Ok([remap(0xA, 0x200)?, remap(0xB, 0x300)?])
}

fn remap(source: u64, target: u64) -> Result<DxfEntityXDataHandleRemap, io::Error> {
    DxfEntityXDataHandleRemap::new(DxfHandle::from_u64(source), DxfHandle::from_u64(target))
        .map_err(|issue| io::Error::other(format!("{issue:?}")))
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Chunk(&'a [u8]),
    Double(f64),
    Int16(i16),
    Int32(i32),
}

fn source_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut groups = document_groups(version);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_READY")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_MISSING")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"LAYER")),
        (0, Value::Text(b"LAYER")),
        (2, Value::Text(b"LAYER_READY")),
        (0, Value::Text(b"LAYER")),
        (2, Value::Text(b"LAYER_MISSING")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (5, Value::Text(b"A")),
        (0, Value::Text(b"POINT")),
        (5, Value::Text(b"B")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (1001, Value::Text(b"APP_READY")),
        (1000, Value::Text(b"SECRET_EXACT_PAYLOAD")),
        (1002, Value::Text(b"{")),
        (1003, Value::Text(b"LAYER_READY")),
        (1004, Value::Chunk(&[0xAB, 0xCD, 0x01])),
        (1005, Value::Text(b"A")),
        (1011, Value::Double(1.0)),
        (1021, Value::Double(2.0)),
        (1031, Value::Double(3.0)),
        (1040, Value::Double(4.5)),
        (1070, Value::Int16(-7)),
        (1071, Value::Int32(123)),
        (1002, Value::Text(b"}")),
        (0, Value::Text(b"CIRCLE")),
        (5, Value::Text(b"11")),
        (1001, Value::Text(b"APP_MISSING")),
        (1000, Value::Text(b"A")),
        (0, Value::Text(b"ARC")),
        (5, Value::Text(b"12")),
        (1001, Value::Text(b"APP_READY")),
        (1003, Value::Text(b"LAYER_MISSING")),
        (0, Value::Text(b"TEXT")),
        (5, Value::Text(b"13")),
        (1001, Value::Text(b"APP_READY")),
        (1005, Value::Text(b"B")),
        (0, Value::Text(b"ELLIPSE")),
        (5, Value::Text(b"14")),
        (1001, Value::Text(b"APP_READY")),
        (1012, Value::Double(1.0)),
        (1022, Value::Double(2.0)),
        (0, Value::Text(b"SPLINE")),
        (5, Value::Text(b"15")),
        (1000, Value::Text(b"SECRET_ORPHAN_PAYLOAD")),
        (0, Value::Text(b"SHAPE")),
        (5, Value::Text(b"16")),
        (1001, Value::Text(b"APP_READY")),
        (1002, Value::Text(b"x")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    encode(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let mut groups = document_groups(version);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_READY")),
        (1000, Value::Text(b"SECRET_DESTINATION_PAYLOAD")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"LAYER")),
        (0, Value::Text(b"LAYER")),
        (2, Value::Text(b"LAYER_READY")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (5, Value::Text(b"200")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    encode(format, version, &groups)
}

fn document_groups(version: DxfAcadVersion) -> Vec<(i16, Value<'static>)> {
    vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
    ]
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> io::Result<Vec<u8>> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Chunk(value) => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                for byte in *value {
                    bytes.push(HEX[usize::from(byte >> 4)]);
                    bytes.push(HEX[usize::from(byte & 0xF)]);
                }
            }
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.push(b'\n');
    }
    bytes
}

fn binary_groups(version: DxfAcadVersion, groups: &[(i16, Value<'_>)]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        push_code(&mut bytes, version, *code)?;
        match value {
            Value::Text(value) => {
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            Value::Chunk(value) => {
                bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk"))?);
                bytes.extend_from_slice(value);
            }
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
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

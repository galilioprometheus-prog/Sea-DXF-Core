use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonFieldDomainIssue, DxfEntityCommonFieldDomainSemanticIssue,
    DxfEntityProxyGraphicsChunkIssue, DxfEntityProxyGraphicsDirectory, DxfEntityProxyGraphicsState,
    DxfError, DxfMemorySource, DxfRawDocumentFormat, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_proxy_size_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.entity_proxy_graphics_directory(&token())?;

        let binary_bytes = fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.entity_proxy_graphics_directory(&token())?;

        let expected = if version == DxfAcadVersion::Ac1009 {
            DxfEntityProxyGraphicsState::Matched {
                declared_bytes: 0,
                payload_bytes: 0,
                chunk_count: 0,
            }
        } else {
            DxfEntityProxyGraphicsState::Matched {
                declared_bytes: 3,
                payload_bytes: 3,
                chunk_count: 2,
            }
        };
        assert_eq!(only_state(&ascii_directory)?, expected);
        assert_eq!(only_state(&binary_directory)?, expected);
        assert_eq!(
            only_state(&ascii_directory)?,
            only_state(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn absence_missing_size_mismatch_and_invalid_size_stay_distinct() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let groups = relation_state_groups();
        let bytes = encode(format, DxfAcadVersion::Ac1032, &groups)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let states = match format {
            DxfRawDocumentFormat::Ascii => open_ascii(&source)?
                .entity_proxy_graphics_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.state())
                .collect::<Vec<_>>(),
            DxfRawDocumentFormat::Binary => open_binary(&source)?
                .entity_proxy_graphics_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.state())
                .collect::<Vec<_>>(),
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(states.len(), 4);
        assert!(matches!(
            states[0],
            DxfEntityProxyGraphicsState::CountMismatch {
                declared_bytes: 4,
                payload_bytes: 3,
                chunk_count: 2
            }
        ));
        assert!(matches!(
            states[1],
            DxfEntityProxyGraphicsState::MissingSize {
                payload_bytes: 1,
                chunk_count: 1
            }
        ));
        assert_eq!(states[2], DxfEntityProxyGraphicsState::Absent);
        assert!(matches!(
            states[3],
            DxfEntityProxyGraphicsState::InvalidSize {
                issue: DxfEntityCommonFieldDomainSemanticIssue::Domain(
                    DxfEntityCommonFieldDomainIssue::UnsupportedInt32 { value: -1, .. }
                )
            }
        ));
    }
    Ok(())
}

#[test]
fn malformed_ascii_chunks_report_first_exact_failure() -> Result<(), Box<dyn Error>> {
    let groups = document_groups(&[
        (
            "LINE",
            &[(92, Value::Int32(1)), (310, Value::AsciiChunk(b"ABC"))],
        ),
        (
            "CIRCLE",
            &[(92, Value::Int32(1)), (310, Value::AsciiChunk(b"0G"))],
        ),
    ]);
    let bytes = encode(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, &groups)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_proxy_graphics_directory(&token())?;
    let states = directory
        .entries()
        .iter()
        .map(|entry| entry.state())
        .collect::<Vec<_>>();
    assert!(matches!(
        states[0],
        DxfEntityProxyGraphicsState::InvalidChunk {
            sequence_ordinal: 0,
            issue: DxfEntityProxyGraphicsChunkIssue::OddHexLength { encoded_bytes: 3 },
            ..
        }
    ));
    let DxfEntityProxyGraphicsState::InvalidChunk {
        raw,
        issue: DxfEntityProxyGraphicsChunkIssue::InvalidHexDigit { byte_offset: 1 },
        ..
    } = states[1]
    else {
        return Err(io::Error::other("invalid hex digit").into());
    };
    assert_eq!(raw.value_span().len(), 2);
    assert!(
        raw.group_occurrence() < seacad_dxf_core::DxfRawDocumentView::from(&document).group_count()
    );
    Ok(())
}

#[test]
fn cancellation_identity_lookup_and_public_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityProxyGraphicsDirectory>();
    assert_copy::<seacad_dxf_core::DxfEntityProxyGraphicsEntry>();
    assert_copy::<DxfEntityProxyGraphicsState>();
    assert!(std::mem::size_of::<seacad_dxf_core::DxfEntityProxyGraphicsEntry>() <= 512);

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_proxy_graphics_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let directory = document.entity_proxy_graphics_directory(&token())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.domain_directory().source_id(),
        document.source_id()
    );
    let entity = directory
        .entries()
        .first()
        .ok_or(io::Error::other("entity"))?
        .entity();
    assert_eq!(directory.entry_for_entity(entity)?, directory.entry(0));

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let other_directory = other.entity_proxy_graphics_directory(&token())?;
    let other_entity = other_directory
        .entries()
        .first()
        .ok_or(io::Error::other("other entity"))?
        .entity();
    assert!(matches!(
        directory.entry_for_entity(other_entity),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn only_state(
    directory: &DxfEntityProxyGraphicsDirectory,
) -> Result<DxfEntityProxyGraphicsState, Box<dyn Error>> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one proxy relation").into());
    };
    Ok(entry.state())
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int32(i32),
    AsciiChunk(&'a [u8]),
    Chunk(&'a [u8]),
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let fields = if version == DxfAcadVersion::Ac1009 {
        vec![(92, Value::Int32(0))]
    } else {
        vec![
            (310, Value::Chunk(b"\x01\x02")),
            (999, Value::Text(b"UNKNOWN")),
            (92, Value::Int32(3)),
            (310, Value::Chunk(b"\x03")),
        ]
    };
    let mut groups = document_groups(&[("LINE", &fields)]);
    groups[3] = (1, Value::Text(version.code().as_bytes()));
    encode(format, version, &groups)
}

fn relation_state_groups<'a>() -> Vec<(i16, Value<'a>)> {
    document_groups(&[
        (
            "LINE",
            &[
                (310, Value::Chunk(b"\x01\x02")),
                (92, Value::Int32(4)),
                (310, Value::Chunk(b"\x03")),
            ],
        ),
        ("CIRCLE", &[(310, Value::Chunk(b"\x04"))]),
        ("ARC", &[]),
        ("POINT", &[(92, Value::Int32(-1))]),
    ])
}

fn document_groups<'a>(entities: &[(&'a str, &[(i16, Value<'a>)])]) -> Vec<(i16, Value<'a>)> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(b"AC1032")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
    ];
    for (index, (marker, fields)) in entities.iter().enumerate() {
        groups.push((0, Value::Text(marker.as_bytes())));
        groups.push((5, Value::Text(if index == 0 { b"10" } else { b"11" })));
        groups.push((100, Value::Text(b"AcDbEntity")));
        groups.extend_from_slice(fields);
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    groups
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = if format == DxfRawDocumentFormat::Binary {
        DXF_BINARY_SENTINEL.to_vec()
    } else {
        Vec::new()
    };
    for (code, value) in groups {
        if format == DxfRawDocumentFormat::Ascii {
            bytes.extend_from_slice(code.to_string().as_bytes());
            bytes.extend_from_slice(b"\r\n");
            match value {
                Value::Text(value) | Value::AsciiChunk(value) => bytes.extend_from_slice(value),
                Value::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
                Value::Chunk(value) => {
                    for byte in *value {
                        bytes.extend_from_slice(format!("{byte:02X}").as_bytes());
                    }
                }
            }
            bytes.extend_from_slice(b"\r\n");
            continue;
        }
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        match value {
            Value::Text(value) | Value::AsciiChunk(value) => {
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            Value::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Chunk(value) => {
                bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk"))?);
                bytes.extend_from_slice(value);
            }
        }
    }
    Ok(bytes)
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

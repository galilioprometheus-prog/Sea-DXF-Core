use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataControl, DxfEntityXDataDoubleRole, DxfEntityXDataTextKind,
    DxfEntityXDataTypedDirectory, DxfEntityXDataTypedEntry, DxfEntityXDataValue,
    DxfEntityXDataValueIssue, DxfError, DxfHandleParseIssue, DxfMemorySource, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_typed_xdata_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = valid_fixture(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            match format {
                DxfRawDocumentFormat::Ascii => {
                    let document = open_ascii(&source)?;
                    assert_valid(DxfRawDocumentView::from(&document))?;
                }
                DxfRawDocumentFormat::Binary => {
                    let document = open_binary(&source)?;
                    assert_valid(DxfRawDocumentView::from(&document))?;
                }
                _ => return Err(io::Error::other("test format").into()),
            }
        }
    }
    Ok(())
}

#[test]
fn malformed_domains_remain_typed_without_normalization() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = malformed_fixture(format)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let issues = match format {
            DxfRawDocumentFormat::Ascii => {
                collect_issues(&open_ascii(&source)?.entity_xdata_typed_directory(&token())?)
            }
            DxfRawDocumentFormat::Binary => {
                collect_issues(&open_binary(&source)?.entity_xdata_typed_directory(&token())?)
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        assert!(issues.contains(&DxfEntityXDataValueIssue::StringTooLong {
            observed_bytes: 256
        }));
        assert!(issues.contains(&DxfEntityXDataValueIssue::InvalidControlString));
        assert!(
            issues.contains(&DxfEntityXDataValueIssue::BinaryChunkTooLong {
                observed_bytes: 128
            })
        );
        assert!(issues.contains(&DxfEntityXDataValueIssue::InvalidHandle(
            DxfHandleParseIssue::InvalidDigit { offset: 0 }
        )));
        assert!(
            issues.contains(&DxfEntityXDataValueIssue::UnsupportedGroupCode { group_code: 1006 })
        );
        assert!(issues.iter().any(|issue| matches!(
            issue,
            DxfEntityXDataValueIssue::InvalidAsciiNumber(_)
                | DxfEntityXDataValueIssue::NonFiniteDouble(_)
        )));
        if format == DxfRawDocumentFormat::Ascii {
            assert!(
                issues.contains(&DxfEntityXDataValueIssue::OddBinaryChunkHexLength {
                    encoded_bytes: 3
                })
            );
            assert!(
                issues.contains(&DxfEntityXDataValueIssue::InvalidBinaryChunkHexDigit {
                    byte_offset: 1
                })
            );
        }
    }
    Ok(())
}

#[test]
fn chunk_read_lookup_source_cancellation_and_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataTypedDirectory>();
    assert_copy::<DxfEntityXDataTypedEntry>();
    assert_copy::<DxfEntityXDataValue>();
    assert!(std::mem::size_of::<DxfEntityXDataTypedEntry>() <= 256);

    let bytes = valid_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = document.entity_xdata_typed_directory(&token())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let chunk = directory.entries()[4];
    assert_eq!(
        directory.entry_for_group(chunk.occurrence().group().occurrence()),
        Some(chunk)
    );
    assert_eq!(directory.entry_for_group(u64::MAX), None);
    let mut decoded = [0_u8; 3];
    directory.read_binary_chunk(view, chunk, &mut decoded, &token())?;
    assert_eq!(decoded, [0xAB, 0xCD, 0x01]);
    let application = directory.xdata_directory().applications()[0];
    assert_eq!(directory.entries_for_application(application)?.len(), 24);
    assert!(!format!("{directory:?}").contains("SECRET_PAYLOAD"));

    let other_bytes = valid_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other = other_document.entity_xdata_typed_directory(&token())?;
    assert!(matches!(
        directory.entries_for_application(other.xdata_directory().applications()[0]),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(matches!(
        directory.read_binary_chunk(
            DxfRawDocumentView::from(&other_document),
            chunk,
            &mut decoded,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.entity_xdata_typed_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        directory.read_binary_chunk(view, chunk, &mut decoded, &cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_valid(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let directory = view.entity_xdata_typed_directory(&token())?;
    assert_eq!(directory.entries().len(), 24);
    assert!(matches!(
        directory.entries()[0].value(),
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::ApplicationName,
            ..
        }
    ));
    assert!(matches!(
        directory.entries()[1].value(),
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::String,
            ..
        }
    ));
    assert!(matches!(
        directory.entries()[2].value(),
        DxfEntityXDataValue::Control {
            control: DxfEntityXDataControl::OpenList,
            ..
        }
    ));
    assert!(matches!(
        directory.entries()[3].value(),
        DxfEntityXDataValue::ExactText {
            kind: DxfEntityXDataTextKind::LayerName,
            ..
        }
    ));
    assert!(matches!(
        directory.entries()[4].value(),
        DxfEntityXDataValue::BinaryChunk {
            decoded_bytes: 3,
            ..
        }
    ));
    assert!(matches!(
        directory.entries()[5].value(),
        DxfEntityXDataValue::Handle { value, .. } if value.value() == 0xAB
    ));
    let expected_roles = [
        DxfEntityXDataDoubleRole::PointX,
        DxfEntityXDataDoubleRole::WorldPositionX,
        DxfEntityXDataDoubleRole::WorldDisplacementX,
        DxfEntityXDataDoubleRole::WorldDirectionX,
        DxfEntityXDataDoubleRole::PointY,
        DxfEntityXDataDoubleRole::WorldPositionY,
        DxfEntityXDataDoubleRole::WorldDisplacementY,
        DxfEntityXDataDoubleRole::WorldDirectionY,
        DxfEntityXDataDoubleRole::PointZ,
        DxfEntityXDataDoubleRole::WorldPositionZ,
        DxfEntityXDataDoubleRole::WorldDisplacementZ,
        DxfEntityXDataDoubleRole::WorldDirectionZ,
        DxfEntityXDataDoubleRole::Real,
        DxfEntityXDataDoubleRole::Distance,
        DxfEntityXDataDoubleRole::ScaleFactor,
    ];
    for (entry, expected) in directory.entries()[6..21].iter().zip(expected_roles) {
        assert!(matches!(
            entry.value(),
            DxfEntityXDataValue::Double { role, value, .. }
                if role == expected && value.to_f64() == 1.25
        ));
    }
    assert!(matches!(
        directory.entries()[21].value(),
        DxfEntityXDataValue::Int16 { value: -7, .. }
    ));
    assert!(matches!(
        directory.entries()[22].value(),
        DxfEntityXDataValue::Int32 { value: 123_456, .. }
    ));
    assert!(matches!(
        directory.entries()[23].value(),
        DxfEntityXDataValue::Control {
            control: DxfEntityXDataControl::CloseList,
            ..
        }
    ));
    let mut decoded = [0_u8; 3];
    directory.read_binary_chunk(view, directory.entries()[4], &mut decoded, &token())?;
    assert_eq!(decoded, [0xAB, 0xCD, 0x01]);
    Ok(())
}

fn collect_issues(directory: &DxfEntityXDataTypedDirectory) -> Vec<DxfEntityXDataValueIssue> {
    directory
        .entries()
        .iter()
        .filter_map(|entry| match entry.value() {
            DxfEntityXDataValue::Invalid { issue, .. } => Some(issue),
            _ => None,
        })
        .collect()
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Chunk(&'a [u8]),
    EncodedChunk(&'a [u8]),
    Double(f64),
    Int16(i16),
    Int32(i32),
}

fn valid_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut groups = header(version);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP")),
        (1000, Value::Text(b"SECRET_PAYLOAD")),
        (1002, Value::Text(b"{")),
        (1003, Value::Text(b"LayerA")),
        (1004, Value::Chunk(&[0xAB, 0xCD, 0x01])),
        (1005, Value::Text(b"aB")),
    ]);
    for code in [
        1010, 1011, 1012, 1013, 1020, 1021, 1022, 1023, 1030, 1031, 1032, 1033, 1040, 1041, 1042,
    ] {
        groups.push((code, Value::Double(1.25)));
    }
    groups.extend([
        (1070, Value::Int16(-7)),
        (1071, Value::Int32(123_456)),
        (1002, Value::Text(b"}")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    encode(format, version, &groups)
}

fn malformed_fixture(format: DxfRawDocumentFormat) -> io::Result<Vec<u8>> {
    const LONG: [u8; 256] = [b'X'; 256];
    const LARGE_CHUNK: [u8; 128] = [0xA5; 128];
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP")),
        (1000, Value::Text(&LONG)),
        (1002, Value::Text(b"x")),
        (1004, Value::Chunk(&LARGE_CHUNK)),
        (1005, Value::Text(b"G")),
        (1040, Value::Double(f64::NAN)),
        (1006, Value::Text(b"UNSUPPORTED")),
    ]);
    if format == DxfRawDocumentFormat::Ascii {
        groups.extend([
            (1004, Value::EncodedChunk(b"ABC")),
            (1004, Value::EncodedChunk(b"0G")),
        ]);
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, DxfAcadVersion::Ac1032, &groups)
}

fn header(version: DxfAcadVersion) -> Vec<(i16, Value<'static>)> {
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
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        match value {
            Value::Text(value) | Value::EncodedChunk(value) => bytes.extend_from_slice(value),
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
            Value::EncodedChunk(_) => return Err(io::Error::other("encoded binary chunk")),
        }
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

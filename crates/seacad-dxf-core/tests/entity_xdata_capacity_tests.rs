use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_XDATA_ENTITY_CAPACITY_BYTES, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfEntityXDataCapacityDirectory,
    DxfEntityXDataCapacityEntry, DxfEntityXDataCapacityIssue, DxfEntityXDataCapacityIssueKind,
    DxfEntityXDataCapacityState, DxfEntityXDataCapacityTextIssue, DxfError, DxfMemorySource,
    DxfRawDocumentFormat, DxfRawRecordSectionKind, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_xdsize_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = exact_fixture(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let directory = match format {
                DxfRawDocumentFormat::Ascii => {
                    open_ascii(&source)?.entity_xdata_capacity_directory(&token())?
                }
                DxfRawDocumentFormat::Binary => {
                    open_binary(&source)?.entity_xdata_capacity_directory(&token())?
                }
                _ => return Err(io::Error::other("test format").into()),
            };
            let entries = entity_entries(&directory)?;
            assert_eq!(entries.len(), 2);
            assert_eq!(
                entries[0].state(),
                DxfEntityXDataCapacityState::WithinLimit {
                    used_bytes: 0,
                    remaining_bytes: DXF_XDATA_ENTITY_CAPACITY_BYTES,
                }
            );
            assert_eq!(
                entries[1].state(),
                DxfEntityXDataCapacityState::WithinLimit {
                    used_bytes: 174,
                    remaining_bytes: DXF_XDATA_ENTITY_CAPACITY_BYTES - 174,
                }
            );
            assert!(directory.issues().is_empty());
        }
    }
    Ok(())
}

#[test]
fn exact_limit_excess_and_invalid_inputs_are_distinct() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = boundary_and_invalid_fixture(format)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => {
                open_ascii(&source)?.entity_xdata_capacity_directory(&token())?
            }
            DxfRawDocumentFormat::Binary => {
                open_binary(&source)?.entity_xdata_capacity_directory(&token())?
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        let entries = entity_entries(&directory)?;
        assert_eq!(entries.len(), 10);
        assert_eq!(
            entries[0].state(),
            DxfEntityXDataCapacityState::WithinLimit {
                used_bytes: 16_383,
                remaining_bytes: 0,
            }
        );
        assert_eq!(
            entries[1].state(),
            DxfEntityXDataCapacityState::Exceeded {
                used_bytes: 16_384,
                excess_bytes: 1,
            }
        );
        assert_eq!(
            entries[2].state(),
            DxfEntityXDataCapacityState::WithinLimit {
                used_bytes: 12,
                remaining_bytes: DXF_XDATA_ENTITY_CAPACITY_BYTES - 12,
            }
        );
        for entry in &entries[3..] {
            assert!(matches!(
                entry.state(),
                DxfEntityXDataCapacityState::Indeterminate { issue_count, .. }
                    if issue_count != 0
            ));
        }
        let kinds: Vec<_> = directory
            .issues()
            .iter()
            .map(|issue| issue.kind())
            .collect();
        assert!(
            kinds
                .iter()
                .any(|kind| matches!(kind, DxfEntityXDataCapacityIssueKind::OrphanValue { .. }))
        );
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::ApplicationResolution {
                target_count: 0,
                ..
            }
        )));
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::ApplicationResolution {
                target_count: 2,
                ..
            }
        )));
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::InvalidApplicationStructure { .. }
        )));
        assert!(
            kinds
                .iter()
                .any(|kind| matches!(kind, DxfEntityXDataCapacityIssueKind::InvalidValue { .. }))
        );
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::PartialPointTuple { .. }
        )));
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::LayerResolution {
                target_count: 0,
                ..
            }
        )));
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::LayerResolution {
                target_count: 2,
                ..
            }
        )));
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::Text {
                issue: DxfEntityXDataCapacityTextIssue::EscapeMalformed,
                ..
            }
        )));
        assert!(kinds.iter().any(|kind| matches!(
            kind,
            DxfEntityXDataCapacityIssueKind::Text {
                issue: DxfEntityXDataCapacityTextIssue::StorageMalformed,
                ..
            }
        )));
    }
    Ok(())
}

#[test]
fn directory_is_source_bound_cancellable_bounded_and_non_disclosing() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityXDataCapacityDirectory>();
    assert_copy::<DxfEntityXDataCapacityEntry>();
    assert_copy::<DxfEntityXDataCapacityIssue>();
    assert!(size_of::<DxfEntityXDataCapacityEntry>() <= 48);
    assert!(size_of::<DxfEntityXDataCapacityIssue>() <= 16);

    let bytes = exact_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_xdata_capacity_directory(&token())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.typed_directory().source_id(),
        document.source_id()
    );
    assert_eq!(directory.entry(u64::MAX), None);
    for entry in directory.entries().iter().copied() {
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let entity = directory.entity_for_entry(entry)?;
        assert_eq!(directory.entry_for_entity(entity)?, Some(entry));
        assert_eq!(
            directory.issues_for_entry(entry)?.len() as u64,
            entry.issue_range().len()
        );
    }
    assert!(!format!("{directory:?}").contains("SECRET_CAPACITY_PAYLOAD"));

    let other_bytes = exact_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?.entity_xdata_capacity_directory(&token())?;
    let foreign_entity = other.entity_for_entry(other.entries()[0])?;
    assert!(matches!(
        directory.entry_for_entity(foreign_entity),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.entity_xdata_capacity_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

#[derive(Clone)]
enum Value {
    Text(Vec<u8>),
    Chunk(Vec<u8>),
    Double(f64),
    Int16(i16),
    Int32(i32),
}

fn text(code: i16, value: &[u8]) -> (i16, Value) {
    (code, Value::Text(value.to_vec()))
}

fn common_prefix(version: DxfAcadVersion) -> Vec<(i16, Value)> {
    let mut groups = vec![
        text(0, b"SECTION"),
        text(2, b"HEADER"),
        text(9, b"$ACADVER"),
        text(1, version.code().as_bytes()),
        text(0, b"ENDSEC"),
        text(0, b"SECTION"),
        text(2, b"TABLES"),
        text(0, b"TABLE"),
        text(2, b"APPID"),
    ];
    for name in [b"APP".as_slice(), b"APP2", b"DUPAPP", b"DUPAPP"] {
        groups.extend([text(0, b"APPID"), text(2, name)]);
    }
    groups.extend([
        text(0, b"ENDTAB"),
        text(0, b"TABLE"),
        text(2, b"LAYER"),
        text(0, b"LAYER"),
        text(2, b"L0"),
        text(0, b"LAYER"),
        text(2, b"DUP_LAYER"),
        text(0, b"LAYER"),
        text(2, b"DUP_LAYER"),
        text(0, b"ENDTAB"),
        text(0, b"ENDSEC"),
        text(0, b"SECTION"),
        text(2, b"ENTITIES"),
    ]);
    groups
}

fn exact_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut groups = common_prefix(version);
    groups.extend([
        text(0, b"POINT"),
        text(0, b"POINT"),
        text(1001, b"APP"),
        text(1000, br"A\U+0111\U+D83D\U+DE00"),
        text(1002, b"{"),
        text(1002, b"}"),
        text(1003, b"L0"),
        (1004, Value::Chunk(vec![0xAB, 0xCD, 0x01])),
        text(1005, b"AB"),
    ]);
    for suffix in 0..=3 {
        groups.extend([
            (1010 + suffix, Value::Double(1.25)),
            (1020 + suffix, Value::Double(2.5)),
            (1030 + suffix, Value::Double(3.75)),
        ]);
    }
    groups.extend([
        (1040, Value::Double(1.0)),
        (1041, Value::Double(2.0)),
        (1042, Value::Double(3.0)),
        (1070, Value::Int16(-7)),
        (1071, Value::Int32(123_456)),
        text(1001, b"APP2"),
        text(1001, b"APP2"),
        (1070, Value::Int16(1)),
        text(0, b"ENDSEC"),
        text(0, b"EOF"),
    ]);
    encode(format, version, &groups)
}

fn boundary_and_invalid_fixture(format: DxfRawDocumentFormat) -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1032;
    let mut groups = common_prefix(version);
    groups.extend([text(0, b"POINT"), text(1001, b"APP")]);
    for _ in 0..31 {
        groups.push(text(1000, &[b'A'; 255]));
    }
    groups.push(text(1000, &[b'B'; 237]));

    groups.extend([text(0, b"POINT"), text(1001, b"APP")]);
    for _ in 0..31 {
        groups.push(text(1000, &[b'C'; 255]));
    }
    groups.extend([text(1000, &[b'D'; 236]), (1070, Value::Int16(1))]);

    groups.extend([
        text(0, b"POINT"),
        text(1001, b"APP"),
        text(1000, "Ađ😀".as_bytes()),
        text(0, b"POINT"),
        (1040, Value::Double(1.0)),
        text(0, b"POINT"),
        text(1001, b"MISSING"),
        (1070, Value::Int16(1)),
        text(0, b"POINT"),
        text(1001, b"DUPAPP"),
        (1070, Value::Int16(1)),
        text(0, b"POINT"),
        text(1001, b"APP"),
        text(1002, b"x"),
        text(0, b"POINT"),
        text(1001, b"APP"),
        text(1000, &[b'X'; 256]),
        text(0, b"POINT"),
        text(1001, b"APP"),
        (1010, Value::Double(1.0)),
        (1020, Value::Double(2.0)),
        text(0, b"POINT"),
        text(1001, b"APP"),
        text(1003, b"MISSING_LAYER"),
        text(1003, b"DUP_LAYER"),
        text(1000, br"\U+ZZZZ"),
        text(1000, b"\xC3("),
        text(0, b"ENDSEC"),
        text(0, b"EOF"),
    ]);
    encode(format, version, &groups)
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value)],
) -> io::Result<Vec<u8>> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Chunk(value) => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                for byte in value {
                    bytes.push(HEX[usize::from(byte >> 4)]);
                    bytes.push(HEX[usize::from(byte & 0x0F)]);
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

fn binary_groups(version: DxfAcadVersion, groups: &[(i16, Value)]) -> io::Result<Vec<u8>> {
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

fn entity_entries(
    directory: &DxfEntityXDataCapacityDirectory,
) -> Result<Vec<DxfEntityXDataCapacityEntry>, DxfError> {
    directory
        .entries()
        .iter()
        .copied()
        .filter_map(|entry| match directory.entity_for_entry(entry) {
            Ok(entity) if entity.record().section_kind() == DxfRawRecordSectionKind::Entities => {
                Some(Ok(entry))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

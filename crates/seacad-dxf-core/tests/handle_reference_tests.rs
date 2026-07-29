use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandleGroupClass, DxfHandleParseIssue,
    DxfHandleReferenceDirectory, DxfHandleReferenceEntry, DxfMemorySource, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_reference_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.handle_reference_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory, version)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.handle_reference_directory(&DxfCancellationToken::default())?;
        assert_directory(
            DxfRawDocumentView::from(&binary),
            &binary_directory,
            version,
        )?;
    }
    Ok(())
}

#[test]
fn partial_records_arbitrary_identity_and_invalid_spelling_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n330\n1\n0\nSECTION\n2\nOBJECTS\n0\nXRECORD\n5\nA\n320\nB\n340\n0x1\n350\n\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = document.handle_reference_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.record_count(), 1);
    assert_eq!(directory.entries().len(), 2);
    assert_eq!(
        directory.entries()[0].class(),
        DxfHandleGroupClass::HardPointer
    );
    assert_eq!(
        directory.entries()[0].value().parse_result(),
        Err(DxfHandleParseIssue::InvalidDigit { offset: 1 })
    );
    assert_eq!(
        directory.entries()[1].class(),
        DxfHandleGroupClass::SoftOwner
    );
    assert_eq!(
        directory.entries()[1].value().parse_result(),
        Err(DxfHandleParseIssue::Empty)
    );

    let arbitrary = occurrence_of_code(view, 320)?;
    let identity = occurrence_of_code(view, 5)?;
    let interrupted = occurrence_of_code(view, 330)?;
    assert_eq!(directory.reference_for_group(arbitrary), None);
    assert_eq!(directory.reference_for_group(identity), None);
    assert_eq!(directory.reference_for_group(interrupted), None);
    assert_eq!(
        directory.references_for_record(0),
        Some(directory.entries())
    );
    assert_eq!(directory.references_for_record(1), None);
    Ok(())
}

#[test]
fn directory_is_cancellable_source_anchored_and_publicly_bounded() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfHandleReferenceDirectory>();
    assert_copy::<DxfHandleReferenceEntry>();
    assert_send_sync::<DxfHandleReferenceEntry>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.handle_reference_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        source.reads() - reads_after_open,
        directory.entries().len() as u64
    );
    assert!(std::mem::size_of::<DxfHandleReferenceEntry>() <= 128);
    assert!(!format!("{directory:?}").contains("0008"));
    assert_eq!(directory.entry(u64::MAX), None);

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.handle_reference_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfHandleReferenceDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(
        directory.record_count(),
        if version == DxfAcadVersion::Ac1009 {
            5
        } else {
            8
        }
    );
    let expected: &[(i16, DxfHandleGroupClass, u64)] = if version == DxfAcadVersion::Ac1009 {
        &[(1005, DxfHandleGroupClass::SoftPointer, 8)]
    } else {
        &[
            (330, DxfHandleGroupClass::SoftPointer, 1),
            (340, DxfHandleGroupClass::HardPointer, 2),
            (350, DxfHandleGroupClass::SoftOwner, 3),
            (360, DxfHandleGroupClass::HardOwner, 4),
            (390, DxfHandleGroupClass::HardPointer, 5),
            (480, DxfHandleGroupClass::HardPointer, 6),
            (330, DxfHandleGroupClass::SoftPointer, 7),
            (1005, DxfHandleGroupClass::SoftPointer, 8),
        ]
    };
    assert_eq!(directory.entries().len(), expected.len());
    for (ordinal, (entry, (code, class, parsed))) in
        directory.entries().iter().zip(expected).enumerate()
    {
        assert_eq!(directory.entry(u64::try_from(ordinal)?), Some(*entry));
        assert_eq!(entry.value().group().group_code().value(), *code);
        assert_eq!(entry.class(), *class);
        assert_eq!(
            entry.value().parse_result().map(|value| value.value()),
            Ok(*parsed)
        );
        assert_eq!(
            directory.reference_for_group(entry.value().group().occurrence()),
            Some(*entry)
        );
        assert!(
            entry.value().group().occurrence() >= entry.record().group_range().start()
                && entry.value().group().occurrence() < entry.record().group_range().end()
        );
    }

    let class_record = directory
        .references_for_record(0)
        .ok_or_else(invalid_test_data)?;
    assert!(class_record.is_empty());
    assert_eq!(
        directory.references_for_record(directory.record_count()),
        None
    );
    let identity = occurrence_of_code(view, 5)?;
    assert_eq!(directory.reference_for_group(identity), None);
    if version != DxfAcadVersion::Ac1009 {
        let arbitrary = occurrence_of_code(view, 320)?;
        assert_eq!(directory.reference_for_group(arbitrary), None);
    }
    let last = directory
        .entries()
        .last()
        .copied()
        .ok_or_else(invalid_test_data)?;
    let mut spelling = [0_u8; 4];
    last.value().read_raw_spelling(view, &mut spelling)?;
    assert_eq!(&spelling, b"0008");
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n",
        version.code()
    );
    push_ascii_section(&mut text, "CLASSES", &[(0, "CLASS"), (1, "C")]);
    if version == DxfAcadVersion::Ac1009 {
        push_ascii_section(&mut text, "TABLES", &[(0, "TABLE"), (0, "ENDTAB")]);
        push_ascii_section(&mut text, "BLOCKS", &[(0, "BLOCK")]);
        push_ascii_section(
            &mut text,
            "OBJECTS",
            &[(0, "XRECORD"), (5, "A"), (1005, "0008")],
        );
    } else {
        push_ascii_section(
            &mut text,
            "TABLES",
            &[
                (0, "TABLE"),
                (330, "1"),
                (0, "LAYER"),
                (340, "2"),
                (0, "ENDTAB"),
            ],
        );
        push_ascii_section(&mut text, "BLOCKS", &[(0, "BLOCK"), (350, "3")]);
        push_ascii_section(
            &mut text,
            "ENTITIES",
            &[
                (0, "LINE"),
                (360, "4"),
                (0, "CIRCLE"),
                (390, "5"),
                (480, "6"),
            ],
        );
        push_ascii_section(
            &mut text,
            "OBJECTS",
            &[
                (0, "XRECORD"),
                (5, "A"),
                (320, "B"),
                (330, "7"),
                (1005, "0008"),
            ],
        );
    }
    text.push_str("0\nEOF\n");
    text.into_bytes()
}

fn push_ascii_section(text: &mut String, name: &str, groups: &[(i16, &str)]) {
    text.push_str(&format!("0\nSECTION\n2\n{name}\n"));
    for (code, value) in groups {
        text.push_str(&format!("{code}\n{value}\n"));
    }
    text.push_str("0\nENDSEC\n");
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary_section(
        &mut bytes,
        version,
        "HEADER",
        &[(9, "$ACADVER"), (1, version.code())],
    )?;
    push_binary_section(&mut bytes, version, "CLASSES", &[(0, "CLASS"), (1, "C")])?;
    if version == DxfAcadVersion::Ac1009 {
        push_binary_section(
            &mut bytes,
            version,
            "TABLES",
            &[(0, "TABLE"), (0, "ENDTAB")],
        )?;
        push_binary_section(&mut bytes, version, "BLOCKS", &[(0, "BLOCK")])?;
        push_binary_section(
            &mut bytes,
            version,
            "OBJECTS",
            &[(0, "XRECORD"), (5, "A"), (1005, "0008")],
        )?;
    } else {
        push_binary_section(
            &mut bytes,
            version,
            "TABLES",
            &[
                (0, "TABLE"),
                (330, "1"),
                (0, "LAYER"),
                (340, "2"),
                (0, "ENDTAB"),
            ],
        )?;
        push_binary_section(&mut bytes, version, "BLOCKS", &[(0, "BLOCK"), (350, "3")])?;
        push_binary_section(
            &mut bytes,
            version,
            "ENTITIES",
            &[
                (0, "LINE"),
                (360, "4"),
                (0, "CIRCLE"),
                (390, "5"),
                (480, "6"),
            ],
        )?;
        push_binary_section(
            &mut bytes,
            version,
            "OBJECTS",
            &[
                (0, "XRECORD"),
                (5, "A"),
                (320, "B"),
                (330, "7"),
                (1005, "0008"),
            ],
        )?;
    }
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_section(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &str,
    groups: &[(i16, &str)],
) -> Result<(), io::Error> {
    for (code, value) in [(0, "SECTION"), (2, name)] {
        push_binary_string(bytes, version, code, value.as_bytes())?;
    }
    for (code, value) in groups {
        push_binary_string(bytes, version, *code, value.as_bytes())?;
    }
    push_binary_string(bytes, version, 0, b"ENDSEC")
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        if (0..=254).contains(&group_code) {
            bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
        } else if (1000..=1071).contains(&group_code) {
            bytes.push(u8::MAX);
            bytes.extend_from_slice(&group_code.to_le_bytes());
        } else {
            return Err(io::Error::other("group code unavailable before R13"));
        }
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn occurrence_of_code(view: DxfRawDocumentView<'_>, code: i16) -> Result<u64, DxfError> {
    for occurrence in 0..view.group_count() {
        if view
            .group(occurrence)
            .is_some_and(|group| group.group_code().value() == code)
        {
            return Ok(occurrence);
        }
    }
    Err(invalid_test_data())
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

struct CountingSource<'a> {
    bytes: &'a [u8],
    reads: AtomicU64,
}

impl<'a> CountingSource<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            reads: AtomicU64::new(0),
        }
    }

    fn reads(&self) -> u64 {
        self.reads.load(Ordering::Relaxed)
    }
}

impl DxfByteSource for CountingSource<'_> {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = self.bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

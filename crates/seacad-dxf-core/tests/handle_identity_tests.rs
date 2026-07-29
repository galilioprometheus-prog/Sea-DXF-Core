use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandle, DxfHandleIdentityCandidateRange,
    DxfHandleIdentityDirectory, DxfHandleIdentityEntry, DxfHandleIdentityLookup,
    DxfHandleIdentityMatch, DxfHandleIdentityState, DxfHandleParseIssue, DxfMemorySource,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_identity_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.handle_identity_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.handle_identity_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&binary), &binary_directory)?;
    }
    Ok(())
}

#[test]
fn malformed_multiple_and_duplicate_identities_remain_distinct() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nNOHANDLE\n1\nA\n0\nBAD\n5\n0x1\n0\nMULTI\n5\n1\n105\n2\n0\nFIRST\n5\nA\n0\nSECOND\n5\na\n0\nNULL\n5\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_identity_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.entries().len(), 6);
    assert_eq!(directory.candidates().len(), 6);
    assert_eq!(directory.matches().len(), 3);
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.candidates_for_record(u64::MAX), None);
    assert_eq!(
        directory.entries()[0].state(),
        DxfHandleIdentityState::Absent
    );
    assert_eq!(
        directory.entries()[1].state(),
        DxfHandleIdentityState::UniqueInvalid(DxfHandleParseIssue::InvalidDigit { offset: 1 })
    );
    assert_eq!(
        directory.entries()[2].state(),
        DxfHandleIdentityState::Multiple { candidate_count: 2 }
    );
    assert_eq!(
        directory.entries()[3].state(),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(0xA))
    );
    assert_eq!(
        directory.entries()[4].state(),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(0xA))
    );
    assert_eq!(
        directory.entries()[5].state(),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(0))
    );

    let multiple = directory
        .candidates_for_record(2)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].group().group_code().value(), 5);
    assert_eq!(multiple[1].group().group_code().value(), 105);
    assert_eq!(
        directory.lookup(DxfHandle::from_u64(1)),
        DxfHandleIdentityLookup::Missing
    );
    match directory.lookup(DxfHandle::from_u64(0xA)) {
        DxfHandleIdentityLookup::Ambiguous(matches) => {
            assert_eq!(matches.len(), 2);
            assert_eq!(matches[0].record().ordinal(), 3);
            assert_eq!(matches[1].record().ordinal(), 4);
        }
        other => return Err(io::Error::other(format!("unexpected lookup: {other:?}")).into()),
    }
    match directory.lookup(DxfHandle::from_u64(0)) {
        DxfHandleIdentityLookup::Unique(identity) => {
            assert_eq!(identity.record().ordinal(), 5);
            assert!(identity.handle().is_null());
        }
        other => return Err(io::Error::other(format!("unexpected lookup: {other:?}")).into()),
    }
    Ok(())
}

#[test]
fn directory_is_cancellable_source_anchored_and_publicly_bounded() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfHandleIdentityDirectory>();
    assert_copy::<DxfHandleIdentityCandidateRange>();
    assert_copy::<DxfHandleIdentityEntry>();
    assert_copy::<DxfHandleIdentityMatch>();
    assert_send_sync::<DxfHandleIdentityEntry>();
    assert_send_sync::<DxfHandleIdentityMatch>();

    let bytes = ascii_fixture("AC1032");
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.handle_identity_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        source.reads() - reads_after_open,
        directory.candidates().len() as u64
    );
    assert!(std::mem::size_of::<DxfHandleIdentityEntry>() <= 64);
    assert!(std::mem::size_of::<DxfHandleIdentityMatch>() <= 128);
    assert!(!format!("{directory:?}").contains("000a"));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.handle_identity_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfHandleIdentityDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.entries().len(), 12);
    assert_eq!(directory.candidates().len(), 10);
    assert_eq!(directory.matches().len(), 10);

    let expected = [
        DxfHandleIdentityState::Absent,
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(1)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(2)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(3)),
        DxfHandleIdentityState::Absent,
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(4)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(5)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(6)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(7)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(8)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(9)),
        DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(0xA)),
    ];
    for (ordinal, (entry, state)) in directory.entries().iter().zip(expected).enumerate() {
        assert_eq!(entry.record().ordinal(), u64::try_from(ordinal)?);
        assert_eq!(entry.state(), state);
        assert_eq!(directory.entry(entry.record().ordinal()), Some(*entry));
        let candidate_count = directory
            .candidates_for_record(entry.record().ordinal())
            .ok_or_else(invalid_test_data)?
            .len();
        assert_eq!(candidate_count as u64, entry.candidate_range().len());
    }

    let dimstyle = directory
        .candidates_for_record(3)
        .and_then(|candidates| candidates.first())
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert_eq!(dimstyle.group().group_code().value(), 105);
    let last = directory
        .candidates_for_record(11)
        .and_then(|candidates| candidates.first())
        .copied()
        .ok_or_else(invalid_test_data)?;
    let mut spelling = [0_u8; 4];
    last.read_raw_spelling(view, &mut spelling)?;
    assert_eq!(&spelling, b"000a");

    match directory.lookup(DxfHandle::from_u64(0xA)) {
        DxfHandleIdentityLookup::Unique(identity) => {
            assert_eq!(identity.record().ordinal(), 11);
            assert_eq!(identity.candidate(), last);
        }
        other => return Err(io::Error::other(format!("unexpected lookup: {other:?}")).into()),
    }
    assert!(
        directory
            .matches_for_handle(DxfHandle::from_u64(0xB))
            .is_empty()
    );
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    let mut text = format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n");
    push_ascii_section(&mut text, "CLASSES", &[(0, "CLASS"), (1, "C")]);
    push_ascii_section(
        &mut text,
        "TABLES",
        &[
            (0, "TABLE"),
            (5, "1"),
            (0, "LAYER"),
            (5, "2"),
            (0, "DIMSTYLE"),
            (105, "3"),
            (0, "ENDTAB"),
        ],
    );
    push_ascii_section(
        &mut text,
        "BLOCKS",
        &[
            (0, "BLOCK"),
            (5, "4"),
            (0, "LINE"),
            (5, "5"),
            (0, "ENDBLK"),
            (5, "6"),
        ],
    );
    push_ascii_section(
        &mut text,
        "ENTITIES",
        &[(0, "LINE"), (5, "7"), (0, "CIRCLE"), (5, "8")],
    );
    push_ascii_section(
        &mut text,
        "OBJECTS",
        &[(0, "DICTIONARY"), (5, "9"), (0, "XRECORD"), (5, "000a")],
    );
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
    push_binary_section(
        &mut bytes,
        version,
        "TABLES",
        &[
            (0, "TABLE"),
            (5, "1"),
            (0, "LAYER"),
            (5, "2"),
            (0, "DIMSTYLE"),
            (105, "3"),
            (0, "ENDTAB"),
        ],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "BLOCKS",
        &[
            (0, "BLOCK"),
            (5, "4"),
            (0, "LINE"),
            (5, "5"),
            (0, "ENDBLK"),
            (5, "6"),
        ],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "ENTITIES",
        &[(0, "LINE"), (5, "7"), (0, "CIRCLE"), (5, "8")],
    )?;
    push_binary_section(
        &mut bytes,
        version,
        "OBJECTS",
        &[(0, "DICTIONARY"), (5, "9"), (0, "XRECORD"), (5, "000a")],
    )?;
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
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&group_code.to_le_bytes());
    }
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
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

use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandleIdentityState, DxfHandleParseIssue,
    DxfHandleResolutionDirectory, DxfHandleResolutionEntry, DxfHandleResolutionState,
    DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_resolution_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.handle_resolution_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory, version)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.handle_resolution_directory(&DxfCancellationToken::default())?;
        assert_directory(
            DxfRawDocumentView::from(&binary),
            &binary_directory,
            version,
        )?;
    }
    Ok(())
}

#[test]
fn anomalous_identities_and_null_targets_never_resolve_by_guessing() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nMULTI\n5\nE\n105\nF\n0\nBAD\n5\n0x1\n0\nZERO\n5\n0\n0\nSOURCE\n5\nC\n330\nE\n340\nF\n350\n1\n360\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_resolution_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.entries().len(), 4);
    assert_eq!(
        directory.identity_directory().entries()[0].state(),
        DxfHandleIdentityState::Multiple { candidate_count: 2 }
    );
    assert!(matches!(
        directory.identity_directory().entries()[1].state(),
        DxfHandleIdentityState::UniqueInvalid(_)
    ));
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfHandleResolutionState::Missing,
            DxfHandleResolutionState::Missing,
            DxfHandleResolutionState::Missing,
            DxfHandleResolutionState::Null,
        ]
    );
    for ordinal in 0..directory.entries().len() {
        assert!(
            directory
                .targets_for_reference(u64::try_from(ordinal)?)
                .ok_or_else(invalid_test_data)?
                .is_empty()
        );
    }
    Ok(())
}

#[test]
fn directory_is_cancellable_linear_and_publicly_bounded() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfHandleResolutionDirectory>();
    assert_copy::<DxfHandleResolutionEntry>();
    assert_copy::<DxfHandleResolutionState>();
    assert_send_sync::<DxfHandleResolutionEntry>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.handle_resolution_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads() - reads_after_open, 11);
    assert_eq!(
        directory.entries().len(),
        directory.reference_directory().entries().len()
    );
    assert!(std::mem::size_of::<DxfHandleResolutionEntry>() <= 160);
    assert!(!format!("{directory:?}").contains("000B"));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.targets_for_reference(u64::MAX), None);

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.handle_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfHandleResolutionDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.identity_directory().source_id(), view.source_id());
    assert_eq!(
        directory.reference_directory().source_id(),
        view.source_id()
    );
    let expected: &[DxfHandleResolutionState] = if version == DxfAcadVersion::Ac1009 {
        &[DxfHandleResolutionState::Unique]
    } else {
        &[
            DxfHandleResolutionState::Unique,
            DxfHandleResolutionState::Missing,
            DxfHandleResolutionState::Null,
            DxfHandleResolutionState::Invalid(DxfHandleParseIssue::InvalidDigit { offset: 1 }),
            DxfHandleResolutionState::Ambiguous { target_count: 2 },
            DxfHandleResolutionState::Unique,
        ]
    };
    assert_eq!(directory.entries().len(), expected.len());
    for (ordinal, (entry, state)) in directory.entries().iter().zip(expected).enumerate() {
        let ordinal = u64::try_from(ordinal)?;
        assert_eq!(directory.entry(ordinal), Some(*entry));
        assert_eq!(entry.state(), *state);
        let occurrence = entry.reference().value().group().occurrence();
        assert_eq!(directory.resolution_for_group(occurrence), Some(*entry));
        let targets = directory
            .targets_for_reference(ordinal)
            .ok_or_else(invalid_test_data)?;
        let expected_count = match state {
            DxfHandleResolutionState::Unique => 1,
            DxfHandleResolutionState::Ambiguous { target_count } => *target_count as usize,
            DxfHandleResolutionState::Invalid(_)
            | DxfHandleResolutionState::Null
            | DxfHandleResolutionState::Missing => 0,
            _ => return Err(invalid_test_data().into()),
        };
        assert_eq!(targets.len(), expected_count);
    }
    let last = directory
        .entries()
        .last()
        .copied()
        .ok_or_else(invalid_test_data)?;
    let mut spelling = [0_u8; 4];
    last.reference()
        .value()
        .read_raw_spelling(view, &mut spelling)?;
    assert_eq!(&spelling, b"000B");
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n",
        version.code()
    );
    for (kind, handle) in [
        ("TARGETA", "A"),
        ("TARGETB", "B"),
        ("TARGETD1", "D"),
        ("TARGETD2", "d"),
    ] {
        text.push_str(&format!("0\n{kind}\n5\n{handle}\n"));
    }
    text.push_str("0\nSOURCE\n5\nC\n");
    if version != DxfAcadVersion::Ac1009 {
        text.push_str("330\nA\n340\nF\n350\n0\n360\n0x1\n390\nD\n");
    }
    text.push_str("1005\n000B\n0\nENDSEC\n0\nEOF\n");
    text.into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version.code()),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "OBJECTS"),
    ] {
        push_binary_string(&mut bytes, version, code, value.as_bytes())?;
    }
    for (kind, handle) in [
        ("TARGETA", "A"),
        ("TARGETB", "B"),
        ("TARGETD1", "D"),
        ("TARGETD2", "d"),
    ] {
        push_binary_string(&mut bytes, version, 0, kind.as_bytes())?;
        push_binary_string(&mut bytes, version, 5, handle.as_bytes())?;
    }
    push_binary_string(&mut bytes, version, 0, b"SOURCE")?;
    push_binary_string(&mut bytes, version, 5, b"C")?;
    if version != DxfAcadVersion::Ac1009 {
        for (code, value) in [(330, "A"), (340, "F"), (350, "0"), (360, "0x1"), (390, "D")] {
            push_binary_string(&mut bytes, version, code, value.as_bytes())?;
        }
    }
    push_binary_string(&mut bytes, version, 1005, b"000B")?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
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

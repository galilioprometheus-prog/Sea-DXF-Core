use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfCommonOwnerCandidateDirectory, DxfCommonOwnerCandidateEntry,
    DxfCommonOwnerCandidateRange, DxfCommonOwnerCandidateState, DxfCommonOwnerRecordEntry,
    DxfError, DxfHandleParseIssue, DxfHandleResolutionState, DxfHandleRoleEvidence,
    DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_common_owner_candidate_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.common_owner_candidate_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory, version)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.common_owner_candidate_directory(&DxfCancellationToken::default())?;
        assert_directory(
            DxfRawDocumentView::from(&binary),
            &binary_directory,
            version,
        )?;
    }
    Ok(())
}

#[test]
fn every_resolution_state_and_target_match_remains_independent() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.common_owner_candidate_directory(&DxfCancellationToken::default())?;

    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfHandleResolutionState::Unique,
            DxfHandleResolutionState::Missing,
            DxfHandleResolutionState::Null,
            DxfHandleResolutionState::Ambiguous { target_count: 2 },
            DxfHandleResolutionState::Invalid(DxfHandleParseIssue::InvalidDigit { offset: 1 }),
        ]
    );
    let expected_target_ordinals: &[&[u64]] = &[&[0], &[], &[], &[1, 2], &[]];
    for (candidate_ordinal, expected) in expected_target_ordinals.iter().enumerate() {
        assert_eq!(
            directory
                .targets_for_candidate(u64::try_from(candidate_ordinal)?)
                .ok_or_else(invalid_test_data)?
                .iter()
                .map(|target| target.record().ordinal())
                .collect::<Vec<_>>(),
            *expected
        );
    }
    Ok(())
}

#[test]
fn every_raw_record_has_typed_common_owner_candidate_cardinality() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.common_owner_candidate_directory(&DxfCancellationToken::default())?;

    let expected = [
        DxfCommonOwnerCandidateState::NoCandidate,
        DxfCommonOwnerCandidateState::NoCandidate,
        DxfCommonOwnerCandidateState::NoCandidate,
        DxfCommonOwnerCandidateState::NoCandidate,
        DxfCommonOwnerCandidateState::UniqueCandidate,
        DxfCommonOwnerCandidateState::MultipleCandidates { candidate_count: 2 },
        DxfCommonOwnerCandidateState::UniqueCandidate,
        DxfCommonOwnerCandidateState::NoCandidate,
        DxfCommonOwnerCandidateState::UniqueCandidate,
    ];
    assert_eq!(directory.record_entries().len(), expected.len());
    for (record_ordinal, (entry, state)) in
        directory.record_entries().iter().zip(expected).enumerate()
    {
        let record_ordinal = u64::try_from(record_ordinal)?;
        assert_eq!(directory.record_entry(record_ordinal), Some(*entry));
        assert_eq!(entry.record().ordinal(), record_ordinal);
        assert_eq!(entry.state(), state);
        assert_eq!(
            entry.candidate_range().len(),
            directory
                .candidates_for_record(record_ordinal)
                .ok_or_else(invalid_test_data)?
                .len() as u64
        );
    }
    assert_eq!(
        directory
            .candidates_for_record(5)
            .ok_or_else(invalid_test_data)?,
        &directory.entries()[1..3]
    );
    assert_eq!(directory.record_entry(u64::MAX), None);
    assert_eq!(directory.candidates_for_record(u64::MAX), None);
    Ok(())
}

#[test]
fn directory_is_cancellable_linear_filtered_and_publicly_bounded() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfCommonOwnerCandidateDirectory>();
    assert_copy::<DxfCommonOwnerCandidateRange>();
    assert_copy::<DxfCommonOwnerCandidateEntry>();
    assert_copy::<DxfCommonOwnerCandidateState>();
    assert_copy::<DxfCommonOwnerRecordEntry>();
    assert_send_sync::<DxfCommonOwnerCandidateEntry>();
    assert_send_sync::<DxfCommonOwnerRecordEntry>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.common_owner_candidate_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads() - reads_after_open, 17);
    assert_eq!(directory.entries().len(), 5);
    assert_eq!(directory.record_entries().len(), 9);
    assert!(directory.entries().len() <= directory.role_directory().entries().len());
    assert!(std::mem::size_of::<DxfCommonOwnerCandidateEntry>() <= 200);
    assert!(std::mem::size_of::<DxfCommonOwnerRecordEntry>() <= 80);
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.targets_for_candidate(u64::MAX), None);
    assert!(!format!("{directory:?}").contains("0x1"));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.common_owner_candidate_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfCommonOwnerCandidateDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.role_directory().source_id(), view.source_id());
    assert_eq!(directory.record_entries().len(), 9);
    if version == DxfAcadVersion::Ac1009 {
        assert!(directory.entries().is_empty());
        assert!(directory.record_entries().iter().all(|entry| {
            entry.state() == DxfCommonOwnerCandidateState::NoCandidate
                && entry.candidate_range().is_empty()
        }));
        return Ok(());
    }

    assert_eq!(directory.entries().len(), 5);
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.role_ordinal())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 5]
    );
    for (candidate_ordinal, entry) in directory.entries().iter().enumerate() {
        let candidate_ordinal = u64::try_from(candidate_ordinal)?;
        assert_eq!(directory.entry(candidate_ordinal), Some(*entry));
        assert_eq!(
            entry.role().role(),
            DxfHandleRoleEvidence::CommonOwnerPointerCandidate
        );
    }
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n",
        version.code()
    );
    for (kind, handle) in [
        ("TARGETA", "A"),
        ("TARGETB1", "B"),
        ("TARGETB2", "b"),
        ("CHILD0", "10"),
    ] {
        text.push_str(&format!("0\n{kind}\n5\n{handle}\n"));
    }
    if version == DxfAcadVersion::Ac1009 {
        for (kind, handle) in [
            ("CHILD1", "11"),
            ("CHILD2", "12"),
            ("CHILD3", "13"),
            ("CHILD4", "14"),
            ("CHILD5", "15"),
        ] {
            text.push_str(&format!("0\n{kind}\n5\n{handle}\n"));
        }
    } else {
        text.push_str("0\nCHILD1\n5\n11\n330\nA\n");
        text.push_str("0\nCHILD2\n5\n12\n330\nF\n330\n0\n");
        text.push_str("0\nCHILD3\n5\n13\n330\nB\n");
        text.push_str("0\nCHILD4\n5\n14\n102\n{ACAD_REACTORS\n330\nA\n102\n}\n");
        text.push_str("0\nCHILD5\n5\n15\n330\n0x1\n");
    }
    text.push_str("0\nENDSEC\n0\nEOF\n");
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
        ("TARGETB1", "B"),
        ("TARGETB2", "b"),
        ("CHILD0", "10"),
    ] {
        push_binary_string(&mut bytes, version, 0, kind.as_bytes())?;
        push_binary_string(&mut bytes, version, 5, handle.as_bytes())?;
    }
    if version == DxfAcadVersion::Ac1009 {
        for (kind, handle) in [
            ("CHILD1", "11"),
            ("CHILD2", "12"),
            ("CHILD3", "13"),
            ("CHILD4", "14"),
            ("CHILD5", "15"),
        ] {
            push_binary_string(&mut bytes, version, 0, kind.as_bytes())?;
            push_binary_string(&mut bytes, version, 5, handle.as_bytes())?;
        }
    } else {
        for (code, value) in [
            (0, "CHILD1"),
            (5, "11"),
            (330, "A"),
            (0, "CHILD2"),
            (5, "12"),
            (330, "F"),
            (330, "0"),
            (0, "CHILD3"),
            (5, "13"),
            (330, "B"),
            (0, "CHILD4"),
            (5, "14"),
            (102, "{ACAD_REACTORS"),
            (330, "A"),
            (102, "}"),
            (0, "CHILD5"),
            (5, "15"),
            (330, "0x1"),
        ] {
            push_binary_string(&mut bytes, version, code, value.as_bytes())?;
        }
    }
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

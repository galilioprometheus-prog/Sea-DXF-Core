use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfCommonOwnerCandidateState, DxfError, DxfHandleResolutionState,
    DxfIncomingOwnershipState, DxfMemorySource, DxfOwnerEvidenceComparisonDirectory,
    DxfOwnerEvidenceComparisonEntry, DxfOwnerEvidenceComparisonState, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_owner_comparison_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.owner_evidence_comparison_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory, version)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.owner_evidence_comparison_directory(&DxfCancellationToken::default())?;
        assert_directory(
            DxfRawDocumentView::from(&binary),
            &binary_directory,
            version,
        )?;
    }
    Ok(())
}

#[test]
fn unique_bidirectional_evidence_is_matched_or_conflicting_by_record_ordinal()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.owner_evidence_comparison_directory(&DxfCancellationToken::default())?;

    let matched = directory.entry(3).ok_or_else(invalid_test_data)?;
    assert_eq!(matched.state(), DxfOwnerEvidenceComparisonState::Matched);
    assert_eq!(matched.candidate_ordinal(), Some(0));
    assert_eq!(matched.incoming_link_ordinal(), Some(0));
    assert_eq!(
        directory
            .candidate_target_for_entry(3)
            .ok_or_else(invalid_test_data)?
            .record()
            .ordinal(),
        0
    );
    assert_eq!(
        directory
            .incoming_source_for_entry(3)
            .ok_or_else(invalid_test_data)?
            .resolution()
            .reference()
            .record()
            .ordinal(),
        0
    );

    let conflicting = directory.entry(4).ok_or_else(invalid_test_data)?;
    assert_eq!(
        conflicting.state(),
        DxfOwnerEvidenceComparisonState::Conflicting
    );
    assert_eq!(conflicting.candidate_ordinal(), Some(1));
    assert_eq!(conflicting.incoming_link_ordinal(), Some(1));
    assert_eq!(
        directory
            .candidate_target_for_entry(4)
            .ok_or_else(invalid_test_data)?
            .record()
            .ordinal(),
        1
    );
    assert_eq!(
        directory
            .incoming_source_for_entry(4)
            .ok_or_else(invalid_test_data)?
            .resolution()
            .reference()
            .record()
            .ordinal(),
        2
    );
    Ok(())
}

#[test]
fn absent_multiple_and_unresolved_dimensions_are_not_comparable() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.owner_evidence_comparison_directory(&DxfCancellationToken::default())?;

    for record_ordinal in [0, 1, 2, 5, 6, 7, 8, 9] {
        let entry = directory
            .entry(record_ordinal)
            .ok_or_else(invalid_test_data)?;
        assert_eq!(
            entry.state(),
            DxfOwnerEvidenceComparisonState::NotComparable
        );
        assert_eq!(entry.candidate_ordinal(), None);
        assert_eq!(entry.incoming_link_ordinal(), None);
        assert_eq!(directory.candidate_for_entry(record_ordinal), None);
        assert_eq!(directory.incoming_link_for_entry(record_ordinal), None);
    }

    let candidate_only = directory.entry(5).ok_or_else(invalid_test_data)?;
    assert_eq!(
        candidate_only.common_owner().state(),
        DxfCommonOwnerCandidateState::UniqueCandidate
    );
    assert_eq!(
        candidate_only.incoming_ownership().state(),
        DxfIncomingOwnershipState::NoIncomingLink
    );
    let incoming_only = directory.entry(6).ok_or_else(invalid_test_data)?;
    assert_eq!(
        incoming_only.common_owner().state(),
        DxfCommonOwnerCandidateState::NoCandidate
    );
    assert_eq!(
        incoming_only.incoming_ownership().state(),
        DxfIncomingOwnershipState::UniqueIncomingLink
    );
    let unresolved = directory
        .common_owner_candidate_directory()
        .candidates_for_record(7)
        .ok_or_else(invalid_test_data)?;
    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0].state(), DxfHandleResolutionState::Missing);
    assert_eq!(
        directory
            .entry(8)
            .ok_or_else(invalid_test_data)?
            .common_owner()
            .state(),
        DxfCommonOwnerCandidateState::MultipleCandidates { candidate_count: 2 }
    );
    assert_eq!(
        directory
            .entry(9)
            .ok_or_else(invalid_test_data)?
            .incoming_ownership()
            .state(),
        DxfIncomingOwnershipState::MultipleIncomingLinks { link_count: 2 }
    );
    Ok(())
}

#[test]
fn directory_is_cancellable_bounded_linear_and_non_disclosing() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfOwnerEvidenceComparisonDirectory>();
    assert_copy::<DxfOwnerEvidenceComparisonEntry>();
    assert_copy::<DxfOwnerEvidenceComparisonState>();
    assert_send_sync::<DxfOwnerEvidenceComparisonEntry>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory =
        document.owner_evidence_comparison_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads() - reads_after_open, 48);
    assert_eq!(directory.entries().len(), 10);
    assert_eq!(
        directory
            .common_owner_candidate_directory()
            .record_entries()
            .len(),
        directory.entries().len()
    );
    assert_eq!(
        directory
            .ownership_evidence_directory()
            .target_entries()
            .len(),
        directory.entries().len()
    );
    assert!(std::mem::size_of::<DxfOwnerEvidenceComparisonEntry>() <= 160);
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.candidate_target_for_entry(u64::MAX), None);
    assert_eq!(directory.incoming_source_for_entry(u64::MAX), None);
    assert!(!format!("{directory:?}").contains("FF"));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.owner_evidence_comparison_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfOwnerEvidenceComparisonDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(
        directory.common_owner_candidate_directory().source_id(),
        view.source_id()
    );
    assert_eq!(
        directory.ownership_evidence_directory().source_id(),
        view.source_id()
    );
    assert_eq!(directory.entries().len(), 10);
    for (record_ordinal, entry) in directory.entries().iter().enumerate() {
        let record_ordinal = u64::try_from(record_ordinal)?;
        assert_eq!(directory.entry(record_ordinal), Some(*entry));
        assert_eq!(entry.record().ordinal(), record_ordinal);
        assert_eq!(entry.common_owner().record(), entry.record());
        assert_eq!(entry.incoming_ownership().record(), entry.record());
    }
    let expected = if version == DxfAcadVersion::Ac1009 {
        [DxfOwnerEvidenceComparisonState::NotComparable; 10]
    } else {
        [
            DxfOwnerEvidenceComparisonState::NotComparable,
            DxfOwnerEvidenceComparisonState::NotComparable,
            DxfOwnerEvidenceComparisonState::NotComparable,
            DxfOwnerEvidenceComparisonState::Matched,
            DxfOwnerEvidenceComparisonState::Conflicting,
            DxfOwnerEvidenceComparisonState::NotComparable,
            DxfOwnerEvidenceComparisonState::NotComparable,
            DxfOwnerEvidenceComparisonState::NotComparable,
            DxfOwnerEvidenceComparisonState::NotComparable,
            DxfOwnerEvidenceComparisonState::NotComparable,
        ]
    };
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        expected
    );
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n",
        version.code()
    );
    if version == DxfAcadVersion::Ac1009 {
        for (kind, handle) in record_identities() {
            text.push_str(&format!("0\n{kind}\n5\n{handle}\n"));
        }
    } else {
        text.push_str("0\nOWNER_A\n5\nA\n350\nB\n350\n10\n350\n11\n350\n12\n350\n13\n");
        text.push_str("0\nOWNER_C\n5\nC\n");
        text.push_str("0\nOWNER_E\n5\nE\n350\nD\n350\n13\n");
        text.push_str("0\nCHILD_MATCH\n5\nB\n330\nA\n");
        text.push_str("0\nCHILD_CONFLICT\n5\nD\n330\nC\n");
        text.push_str("0\nCHILD_CANDIDATE_ONLY\n5\nF\n330\nA\n");
        text.push_str("0\nCHILD_INCOMING_ONLY\n5\n10\n");
        text.push_str("0\nCHILD_UNRESOLVED\n5\n11\n330\nFF\n");
        text.push_str("0\nCHILD_MULTIPLE_CANDIDATE\n5\n12\n330\nA\n330\nC\n");
        text.push_str("0\nCHILD_MULTIPLE_INCOMING\n5\n13\n330\nA\n");
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
    if version == DxfAcadVersion::Ac1009 {
        for (kind, handle) in record_identities() {
            push_binary_string(&mut bytes, version, 0, kind.as_bytes())?;
            push_binary_string(&mut bytes, version, 5, handle.as_bytes())?;
        }
    } else {
        for (code, value) in modern_records() {
            push_binary_string(&mut bytes, version, code, value.as_bytes())?;
        }
    }
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn record_identities() -> [(&'static str, &'static str); 10] {
    [
        ("OWNER_A", "A"),
        ("OWNER_C", "C"),
        ("OWNER_E", "E"),
        ("CHILD_MATCH", "B"),
        ("CHILD_CONFLICT", "D"),
        ("CHILD_CANDIDATE_ONLY", "F"),
        ("CHILD_INCOMING_ONLY", "10"),
        ("CHILD_UNRESOLVED", "11"),
        ("CHILD_MULTIPLE_CANDIDATE", "12"),
        ("CHILD_MULTIPLE_INCOMING", "13"),
    ]
}

fn modern_records() -> [(i16, &'static str); 34] {
    [
        (0, "OWNER_A"),
        (5, "A"),
        (350, "B"),
        (350, "10"),
        (350, "11"),
        (350, "12"),
        (350, "13"),
        (0, "OWNER_C"),
        (5, "C"),
        (0, "OWNER_E"),
        (5, "E"),
        (350, "D"),
        (350, "13"),
        (0, "CHILD_MATCH"),
        (5, "B"),
        (330, "A"),
        (0, "CHILD_CONFLICT"),
        (5, "D"),
        (330, "C"),
        (0, "CHILD_CANDIDATE_ONLY"),
        (5, "F"),
        (330, "A"),
        (0, "CHILD_INCOMING_ONLY"),
        (5, "10"),
        (0, "CHILD_UNRESOLVED"),
        (5, "11"),
        (330, "FF"),
        (0, "CHILD_MULTIPLE_CANDIDATE"),
        (5, "12"),
        (330, "A"),
        (330, "C"),
        (0, "CHILD_MULTIPLE_INCOMING"),
        (5, "13"),
        (330, "A"),
    ]
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

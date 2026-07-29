use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandleGroupClass, DxfHandleParseIssue,
    DxfHandleResolutionState, DxfMemorySource, DxfOwnershipEvidenceDirectory,
    DxfOwnershipEvidenceEntry, DxfRawDocumentView, DxfReadOptions, DxfResolvedOwnershipLink,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_ownership_evidence_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.ownership_evidence_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory, version)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.ownership_evidence_directory(&DxfCancellationToken::default())?;
        assert_directory(
            DxfRawDocumentView::from(&binary),
            &binary_directory,
            version,
        )?;
    }
    Ok(())
}

#[test]
fn unresolved_owner_occurrences_do_not_enter_the_incoming_target_index()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.ownership_evidence_directory(&DxfCancellationToken::default())?;

    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfHandleResolutionState::Unique,
            DxfHandleResolutionState::Unique,
            DxfHandleResolutionState::Unique,
            DxfHandleResolutionState::Missing,
            DxfHandleResolutionState::Null,
            DxfHandleResolutionState::Invalid(DxfHandleParseIssue::InvalidDigit { offset: 1 }),
            DxfHandleResolutionState::Ambiguous { target_count: 2 },
        ]
    );
    assert_eq!(directory.resolved_links().len(), 3);
    assert_eq!(
        directory
            .incoming_links_for_target_record(0)
            .ok_or_else(invalid_test_data)?
            .iter()
            .map(|link| link.evidence_ordinal())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        directory
            .incoming_links_for_target_record(1)
            .ok_or_else(invalid_test_data)?
            .iter()
            .map(|link| link.evidence_ordinal())
            .collect::<Vec<_>>(),
        [2]
    );
    for record_ordinal in 2..8 {
        assert!(
            directory
                .incoming_links_for_target_record(record_ordinal)
                .ok_or_else(invalid_test_data)?
                .is_empty()
        );
    }
    Ok(())
}

#[test]
fn directory_is_cancellable_linear_filtered_and_publicly_bounded() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfOwnershipEvidenceDirectory>();
    assert_copy::<DxfOwnershipEvidenceEntry>();
    assert_copy::<DxfResolvedOwnershipLink>();
    assert_send_sync::<DxfOwnershipEvidenceEntry>();
    assert_send_sync::<DxfResolvedOwnershipLink>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.ownership_evidence_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads() - reads_after_open, 17);
    assert_eq!(directory.entries().len(), 7);
    assert_eq!(directory.resolution_directory().entries().len(), 9);
    assert!(directory.resolved_links().len() <= directory.entries().len());
    assert!(std::mem::size_of::<DxfOwnershipEvidenceEntry>() <= 176);
    assert!(std::mem::size_of::<DxfResolvedOwnershipLink>() <= 144);
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.evidence_for_source_record(u64::MAX), None);
    assert_eq!(directory.incoming_links_for_target_record(u64::MAX), None);

    let source_record = directory.entries()[0]
        .resolution()
        .reference()
        .record()
        .ordinal();
    assert_eq!(
        directory
            .evidence_for_source_record(source_record)
            .ok_or_else(invalid_test_data)?,
        &directory.entries()[0..1]
    );
    assert_eq!(
        directory.entries()[0].resolution_ordinal(),
        1,
        "soft-pointer evidence before the owner link must remain excluded"
    );
    assert_eq!(directory.entries()[1].resolution_ordinal(), 3);

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.ownership_evidence_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfOwnershipEvidenceDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(
        directory.resolution_directory().source_id(),
        view.source_id()
    );
    if version == DxfAcadVersion::Ac1009 {
        assert!(directory.entries().is_empty());
        assert!(directory.resolved_links().is_empty());
        return Ok(());
    }

    assert_eq!(directory.entries().len(), 7);
    let expected_classes = [
        DxfHandleGroupClass::SoftOwner,
        DxfHandleGroupClass::HardOwner,
        DxfHandleGroupClass::SoftOwner,
        DxfHandleGroupClass::SoftOwner,
        DxfHandleGroupClass::HardOwner,
        DxfHandleGroupClass::SoftOwner,
        DxfHandleGroupClass::HardOwner,
    ];
    for (ordinal, (entry, class)) in directory.entries().iter().zip(expected_classes).enumerate() {
        assert_eq!(directory.entry(u64::try_from(ordinal)?), Some(*entry));
        assert_eq!(entry.class(), class);
        assert!(matches!(
            entry.class(),
            DxfHandleGroupClass::SoftOwner | DxfHandleGroupClass::HardOwner
        ));
    }
    assert_eq!(directory.resolved_links().len(), 3);
    for link in directory.resolved_links() {
        let entry = directory
            .entry(link.evidence_ordinal())
            .ok_or_else(invalid_test_data)?;
        assert_eq!(entry.state(), DxfHandleResolutionState::Unique);
        assert_eq!(
            directory
                .resolution_directory()
                .targets_for_reference(entry.resolution_ordinal())
                .ok_or_else(invalid_test_data)?,
            &[link.target()]
        );
    }
    let invalid = directory.entries()[5];
    let mut spelling = [0_u8; 3];
    invalid
        .resolution()
        .reference()
        .value()
        .read_raw_spelling(view, &mut spelling)?;
    assert_eq!(&spelling, b"0x1");
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
    if version != DxfAcadVersion::Ac1009 {
        text.push_str("0\nOWNER1\n5\n1\n330\nA\n350\nA\n");
        text.push_str("0\nOWNER2\n5\n2\n340\nA\n360\nA\n");
        text.push_str("0\nOWNER3\n5\n3\n350\nB\n");
        text.push_str("0\nBROKEN\n5\n4\n350\nF\n360\n0\n350\n0x1\n360\nD\n");
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
        ("TARGETB", "B"),
        ("TARGETD1", "D"),
        ("TARGETD2", "d"),
    ] {
        push_binary_string(&mut bytes, version, 0, kind.as_bytes())?;
        push_binary_string(&mut bytes, version, 5, handle.as_bytes())?;
    }
    if version != DxfAcadVersion::Ac1009 {
        for (code, value) in [
            (0, "OWNER1"),
            (5, "1"),
            (330, "A"),
            (350, "A"),
            (0, "OWNER2"),
            (5, "2"),
            (340, "A"),
            (360, "A"),
            (0, "OWNER3"),
            (5, "3"),
            (350, "B"),
            (0, "BROKEN"),
            (5, "4"),
            (350, "F"),
            (360, "0"),
            (350, "0x1"),
            (360, "D"),
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

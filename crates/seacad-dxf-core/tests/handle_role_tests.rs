use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfApplicationGroupState, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfHandleResolutionState,
    DxfHandleRoleDirectory, DxfHandleRoleEntry, DxfHandleRoleEvidence, DxfMemorySource,
    DxfRawDocumentView, DxfRawRecordSectionKind, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_handle_role_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.handle_role_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory, version)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.handle_role_directory(&DxfCancellationToken::default())?;
        assert_directory(
            DxfRawDocumentView::from(&binary),
            &binary_directory,
            version,
        )?;
    }
    Ok(())
}

#[test]
fn candidate_roles_require_exact_code_context_closure_and_common_section()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_role_directory(&DxfCancellationToken::default())?;

    let expected = [
        DxfHandleRoleEvidence::GenericPointer,
        DxfHandleRoleEvidence::PersistentReactorCandidate,
        DxfHandleRoleEvidence::GenericPointer,
        DxfHandleRoleEvidence::ExtensionDictionaryCandidate,
        DxfHandleRoleEvidence::GenericOwnership,
        DxfHandleRoleEvidence::CommonOwnerPointerCandidate,
        DxfHandleRoleEvidence::GenericPointer,
        DxfHandleRoleEvidence::GenericOwnership,
        DxfHandleRoleEvidence::GenericPointer,
    ];
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.role())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(matches!(
        directory.entries()[5].contextual().resolution().state(),
        DxfHandleResolutionState::Invalid(_)
    ));
    assert_eq!(
        directory.entries()[5].role(),
        DxfHandleRoleEvidence::CommonOwnerPointerCandidate,
        "role shape must not erase an independent lexical target failure"
    );
    Ok(())
}

#[test]
fn directory_is_cancellable_linear_source_ordered_and_publicly_bounded()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfHandleRoleDirectory>();
    assert_copy::<DxfHandleRoleEntry>();
    assert_copy::<DxfHandleRoleEvidence>();
    assert_send_sync::<DxfHandleRoleEntry>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.handle_role_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads() - reads_after_open, 14);
    assert_eq!(
        directory.entries().len(),
        directory.contextual_directory().entries().len()
    );
    assert!(std::mem::size_of::<DxfHandleRoleEntry>() <= 192);
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_group(u64::MAX), None);
    assert_eq!(directory.entries_for_record(u64::MAX), None);
    assert!(!format!("{directory:?}").contains("0x1"));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.handle_role_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

#[test]
fn candidate_roles_cover_every_common_record_section() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nTABLE\n102\n{ACAD_REACTORS\n330\nA\n102\n}\n102\n{ACAD_XDICTIONARY\n360\nB\n102\n}\n330\nC\n0\nENDTAB\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n102\n{ACAD_REACTORS\n330\nD\n102\n}\n102\n{ACAD_XDICTIONARY\n360\nE\n102\n}\n330\nF\n0\nENDBLK\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nLINE\n102\n{ACAD_REACTORS\n330\n10\n102\n}\n102\n{ACAD_XDICTIONARY\n360\n11\n102\n}\n330\n12\n0\nENDSEC\n\
0\nSECTION\n2\nOBJECTS\n0\nDICTIONARY\n102\n{ACAD_REACTORS\n330\n13\n102\n}\n102\n{ACAD_XDICTIONARY\n360\n14\n102\n}\n330\n15\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_role_directory(&DxfCancellationToken::default())?;

    let expected_roles = [
        DxfHandleRoleEvidence::PersistentReactorCandidate,
        DxfHandleRoleEvidence::ExtensionDictionaryCandidate,
        DxfHandleRoleEvidence::CommonOwnerPointerCandidate,
    ];
    let expected_sections = [
        DxfRawRecordSectionKind::Tables,
        DxfRawRecordSectionKind::Blocks,
        DxfRawRecordSectionKind::Entities,
        DxfRawRecordSectionKind::Objects,
    ];
    assert_eq!(directory.entries().len(), 12);
    for (entries, section) in directory.entries().chunks_exact(3).zip(expected_sections) {
        assert_eq!(
            entries.iter().map(|entry| entry.role()).collect::<Vec<_>>(),
            expected_roles
        );
        assert!(entries.iter().all(|entry| {
            entry
                .contextual()
                .resolution()
                .reference()
                .record()
                .section_kind()
                == section
        }));
    }
    Ok(())
}

#[test]
fn interrupted_and_unclosed_application_groups_remain_generic() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nLINE\n102\n{ACAD_REACTORS\n330\nA\n\
102\n{ACAD_XDICTIONARY\n360\nB\n102\n}\n\
102\n{ACAD_XDICTIONARY\n360\nC\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_role_directory(&DxfCancellationToken::default())?;

    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.role())
            .collect::<Vec<_>>(),
        [
            DxfHandleRoleEvidence::GenericPointer,
            DxfHandleRoleEvidence::ExtensionDictionaryCandidate,
            DxfHandleRoleEvidence::GenericOwnership,
        ]
    );
    assert_eq!(
        directory
            .contextual_directory()
            .application_group_for_entry(0)
            .ok_or_else(invalid_test_data)?
            .state(),
        DxfApplicationGroupState::Interrupted
    );
    assert_eq!(
        directory
            .contextual_directory()
            .application_group_for_entry(1)
            .ok_or_else(invalid_test_data)?
            .state(),
        DxfApplicationGroupState::Closed
    );
    assert_eq!(
        directory
            .contextual_directory()
            .application_group_for_entry(2)
            .ok_or_else(invalid_test_data)?
            .state(),
        DxfApplicationGroupState::Unclosed
    );
    Ok(())
}

#[test]
fn candidate_role_is_independent_of_every_target_resolution_state() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nLINE\n5\nA\n0\nLINE\n5\nB\n0\nLINE\n5\nB\n0\nLINE\n\
330\n0x1\n330\n0\n330\nF\n330\nA\n330\nB\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_role_directory(&DxfCancellationToken::default())?;

    assert!(
        directory
            .entries()
            .iter()
            .all(|entry| entry.role() == DxfHandleRoleEvidence::CommonOwnerPointerCandidate)
    );
    assert!(matches!(
        directory.entries()[0].contextual().resolution().state(),
        DxfHandleResolutionState::Invalid(_)
    ));
    assert_eq!(
        directory.entries()[1].contextual().resolution().state(),
        DxfHandleResolutionState::Null
    );
    assert_eq!(
        directory.entries()[2].contextual().resolution().state(),
        DxfHandleResolutionState::Missing
    );
    assert_eq!(
        directory.entries()[3].contextual().resolution().state(),
        DxfHandleResolutionState::Unique
    );
    assert_eq!(
        directory.entries()[4].contextual().resolution().state(),
        DxfHandleResolutionState::Ambiguous { target_count: 2 }
    );
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfHandleRoleDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(
        directory.contextual_directory().source_id(),
        view.source_id()
    );
    let expected: &[DxfHandleRoleEvidence] = if version == DxfAcadVersion::Ac1009 {
        &[
            DxfHandleRoleEvidence::GenericPointer,
            DxfHandleRoleEvidence::GenericPointer,
            DxfHandleRoleEvidence::GenericPointer,
        ]
    } else {
        &[
            DxfHandleRoleEvidence::GenericPointer,
            DxfHandleRoleEvidence::PersistentReactorCandidate,
            DxfHandleRoleEvidence::GenericPointer,
            DxfHandleRoleEvidence::ExtensionDictionaryCandidate,
            DxfHandleRoleEvidence::GenericOwnership,
            DxfHandleRoleEvidence::CommonOwnerPointerCandidate,
            DxfHandleRoleEvidence::GenericPointer,
            DxfHandleRoleEvidence::GenericOwnership,
            DxfHandleRoleEvidence::GenericPointer,
        ]
    };
    assert_eq!(directory.entries().len(), expected.len());
    for (ordinal, (entry, role)) in directory.entries().iter().zip(expected).enumerate() {
        let ordinal = u64::try_from(ordinal)?;
        assert_eq!(directory.entry(ordinal), Some(*entry));
        assert_eq!(entry.role(), *role);
        let occurrence = entry
            .contextual()
            .resolution()
            .reference()
            .value()
            .group()
            .occurrence();
        assert_eq!(directory.entry_for_group(occurrence), Some(*entry));
    }
    let first_record = directory.entries()[0]
        .contextual()
        .resolution()
        .reference()
        .record()
        .ordinal();
    assert_eq!(
        directory
            .entries_for_record(first_record)
            .ok_or_else(invalid_test_data)?
            .len(),
        1
    );
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nCLASSES\n0\nCLASS\n",
        version.code()
    );
    if version == DxfAcadVersion::Ac1009 {
        text.push_str("1005\nA\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n102\n{ACAD_REACTORS\n1005\nB\n102\n}\n1005\nC\n");
    } else {
        text.push_str("330\nA\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n102\n{ACAD_REACTORS\n330\nB\n331\nC\n102\n}\n102\n{ACAD_XDICTIONARY\n360\nD\n361\nE\n102\n}\n330\n0x1\n340\nG\n350\nH\n102\n{ACAD_REACTORS\n330\nI\n");
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
        (2, "CLASSES"),
        (0, "CLASS"),
    ] {
        push_binary_string(&mut bytes, version, code, value.as_bytes())?;
    }
    if version == DxfAcadVersion::Ac1009 {
        for (code, value) in [
            (1005, "A"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "LINE"),
            (102, "{ACAD_REACTORS"),
            (1005, "B"),
            (102, "}"),
            (1005, "C"),
        ] {
            push_binary_string(&mut bytes, version, code, value.as_bytes())?;
        }
    } else {
        for (code, value) in [
            (330, "A"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "LINE"),
            (102, "{ACAD_REACTORS"),
            (330, "B"),
            (331, "C"),
            (102, "}"),
            (102, "{ACAD_XDICTIONARY"),
            (360, "D"),
            (361, "E"),
            (102, "}"),
            (330, "0x1"),
            (340, "G"),
            (350, "H"),
            (102, "{ACAD_REACTORS"),
            (330, "I"),
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

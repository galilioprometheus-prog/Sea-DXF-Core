use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfApplicationControlEntry, DxfApplicationControlKind,
    DxfApplicationGroupDirectory, DxfApplicationGroupEntry, DxfApplicationGroupKind,
    DxfApplicationGroupState, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_application_group_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.application_group_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.application_group_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&binary), &binary_directory)?;
    }
    Ok(())
}

#[test]
fn handle_occurrences_retain_exact_known_application_group_context() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nOBJ\n5\n1\n102\n{ACAD_REACTORS\n330\nA\n102\n}\n102\n{ACAD_XDICTIONARY\n360\nB\n102\n}\n330\nC\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = document.application_group_directory(&DxfCancellationToken::default())?;

    let reactor_handle = occurrence_of_code(view, 330, 0)?;
    let extension_dictionary = occurrence_of_code(view, 360, 0)?;
    let outside_owner_pointer = occurrence_of_code(view, 330, 1)?;
    assert_eq!(
        directory
            .group_for_content_occurrence(reactor_handle)
            .ok_or_else(invalid_test_data)?
            .kind(),
        DxfApplicationGroupKind::AcadReactors
    );
    assert_eq!(
        directory
            .group_for_content_occurrence(extension_dictionary)
            .ok_or_else(invalid_test_data)?
            .kind(),
        DxfApplicationGroupKind::AcadXDictionary
    );
    assert_eq!(
        directory.group_for_content_occurrence(outside_owner_pointer),
        None
    );
    Ok(())
}

#[test]
fn directory_is_cancellable_bounded_source_anchored_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfApplicationGroupDirectory>();
    assert_copy::<DxfApplicationControlEntry>();
    assert_copy::<DxfApplicationControlKind>();
    assert_copy::<DxfApplicationGroupEntry>();
    assert_copy::<DxfApplicationGroupKind>();
    assert_copy::<DxfApplicationGroupState>();
    assert_send_sync::<DxfApplicationControlEntry>();
    assert_send_sync::<DxfApplicationGroupEntry>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory = document.application_group_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads() - reads_after_open, 11);
    assert!(std::mem::size_of::<DxfApplicationControlEntry>() <= 112);
    assert!(std::mem::size_of::<DxfApplicationGroupEntry>() <= 112);
    assert_eq!(directory.control(u64::MAX), None);
    assert_eq!(directory.group(u64::MAX), None);
    assert_eq!(directory.controls_for_record(u64::MAX), None);
    assert_eq!(directory.groups_for_record(u64::MAX), None);
    assert_eq!(directory.group_for_content_occurrence(u64::MAX), None);
    assert!(!format!("{directory:?}").contains("ACAD_REACTORS"));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.application_group_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfApplicationGroupDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(directory.record_count(), 3);
    assert_eq!(directory.controls().len(), 11);
    assert_eq!(directory.groups().len(), 6);

    let expected_controls = [
        DxfApplicationControlKind::Start(DxfApplicationGroupKind::AcadReactors),
        DxfApplicationControlKind::Close,
        DxfApplicationControlKind::Start(DxfApplicationGroupKind::AcadXDictionary),
        DxfApplicationControlKind::Close,
        DxfApplicationControlKind::Start(DxfApplicationGroupKind::Other),
        DxfApplicationControlKind::Close,
        DxfApplicationControlKind::Invalid,
        DxfApplicationControlKind::Close,
        DxfApplicationControlKind::Start(DxfApplicationGroupKind::Other),
        DxfApplicationControlKind::Start(DxfApplicationGroupKind::AcadReactors),
        DxfApplicationControlKind::Start(DxfApplicationGroupKind::AcadXDictionary),
    ];
    for (ordinal, (entry, expected)) in directory
        .controls()
        .iter()
        .zip(expected_controls)
        .enumerate()
    {
        assert_eq!(directory.control(u64::try_from(ordinal)?), Some(*entry));
        assert_eq!(entry.kind(), expected);
        assert_eq!(entry.group().group_code().value(), 102);
        assert!(
            entry.group().occurrence() >= entry.record().group_range().start()
                && entry.group().occurrence() < entry.record().group_range().end()
        );
    }

    let expected_groups = [
        (
            DxfApplicationGroupKind::AcadReactors,
            DxfApplicationGroupState::Closed,
            Some(1),
        ),
        (
            DxfApplicationGroupKind::AcadXDictionary,
            DxfApplicationGroupState::Closed,
            Some(3),
        ),
        (
            DxfApplicationGroupKind::Other,
            DxfApplicationGroupState::Closed,
            Some(5),
        ),
        (
            DxfApplicationGroupKind::Other,
            DxfApplicationGroupState::Interrupted,
            None,
        ),
        (
            DxfApplicationGroupKind::AcadReactors,
            DxfApplicationGroupState::Unclosed,
            None,
        ),
        (
            DxfApplicationGroupKind::AcadXDictionary,
            DxfApplicationGroupState::Unclosed,
            None,
        ),
    ];
    for (ordinal, (entry, (kind, state, end_control))) in
        directory.groups().iter().zip(expected_groups).enumerate()
    {
        assert_eq!(directory.group(u64::try_from(ordinal)?), Some(*entry));
        assert_eq!(entry.kind(), kind);
        assert_eq!(entry.state(), state);
        assert_eq!(entry.end_control_ordinal(), end_control);
        assert!(entry.source_group_range().start() <= entry.content_group_range().start());
        assert!(entry.content_group_range().end() <= entry.source_group_range().end());
    }
    assert_eq!(
        directory
            .controls_for_record(0)
            .ok_or_else(invalid_test_data)?
            .len(),
        8
    );
    assert_eq!(
        directory
            .groups_for_record(0)
            .ok_or_else(invalid_test_data)?
            .len(),
        3
    );
    assert_eq!(
        directory
            .controls_for_record(1)
            .ok_or_else(invalid_test_data)?
            .len(),
        2
    );
    assert_eq!(
        directory
            .groups_for_record(1)
            .ok_or_else(invalid_test_data)?
            .len(),
        2
    );
    assert_eq!(
        directory
            .controls_for_record(2)
            .ok_or_else(invalid_test_data)?
            .len(),
        1
    );
    assert_eq!(
        directory
            .groups_for_record(2)
            .ok_or_else(invalid_test_data)?
            .len(),
        1
    );

    let first_group = directory.groups()[0];
    let first_content = first_group.content_group_range().start();
    assert_eq!(
        directory.group_for_content_occurrence(first_content),
        Some(first_group)
    );
    assert_eq!(
        directory.group_for_content_occurrence(first_group.source_group_range().start()),
        None
    );
    let other_start = directory.controls()[4];
    let span = other_start.group().value_payload_span();
    let mut spelling = vec![0_u8; usize::try_from(span.len())?];
    view.read_span(span, &mut spelling)?;
    assert_eq!(spelling, b"{CUSTOM");
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nCOMPLETE\n102\n{{ACAD_REACTORS\n1\nA\n102\n}}\n102\n{{ACAD_XDICTIONARY\n1\nB\n102\n}}\n102\n{{CUSTOM\n1\nC\n102\n}}\n102\nBROKEN\n102\n}}\n0\nINTERRUPTED\n102\n{{FIRST\n1\nX\n102\n{{ACAD_REACTORS\n1\nY\n0\nUNCLOSED\n102\n{{ACAD_XDICTIONARY\n1\nZ\n0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
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
        (0, "COMPLETE"),
        (102, "{ACAD_REACTORS"),
        (1, "A"),
        (102, "}"),
        (102, "{ACAD_XDICTIONARY"),
        (1, "B"),
        (102, "}"),
        (102, "{CUSTOM"),
        (1, "C"),
        (102, "}"),
        (102, "BROKEN"),
        (102, "}"),
        (0, "INTERRUPTED"),
        (102, "{FIRST"),
        (1, "X"),
        (102, "{ACAD_REACTORS"),
        (1, "Y"),
        (0, "UNCLOSED"),
        (102, "{ACAD_XDICTIONARY"),
        (1, "Z"),
        (0, "ENDSEC"),
        (0, "EOF"),
    ] {
        push_binary_string(&mut bytes, version, code, value.as_bytes())?;
    }
    Ok(bytes)
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

fn occurrence_of_code(
    view: DxfRawDocumentView<'_>,
    code: i16,
    wanted_index: usize,
) -> Result<u64, DxfError> {
    let mut observed_index = 0_usize;
    for occurrence in 0..view.group_count() {
        if view
            .group(occurrence)
            .is_some_and(|group| group.group_code().value() == code)
        {
            if observed_index == wanted_index {
                return Ok(occurrence);
            }
            observed_index = observed_index.saturating_add(1);
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

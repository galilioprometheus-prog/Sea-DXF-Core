use std::{
    error::Error,
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfApplicationGroupState, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken,
    DxfContextualHandleReferenceDirectory, DxfContextualHandleReferenceEntry, DxfError,
    DxfHandleGroupClass, DxfHandleReferenceContext, DxfHandleResolutionState, DxfMemorySource,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_contextual_reference_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.contextual_handle_reference_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&ascii), &ascii_directory)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.contextual_handle_reference_directory(&DxfCancellationToken::default())?;
        assert_directory(DxfRawDocumentView::from(&binary), &binary_directory)?;
    }
    Ok(())
}

#[test]
fn numeric_reference_class_and_application_context_remain_independent() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nTARGETA\n5\nA\n0\nTARGETB\n5\nB\n0\nTARGETC\n5\nC\n0\nSOURCE\n5\n1\n102\n{ACAD_REACTORS\n330\nA\n102\n}\n102\n{ACAD_XDICTIONARY\n360\nB\n102\n}\n330\nC\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.contextual_handle_reference_directory(&DxfCancellationToken::default())?;

    let expected = [
        (
            DxfHandleGroupClass::SoftPointer,
            DxfHandleReferenceContext::AcadReactors,
        ),
        (
            DxfHandleGroupClass::HardOwner,
            DxfHandleReferenceContext::AcadXDictionary,
        ),
        (
            DxfHandleGroupClass::SoftPointer,
            DxfHandleReferenceContext::OutsideApplicationGroup,
        ),
    ];
    for (entry, (class, context)) in directory.entries().iter().zip(expected) {
        assert_eq!(entry.resolution().reference().class(), class);
        assert_eq!(entry.context(), context);
        assert_eq!(entry.resolution().state(), DxfHandleResolutionState::Unique);
    }
    Ok(())
}

#[test]
fn directory_is_cancellable_linear_source_ordered_and_publicly_bounded()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfContextualHandleReferenceDirectory>();
    assert_copy::<DxfContextualHandleReferenceEntry>();
    assert_copy::<DxfHandleReferenceContext>();
    assert_send_sync::<DxfContextualHandleReferenceEntry>();

    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = CountingSource::new(&bytes);
    let document = open_ascii(&source)?;
    let reads_after_open = source.reads();
    let directory =
        document.contextual_handle_reference_directory(&DxfCancellationToken::default())?;
    assert_eq!(source.reads() - reads_after_open, 18);
    assert_eq!(
        directory.entries().len(),
        directory.resolution_directory().entries().len()
    );
    assert!(std::mem::size_of::<DxfContextualHandleReferenceEntry>() <= 176);
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_group(u64::MAX), None);
    assert_eq!(directory.entries_for_record(u64::MAX), None);
    assert_eq!(directory.application_group_for_entry(u64::MAX), None);
    assert!(!format!("{directory:?}").contains("ACAD_REACTORS"));

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.contextual_handle_reference_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_directory(
    view: DxfRawDocumentView<'_>,
    directory: &DxfContextualHandleReferenceDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), view.source_id());
    assert_eq!(
        directory.resolution_directory().source_id(),
        view.source_id()
    );
    assert_eq!(
        directory.application_group_directory().source_id(),
        view.source_id()
    );
    assert_eq!(directory.entries().len(), 5);
    let expected = [
        DxfHandleReferenceContext::AcadReactors,
        DxfHandleReferenceContext::AcadXDictionary,
        DxfHandleReferenceContext::OtherApplicationGroup,
        DxfHandleReferenceContext::OutsideApplicationGroup,
        DxfHandleReferenceContext::AcadReactors,
    ];
    for (ordinal, (entry, context)) in directory.entries().iter().zip(expected).enumerate() {
        let ordinal = u64::try_from(ordinal)?;
        assert_eq!(directory.entry(ordinal), Some(*entry));
        assert_eq!(entry.context(), context);
        assert_eq!(entry.resolution().state(), DxfHandleResolutionState::Unique);
        let occurrence = entry.resolution().reference().value().group().occurrence();
        assert_eq!(directory.entry_for_group(occurrence), Some(*entry));
        match entry.context() {
            DxfHandleReferenceContext::OutsideApplicationGroup => {
                assert_eq!(entry.application_group_ordinal(), None);
                assert_eq!(directory.application_group_for_entry(ordinal), None);
            }
            DxfHandleReferenceContext::AcadReactors
            | DxfHandleReferenceContext::AcadXDictionary
            | DxfHandleReferenceContext::OtherApplicationGroup => {
                let group = directory
                    .application_group_for_entry(ordinal)
                    .ok_or_else(invalid_test_data)?;
                assert_eq!(Some(group.kind()), context_group_kind(context));
            }
            _ => return Err(invalid_test_data().into()),
        }
    }
    assert_eq!(
        directory
            .application_group_for_entry(4)
            .ok_or_else(invalid_test_data)?
            .state(),
        DxfApplicationGroupState::Unclosed
    );
    let source_record = directory.entries()[0]
        .resolution()
        .reference()
        .record()
        .ordinal();
    assert_eq!(
        directory
            .entries_for_record(source_record)
            .ok_or_else(invalid_test_data)?,
        &directory.entries()[0..4]
    );
    Ok(())
}

fn context_group_kind(
    context: DxfHandleReferenceContext,
) -> Option<seacad_dxf_core::DxfApplicationGroupKind> {
    match context {
        DxfHandleReferenceContext::AcadReactors => {
            Some(seacad_dxf_core::DxfApplicationGroupKind::AcadReactors)
        }
        DxfHandleReferenceContext::AcadXDictionary => {
            Some(seacad_dxf_core::DxfApplicationGroupKind::AcadXDictionary)
        }
        DxfHandleReferenceContext::OtherApplicationGroup => {
            Some(seacad_dxf_core::DxfApplicationGroupKind::Other)
        }
        DxfHandleReferenceContext::OutsideApplicationGroup => None,
        _ => None,
    }
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nTARGETA\n5\nA\n0\nTARGETB\n5\nB\n0\nTARGETC\n5\nC\n0\nTARGETD\n5\nD\n0\nSOURCE\n5\n1\n102\n{{ACAD_REACTORS\n1005\nA\n102\n}}\n102\n{{ACAD_XDICTIONARY\n1005\nB\n102\n}}\n102\n{{CUSTOM\n1005\nC\n102\n}}\n1005\nD\n0\nUNCLOSED\n5\n2\n102\n{{ACAD_REACTORS\n1005\nA\n0\nENDSEC\n0\nEOF\n",
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
        (0, "TARGETA"),
        (5, "A"),
        (0, "TARGETB"),
        (5, "B"),
        (0, "TARGETC"),
        (5, "C"),
        (0, "TARGETD"),
        (5, "D"),
        (0, "SOURCE"),
        (5, "1"),
        (102, "{ACAD_REACTORS"),
        (1005, "A"),
        (102, "}"),
        (102, "{ACAD_XDICTIONARY"),
        (1005, "B"),
        (102, "}"),
        (102, "{CUSTOM"),
        (1005, "C"),
        (102, "}"),
        (1005, "D"),
        (0, "UNCLOSED"),
        (5, "2"),
        (102, "{ACAD_REACTORS"),
        (1005, "A"),
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

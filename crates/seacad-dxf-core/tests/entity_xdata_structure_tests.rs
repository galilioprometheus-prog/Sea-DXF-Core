use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataStructureDirectory, DxfEntityXDataStructureEntry,
    DxfEntityXDataStructureIssue, DxfEntityXDataStructureIssueKind, DxfEntityXDataStructureState,
    DxfError, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_structure_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let signature = match format {
                DxfRawDocumentFormat::Ascii => {
                    let document = open_ascii(&source)?;
                    signature(DxfRawDocumentView::from(&document))?
                }
                DxfRawDocumentFormat::Binary => {
                    let document = open_binary(&source)?;
                    signature(DxfRawDocumentView::from(&document))?
                }
                _ => return Err(io::Error::other("test format").into()),
            };
            assert_eq!(signature, [0, 1, 2, 3, 4, 5, 3]);
        }
    }
    Ok(())
}

#[test]
fn length_controls_balance_and_interruption_remain_distinct() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_xdata_structure_directory(&token())?;
    assert_eq!(directory.entries().len(), 7);
    assert_eq!(directory.issues().len(), 7);
    assert!(matches!(
        directory.entries()[0].state(),
        DxfEntityXDataStructureState::Valid
    ));
    let expected = [
        DxfEntityXDataStructureIssueKind::ApplicationNameTooLong { byte_count: 32 },
        DxfEntityXDataStructureIssueKind::InvalidControlString {
            group: directory.issues()[1].kind_group()?,
        },
        DxfEntityXDataStructureIssueKind::UnexpectedListEnd {
            group: directory.issues()[2].kind_group()?,
        },
        DxfEntityXDataStructureIssueKind::UnclosedLists { open_count: 1 },
        DxfEntityXDataStructureIssueKind::Interrupted,
        DxfEntityXDataStructureIssueKind::UnclosedLists { open_count: 1 },
        DxfEntityXDataStructureIssueKind::UnexpectedListEnd {
            group: directory.issues()[6].kind_group()?,
        },
    ];
    assert_eq!(
        directory
            .issues()
            .iter()
            .map(|issue| issue.kind())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(directory.issues_for_entry(directory.entries()[5])?.len(), 2);
    Ok(())
}

#[test]
fn directory_is_source_bound_cancellable_bounded_and_non_disclosing() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityXDataStructureDirectory>();
    assert_copy::<DxfEntityXDataStructureEntry>();
    assert_copy::<DxfEntityXDataStructureIssue>();
    assert_copy::<DxfEntityXDataStructureIssueKind>();
    assert!(std::mem::size_of::<DxfEntityXDataStructureEntry>() <= 224);
    assert!(std::mem::size_of::<DxfEntityXDataStructureIssue>() <= 224);

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_xdata_structure_directory(&token())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.xdata_directory().source_id(),
        document.source_id()
    );
    assert_eq!(directory.entry(u64::MAX), None);
    let first = directory.entries()[0];
    assert_eq!(directory.entry_for_application(first.application())?, first);
    assert!(directory.issues_for_entry(first)?.is_empty());
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_VALID"));
    assert!(!debug.contains("SECRET_CONTROL"));

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other = other_document.entity_xdata_structure_directory(&token())?;
    assert!(matches!(
        directory.entry_for_application(other.entries()[0].application()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.entity_xdata_structure_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

trait IssueGroup {
    fn kind_group(self) -> Result<seacad_dxf_core::DxfRawGroup, io::Error>;
}

impl IssueGroup for DxfEntityXDataStructureIssue {
    fn kind_group(self) -> Result<seacad_dxf_core::DxfRawGroup, io::Error> {
        match self.kind() {
            DxfEntityXDataStructureIssueKind::InvalidControlString { group }
            | DxfEntityXDataStructureIssueKind::UnexpectedListEnd { group } => Ok(group),
            _ => Err(io::Error::other("group issue")),
        }
    }
}

fn signature(view: DxfRawDocumentView<'_>) -> Result<Vec<u8>, Box<dyn Error>> {
    let directory = view.entity_xdata_structure_directory(&token())?;
    let mut result = Vec::new();
    for entry in directory.entries().iter().copied() {
        let code = match directory.issues_for_entry(entry)? {
            [] => 0,
            [issue] => match issue.kind() {
                DxfEntityXDataStructureIssueKind::ApplicationNameTooLong { byte_count: 32 } => 1,
                DxfEntityXDataStructureIssueKind::InvalidControlString { .. } => 2,
                DxfEntityXDataStructureIssueKind::UnexpectedListEnd { .. } => 3,
                DxfEntityXDataStructureIssueKind::UnclosedLists { open_count: 1 } => 4,
                _ => return Err(io::Error::other("single issue").into()),
            },
            [first, second]
                if first.kind() == DxfEntityXDataStructureIssueKind::Interrupted
                    && second.kind()
                        == (DxfEntityXDataStructureIssueKind::UnclosedLists { open_count: 1 }) =>
            {
                5
            }
            _ => return Err(io::Error::other("issue signature").into()),
        };
        result.push(code);
    }
    Ok(result)
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let groups: Vec<(i16, &[u8])> = vec![
        (0, b"SECTION"),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"POINT"),
        (1001, b"SECRET_VALID"),
        (1002, b"{"),
        (1002, b"{"),
        (1000, b"A"),
        (1002, b"}"),
        (1002, b"}"),
        (1001, b"12345678901234567890123456789012"),
        (1000, b"B"),
        (1001, b"BAD_CONTROL"),
        (1002, b"SECRET_CONTROL"),
        (1001, b"BAD_CLOSE"),
        (1002, b"}"),
        (1001, b"BAD_OPEN"),
        (1002, b"{"),
        (1002, b"{"),
        (1002, b"}"),
        (1001, b"INTERRUPTED"),
        (1002, b"{"),
        (8, b"NORMAL"),
        (1001, b"AFTER_INTERRUPT"),
        (1002, b"}"),
        (0, b"ENDSEC"),
        (0, b"EOF"),
    ];
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(value);
        bytes.push(b'\n');
    }
    bytes
}

fn binary_groups(version: DxfAcadVersion, groups: &[(i16, &[u8])]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        push_code(&mut bytes, version, *code)?;
        bytes.extend_from_slice(value);
        bytes.push(0);
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

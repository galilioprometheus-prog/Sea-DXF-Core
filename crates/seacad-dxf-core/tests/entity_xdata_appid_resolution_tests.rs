use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataAppIdResolutionDirectory,
    DxfEntityXDataAppIdResolutionEntry, DxfEntityXDataAppIdResolutionState, DxfError,
    DxfMemorySource, DxfNamedSymbolTableKind, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_exact_appid_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let expected = [0_u8, 1, 2, 2];
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, true)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let signature = match format {
                DxfRawDocumentFormat::Ascii => {
                    let document = open_ascii(&source)?;
                    assert_valid(DxfRawDocumentView::from(&document))?
                }
                DxfRawDocumentFormat::Binary => {
                    let document = open_binary(&source)?;
                    assert_valid(DxfRawDocumentView::from(&document))?
                }
                _ => return Err(io::Error::other("test format").into()),
            };
            assert_eq!(signature, expected);
        }
    }
    Ok(())
}

#[test]
fn malformed_near_match_and_unclosed_appid_tables_fail_closed() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, false)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let states: Vec<_> = match format {
            DxfRawDocumentFormat::Ascii => open_ascii(&source)?
                .entity_xdata_appid_resolution_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.state())
                .collect(),
            DxfRawDocumentFormat::Binary => open_binary(&source)?
                .entity_xdata_appid_resolution_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.state())
                .collect(),
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(states.len(), 4);
        assert!(
            states
                .iter()
                .all(|state| *state == DxfEntityXDataAppIdResolutionState::Missing)
        );
    }
    Ok(())
}

#[test]
fn resolution_is_source_bound_cancellable_bounded_and_non_disclosing() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityXDataAppIdResolutionDirectory>();
    assert_copy::<DxfEntityXDataAppIdResolutionEntry>();
    assert_copy::<DxfEntityXDataAppIdResolutionState>();
    let entry_size = std::mem::size_of::<DxfEntityXDataAppIdResolutionEntry>();
    assert!(
        entry_size <= 256,
        "APPID resolution entry grew to {entry_size} bytes"
    );

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, true)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_xdata_appid_resolution_directory(&token())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.xdata_directory().source_id(),
        document.source_id()
    );
    assert_eq!(
        directory.named_symbol_table_directory().source_id(),
        document.source_id()
    );
    assert_eq!(directory.entry(u64::MAX), None);
    let application = directory.xdata_directory().applications()[0];
    assert_eq!(
        directory.entry_for_application(application)?,
        directory.entries()[0]
    );
    assert_eq!(directory.entries_for_entity(application.entity())?.len(), 4);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_APP_ONE"));
    assert!(!debug.contains("SECRET_PAYLOAD"));

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027, true)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other = other_document.entity_xdata_appid_resolution_directory(&token())?;
    assert!(matches!(
        directory.entry_for_application(other.xdata_directory().applications()[0]),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(matches!(
        directory.entries_for_entity(other.xdata_directory().applications()[0].entity()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.entity_xdata_appid_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_valid(view: DxfRawDocumentView<'_>) -> Result<Vec<u8>, Box<dyn Error>> {
    let directory = view.entity_xdata_appid_resolution_directory(&token())?;
    assert_eq!(directory.entries().len(), 4);
    assert_eq!(
        directory
            .named_symbol_table_directory()
            .entries()
            .iter()
            .filter(|entry| entry.kind() == DxfNamedSymbolTableKind::AppId)
            .count(),
        4
    );
    let mut signature = Vec::new();
    for entry in directory.entries().iter().copied() {
        let code = match entry.state() {
            DxfEntityXDataAppIdResolutionState::Unique { target } => {
                assert_eq!(target.kind(), DxfNamedSymbolTableKind::AppId);
                assert!(!target.name().value_span().is_empty());
                0
            }
            DxfEntityXDataAppIdResolutionState::Ambiguous { target_count: 2 } => 1,
            DxfEntityXDataAppIdResolutionState::Missing => 2,
            _ => return Err(io::Error::other("APPID state").into()),
        };
        assert_eq!(directory.entry_for_application(entry.application())?, entry);
        signature.push(code);
    }
    Ok(signature)
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    closed_exact_table: bool,
) -> Result<Vec<u8>, io::Error> {
    let mut groups: Vec<(i16, &[u8])> = vec![
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"TABLES"),
        (0, b"TABLE"),
        (2, b"APPID"),
        (0, b"APPID"),
        (2, b"SECRET_APP_ONE"),
        (0, b"APPID"),
        (2, b"DUP"),
        (0, b"APPID"),
        (2, b"DUP"),
        (0, b"APPID"),
        (2, b"NearCase"),
    ];
    if closed_exact_table {
        groups.push((0, b"ENDTAB"));
    }
    groups.push((0, b"ENDSEC"));
    if !closed_exact_table {
        groups.extend([
            (0, b"SECTION".as_slice()),
            (2, b"TABLES"),
            (0, b"TABLE"),
            (2, b"appid"),
            (0, b"APPID"),
            (2, b"SECRET_APP_ONE"),
            (0, b"ENDTAB"),
            (0, b"ENDSEC"),
            (0, b"SECTION"),
            (2, b"TABLES"),
            (0, b"TABLE"),
            (2, b"APPID"),
            (0, b"APPID"),
            (2, b"SECRET_APP_ONE"),
            (2, b"DUP"),
            (0, b"ENDTAB"),
            (0, b"ENDSEC"),
        ]);
    }
    groups.extend([
        (0, b"SECTION".as_slice()),
        (0, b"SECTION"),
        (2, b"TABLES"),
        (0, b"TABLE"),
        (2, b"LAYER"),
        (0, b"APPID"),
        (2, b"MISSING"),
        (0, b"ENDTAB"),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"POINT"),
        (1001, b"SECRET_APP_ONE"),
        (1000, b"SECRET_PAYLOAD"),
        (1001, b"DUP"),
        (1000, b"A"),
        (1001, b"MISSING"),
        (1000, b"B"),
        (1001, b"nearcase"),
        (1000, b"C"),
        (0, b"ENDSEC"),
        (0, b"EOF"),
    ]);
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

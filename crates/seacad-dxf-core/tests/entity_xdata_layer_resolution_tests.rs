use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataLayerResolutionDirectory,
    DxfEntityXDataLayerResolutionEntry, DxfEntityXDataLayerResolutionState, DxfError,
    DxfMemorySource, DxfNamedSymbolTableKind, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_exact_layer_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
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
            assert_eq!(signature, [0_u8, 1, 2, 2, 0, 2, 2]);
        }
    }
    Ok(())
}

#[test]
fn unclosed_near_case_wrong_table_and_malformed_layers_fail_closed() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = fixture(format, DxfAcadVersion::Ac1032, false)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let states: Vec<_> = match format {
            DxfRawDocumentFormat::Ascii => open_ascii(&source)?
                .entity_xdata_layer_resolution_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.state())
                .collect(),
            DxfRawDocumentFormat::Binary => open_binary(&source)?
                .entity_xdata_layer_resolution_directory(&token())?
                .entries()
                .iter()
                .map(|entry| entry.state())
                .collect(),
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(states.len(), 7);
        assert!(
            states
                .iter()
                .all(|state| *state == DxfEntityXDataLayerResolutionState::Missing)
        );
    }
    Ok(())
}

#[test]
fn resolution_is_source_bound_cancellable_bounded_and_non_disclosing() -> Result<(), Box<dyn Error>>
{
    assert_send_sync::<DxfEntityXDataLayerResolutionDirectory>();
    assert_copy::<DxfEntityXDataLayerResolutionEntry>();
    assert_copy::<DxfEntityXDataLayerResolutionState>();
    let entry_size = size_of::<DxfEntityXDataLayerResolutionEntry>();
    assert!(
        entry_size <= 192,
        "layer resolution entry grew to {entry_size} bytes"
    );

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, true)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_xdata_layer_resolution_directory(&token())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.typed_directory().source_id(),
        document.source_id()
    );
    assert_eq!(
        directory.named_symbol_table_directory().source_id(),
        document.source_id()
    );
    assert_eq!(directory.entry(u64::MAX), None);
    let first = directory.entries()[0];
    assert_eq!(first.source_id(), directory.source_id());
    let first_source = directory
        .typed_entry(first)
        .ok_or(io::Error::other("first source"))?;
    assert_eq!(directory.entry_for_source(first_source), Some(first));
    let non_layer = directory.typed_directory().entries()[0];
    assert_eq!(directory.entry_for_source(non_layer), None);
    let applications = directory.typed_directory().xdata_directory().applications();
    assert_eq!(directory.entries_for_application(applications[0])?.len(), 4);
    assert_eq!(directory.entries_for_application(applications[1])?.len(), 2);
    assert_eq!(
        directory
            .entries_for_entity(applications[0].entity())?
            .len(),
        5
    );
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_LAYER"));
    assert!(!debug.contains("SECRET_PAYLOAD"));

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027, true)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other = other_document.entity_xdata_layer_resolution_directory(&token())?;
    let other_applications = other.typed_directory().xdata_directory().applications();
    assert!(matches!(
        directory.entries_for_application(other_applications[0]),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(matches!(
        directory.entries_for_entity(other_applications[0].entity()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    let foreign_resolution = other.entries()[0];
    let foreign_source = other
        .typed_entry(foreign_resolution)
        .ok_or(io::Error::other("foreign source"))?;
    assert_eq!(directory.entry_for_source(foreign_source), None);
    assert_eq!(directory.typed_entry(foreign_resolution), None);

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.entity_xdata_layer_resolution_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_valid(view: DxfRawDocumentView<'_>) -> Result<Vec<u8>, Box<dyn Error>> {
    let directory = view.entity_xdata_layer_resolution_directory(&token())?;
    assert_eq!(directory.entries().len(), 7);
    assert_eq!(
        directory
            .named_symbol_table_directory()
            .entries()
            .iter()
            .filter(|entry| entry.kind() == DxfNamedSymbolTableKind::Layer)
            .count(),
        4
    );
    let mut signature = Vec::new();
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let source = directory
            .typed_entry(entry)
            .ok_or(io::Error::other("source entry"))?;
        assert_eq!(directory.entry_for_source(source), Some(entry));
        let code = match entry.state() {
            DxfEntityXDataLayerResolutionState::Unique { target } => {
                assert_eq!(target.kind(), DxfNamedSymbolTableKind::Layer);
                assert!(!target.name().value_span().is_empty());
                0
            }
            DxfEntityXDataLayerResolutionState::Ambiguous { target_count: 2 } => 1,
            DxfEntityXDataLayerResolutionState::Missing => 2,
            _ => return Err(io::Error::other("layer state").into()),
        };
        signature.push(code);
    }
    Ok(signature)
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    closed_exact_table: bool,
) -> io::Result<Vec<u8>> {
    let mut groups: Vec<(i16, &[u8])> = vec![
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"TABLES"),
        (0, b"TABLE"),
        (2, b"LAYER"),
        (0, b"LAYER"),
        (2, b"SECRET_LAYER"),
        (0, b"LAYER"),
        (2, b"DUP"),
        (0, b"LAYER"),
        (2, b"DUP"),
        (0, b"LAYER"),
        (2, b"NearCase"),
    ];
    if closed_exact_table {
        groups.push((0, b"ENDTAB"));
    }
    groups.extend([(0, b"ENDSEC".as_slice()), (0, b"SECTION"), (2, b"TABLES")]);
    if !closed_exact_table {
        groups.extend([
            (0, b"TABLE".as_slice()),
            (2, b"layer"),
            (0, b"LAYER"),
            (2, b"SECRET_LAYER"),
            (0, b"ENDTAB"),
            (0, b"TABLE"),
            (2, b"LAYER"),
            (0, b"LAYER"),
            (2, b"SECRET_LAYER"),
            (2, b"DUP"),
            (0, b"ENDTAB"),
        ]);
    }
    groups.extend([
        (0, b"TABLE".as_slice()),
        (2, b"LTYPE"),
        (0, b"LTYPE"),
        (2, b"WrongTable"),
        (0, b"ENDTAB"),
        (0, b"TABLE"),
        (2, b"LAYER"),
        (0, b"LAYER"),
        (2, b"Unclosed"),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"POINT"),
        (1001, b"APP"),
        (1000, b"SECRET_PAYLOAD"),
        (1003, b"SECRET_LAYER"),
        (1003, b"DUP"),
        (1003, b"MISSING"),
        (1003, b"nearcase"),
        (8, b"0"),
        (1003, b"SECRET_LAYER"),
        (0, b"LINE"),
        (1001, b"APP2"),
        (1003, b"WrongTable"),
        (1003, b"Unclosed"),
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

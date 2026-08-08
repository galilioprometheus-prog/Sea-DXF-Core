use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataAppIdDestinationState, DxfEntityXDataLayerDestinationState,
    DxfEntityXDataSymbolDestinationDirectory, DxfEntityXDataSymbolDestinationEntry,
    DxfEntityXDataSymbolDestinationIssue, DxfEntityXDataSymbolDestinationIssueKind,
    DxfEntityXDataSymbolDestinationState, DxfError, DxfMemorySource, DxfNamedSymbolTableKind,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_symbol_readiness_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let binary_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, version)?;
        let ascii_destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let binary_destination_bytes = destination_fixture(DxfRawDocumentFormat::Binary, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_source_bytes, DxfResourceProfile::Safe)?;
        let binary_source = DxfMemorySource::new(&binary_source_bytes, DxfResourceProfile::Safe)?;
        let ascii_destination =
            DxfMemorySource::new(&ascii_destination_bytes, DxfResourceProfile::Safe)?;
        let binary_destination =
            DxfMemorySource::new(&binary_destination_bytes, DxfResourceProfile::Safe)?;
        let ascii_source = open_ascii(&ascii_source)?;
        let binary_source = open_binary(&binary_source)?;
        let ascii_destination = open_ascii(&ascii_destination)?;
        let binary_destination = open_binary(&binary_destination)?;

        let directories = [
            DxfRawDocumentView::from(&ascii_source).entity_xdata_symbol_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&ascii_source).entity_xdata_symbol_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_symbol_destination_directory(
                DxfRawDocumentView::from(&ascii_destination),
                &token(),
            )?,
            DxfRawDocumentView::from(&binary_source).entity_xdata_symbol_destination_directory(
                DxfRawDocumentView::from(&binary_destination),
                &token(),
            )?,
        ];
        for directory in &directories {
            assert_directory(directory)?;
        }
    }

    assert_cross_dialect_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
    )?;
    assert_cross_dialect_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
    )?;
    Ok(())
}

#[test]
fn malformed_appid_and_layer_destinations_accumulate_exact_issues() -> Result<(), Box<dyn Error>> {
    let source_bytes =
        malformed_source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let destination_bytes = malformed_destination_fixture(format, DxfAcadVersion::Ac1032)?;
        let destination_storage =
            DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => {
                let destination = open_ascii(&destination_storage)?;
                source.entity_xdata_symbol_destination_directory(
                    DxfRawDocumentView::from(&destination),
                    &token(),
                )?
            }
            DxfRawDocumentFormat::Binary => {
                let destination = open_binary(&destination_storage)?;
                source.entity_xdata_symbol_destination_directory(
                    DxfRawDocumentView::from(&destination),
                    &token(),
                )?
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_eq!(directory.entries().len(), 1);
        assert_eq!(
            directory.entries()[0].state(),
            DxfEntityXDataSymbolDestinationState::Unavailable { issue_count: 2 }
        );
        let issues = directory.issues_for_entry(directory.entries()[0])?;
        assert_eq!(issues.len(), 2);
        assert!(matches!(
            issues[0].kind(),
            DxfEntityXDataSymbolDestinationIssueKind::AppId(
                DxfEntityXDataAppIdDestinationState::DestinationMissing
            )
        ));
        assert!(matches!(
            issues[1].kind(),
            DxfEntityXDataSymbolDestinationIssueKind::Layer {
                state: DxfEntityXDataLayerDestinationState::DestinationMissing,
                ..
            }
        ));
    }
    Ok(())
}

#[test]
fn readiness_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataSymbolDestinationDirectory>();
    assert_copy::<DxfEntityXDataSymbolDestinationEntry>();
    assert_copy::<DxfEntityXDataSymbolDestinationIssue>();
    assert_copy::<DxfEntityXDataSymbolDestinationIssueKind>();
    assert_copy::<DxfEntityXDataSymbolDestinationState>();
    let entry_size = size_of::<DxfEntityXDataSymbolDestinationEntry>();
    let issue_size = size_of::<DxfEntityXDataSymbolDestinationIssue>();
    assert!(
        entry_size <= 128,
        "symbol destination entry is {entry_size} bytes"
    );
    assert!(
        issue_size <= 128,
        "symbol destination issue is {issue_size} bytes"
    );

    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1032)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        source.entity_xdata_symbol_destination_directory(
            DxfRawDocumentView::from(&destination),
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));

    let directory = source.entity_xdata_symbol_destination_directory(
        DxfRawDocumentView::from(&destination),
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let applications = directory
        .appid_destination_directory()
        .source_resolution_directory()
        .xdata_directory()
        .applications();
    assert_eq!(
        directory
            .entries_for_entity(applications[0].entity())?
            .len(),
        4
    );
    assert_eq!(
        directory
            .entries_for_entity(applications[4].entity())?
            .len(),
        3
    );
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_APP_READY"));
    assert!(!debug.contains("SECRET_LAYER_A"));
    assert!(!debug.contains("SECRET_SYMBOL_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_SYMBOL"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let other_destination_directory = source.entity_xdata_symbol_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        &token(),
    )?;
    let foreign_destination_entry = other_destination_directory.entries()[0];
    assert_eq!(
        directory.application_for_entry(foreign_destination_entry),
        None
    );
    assert_eq!(
        directory.destination_appid_target_for_entry(foreign_destination_entry),
        None
    );
    assert!(
        directory
            .issues_for_entry(foreign_destination_entry)
            .is_err()
    );

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_symbol_destination_directory(
            DxfRawDocumentView::from(&destination),
            &token(),
        )?;
    let foreign_application = other_source_directory
        .appid_destination_directory()
        .source_resolution_directory()
        .xdata_directory()
        .applications()[0];
    assert!(matches!(
        directory.entry_for_application(foreign_application),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(matches!(
        directory.entries_for_entity(foreign_application.entity()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn assert_cross_dialect_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(source_format, source_version)?;
    let destination_bytes = destination_fixture(destination_format, destination_version)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    match (source_format, destination_format) {
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_directory(&source.entity_xdata_symbol_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(&source.entity_xdata_symbol_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?)?;
        }
        _ => return Err(io::Error::other("cross-dialect test format pair").into()),
    }
    Ok(())
}

fn assert_directory(
    directory: &DxfEntityXDataSymbolDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    let expected_issue_counts = [0_u32, 0, 1, 1, 1, 1, 3];
    let expected_layer_counts = [2_usize, 0, 0, 0, 1, 1, 2];
    assert_eq!(directory.entries().len(), expected_issue_counts.len());
    assert_eq!(directory.issues().len(), 7);
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.application_ordinal(), ordinal as u64);
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(entry.destination_id(), directory.destination_id());
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let application = directory
            .application_for_entry(entry)
            .ok_or(io::Error::other("application"))?;
        assert_eq!(application.ordinal(), ordinal as u64);
        assert_eq!(
            directory
                .entry_for_application(application)
                .map_err(|error| io::Error::other(format!("{error:?}")))?,
            entry
        );
        let layers = directory
            .layer_entries_for_entry(entry)
            .map_err(|error| io::Error::other(format!("{error:?}")))?;
        assert_eq!(layers.len(), expected_layer_counts[ordinal]);
        let issues = directory
            .issues_for_entry(entry)
            .map_err(|error| io::Error::other(format!("{error:?}")))?;
        assert_eq!(issues.len(), expected_issue_counts[ordinal] as usize);
        assert!(
            issues
                .iter()
                .all(|issue| issue.application_ordinal() == ordinal as u64)
        );
        if expected_issue_counts[ordinal] == 0 {
            assert_eq!(
                entry.state(),
                DxfEntityXDataSymbolDestinationState::Ready {
                    destination_layer_occurrence_count: u32::try_from(layers.len())
                        .map_err(|_| io::Error::other("layer count"))?,
                }
            );
            let target = directory
                .destination_appid_target_for_entry(entry)
                .ok_or(io::Error::other("destination APPID"))?;
            assert_eq!(target.kind(), DxfNamedSymbolTableKind::AppId);
        } else {
            assert_eq!(
                entry.state(),
                DxfEntityXDataSymbolDestinationState::Unavailable {
                    issue_count: expected_issue_counts[ordinal],
                }
            );
            assert_eq!(directory.destination_appid_target_for_entry(entry), None);
        }
    }

    assert!(matches!(
        directory.issues_for_entry(directory.entries()[2])?[0].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::AppId(
            DxfEntityXDataAppIdDestinationState::DestinationMissing
        )
    ));
    assert!(matches!(
        directory.issues_for_entry(directory.entries()[3])?[0].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::AppId(
            DxfEntityXDataAppIdDestinationState::SourceMissing
        )
    ));
    assert!(matches!(
        directory.issues_for_entry(directory.entries()[4])?[0].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::Layer {
            state: DxfEntityXDataLayerDestinationState::DestinationMissing,
            ..
        }
    ));
    assert!(matches!(
        directory.issues_for_entry(directory.entries()[5])?[0].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::Layer {
            state: DxfEntityXDataLayerDestinationState::SourceAmbiguous { target_count: 2 },
            ..
        }
    ));
    let final_issues = directory.issues_for_entry(directory.entries()[6])?;
    assert!(matches!(
        final_issues[0].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::AppId(
            DxfEntityXDataAppIdDestinationState::DestinationAmbiguous { target_count: 2 }
        )
    ));
    assert!(matches!(
        final_issues[1].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::Layer {
            state: DxfEntityXDataLayerDestinationState::DestinationMissing,
            ..
        }
    ));
    assert!(matches!(
        final_issues[2].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::Layer {
            state: DxfEntityXDataLayerDestinationState::DestinationAmbiguous { target_count: 2 },
            ..
        }
    ));
    Ok(())
}

#[derive(Clone, Copy)]
struct Group<'a>(i16, &'a [u8]);

fn source_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"SECRET_APP_READY"),
            Group(0, b"APPID"),
            Group(2, b"APP_NO_LAYER"),
            Group(0, b"APPID"),
            Group(2, b"APPID_DEST_MISSING"),
            Group(0, b"APPID"),
            Group(2, b"APP_LAYER_CASE"),
            Group(0, b"APPID"),
            Group(2, b"APP_LAYER_SOURCE_DUP"),
            Group(0, b"APPID"),
            Group(2, b"APP_MULTI_ISSUE"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"LAYER"),
            Group(2, b"SECRET_LAYER_A"),
            Group(0, b"LAYER"),
            Group(2, b"LAYER_B"),
            Group(0, b"LAYER"),
            Group(2, b"CaseLayer"),
            Group(0, b"LAYER"),
            Group(2, b"SOURCE_DUP"),
            Group(0, b"LAYER"),
            Group(2, b"SOURCE_DUP"),
            Group(0, b"LAYER"),
            Group(2, b"DEST_MISSING"),
            Group(0, b"LAYER"),
            Group(2, b"DEST_DUP"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(1001, b"SECRET_APP_READY"),
            Group(1003, b"SECRET_LAYER_A"),
            Group(1003, b"LAYER_B"),
            Group(1000, b"SECRET_SYMBOL_PAYLOAD"),
            Group(1001, b"APP_NO_LAYER"),
            Group(1000, b"A"),
            Group(1001, b"APPID_DEST_MISSING"),
            Group(1000, b"B"),
            Group(1001, b"APPID_SOURCE_MISSING"),
            Group(1000, b"C"),
            Group(0, b"LINE"),
            Group(1001, b"APP_LAYER_CASE"),
            Group(1003, b"CaseLayer"),
            Group(1001, b"APP_LAYER_SOURCE_DUP"),
            Group(1003, b"SOURCE_DUP"),
            Group(1001, b"APP_MULTI_ISSUE"),
            Group(1003, b"DEST_MISSING"),
            Group(1003, b"DEST_DUP"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"SECRET_APP_READY"),
            Group(1000, b"SECRET_DESTINATION_SYMBOL"),
            Group(0, b"APPID"),
            Group(2, b"APP_NO_LAYER"),
            Group(0, b"APPID"),
            Group(2, b"APPID_SOURCE_MISSING"),
            Group(0, b"APPID"),
            Group(2, b"APP_LAYER_CASE"),
            Group(0, b"APPID"),
            Group(2, b"APP_LAYER_SOURCE_DUP"),
            Group(0, b"APPID"),
            Group(2, b"APP_MULTI_ISSUE"),
            Group(0, b"APPID"),
            Group(2, b"APP_MULTI_ISSUE"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"LAYER"),
            Group(2, b"SECRET_LAYER_A"),
            Group(0, b"LAYER"),
            Group(2, b"LAYER_B"),
            Group(0, b"LAYER"),
            Group(2, b"caselayer"),
            Group(0, b"LAYER"),
            Group(2, b"SOURCE_DUP"),
            Group(0, b"LAYER"),
            Group(2, b"DEST_DUP"),
            Group(0, b"LAYER"),
            Group(2, b"DEST_DUP"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn malformed_source_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"APP_MALFORMED"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"LAYER"),
            Group(2, b"LAYER_MALFORMED"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(1001, b"APP_MALFORMED"),
            Group(1003, b"LAYER_MALFORMED"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn malformed_destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let groups = document_groups(
        version,
        &[
            Group(0, b"SECTION"),
            Group(2, b"TABLES"),
            Group(0, b"TABLE"),
            Group(2, b"APPID"),
            Group(0, b"APPID"),
            Group(2, b"APP_MALFORMED"),
            Group(2, b"DECOY"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"LAYER"),
            Group(2, b"LAYER_MALFORMED"),
            Group(0, b"ENDSEC"),
        ],
    );
    encode(format, version, &groups)
}

fn document_groups<'a>(version: DxfAcadVersion, body: &[Group<'a>]) -> Vec<Group<'a>> {
    let mut groups = vec![
        Group(0, b"SECTION"),
        Group(2, b"HEADER"),
        Group(9, b"$ACADVER"),
        Group(1, version.code().as_bytes()),
        Group(0, b"ENDSEC"),
    ];
    groups.extend_from_slice(body);
    groups.push(Group(0, b"EOF"));
    groups
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[Group<'_>],
) -> io::Result<Vec<u8>> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[Group<'_>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for Group(code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(value);
        bytes.push(b'\n');
    }
    bytes
}

fn binary_groups(version: DxfAcadVersion, groups: &[Group<'_>]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for Group(code, value) in groups {
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

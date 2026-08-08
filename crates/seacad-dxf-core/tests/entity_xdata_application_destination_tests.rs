use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityXDataAppIdDestinationState,
    DxfEntityXDataApplicationDestinationDirectory, DxfEntityXDataApplicationDestinationEntry,
    DxfEntityXDataApplicationDestinationState, DxfEntityXDataLayerDestinationState,
    DxfEntityXDataStructureIssueKind, DxfEntityXDataSymbolDestinationIssueKind, DxfError,
    DxfMemorySource, DxfNamedSymbolTableKind, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_and_format_pair_has_application_readiness_parity()
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
            DxfRawDocumentView::from(&ascii_source)
                .entity_xdata_application_destination_directory(
                    DxfRawDocumentView::from(&ascii_destination),
                    &token(),
                )?,
            DxfRawDocumentView::from(&ascii_source)
                .entity_xdata_application_destination_directory(
                    DxfRawDocumentView::from(&binary_destination),
                    &token(),
                )?,
            DxfRawDocumentView::from(&binary_source)
                .entity_xdata_application_destination_directory(
                    DxfRawDocumentView::from(&ascii_destination),
                    &token(),
                )?,
            DxfRawDocumentView::from(&binary_source)
                .entity_xdata_application_destination_directory(
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
fn symbol_and_structure_blockers_remain_independently_owned() -> Result<(), Box<dyn Error>> {
    for (source_format, destination_format) in [
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary),
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii),
    ] {
        let source_bytes = source_fixture(source_format, DxfAcadVersion::Ac1032)?;
        let destination_bytes = destination_fixture(destination_format, DxfAcadVersion::Ac1032)?;
        let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
        let destination_storage =
            DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
        let directory = match source_format {
            DxfRawDocumentFormat::Ascii => {
                let source = open_ascii(&source_storage)?;
                let destination = open_binary(&destination_storage)?;
                source.entity_xdata_application_destination_directory(
                    DxfRawDocumentView::from(&destination),
                    &token(),
                )?
            }
            DxfRawDocumentFormat::Binary => {
                let source = open_binary(&source_storage)?;
                let destination = open_ascii(&destination_storage)?;
                source.entity_xdata_application_destination_directory(
                    DxfRawDocumentView::from(&destination),
                    &token(),
                )?
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        assert_blockers(&directory)?;
    }
    Ok(())
}

#[test]
fn readiness_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataApplicationDestinationDirectory>();
    assert_copy::<DxfEntityXDataApplicationDestinationEntry>();
    assert_copy::<DxfEntityXDataApplicationDestinationState>();
    let entry_size = size_of::<DxfEntityXDataApplicationDestinationEntry>();
    assert!(
        entry_size <= 128,
        "application destination entry is {entry_size} bytes"
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
        source.entity_xdata_application_destination_directory(
            DxfRawDocumentView::from(&destination),
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));

    let directory = source.entity_xdata_application_destination_directory(
        DxfRawDocumentView::from(&destination),
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let applications = directory
        .symbol_destination_directory()
        .appid_destination_directory()
        .source_resolution_directory()
        .xdata_directory()
        .applications();
    assert_eq!(
        directory
            .entries_for_entity(applications[0].entity())?
            .len(),
        3
    );
    assert_eq!(
        directory
            .entries_for_entity(applications[3].entity())?
            .len(),
        2
    );
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_APP_READY"));
    assert!(!debug.contains("SECRET_LAYER_READY"));
    assert!(!debug.contains("SECRET_STRUCTURE_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let other_destination_directory = source.entity_xdata_application_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        &token(),
    )?;
    let foreign_destination_entry = other_destination_directory.entries()[0];
    assert_eq!(
        directory.application_for_entry(foreign_destination_entry),
        None
    );
    assert_eq!(
        directory.symbol_entry_for_entry(foreign_destination_entry),
        None
    );
    assert_eq!(
        directory.structure_entry_for_entry(foreign_destination_entry),
        None
    );
    assert_eq!(
        directory.destination_appid_target_for_entry(foreign_destination_entry),
        None
    );
    assert!(
        directory
            .symbol_issues_for_entry(foreign_destination_entry)
            .is_err()
    );

    let other_source_bytes = source_fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1027)?;
    let other_source_storage = DxfMemorySource::new(&other_source_bytes, DxfResourceProfile::Safe)?;
    let other_source = open_binary(&other_source_storage)?;
    let other_source_directory = DxfRawDocumentView::from(&other_source)
        .entity_xdata_application_destination_directory(
            DxfRawDocumentView::from(&destination),
            &token(),
        )?;
    let foreign_application = other_source_directory
        .symbol_destination_directory()
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
            assert_directory(&source.entity_xdata_application_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(&source.entity_xdata_application_destination_directory(
                DxfRawDocumentView::from(&destination),
                &token(),
            )?)?;
        }
        _ => return Err(io::Error::other("cross-dialect test format pair").into()),
    }
    Ok(())
}

fn assert_directory(
    directory: &DxfEntityXDataApplicationDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    let expected = [
        DxfEntityXDataApplicationDestinationState::Ready {
            destination_layer_occurrence_count: 1,
        },
        DxfEntityXDataApplicationDestinationState::Unavailable {
            symbol_issue_count: 1,
            structure_issue_count: 0,
        },
        DxfEntityXDataApplicationDestinationState::Unavailable {
            symbol_issue_count: 0,
            structure_issue_count: 1,
        },
        DxfEntityXDataApplicationDestinationState::Unavailable {
            symbol_issue_count: 2,
            structure_issue_count: 1,
        },
        DxfEntityXDataApplicationDestinationState::Ready {
            destination_layer_occurrence_count: 1,
        },
    ];
    assert_eq!(directory.entries().len(), expected.len());
    for (ordinal, (entry, expected_state)) in directory
        .entries()
        .iter()
        .copied()
        .zip(expected)
        .enumerate()
    {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.application_ordinal(), ordinal as u64);
        assert_eq!(entry.source_id(), directory.source_id());
        assert_eq!(entry.destination_id(), directory.destination_id());
        assert_eq!(entry.state(), expected_state);
        assert_eq!(directory.entry(entry.ordinal()), Some(entry));
        let application = directory
            .application_for_entry(entry)
            .ok_or(io::Error::other("application"))?;
        assert_eq!(application.ordinal(), ordinal as u64);
        assert_eq!(directory.entry_for_application(application)?, entry);
        let symbol = directory
            .symbol_entry_for_entry(entry)
            .ok_or(io::Error::other("symbol entry"))?;
        let structure = directory
            .structure_entry_for_entry(entry)
            .ok_or(io::Error::other("structure entry"))?;
        assert_eq!(symbol.application_ordinal(), ordinal as u64);
        assert_eq!(structure.application(), application);
        let symbol_issues = directory.symbol_issues_for_entry(entry)?;
        let structure_issues = directory.structure_issues_for_entry(entry)?;
        match expected_state {
            DxfEntityXDataApplicationDestinationState::Ready { .. } => {
                assert!(symbol_issues.is_empty());
                assert!(structure_issues.is_empty());
                let target = directory
                    .destination_appid_target_for_entry(entry)
                    .ok_or(io::Error::other("destination APPID"))?;
                assert_eq!(target.kind(), DxfNamedSymbolTableKind::AppId);
            }
            DxfEntityXDataApplicationDestinationState::Unavailable {
                symbol_issue_count,
                structure_issue_count,
            } => {
                assert_eq!(symbol_issues.len(), symbol_issue_count as usize);
                assert_eq!(structure_issues.len(), structure_issue_count as usize);
                assert_eq!(directory.destination_appid_target_for_entry(entry), None);
            }
            _ => return Err(io::Error::other("application destination state").into()),
        }
    }
    assert_blockers(directory)
}

fn assert_blockers(
    directory: &DxfEntityXDataApplicationDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    let symbol_only = directory.symbol_issues_for_entry(directory.entries()[1])?;
    assert!(matches!(
        symbol_only[0].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::AppId(
            DxfEntityXDataAppIdDestinationState::DestinationMissing
        )
    ));
    assert!(
        directory
            .structure_issues_for_entry(directory.entries()[1])?
            .is_empty()
    );

    assert!(
        directory
            .symbol_issues_for_entry(directory.entries()[2])?
            .is_empty()
    );
    let structure_only = directory.structure_issues_for_entry(directory.entries()[2])?;
    assert!(matches!(
        structure_only[0].kind(),
        DxfEntityXDataStructureIssueKind::UnclosedLists { open_count: 1 }
    ));

    let both_symbols = directory.symbol_issues_for_entry(directory.entries()[3])?;
    assert_eq!(both_symbols.len(), 2);
    assert!(matches!(
        both_symbols[0].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::AppId(
            DxfEntityXDataAppIdDestinationState::DestinationMissing
        )
    ));
    assert!(matches!(
        both_symbols[1].kind(),
        DxfEntityXDataSymbolDestinationIssueKind::Layer {
            state: DxfEntityXDataLayerDestinationState::DestinationMissing,
            ..
        }
    ));
    let both_structure = directory.structure_issues_for_entry(directory.entries()[3])?;
    assert!(matches!(
        both_structure[0].kind(),
        DxfEntityXDataStructureIssueKind::UnclosedLists { open_count: 1 }
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
            Group(2, b"APP_SYMBOL_BAD"),
            Group(0, b"APPID"),
            Group(2, b"APP_STRUCTURE_BAD"),
            Group(0, b"APPID"),
            Group(2, b"APP_BOTH_BAD"),
            Group(0, b"APPID"),
            Group(2, b"APP_READY_LAYER"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"LAYER"),
            Group(2, b"SECRET_LAYER_READY"),
            Group(0, b"LAYER"),
            Group(2, b"LAYER_BOTH_BAD"),
            Group(0, b"LAYER"),
            Group(2, b"LAYER_READY_2"),
            Group(0, b"ENDTAB"),
            Group(0, b"ENDSEC"),
            Group(0, b"SECTION"),
            Group(2, b"ENTITIES"),
            Group(0, b"POINT"),
            Group(1001, b"SECRET_APP_READY"),
            Group(1003, b"SECRET_LAYER_READY"),
            Group(1002, b"{"),
            Group(1000, b"SECRET_STRUCTURE_PAYLOAD"),
            Group(1002, b"}"),
            Group(1001, b"APP_SYMBOL_BAD"),
            Group(1000, b"A"),
            Group(1001, b"APP_STRUCTURE_BAD"),
            Group(1002, b"{"),
            Group(1000, b"B"),
            Group(0, b"LINE"),
            Group(1001, b"APP_BOTH_BAD"),
            Group(1002, b"{"),
            Group(1003, b"LAYER_BOTH_BAD"),
            Group(1001, b"APP_READY_LAYER"),
            Group(1003, b"LAYER_READY_2"),
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
            Group(1000, b"SECRET_DESTINATION_PAYLOAD"),
            Group(0, b"APPID"),
            Group(2, b"APP_STRUCTURE_BAD"),
            Group(0, b"APPID"),
            Group(2, b"APP_READY_LAYER"),
            Group(0, b"ENDTAB"),
            Group(0, b"TABLE"),
            Group(2, b"LAYER"),
            Group(0, b"LAYER"),
            Group(2, b"SECRET_LAYER_READY"),
            Group(0, b"LAYER"),
            Group(2, b"LAYER_READY_2"),
            Group(0, b"ENDTAB"),
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

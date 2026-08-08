use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataDestinationEncodeIssue, DxfEntityXDataEncodedApplicationDestinationDirectory,
    DxfEntityXDataEncodedApplicationDestinationEntry,
    DxfEntityXDataEncodedApplicationDestinationState, DxfEntityXDataEncodedDestinationDirectory,
    DxfEntityXDataEncodedDestinationEntry, DxfEntityXDataEncodedDestinationState,
    DxfEntityXDataEncodedEntityDestinationDirectory, DxfEntityXDataEncodedEntityDestinationEntry,
    DxfEntityXDataEncodedEntityDestinationState, DxfEntityXDataHandleRemap,
    DxfEntityXDataLogicalDestinationIssue, DxfEntityXDataPayloadDestinationState, DxfError,
    DxfHandle, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfTextEncodeStatus, DxfTextTranscodeIssue, NoopDxfReadObserver,
};

#[test]
fn encoded_entities_preserve_exact_payloads_for_every_format_and_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                assert_entity_format_pair(source_format, version, destination_format, version)?;
            }
        }
    }
    assert_entity_format_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
    )?;
    assert_entity_format_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
    )?;
    Ok(())
}

#[test]
fn encoded_entity_directory_is_cancellable_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataEncodedEntityDestinationDirectory>();
    assert_copy::<DxfEntityXDataEncodedEntityDestinationEntry>();
    assert_copy::<DxfEntityXDataEncodedEntityDestinationState>();
    assert!(size_of::<DxfEntityXDataEncodedEntityDestinationEntry>() <= 192);
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
        source.entity_xdata_encoded_entity_destination_directory(
            DxfRawDocumentView::from(&destination),
            transform()?,
            &mappings()?,
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = build_entity_directory(
        DxfRawDocumentView::from(&source),
        DxfRawDocumentView::from(&destination),
    )?;
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_ENCODE_PAYLOAD"));
    assert!(!debug.contains("SECRET_ORPHAN_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_storage = DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_storage)?;
    let foreign = build_entity_directory(
        DxfRawDocumentView::from(&source),
        DxfRawDocumentView::from(&other_destination),
    )?;
    assert_eq!(directory.payload_for_entry(foreign.entries()[0]), None);
    assert_eq!(
        directory.encoded_bytes_for_entry(foreign.entries()[0]),
        None
    );
    assert!(directory.entity_for_entry(foreign.entries()[0]).is_err());
    Ok(())
}

#[test]
fn encoded_entity_keeps_empty_nested_and_orphan_payloads_distinct() -> Result<(), Box<dyn Error>> {
    let source_bytes = empty_nested_fixture()?;
    let destination_bytes = empty_nested_destination_fixture()?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_encoded_entity_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(directory.entries().len(), 3);
    assert!(matches!(
        directory.entries()[0].state(),
        DxfEntityXDataEncodedEntityDestinationState::Ready {
            application_count: 0,
            member_count: 0,
            encoded_byte_count: 0,
        }
    ));
    assert!(matches!(
        directory.entries()[1].state(),
        DxfEntityXDataEncodedEntityDestinationState::Ready {
            application_count: 2,
            member_count: 7,
            ..
        }
    ));
    assert_eq!(
        directory
            .applications_for_entry(directory.entries()[1])?
            .len(),
        2
    );
    assert!(matches!(
        directory.entries()[2].state(),
        DxfEntityXDataEncodedEntityDestinationState::Unavailable {
            application_count: 0,
            unavailable_application_count: 0,
            member_count: 0,
            ..
        }
    ));
    assert_eq!(
        directory.encoded_bytes_for_entry(directory.entries()[2]),
        None
    );
    Ok(())
}

#[test]
fn encoded_applications_preserve_exact_sets_for_every_format_and_dialect()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                assert_application_format_pair(
                    source_format,
                    version,
                    destination_format,
                    version,
                )?;
            }
        }
    }
    assert_application_format_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
    )?;
    assert_application_format_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
    )?;
    Ok(())
}

#[test]
fn encoded_application_fails_closed_for_unmappable_text_and_parent_envelope()
-> Result<(), Box<dyn Error>> {
    let source_bytes = text_fixture(b"ANSI_1252", &[0x80])?;
    let destination_bytes = text_destination_fixture(b"ANSI_932")?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_ascii(&destination_storage)?;
    let directory = source.entity_xdata_encoded_application_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(directory.entries().len(), 1);
    let entry = directory.entries()[0];
    assert!(matches!(
        entry.state(),
        DxfEntityXDataEncodedApplicationDestinationState::Unavailable {
            payload_state: DxfEntityXDataPayloadDestinationState::Ready { .. },
            member_count: 2,
            unavailable_member_count: 1,
            first_unavailable_member_ordinal: Some(1),
        }
    ));
    assert_eq!(directory.encoded_bytes_for_entry(entry), None);
    assert!(matches!(
        directory.encoded_entries_for_entry(entry)?[1].state(),
        DxfEntityXDataEncodedDestinationState::EncodingUnavailable(
            DxfEntityXDataDestinationEncodeIssue::TextTranscode(
                DxfTextTranscodeIssue::DestinationEncode {
                    status: DxfTextEncodeStatus::Unmappable { .. }
                }
            )
        )
    ));

    let matching_destination_bytes = text_destination_fixture(b"ANSI_1252")?;
    let matching_storage =
        DxfMemorySource::new(&matching_destination_bytes, DxfResourceProfile::Safe)?;
    let matching_destination = open_ascii(&matching_storage)?;
    let matching = source.entity_xdata_encoded_application_destination_directory(
        DxfRawDocumentView::from(&matching_destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert!(matches!(
        matching.entries()[0].state(),
        DxfEntityXDataEncodedApplicationDestinationState::Ready {
            member_count: 2,
            ..
        }
    ));
    assert!(
        matching
            .encoded_bytes_for_entry(matching.entries()[0])
            .is_some()
    );
    Ok(())
}

#[test]
fn encoded_applications_keep_empty_and_nested_sets_separate_from_orphans()
-> Result<(), Box<dyn Error>> {
    let source_bytes = empty_nested_fixture()?;
    let destination_bytes = empty_nested_destination_fixture()?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_binary(&destination_storage)?;
    let directory = source.entity_xdata_encoded_application_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(directory.entries().len(), 2);
    assert!(matches!(
        directory.entries()[0].state(),
        DxfEntityXDataEncodedApplicationDestinationState::Ready {
            member_count: 1,
            ..
        }
    ));
    assert!(matches!(
        directory.entries()[1].state(),
        DxfEntityXDataEncodedApplicationDestinationState::Ready {
            member_count: 6,
            ..
        }
    ));
    let second = directory.encoded_entries_for_entry(directory.entries()[1])?;
    let codes: Vec<i16> = second
        .iter()
        .copied()
        .map(|entry| {
            directory
                .encoded_destination_directory()
                .logical_for_entry(entry)
                .and_then(|logical| {
                    directory
                        .encoded_destination_directory()
                        .logical_destination_directory()
                        .typed_for_entry(logical)
                })
                .map(|typed| typed.occurrence().group().group_code().value())
                .ok_or_else(|| io::Error::other("typed member"))
        })
        .collect::<Result<_, _>>()?;
    assert_eq!(codes, [1001, 1002, 1002, 1000, 1002, 1002]);
    assert_eq!(directory.encoded_destination_directory().entries().len(), 8);
    Ok(())
}

#[test]
fn encoded_application_directory_is_cancellable_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataEncodedApplicationDestinationDirectory>();
    assert_copy::<DxfEntityXDataEncodedApplicationDestinationEntry>();
    assert_copy::<DxfEntityXDataEncodedApplicationDestinationState>();
    let entry_size = size_of::<DxfEntityXDataEncodedApplicationDestinationEntry>();
    assert!(
        entry_size <= 192,
        "encoded application entry is {entry_size} bytes"
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
        source.entity_xdata_encoded_application_destination_directory(
            DxfRawDocumentView::from(&destination),
            transform()?,
            &mappings()?,
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = build_application_directory(
        DxfRawDocumentView::from(&source),
        DxfRawDocumentView::from(&destination),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_ENCODE_PAYLOAD"));
    assert!(!debug.contains("SECRET_ORPHAN_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let foreign = build_application_directory(
        DxfRawDocumentView::from(&source),
        DxfRawDocumentView::from(&other_destination),
    )?;
    assert_eq!(directory.application_for_entry(foreign.entries()[0]), None);
    assert_eq!(directory.payload_for_entry(foreign.entries()[0]), None);
    assert!(
        directory
            .encoded_entries_for_entry(foreign.entries()[0])
            .is_err()
    );
    assert_eq!(
        directory.encoded_bytes_for_entry(foreign.entries()[0]),
        None
    );
    Ok(())
}

#[test]
fn every_supported_version_and_format_pair_has_canonical_encoded_groups()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                assert_format_pair(source_format, version, destination_format, version)?;
            }
        }
    }
    assert_format_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
    )?;
    assert_format_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
    )?;
    Ok(())
}

#[test]
fn unmappable_non_ascii_text_fails_closed_without_destination_bytes() -> Result<(), Box<dyn Error>>
{
    let source_bytes = text_fixture(b"ANSI_1252", &[0x80])?;
    let destination_bytes = text_destination_fixture(b"ANSI_932")?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_ascii(&source_storage)?;
    let destination = open_ascii(&destination_storage)?;
    let directory = source.entity_xdata_encoded_destination_directory(
        DxfRawDocumentView::from(&destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(directory.entries().len(), 2);
    assert!(matches!(
        directory.entries()[0].state(),
        DxfEntityXDataEncodedDestinationState::Ready { .. }
    ));
    assert!(matches!(
        directory.entries()[1].state(),
        DxfEntityXDataEncodedDestinationState::EncodingUnavailable(
            DxfEntityXDataDestinationEncodeIssue::TextTranscode(
                DxfTextTranscodeIssue::DestinationEncode {
                    status: DxfTextEncodeStatus::Unmappable { .. }
                }
            )
        )
    ));
    assert_eq!(
        directory.encoded_bytes_for_entry(directory.entries()[1]),
        None
    );

    let matching_destination_bytes = text_destination_fixture(b"ANSI_1252")?;
    let matching_storage =
        DxfMemorySource::new(&matching_destination_bytes, DxfResourceProfile::Safe)?;
    let matching_destination = open_ascii(&matching_storage)?;
    let matching = source.entity_xdata_encoded_destination_directory(
        DxfRawDocumentView::from(&matching_destination),
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert!(matches!(
        matching.entries()[1].state(),
        DxfEntityXDataEncodedDestinationState::Ready { .. }
    ));
    Ok(())
}

#[test]
fn encoded_directory_is_cancellable_dual_source_bound_bounded_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataEncodedDestinationDirectory>();
    assert_copy::<DxfEntityXDataDestinationEncodeIssue>();
    assert_copy::<DxfEntityXDataEncodedDestinationEntry>();
    assert_copy::<DxfEntityXDataEncodedDestinationState>();
    let entry_size = size_of::<DxfEntityXDataEncodedDestinationEntry>();
    assert!(
        entry_size <= 160,
        "encoded destination entry is {entry_size} bytes"
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
        source.entity_xdata_encoded_destination_directory(
            DxfRawDocumentView::from(&destination),
            transform()?,
            &[],
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    let directory = source.entity_xdata_encoded_destination_directory(
        DxfRawDocumentView::from(&destination),
        transform()?,
        &mappings()?,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(directory.source_id(), source.source_id());
    assert_eq!(directory.destination_id(), destination.source_id());
    assert_eq!(directory.entry(u64::MAX), None);
    let debug = format!("{directory:?}");
    assert!(!debug.contains("SECRET_ENCODE_PAYLOAD"));
    assert!(!debug.contains("SECRET_ORPHAN_PAYLOAD"));
    assert!(!debug.contains("SECRET_DESTINATION_PAYLOAD"));
    for entry in directory.entries().iter().copied() {
        assert!(directory.logical_for_entry(entry).is_some());
        assert_eq!(
            directory.encoded_bytes_for_entry(entry).is_some(),
            matches!(
                entry.state(),
                DxfEntityXDataEncodedDestinationState::Ready { .. }
            )
        );
    }

    let other_destination_bytes =
        destination_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_destination_storage =
        DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let other_destination = open_ascii(&other_destination_storage)?;
    let foreign = source.entity_xdata_encoded_destination_directory(
        DxfRawDocumentView::from(&other_destination),
        transform()?,
        &mappings()?,
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(directory.logical_for_entry(foreign.entries()[0]), None);
    assert_eq!(
        directory.encoded_bytes_for_entry(foreign.entries()[0]),
        None
    );
    Ok(())
}

fn assert_format_pair(
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
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Ascii) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(
                &build_directory(
                    DxfRawDocumentView::from(&source),
                    DxfRawDocumentView::from(&destination),
                )?,
                destination_version,
            )?;
        }
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_directory(
                &build_directory(
                    DxfRawDocumentView::from(&source),
                    DxfRawDocumentView::from(&destination),
                )?,
                destination_version,
            )?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_directory(
                &build_directory(
                    DxfRawDocumentView::from(&source),
                    DxfRawDocumentView::from(&destination),
                )?,
                destination_version,
            )?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Binary) => {
            let source = open_binary(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_directory(
                &build_directory(
                    DxfRawDocumentView::from(&source),
                    DxfRawDocumentView::from(&destination),
                )?,
                destination_version,
            )?;
        }
        _ => return Err(io::Error::other("format pair").into()),
    }
    Ok(())
}

fn assert_application_format_pair(
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
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Ascii) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_application_directory(&build_application_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_application_directory(&build_application_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_application_directory(&build_application_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Binary) => {
            let source = open_binary(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_application_directory(&build_application_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        _ => return Err(io::Error::other("format pair").into()),
    }
    Ok(())
}

fn assert_entity_format_pair(
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
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Ascii) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_entity_directory(&build_entity_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary) => {
            let source = open_ascii(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_entity_directory(&build_entity_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Ascii) => {
            let source = open_binary(&source_storage)?;
            let destination = open_ascii(&destination_storage)?;
            assert_entity_directory(&build_entity_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        (DxfRawDocumentFormat::Binary, DxfRawDocumentFormat::Binary) => {
            let source = open_binary(&source_storage)?;
            let destination = open_binary(&destination_storage)?;
            assert_entity_directory(&build_entity_directory(
                DxfRawDocumentView::from(&source),
                DxfRawDocumentView::from(&destination),
            )?)?;
        }
        _ => return Err(io::Error::other("format pair").into()),
    }
    Ok(())
}

fn build_entity_directory(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataEncodedEntityDestinationDirectory, Box<dyn Error>> {
    Ok(source.entity_xdata_encoded_entity_destination_directory(
        destination,
        transform()?,
        &mappings()?,
        DxfResourceProfile::Safe,
        &token(),
    )?)
}

fn assert_entity_directory(
    directory: &DxfEntityXDataEncodedEntityDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 7);
    for entry in &directory.entries()[..4] {
        assert!(matches!(
            entry.state(),
            DxfEntityXDataEncodedEntityDestinationState::Ready {
                application_count: 0,
                member_count: 0,
                encoded_byte_count: 0,
            }
        ));
        assert_eq!(directory.encoded_bytes_for_entry(*entry), Some(&[][..]));
    }
    let ready = directory.entries()[4];
    let unavailable = directory.entries()[5];
    let orphan = directory.entries()[6];
    assert!(matches!(
        ready.state(),
        DxfEntityXDataEncodedEntityDestinationState::Ready {
            application_count: 1,
            member_count: 13,
            encoded_byte_count,
        } if encoded_byte_count == directory.encoded_bytes_for_entry(ready)
            .ok_or(io::Error::other("entity bytes"))?.len() as u64
    ));
    assert!(matches!(
        unavailable.state(),
        DxfEntityXDataEncodedEntityDestinationState::Unavailable {
            application_count: 1,
            unavailable_application_count: 1,
            first_unavailable_application_ordinal: Some(0),
            member_count: 2,
            unavailable_member_count: 1,
            ..
        }
    ));
    assert!(matches!(
        orphan.state(),
        DxfEntityXDataEncodedEntityDestinationState::Unavailable {
            application_count: 0,
            unavailable_application_count: 0,
            first_unavailable_application_ordinal: None,
            member_count: 0,
            unavailable_member_count: 0,
            ..
        }
    ));
    for entry in directory.entries().iter().copied() {
        let entity = directory.entity_for_entry(entry)?;
        assert_eq!(directory.entry_for_entity(entity)?, Some(entry));
        assert!(directory.payload_for_entry(entry).is_some());
    }
    assert_eq!(directory.encoded_bytes_for_entry(unavailable), None);
    assert_eq!(directory.encoded_bytes_for_entry(orphan), None);
    Ok(())
}

fn build_application_directory(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataEncodedApplicationDestinationDirectory, Box<dyn Error>> {
    Ok(
        source.entity_xdata_encoded_application_destination_directory(
            destination,
            transform()?,
            &mappings()?,
            DxfResourceProfile::Safe,
            &token(),
        )?,
    )
}

fn assert_application_directory(
    directory: &DxfEntityXDataEncodedApplicationDestinationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 2);
    let ready = directory.entries()[0];
    let unavailable = directory.entries()[1];
    let ready_bytes = directory
        .encoded_bytes_for_entry(ready)
        .ok_or(io::Error::other("ready application bytes"))?;
    assert!(matches!(
        ready.state(),
        DxfEntityXDataEncodedApplicationDestinationState::Ready {
            member_count: 13,
            encoded_byte_count,
        } if encoded_byte_count == ready_bytes.len() as u64
    ));
    assert!(matches!(
        unavailable.state(),
        DxfEntityXDataEncodedApplicationDestinationState::Unavailable {
            payload_state: DxfEntityXDataPayloadDestinationState::Unavailable { .. },
            member_count: 2,
            unavailable_member_count: 1,
            first_unavailable_member_ordinal: Some(1),
        }
    ));
    assert_eq!(directory.encoded_bytes_for_entry(unavailable), None);
    for entry in [ready, unavailable] {
        let application = directory
            .application_for_entry(entry)
            .ok_or(io::Error::other("application"))?;
        assert_eq!(directory.entry_for_application(application)?, entry);
        assert_eq!(
            application.occurrence_range().len(),
            directory.encoded_entries_for_entry(entry)?.len() as u64
        );
        assert_eq!(
            directory.entries_for_entity(application.entity())?,
            std::slice::from_ref(&entry)
        );
        assert!(directory.payload_for_entry(entry).is_some());
    }
    let members = directory.encoded_entries_for_entry(ready)?;
    for pair in members.windows(2) {
        assert!(pair[0].ordinal() < pair[1].ordinal());
    }
    Ok(())
}

fn build_directory(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataEncodedDestinationDirectory, Box<dyn Error>> {
    Ok(source.entity_xdata_encoded_destination_directory(
        destination,
        transform()?,
        &mappings()?,
        DxfResourceProfile::Safe,
        &token(),
    )?)
}

fn assert_directory(
    directory: &DxfEntityXDataEncodedDestinationDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 16);
    for (ordinal, entry) in directory.entries().iter().copied().enumerate() {
        assert_eq!(entry.ordinal(), ordinal as u64);
        assert_eq!(entry.logical_ordinal(), ordinal as u64);
        if ordinal == 14 {
            assert!(matches!(
                entry.state(),
                DxfEntityXDataEncodedDestinationState::LogicalUnavailable(
                    DxfEntityXDataLogicalDestinationIssue::Handle(_)
                )
            ));
        } else if ordinal == 15 {
            assert_eq!(
                entry.state(),
                DxfEntityXDataEncodedDestinationState::LogicalUnavailable(
                    DxfEntityXDataLogicalDestinationIssue::Orphan
                )
            );
        } else {
            let bytes = directory
                .encoded_bytes_for_entry(entry)
                .ok_or(io::Error::other("encoded bytes"))?;
            assert!(matches!(
                entry.state(),
                DxfEntityXDataEncodedDestinationState::Ready { encoded_byte_count }
                    if u64::from(encoded_byte_count) == bytes.len() as u64
            ));
            assert_group_framing(bytes, directory.destination_format(), version)?;
        }
    }
    let chunk = directory
        .encoded_bytes_for_entry(directory.entries()[4])
        .ok_or(io::Error::other("chunk"))?;
    let chunk_payload =
        binary_or_ascii_payload(chunk, directory.destination_format(), version, 1004)?;
    match directory.destination_format() {
        DxfRawDocumentFormat::Ascii => assert_eq!(chunk_payload, b"ABCD01"),
        DxfRawDocumentFormat::Binary => assert_eq!(chunk_payload, &[3, 0xAB, 0xCD, 0x01]),
        _ => return Err(io::Error::other("format").into()),
    }
    let transformed = directory
        .encoded_bytes_for_entry(directory.entries()[6])
        .ok_or(io::Error::other("transformed"))?;
    let payload =
        binary_or_ascii_payload(transformed, directory.destination_format(), version, 1011)?;
    match directory.destination_format() {
        DxfRawDocumentFormat::Ascii => assert_eq!(payload, b"11"),
        DxfRawDocumentFormat::Binary => {
            let bits = payload.get(..8).ok_or(io::Error::other("double bytes"))?;
            let bits: [u8; 8] = bits
                .try_into()
                .map_err(|_| io::Error::other("double array"))?;
            assert_eq!(f64::from_le_bytes(bits), 11.0);
        }
        _ => return Err(io::Error::other("format").into()),
    }
    Ok(())
}

fn assert_group_framing(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<(), io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => {
            if !bytes.ends_with(b"\n") || !bytes.contains(&b'\n') {
                return Err(io::Error::other("ASCII framing"));
            }
        }
        DxfRawDocumentFormat::Binary => {
            if version == DxfAcadVersion::Ac1009 && bytes.first() != Some(&u8::MAX) {
                return Err(io::Error::other("AC1009 XDATA escape"));
            }
            if version != DxfAcadVersion::Ac1009 && bytes.len() < 2 {
                return Err(io::Error::other("binary group code"));
            }
        }
        _ => return Err(io::Error::other("format")),
    }
    Ok(())
}

fn binary_or_ascii_payload(
    bytes: &[u8],
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code: i16,
) -> Result<&[u8], io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => {
            let prefix = format!("{code}\n");
            bytes
                .strip_prefix(prefix.as_bytes())
                .and_then(|bytes| bytes.strip_suffix(b"\n"))
                .ok_or_else(|| io::Error::other("ASCII payload"))
        }
        DxfRawDocumentFormat::Binary => {
            let prefix = if version == DxfAcadVersion::Ac1009 {
                3
            } else {
                2
            };
            bytes
                .get(prefix..)
                .ok_or_else(|| io::Error::other("binary payload"))
        }
        _ => Err(io::Error::other("format")),
    }
}

fn transform() -> Result<DxfEntityXDataCoordinateTransform, Box<dyn Error>> {
    DxfEntityXDataCoordinateTransform::translation([
        DxfDouble::from_f64(10.0),
        DxfDouble::from_f64(20.0),
        DxfDouble::from_f64(30.0),
    ])
    .map_err(|issue| io::Error::other(format!("{issue:?}")).into())
}

fn mappings() -> Result<[DxfEntityXDataHandleRemap; 2], io::Error> {
    Ok([remap(0xA, 0x200)?, remap(0xB, 0x300)?])
}

fn remap(source: u64, target: u64) -> Result<DxfEntityXDataHandleRemap, io::Error> {
    DxfEntityXDataHandleRemap::new(DxfHandle::from_u64(source), DxfHandle::from_u64(target))
        .map_err(|issue| io::Error::other(format!("{issue:?}")))
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Chunk(&'a [u8]),
    Double(f64),
    Int16(i16),
    Int32(i32),
}

fn source_fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut groups = header(version, b"ANSI_1252");
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_READY")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"LAYER")),
        (0, Value::Text(b"LAYER")),
        (2, Value::Text(b"LAYER_READY")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (5, Value::Text(b"A")),
        (0, Value::Text(b"POINT")),
        (5, Value::Text(b"B")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (1001, Value::Text(b"APP_READY")),
        (1000, Value::Text(b"SECRET_ENCODE_PAYLOAD")),
        (1002, Value::Text(b"{")),
        (1003, Value::Text(b"LAYER_READY")),
        (1004, Value::Chunk(&[0xAB, 0xCD, 0x01])),
        (1005, Value::Text(b"A")),
        (1011, Value::Double(1.0)),
        (1021, Value::Double(2.0)),
        (1031, Value::Double(3.0)),
        (1040, Value::Double(4.5)),
        (1070, Value::Int16(-7)),
        (1071, Value::Int32(123)),
        (1002, Value::Text(b"}")),
        (0, Value::Text(b"CIRCLE")),
        (5, Value::Text(b"11")),
        (1001, Value::Text(b"APP_READY")),
        (1005, Value::Text(b"B")),
        (0, Value::Text(b"ARC")),
        (5, Value::Text(b"12")),
        (1000, Value::Text(b"SECRET_ORPHAN_PAYLOAD")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    encode(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> io::Result<Vec<u8>> {
    let mut groups = header(version, b"ANSI_1252");
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_READY")),
        (1000, Value::Text(b"SECRET_DESTINATION_PAYLOAD")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"LAYER")),
        (0, Value::Text(b"LAYER")),
        (2, Value::Text(b"LAYER_READY")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (5, Value::Text(b"200")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    encode(format, version, &groups)
}

fn text_fixture(code_page: &'static [u8], payload: &'static [u8]) -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1009;
    let mut groups = header(version, code_page);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_READY")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP_READY")),
        (1000, Value::Text(payload)),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    Ok(ascii_groups(&groups))
}

fn text_destination_fixture(code_page: &'static [u8]) -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1009;
    let mut groups = header(version, code_page);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_READY")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    Ok(ascii_groups(&groups))
}

fn empty_nested_fixture() -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1032;
    let mut groups = header(version, b"ANSI_1252");
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_EMPTY")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_NESTED")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"POINT")),
        (1001, Value::Text(b"APP_EMPTY")),
        (1001, Value::Text(b"APP_NESTED")),
        (1002, Value::Text(b"{")),
        (1002, Value::Text(b"{")),
        (1000, Value::Text(b"NESTED_SECRET")),
        (1002, Value::Text(b"}")),
        (1002, Value::Text(b"}")),
        (0, Value::Text(b"LINE")),
        (1000, Value::Text(b"ORPHAN_SECRET")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    Ok(ascii_groups(&groups))
}

fn empty_nested_destination_fixture() -> io::Result<Vec<u8>> {
    let version = DxfAcadVersion::Ac1032;
    let mut groups = header(version, b"ANSI_1252");
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"TABLE")),
        (2, Value::Text(b"APPID")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_EMPTY")),
        (0, Value::Text(b"APPID")),
        (2, Value::Text(b"APP_NESTED")),
        (0, Value::Text(b"ENDTAB")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    binary_groups(version, &groups)
}

fn header(version: DxfAcadVersion, code_page: &'static [u8]) -> Vec<(i16, Value<'static>)> {
    vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (9, Value::Text(b"$DWGCODEPAGE")),
        (3, Value::Text(code_page)),
        (0, Value::Text(b"ENDSEC")),
    ]
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> io::Result<Vec<u8>> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Chunk(value) => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                for byte in *value {
                    bytes.push(HEX[usize::from(byte >> 4)]);
                    bytes.push(HEX[usize::from(byte & 0xF)]);
                }
            }
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.push(b'\n');
    }
    bytes
}

fn binary_groups(version: DxfAcadVersion, groups: &[(i16, Value<'_>)]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        push_code(&mut bytes, version, *code)?;
        match value {
            Value::Text(value) => {
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            Value::Chunk(value) => {
                bytes.push(u8::try_from(value.len()).map_err(|_| io::Error::other("chunk"))?);
                bytes.extend_from_slice(value);
            }
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }
    }
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 && (0..=254).contains(&code) {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("code"))?);
    } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(&code) {
        bytes.push(u8::MAX);
        bytes.extend_from_slice(&code.to_le_bytes());
    } else if version == DxfAcadVersion::Ac1009 {
        return Err(io::Error::other("code"));
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

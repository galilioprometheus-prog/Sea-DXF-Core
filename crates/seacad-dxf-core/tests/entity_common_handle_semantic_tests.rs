use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityCommonHandleDirectory, DxfEntityCommonHandleSemantics,
    DxfEntityCommonReferenceIssue, DxfEntityCommonReferenceSemanticValue,
    DxfEntityCommonReferenceTargetDirectory, DxfEntityCommonReferenceTargetIssue,
    DxfEntityCommonReferenceTargetKind, DxfEntityCommonReferenceTargetSemantics,
    DxfEntityCommonReferenceValue, DxfEntityField, DxfEntityFieldSemanticIssue,
    DxfEntityFieldValue, DxfError, DxfHandle, DxfHandleParseIssue, DxfMemorySource,
    DxfRawDocumentFormat, DxfRawRecordSectionKind, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValue, DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_common_handle_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = valid_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_handles = ascii.entity_common_handle_directory(&token())?;

        let binary_bytes = valid_fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_handles = binary.entity_common_handle_directory(&token())?;

        assert_valid(&ascii_handles, version)?;
        assert_valid(&binary_handles, version)?;
        assert_eq!(signature(&ascii_handles)?, signature(&binary_handles)?);
    }
    Ok(())
}

#[test]
fn null_missing_ambiguous_and_unique_targets_remain_distinct() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let groups = negative_target_groups();
        let bytes = encode(format, DxfAcadVersion::Ac1032, &groups)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        match format {
            DxfRawDocumentFormat::Ascii => {
                let document = open_ascii(&source)?;
                assert_target_failures(&document.entity_common_handle_directory(&token())?)?;
            }
            DxfRawDocumentFormat::Binary => {
                let document = open_binary(&source)?;
                assert_target_failures(&document.entity_common_handle_directory(&token())?)?;
            }
            _ => return Err(io::Error::other("test format").into()),
        }
    }
    Ok(())
}

#[test]
fn raw_field_failures_defaults_and_absence_are_not_resolution_guesses() -> Result<(), Box<dyn Error>>
{
    let groups = field_failure_groups();
    let bytes = encode(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, &groups)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let handles = document.entity_common_handle_directory(&token())?;
    let entity = only_entity(&handles)?;

    assert!(matches!(
        reference(&handles, entity, DxfEntityField::OWNER)?,
        DxfSemanticValue::Invalid {
            issue: DxfEntityCommonReferenceIssue::Field(
                DxfEntityFieldSemanticIssue::MultipleValues {
                    occurrence_count: 2
                }
            ),
            raw: None,
            ..
        }
    ));
    assert!(matches!(
        reference(&handles, entity, DxfEntityField::PLOT_STYLE)?,
        DxfSemanticValue::Invalid {
            issue: DxfEntityCommonReferenceIssue::Field(
                DxfEntityFieldSemanticIssue::InvalidHandle(DxfHandleParseIssue::InvalidDigit {
                    offset: 0
                })
            ),
            raw: Some(_),
            ..
        }
    ));
    assert!(matches!(
        reference(&handles, entity, DxfEntityField::MATERIAL)?.value(),
        Some(DxfEntityCommonReferenceValue::ByLayer)
    ));
    assert_eq!(
        reference(&handles, entity, DxfEntityField::MATERIAL)?.state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        reference(&handles, entity, DxfEntityField::EXTENSION_DICTIONARY)?.state(),
        DxfSemanticValueState::Absent
    );
    assert!(!format!("{handles:?}").contains("SECRET"));
    Ok(())
}

#[test]
fn cancellation_source_bound_lookups_and_public_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityCommonHandleDirectory>();
    assert_copy::<seacad_dxf_core::DxfEntityCommonHandleEntry>();
    assert_copy::<DxfEntityCommonHandleSemantics>();
    let entry_size = std::mem::size_of::<seacad_dxf_core::DxfEntityCommonHandleEntry>();
    assert!(entry_size <= 640, "handle entry size: {entry_size}");

    let bytes = valid_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_common_handle_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let handles = document.entity_common_handle_directory(&token())?;
    let entity = only_entity(&handles)?;
    assert_eq!(handles.source_id(), document.source_id());
    assert_eq!(handles.source_directory().source_id(), document.source_id());
    assert_eq!(
        handles.resolution_directory().source_id(),
        document.source_id()
    );
    assert_eq!(handles.entries().len(), 5);
    assert_eq!(handles.entries_for_entity(entity)?.len(), 5);
    assert_eq!(handles.entry(u64::MAX), None);
    assert_eq!(
        handles.entry_for_field(entity, DxfEntityField::LAYER)?,
        None
    );

    let other_bytes = valid_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let other_handles = other.entity_common_handle_directory(&token())?;
    assert!(matches!(
        handles.entries_for_entity(only_entity(&other_handles)?),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn every_dialect_has_ascii_binary_reference_target_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = target_kind_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_targets = ascii.entity_common_reference_target_directory(&token())?;

        let binary_bytes = target_kind_fixture(DxfRawDocumentFormat::Binary, version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_targets = binary.entity_common_reference_target_directory(&token())?;

        assert_valid_target_kinds(&ascii_targets, version)?;
        assert_valid_target_kinds(&binary_targets, version)?;
        assert_eq!(
            target_signature(&ascii_targets)?,
            target_signature(&binary_targets)?
        );
    }
    Ok(())
}

#[test]
fn unique_wrong_markers_and_sections_are_incompatible() -> Result<(), Box<dyn Error>> {
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = encode(format, DxfAcadVersion::Ac1032, &wrong_target_kind_groups())?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => {
                open_ascii(&source)?.entity_common_reference_target_directory(&token())?
            }
            DxfRawDocumentFormat::Binary => {
                open_binary(&source)?.entity_common_reference_target_directory(&token())?
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        let entity = target_entity(&directory)?;
        for (field, expected) in [
            (
                DxfEntityField::EXTENSION_DICTIONARY,
                DxfEntityCommonReferenceTargetKind::ExtensionDictionary,
            ),
            (
                DxfEntityField::MATERIAL,
                DxfEntityCommonReferenceTargetKind::Material,
            ),
            (
                DxfEntityField::PLOT_STYLE,
                DxfEntityCommonReferenceTargetKind::PlotStyle,
            ),
        ] {
            let value = reviewed_target(&directory, entity, field)?;
            assert!(matches!(
                value.invalid_issue(),
                Some(DxfEntityCommonReferenceTargetIssue::IncompatibleTarget {
                    expected: observed,
                    target,
                }) if *observed == expected
                    && target.record().section_kind() == DxfRawRecordSectionKind::Objects
            ));
            assert!(value.raw_provenance().is_some());
        }

        let bytes = encode(
            format,
            DxfAcadVersion::Ac1032,
            &wrong_target_section_groups(),
        )?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => {
                open_ascii(&source)?.entity_common_reference_target_directory(&token())?
            }
            DxfRawDocumentFormat::Binary => {
                open_binary(&source)?.entity_common_reference_target_directory(&token())?
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        let value = reviewed_target(
            &directory,
            target_entity(&directory)?,
            DxfEntityField::EXTENSION_DICTIONARY,
        )?;
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfEntityCommonReferenceTargetIssue::IncompatibleTarget { target, .. })
                if target.record().section_kind() == DxfRawRecordSectionKind::Tables
        ));
    }
    Ok(())
}

#[test]
fn source_resolution_failures_precede_target_kind_checks() -> Result<(), Box<dyn Error>> {
    let groups = negative_target_groups();
    for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
        let bytes = encode(format, DxfAcadVersion::Ac1032, &groups)?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let directory = match format {
            DxfRawDocumentFormat::Ascii => {
                open_ascii(&source)?.entity_common_reference_target_directory(&token())?
            }
            DxfRawDocumentFormat::Binary => {
                open_binary(&source)?.entity_common_reference_target_directory(&token())?
            }
            _ => return Err(io::Error::other("test format").into()),
        };
        let entity = target_entity(&directory)?;
        assert!(matches!(
            reviewed_target(&directory, entity, DxfEntityField::EXTENSION_DICTIONARY)?
                .invalid_issue(),
            Some(DxfEntityCommonReferenceTargetIssue::Source(
                DxfEntityCommonReferenceIssue::Missing
            ))
        ));
        assert!(matches!(
            reviewed_target(&directory, entity, DxfEntityField::MATERIAL)?.invalid_issue(),
            Some(DxfEntityCommonReferenceTargetIssue::Source(
                DxfEntityCommonReferenceIssue::Ambiguous { target_count: 2 }
            ))
        ));
        assert!(matches!(
            reviewed_target(&directory, entity, DxfEntityField::PLOT_STYLE)?.invalid_issue(),
            Some(DxfEntityCommonReferenceTargetIssue::IncompatibleTarget { .. })
        ));
        assert!(matches!(
            target_entry(&directory, entity, DxfEntityField::OWNER)?.semantics(),
            DxfEntityCommonReferenceTargetSemantics::Unreviewed(DxfSemanticValue::Invalid {
                issue: DxfEntityCommonReferenceIssue::Null,
                ..
            })
        ));
    }
    Ok(())
}

#[test]
fn target_directory_cancellation_identity_and_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityCommonReferenceTargetDirectory>();
    assert_copy::<seacad_dxf_core::DxfEntityCommonReferenceTargetEntry>();
    assert_copy::<DxfEntityCommonReferenceTargetSemantics>();

    let bytes = target_kind_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.entity_common_reference_target_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));
    let directory = document.entity_common_reference_target_directory(&token())?;
    let entity = target_entity(&directory)?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(directory.entries().len(), 4);
    assert_eq!(directory.entries_for_entity(entity)?.len(), 4);
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(
        directory.entry_for_field(entity, DxfEntityField::HANDLE)?,
        None
    );

    let other_bytes = target_kind_fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let other_directory = other.entity_common_reference_target_directory(&token())?;
    assert!(matches!(
        directory.entries_for_entity(target_entity(&other_directory)?),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn assert_valid(
    handles: &DxfEntityCommonHandleDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(handles.entries().len(), 5);
    let entity = only_entity(handles)?;
    let identity = entry(handles, entity, DxfEntityField::HANDLE)?;
    assert!(matches!(
        identity.semantics(),
        DxfEntityCommonHandleSemantics::Identity(DxfSemanticValue::Explicit {
            value: DxfEntityFieldValue::Handle(handle),
            ..
        }) if handle == DxfHandle::from_u64(0x10)
    ));

    if version == DxfAcadVersion::Ac1009 {
        for field in [DxfEntityField::OWNER, DxfEntityField::PLOT_STYLE] {
            assert!(matches!(
                reference(handles, entity, field)?,
                DxfSemanticValue::Invalid {
                    issue: DxfEntityCommonReferenceIssue::Field(
                        DxfEntityFieldSemanticIssue::MissingRequired
                    ),
                    raw: None,
                    ..
                }
            ));
        }
        assert_eq!(
            reference(handles, entity, DxfEntityField::EXTENSION_DICTIONARY)?.state(),
            DxfSemanticValueState::Absent
        );
        assert_eq!(
            reference(handles, entity, DxfEntityField::MATERIAL)?.value(),
            Some(&DxfEntityCommonReferenceValue::ByLayer)
        );
    } else {
        for (field, expected) in [
            (DxfEntityField::OWNER, 0x1f),
            (DxfEntityField::EXTENSION_DICTIONARY, 0x2f),
            (DxfEntityField::MATERIAL, 0x3f),
            (DxfEntityField::PLOT_STYLE, 0x4f),
        ] {
            let value = reference(handles, entity, field)?;
            assert_eq!(value.state(), DxfSemanticValueState::Explicit);
            let Some(DxfEntityCommonReferenceValue::Resolved { handle, target }) = value.value()
            else {
                return Err(io::Error::other("resolved reference").into());
            };
            assert_eq!(handle.value(), expected);
            assert_eq!(target.handle().value(), expected);
            assert!(!target.candidate().group().value_payload_span().is_empty());
        }
    }
    Ok(())
}

fn assert_target_failures(handles: &DxfEntityCommonHandleDirectory) -> Result<(), Box<dyn Error>> {
    let entity = only_entity(handles)?;
    let cases = [
        (DxfEntityField::OWNER, DxfEntityCommonReferenceIssue::Null),
        (
            DxfEntityField::EXTENSION_DICTIONARY,
            DxfEntityCommonReferenceIssue::Missing,
        ),
        (
            DxfEntityField::MATERIAL,
            DxfEntityCommonReferenceIssue::Ambiguous { target_count: 2 },
        ),
    ];
    for (field, expected) in cases {
        let value = reference(handles, entity, field)?;
        assert_eq!(value.invalid_issue(), Some(&expected));
        assert!(value.raw_provenance().is_some());
    }
    assert!(matches!(
        reference(handles, entity, DxfEntityField::PLOT_STYLE)?.value(),
        Some(DxfEntityCommonReferenceValue::Resolved { handle, .. })
            if handle.value() == 0x1f
    ));
    Ok(())
}

type Signature = Vec<(DxfEntityField, u8, Option<u64>)>;

fn signature(handles: &DxfEntityCommonHandleDirectory) -> Result<Signature, Box<dyn Error>> {
    let mut result = Vec::new();
    for entry in handles.entries().iter().copied() {
        let (state, target) = match entry.semantics() {
            DxfEntityCommonHandleSemantics::Identity(value) => {
                let handle = match value.value() {
                    Some(DxfEntityFieldValue::Handle(handle)) => Some(handle.value()),
                    _ => None,
                };
                (value.state() as u8, handle)
            }
            DxfEntityCommonHandleSemantics::Reference(value) => {
                let target = match value.value() {
                    Some(DxfEntityCommonReferenceValue::Resolved { target, .. }) => {
                        Some(target.handle().value())
                    }
                    _ => None,
                };
                (value.state() as u8, target)
            }
            _ => return Err(io::Error::other("unknown common handle semantics").into()),
        };
        result.push((entry.field(), state, target));
    }
    Ok(result)
}

fn reference(
    handles: &DxfEntityCommonHandleDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<DxfEntityCommonReferenceSemanticValue, Box<dyn Error>> {
    match entry(handles, entity, field)?.semantics() {
        DxfEntityCommonHandleSemantics::Reference(value) => Ok(value),
        _ => Err(io::Error::other("reference semantics").into()),
    }
}

fn entry(
    handles: &DxfEntityCommonHandleDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<seacad_dxf_core::DxfEntityCommonHandleEntry, Box<dyn Error>> {
    handles
        .entry_for_field(entity, field)?
        .ok_or_else(|| io::Error::other("common handle entry").into())
}

fn only_entity(
    handles: &DxfEntityCommonHandleDirectory,
) -> Result<seacad_dxf_core::DxfEntityRef, Box<dyn Error>> {
    let [entity] = handles
        .source_directory()
        .evidence_directory()
        .entity_directory()
        .entities()
    else {
        return Err(io::Error::other("one entity").into());
    };
    Ok(*entity)
}

fn assert_valid_target_kinds(
    directory: &DxfEntityCommonReferenceTargetDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 4);
    let entity = target_entity(directory)?;
    let owner = target_entry(directory, entity, DxfEntityField::OWNER)?;
    assert_eq!(owner.expected_target_kind(), None);
    assert!(matches!(
        owner.semantics(),
        DxfEntityCommonReferenceTargetSemantics::Unreviewed(_)
    ));

    if version == DxfAcadVersion::Ac1009 {
        assert_eq!(
            reviewed_target(directory, entity, DxfEntityField::EXTENSION_DICTIONARY)?.state(),
            DxfSemanticValueState::Absent
        );
        let material = reviewed_target(directory, entity, DxfEntityField::MATERIAL)?;
        assert_eq!(material.state(), DxfSemanticValueState::Defaulted);
        assert_eq!(
            material.value(),
            Some(&DxfEntityCommonReferenceValue::ByLayer)
        );
        assert!(matches!(
            reviewed_target(directory, entity, DxfEntityField::PLOT_STYLE)?.invalid_issue(),
            Some(DxfEntityCommonReferenceTargetIssue::Source(
                DxfEntityCommonReferenceIssue::Field(DxfEntityFieldSemanticIssue::MissingRequired)
            ))
        ));
    } else {
        for (field, kind, handle) in [
            (
                DxfEntityField::EXTENSION_DICTIONARY,
                DxfEntityCommonReferenceTargetKind::ExtensionDictionary,
                0x2f,
            ),
            (
                DxfEntityField::MATERIAL,
                DxfEntityCommonReferenceTargetKind::Material,
                0x3f,
            ),
            (
                DxfEntityField::PLOT_STYLE,
                DxfEntityCommonReferenceTargetKind::PlotStyle,
                0x4f,
            ),
        ] {
            let entry = target_entry(directory, entity, field)?;
            assert_eq!(entry.expected_target_kind(), Some(kind));
            let value = reviewed_target(directory, entity, field)?;
            assert_eq!(value.state(), DxfSemanticValueState::Explicit);
            assert!(matches!(
                value.value(),
                Some(DxfEntityCommonReferenceValue::Resolved { target, .. })
                    if target.handle().value() == handle
            ));
        }
    }
    Ok(())
}

type TargetSignature = Vec<(
    DxfEntityField,
    Option<DxfEntityCommonReferenceTargetKind>,
    bool,
    u8,
    Option<u64>,
)>;

fn target_signature(
    directory: &DxfEntityCommonReferenceTargetDirectory,
) -> Result<TargetSignature, Box<dyn Error>> {
    let mut result = Vec::new();
    for entry in directory.entries().iter().copied() {
        let (reviewed, state, target) = match entry.semantics() {
            DxfEntityCommonReferenceTargetSemantics::Unreviewed(value) => {
                (false, value.state(), resolved_target(value.value()))
            }
            DxfEntityCommonReferenceTargetSemantics::Reviewed(value) => {
                (true, value.state(), resolved_target(value.value()))
            }
            _ => return Err(io::Error::other("unknown target semantics").into()),
        };
        result.push((
            entry.field(),
            entry.expected_target_kind(),
            reviewed,
            state as u8,
            target,
        ));
    }
    Ok(result)
}

fn resolved_target(value: Option<&DxfEntityCommonReferenceValue>) -> Option<u64> {
    match value {
        Some(DxfEntityCommonReferenceValue::Resolved { target, .. }) => {
            Some(target.handle().value())
        }
        _ => None,
    }
}

fn reviewed_target(
    directory: &DxfEntityCommonReferenceTargetDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<seacad_dxf_core::DxfEntityCommonReferenceTargetSemanticValue, Box<dyn Error>> {
    match target_entry(directory, entity, field)?.semantics() {
        DxfEntityCommonReferenceTargetSemantics::Reviewed(value) => Ok(value),
        _ => Err(io::Error::other("reviewed target semantics").into()),
    }
}

fn target_entry(
    directory: &DxfEntityCommonReferenceTargetDirectory,
    entity: seacad_dxf_core::DxfEntityRef,
    field: DxfEntityField,
) -> Result<seacad_dxf_core::DxfEntityCommonReferenceTargetEntry, Box<dyn Error>> {
    directory
        .entry_for_field(entity, field)?
        .ok_or_else(|| io::Error::other("common reference target entry").into())
}

fn target_entity(
    directory: &DxfEntityCommonReferenceTargetDirectory,
) -> Result<seacad_dxf_core::DxfEntityRef, Box<dyn Error>> {
    let [entity] = directory
        .source_directory()
        .source_directory()
        .evidence_directory()
        .entity_directory()
        .entities()
    else {
        return Err(io::Error::other("one target entity").into());
    };
    Ok(*entity)
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
}

fn valid_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend(target_records(&[b"1F", b"2F", b"3F", b"4F"]));
    }
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (330, Value::Text(b"1F")),
            (102, Value::Text(b"{ACAD_XDICTIONARY")),
            (360, Value::Text(b"2F")),
            (102, Value::Text(b"}")),
            (100, Value::Text(b"AcDbEntity")),
            (390, Value::Text(b"4F")),
            (347, Value::Text(b"3F")),
            (8, Value::Text(b"SECRET")),
            (100, Value::Text(b"AcDbLine")),
        ]);
    } else {
        groups.push((8, Value::Text(b"SECRET")));
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
}

fn target_kind_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend(target_records_with_markers(&[
            (b"DICTIONARY", b"1F"),
            (b"DICTIONARY", b"2F"),
            (b"MATERIAL", b"3F"),
            (b"ACDBPLACEHOLDER", b"4F"),
        ]));
    }
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([
            (330, Value::Text(b"1F")),
            (102, Value::Text(b"{ACAD_XDICTIONARY")),
            (360, Value::Text(b"2F")),
            (102, Value::Text(b"}")),
            (100, Value::Text(b"AcDbEntity")),
            (390, Value::Text(b"4F")),
            (347, Value::Text(b"3F")),
            (8, Value::Text(b"SECRET")),
            (100, Value::Text(b"AcDbLine")),
        ]);
    } else {
        groups.push((8, Value::Text(b"SECRET")));
    }
    groups.extend([(0, Value::Text(b"ENDSEC")), (0, Value::Text(b"EOF"))]);
    encode(format, version, &groups)
}

fn wrong_target_kind_groups() -> Vec<(i16, Value<'static>)> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend(target_records_with_markers(&[
        (b"DICTIONARY", b"1F"),
        (b"XRECORD", b"2F"),
        (b"DICTIONARY", b"3F"),
        (b"MATERIAL", b"4F"),
    ]));
    groups.extend(reference_entity_groups());
    groups
}

fn wrong_target_section_groups() -> Vec<(i16, Value<'static>)> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"TABLES")),
        (0, Value::Text(b"DICTIONARY")),
        (5, Value::Text(b"2F")),
        (0, Value::Text(b"ENDSEC")),
    ]);
    groups.extend(target_records_with_markers(&[
        (b"DICTIONARY", b"1F"),
        (b"MATERIAL", b"3F"),
        (b"ACDBPLACEHOLDER", b"4F"),
    ]));
    groups.extend(reference_entity_groups());
    groups
}

fn reference_entity_groups() -> [(i16, Value<'static>); 15] {
    [
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (330, Value::Text(b"1F")),
        (102, Value::Text(b"{ACAD_XDICTIONARY")),
        (360, Value::Text(b"2F")),
        (102, Value::Text(b"}")),
        (100, Value::Text(b"AcDbEntity")),
        (390, Value::Text(b"4F")),
        (347, Value::Text(b"3F")),
        (8, Value::Text(b"SECRET")),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]
}

fn negative_target_groups() -> Vec<(i16, Value<'static>)> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend(target_records(&[b"1F", b"4F", b"4F"]));
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (330, Value::Text(b"0")),
        (102, Value::Text(b"{ACAD_XDICTIONARY")),
        (360, Value::Text(b"99")),
        (102, Value::Text(b"}")),
        (100, Value::Text(b"AcDbEntity")),
        (347, Value::Text(b"4F")),
        (390, Value::Text(b"1F")),
        (8, Value::Text(b"SECRET")),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    groups
}

fn field_failure_groups() -> Vec<(i16, Value<'static>)> {
    let mut groups = header(DxfAcadVersion::Ac1032);
    groups.extend(target_records(&[b"1F"]));
    groups.extend([
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
        (330, Value::Text(b"1F")),
        (330, Value::Text(b"1F")),
        (100, Value::Text(b"AcDbEntity")),
        (390, Value::Text(b"Z")),
        (8, Value::Text(b"SECRET")),
        (100, Value::Text(b"AcDbLine")),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    groups
}

fn header(version: DxfAcadVersion) -> Vec<(i16, Value<'static>)> {
    vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
    ]
}

fn target_records(handles: &[&'static [u8]]) -> Vec<(i16, Value<'static>)> {
    let mut groups = vec![(0, Value::Text(b"SECTION")), (2, Value::Text(b"OBJECTS"))];
    for handle in handles {
        groups.extend([(0, Value::Text(b"DICTIONARY")), (5, Value::Text(handle))]);
    }
    groups.push((0, Value::Text(b"ENDSEC")));
    groups
}

fn target_records_with_markers(
    records: &[(&'static [u8], &'static [u8])],
) -> Vec<(i16, Value<'static>)> {
    let mut groups = vec![(0, Value::Text(b"SECTION")), (2, Value::Text(b"OBJECTS"))];
    for (marker, handle) in records {
        groups.extend([(0, Value::Text(marker)), (5, Value::Text(handle))]);
    }
    groups.push((0, Value::Text(b"ENDSEC")));
    groups
}

fn encode(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, Value::Text(value)) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(value);
        bytes.extend_from_slice(b"\r\n");
    }
    bytes
}

fn binary_groups(
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, Value::Text(value)) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        bytes.extend_from_slice(value);
        bytes.push(0);
    }
    Ok(bytes)
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

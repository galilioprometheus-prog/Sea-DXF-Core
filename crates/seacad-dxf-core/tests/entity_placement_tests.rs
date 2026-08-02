use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockDefinitionState, DxfByteSource, DxfCancellationToken, DxfEntityPlacementDirectory,
    DxfEntityPlacementState, DxfEntityPlacementTarget, DxfError, DxfMemorySource,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfRawRecordSectionKind, DxfRawRecordSectionState,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_placement_parity_and_executable_anchors()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let directory = view.entity_placement_directory(&token())?;
            assert_eq!(directory.source_id(), view.source_id());
            assert_eq!(
                directory.raw_record_directory().source_id(),
                view.source_id()
            );
            assert_eq!(directory.assessments().len(), 4);

            for assessment in directory.assessments() {
                let placement = assessment
                    .placement()
                    .ok_or_else(|| io::Error::other("ready placement"))?;
                assert_eq!(placement.source_id(), view.source_id());
                assert!(placement.insertion_span().is_empty());
                assert_eq!(placement.target(), assessment.target());
                let following = view
                    .group(placement.following_group_occurrence())
                    .ok_or_else(|| io::Error::other("following group"))?;
                assert_eq!(following.group_code().value(), 0);
                assert_eq!(
                    following.full_span().start(),
                    placement.insertion_span().start()
                );

                let mut builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
                builder.replace_raw_span(
                    placement.insertion_span(),
                    &point_record(format, version)?,
                    &token(),
                )?;
                let plan = builder.finish(&token())?;
                let output = materialize(&bytes, &plan)?;
                let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                let post_document = open_document(&output_source, format)?;
                let post = post_document.view();
                assert_eq!(
                    post.raw_record_directory(&token())?.records().len(),
                    view.raw_record_directory(&token())?.records().len() + 1
                );
            }
        }
    }
    Ok(())
}

#[test]
fn targets_distinguish_entities_sections_and_closed_block_member_lists()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let directory = view.entity_placement_directory(&token())?;

    let entities = directory
        .assessments()
        .iter()
        .filter(|assessment| {
            matches!(
                assessment.target(),
                DxfEntityPlacementTarget::EntitiesSection { .. }
            )
        })
        .count();
    let blocks = directory
        .assessments()
        .iter()
        .filter(|assessment| {
            matches!(
                assessment.target(),
                DxfEntityPlacementTarget::BlockDefinition { .. }
            )
        })
        .count();
    assert_eq!((entities, blocks), (2, 2));

    for assessment in directory.assessments() {
        let looked_up = directory
            .assessment_for_target(assessment.target())
            .ok_or_else(|| io::Error::other("target lookup"))?;
        assert_eq!(looked_up, *assessment);
        let placement = looked_up
            .placement()
            .ok_or_else(|| io::Error::other("placement"))?;
        match placement.target() {
            DxfEntityPlacementTarget::EntitiesSection { .. } => {
                assert_eq!(placement.section_kind(), DxfRawRecordSectionKind::Entities);
            }
            DxfEntityPlacementTarget::BlockDefinition { .. } => {
                assert_eq!(placement.section_kind(), DxfRawRecordSectionKind::Blocks);
            }
            _ => return Err(io::Error::other("unexpected placement target").into()),
        }
    }
    assert_send_sync::<DxfEntityPlacementDirectory>();
    Ok(())
}

#[test]
fn malformed_sections_blocks_and_orphan_groups_are_typed_without_guessing()
-> Result<(), Box<dyn Error>> {
    let interrupted = ascii_document(
        DxfAcadVersion::Ac1032,
        &[
            (0, b"SECTION"),
            (2, b"ENTITIES"),
            (0, b"LINE"),
            (0, b"SECTION"),
            (2, b"ENTITIES"),
            (0, b"ENDSEC"),
            (0, b"EOF"),
        ],
    );
    let interrupted_source = DxfMemorySource::new(&interrupted, DxfResourceProfile::Safe)?;
    let interrupted_document = open_ascii(&interrupted_source)?;
    let interrupted_directory = interrupted_document.entity_placement_directory(&token())?;
    assert!(interrupted_directory.assessments().iter().any(|entry| {
        matches!(
            entry.state(),
            DxfEntityPlacementState::SectionUnavailable {
                state: DxfRawRecordSectionState::Interrupted
            }
        )
    }));

    let unclosed = ascii_document(
        DxfAcadVersion::Ac1032,
        &[(0, b"SECTION"), (2, b"ENTITIES"), (0, b"LINE"), (0, b"EOF")],
    );
    let unclosed_source = DxfMemorySource::new(&unclosed, DxfResourceProfile::Safe)?;
    let unclosed_document = open_ascii(&unclosed_source)?;
    let unclosed_directory = unclosed_document.entity_placement_directory(&token())?;
    assert!(matches!(
        unclosed_directory.assessments()[0].state(),
        DxfEntityPlacementState::SectionUnavailable {
            state: DxfRawRecordSectionState::Unclosed
        }
    ));

    let blocks = ascii_document(
        DxfAcadVersion::Ac1032,
        &[
            (0, b"SECTION"),
            (2, b"BLOCKS"),
            (0, b"BLOCK"),
            (2, b"A"),
            (0, b"BLOCK"),
            (2, b"B"),
            (0, b"ENDSEC"),
            (0, b"EOF"),
        ],
    );
    let blocks_source = DxfMemorySource::new(&blocks, DxfResourceProfile::Safe)?;
    let blocks_document = open_ascii(&blocks_source)?;
    let blocks_directory = blocks_document.entity_placement_directory(&token())?;
    assert!(blocks_directory.assessments().iter().any(|entry| {
        matches!(
            entry.state(),
            DxfEntityPlacementState::BlockDefinitionUnavailable {
                state: DxfBlockDefinitionState::Interrupted
            }
        )
    }));
    assert!(blocks_directory.assessments().iter().any(|entry| {
        matches!(
            entry.state(),
            DxfEntityPlacementState::BlockDefinitionUnavailable {
                state: DxfBlockDefinitionState::Unclosed
            }
        )
    }));

    let orphan = ascii_document(
        DxfAcadVersion::Ac1032,
        &[
            (0, b"SECTION"),
            (2, b"ENTITIES"),
            (8, b"ORPHAN"),
            (0, b"LINE"),
            (0, b"ENDSEC"),
            (0, b"EOF"),
        ],
    );
    let orphan_source = DxfMemorySource::new(&orphan, DxfResourceProfile::Safe)?;
    let orphan_document = open_ascii(&orphan_source)?;
    let orphan_directory = orphan_document.entity_placement_directory(&token())?;
    assert!(matches!(
        orphan_directory.assessments()[0].state(),
        DxfEntityPlacementState::InvalidAnchorGroup {
            group_occurrence: 7,
            group_code
        } if group_code.value() == 8
    ));
    Ok(())
}

#[test]
fn pre_cancelled_directory_stops_typed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.entity_placement_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let groups = [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"BLOCKS"),
        (0, b"BLOCK"),
        (2, b"WITH_MEMBER"),
        (0, b"LINE"),
        (0, b"ENDBLK"),
        (0, b"BLOCK"),
        (2, b"EMPTY"),
        (0, b"ENDBLK"),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"LINE"),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"ENDSEC"),
        (0, b"EOF"),
    ];
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_document(version, &groups[5..])),
        DxfRawDocumentFormat::Binary => binary_document(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_document(version: DxfAcadVersion, tail: &[(i16, &[u8])]) -> Vec<u8> {
    let header = [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
    ];
    let mut bytes = Vec::new();
    for (code, value) in header.into_iter().chain(tail.iter().copied()) {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(value);
        bytes.push(b'\n');
    }
    bytes
}

fn binary_document(version: DxfAcadVersion, groups: &[(i16, &[u8])]) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("group code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        bytes.extend_from_slice(value);
        bytes.push(0);
    }
    Ok(bytes)
}

fn point_record(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(b"0\nPOINT\n".to_vec()),
        DxfRawDocumentFormat::Binary => {
            let mut bytes = Vec::new();
            if version == DxfAcadVersion::Ac1009 {
                bytes.push(0);
            } else {
                bytes.extend_from_slice(&0_i16.to_le_bytes());
            }
            bytes.extend_from_slice(b"POINT\0");
            Ok(bytes)
        }
        _ => Err(io::Error::other("format")),
    }
}

fn materialize(
    source: &[u8],
    plan: &seacad_dxf_core::DxfTransactionPlan,
) -> Result<Vec<u8>, DxfError> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start =
            usize::try_from(patch.source_span().start()).map_err(|_| invalid_test_data())?;
        let end = usize::try_from(patch.source_span().end()).map_err(|_| invalid_test_data())?;
        output.extend_from_slice(source.get(cursor..start).ok_or_else(invalid_test_data)?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or_else(invalid_test_data)?,
        );
        cursor = end;
    }
    output.extend_from_slice(source.get(cursor..).ok_or_else(invalid_test_data)?);
    Ok(output)
}

enum OpenedDocument<'a> {
    Ascii(DxfAsciiRawDocument<'a>),
    Binary(DxfBinaryRawDocument<'a>),
}

impl OpenedDocument<'_> {
    fn view(&self) -> DxfRawDocumentView<'_> {
        match self {
            Self::Ascii(document) => DxfRawDocumentView::from(document),
            Self::Binary(document) => DxfRawDocumentView::from(document),
        }
    }
}

fn open_document<'a>(
    source: &'a dyn DxfByteSource,
    format: DxfRawDocumentFormat,
) -> Result<OpenedDocument<'a>, Box<dyn Error>> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(OpenedDocument::Ascii(open_ascii(source)?)),
        DxfRawDocumentFormat::Binary => Ok(OpenedDocument::Binary(open_binary(source)?)),
        _ => Err(io::Error::other("format").into()),
    }
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_send_sync<T: Send + Sync>() {}

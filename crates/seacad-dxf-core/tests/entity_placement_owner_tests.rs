use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfCommonOwnerCandidateState, DxfEntityPlacement,
    DxfEntityPlacementOwnerBinding, DxfEntityPlacementOwnerDirectory, DxfEntityPlacementOwnerIssue,
    DxfEntityPlacementOwnerOutcome, DxfEntityPlacementTarget, DxfError, DxfHandle,
    DxfHandleParseIssue, DxfHandleResolutionState, DxfMemorySource, DxfNamedSymbolTableKind,
    DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_owner_binding_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let mut signatures = Vec::new();
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, &FixtureOptions::default())?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let directory = view.entity_placement_owner_directory(&token())?;
            assert_eq!(directory.source_id(), view.source_id());
            assert_eq!(directory.placement_directory().assessments().len(), 2);
            assert_eq!(directory.named_symbol_table_directory().entries().len(), 3);

            let entities = placement(&directory, false)?;
            let entities_outcome = directory.bind(entities, handle(0x10), &token())?;
            assert_bound(entities_outcome, entities, handle(0x10))?;

            let block = placement(&directory, true)?;
            let block_outcome = directory.bind(block, handle(0x11), &token())?;
            let block_signature = match block_outcome {
                DxfEntityPlacementOwnerOutcome::Bound(binding) => {
                    assert_ne!(version, DxfAcadVersion::Ac1009);
                    assert_bound(
                        DxfEntityPlacementOwnerOutcome::Bound(binding),
                        block,
                        handle(0x11),
                    )?;
                    1_u8
                }
                DxfEntityPlacementOwnerOutcome::Rejected(
                    DxfEntityPlacementOwnerIssue::BlockOwnerCardinality {
                        state: DxfCommonOwnerCandidateState::NoCandidate,
                    },
                ) => {
                    assert_eq!(version, DxfAcadVersion::Ac1009);
                    0
                }
                _ => return Err(io::Error::other("unexpected block outcome").into()),
            };
            signatures.push((
                directory
                    .named_symbol_table_directory()
                    .entries()
                    .iter()
                    .map(|entry| (entry.record().ordinal(), entry.kind()))
                    .collect::<Vec<_>>(),
                matches!(entities_outcome, DxfEntityPlacementOwnerOutcome::Bound(_)),
                block_signature,
            ));
        }
        assert_eq!(signatures[0], signatures[1]);
    }
    Ok(())
}

#[test]
fn requested_owner_must_be_unique_non_null_and_a_closed_block_record() -> Result<(), Box<dyn Error>>
{
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        &FixtureOptions {
            duplicate_model_handle: true,
            ..FixtureOptions::default()
        },
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_placement_owner_directory(&token())?;
    let entities = placement(&directory, false)?;

    assert_eq!(
        directory.bind(entities, handle(0), &token())?,
        DxfEntityPlacementOwnerOutcome::Rejected(DxfEntityPlacementOwnerIssue::NullOwner)
    );
    assert_eq!(
        directory.bind(entities, handle(0xFE), &token())?,
        DxfEntityPlacementOwnerOutcome::Rejected(DxfEntityPlacementOwnerIssue::OwnerMissing {
            handle: handle(0xFE)
        })
    );
    assert!(matches!(
        directory.bind(entities, handle(0x10), &token())?,
        DxfEntityPlacementOwnerOutcome::Rejected(
            DxfEntityPlacementOwnerIssue::OwnerAmbiguous {
                handle: value,
                target_count: 2
            }
        ) if value == handle(0x10)
    ));
    assert!(matches!(
        directory.bind(entities, handle(0x20), &token())?,
        DxfEntityPlacementOwnerOutcome::Rejected(
            DxfEntityPlacementOwnerIssue::OwnerNotBlockRecord { target }
        ) if target.handle() == handle(0x20)
            && target.record().section_kind() == seacad_dxf_core::DxfRawRecordSectionKind::Tables
    ));
    Ok(())
}

#[test]
fn block_header_owner_cardinality_resolution_and_match_are_fail_closed()
-> Result<(), Box<dyn Error>> {
    let cases: &[(&[&str], bool, ExpectedBlockIssue)] = &[
        (&[], false, ExpectedBlockIssue::NoCandidate),
        (&["GG"], false, ExpectedBlockIssue::Invalid),
        (&["0"], false, ExpectedBlockIssue::Null),
        (&["FE"], false, ExpectedBlockIssue::Missing),
        (&["AA"], true, ExpectedBlockIssue::Ambiguous),
        (&["10"], false, ExpectedBlockIssue::Mismatch),
        (&["11", "10"], false, ExpectedBlockIssue::Multiple),
    ];
    for (block_owners, duplicate_aux_handle, expected) in cases.iter().copied() {
        let bytes = fixture(
            DxfRawDocumentFormat::Ascii,
            DxfAcadVersion::Ac1032,
            &FixtureOptions {
                block_owners,
                duplicate_aux_handle,
                ..FixtureOptions::default()
            },
        )?;
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let directory = document.entity_placement_owner_directory(&token())?;
        let outcome = directory.bind(placement(&directory, true)?, handle(0x11), &token())?;
        assert_block_issue(outcome, expected)?;
    }
    Ok(())
}

#[test]
fn cancellation_source_identity_and_public_bounds_remain_explicit() -> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityPlacementOwnerDirectory>();
    assert_copy::<DxfEntityPlacementOwnerBinding>();
    assert_copy::<DxfEntityPlacementOwnerIssue>();
    assert_copy::<DxfEntityPlacementOwnerOutcome>();
    assert!(std::mem::size_of::<DxfEntityPlacementOwnerBinding>() <= 256);
    assert!(std::mem::size_of::<DxfEntityPlacementOwnerIssue>() <= 256);

    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        &FixtureOptions::default(),
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.entity_placement_owner_directory(&token())?;
    assert!(
        directory.common_owner_candidate_directory().entries().len()
            <= directory
                .common_owner_candidate_directory()
                .record_entries()
                .len()
                .saturating_mul(2)
    );

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        document.entity_placement_owner_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    assert!(matches!(
        directory.bind(placement(&directory, false)?, handle(0x10), &cancellation),
        Err(DxfError::Cancelled)
    ));

    let other_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1027,
        &FixtureOptions::default(),
    )?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other_directory = other_document.entity_placement_owner_directory(&token())?;
    let foreign = placement(&other_directory, false)?;
    assert!(matches!(
        directory.bind(foreign, handle(0x10), &token()),
        Err(DxfError::SourceIdentityMismatch { expected, observed })
            if expected == directory.source_id() && observed == other_directory.source_id()
    ));
    Ok(())
}

#[derive(Clone, Copy)]
struct FixtureOptions<'a> {
    block_owners: &'a [&'a str],
    duplicate_model_handle: bool,
    duplicate_aux_handle: bool,
}

impl Default for FixtureOptions<'_> {
    fn default() -> Self {
        Self {
            block_owners: &["11"],
            duplicate_model_handle: false,
            duplicate_aux_handle: false,
        }
    }
}

#[derive(Clone, Copy)]
enum ExpectedBlockIssue {
    NoCandidate,
    Invalid,
    Null,
    Missing,
    Ambiguous,
    Mismatch,
    Multiple,
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    options: &FixtureOptions<'_>,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0, "SECTION".to_string()),
        (2, "HEADER".to_string()),
        (9, "$ACADVER".to_string()),
        (1, version.code().to_string()),
        (0, "ENDSEC".to_string()),
        (0, "SECTION".to_string()),
        (2, "TABLES".to_string()),
        (0, "TABLE".to_string()),
        (2, "BLOCK_RECORD".to_string()),
        (0, "BLOCK_RECORD".to_string()),
        (5, "10".to_string()),
        (2, "*Model_Space".to_string()),
        (0, "BLOCK_RECORD".to_string()),
        (5, "11".to_string()),
        (2, "B".to_string()),
    ];
    if options.duplicate_model_handle {
        groups.extend([
            (0, "BLOCK_RECORD".to_string()),
            (5, "10".to_string()),
            (2, "DUPLICATE_MODEL".to_string()),
        ]);
    }
    groups.extend([
        (0, "ENDTAB".to_string()),
        (0, "TABLE".to_string()),
        (2, "LAYER".to_string()),
        (0, "LAYER".to_string()),
        (5, "20".to_string()),
        (2, "0".to_string()),
        (0, "ENDTAB".to_string()),
        (0, "ENDSEC".to_string()),
        (0, "SECTION".to_string()),
        (2, "OBJECTS".to_string()),
        (0, "AUX".to_string()),
        (5, "AA".to_string()),
    ]);
    if options.duplicate_aux_handle {
        groups.extend([(0, "AUX".to_string()), (5, "AA".to_string())]);
    }
    groups.extend([
        (0, "ENDSEC".to_string()),
        (0, "SECTION".to_string()),
        (2, "BLOCKS".to_string()),
        (0, "BLOCK".to_string()),
        (5, "30".to_string()),
    ]);
    if version != DxfAcadVersion::Ac1009 {
        for owner in options.block_owners {
            groups.push((330, (*owner).to_string()));
        }
    }
    groups.extend([
        (2, "B".to_string()),
        (0, "ENDBLK".to_string()),
        (0, "ENDSEC".to_string()),
        (0, "SECTION".to_string()),
        (2, "ENTITIES".to_string()),
        (0, "POINT".to_string()),
        (5, "31".to_string()),
        (0, "ENDSEC".to_string()),
        (0, "EOF".to_string()),
    ]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_document(&groups)),
        DxfRawDocumentFormat::Binary => binary_document(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_document(groups: &[(i16, String)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

fn binary_document(
    version: DxfAcadVersion,
    groups: &[(i16, String)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
    }
    Ok(bytes)
}

fn placement(
    directory: &DxfEntityPlacementOwnerDirectory,
    block: bool,
) -> Result<DxfEntityPlacement, io::Error> {
    directory
        .placement_directory()
        .assessments()
        .iter()
        .find_map(|assessment| {
            let matches = matches!(
                assessment.target(),
                DxfEntityPlacementTarget::BlockDefinition { .. }
            ) == block;
            matches.then(|| assessment.placement()).flatten()
        })
        .ok_or_else(|| io::Error::other("placement"))
}

fn assert_bound(
    outcome: DxfEntityPlacementOwnerOutcome,
    placement: DxfEntityPlacement,
    owner: DxfHandle,
) -> Result<(), io::Error> {
    let DxfEntityPlacementOwnerOutcome::Bound(binding) = outcome else {
        return Err(io::Error::other("binding"));
    };
    assert_eq!(binding.source_id(), placement.source_id());
    assert_eq!(binding.placement(), placement);
    assert_eq!(binding.owner_handle(), owner);
    assert_eq!(
        binding.block_record_entry().kind(),
        DxfNamedSymbolTableKind::BlockRecord
    );
    Ok(())
}

fn assert_block_issue(
    outcome: DxfEntityPlacementOwnerOutcome,
    expected: ExpectedBlockIssue,
) -> Result<(), io::Error> {
    let DxfEntityPlacementOwnerOutcome::Rejected(issue) = outcome else {
        return Err(io::Error::other("rejected block binding"));
    };
    let matches = match (expected, issue) {
        (
            ExpectedBlockIssue::NoCandidate,
            DxfEntityPlacementOwnerIssue::BlockOwnerCardinality {
                state: DxfCommonOwnerCandidateState::NoCandidate,
            },
        )
        | (
            ExpectedBlockIssue::Multiple,
            DxfEntityPlacementOwnerIssue::BlockOwnerCardinality {
                state: DxfCommonOwnerCandidateState::MultipleCandidates { candidate_count: 2 },
            },
        ) => true,
        (
            ExpectedBlockIssue::Invalid,
            DxfEntityPlacementOwnerIssue::BlockOwnerResolution {
                state:
                    DxfHandleResolutionState::Invalid(DxfHandleParseIssue::InvalidDigit { offset: 0 }),
            },
        )
        | (
            ExpectedBlockIssue::Null,
            DxfEntityPlacementOwnerIssue::BlockOwnerResolution {
                state: DxfHandleResolutionState::Null,
            },
        )
        | (
            ExpectedBlockIssue::Missing,
            DxfEntityPlacementOwnerIssue::BlockOwnerResolution {
                state: DxfHandleResolutionState::Missing,
            },
        )
        | (
            ExpectedBlockIssue::Ambiguous,
            DxfEntityPlacementOwnerIssue::BlockOwnerResolution {
                state: DxfHandleResolutionState::Ambiguous { target_count: 2 },
            },
        ) => true,
        (
            ExpectedBlockIssue::Mismatch,
            DxfEntityPlacementOwnerIssue::BlockOwnerMismatch {
                declared,
                requested,
            },
        ) => declared.handle() == handle(0x10) && requested.handle() == handle(0x11),
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(io::Error::other("wrong block issue"))
    }
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

const fn handle(value: u64) -> DxfHandle {
    DxfHandle::from_u64(value)
}

fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

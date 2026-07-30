use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandle, DxfHandleAllocationOutcome,
    DxfHandleAllocationPolicyDirectory, DxfHandleAllocationPolicyState,
    DxfHandleAllocationProposal, DxfHandleParseIssue, DxfHandseedState, DxfMemorySource,
    DxfReadOptions, DxfResource, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_has_ascii_binary_allocation_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(
            version.code(),
            "10",
            &[
                (0, "FIRST"),
                (5, "1"),
                (0, "SECOND"),
                (5, "A"),
                (0, "THIRD"),
                (5, "F"),
            ],
        );
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        assert_ready_proposal(
            &ascii.handle_allocation_policy_directory(&DxfCancellationToken::default())?,
        )?;

        let binary_bytes = binary_fixture(
            version,
            "10",
            &[
                (0, "FIRST"),
                (5, "1"),
                (0, "SECOND"),
                (5, "A"),
                (0, "THIRD"),
                (5, "F"),
            ],
        )?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_ready_proposal(
            &binary.handle_allocation_policy_directory(&DxfCancellationToken::default())?,
        )?;
    }
    Ok(())
}

#[test]
fn malformed_identity_and_handseed_evidence_fail_closed() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            ascii_without_handseed(&[(0, "ITEM"), (5, "1")]),
            DxfHandleAllocationPolicyState::HandseedUnavailable {
                state: DxfHandseedState::Absent,
            },
        ),
        (
            ascii_fixture("AC1032", "0x10", &[(0, "ITEM"), (5, "1")]),
            DxfHandleAllocationPolicyState::HandseedUnavailable {
                state: DxfHandseedState::Invalid,
            },
        ),
        (
            ascii_with_duplicate_handseed(),
            DxfHandleAllocationPolicyState::HandseedUnavailable {
                state: DxfHandseedState::Ambiguous,
            },
        ),
        (
            ascii_fixture("AC1032", "0", &[]),
            DxfHandleAllocationPolicyState::NullHandseed,
        ),
        (
            ascii_fixture("AC1032", "10", &[(0, "ITEM"), (5, "0x1")]),
            DxfHandleAllocationPolicyState::IdentityInvalid {
                record_ordinal: 0,
                issue: DxfHandleParseIssue::InvalidDigit { offset: 1 },
            },
        ),
        (
            ascii_fixture("AC1032", "10", &[(0, "ITEM"), (5, "1"), (105, "2")]),
            DxfHandleAllocationPolicyState::IdentityMultiple {
                record_ordinal: 0,
                candidate_count: 2,
            },
        ),
        (
            ascii_fixture("AC1032", "10", &[(0, "ITEM"), (5, "0")]),
            DxfHandleAllocationPolicyState::NullIdentity { record_ordinal: 0 },
        ),
        (
            ascii_fixture(
                "AC1032",
                "10",
                &[(0, "FIRST"), (5, "A"), (0, "SECOND"), (5, "a")],
            ),
            DxfHandleAllocationPolicyState::DuplicateIdentity {
                handle: DxfHandle::from_u64(0xA),
                record_count: 2,
            },
        ),
        (
            ascii_fixture("AC1032", "A", &[(0, "ITEM"), (5, "F")]),
            DxfHandleAllocationPolicyState::HandseedNotAboveOccupied {
                handseed: DxfHandle::from_u64(0xA),
                greatest_occupied: DxfHandle::from_u64(0xF),
            },
        ),
    ];

    for (bytes, expected) in cases {
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let directory =
            document.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
        assert_eq!(directory.state(), expected);
        assert_eq!(
            directory.propose_allocation(1, DxfResourceProfile::Safe)?,
            DxfHandleAllocationOutcome::Unavailable { state: expected }
        );
    }

    let bytes = ascii_fixture("AC1032", "1", &[]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    assert_eq!(
        document
            .handle_allocation_policy_directory(&DxfCancellationToken::default())?
            .state(),
        DxfHandleAllocationPolicyState::Ready
    );
    Ok(())
}

#[test]
fn proposals_are_bounded_constant_space_and_detect_exhaustion() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032", "FFFFFFFFFFFFFFFF", &[]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_allocation_policy_directory(&DxfCancellationToken::default())?;

    let zero = proposed(directory.propose_allocation(0, DxfResourceProfile::Safe)?)?;
    assert_eq!(zero.first_handle(), DxfHandle::from_u64(u64::MAX));
    assert_eq!(zero.handle_count(), 0);
    assert_eq!(zero.next_handseed(), DxfHandle::from_u64(u64::MAX));
    assert_eq!(zero.handle_at(0), None);
    assert_eq!(
        directory.propose_allocation(1, DxfResourceProfile::Safe)?,
        DxfHandleAllocationOutcome::Exhausted {
            handseed: DxfHandle::from_u64(u64::MAX),
            requested_count: 1,
        }
    );

    let observed = DxfResourceProfile::Safe.limits().max_records() + 1;
    assert!(matches!(
        directory.propose_allocation(observed, DxfResourceProfile::Safe),
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::Records,
            limit: 5_000_000,
            observed: actual,
        }) if actual == observed
    ));
    assert!(std::mem::size_of::<DxfHandleAllocationProposal>() <= 64);
    Ok(())
}

#[test]
fn policy_is_cancellable_source_anchored_and_publicly_bounded() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfHandleAllocationPolicyState>();
    assert_copy::<DxfHandleAllocationProposal>();
    assert_copy::<DxfHandleAllocationOutcome>();
    assert_send_sync::<DxfHandleAllocationPolicyDirectory>();

    let bytes = ascii_fixture("AC1032", "10", &[(0, "ITEM"), (5, "F")]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.identity_directory().source_id(),
        document.source_id()
    );
    assert_eq!(directory.handseed_state(), DxfHandseedState::Parsed);
    assert!(directory.handseed_occurrence().is_some());
    assert_eq!(directory.handseed(), Some(DxfHandle::from_u64(0x10)));
    assert_eq!(
        directory.greatest_occupied(),
        Some(DxfHandle::from_u64(0xF))
    );

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.handle_allocation_policy_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn assert_ready_proposal(
    directory: &DxfHandleAllocationPolicyDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.state(), DxfHandleAllocationPolicyState::Ready);
    assert_eq!(directory.handseed(), Some(DxfHandle::from_u64(0x10)));
    assert_eq!(
        directory.greatest_occupied(),
        Some(DxfHandle::from_u64(0xF))
    );
    let proposal = proposed(directory.propose_allocation(3, DxfResourceProfile::Safe)?)?;
    assert_eq!(proposal.source_id(), directory.source_id());
    assert_eq!(proposal.first_handle(), DxfHandle::from_u64(0x10));
    assert_eq!(proposal.handle_count(), 3);
    assert_eq!(proposal.handle_at(0), Some(DxfHandle::from_u64(0x10)));
    assert_eq!(proposal.handle_at(1), Some(DxfHandle::from_u64(0x11)));
    assert_eq!(proposal.handle_at(2), Some(DxfHandle::from_u64(0x12)));
    assert_eq!(proposal.handle_at(3), None);
    assert_eq!(proposal.next_handseed(), DxfHandle::from_u64(0x13));
    Ok(())
}

fn proposed(outcome: DxfHandleAllocationOutcome) -> Result<DxfHandleAllocationProposal, io::Error> {
    match outcome {
        DxfHandleAllocationOutcome::Proposed(proposal) => Ok(proposal),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn ascii_fixture(version: &str, handseed: &str, object_groups: &[(i16, &str)]) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n9\n$HANDSEED\n5\n{handseed}\n0\nENDSEC\n"
    );
    push_ascii_section(&mut text, "OBJECTS", object_groups);
    text.push_str("0\nEOF\n");
    text.into_bytes()
}

fn ascii_without_handseed(object_groups: &[(i16, &str)]) -> Vec<u8> {
    let mut text = "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n".to_owned();
    push_ascii_section(&mut text, "OBJECTS", object_groups);
    text.push_str("0\nEOF\n");
    text.into_bytes()
}

fn ascii_with_duplicate_handseed() -> Vec<u8> {
    b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$HANDSEED\n5\n10\n9\n$HANDSEED\n5\n11\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nENDSEC\n0\nEOF\n".to_vec()
}

fn push_ascii_section(text: &mut String, name: &str, groups: &[(i16, &str)]) {
    text.push_str(&format!("0\nSECTION\n2\n{name}\n"));
    for (code, value) in groups {
        text.push_str(&format!("{code}\n{value}\n"));
    }
    text.push_str("0\nENDSEC\n");
}

fn binary_fixture(
    version: DxfAcadVersion,
    handseed: &str,
    object_groups: &[(i16, &str)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary_section(
        &mut bytes,
        version,
        "HEADER",
        &[
            (9, "$ACADVER"),
            (1, version.code()),
            (9, "$HANDSEED"),
            (5, handseed),
        ],
    )?;
    push_binary_section(&mut bytes, version, "OBJECTS", object_groups)?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_section(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &str,
    groups: &[(i16, &str)],
) -> Result<(), io::Error> {
    for (code, value) in [(0, "SECTION"), (2, name)] {
        push_binary_string(bytes, version, code, value.as_bytes())?;
    }
    for (code, value) in groups {
        push_binary_string(bytes, version, *code, value.as_bytes())?;
    }
    push_binary_string(bytes, version, 0, b"ENDSEC")
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    group_code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(group_code).map_err(|_| io::Error::other("group code"))?);
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

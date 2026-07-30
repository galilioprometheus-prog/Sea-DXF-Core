use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHandle, DxfHandleAllocationPolicyState,
    DxfHandleAssignmentPlan, DxfHandleAssignmentPlanOutcome, DxfHandleAssignmentTargetState,
    DxfHandleIdentityState, DxfMemorySource, DxfRawDocumentView, DxfRawRecordSectionKind,
    DxfReadOptions, DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_supported_version_plans_reopenable_ascii_binary_assignments() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(
            version.code(),
            "10",
            "OBJECTS",
            &[
                (0, "EXISTING"),
                (5, "F"),
                (0, "FIRST"),
                (1, "A"),
                (0, "SECOND"),
                (1, "B"),
            ],
        );
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_modified = plan_and_materialize(DxfRawDocumentView::from(&ascii), &ascii_bytes)?;
        assert_modified_ascii(&ascii_modified)?;

        let binary_bytes = binary_fixture(
            version,
            "10",
            "OBJECTS",
            &[
                (0, "EXISTING"),
                (5, "F"),
                (0, "FIRST"),
                (1, "A"),
                (0, "SECOND"),
                (1, "B"),
            ],
        )?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_modified =
            plan_and_materialize(DxfRawDocumentView::from(&binary), &binary_bytes)?;
        assert_modified_binary(&binary_modified)?;
    }
    Ok(())
}

#[test]
fn table_assignments_use_group_105_only_for_dimstyle_entries() -> Result<(), Box<dyn Error>> {
    let lf_bytes = ascii_fixture(
        "AC1032",
        "1",
        "TABLES",
        &[
            (0, "TABLE"),
            (2, "DIMSTYLE"),
            (0, "DIMSTYLE"),
            (2, "Standard"),
            (0, "ENDTAB"),
        ],
    );
    let bytes = String::from_utf8(lf_bytes)?
        .replace('\n', "\r\n")
        .into_bytes();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let policy = document.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    let assignment = planned(document.plan_handle_assignments(
        &policy,
        &[0, 1],
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?)?;
    let modified = materialize(&bytes, assignment.transaction())?;
    let text = std::str::from_utf8(&modified)?;
    assert!(text.contains("0\r\nTABLE\r\n2\r\nDIMSTYLE\r\n5\r\n1\r\n"));
    assert!(text.contains("0\r\nDIMSTYLE\r\n2\r\nStandard\r\n105\r\n2\r\n"));

    let modified_source = DxfMemorySource::new(&modified, DxfResourceProfile::Safe)?;
    let modified_document = open_ascii(&modified_source)?;
    let identities =
        modified_document.handle_identity_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        identities
            .candidates_for_record(0)
            .map(|values| values[0].group().group_code().value()),
        Some(5)
    );
    assert_eq!(
        identities
            .candidates_for_record(1)
            .map(|values| values[0].group().group_code().value()),
        Some(105)
    );
    assert_eq!(
        modified_document
            .handle_allocation_policy_directory(&DxfCancellationToken::default())?
            .state(),
        DxfHandleAllocationPolicyState::Ready
    );
    Ok(())
}

#[test]
fn target_failures_are_typed_and_no_partial_plan_escapes() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_mixed_target_fixture();
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let policy = document.handle_allocation_policy_directory(&DxfCancellationToken::default())?;

    assert_target_state(
        document.plan_handle_assignments(
            &policy,
            &[3, 3],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        )?,
        DxfHandleAssignmentTargetState::DuplicateTarget { record_ordinal: 3 },
    )?;
    assert_target_state(
        document.plan_handle_assignments(
            &policy,
            &[99],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        )?,
        DxfHandleAssignmentTargetState::MissingRecord { record_ordinal: 99 },
    )?;
    assert_target_state(
        document.plan_handle_assignments(
            &policy,
            &[4],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        )?,
        DxfHandleAssignmentTargetState::IdentityAlreadyPresent {
            record_ordinal: 4,
            state: DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(1)),
        },
    )?;
    assert_target_state(
        document.plan_handle_assignments(
            &policy,
            &[0],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        )?,
        DxfHandleAssignmentTargetState::UnsupportedSection {
            record_ordinal: 0,
            section: DxfRawRecordSectionKind::Classes,
        },
    )?;
    assert_target_state(
        document.plan_handle_assignments(
            &policy,
            &[1],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        )?,
        DxfHandleAssignmentTargetState::TableNameUnavailable { record_ordinal: 1 },
    )?;
    assert_target_state(
        document.plan_handle_assignments(
            &policy,
            &[2],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        )?,
        DxfHandleAssignmentTargetState::TableBoundary { record_ordinal: 2 },
    )?;

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.plan_handle_assignments(&policy, &[3], DxfResourceProfile::Safe, &cancellation,),
        Err(DxfError::Cancelled)
    ));

    let other_bytes = ascii_fixture("AC1032", "2", "OBJECTS", &[(0, "OTHER")]);
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    let other_policy = other.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        document.plan_handle_assignments(
            &other_policy,
            &[3],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn policy_unavailability_empty_plans_and_public_bounds_remain_explicit()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfHandleAssignmentTargetState>();
    assert_send_sync::<DxfHandleAssignmentPlan>();

    let stale_bytes = ascii_fixture(
        "AC1032",
        "1",
        "OBJECTS",
        &[(0, "EXISTING"), (5, "F"), (0, "TARGET")],
    );
    let stale_source = DxfMemorySource::new(&stale_bytes, DxfResourceProfile::Safe)?;
    let stale = open_ascii(&stale_source)?;
    let stale_policy = stale.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        stale.plan_handle_assignments(
            &stale_policy,
            &[1],
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
        )?,
        DxfHandleAssignmentPlanOutcome::PolicyUnavailable {
            state: DxfHandleAllocationPolicyState::HandseedNotAboveOccupied { .. }
        }
    ));

    let bytes = ascii_fixture("AC1032", "1", "OBJECTS", &[(0, "TARGET")]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let policy = document.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    let assignment = planned(document.plan_handle_assignments(
        &policy,
        &[],
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?)?;
    assert_eq!(assignment.allocation().handle_count(), 0);
    assert!(assignment.transaction().patches().is_empty());
    assert_eq!(
        assignment.transaction().projected_len(),
        assignment.transaction().source_len()
    );
    assert!(!format!("{assignment:?}").contains("$HANDSEED"));
    Ok(())
}

fn plan_and_materialize(
    view: DxfRawDocumentView<'_>,
    bytes: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let policy = view.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    let assignment = planned(view.plan_handle_assignments(
        &policy,
        &[2, 1],
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
    )?)?;
    assert_eq!(assignment.source_id(), view.source_id());
    assert_eq!(
        assignment.allocation().handle_at(0),
        Some(DxfHandle::from_u64(0x10))
    );
    assert_eq!(
        assignment.allocation().handle_at(1),
        Some(DxfHandle::from_u64(0x11))
    );
    assert_eq!(
        assignment.allocation().next_handseed(),
        DxfHandle::from_u64(0x12)
    );
    assert_eq!(assignment.transaction().patches().len(), 3);
    Ok(materialize(bytes, assignment.transaction())?)
}

fn assert_modified_ascii(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    assert_modified(DxfRawDocumentView::from(&document))
}

fn assert_modified_binary(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    assert_modified(DxfRawDocumentView::from(&document))
}

fn assert_modified(view: DxfRawDocumentView<'_>) -> Result<(), Box<dyn Error>> {
    let policy = view.handle_allocation_policy_directory(&DxfCancellationToken::default())?;
    assert_eq!(policy.state(), DxfHandleAllocationPolicyState::Ready);
    assert_eq!(policy.handseed(), Some(DxfHandle::from_u64(0x12)));
    assert_eq!(policy.greatest_occupied(), Some(DxfHandle::from_u64(0x11)));
    assert_eq!(
        policy
            .identity_directory()
            .entry(1)
            .map(|entry| entry.state()),
        Some(DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(
            0x11
        )))
    );
    assert_eq!(
        policy
            .identity_directory()
            .entry(2)
            .map(|entry| entry.state()),
        Some(DxfHandleIdentityState::UniqueParsed(DxfHandle::from_u64(
            0x10
        )))
    );
    Ok(())
}

fn materialize(bytes: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::with_capacity(
        usize::try_from(plan.projected_len()).map_err(|_| io::Error::other("projected length"))?,
    );
    let mut source_offset = 0_usize;
    for patch in plan.patches() {
        let start = usize::try_from(patch.source_span().start())
            .map_err(|_| io::Error::other("patch start"))?;
        let end = usize::try_from(patch.source_span().end())
            .map_err(|_| io::Error::other("patch end"))?;
        output.extend_from_slice(
            bytes
                .get(source_offset..start)
                .ok_or_else(|| io::Error::other("source range"))?,
        );
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or_else(|| io::Error::other("replacement"))?,
        );
        source_offset = end;
    }
    output.extend_from_slice(
        bytes
            .get(source_offset..)
            .ok_or_else(|| io::Error::other("source tail"))?,
    );
    Ok(output)
}

fn planned(outcome: DxfHandleAssignmentPlanOutcome) -> Result<DxfHandleAssignmentPlan, io::Error> {
    match outcome {
        DxfHandleAssignmentPlanOutcome::Planned(plan) => Ok(plan),
        other => Err(io::Error::other(format!("unexpected outcome: {other:?}"))),
    }
}

fn assert_target_state(
    outcome: DxfHandleAssignmentPlanOutcome,
    expected: DxfHandleAssignmentTargetState,
) -> Result<(), io::Error> {
    match outcome {
        DxfHandleAssignmentPlanOutcome::TargetsUnavailable { state } if state == expected => Ok(()),
        other => Err(io::Error::other(format!(
            "unexpected outcome: {other:?}, expected {expected:?}"
        ))),
    }
}

fn ascii_fixture(version: &str, handseed: &str, section: &str, groups: &[(i16, &str)]) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n9\n$HANDSEED\n5\n{handseed}\n0\nENDSEC\n0\nSECTION\n2\n{section}\n"
    );
    for (code, value) in groups {
        text.push_str(&format!("{code}\n{value}\n"));
    }
    text.push_str("0\nENDSEC\n0\nEOF\n");
    text.into_bytes()
}

fn ascii_mixed_target_fixture() -> Vec<u8> {
    b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n9\n$HANDSEED\n5\n10\n0\nENDSEC\n0\nSECTION\n2\nCLASSES\n0\nCLASS\n1\nC\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nLAYER\n70\n0\n0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nTARGET\n1\nA\n0\nEXISTING\n5\n1\n0\nENDSEC\n0\nEOF\n"
        .to_vec()
}

fn binary_fixture(
    version: DxfAcadVersion,
    handseed: &str,
    section: &str,
    groups: &[(i16, &str)],
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
    push_binary_section(&mut bytes, version, section, groups)?;
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
    code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
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

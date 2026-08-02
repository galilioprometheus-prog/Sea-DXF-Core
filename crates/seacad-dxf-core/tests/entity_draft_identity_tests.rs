use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityDraftIdentityIssue, DxfEntityDraftIdentityPlan,
    DxfEntityDraftName, DxfEntityNameClassification, DxfEntityPlacementOwnerBinding,
    DxfEntityPlacementOwnerOutcome, DxfError, DxfHandle, DxfHandleReservationPlan,
    DxfHandleReservationPlanOutcome, DxfHandseedState, DxfHandseedValue, DxfMemorySource,
    DxfNamedSymbolTableKind, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver, dxf_entity_aliases,
    dxf_entity_topics,
};

#[test]
fn draft_names_cover_all_45_topics_and_14_aliases_without_unknown_escape() {
    assert_eq!(dxf_entity_topics().len(), 45);
    assert_eq!(dxf_entity_aliases().len(), 14);
    for descriptor in dxf_entity_topics() {
        let name = DxfEntityDraftName::canonical(descriptor.topic());
        assert_eq!(
            name.classification(),
            DxfEntityNameClassification::Canonical(descriptor.topic())
        );
        assert_eq!(name.canonical_topic(), Some(descriptor.topic()));
        assert_eq!(name.exact_name(), Some(descriptor.dxf_name()));
        assert_eq!(
            DxfEntityDraftName::from_classification(name.classification()),
            Some(name)
        );
    }
    for descriptor in dxf_entity_aliases() {
        let name = DxfEntityDraftName::alias(descriptor.alias());
        assert_eq!(
            name.classification(),
            DxfEntityNameClassification::Alias(descriptor.alias())
        );
        assert_eq!(name.canonical_topic(), Some(descriptor.topic()));
        assert_eq!(name.exact_name(), Some(descriptor.dxf_name()));
        assert_eq!(
            DxfEntityDraftName::from_classification(name.classification()),
            Some(name)
        );
    }
    assert_eq!(
        DxfEntityDraftName::from_classification(DxfEntityNameClassification::Unknown),
        None
    );
}

#[test]
fn every_registered_name_prepares_on_all_ascii_binary_dialects_with_exact_inverse()
-> Result<(), Box<dyn Error>> {
    let names = draft_names();
    assert_eq!(names.len(), 59);
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let binding = owner_binding(view)?;
            let policy = view.handle_allocation_policy_directory(&token())?;

            for (index, name) in names.iter().copied().enumerate() {
                let reservation = reserve(view, &policy, 1)?;
                let plan = prepared(view.prepare_entity_draft_identity(
                    name,
                    binding,
                    reservation,
                    &token(),
                )?)?;
                assert_identity(&plan, view.source_id(), name, binding);
                if index == 0 {
                    let output = materialize(&bytes, plan.transaction())?;
                    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
                    let post_document = open_document(&output_source, format)?;
                    let post = post_document.view();
                    assert_handseed(post, 0x41);
                    let inverse = plan.transaction().materialize_inverse_plan(
                        view,
                        post,
                        DxfResourceProfile::Safe,
                        &token(),
                    )?;
                    assert_eq!(materialize(&output, &inverse)?, bytes);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn zero_and_multi_handle_reservations_are_rejected_before_draft_encoding()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let binding = owner_binding(view)?;
    let policy = view.handle_allocation_policy_directory(&token())?;
    let name = DxfEntityDraftName::canonical(seacad_dxf_core::DxfEntityTopic::POINT);

    for handle_count in [0_u64, 2] {
        let outcome = view.prepare_entity_draft_identity(
            name,
            binding,
            reserve(view, &policy, handle_count)?,
            &token(),
        )?;
        assert!(matches!(
            outcome,
            Err(DxfEntityDraftIdentityIssue::ReservationCardinality {
                handle_count: observed
            }) if observed == handle_count
        ));
    }
    Ok(())
}

#[test]
fn source_identity_cancellation_and_public_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfEntityDraftName>();
    assert_copy::<DxfEntityDraftIdentityIssue>();
    assert_send_sync::<DxfEntityDraftIdentityPlan>();
    assert!(std::mem::size_of::<DxfEntityDraftName>() <= 2);
    assert!(std::mem::size_of::<DxfEntityDraftIdentityIssue>() <= 16);

    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let binding = owner_binding(view)?;
    let policy = view.handle_allocation_policy_directory(&token())?;
    let name = DxfEntityDraftName::canonical(seacad_dxf_core::DxfEntityTopic::LINE);

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1027)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other_view = DxfRawDocumentView::from(&other_document);
    let other_binding = owner_binding(other_view)?;
    let other_policy = other_view.handle_allocation_policy_directory(&token())?;

    assert!(matches!(
        view.prepare_entity_draft_identity(
            name,
            other_binding,
            reserve(view, &policy, 1)?,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { expected, observed })
            if expected == view.source_id() && observed == other_view.source_id()
    ));
    assert!(matches!(
        view.prepare_entity_draft_identity(
            name,
            binding,
            reserve(other_view, &other_policy, 1)?,
            &token()
        ),
        Err(DxfError::SourceIdentityMismatch { expected, observed })
            if expected == view.source_id() && observed == other_view.source_id()
    ));

    let cancellation = token();
    cancellation.cancel();
    assert!(matches!(
        view.prepare_entity_draft_identity(
            name,
            binding,
            reserve(view, &policy, 1)?,
            &cancellation
        ),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

fn draft_names() -> Vec<DxfEntityDraftName> {
    dxf_entity_topics()
        .iter()
        .map(|descriptor| DxfEntityDraftName::canonical(descriptor.topic()))
        .chain(
            dxf_entity_aliases()
                .iter()
                .map(|descriptor| DxfEntityDraftName::alias(descriptor.alias())),
        )
        .collect()
}

fn assert_identity(
    plan: &DxfEntityDraftIdentityPlan,
    source_id: seacad_dxf_core::DxfSourceId,
    name: DxfEntityDraftName,
    binding: DxfEntityPlacementOwnerBinding,
) {
    assert_eq!(plan.source_id(), source_id);
    assert_eq!(plan.name(), name);
    assert_eq!(plan.binding(), binding);
    assert_eq!(plan.placement(), binding.placement());
    assert_eq!(plan.handle(), handle(0x40));
    assert_eq!(plan.owner_handle(), handle(0x10));
    assert_eq!(
        plan.owner_block_record().kind(),
        DxfNamedSymbolTableKind::BlockRecord
    );
    assert_eq!(plan.allocation().handle_count(), 1);
    assert_eq!(plan.reservation().allocation(), plan.allocation());
    assert_eq!(plan.transaction().patches().len(), 1);
}

fn owner_binding(
    view: DxfRawDocumentView<'_>,
) -> Result<DxfEntityPlacementOwnerBinding, Box<dyn Error>> {
    let directory = view.entity_placement_owner_directory(&token())?;
    let [assessment] = directory.placement_directory().assessments() else {
        return Err(io::Error::other("one placement").into());
    };
    let placement = assessment
        .placement()
        .ok_or_else(|| io::Error::other("ready placement"))?;
    match directory.bind(placement, handle(0x10), &token())? {
        DxfEntityPlacementOwnerOutcome::Bound(binding) => Ok(binding),
        DxfEntityPlacementOwnerOutcome::Rejected(_) => {
            Err(io::Error::other("owner binding").into())
        }
        _ => Err(io::Error::other("unknown owner outcome").into()),
    }
}

fn reserve(
    view: DxfRawDocumentView<'_>,
    policy: &seacad_dxf_core::DxfHandleAllocationPolicyDirectory,
    handle_count: u64,
) -> Result<DxfHandleReservationPlan, Box<dyn Error>> {
    match view.plan_handle_reservation(policy, handle_count, DxfResourceProfile::Safe, &token())? {
        DxfHandleReservationPlanOutcome::Planned(plan) => Ok(plan),
        DxfHandleReservationPlanOutcome::PolicyUnavailable { .. }
        | DxfHandleReservationPlanOutcome::Exhausted { .. } => {
            Err(io::Error::other("reservation").into())
        }
        _ => Err(io::Error::other("unknown reservation outcome").into()),
    }
}

fn prepared(
    outcome: Result<DxfEntityDraftIdentityPlan, DxfEntityDraftIdentityIssue>,
) -> Result<DxfEntityDraftIdentityPlan, io::Error> {
    match outcome {
        Ok(plan) => Ok(plan),
        Err(_) => Err(io::Error::other("draft identity")),
    }
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let groups = [
        (0_i16, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version.code()),
        (9, "$HANDSEED"),
        (5, "40"),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "TABLES"),
        (0, "TABLE"),
        (2, "BLOCK_RECORD"),
        (0, "BLOCK_RECORD"),
        (5, "10"),
        (2, "*Model_Space"),
        (0, "ENDTAB"),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "ENTITIES"),
        (0, "ENDSEC"),
        (0, "EOF"),
    ];
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_document(&groups)),
        DxfRawDocumentFormat::Binary => binary_document(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_document(groups: &[(i16, &str)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

fn binary_document(version: DxfAcadVersion, groups: &[(i16, &str)]) -> Result<Vec<u8>, io::Error> {
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

fn assert_handseed(view: DxfRawDocumentView<'_>, expected: u64) {
    assert_eq!(view.handseed_report().state(), DxfHandseedState::Parsed);
    assert!(matches!(
        view.handseed_report().primary_occurrence().map(|entry| entry.value()),
        Some(DxfHandseedValue::Parsed(value)) if value == handle(expected)
    ));
}

fn materialize(source: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, DxfError> {
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

const fn handle(value: u64) -> DxfHandle {
    DxfHandle::from_u64(value)
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

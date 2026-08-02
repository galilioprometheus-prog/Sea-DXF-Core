use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAcadVersionState, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfEntityAlias,
    DxfEntityApplicability, DxfEntityApplicabilityEvidence, DxfEntityDraftApplicabilityIssue,
    DxfEntityDraftApplicabilityPlan, DxfEntityDraftIdentityIssue, DxfEntityDraftIdentityPlan,
    DxfEntityDraftName, DxfEntityPlacementOwnerBinding, DxfEntityPlacementOwnerOutcome,
    DxfEntityTopic, DxfError, DxfHandle, DxfHandleReservationPlan, DxfHandleReservationPlanOutcome,
    DxfMemorySource, DxfNamedSymbolTableKind, DxfRawDocumentFormat, DxfRawDocumentView,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver, dxf_entity_aliases, dxf_entity_topics,
};

#[test]
fn all_59_names_follow_reviewed_matrix_on_every_ascii_binary_dialect() -> Result<(), Box<dyn Error>>
{
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

            for name in names.iter().copied() {
                let identity = draft_identity(view, name, binding, &policy)?;
                let outcome = view.prepare_entity_draft_applicability(identity, &token())?;
                assert_outcome(outcome, name, version, view.source_id())?;
            }
        }
    }
    Ok(())
}

#[test]
fn unavailable_version_states_are_typed_before_applicability_lookup() -> Result<(), Box<dyn Error>>
{
    let cases: &[(&[(i16, &str)], DxfAcadVersionState)] = &[
        (&[], DxfAcadVersionState::Absent),
        (
            &[(9, "$ACADVER"), (1, "AC1006")],
            DxfAcadVersionState::Unsupported,
        ),
        (
            &[(9, "$ACADVER"), (70, "1032")],
            DxfAcadVersionState::Invalid,
        ),
        (
            &[
                (9, "$ACADVER"),
                (1, "AC1021"),
                (9, "$ACADVER"),
                (1, "AC1032"),
            ],
            DxfAcadVersionState::Ambiguous,
        ),
    ];
    for (version_groups, expected) in cases {
        let bytes = ascii_fixture(version_groups);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let view = DxfRawDocumentView::from(&document);
        let binding = owner_binding(view)?;
        let policy = view.handle_allocation_policy_directory(&token())?;
        let identity = draft_identity(
            view,
            DxfEntityDraftName::alias(DxfEntityAlias::DGNUNDERLAY),
            binding,
            &policy,
        )?;
        assert!(matches!(
            view.prepare_entity_draft_applicability(identity, &token())?,
            Err(DxfEntityDraftApplicabilityIssue::VersionUnavailable { state })
                if state == *expected
        ));
    }
    Ok(())
}

#[test]
fn source_identity_and_cancellation_fail_before_an_admitted_plan_escapes()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let binding = owner_binding(view)?;
    let policy = view.handle_allocation_policy_directory(&token())?;
    let name = DxfEntityDraftName::alias(DxfEntityAlias::PDFUNDERLAY);

    let other_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1024)?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other_view = DxfRawDocumentView::from(&other_document);
    let foreign = draft_identity(view, name, binding, &policy)?;
    assert!(matches!(
        other_view.prepare_entity_draft_applicability(foreign, &token()),
        Err(DxfError::SourceIdentityMismatch { expected, observed })
            if expected == other_view.source_id() && observed == view.source_id()
    ));

    let cancellation = token();
    cancellation.cancel();
    let cancelled = draft_identity(view, name, binding, &policy)?;
    assert!(matches!(
        view.prepare_entity_draft_applicability(cancelled, &cancellation),
        Err(DxfError::Cancelled)
    ));
    Ok(())
}

#[test]
fn admitted_plan_retains_reviewed_provenance_identity_and_inverse() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfEntityDraftApplicabilityIssue>();
    assert_send_sync::<DxfEntityDraftApplicabilityPlan>();
    assert!(std::mem::size_of::<DxfEntityDraftApplicabilityIssue>() <= 24);

    let bytes = fixture(DxfRawDocumentFormat::Binary, DxfAcadVersion::Ac1024)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let binding = owner_binding(view)?;
    let policy = view.handle_allocation_policy_directory(&token())?;
    let name = DxfEntityDraftName::alias(DxfEntityAlias::PDFUNDERLAY);
    let identity = draft_identity(view, name, binding, &policy)?;
    let plan = admitted(view.prepare_entity_draft_applicability(identity, &token())?)?;

    assert_eq!(plan.source_id(), view.source_id());
    assert_eq!(plan.name(), name);
    assert_eq!(plan.classification(), name.classification());
    assert_eq!(plan.version(), DxfAcadVersion::Ac1024);
    assert_eq!(plan.descriptor().classification(), name.classification());
    assert_eq!(
        plan.descriptor().evidence(),
        DxfEntityApplicabilityEvidence::AutodeskCompatibility
    );
    assert_eq!(
        plan.descriptor().minimum_version(),
        Some(DxfAcadVersion::Ac1024)
    );
    assert_eq!(plan.descriptor().maximum_version(), None);
    assert!(plan.descriptor().source_id().is_some());
    assert!(plan.descriptor().source_reference().is_some());
    assert!(plan.descriptor().source_facts_sha256().is_some());
    assert_eq!(plan.handle(), handle(0x40));
    assert_eq!(
        plan.owner_block_record().kind(),
        DxfNamedSymbolTableKind::BlockRecord
    );

    let output = materialize(&bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_binary(&output_source)?;
    let inverse = plan.transaction().materialize_inverse_plan(
        view,
        DxfRawDocumentView::from(&output_document),
        DxfResourceProfile::Safe,
        &token(),
    )?;
    assert_eq!(materialize(&output, &inverse)?, bytes);
    assert_eq!(plan.into_identity().name(), name);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedApplicability {
    Applicable { minimum: DxfAcadVersion },
    NotApplicable { minimum: DxfAcadVersion },
    NotYetReviewed,
}

fn expected(name: DxfEntityDraftName, version: DxfAcadVersion) -> ExpectedApplicability {
    let minimum = match name {
        DxfEntityDraftName::Canonical(topic) if topic == DxfEntityTopic::MESH => {
            Some(DxfAcadVersion::Ac1024)
        }
        DxfEntityDraftName::Canonical(topic) if topic == DxfEntityTopic::MLEADER => {
            Some(DxfAcadVersion::Ac1021)
        }
        DxfEntityDraftName::Alias(alias)
            if alias == DxfEntityAlias::DGNUNDERLAY || alias == DxfEntityAlias::DWFUNDERLAY =>
        {
            Some(DxfAcadVersion::Ac1021)
        }
        DxfEntityDraftName::Alias(alias) if alias == DxfEntityAlias::PDFUNDERLAY => {
            Some(DxfAcadVersion::Ac1024)
        }
        _ => None,
    };
    match minimum {
        Some(minimum) if version >= minimum => ExpectedApplicability::Applicable { minimum },
        Some(minimum) => ExpectedApplicability::NotApplicable { minimum },
        None => ExpectedApplicability::NotYetReviewed,
    }
}

fn assert_outcome(
    outcome: Result<DxfEntityDraftApplicabilityPlan, DxfEntityDraftApplicabilityIssue>,
    name: DxfEntityDraftName,
    version: DxfAcadVersion,
    source_id: seacad_dxf_core::DxfSourceId,
) -> Result<(), io::Error> {
    match (outcome, expected(name, version)) {
        (Ok(plan), ExpectedApplicability::Applicable { minimum }) => {
            assert_eq!(plan.source_id(), source_id);
            assert_eq!(plan.name(), name);
            assert_eq!(plan.version(), version);
            assert_eq!(plan.descriptor().minimum_version(), Some(minimum));
            assert_eq!(
                plan.descriptor().applicability(version),
                DxfEntityApplicability::Applicable
            );
            Ok(())
        }
        (
            Err(DxfEntityDraftApplicabilityIssue::NotApplicable {
                classification,
                version: observed,
                minimum_version,
                maximum_version,
            }),
            ExpectedApplicability::NotApplicable { minimum },
        ) if classification == name.classification()
            && observed == version
            && minimum_version == Some(minimum)
            && maximum_version.is_none() =>
        {
            Ok(())
        }
        (
            Err(DxfEntityDraftApplicabilityIssue::NotYetReviewed {
                classification,
                version: observed,
            }),
            ExpectedApplicability::NotYetReviewed,
        ) if classification == name.classification() && observed == version => Ok(()),
        _ => Err(io::Error::other("unexpected applicability outcome")),
    }
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

fn draft_identity(
    view: DxfRawDocumentView<'_>,
    name: DxfEntityDraftName,
    binding: DxfEntityPlacementOwnerBinding,
    policy: &seacad_dxf_core::DxfHandleAllocationPolicyDirectory,
) -> Result<DxfEntityDraftIdentityPlan, Box<dyn Error>> {
    let reservation = reserve(view, policy)?;
    match view.prepare_entity_draft_identity(name, binding, reservation, &token())? {
        Ok(plan) => Ok(plan),
        Err(DxfEntityDraftIdentityIssue::ReservationCardinality { .. }) => {
            Err(io::Error::other("draft identity").into())
        }
        _ => Err(io::Error::other("unknown draft identity issue").into()),
    }
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
) -> Result<DxfHandleReservationPlan, Box<dyn Error>> {
    match view.plan_handle_reservation(policy, 1, DxfResourceProfile::Safe, &token())? {
        DxfHandleReservationPlanOutcome::Planned(plan) => Ok(plan),
        DxfHandleReservationPlanOutcome::PolicyUnavailable { .. }
        | DxfHandleReservationPlanOutcome::Exhausted { .. } => {
            Err(io::Error::other("reservation").into())
        }
        _ => Err(io::Error::other("unknown reservation outcome").into()),
    }
}

fn admitted(
    outcome: Result<DxfEntityDraftApplicabilityPlan, DxfEntityDraftApplicabilityIssue>,
) -> Result<DxfEntityDraftApplicabilityPlan, io::Error> {
    outcome.map_err(|_| io::Error::other("applicability"))
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let groups = fixture_groups(&[(9, "$ACADVER"), (1, version.code())]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_document(&groups)),
        DxfRawDocumentFormat::Binary => binary_document(version, &groups),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_fixture(version_groups: &[(i16, &str)]) -> Vec<u8> {
    ascii_document(&fixture_groups(version_groups))
}

fn fixture_groups<'a>(version_groups: &[(i16, &'a str)]) -> Vec<(i16, &'a str)> {
    let mut groups = vec![(0_i16, "SECTION"), (2, "HEADER")];
    groups.extend_from_slice(version_groups);
    groups.extend_from_slice(&[
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
    ]);
    groups
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

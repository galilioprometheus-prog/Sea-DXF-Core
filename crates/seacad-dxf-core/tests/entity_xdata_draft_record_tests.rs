use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityDraft, DxfEntityDraftApplicabilityIssue,
    DxfEntityDraftApplicabilityPlan, DxfEntityDraftIdentityIssue, DxfEntityDraftName,
    DxfEntityDraftRecordPlan, DxfEntityLineweight, DxfEntityPlacementOwnerBinding,
    DxfEntityPlacementOwnerOutcome, DxfEntityTopic, DxfEntityXDataCoordinateTransform,
    DxfEntityXDataDraftRecordIssue, DxfEntityXDataDraftRecordPlan,
    DxfEntityXDataEncodedEntityDestinationDirectory, DxfEntityXDataEncodedEntityDestinationEntry,
    DxfEntityXDataEncodedEntityDestinationState, DxfError, DxfHandleReservationPlan,
    DxfHandleReservationPlanOutcome, DxfMemorySource, DxfPointDraft, DxfRawDocumentFormat,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

const LOCATION: [DxfDouble; 3] = [
    DxfDouble::from_bits(1.0_f64.to_bits()),
    DxfDouble::from_bits(2.0_f64.to_bits()),
    DxfDouble::from_bits(3.0_f64.to_bits()),
];

#[test]
fn ready_xdata_appends_to_drafts_for_every_format_and_dialect() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for source_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            for destination_format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
                assert_format_pair(source_format, version, destination_format, version, 0x40)?;
            }
        }
    }
    assert_format_pair(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        0x40,
    )?;
    assert_format_pair(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        0x40,
    )?;
    Ok(())
}

#[test]
fn zero_xdata_is_an_exact_noop_and_unavailable_payloads_fail_closed() -> Result<(), Box<dyn Error>>
{
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Binary, version, 0x40)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Binary)?;
    let directory = build_directory(source.view(), destination.view())?;

    let zero = find_entry(&directory, |state| {
        matches!(
            state,
            DxfEntityXDataEncodedEntityDestinationState::Ready {
                application_count: 0,
                member_count: 0,
                encoded_byte_count: 0,
            }
        )
    })?;
    let record = draft_record(destination.view(), version)?;
    let original = record.bytes().to_vec();
    let plan = planned(directory.compose_entity_draft_record(
        zero,
        record,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    assert_eq!(plan.bytes(), original);

    let unavailable = find_entry(&directory, |state| {
        matches!(
            state,
            DxfEntityXDataEncodedEntityDestinationState::Unavailable {
                application_count: 0,
                member_count: 0,
                ..
            }
        )
    })?;
    let record = draft_record(destination.view(), version)?;
    assert!(matches!(
        directory.compose_entity_draft_record(
            unavailable,
            record,
            DxfResourceProfile::Safe,
            &token(),
        )?,
        Err(DxfEntityXDataDraftRecordIssue::EncodedPayloadUnavailable { state })
            if state == unavailable.state()
    ));
    Ok(())
}

#[test]
fn draft_composition_is_cancellable_dual_source_bound_and_non_disclosing()
-> Result<(), Box<dyn Error>> {
    assert_send_sync::<DxfEntityXDataDraftRecordPlan>();
    assert_copy::<DxfEntityXDataDraftRecordIssue>();
    let version = DxfAcadVersion::Ac1032;
    let source_bytes = source_fixture(DxfRawDocumentFormat::Ascii, version)?;
    let destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x40)?;
    let other_destination_bytes = destination_fixture(DxfRawDocumentFormat::Ascii, version, 0x50)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let other_storage = DxfMemorySource::new(&other_destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, DxfRawDocumentFormat::Ascii)?;
    let destination = open_document(&destination_storage, DxfRawDocumentFormat::Ascii)?;
    let other_destination = open_document(&other_storage, DxfRawDocumentFormat::Ascii)?;
    let directory = build_directory(source.view(), destination.view())?;
    let other_directory = build_directory(source.view(), other_destination.view())?;
    let ready = ready_entry(&directory)?;
    let foreign = ready_entry(&other_directory)?;

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        directory.compose_entity_draft_record(
            ready,
            draft_record(destination.view(), version)?,
            DxfResourceProfile::Safe,
            &cancelled,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(
        directory
            .compose_entity_draft_record(
                foreign,
                draft_record(destination.view(), version)?,
                DxfResourceProfile::Safe,
                &token(),
            )
            .is_err()
    );
    assert!(matches!(
        directory.compose_entity_draft_record(
            ready,
            draft_record(other_destination.view(), version)?,
            DxfResourceProfile::Safe,
            &token(),
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let record = draft_record(destination.view(), version)?;
    let base_len = record.bytes().len();
    let plan = planned(directory.compose_entity_draft_record(
        ready,
        record,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    let payload = directory
        .encoded_bytes_for_entry(ready)
        .ok_or(io::Error::other("ready payload"))?;
    assert_eq!(plan.source_id(), source.view().source_id());
    assert_eq!(plan.destination_id(), destination.view().source_id());
    assert_eq!(plan.encoded_entry(), ready);
    assert_eq!(plan.encoded_state(), ready.state());
    assert_eq!(plan.source_entity(), directory.entity_for_entry(ready)?);
    assert_eq!(plan.bytes().get(base_len..), Some(payload));
    let debug = format!("{plan:?}");
    assert!(!debug.contains("SECRET_DRAFT_XDATA"));
    assert!(!debug.contains("SECRET_ORPHAN_XDATA"));
    assert_eq!(
        plan.into_draft_record().bytes().get(base_len..),
        Some(payload)
    );
    Ok(())
}

fn assert_format_pair(
    source_format: DxfRawDocumentFormat,
    source_version: DxfAcadVersion,
    destination_format: DxfRawDocumentFormat,
    destination_version: DxfAcadVersion,
    handseed: u64,
) -> Result<(), Box<dyn Error>> {
    let source_bytes = source_fixture(source_format, source_version)?;
    let destination_bytes = destination_fixture(destination_format, destination_version, handseed)?;
    let source_storage = DxfMemorySource::new(&source_bytes, DxfResourceProfile::Safe)?;
    let destination_storage = DxfMemorySource::new(&destination_bytes, DxfResourceProfile::Safe)?;
    let source = open_document(&source_storage, source_format)?;
    let destination = open_document(&destination_storage, destination_format)?;
    let directory = build_directory(source.view(), destination.view())?;
    let entry = ready_entry(&directory)?;
    let payload = directory
        .encoded_bytes_for_entry(entry)
        .ok_or(io::Error::other("encoded payload"))?
        .to_vec();
    let record = draft_record(destination.view(), destination_version)?;
    let base = record.bytes().to_vec();
    let plan = planned(directory.compose_entity_draft_record(
        entry,
        record,
        DxfResourceProfile::Safe,
        &token(),
    )?)?;
    assert_eq!(plan.bytes().get(..base.len()), Some(base.as_slice()));
    assert_eq!(plan.bytes().get(base.len()..), Some(payload.as_slice()));
    Ok(())
}

fn build_directory(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
) -> Result<DxfEntityXDataEncodedEntityDestinationDirectory, Box<dyn Error>> {
    Ok(source.entity_xdata_encoded_entity_destination_directory(
        destination,
        DxfEntityXDataCoordinateTransform::identity(),
        &[],
        DxfResourceProfile::Safe,
        &token(),
    )?)
}

fn ready_entry(
    directory: &DxfEntityXDataEncodedEntityDestinationDirectory,
) -> Result<DxfEntityXDataEncodedEntityDestinationEntry, io::Error> {
    find_entry(directory, |state| {
        matches!(
            state,
            DxfEntityXDataEncodedEntityDestinationState::Ready {
                application_count: 1,
                member_count: 2,
                ..
            }
        )
    })
}

fn find_entry(
    directory: &DxfEntityXDataEncodedEntityDestinationDirectory,
    predicate: impl Fn(DxfEntityXDataEncodedEntityDestinationState) -> bool,
) -> Result<DxfEntityXDataEncodedEntityDestinationEntry, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .find(|entry| predicate(entry.state()))
        .ok_or_else(|| io::Error::other("encoded entity entry"))
}

fn planned(
    outcome: Result<DxfEntityXDataDraftRecordPlan, DxfEntityXDataDraftRecordIssue>,
) -> Result<DxfEntityXDataDraftRecordPlan, io::Error> {
    outcome.map_err(|issue| io::Error::other(format!("{issue:?}")))
}

fn draft_record(
    view: DxfRawDocumentView<'_>,
    version: DxfAcadVersion,
) -> Result<DxfEntityDraftRecordPlan, Box<dyn Error>> {
    let applicability = admitted_plan(view)?;
    let draft = DxfPointDraft::new(b"Layer0", LOCATION);
    let draft = if version >= DxfAcadVersion::Ac1015 {
        draft
            .with_layout(b"Model")
            .with_lineweight(DxfEntityLineweight::BY_LAYER)
    } else {
        draft
    };
    match view.encode_entity_draft_record(
        applicability,
        DxfEntityDraft::point(draft),
        DxfResourceProfile::Safe,
        &token(),
    )? {
        Ok(plan) => Ok(plan),
        Err(issue) => Err(io::Error::other(format!("{issue:?}")).into()),
    }
}

fn admitted_plan(
    view: DxfRawDocumentView<'_>,
) -> Result<DxfEntityDraftApplicabilityPlan, Box<dyn Error>> {
    let binding = owner_binding(view)?;
    let policy = view.handle_allocation_policy_directory(&token())?;
    let reservation = reserve(view, &policy)?;
    let name = DxfEntityDraftName::canonical(DxfEntityTopic::POINT);
    let identity = match view.prepare_entity_draft_identity(name, binding, reservation, &token())? {
        Ok(plan) => plan,
        Err(DxfEntityDraftIdentityIssue::ReservationCardinality { .. }) => {
            return Err(io::Error::other("draft identity").into());
        }
        _ => return Err(io::Error::other("draft identity issue").into()),
    };
    match view.prepare_entity_draft_applicability(identity, &token())? {
        Ok(plan) => Ok(plan),
        Err(
            DxfEntityDraftApplicabilityIssue::VersionUnavailable { .. }
            | DxfEntityDraftApplicabilityIssue::NotApplicable { .. }
            | DxfEntityDraftApplicabilityIssue::NotYetReviewed { .. },
        ) => Err(io::Error::other("draft applicability").into()),
        _ => Err(io::Error::other("draft applicability issue").into()),
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
        .ok_or(io::Error::other("ready placement"))?;
    match directory.bind(
        placement,
        seacad_dxf_core::DxfHandle::from_u64(0x10),
        &token(),
    )? {
        DxfEntityPlacementOwnerOutcome::Bound(binding) => Ok(binding),
        DxfEntityPlacementOwnerOutcome::Rejected(_) => {
            Err(io::Error::other("owner binding").into())
        }
        _ => Err(io::Error::other("owner outcome").into()),
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
        _ => Err(io::Error::other("reservation outcome").into()),
    }
}

fn source_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version, 0x30);
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"TABLES"),
        group(0, b"TABLE"),
        group(2, b"APPID"),
        group(0, b"APPID"),
        group(2, b"APP_READY"),
        group(0, b"ENDTAB"),
        group(0, b"ENDSEC"),
        group(0, b"SECTION"),
        group(2, b"ENTITIES"),
        group(0, b"POINT"),
        group(1001, b"APP_READY"),
        group(1000, b"SECRET_DRAFT_XDATA"),
        group(0, b"LINE"),
        group(1000, b"SECRET_ORPHAN_XDATA"),
        group(0, b"ENDSEC"),
        group(0, b"EOF"),
    ]);
    encode_fixture(format, version, &groups)
}

fn destination_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handseed: u64,
) -> Result<Vec<u8>, io::Error> {
    let mut groups = header(version, handseed);
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"TABLES"),
        group(0, b"TABLE"),
        group(2, b"BLOCK_RECORD"),
        group(0, b"BLOCK_RECORD"),
        group(5, b"10"),
        group(2, b"*Model_Space"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"LAYER"),
        group(0, b"LAYER"),
        group(5, b"11"),
        group(2, b"Layer0"),
        group(0, b"ENDTAB"),
        group(0, b"TABLE"),
        group(2, b"APPID"),
        group(0, b"APPID"),
        group(2, b"APP_READY"),
        group(0, b"ENDTAB"),
        group(0, b"ENDSEC"),
    ]);
    if version >= DxfAcadVersion::Ac1015 {
        groups.extend([
            group(0, b"SECTION"),
            group(2, b"OBJECTS"),
            group(0, b"LAYOUT"),
            group(100, b"AcDbPlotSettings"),
            group(1, b"PAGE_SETUP"),
            group(100, b"AcDbLayout"),
            group(1, b"Model"),
            group(0, b"ENDSEC"),
        ]);
    }
    groups.extend([
        group(0, b"SECTION"),
        group(2, b"ENTITIES"),
        group(0, b"ENDSEC"),
        group(0, b"EOF"),
    ]);
    encode_fixture(format, version, &groups)
}

fn header(version: DxfAcadVersion, handseed: u64) -> Vec<(i16, Vec<u8>)> {
    vec![
        group(0, b"SECTION"),
        group(2, b"HEADER"),
        group(9, b"$ACADVER"),
        group(1, version.code().as_bytes()),
        group(9, b"$DWGCODEPAGE"),
        group(3, b"ANSI_1252"),
        group(9, b"$HANDSEED"),
        (5, format!("{handseed:X}").into_bytes()),
        group(0, b"ENDSEC"),
    ]
}

fn group(code: i16, value: &[u8]) -> (i16, Vec<u8>) {
    (code, value.to_vec())
}

fn encode_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, Vec<u8>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = if format == DxfRawDocumentFormat::Binary {
        DXF_BINARY_SENTINEL.to_vec()
    } else {
        Vec::new()
    };
    for (code, value) in groups {
        match format {
            DxfRawDocumentFormat::Ascii => {
                bytes.extend_from_slice(code.to_string().as_bytes());
                bytes.push(b'\n');
                bytes.extend_from_slice(value);
                bytes.push(b'\n');
            }
            DxfRawDocumentFormat::Binary => {
                if version == DxfAcadVersion::Ac1009 && (0..=254).contains(code) {
                    bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("code"))?);
                } else if version == DxfAcadVersion::Ac1009 && (1000..=1071).contains(code) {
                    bytes.push(u8::MAX);
                    bytes.extend_from_slice(&code.to_le_bytes());
                } else {
                    bytes.extend_from_slice(&code.to_le_bytes());
                }
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            _ => return Err(io::Error::other("format")),
        }
    }
    Ok(bytes)
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
    let mut observer = NoopDxfReadObserver;
    match format {
        DxfRawDocumentFormat::Ascii => Ok(OpenedDocument::Ascii(DxfAsciiRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &token(),
            &mut observer,
        )?)),
        DxfRawDocumentFormat::Binary => Ok(OpenedDocument::Binary(DxfBinaryRawDocument::open(
            source,
            DxfReadOptions::strict(),
            &token(),
            &mut observer,
        )?)),
        _ => Err(io::Error::other("format").into()),
    }
}

fn token() -> DxfCancellationToken {
    DxfCancellationToken::default()
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfEntityPlacementState, DxfError, DxfHandle, DxfHandleIdentityLookup,
    DxfHandleReservationPlan, DxfHandleReservationPlanOutcome, DxfHandseedState, DxfHandseedValue,
    DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions, DxfResource,
    DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
};

#[test]
fn every_dialect_reserves_ascii_binary_ranges_and_exact_inverse() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, Some(b"10"))?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let policy = view.handle_allocation_policy_directory(&token())?;
            let reservation = planned(view.plan_handle_reservation(
                &policy,
                3,
                DxfResourceProfile::Safe,
                &token(),
            )?)?;
            assert_eq!(reservation.source_id(), view.source_id());
            assert_eq!(
                reservation.allocation().first_handle(),
                DxfHandle::from_u64(0x10)
            );
            assert_eq!(
                reservation.allocation().handle_at(1),
                Some(DxfHandle::from_u64(0x11))
            );
            assert_eq!(
                reservation.allocation().handle_at(2),
                Some(DxfHandle::from_u64(0x12))
            );
            assert_eq!(reservation.allocation().handle_at(3), None);
            assert_eq!(
                reservation.allocation().next_handseed(),
                DxfHandle::from_u64(0x13)
            );
            assert_eq!(reservation.transaction().patches().len(), 1);

            let output = materialize(&bytes, reservation.transaction())?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let post_document = open_document(&output_source, format)?;
            let post = post_document.view();
            assert_handseed(post, 0x13);
            let inverse = reservation.transaction().materialize_inverse_plan(
                view,
                post,
                DxfResourceProfile::Safe,
                &token(),
            )?;
            assert_eq!(materialize(&output, &inverse)?, bytes);
        }
    }
    Ok(())
}

#[test]
fn reservation_composes_with_every_entity_placement_wire() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, Some(b"10"))?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let policy = view.handle_allocation_policy_directory(&token())?;
            let reservation = planned(view.plan_handle_reservation(
                &policy,
                1,
                DxfResourceProfile::Safe,
                &token(),
            )?)?;
            let placement_directory = view.entity_placement_directory(&token())?;
            let [assessment] = placement_directory.assessments() else {
                return Err(io::Error::other("one ENTITIES placement").into());
            };
            let DxfEntityPlacementState::Ready(placement) = assessment.state() else {
                return Err(io::Error::other("ready placement").into());
            };

            let mut insertion_builder = view.transaction_plan_builder(DxfResourceProfile::Safe)?;
            insertion_builder.replace_raw_span(
                placement.insertion_span(),
                &point_record(format, version, reservation.allocation().first_handle())?,
                &token(),
            )?;
            let insertion = insertion_builder.finish(&token())?;
            let composed = view.compose_transaction_plans(
                &[reservation.transaction(), &insertion],
                DxfResourceProfile::Safe,
                &token(),
            )?;

            let output = materialize(&bytes, &composed)?;
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let post_document = open_document(&output_source, format)?;
            let post = post_document.view();
            assert_handseed(post, 0x11);
            assert!(matches!(
                post.handle_identity_directory(&token())?
                    .lookup(DxfHandle::from_u64(0x10)),
                DxfHandleIdentityLookup::Unique(_)
            ));
            assert_eq!(
                post.raw_record_directory(&token())?.records().len(),
                view.raw_record_directory(&token())?.records().len() + 1
            );
            let inverse = composed.materialize_inverse_plan(
                view,
                post,
                DxfResourceProfile::Safe,
                &token(),
            )?;
            assert_eq!(materialize(&output, &inverse)?, bytes);
        }
    }
    Ok(())
}

#[test]
fn zero_unavailable_exhausted_and_resource_states_remain_typed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        Some(b"10"),
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let policy = view.handle_allocation_policy_directory(&token())?;
    let zero =
        planned(view.plan_handle_reservation(&policy, 0, DxfResourceProfile::Safe, &token())?)?;
    assert_eq!(zero.allocation().handle_count(), 0);
    assert_eq!(zero.allocation().next_handseed(), DxfHandle::from_u64(0x10));
    assert!(zero.transaction().patches().is_empty());

    let observed = DxfResourceProfile::Safe.limits().max_records() + 1;
    assert!(matches!(
        view.plan_handle_reservation(&policy, observed, DxfResourceProfile::Safe, &token()),
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::Records,
            ..
        })
    ));

    let absent_bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, None)?;
    let absent_source = DxfMemorySource::new(&absent_bytes, DxfResourceProfile::Safe)?;
    let absent_document = open_ascii(&absent_source)?;
    let absent_policy = absent_document.handle_allocation_policy_directory(&token())?;
    assert!(matches!(
        absent_document.plan_handle_reservation(
            &absent_policy,
            1,
            DxfResourceProfile::Safe,
            &token()
        )?,
        DxfHandleReservationPlanOutcome::PolicyUnavailable {
            state: seacad_dxf_core::DxfHandleAllocationPolicyState::HandseedUnavailable {
                state: DxfHandseedState::Absent
            }
        }
    ));

    let max_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        Some(b"FFFFFFFFFFFFFFFF"),
    )?;
    let max_source = DxfMemorySource::new(&max_bytes, DxfResourceProfile::Safe)?;
    let max_document = open_ascii(&max_source)?;
    let max_policy = max_document.handle_allocation_policy_directory(&token())?;
    assert!(matches!(
        max_document.plan_handle_reservation(
            &max_policy,
            1,
            DxfResourceProfile::Safe,
            &token()
        )?,
        DxfHandleReservationPlanOutcome::Exhausted {
            handseed,
            requested_count: 1
        } if handseed == DxfHandle::from_u64(u64::MAX)
    ));
    Ok(())
}

#[test]
fn source_identity_cancellation_and_public_bounds_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        Some(b"10"),
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let policy = document.handle_allocation_policy_directory(&token())?;

    let other_bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1027,
        Some(b"20"),
    )?;
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        other.plan_handle_reservation(&policy, 1, DxfResourceProfile::Safe, &token()),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    let cancelled = token();
    cancelled.cancel();
    assert!(matches!(
        document.plan_handle_reservation(&policy, 1, DxfResourceProfile::Safe, &cancelled),
        Err(DxfError::Cancelled)
    ));
    assert_send_sync::<DxfHandleReservationPlan>();
    assert_send_sync::<DxfHandleReservationPlanOutcome>();
    Ok(())
}

fn planned(
    outcome: DxfHandleReservationPlanOutcome,
) -> Result<DxfHandleReservationPlan, io::Error> {
    match outcome {
        DxfHandleReservationPlanOutcome::Planned(plan) => Ok(plan),
        _ => Err(io::Error::other("planned reservation")),
    }
}

fn assert_handseed(view: DxfRawDocumentView<'_>, expected: u64) {
    assert!(matches!(
        view.handseed_report()
            .primary_occurrence()
            .map(|occurrence| occurrence.value()),
        Some(DxfHandseedValue::Parsed(handle)) if handle == DxfHandle::from_u64(expected)
    ));
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handseed: Option<&[u8]>,
) -> Result<Vec<u8>, io::Error> {
    let mut groups: Vec<(i16, &[u8])> = vec![
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
    ];
    if let Some(handseed) = handseed {
        groups.extend([(9, b"$HANDSEED".as_slice()), (5, handseed)]);
    }
    groups.extend([
        (0, b"ENDSEC".as_slice()),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"LINE"),
        (5, b"1"),
        (0, b"ENDSEC"),
        (0, b"EOF"),
    ]);
    encode_groups(format, version, &groups, true)
}

fn point_record(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    handle: DxfHandle,
) -> Result<Vec<u8>, io::Error> {
    let text = format!("{:X}", handle.value());
    encode_groups(
        format,
        version,
        &[(0, b"POINT".as_slice()), (5, text.as_bytes())],
        false,
    )
}

fn encode_groups(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    groups: &[(i16, &[u8])],
    include_binary_sentinel: bool,
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = if format == DxfRawDocumentFormat::Binary && include_binary_sentinel {
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
                if version == DxfAcadVersion::Ac1009 {
                    bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("group code"))?);
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

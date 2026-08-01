use std::{error::Error, io, num::NonZeroU8};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityCommonFieldDomainIssue,
    DxfEntityCommonFieldDomainOutcome, DxfEntityCommonFieldDomainValue, DxfEntityCommonFieldPatch,
    DxfEntityEditIssue, DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityEditValueKind,
    DxfEntityEditVerificationOutcome, DxfEntityField, DxfEntityIndexedColor, DxfEntityLineweight,
    DxfEntityPatch, DxfEntityShadowMode, DxfEntitySpace, DxfEntityTrueColor, DxfEntityVisibility,
    DxfError, DxfMemorySource, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
    classify_entity_common_field_edit_domain,
};

#[test]
fn reviewed_scalar_boundaries_classify_to_typed_domains() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        valid(domain(DxfEntityField::PAPER_SPACE, int16(0)))?,
        DxfEntityCommonFieldDomainValue::Space(DxfEntitySpace::Model)
    );
    assert_eq!(DxfEntitySpace::Paper.raw(), 1);
    assert!(DxfEntitySpace::from_raw(2).is_none());

    for raw in [-255_i16, -1, 0, 1, 255, 256] {
        let DxfEntityCommonFieldDomainValue::IndexedColor(color) =
            valid(domain(DxfEntityField::COLOR, int16(raw)))?
        else {
            return Err(io::Error::other("indexed color shape").into());
        };
        assert_eq!(color.raw(), raw);
    }
    assert_eq!(
        DxfEntityIndexedColor::from_raw(0),
        Some(DxfEntityIndexedColor::ByBlock)
    );
    assert_eq!(
        DxfEntityIndexedColor::from_raw(256),
        Some(DxfEntityIndexedColor::ByLayer)
    );
    assert_eq!(
        DxfEntityIndexedColor::from_raw(-7),
        Some(DxfEntityIndexedColor::Aci {
            index: NonZeroU8::new(7).ok_or(io::Error::other("nonzero"))?,
            layer_off: true,
        })
    );
    assert!(DxfEntityIndexedColor::from_raw(-256).is_none());
    assert!(DxfEntityIndexedColor::from_raw(257).is_none());

    let lineweights = [
        -3_i16, -2, -1, 0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106,
        120, 140, 158, 200, 211,
    ];
    for raw in lineweights {
        let value =
            DxfEntityLineweight::from_raw(raw).ok_or(io::Error::other("reviewed lineweight"))?;
        assert_eq!(value.raw(), raw);
        assert!(matches!(
            domain(DxfEntityField::LINEWEIGHT, int16(raw)),
            DxfEntityCommonFieldDomainOutcome::Valid(
                DxfEntityCommonFieldDomainValue::Lineweight(observed)
            ) if observed == value
        ));
    }
    assert_eq!(DxfEntityLineweight::BY_LINEWEIGHT_DEFAULT.raw(), -3);
    assert_eq!(DxfEntityLineweight::BY_BLOCK.raw(), -2);
    assert_eq!(DxfEntityLineweight::BY_LAYER.raw(), -1);
    for raw in [-4_i16, 1, 4, 6, 212] {
        assert!(DxfEntityLineweight::from_raw(raw).is_none());
    }

    for raw in [0.0_f64, -0.0, 2.5] {
        assert!(matches!(
            domain(
                DxfEntityField::LINETYPE_SCALE,
                DxfEntityEditValue::Double(DxfDouble::from_f64(raw))
            ),
            DxfEntityCommonFieldDomainOutcome::Valid(
                DxfEntityCommonFieldDomainValue::LinetypeScale(_)
            )
        ));
    }
    assert!(matches!(
        domain(
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(-0.01))
        ),
        DxfEntityCommonFieldDomainOutcome::Invalid(
            DxfEntityCommonFieldDomainIssue::InvalidDouble { .. }
        )
    ));
    assert!(matches!(
        domain(
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(f64::NAN))
        ),
        DxfEntityCommonFieldDomainOutcome::Invalid(
            DxfEntityCommonFieldDomainIssue::InvalidDouble { .. }
        )
    ));
    assert!(matches!(
        domain(
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(f64::INFINITY))
        ),
        DxfEntityCommonFieldDomainOutcome::Invalid(
            DxfEntityCommonFieldDomainIssue::InvalidDouble { .. }
        )
    ));

    assert_eq!(DxfEntityVisibility::Visible.raw(), 0);
    assert_eq!(DxfEntityVisibility::Invisible.raw(), 1);
    assert!(DxfEntityVisibility::from_raw(-1).is_none());
    assert!(matches!(
        domain(DxfEntityField::PROXY_GRAPHICS_SIZE, int32(i32::MAX)),
        DxfEntityCommonFieldDomainOutcome::Valid(
            DxfEntityCommonFieldDomainValue::ProxyGraphicsSize(value)
        ) if value == i32::MAX as u32
    ));

    let color = DxfEntityTrueColor::from_raw(0x12_34_56).ok_or(io::Error::other("true color"))?;
    assert_eq!(
        (color.red(), color.green(), color.blue()),
        (0x12, 0x34, 0x56)
    );
    assert_eq!(color.raw(), 0x12_34_56);
    assert!(DxfEntityTrueColor::from_raw(-1).is_none());
    assert!(DxfEntityTrueColor::from_raw(0x01_00_00_00).is_none());

    for (raw, mode) in [
        (0, DxfEntityShadowMode::CastsAndReceives),
        (1, DxfEntityShadowMode::Casts),
        (2, DxfEntityShadowMode::Receives),
        (3, DxfEntityShadowMode::Ignores),
    ] {
        assert_eq!(DxfEntityShadowMode::from_raw(raw), Some(mode));
        assert_eq!(mode.raw(), raw);
    }
    assert!(DxfEntityShadowMode::from_raw(4).is_none());
    Ok(())
}

#[test]
fn invalid_values_and_unreviewed_fields_stay_distinct() -> Result<(), Box<dyn Error>> {
    for (field, value) in [
        (DxfEntityField::PAPER_SPACE, 2_i16),
        (DxfEntityField::COLOR, -256),
        (DxfEntityField::COLOR, 257),
        (DxfEntityField::LINEWEIGHT, -4),
        (DxfEntityField::VISIBILITY, 2),
        (DxfEntityField::SHADOW, 4),
    ] {
        assert_eq!(
            invalid(domain(field, int16(value)))?,
            DxfEntityCommonFieldDomainIssue::UnsupportedInt16 { field, value }
        );
    }
    for (field, value) in [
        (DxfEntityField::PROXY_GRAPHICS_SIZE, -1_i32),
        (DxfEntityField::TRUE_COLOR, -1),
        (DxfEntityField::TRUE_COLOR, 0x01_00_00_00),
    ] {
        assert_eq!(
            invalid(domain(field, int32(value)))?,
            DxfEntityCommonFieldDomainIssue::UnsupportedInt32 { field, value }
        );
    }
    assert_eq!(
        invalid(domain(DxfEntityField::VISIBILITY, int32(1)))?,
        DxfEntityCommonFieldDomainIssue::ValueKindMismatch {
            field: DxfEntityField::VISIBILITY,
            expected: DxfEntityEditValueKind::Int16,
            observed: DxfEntityEditValueKind::Int32,
        }
    );
    assert_eq!(
        domain(
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"UNREVIEWED")
        ),
        DxfEntityCommonFieldDomainOutcome::Unreviewed {
            field: DxfEntityField::LAYER
        }
    );
    assert_copy_send_sync::<DxfEntityCommonFieldDomainOutcome>();
    assert_copy_send_sync::<DxfEntityCommonFieldDomainValue>();
    assert_copy_send_sync::<DxfEntityCommonFieldDomainIssue>();
    Ok(())
}

#[test]
fn edit_session_rejects_invalid_domains_without_queueing() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = evidence.entity_directory().entities()[0].key();
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;

    for (field, value) in [
        (DxfEntityField::PAPER_SPACE, int16(2)),
        (DxfEntityField::COLOR, int16(257)),
        (DxfEntityField::LINEWEIGHT, int16(-4)),
        (
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(-1.0)),
        ),
        (DxfEntityField::VISIBILITY, int16(2)),
        (DxfEntityField::PROXY_GRAPHICS_SIZE, int32(-1)),
        (DxfEntityField::TRUE_COLOR, int32(0x01_00_00_00)),
        (DxfEntityField::SHADOW, int16(4)),
    ] {
        assert!(matches!(
            session.update(key, set(field, value))?,
            DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Domain(_))
        ));
        assert_eq!(session.queued_edit_count(), 0);
    }

    assert!(matches!(
        session.update(key, set(DxfEntityField::COLOR, int16(256)))?,
        DxfEntityEditOutcome::Applied(_)
    ));
    assert_eq!(session.queued_edit_count(), 1);
    assert!(matches!(
        session.update(key, set(DxfEntityField::COLOR, int16(257)))?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::DuplicateFieldEdit { .. })
    ));
    assert_eq!(session.queued_edit_count(), 1);
    Ok(())
}

#[test]
fn every_dialect_accepts_reviewed_ascii_binary_edits_and_verifies_them()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            verify_dialect(format, version)?;
        }
    }
    Ok(())
}

fn verify_dialect(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    let bytes = fixture(format, version)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    match format {
        DxfRawDocumentFormat::Ascii => {
            let document = open_ascii(&source)?;
            verify_document(&bytes, DxfRawDocumentView::from(&document), version)
        }
        DxfRawDocumentFormat::Binary => {
            let document = open_binary(&source)?;
            verify_document(&bytes, DxfRawDocumentView::from(&document), version)
        }
        _ => Err(io::Error::other("test format").into()),
    }
}

fn verify_document(
    bytes: &[u8],
    view: DxfRawDocumentView<'_>,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = evidence.entity_directory().entities()[0].key();
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    let base = [
        (DxfEntityField::PAPER_SPACE, int16(1)),
        (DxfEntityField::COLOR, int16(-255)),
        (
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(0.0)),
        ),
        (DxfEntityField::VISIBILITY, int16(1)),
        (DxfEntityField::PROXY_GRAPHICS_SIZE, int32(0)),
    ];
    for (field, value) in base {
        applied(session.update(key, set(field, value))?)?;
    }
    if version != DxfAcadVersion::Ac1009 {
        for (field, value) in [
            (DxfEntityField::LINEWEIGHT, int16(211)),
            (DxfEntityField::TRUE_COLOR, int32(0x12_34_56)),
            (DxfEntityField::SHADOW, int16(3)),
        ] {
            applied(session.update(key, set(field, value))?)?;
        }
    }
    let expected_count = if version == DxfAcadVersion::Ac1009 {
        5
    } else {
        8
    };
    assert_eq!(session.queued_edit_count(), expected_count);
    let plan = session.finish_verifiable()?;
    let output = materialize(bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let outcome = match view.format() {
        DxfRawDocumentFormat::Ascii => {
            let post = open_ascii(&output_source)?;
            plan.verify_post_image(
                view,
                DxfRawDocumentView::from(&post),
                DxfResourceProfile::Safe,
                &token(),
            )?
        }
        DxfRawDocumentFormat::Binary => {
            let post = open_binary(&output_source)?;
            plan.verify_post_image(
                view,
                DxfRawDocumentView::from(&post),
                DxfResourceProfile::Safe,
                &token(),
            )?
        }
        _ => return Err(io::Error::other("test format").into()),
    };
    let DxfEntityEditVerificationOutcome::Verified(journal) = outcome else {
        return Err(io::Error::other(format!("unexpected verification: {outcome:?}")).into());
    };
    assert_eq!(journal.receipt().edit_count(), expected_count);
    assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
    Ok(())
}

fn domain(
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> DxfEntityCommonFieldDomainOutcome {
    classify_entity_common_field_edit_domain(field, value)
}

fn valid(
    outcome: DxfEntityCommonFieldDomainOutcome,
) -> Result<DxfEntityCommonFieldDomainValue, io::Error> {
    match outcome {
        DxfEntityCommonFieldDomainOutcome::Valid(value) => Ok(value),
        other => Err(io::Error::other(format!("unexpected domain: {other:?}"))),
    }
}

fn invalid(
    outcome: DxfEntityCommonFieldDomainOutcome,
) -> Result<DxfEntityCommonFieldDomainIssue, io::Error> {
    match outcome {
        DxfEntityCommonFieldDomainOutcome::Invalid(issue) => Ok(issue),
        other => Err(io::Error::other(format!("unexpected domain: {other:?}"))),
    }
}

const fn int16(value: i16) -> DxfEntityEditValue<'static> {
    DxfEntityEditValue::Int16(value)
}

const fn int32(value: i32) -> DxfEntityEditValue<'static> {
    DxfEntityEditValue::Int32(value)
}

fn set(field: DxfEntityField, value: DxfEntityEditValue<'_>) -> DxfEntityPatch<'_> {
    DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit { field, value })
}

fn applied(outcome: DxfEntityEditOutcome) -> Result<(), io::Error> {
    match outcome {
        DxfEntityEditOutcome::Applied(_) => Ok(()),
        other => Err(io::Error::other(format!("unexpected edit: {other:?}"))),
    }
}

fn materialize(bytes: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start = usize::try_from(patch.source_span().start())
            .map_err(|_| io::Error::other("patch start"))?;
        let end = usize::try_from(patch.source_span().end())
            .map_err(|_| io::Error::other("patch end"))?;
        output.extend_from_slice(bytes.get(cursor..start).ok_or(io::Error::other("source"))?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or(io::Error::other("replacement"))?,
        );
        cursor = end;
    }
    output.extend_from_slice(bytes.get(cursor..).ok_or(io::Error::other("tail"))?);
    Ok(output)
}

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a [u8]),
    Int16(i16),
    Double(f64),
}

fn fixture(format: DxfRawDocumentFormat, version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut groups = vec![
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"HEADER")),
        (9, Value::Text(b"$ACADVER")),
        (1, Value::Text(version.code().as_bytes())),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"SECTION")),
        (2, Value::Text(b"ENTITIES")),
        (0, Value::Text(b"LINE")),
        (5, Value::Text(b"10")),
    ];
    if version != DxfAcadVersion::Ac1009 {
        groups.extend([(330, Value::Text(b"1F")), (100, Value::Text(b"AcDbEntity"))]);
    }
    groups.extend([(8, Value::Text(b"OLD")), (62, Value::Int16(7))]);
    if version != DxfAcadVersion::Ac1009 {
        groups.push((100, Value::Text(b"AcDbLine")));
    }
    groups.extend([
        (10, Value::Double(0.0)),
        (0, Value::Text(b"ENDSEC")),
        (0, Value::Text(b"EOF")),
    ]);
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_groups(&groups)),
        DxfRawDocumentFormat::Binary => binary_groups(version, &groups),
        _ => Err(io::Error::other("test format")),
    }
}

fn ascii_groups(groups: &[(i16, Value<'_>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (code, value) in groups {
        bytes.extend_from_slice(code.to_string().as_bytes());
        bytes.extend_from_slice(b"\r\n");
        match value {
            Value::Text(value) => bytes.extend_from_slice(value),
            Value::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
            Value::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        }
        bytes.extend_from_slice(b"\r\n");
    }
    bytes
}

fn binary_groups(
    version: DxfAcadVersion,
    groups: &[(i16, Value<'_>)],
) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in groups {
        if version == DxfAcadVersion::Ac1009 {
            bytes.push(u8::try_from(*code).map_err(|_| io::Error::other("AC1009 code"))?);
        } else {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        match value {
            Value::Text(value) => {
                bytes.extend_from_slice(value);
                bytes.push(0);
            }
            Value::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Value::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        }
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

fn assert_copy_send_sync<T: Copy + Send + Sync>() {}

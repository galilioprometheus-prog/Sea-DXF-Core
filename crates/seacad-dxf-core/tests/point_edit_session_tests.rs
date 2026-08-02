use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfEntityCommonFieldPatch, DxfEntityDraft, DxfEntityEditIssue,
    DxfEntityEditOutcome, DxfEntityEditValue, DxfEntityField, DxfEntityFieldEvidenceDirectory,
    DxfEntityPatch, DxfEntityTopic, DxfError, DxfMemorySource, DxfPointDraft, DxfPointEditIssue,
    DxfPointPatch, DxfPointPatchKind, DxfRawDocumentFormat, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfTransactionPlan, NoopDxfReadObserver,
};

const UPDATED: [DxfDouble; 3] = [
    DxfDouble::from_bits(4.0_f64.to_bits()),
    DxfDouble::from_bits(5.0_f64.to_bits()),
    DxfDouble::from_bits(6.0_f64.to_bits()),
];

#[test]
fn point_location_update_is_atomic_across_every_ascii_binary_dialect() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        for format in [DxfRawDocumentFormat::Ascii, DxfRawDocumentFormat::Binary] {
            let bytes = fixture(format, version, PointShape::Complete)?;
            let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
            let document = open_document(&source, format)?;
            let view = document.view();
            let evidence = view.entity_field_evidence_directory(&token())?;
            let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
            let cancellation = token();
            let mut session =
                view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
            let DxfEntityEditOutcome::PointApplied(receipt) = session.update(
                key,
                DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED)),
            )?
            else {
                return Err(io::Error::other("POINT location update").into());
            };
            assert_eq!(receipt.key(), key);
            assert_eq!(receipt.kind(), DxfPointPatchKind::Location);
            assert_eq!(receipt.queued_edit_count(), 1);
            assert_eq!(session.queued_edit_count(), 1);

            let plan = session.finish_verifiable()?;
            assert_eq!(plan.edit_count(), 1);
            assert_eq!(plan.transaction().patches().len(), 3);
            let output = materialize(&bytes, plan.transaction())?;
            assert_eq!(bytes, fixture(format, version, PointShape::Complete)?);
            let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
            let output_document = open_document(&output_source, format)?;
            let post = output_document.view();
            let geometry = post.basic_geometry_semantic_directory(&token())?;
            let point = geometry
                .point_for_raw_record(key.raw_record_ordinal())?
                .ok_or_else(|| io::Error::other("updated POINT semantics"))?;
            assert_eq!(point.location_value(), Some(UPDATED));
            let seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(journal) =
                plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?
            else {
                return Err(io::Error::other("verified POINT update").into());
            };
            assert_eq!(journal.receipt().edit_count(), 1);
            assert_eq!(materialize(&output, journal.inverse_plan())?, bytes);
        }
    }
    Ok(())
}

#[test]
fn point_location_rejects_duplicate_missing_wrong_family_and_nonfinite()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let point_key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let line_key = key_for_topic(&evidence, DxfEntityTopic::LINE)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            line_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::WrongClassification { key, .. }
        )) if key == line_key
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location([
                DxfDouble::from_f64(f64::NAN),
                UPDATED[1],
                UPDATED[2],
            ]))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(DxfPointEditIssue::Encoding {
            group_code: 10,
            ..
        }))
    ));
    assert_eq!(session.queued_edit_count(), 0);
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    assert!(matches!(
        session.update(
            point_key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
            DxfPointEditIssue::DuplicatePatch {
                kind: DxfPointPatchKind::Location,
                ..
            }
        ))
    ));
    assert_eq!(session.queued_edit_count(), 1);

    for (shape, expected_role, duplicate) in [
        (
            PointShape::MissingY,
            seacad_dxf_core::DxfBasicGeometryComponentRole::WcsLocationOrStartY,
            false,
        ),
        (
            PointShape::DuplicateX,
            seacad_dxf_core::DxfBasicGeometryComponentRole::WcsLocationOrStartX,
            true,
        ),
    ] {
        let malformed = fixture(DxfRawDocumentFormat::Ascii, DxfAcadVersion::Ac1032, shape)?;
        let malformed_source = DxfMemorySource::new(&malformed, DxfResourceProfile::Safe)?;
        let malformed_document = open_ascii(&malformed_source)?;
        let malformed_view = DxfRawDocumentView::from(&malformed_document);
        let malformed_evidence = malformed_view.entity_field_evidence_directory(&token())?;
        let key = key_for_topic(&malformed_evidence, DxfEntityTopic::POINT)?;
        let malformed_cancellation = token();
        let mut malformed_session = malformed_view.entity_edit_session(
            &malformed_evidence,
            DxfResourceProfile::Safe,
            &malformed_cancellation,
        )?;
        let outcome = malformed_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED)),
        )?;
        assert!(if duplicate {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::DuplicateLocationComponent { role, occurrence_count: 2 }
                )) if role == expected_role
            )
        } else {
            matches!(
                outcome,
                DxfEntityEditOutcome::Unavailable(DxfEntityEditIssue::Point(
                    DxfPointEditIssue::MissingLocationComponent { role }
                )) if role == expected_role
            )
        });
        assert_eq!(malformed_session.queued_edit_count(), 0);
    }
    Ok(())
}

#[test]
fn point_location_composes_with_common_update_and_blocks_insert_mixing()
-> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let DxfEntityEditOutcome::Applied(common) = session.update(
        key,
        DxfEntityPatch::CommonField(DxfEntityCommonFieldPatch::SetExplicit {
            field: DxfEntityField::VISIBILITY,
            value: DxfEntityEditValue::Int16(1),
        }),
    )?
    else {
        return Err(io::Error::other("common update after POINT").into());
    };
    assert_eq!(common.queued_edit_count(), 2);

    let placement = view.entity_placement_directory(&token())?.assessments()[0]
        .placement()
        .ok_or_else(|| io::Error::other("placement"))?;
    assert!(matches!(
        session.insert(
            placement,
            DxfEntityDraft::point(DxfPointDraft::new(b"0", UPDATED))
        )?,
        seacad_dxf_core::DxfEntityInsertOutcome::Unavailable(
            seacad_dxf_core::DxfEntityInsertIssue::UpdatePending {
                queued_update_count: 2
            }
        )
    ));
    let plan = session.finish_verifiable()?;
    assert_eq!(plan.edit_count(), 2);
    assert_eq!(plan.transaction().patches().len(), 4);
    let output = materialize(&bytes, plan.transaction())?;
    let output_source = DxfMemorySource::new(&output, DxfResourceProfile::Safe)?;
    let output_document = open_ascii(&output_source)?;
    let post = DxfRawDocumentView::from(&output_document);
    assert!(matches!(
        plan.verify_post_image(view, post, DxfResourceProfile::Safe, &token())?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Verified(_)
    ));
    Ok(())
}

#[test]
fn point_location_verifier_and_cancellation_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = fixture(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        PointShape::Complete,
    )?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let evidence = view.entity_field_evidence_directory(&token())?;
    let key = key_for_topic(&evidence, DxfEntityTopic::POINT)?;
    let cancellation = token();
    let mut session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancellation)?;
    assert!(matches!(
        session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        )?,
        DxfEntityEditOutcome::PointApplied(_)
    ));
    let plan = session.finish_verifiable()?;
    let output = materialize(&bytes, plan.transaction())?;
    let tampered = replace_once(&output, b"10\n4\n", b"10\n7\n")?;
    let tampered_source = DxfMemorySource::new(&tampered, DxfResourceProfile::Safe)?;
    let tampered_document = open_ascii(&tampered_source)?;
    assert!(matches!(
        plan.verify_post_image(
            view,
            DxfRawDocumentView::from(&tampered_document),
            DxfResourceProfile::Safe,
            &token()
        )?,
        seacad_dxf_core::DxfEntityEditVerificationOutcome::Unavailable(
            seacad_dxf_core::DxfEntityEditVerificationIssue::UpdatedPointLocationMismatch {
                raw_record_ordinal
            }
        ) if raw_record_ordinal == key.raw_record_ordinal()
    ));

    let cancelled = token();
    let mut cancelled_session =
        view.entity_edit_session(&evidence, DxfResourceProfile::Safe, &cancelled)?;
    cancelled.cancel();
    assert!(matches!(
        cancelled_session.update(
            key,
            DxfEntityPatch::Point(DxfPointPatch::set_location(UPDATED))
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(cancelled_session.queued_edit_count(), 0);
    Ok(())
}

#[derive(Clone, Copy)]
enum PointShape {
    Complete,
    MissingY,
    DuplicateX,
}

fn fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    shape: PointShape,
) -> Result<Vec<u8>, io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => Ok(ascii_fixture(version, shape)),
        DxfRawDocumentFormat::Binary => binary_fixture(version, shape),
        _ => Err(io::Error::other("format")),
    }
}

fn ascii_fixture(version: DxfAcadVersion, shape: PointShape) -> Vec<u8> {
    let point = match shape {
        PointShape::Complete => "30\n3\n10\n1\n20\n2\n",
        PointShape::MissingY => "30\n3\n10\n1\n",
        PointShape::DuplicateX => "30\n3\n10\n1\n20\n2\n10\n9\n",
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOINT\n60\n0\n{}39\n2.5\n50\n30\n0\nLINE\n10\n7\n20\n8\n30\n9\n11\n10\n21\n11\n31\n12\n0\nENDSEC\n0\nEOF\n",
        version.code(),
        point
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, shape: PointShape) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"POINT")?;
    push_i16(&mut bytes, version, 60, 0)?;
    for (code, value) in [(30, 3.0), (10, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    if !matches!(shape, PointShape::MissingY) {
        push_double(&mut bytes, version, 20, 2.0)?;
    }
    if matches!(shape, PointShape::DuplicateX) {
        push_double(&mut bytes, version, 10, 9.0)?;
    }
    push_double(&mut bytes, version, 39, 2.5)?;
    push_double(&mut bytes, version, 50, 30.0)?;
    push_string(&mut bytes, version, 0, b"LINE")?;
    for (code, value) in [
        (10, 7.0),
        (20, 8.0),
        (30, 9.0),
        (11, 10.0),
        (21, 11.0),
        (31, 12.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn key_for_topic(
    evidence: &DxfEntityFieldEvidenceDirectory,
    topic: DxfEntityTopic,
) -> Result<seacad_dxf_core::DxfEntityKey, io::Error> {
    evidence
        .entity_directory()
        .entities()
        .iter()
        .copied()
        .find(|entity| entity.classification().topic() == Some(topic))
        .map(|entity| entity.key())
        .ok_or_else(|| io::Error::other("entity key"))
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn materialize(source: &[u8], plan: &DxfTransactionPlan) -> Result<Vec<u8>, DxfError> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    for patch in plan.patches() {
        let start = usize::try_from(patch.source_span().start()).map_err(|_| test_error())?;
        let end = usize::try_from(patch.source_span().end()).map_err(|_| test_error())?;
        output.extend_from_slice(source.get(cursor..start).ok_or_else(test_error)?);
        output.extend_from_slice(
            plan.replacement_bytes_for_patch_ordinal(patch.ordinal())
                .ok_or_else(test_error)?,
        );
        cursor = end;
    }
    output.extend_from_slice(source.get(cursor..).ok_or_else(test_error)?);
    Ok(output)
}

fn replace_once(source: &[u8], needle: &[u8], replacement: &[u8]) -> Result<Vec<u8>, io::Error> {
    if needle.len() != replacement.len() {
        return Err(io::Error::other("same-length replacement"));
    }
    let matches: Vec<_> = source
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == needle).then_some(offset))
        .collect();
    let [offset] = matches.as_slice() else {
        return Err(io::Error::other("one replacement target"));
    };
    let mut output = source.to_vec();
    output[*offset..*offset + replacement.len()].copy_from_slice(replacement);
    Ok(output)
}

fn open_document<'a>(
    source: &'a dyn DxfByteSource,
    format: DxfRawDocumentFormat,
) -> Result<OpenedDocument<'a>, DxfError> {
    Ok(match format {
        DxfRawDocumentFormat::Ascii => OpenedDocument::Ascii(open_ascii(source)?),
        DxfRawDocumentFormat::Binary => OpenedDocument::Binary(open_binary(source)?),
        _ => return Err(test_error()),
    })
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

fn test_error() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::other("test data"),
    )
}

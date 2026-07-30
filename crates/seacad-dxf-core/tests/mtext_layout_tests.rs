use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextAttachment, DxfMTextDrawingDirection,
    DxfMTextLayoutDirectory, DxfMTextLayoutIssue, DxfMTextLayoutSemantics,
    DxfMTextLineSpacingStyle, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    attachment: Option<i16>,
    direction: Option<i16>,
    spacing: Option<i16>,
    spacing_state: DxfSemanticValueState,
}

#[test]
fn every_dialect_has_ascii_binary_mtext_layout_parity() -> Result<(), Box<dyn Error>> {
    let rows = [(1, 1, None), (5, 3, Some(1)), (9, 5, Some(2))];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.mtext_layout_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.mtext_layout_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
        assert_eq!(
            signatures(&ascii_directory)?,
            [
                Signature {
                    attachment: Some(1),
                    direction: Some(1),
                    spacing: None,
                    spacing_state: DxfSemanticValueState::Absent,
                },
                Signature {
                    attachment: Some(5),
                    direction: Some(3),
                    spacing: Some(1),
                    spacing_state: DxfSemanticValueState::Explicit,
                },
                Signature {
                    attachment: Some(9),
                    direction: Some(5),
                    spacing: Some(2),
                    spacing_state: DxfSemanticValueState::Explicit,
                },
            ]
        );
    }
    Ok(())
}

#[test]
fn every_documented_attachment_direction_and_spacing_code_is_classified()
-> Result<(), Box<dyn Error>> {
    let rows = [
        (1, 1, Some(1)),
        (2, 3, Some(2)),
        (3, 5, None),
        (4, 1, None),
        (5, 1, None),
        (6, 1, None),
        (7, 1, None),
        (8, 1, None),
        (9, 1, None),
    ];
    let bytes = ascii_fixture("AC1032", &rows);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_layout_directory(&DxfCancellationToken::default())?;
    let expected = [
        DxfMTextAttachment::TopLeft,
        DxfMTextAttachment::TopCenter,
        DxfMTextAttachment::TopRight,
        DxfMTextAttachment::MiddleLeft,
        DxfMTextAttachment::MiddleCenter,
        DxfMTextAttachment::MiddleRight,
        DxfMTextAttachment::BottomLeft,
        DxfMTextAttachment::BottomCenter,
        DxfMTextAttachment::BottomRight,
    ];
    for (record, attachment) in directory.records().iter().zip(expected) {
        let value = semantics(&directory, *record)?;
        assert_eq!(value.attachment().value(), Some(&attachment));
        assert_eq!(
            attachment.code(),
            rows[usize::from((attachment.code() - 1) as u8)].0
        );
    }

    let first = semantics(&directory, directory.records()[0])?;
    assert_eq!(
        first.drawing_direction().value(),
        Some(&DxfMTextDrawingDirection::LeftToRight)
    );
    assert_eq!(
        first.line_spacing_style().value(),
        Some(&DxfMTextLineSpacingStyle::AtLeast)
    );
    let second = semantics(&directory, directory.records()[1])?;
    assert_eq!(
        second.drawing_direction().value(),
        Some(&DxfMTextDrawingDirection::TopToBottom)
    );
    assert_eq!(
        second.line_spacing_style().value(),
        Some(&DxfMTextLineSpacingStyle::Exact)
    );
    let third = semantics(&directory, directory.records()[2])?;
    assert_eq!(
        third.drawing_direction().value(),
        Some(&DxfMTextDrawingDirection::ByStyle)
    );
    Ok(())
}

#[test]
fn unsupported_invalid_and_duplicate_codes_fail_typed_with_provenance() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nMTEXT\n71\n0\n72\n2\n73\n3\n\
0\nMTEXT\n71\nbad\n72\nbad\n73\nbad\n\
0\nMTEXT\n71\n1\n71\n2\n72\n1\n72\n3\n73\n1\n73\n2\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_layout_directory(&DxfCancellationToken::default())?;

    let unsupported = semantics(&directory, directory.records()[0])?;
    assert_eq!(
        unsupported.attachment().invalid_issue(),
        Some(&DxfMTextLayoutIssue::UnsupportedAttachmentCode { code: 0 })
    );
    assert_eq!(
        unsupported.drawing_direction().invalid_issue(),
        Some(&DxfMTextLayoutIssue::UnsupportedDrawingDirectionCode { code: 2 })
    );
    assert_eq!(
        unsupported.line_spacing_style().invalid_issue(),
        Some(&DxfMTextLayoutIssue::UnsupportedLineSpacingStyleCode { code: 3 })
    );
    assert!(unsupported.attachment().raw_provenance().is_some());

    let invalid = semantics(&directory, directory.records()[1])?;
    for issue in [
        invalid.attachment().invalid_issue(),
        invalid.drawing_direction().invalid_issue(),
        invalid.line_spacing_style().invalid_issue(),
    ] {
        assert!(matches!(issue, Some(DxfMTextLayoutIssue::Scalar(_))));
    }

    let multiple = semantics(&directory, directory.records()[2])?;
    assert!(matches!(
        multiple.attachment().invalid_issue(),
        Some(DxfMTextLayoutIssue::Scalar(
            seacad_dxf_core::DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2
            }
        ))
    ));
    for raw in [
        multiple.attachment().raw_provenance(),
        multiple.drawing_direction().raw_provenance(),
        multiple.line_spacing_style().raw_provenance(),
    ] {
        assert!(raw.is_some());
    }
    Ok(())
}

#[test]
fn required_optional_cancellation_scope_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextLayoutSemantics>();
    assert_send_sync::<DxfMTextLayoutDirectory>();

    let bytes = ascii_fixture("AC1032", &[]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_layout_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n0\nTOLERANCE\n3\nISO\n1\nT\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_layout_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.scalar_directory().source_id()
    );

    let mtext = semantics(&directory, directory.records()[0])?;
    assert!(matches!(
        mtext.attachment().invalid_issue(),
        Some(DxfMTextLayoutIssue::Scalar(
            seacad_dxf_core::DxfTextSymbolScalarIssue::MissingRequiredValue
        ))
    ));
    assert!(matches!(
        mtext.drawing_direction().invalid_issue(),
        Some(DxfMTextLayoutIssue::Scalar(
            seacad_dxf_core::DxfTextSymbolScalarIssue::MissingRequiredValue
        ))
    ));
    assert_eq!(
        mtext.line_spacing_style().state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(
        directory.semantics_for_record(directory.records()[1])?,
        None
    );
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    Ok(())
}

fn signatures(directory: &DxfMTextLayoutDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(value)) => Some(Ok(Signature {
                attachment: value.attachment().value().map(|item| item.code()),
                direction: value.drawing_direction().value().map(|item| item.code()),
                spacing: value.line_spacing_style().value().map(|item| item.code()),
                spacing_state: value.line_spacing_style().state(),
            })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfMTextLayoutDirectory,
    record: seacad_dxf_core::DxfTextSymbolRecordEntry,
) -> Result<DxfMTextLayoutSemantics, Box<dyn Error>> {
    directory
        .semantics_for_record(record)?
        .ok_or_else(|| invalid_test_data().into())
}

fn ascii_fixture(version: &str, rows: &[(i16, i16, Option<i16>)]) -> Vec<u8> {
    let mut entities = String::new();
    for (attachment, direction, spacing) in rows {
        entities.push_str(&format!("0\nMTEXT\n71\n{attachment}\n72\n{direction}\n"));
        if let Some(spacing) = spacing {
            entities.push_str(&format!("73\n{spacing}\n"));
        }
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(
    version: DxfAcadVersion,
    rows: &[(i16, i16, Option<i16>)],
) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    for (attachment, direction, spacing) in rows {
        push_string(&mut bytes, version, 0, b"MTEXT")?;
        push_i16(&mut bytes, version, 71, *attachment)?;
        push_i16(&mut bytes, version, 72, *direction)?;
        if let Some(spacing) = spacing {
            push_i16(&mut bytes, version, 73, *spacing)?;
        }
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
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

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| invalid_test_data())?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
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

fn invalid_test_data() -> io::Error {
    io::Error::other("invalid test data")
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

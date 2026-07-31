use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextColumnMode, DxfMTextColumnRelationDirectory,
    DxfMTextColumnRelationIssue, DxfMTextColumnRelationSemantics, DxfMTextEmbeddedColumnRole,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_column_mode_parity() -> Result<(), Box<dyn Error>> {
    let expected = [
        DxfMTextColumnMode::NoColumns,
        DxfMTextColumnMode::Static,
        DxfMTextColumnMode::DynamicAutomatic,
        DxfMTextColumnMode::DynamicManual,
        DxfMTextColumnMode::DynamicManual,
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_document(version.code(), valid_payload());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_relations =
            ascii.mtext_column_relation_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_document(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_relations =
            binary.mtext_column_relation_directory(&DxfCancellationToken::default())?;

        assert_eq!(modes(&ascii_relations)?, expected);
        assert_eq!(modes(&binary_relations)?, expected);
    }
    Ok(())
}

#[test]
fn invalid_cross_field_combinations_fail_typed() -> Result<(), Box<dyn Error>> {
    let payload = "\
0\nMTEXT\n101\nEmbedded Object\n71\n1\n72\n0\n44\n20\n45\n1\n41\n50\n\
0\nMTEXT\n101\nEmbedded Object\n71\n1\n72\n2\n44\n20\n45\n1\n41\n0\n\
0\nMTEXT\n101\nEmbedded Object\n71\n1\n72\n2\n44\n20\n45\n1\n41\n50\n73\n1\n\
0\nMTEXT\n101\nEmbedded Object\n71\n1\n72\n2\n44\n20\n45\n1\n41\n50\n46\n10\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n0\n44\n20\n45\n1\n41\n50\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n0\n44\n20\n45\n1\n41\n50\n73\n1\n46\n10\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n2\n44\n20\n45\n1\n41\n0\n73\n0\n46\n10\n46\n20\n46\n0\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n1\n44\n20\n45\n1\n41\n0\n73\n0\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n0\n45\n1\n41\n50\n73\n1\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n0\n44\n0\n45\n1\n41\n50\n73\n1\n";
    let bytes = ascii_document("AC1032", payload);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_column_relation_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.semantics().len(), 10);

    assert_issue(
        &directory,
        0,
        DxfMTextColumnRelationIssue::PositiveCountRequired {
            mode: DxfMTextColumnMode::Static,
        },
    )?;
    assert_issue(
        &directory,
        1,
        DxfMTextColumnRelationIssue::PositiveSharedHeightRequired {
            mode: DxfMTextColumnMode::Static,
        },
    )?;
    assert_issue(
        &directory,
        2,
        DxfMTextColumnRelationIssue::AutomaticHeightRequiresDynamic,
    )?;
    assert_issue(
        &directory,
        3,
        DxfMTextColumnRelationIssue::IndividualHeightsRequireDynamicManual,
    )?;
    assert_issue(
        &directory,
        4,
        DxfMTextColumnRelationIssue::MissingRequired {
            role: DxfMTextEmbeddedColumnRole::ColumnAutoHeight,
        },
    )?;
    assert_issue(
        &directory,
        5,
        DxfMTextColumnRelationIssue::IndividualHeightsRequireDynamicManual,
    )?;
    assert_issue(
        &directory,
        6,
        DxfMTextColumnRelationIssue::IndividualHeightCountMismatch {
            column_count: 2,
            height_count: 3,
        },
    )?;
    assert_issue(
        &directory,
        7,
        DxfMTextColumnRelationIssue::PositiveSharedHeightRequired {
            mode: DxfMTextColumnMode::DynamicManual,
        },
    )?;
    assert_issue(
        &directory,
        8,
        DxfMTextColumnRelationIssue::MissingRequired {
            role: DxfMTextEmbeddedColumnRole::ColumnWidth,
        },
    )?;
    assert!(matches!(
        relation(&directory, 9)?.mode().invalid_issue(),
        Some(DxfMTextColumnRelationIssue::Scalar(_))
    ));
    Ok(())
}

#[test]
fn zero_terminal_height_identity_lookup_cancellation_and_traits_hold() -> Result<(), Box<dyn Error>>
{
    assert_copy::<DxfMTextColumnMode>();
    assert_copy::<DxfMTextColumnRelationSemantics>();
    assert_send_sync::<DxfMTextColumnRelationDirectory>();

    let bytes = ascii_document("AC1032", valid_payload());
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_column_relation_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.mtext_column_relation_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.source_id(),
        directory.scalar_directory().source_id()
    );
    let manual = relation(&directory, 3)?;
    let heights = directory
        .scalar_directory()
        .individual_heights(manual.scalar_semantics())
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        heights[2].value().ok_or_else(invalid_test_data)?.to_f64(),
        0.0
    );
    let occurrence = manual.scalar_semantics().entry().marker().occurrence();
    assert_eq!(
        directory.semantics_for_marker_occurrence(occurrence),
        Some(manual)
    );
    assert_eq!(directory.semantics_for_marker_occurrence(u64::MAX), None);
    assert_eq!(
        manual.mode().field_provenance().schema_namespace(),
        "entity.mtext.embedded_column_relation"
    );
    Ok(())
}

fn modes(
    directory: &DxfMTextColumnRelationDirectory,
) -> Result<Vec<DxfMTextColumnMode>, io::Error> {
    directory
        .semantics()
        .iter()
        .map(|item| item.mode().value().copied().ok_or_else(invalid_test_data))
        .collect()
}

fn relation(
    directory: &DxfMTextColumnRelationDirectory,
    index: usize,
) -> Result<DxfMTextColumnRelationSemantics, Box<dyn Error>> {
    directory
        .semantics()
        .get(index)
        .copied()
        .ok_or_else(|| invalid_test_data().into())
}

fn assert_issue(
    directory: &DxfMTextColumnRelationDirectory,
    index: usize,
    expected: DxfMTextColumnRelationIssue,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        relation(directory, index)?.mode().invalid_issue(),
        Some(&expected)
    );
    Ok(())
}

fn valid_payload() -> &'static str {
    "\
0\nMTEXT\n101\nEmbedded Object\n71\n0\n\
0\nMTEXT\n101\nEmbedded Object\n71\n1\n72\n3\n44\n20\n45\n1\n41\n100\n73\n0\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n0\n44\n20\n45\n1\n41\n50\n73\n1\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n3\n44\n20\n45\n1\n41\n0\n73\n0\n46\n20\n46\n30\n46\n0\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n1\n44\n20\n45\n1\n41\n50\n73\n0\n"
}

fn ascii_document(version: &str, entities: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_document(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_mode(&mut bytes, version, 0, None, None, &[])?;
    push_mode(&mut bytes, version, 1, Some((3, 0)), Some(100.0), &[])?;
    push_mode(&mut bytes, version, 2, Some((0, 1)), Some(50.0), &[])?;
    push_mode(
        &mut bytes,
        version,
        2,
        Some((3, 0)),
        Some(0.0),
        &[20.0, 30.0, 0.0],
    )?;
    push_mode(&mut bytes, version, 2, Some((1, 0)), Some(50.0), &[])?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_mode(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    column_type: i16,
    count_and_auto: Option<(i16, i16)>,
    shared_height: Option<f64>,
    heights: &[f64],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"MTEXT")?;
    push_string(bytes, version, 101, b"Embedded Object")?;
    push_i16(bytes, version, 71, column_type)?;
    if let Some((count, auto)) = count_and_auto {
        push_i16(bytes, version, 72, count)?;
        push_double(bytes, version, 44, 20.0)?;
        push_double(bytes, version, 45, 1.0)?;
        if let Some(height) = shared_height {
            push_double(bytes, version, 41, height)?;
        }
        push_i16(bytes, version, 73, auto)?;
    }
    for height in heights {
        push_double(bytes, version, 46, *height)?;
    }
    Ok(())
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

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
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

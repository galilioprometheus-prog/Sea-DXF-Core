use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextColumnIssue, DxfMTextColumnSemanticDirectory,
    DxfMTextColumnSemantics, DxfMTextColumnType, DxfMTextEmbeddedColumnRole, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_column_scalar_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_document(version.code(), valid_payload());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_columns =
            ascii.mtext_column_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_document(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_columns =
            binary.mtext_column_semantic_directory(&DxfCancellationToken::default())?;

        assert_eq!(signature(&ascii_columns)?, signature(&binary_columns)?);
        assert_eq!(
            signature(&ascii_columns)?,
            (
                DxfMTextColumnType::Dynamic,
                3,
                20.0_f64.to_bits(),
                0.0_f64.to_bits(),
                false,
                true,
                50.0_f64.to_bits(),
                vec![10.0_f64.to_bits(), 20.0_f64.to_bits(), 30.0_f64.to_bits()],
            )
        );
    }
    Ok(())
}

#[test]
fn invalid_domains_duplicates_and_missing_type_fail_closed() -> Result<(), Box<dyn Error>> {
    let payload = "\
0\nMTEXT\n101\nEmbedded Object\n72\n1\n\
0\nMTEXT\n101\nEmbedded Object\n71\n9\n\
0\nMTEXT\n101\nEmbedded Object\n71\n2\n72\n-1\n44\n0\n45\n-1\n73\n2\n74\n-1\n41\n0\n46\n-2\n\
0\nMTEXT\n101\nEmbedded Object\n71\n1\n71\n2\n\
0\nMTEXT\n101\nEmbedded Object\n71\nbad\n\
0\nMTEXT\n101\nEmbedded Object\n71\n0\n72\n0\n";
    let bytes = ascii_document("AC1032", payload);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_column_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.semantics().len(), 6);

    assert_eq!(
        semantics(&directory, 0)?.column_type().invalid_issue(),
        Some(&DxfMTextColumnIssue::MissingColumnType)
    );
    assert_eq!(
        semantics(&directory, 1)?.column_type().invalid_issue(),
        Some(&DxfMTextColumnIssue::UnsupportedColumnType { code: 9 })
    );

    let domains = semantics(&directory, 2)?;
    assert_eq!(
        domains.column_count().invalid_issue(),
        Some(&DxfMTextColumnIssue::NegativeCount { count: -1 })
    );
    assert_non_positive(
        domains.column_width().invalid_issue(),
        DxfMTextEmbeddedColumnRole::ColumnWidth,
    );
    assert!(matches!(
        domains.column_gutter().invalid_issue(),
        Some(DxfMTextColumnIssue::NegativeValue {
            role: DxfMTextEmbeddedColumnRole::ColumnGutter,
            ..
        })
    ));
    for value in [domains.auto_height(), domains.flow_reversed()] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfMTextColumnIssue::BooleanOutOfDomain { .. })
        ));
    }
    assert_non_positive(
        domains.shared_height().invalid_issue(),
        DxfMTextEmbeddedColumnRole::SharedHeight,
    );
    assert_non_positive(
        directory
            .individual_heights(domains)
            .ok_or_else(invalid_test_data)?[0]
            .invalid_issue(),
        DxfMTextEmbeddedColumnRole::ColumnHeight,
    );

    assert_eq!(
        semantics(&directory, 3)?.column_type().invalid_issue(),
        Some(&DxfMTextColumnIssue::MultipleValues {
            role: DxfMTextEmbeddedColumnRole::ColumnType,
            occurrence_count: 2,
        })
    );
    assert!(matches!(
        semantics(&directory, 4)?.column_type().invalid_issue(),
        Some(DxfMTextColumnIssue::Numeric(_))
    ));
    assert_eq!(semantics(&directory, 5)?.column_count().value(), Some(&0));
    for item in &directory.semantics()[1..] {
        assert!(item.column_type().raw_provenance().is_some());
    }
    Ok(())
}

#[test]
fn absence_lookup_identity_cancellation_and_public_traits_are_bounded() -> Result<(), Box<dyn Error>>
{
    assert_copy::<DxfMTextColumnType>();
    assert_copy::<DxfMTextColumnSemantics>();
    assert_send_sync::<DxfMTextColumnSemanticDirectory>();

    let bytes = ascii_document("AC1032", valid_payload());
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_column_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.mtext_column_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    let item = semantics(&directory, 0)?;
    assert_eq!(
        directory.semantics_for_marker_occurrence(item.entry().marker().occurrence()),
        Some(item)
    );
    assert_eq!(directory.semantics_for_marker_occurrence(u64::MAX), None);
    assert_eq!(item.individual_height_count(), 3);
    for field in [
        item.column_type().field_provenance(),
        item.column_count().field_provenance(),
        item.column_width().field_provenance(),
    ] {
        assert_eq!(field.document_source_id(), document.source_id());
        assert_eq!(field.schema_namespace(), "entity.mtext.embedded_columns");
    }
    Ok(())
}

type Signature = (DxfMTextColumnType, u16, u64, u64, bool, bool, u64, Vec<u64>);

fn signature(directory: &DxfMTextColumnSemanticDirectory) -> Result<Signature, Box<dyn Error>> {
    let item = semantics(directory, 0)?;
    assert_eq!(item.column_type().state(), DxfSemanticValueState::Explicit);
    Ok((
        *item.column_type().value().ok_or_else(invalid_test_data)?,
        *item.column_count().value().ok_or_else(invalid_test_data)?,
        item.column_width()
            .value()
            .ok_or_else(invalid_test_data)?
            .to_bits(),
        item.column_gutter()
            .value()
            .ok_or_else(invalid_test_data)?
            .to_bits(),
        *item.auto_height().value().ok_or_else(invalid_test_data)?,
        *item.flow_reversed().value().ok_or_else(invalid_test_data)?,
        item.shared_height()
            .value()
            .ok_or_else(invalid_test_data)?
            .to_bits(),
        directory
            .individual_heights(item)
            .ok_or_else(invalid_test_data)?
            .iter()
            .map(|height| {
                height
                    .value()
                    .map(|value| value.to_bits())
                    .ok_or_else(invalid_test_data)
            })
            .collect::<Result<_, _>>()?,
    ))
}

fn semantics(
    directory: &DxfMTextColumnSemanticDirectory,
    index: usize,
) -> Result<DxfMTextColumnSemantics, Box<dyn Error>> {
    directory
        .semantics()
        .get(index)
        .copied()
        .ok_or_else(|| invalid_test_data().into())
}

fn assert_non_positive(issue: Option<&DxfMTextColumnIssue>, role: DxfMTextEmbeddedColumnRole) {
    assert!(matches!(
        issue,
        Some(DxfMTextColumnIssue::NonPositiveValue {
            role: observed,
            ..
        }) if *observed == role
    ));
}

fn valid_payload() -> &'static str {
    "0\nMTEXT\n101\nEmbedded Object\n70\n1\n41\n50\n71\n2\n72\n3\n\
44\n20\n45\n0\n73\n0\n74\n1\n46\n10\n46\n20\n46\n30\n"
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
        (0, b"MTEXT"),
        (101, b"Embedded Object"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i16(&mut bytes, version, 70, 1)?;
    push_double(&mut bytes, version, 41, 50.0)?;
    for (code, value) in [(71, 2), (72, 3)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(44, 20.0), (45, 0.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(73, 0), (74, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for height in [10.0, 20.0, 30.0] {
        push_double(&mut bytes, version, 46, height)?;
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

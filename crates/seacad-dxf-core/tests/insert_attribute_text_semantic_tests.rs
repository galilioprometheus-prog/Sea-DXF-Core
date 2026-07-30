use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfInsertAttributeTextSemanticDirectory,
    DxfInsertAttributeTextSemanticIssue, DxfInsertAttributeTextSemantics,
    DxfInsertAttributeTextStyleName, DxfMemorySource, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct TextSignature {
    text_state: DxfSemanticValueState,
    text: Option<Vec<u8>>,
    tag_state: DxfSemanticValueState,
    tag: Option<Vec<u8>>,
    style_state: DxfSemanticValueState,
    style: Option<Vec<u8>>,
}

#[test]
fn every_supported_dialect_has_ascii_binary_text_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_text_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_text_semantic_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(DxfRawDocumentView::from(&ascii), &ascii_directory)?,
            signatures(DxfRawDocumentView::from(&binary), &binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn required_text_and_tag_fail_while_absent_style_defaults_to_standard() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_text_semantic_directory(&DxfCancellationToken::default())?;
    let empty = directory
        .semantics_for_entry(directory.records()[1])?
        .ok_or_else(invalid_test_data)?;
    for required in [empty.text_value(), empty.attribute_tag()] {
        assert_eq!(required.state(), DxfSemanticValueState::Invalid);
        assert_eq!(
            required.invalid_issue(),
            Some(&DxfInsertAttributeTextSemanticIssue::MissingRequiredValue)
        );
        assert_eq!(required.raw_provenance(), None);
    }
    assert_eq!(
        empty.text_style_name().state(),
        DxfSemanticValueState::Defaulted
    );
    let style = empty
        .text_style_name()
        .value()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert!(style.is_standard_default());
    assert_eq!(style.source(), None);
    assert_eq!(
        DxfInsertAttributeTextStyleName::STANDARD,
        b"STANDARD".as_slice()
    );
    Ok(())
}

#[test]
fn duplicate_text_roles_fail_typed_with_first_occurrence_provenance() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n1\nfirst\n1\nsecond\n2\nA\n2\nB\n7\nS1\n7\nS2\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_text_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = directory
        .semantics_for_raw_record(directory.records()[0].record().ordinal())?
        .ok_or_else(invalid_test_data)?;
    for value in [
        semantics.text_value().as_ref().map_value(|_| ()),
        semantics.attribute_tag().as_ref().map_value(|_| ()),
        semantics.text_style_name().as_ref().map_value(|_| ()),
    ] {
        assert!(matches!(
            value.invalid_issue(),
            Some(DxfInsertAttributeTextSemanticIssue::MultipleValues {
                occurrence_count: 2
            })
        ));
        assert!(value.raw_provenance().is_some());
    }
    Ok(())
}

#[test]
fn decoding_cancellation_lookups_source_identity_and_public_traits_hold()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeTextSemantics>();
    assert_send_sync::<DxfInsertAttributeTextSemanticDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_text_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.insert_attribute_text_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert!(directory.semantics_for_raw_record(u64::MAX)?.is_none());
    assert!(
        directory
            .semantics_for_insert_sequence_attribute(u64::MAX, 0)?
            .is_none()
    );
    let first = directory.records()[0];
    let insert = first.sequence().insert().record().ordinal();
    let semantics = directory
        .semantics_for_insert_sequence_attribute(insert, 0)?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(semantics.record(), first);
    assert!(
        directory
            .semantics_for_insert_sequence_attribute(insert, u64::MAX)?
            .is_none()
    );

    let text = semantics
        .text_value()
        .value()
        .copied()
        .ok_or_else(invalid_test_data)?;
    let mut decoded = [0_u8; 16];
    let receipt =
        text.decode_to_utf8_without_replacement(DxfRawDocumentView::from(&document), &mut decoded)?;
    let written = receipt
        .decode_result()
        .ok_or_else(invalid_test_data)?
        .written();
    assert_eq!(&decoded[..written], b"value");
    Ok(())
}

fn signatures(
    document: DxfRawDocumentView<'_>,
    directory: &DxfInsertAttributeTextSemanticDirectory,
) -> Result<Vec<TextSignature>, DxfError> {
    directory
        .records()
        .iter()
        .map(|record| {
            let semantics = directory
                .semantics_for_entry(*record)?
                .ok_or_else(invalid_test_data)?;
            Ok(TextSignature {
                text_state: semantics.text_value().state(),
                text: source_text_bytes(document, semantics.text_value().value().copied())?,
                tag_state: semantics.attribute_tag().state(),
                tag: source_text_bytes(document, semantics.attribute_tag().value().copied())?,
                style_state: semantics.text_style_name().state(),
                style: style_bytes(document, semantics.text_style_name().value().copied())?,
            })
        })
        .collect()
}

fn source_text_bytes(
    document: DxfRawDocumentView<'_>,
    text: Option<seacad_dxf_core::DxfInsertAttributeTextValue>,
) -> Result<Option<Vec<u8>>, DxfError> {
    let Some(text) = text else {
        return Ok(None);
    };
    read_span(document, text.value_span()).map(Some)
}

fn style_bytes(
    document: DxfRawDocumentView<'_>,
    style: Option<DxfInsertAttributeTextStyleName>,
) -> Result<Option<Vec<u8>>, DxfError> {
    match style {
        Some(DxfInsertAttributeTextStyleName::Source(text)) => {
            read_span(document, text.value_span()).map(Some)
        }
        Some(DxfInsertAttributeTextStyleName::Standard) => {
            Ok(Some(DxfInsertAttributeTextStyleName::STANDARD.to_vec()))
        }
        Some(_) | None => Ok(None),
    }
}

fn read_span(
    document: DxfRawDocumentView<'_>,
    span: seacad_dxf_core::ByteSpan,
) -> Result<Vec<u8>, DxfError> {
    let length = usize::try_from(span.len()).map_err(|_| invalid_test_data())?;
    let mut bytes = vec![0_u8; length];
    document.read_span(span, &mut bytes)?;
    Ok(bytes)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n1\nvalue\n2\ntag\n7\nSTYLE\n0\nATTRIB\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"B")?;
    for code in [10, 20, 30] {
        push_double(&mut bytes, version, code, 0.0)?;
    }
    push_i16(&mut bytes, version, 66, 1)?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 1, b"value")?;
    push_string(&mut bytes, version, 2, b"tag")?;
    push_string(&mut bytes, version, 7, b"STYLE")?;
    push_string(&mut bytes, version, 0, b"ATTRIB")?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

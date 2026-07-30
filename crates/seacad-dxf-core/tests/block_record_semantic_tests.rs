use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfBlockDefinitionState, DxfBlockRecordSemanticDirectory,
    DxfBlockRecordSemanticDouble, DxfBlockRecordSemanticIssue, DxfBlockRecordSemanticText,
    DxfBlockRecordSemantics, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct SemanticEvidence {
    primary_name: Option<Vec<u8>>,
    flags: Option<i16>,
    base_point: [Option<u64>; 3],
    secondary_name: Option<Vec<u8>>,
    xref_path: Option<Vec<u8>>,
    description: Option<Vec<u8>>,
    flag_bits: [Option<bool>; 7],
}

#[test]
fn every_supported_dialect_has_ascii_binary_block_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.block_record_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.block_record_semantic_directory(&DxfCancellationToken::default())?;

        let ascii_semantics = only_semantics(&ascii_directory)?;
        let binary_semantics = only_semantics(&binary_directory)?;
        assert_valid_semantics(&ascii_directory, &ascii_semantics);
        assert_valid_semantics(&binary_directory, &binary_semantics);
        assert_eq!(
            evidence(DxfRawDocumentView::from(&ascii), &ascii_semantics)?,
            evidence(DxfRawDocumentView::from(&binary), &binary_semantics)?
        );
    }
    Ok(())
}

#[test]
fn missing_optional_duplicate_and_numeric_failure_states_remain_explicit()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n0\nENDBLK\n0\nBLOCK\n2\nA\n2\nB\n70\n.\n10\n.\n30\n3\n3\nA\n1\nx\n1\ny\n4\n\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_record_semantic_directory(&DxfCancellationToken::default())?;

    let empty = semantics(&directory, 0)?;
    for value in [
        empty.primary_name().state(),
        empty.flags().state(),
        empty.base_point()[0].state(),
        empty.base_point()[1].state(),
        empty.base_point()[2].state(),
        empty.secondary_name().state(),
    ] {
        assert_eq!(value, DxfSemanticValueState::Invalid);
    }
    assert_eq!(empty.xref_path().state(), DxfSemanticValueState::Absent);
    assert_eq!(empty.description().state(), DxfSemanticValueState::Absent);
    assert_eq!(empty.flags_value(), None);
    assert_eq!(empty.base_point_value(), None);
    assert_eq!(
        empty.primary_name().invalid_issue(),
        Some(&DxfBlockRecordSemanticIssue::MissingRequiredValue)
    );

    let invalid = semantics(&directory, 1)?;
    assert_multiple_text(invalid.primary_name(), 2);
    assert_invalid_ascii_integer(invalid.flags());
    assert_invalid_ascii_double(&invalid.base_point()[0]);
    assert_eq!(
        invalid.base_point()[1].invalid_issue(),
        Some(&DxfBlockRecordSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(
        invalid.base_point()[2].value(),
        Some(&DxfDouble::from_f64(3.0))
    );
    assert_eq!(
        invalid.secondary_name().state(),
        DxfSemanticValueState::Explicit
    );
    assert_multiple_text(invalid.xref_path(), 2);
    assert_eq!(
        invalid.description().state(),
        DxfSemanticValueState::Explicit
    );
    assert!(
        invalid
            .description()
            .value()
            .ok_or(io::Error::other("description"))?
            .value_span()
            .is_empty()
    );
    Ok(())
}

#[test]
fn semantics_remain_available_for_every_definition_state() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n70\n1\n0\nLINE\n0\nBLOCK\n70\n2\n0\nLINE\n0\nENDBLK\n0\nBLOCK\n70\n4\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_record_semantic_directory(&DxfCancellationToken::default())?;

    let expected = [
        (DxfBlockDefinitionState::Interrupted, 1_i16),
        (DxfBlockDefinitionState::Closed, 2_i16),
        (DxfBlockDefinitionState::Unclosed, 4_i16),
    ];
    assert_eq!(directory.records().len(), expected.len());
    for (index, (state, flags)) in expected.into_iter().enumerate() {
        let semantic = semantics(&directory, index)?;
        assert_eq!(semantic.record().definition().state(), state);
        assert_eq!(semantic.flags_value(), Some(flags));
        assert_eq!(semantic.is_anonymous(), Some(flags & 1 != 0));
        assert_eq!(semantic.is_external_reference(), Some(flags & 4 != 0));
    }
    Ok(())
}

#[test]
fn cancellation_lookups_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.block_record_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.block_record_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(directory.semantics_for_block_raw_ordinal(u64::MAX)?, None);
    let record = directory.records()[0];
    assert_eq!(
        directory.semantics_for_entry(record)?,
        directory.semantics_for_block_raw_ordinal(record.definition().block_record().ordinal())?
    );
    assert_copy::<DxfBlockRecordSemantics>();
    assert_send_sync::<DxfBlockRecordSemanticDirectory>();
    Ok(())
}

fn assert_valid_semantics(
    directory: &DxfBlockRecordSemanticDirectory,
    semantics: &DxfBlockRecordSemantics,
) {
    assert_eq!(
        semantics.record().definition().state(),
        DxfBlockDefinitionState::Closed
    );
    for state in [
        semantics.primary_name().state(),
        semantics.flags().state(),
        semantics.base_point()[0].state(),
        semantics.base_point()[1].state(),
        semantics.base_point()[2].state(),
        semantics.secondary_name().state(),
        semantics.xref_path().state(),
        semantics.description().state(),
    ] {
        assert_eq!(state, DxfSemanticValueState::Explicit);
    }
    assert_eq!(
        semantics
            .base_point_value()
            .map(|point| point.map(DxfDouble::to_bits)),
        Some([
            (-0.0_f64).to_bits(),
            1.25_f64.to_bits(),
            (-2.5_f64).to_bits(),
        ])
    );
    assert_eq!(semantics.flags_value(), Some(127));
    assert_eq!(
        [
            semantics.is_anonymous(),
            semantics.has_non_constant_attribute_definitions(),
            semantics.is_external_reference(),
            semantics.is_external_reference_overlay(),
            semantics.is_externally_dependent(),
            semantics.is_resolved_external_reference_or_dependent(),
            semantics.is_referenced_external_reference(),
        ],
        [Some(true); 7]
    );
    for value in semantics.base_point() {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
        assert!(value.raw_provenance().is_some());
    }
}

fn evidence(
    view: DxfRawDocumentView<'_>,
    semantics: &DxfBlockRecordSemantics,
) -> Result<SemanticEvidence, io::Error> {
    Ok(SemanticEvidence {
        primary_name: text_bytes(view, semantics.primary_name())?,
        flags: semantics.flags_value(),
        base_point: semantics.base_point().each_ref().map(double_bits),
        secondary_name: text_bytes(view, semantics.secondary_name())?,
        xref_path: text_bytes(view, semantics.xref_path())?,
        description: text_bytes(view, semantics.description())?,
        flag_bits: [
            semantics.is_anonymous(),
            semantics.has_non_constant_attribute_definitions(),
            semantics.is_external_reference(),
            semantics.is_external_reference_overlay(),
            semantics.is_externally_dependent(),
            semantics.is_resolved_external_reference_or_dependent(),
            semantics.is_referenced_external_reference(),
        ],
    })
}

fn text_bytes(
    view: DxfRawDocumentView<'_>,
    semantic: &DxfBlockRecordSemanticText,
) -> Result<Option<Vec<u8>>, io::Error> {
    let Some(text) = semantic.value().copied() else {
        return Ok(None);
    };
    let length = usize::try_from(text.value_span().len()).map_err(io::Error::other)?;
    let mut bytes = vec![0_u8; length];
    view.read_span(text.value_span(), &mut bytes)
        .map_err(io::Error::other)?;
    Ok(Some(bytes))
}

fn double_bits(value: &DxfBlockRecordSemanticDouble) -> Option<u64> {
    value.value().copied().map(DxfDouble::to_bits)
}

fn only_semantics(
    directory: &DxfBlockRecordSemanticDirectory,
) -> Result<DxfBlockRecordSemantics, Box<dyn Error>> {
    let [record] = directory.records() else {
        return Err(io::Error::other("one record").into());
    };
    directory
        .semantics_for_entry(*record)?
        .ok_or_else(|| io::Error::other("record semantics").into())
}

fn semantics(
    directory: &DxfBlockRecordSemanticDirectory,
    index: usize,
) -> Result<DxfBlockRecordSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or(io::Error::other("record"))?;
    directory
        .semantics_for_entry(record)?
        .ok_or_else(|| io::Error::other("record semantics").into())
}

fn assert_multiple_text(value: &DxfBlockRecordSemanticText, occurrence_count: u32) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfBlockRecordSemanticIssue::MultipleValues { occurrence_count })
    );
    assert!(value.raw_provenance().is_some());
}

fn assert_invalid_ascii_double(value: &DxfBlockRecordSemanticDouble) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfBlockRecordSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert!(value.raw_provenance().is_some());
}

fn assert_invalid_ascii_integer(value: &seacad_dxf_core::DxfBlockRecordSemanticInteger) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfBlockRecordSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert!(value.raw_provenance().is_some());
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nBLOCK_A\n70\n127\n10\n-0\n20\n1.25\n30\n-2.5\n3\nBLOCK_A\n1\nxref.dxf\n4\ndescription\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"BLOCKS")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 2, b"BLOCK_A")?;
    push_i16(&mut bytes, version, 70, 127)?;
    push_double(&mut bytes, version, 10, -0.0)?;
    push_double(&mut bytes, version, 20, 1.25)?;
    push_double(&mut bytes, version, 30, -2.5)?;
    push_string(&mut bytes, version, 3, b"BLOCK_A")?;
    push_string(&mut bytes, version, 1, b"xref.dxf")?;
    push_string(&mut bytes, version, 4, b"description")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

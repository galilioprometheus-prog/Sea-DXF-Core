use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextChunkKind, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, DxfTextSymbolKind, DxfTextSymbolStyleName,
    DxfTextSymbolTextDirectory, DxfTextSymbolTextIssue, DxfTextSymbolValueRole,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct RecordSignature {
    kind: DxfTextSymbolKind,
    states: Vec<DxfSemanticValueState>,
    chunk_roles: Vec<(DxfMTextChunkKind, DxfTextSymbolValueRole)>,
    terminal_count: u64,
    additional_after_terminal_count: u64,
}

#[test]
fn every_dialect_has_ascii_binary_text_selection_and_chunk_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.text_symbol_text_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.text_symbol_text_directory(&DxfCancellationToken::default())?;

        assert_valid_fields(&ascii_directory)?;
        assert_valid_fields(&binary_directory)?;
        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn missing_duplicates_and_every_mtext_chunk_failure_remain_distinct() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n7\nA\n7\nB\n\
0\nMTEXT\n3\nonly-additional\n\
0\nMTEXT\n1\nterminal\n3\nlate\n\
0\nMTEXT\n1\nfirst-terminal\n1\nsecond-terminal\n\
0\nSHAPE\n2\nA\n2\nB\n\
0\nTOLERANCE\n3\nISO\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_symbol_text_directory(&DxfCancellationToken::default())?;
    let records = directory.records();

    let text = directory
        .text_semantics_for_record(records[0])?
        .ok_or(io::Error::other("TEXT"))?;
    assert_eq!(
        text.content().invalid_issue(),
        Some(&DxfTextSymbolTextIssue::MissingRequiredValue)
    );
    assert_eq!(text.content().raw_provenance(), None);
    assert_eq!(
        text.style_name().invalid_issue(),
        Some(&DxfTextSymbolTextIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(text.style_name().raw_provenance().is_some());

    let missing_terminal = mtext(&directory, records[1])?;
    assert_eq!(missing_terminal.chunks().terminal_count(), 0);
    assert_eq!(
        missing_terminal.chunks().additional_after_terminal_count(),
        0
    );
    assert!(!missing_terminal.chunks().is_valid());

    let late_additional = mtext(&directory, records[2])?;
    assert_eq!(late_additional.chunks().terminal_count(), 1);
    assert_eq!(
        late_additional.chunks().additional_after_terminal_count(),
        1
    );
    assert!(!late_additional.chunks().is_valid());

    let multiple_terminal = mtext(&directory, records[3])?;
    assert_eq!(multiple_terminal.chunks().terminal_count(), 2);
    assert_eq!(
        multiple_terminal.chunks().additional_after_terminal_count(),
        0
    );
    assert!(!multiple_terminal.chunks().is_valid());

    let shape = directory
        .shape_semantics_for_record(records[4])?
        .ok_or(io::Error::other("SHAPE"))?;
    assert_eq!(
        shape.shape_name().invalid_issue(),
        Some(&DxfTextSymbolTextIssue::MultipleValues {
            occurrence_count: 2
        })
    );

    let tolerance = directory
        .tolerance_semantics_for_record(records[5])?
        .ok_or(io::Error::other("TOLERANCE"))?;
    assert_eq!(
        tolerance.content().invalid_issue(),
        Some(&DxfTextSymbolTextIssue::MissingRequiredValue)
    );
    assert_eq!(
        tolerance.dimension_style_name().state(),
        DxfSemanticValueState::Explicit
    );
    Ok(())
}

#[test]
fn cancellation_identity_scope_lookups_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.text_symbol_text_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.text_symbol_text_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(directory.sequence_for_raw_record(u64::MAX), None);
    let sequence = directory.sequences()[0];
    assert_eq!(
        directory
            .chunks_for_sequence(sequence)
            .ok_or(io::Error::other("chunks"))?
            .len(),
        3
    );

    let other_bytes = fixture("AC1021");
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other_document = open_ascii(&other_source)?;
    let other = other_document.text_symbol_text_directory(&DxfCancellationToken::default())?;
    assert_eq!(other.chunks_for_sequence(sequence), None);

    let mtext_record = directory.records()[1];
    assert_eq!(directory.text_semantics_for_record(mtext_record)?, None);
    assert_eq!(directory.shape_semantics_for_record(mtext_record)?, None);
    assert_eq!(
        directory.tolerance_semantics_for_record(mtext_record)?,
        None
    );

    assert_copy::<seacad_dxf_core::DxfTextFieldSemantics>();
    assert_copy::<seacad_dxf_core::DxfMTextFieldSemantics>();
    assert_copy::<seacad_dxf_core::DxfShapeFieldSemantics>();
    assert_copy::<seacad_dxf_core::DxfToleranceFieldSemantics>();
    assert_send_sync::<DxfTextSymbolTextDirectory>();
    Ok(())
}

fn assert_valid_fields(directory: &DxfTextSymbolTextDirectory) -> Result<(), Box<dyn Error>> {
    let records = directory.records();
    let text = directory
        .text_semantics_for_record(records[0])?
        .ok_or(io::Error::other("TEXT"))?;
    assert_eq!(text.content().state(), DxfSemanticValueState::Explicit);
    assert_standard(text.style_name())?;

    let mtext = mtext(directory, records[1])?;
    assert_standard(mtext.style_name())?;
    assert!(mtext.chunks().is_valid());
    assert_eq!(mtext.chunks().terminal_count(), 1);
    assert_eq!(mtext.chunks().chunk_range().len(), 3);
    let chunks = directory
        .chunks_for_sequence(mtext.chunks())
        .ok_or(io::Error::other("MTEXT chunks"))?;
    assert_eq!(
        chunks.iter().map(|chunk| chunk.kind()).collect::<Vec<_>>(),
        vec![
            DxfMTextChunkKind::Additional,
            DxfMTextChunkKind::Additional,
            DxfMTextChunkKind::Terminal
        ]
    );

    let shape = directory
        .shape_semantics_for_record(records[2])?
        .ok_or(io::Error::other("SHAPE"))?;
    assert_eq!(shape.shape_name().state(), DxfSemanticValueState::Explicit);

    let tolerance = directory
        .tolerance_semantics_for_record(records[3])?
        .ok_or(io::Error::other("TOLERANCE"))?;
    assert_eq!(
        tolerance.dimension_style_name().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(tolerance.content().state(), DxfSemanticValueState::Explicit);
    Ok(())
}

fn assert_standard(
    style: &seacad_dxf_core::DxfTextSymbolSemanticStyle,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(style.state(), DxfSemanticValueState::Defaulted);
    let value = style.value().ok_or(io::Error::other("style default"))?;
    assert_eq!(*value, DxfTextSymbolStyleName::Standard);
    assert_eq!(value.source(), None);
    assert!(value.is_standard_default());
    assert_eq!(DxfTextSymbolStyleName::STANDARD, b"STANDARD");
    Ok(())
}

fn mtext(
    directory: &DxfTextSymbolTextDirectory,
    record: seacad_dxf_core::DxfTextSymbolRecordEntry,
) -> Result<seacad_dxf_core::DxfMTextFieldSemantics, Box<dyn Error>> {
    directory
        .mtext_semantics_for_record(record)?
        .ok_or_else(|| io::Error::other("MTEXT").into())
}

fn signatures(directory: &DxfTextSymbolTextDirectory) -> Result<Vec<RecordSignature>, DxfError> {
    let mut output = Vec::new();
    for record in directory.records().iter().copied() {
        let signature = match record.kind() {
            DxfTextSymbolKind::Text => {
                let Some(value) = directory.text_semantics_for_record(record)? else {
                    continue;
                };
                RecordSignature {
                    kind: record.kind(),
                    states: vec![value.content().state(), value.style_name().state()],
                    chunk_roles: Vec::new(),
                    terminal_count: 0,
                    additional_after_terminal_count: 0,
                }
            }
            DxfTextSymbolKind::MText => {
                let Some(value) = directory.mtext_semantics_for_record(record)? else {
                    continue;
                };
                let sequence = value.chunks();
                let chunks = directory
                    .chunks_for_sequence(sequence)
                    .ok_or_else(invalid_internal_data)?;
                RecordSignature {
                    kind: record.kind(),
                    states: vec![value.style_name().state()],
                    chunk_roles: chunks
                        .iter()
                        .map(|chunk| (chunk.kind(), chunk.value().role()))
                        .collect(),
                    terminal_count: sequence.terminal_count(),
                    additional_after_terminal_count: sequence.additional_after_terminal_count(),
                }
            }
            DxfTextSymbolKind::Shape => {
                let Some(value) = directory.shape_semantics_for_record(record)? else {
                    continue;
                };
                RecordSignature {
                    kind: record.kind(),
                    states: vec![value.shape_name().state()],
                    chunk_roles: Vec::new(),
                    terminal_count: 0,
                    additional_after_terminal_count: 0,
                }
            }
            DxfTextSymbolKind::Tolerance => {
                let Some(value) = directory.tolerance_semantics_for_record(record)? else {
                    continue;
                };
                RecordSignature {
                    kind: record.kind(),
                    states: vec![
                        value.dimension_style_name().state(),
                        value.content().state(),
                    ],
                    chunk_roles: Vec::new(),
                    terminal_count: 0,
                    additional_after_terminal_count: 0,
                }
            }
            _ => continue,
        };
        output.push(signature);
    }
    Ok(output)
}

fn fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n1\nTEXT-A\n\
0\nMTEXT\n3\nfirst-\n3\nsecond-\n1\ntail\n\
0\nSHAPE\n2\nBOLT\n\
0\nTOLERANCE\n3\nISO-25\n1\nTOL-A\n\
0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"TEXT")?;
    push_string(&mut bytes, version, 1, b"TEXT-A")?;
    push_string(&mut bytes, version, 0, b"MTEXT")?;
    push_string(&mut bytes, version, 3, b"first-")?;
    push_string(&mut bytes, version, 3, b"second-")?;
    push_string(&mut bytes, version, 1, b"tail")?;
    push_string(&mut bytes, version, 0, b"SHAPE")?;
    push_string(&mut bytes, version, 2, b"BOLT")?;
    push_string(&mut bytes, version, 0, b"TOLERANCE")?;
    push_string(&mut bytes, version, 3, b"ISO-25")?;
    push_string(&mut bytes, version, 1, b"TOL-A")?;
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
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    bytes.extend_from_slice(value);
    bytes.push(0);
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

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

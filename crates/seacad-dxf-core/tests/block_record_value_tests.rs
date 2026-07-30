use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfBlockDefinitionState, DxfBlockRecordTextValue, DxfBlockRecordValue,
    DxfBlockRecordValueData, DxfBlockRecordValueDirectory, DxfBlockRecordValueEntry,
    DxfBlockRecordValueIssue, DxfBlockRecordValueRange, DxfBlockRecordValueRole, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfRawDocumentView, DxfReadOptions,
    DxfResourceProfile, DxfTextDecodeStatus, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum ValueEvidence {
    Text(Vec<u8>),
    Double(u64),
    Int16(i16),
}

type RoleEvidence = (DxfBlockRecordValueRole, ValueEvidence);

#[test]
fn every_supported_dialect_has_ascii_binary_block_record_value_parity() -> Result<(), Box<dyn Error>>
{
    let expected = expected_evidence();
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.block_record_value_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.block_record_value_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory);
        assert_directory(&binary_directory);
        assert_eq!(
            evidence(DxfRawDocumentView::from(&ascii), &ascii_directory)?,
            expected
        );
        assert_eq!(
            evidence(DxfRawDocumentView::from(&binary), &binary_directory)?,
            expected
        );
    }
    Ok(())
}

#[test]
fn duplicates_failures_and_application_or_member_values_remain_separate()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n102\n{APP\n70\n7\n2\nhidden\n102\n}\n2\nB\n10\n.\n70\n32768\n4\n\n0\nLINE\n2\nmember\n70\n9\n0\nENDBLK\n2\nboundary\n70\n10\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_record_value_directory(&DxfCancellationToken::default())?;
    let view = DxfRawDocumentView::from(&document);

    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.values().len(), 5);
    let entry = directory.records()[0];
    assert_eq!(entry.definition().state(), DxfBlockDefinitionState::Closed);
    let values = directory
        .values_for_block_raw_ordinal(entry.definition().block_record().ordinal())
        .ok_or(io::Error::other("block values"))?;
    assert_eq!(
        values.iter().map(|value| value.role()).collect::<Vec<_>>(),
        [
            DxfBlockRecordValueRole::PrimaryName,
            DxfBlockRecordValueRole::PrimaryName,
            DxfBlockRecordValueRole::BasePointX,
            DxfBlockRecordValueRole::Flags,
            DxfBlockRecordValueRole::Description,
        ]
    );
    assert_eq!(text_bytes(view, text_value(values[0])?)?, b"A".as_slice());
    assert_eq!(text_bytes(view, text_value(values[1])?)?, b"B".as_slice());
    assert_eq!(
        values[2].value(),
        Err(DxfBlockRecordValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        values[3].value(),
        Err(DxfBlockRecordValueIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert!(text_value(values[4])?.value_span().is_empty());

    let application_group = directory
        .application_group_directory()
        .groups()
        .first()
        .copied()
        .ok_or(io::Error::other("application group"))?;
    for occurrence in application_group.content_group_range().start()
        ..application_group.content_group_range().end()
    {
        assert_eq!(directory.value_for_group(occurrence), None);
    }
    Ok(())
}

#[test]
fn values_remain_available_for_every_definition_state() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n70\n1\n0\nLINE\n0\nBLOCK\n70\n2\n0\nLINE\n0\nENDBLK\n0\nBLOCK\n70\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_record_value_directory(&DxfCancellationToken::default())?;

    let expected_states = [
        DxfBlockDefinitionState::Interrupted,
        DxfBlockDefinitionState::Closed,
        DxfBlockDefinitionState::Unclosed,
    ];
    let expected_flags = [1_i16, 2, 3];
    assert_eq!(directory.records().len(), expected_states.len());
    for ((entry, state), flags) in directory
        .records()
        .iter()
        .zip(expected_states)
        .zip(expected_flags)
    {
        assert_eq!(entry.definition().state(), state);
        let values = directory
            .values_for_block_raw_ordinal(entry.definition().block_record().ordinal())
            .ok_or(io::Error::other("definition-state values"))?;
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].role(), DxfBlockRecordValueRole::Flags);
        assert_eq!(values[0].value(), Ok(DxfBlockRecordValueData::Int16(flags)));
    }
    Ok(())
}

#[test]
fn cancellation_text_identity_lookups_and_public_traits_remain_bounded()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.block_record_value_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.block_record_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.definition_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.application_group_directory().source_id()
    );
    assert_eq!(directory.record_for_block_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_block_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);

    let text = text_value(directory.values()[0])?;
    let mut destination = [0_u8; 16];
    let receipt = text.decode_to_utf8_without_replacement(
        DxfRawDocumentView::from(&document),
        &mut destination,
    )?;
    let decoded = receipt
        .decode_result()
        .ok_or(io::Error::other("text decoder"))?;
    assert_eq!(decoded.status(), DxfTextDecodeStatus::Complete);
    assert_eq!(&destination[..decoded.written()], b"BLOCK_A");

    let other_bytes = ascii_fixture("AC1027");
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        text.decode_to_utf8_without_replacement(
            DxfRawDocumentView::from(&other),
            &mut destination,
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));

    assert_copy::<DxfBlockRecordTextValue>();
    assert_copy::<DxfBlockRecordValue>();
    assert_copy::<DxfBlockRecordValueEntry>();
    assert_copy::<DxfBlockRecordValueRange>();
    assert_send_sync::<DxfBlockRecordValueDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfBlockRecordValueDirectory) {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.values().len(), 8);
    assert_eq!(directory.definition_directory().definitions().len(), 1);
    let entry = directory.records()[0];
    assert_eq!(entry.definition().state(), DxfBlockDefinitionState::Closed);
    assert_eq!(entry.value_range().len(), 8);
    assert_eq!(
        directory.record_for_block_raw_ordinal(entry.definition().block_record().ordinal()),
        Some(entry)
    );
    for value in directory.values() {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(*value)
        );
    }
}

fn evidence(
    view: DxfRawDocumentView<'_>,
    directory: &DxfBlockRecordValueDirectory,
) -> Result<Vec<RoleEvidence>, io::Error> {
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            let evidence = match value.value().map_err(|_| io::Error::other("block value"))? {
                DxfBlockRecordValueData::Text(text) => ValueEvidence::Text(text_bytes(view, text)?),
                DxfBlockRecordValueData::Double(number) => ValueEvidence::Double(number.to_bits()),
                DxfBlockRecordValueData::Int16(number) => ValueEvidence::Int16(number),
                _ => return Err(io::Error::other("unknown block value")),
            };
            Ok((value.role(), evidence))
        })
        .collect()
}

fn text_value(value: DxfBlockRecordValue) -> Result<DxfBlockRecordTextValue, io::Error> {
    match value
        .value()
        .map_err(|_| io::Error::other("invalid text value"))?
    {
        DxfBlockRecordValueData::Text(text) => Ok(text),
        _ => Err(io::Error::other("non-text value")),
    }
}

fn text_bytes(
    view: DxfRawDocumentView<'_>,
    text: DxfBlockRecordTextValue,
) -> Result<Vec<u8>, io::Error> {
    let length = usize::try_from(text.value_span().len()).map_err(io::Error::other)?;
    let mut bytes = vec![0_u8; length];
    view.read_span(text.value_span(), &mut bytes)
        .map_err(io::Error::other)?;
    Ok(bytes)
}

fn expected_evidence() -> Vec<RoleEvidence> {
    use DxfBlockRecordValueRole::{
        BasePointX, BasePointY, BasePointZ, Description, Flags, PrimaryName, SecondaryName,
        XrefPath,
    };
    vec![
        (PrimaryName, ValueEvidence::Text(b"BLOCK_A".to_vec())),
        (Flags, ValueEvidence::Int16(-32768)),
        (BasePointX, ValueEvidence::Double((-0.0_f64).to_bits())),
        (BasePointY, ValueEvidence::Double(1.25_f64.to_bits())),
        (BasePointZ, ValueEvidence::Double((-2.5_f64).to_bits())),
        (SecondaryName, ValueEvidence::Text(b"BLOCK_A".to_vec())),
        (XrefPath, ValueEvidence::Text(b"xref/path.dxf".to_vec())),
        (Description, ValueEvidence::Text(b"description".to_vec())),
    ]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nBLOCK_A\n70\n-32768\n10\n-0\n20\n1.25\n30\n-2.5\n3\nBLOCK_A\n1\nxref/path.dxf\n4\ndescription\n0\nLINE\n2\nmember\n70\n7\n0\nENDBLK\n2\nboundary\n0\nENDSEC\n0\nEOF\n"
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
    push_i16(&mut bytes, version, 70, i16::MIN)?;
    push_double(&mut bytes, version, 10, -0.0)?;
    push_double(&mut bytes, version, 20, 1.25)?;
    push_double(&mut bytes, version, 30, -2.5)?;
    push_string(&mut bytes, version, 3, b"BLOCK_A")?;
    push_string(&mut bytes, version, 1, b"xref/path.dxf")?;
    push_string(&mut bytes, version, 4, b"description")?;
    push_string(&mut bytes, version, 0, b"LINE")?;
    push_string(&mut bytes, version, 2, b"member")?;
    push_i16(&mut bytes, version, 70, 7)?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_string(&mut bytes, version, 2, b"boundary")?;
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

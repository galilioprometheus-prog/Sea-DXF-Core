use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, DxfTextSymbolDirectory,
    DxfTextSymbolKind, DxfTextSymbolNumericIssue, DxfTextSymbolRecordEntry, DxfTextSymbolValue,
    DxfTextSymbolValueData, DxfTextSymbolValueRole, NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum FixtureValue {
    Text(&'static [u8]),
    Double(f64),
    Int16(i16),
    Int32(i32),
}

#[derive(Debug, Eq, PartialEq)]
enum ValueSignature {
    Text(Vec<u8>),
    Double(u64),
    Int16(i16),
    Int32(i32),
}

type RecordSignature = (
    DxfTextSymbolKind,
    Vec<(DxfTextSymbolValueRole, ValueSignature)>,
);

const TEXT_VALUES: &[(i16, FixtureValue)] = &[
    (39, FixtureValue::Double(0.5)),
    (10, FixtureValue::Double(1.0)),
    (20, FixtureValue::Double(2.0)),
    (30, FixtureValue::Double(3.0)),
    (40, FixtureValue::Double(2.5)),
    (1, FixtureValue::Text(b" Parcel 42 ")),
    (50, FixtureValue::Double(30.0)),
    (41, FixtureValue::Double(0.8)),
    (51, FixtureValue::Double(10.0)),
    (7, FixtureValue::Text(b"VN-TEXT")),
    (71, FixtureValue::Int16(6)),
    (72, FixtureValue::Int16(2)),
    (11, FixtureValue::Double(4.0)),
    (21, FixtureValue::Double(5.0)),
    (31, FixtureValue::Double(6.0)),
    (210, FixtureValue::Double(0.0)),
    (220, FixtureValue::Double(-0.0)),
    (230, FixtureValue::Double(1.0)),
    (73, FixtureValue::Int16(3)),
];

const MTEXT_VALUES: &[(i16, FixtureValue)] = &[
    (10, FixtureValue::Double(7.0)),
    (20, FixtureValue::Double(8.0)),
    (30, FixtureValue::Double(9.0)),
    (40, FixtureValue::Double(2.0)),
    (41, FixtureValue::Double(12.0)),
    (71, FixtureValue::Int16(5)),
    (72, FixtureValue::Int16(1)),
    (3, FixtureValue::Text(b"first-")),
    (3, FixtureValue::Text(b"second-")),
    (1, FixtureValue::Text(b"last")),
    (7, FixtureValue::Text(b"STANDARD")),
    (210, FixtureValue::Double(0.0)),
    (220, FixtureValue::Double(0.0)),
    (230, FixtureValue::Double(1.0)),
    (11, FixtureValue::Double(1.0)),
    (21, FixtureValue::Double(0.0)),
    (31, FixtureValue::Double(0.0)),
    (42, FixtureValue::Double(9.5)),
    (43, FixtureValue::Double(4.5)),
    (50, FixtureValue::Double(0.25)),
    (50, FixtureValue::Double(3.5)),
    (73, FixtureValue::Int16(2)),
    (44, FixtureValue::Double(1.25)),
    (90, FixtureValue::Int32(1)),
    (420, FixtureValue::Int32(0x112233)),
    (430, FixtureValue::Text(b"Book$Color")),
    (45, FixtureValue::Double(1.5)),
    (63, FixtureValue::Int16(7)),
    (441, FixtureValue::Int32(0x01020304)),
    (75, FixtureValue::Int16(2)),
    (76, FixtureValue::Int16(2)),
    (78, FixtureValue::Int16(1)),
    (79, FixtureValue::Int16(0)),
    (48, FixtureValue::Double(5.0)),
    (49, FixtureValue::Double(0.5)),
];

const SHAPE_VALUES: &[(i16, FixtureValue)] = &[
    (39, FixtureValue::Double(0.25)),
    (10, FixtureValue::Double(1.0)),
    (20, FixtureValue::Double(2.0)),
    (30, FixtureValue::Double(3.0)),
    (40, FixtureValue::Double(4.0)),
    (2, FixtureValue::Text(b"BOLT")),
    (50, FixtureValue::Double(15.0)),
    (41, FixtureValue::Double(1.5)),
    (51, FixtureValue::Double(5.0)),
    (210, FixtureValue::Double(0.0)),
    (220, FixtureValue::Double(1.0)),
    (230, FixtureValue::Double(0.0)),
];

const TOLERANCE_VALUES: &[(i16, FixtureValue)] = &[
    (3, FixtureValue::Text(b"ISO-25")),
    (10, FixtureValue::Double(1.0)),
    (20, FixtureValue::Double(2.0)),
    (30, FixtureValue::Double(3.0)),
    (1, FixtureValue::Text(b"{\\Fgdt;j}")),
    (210, FixtureValue::Double(0.0)),
    (220, FixtureValue::Double(0.0)),
    (230, FixtureValue::Double(1.0)),
    (11, FixtureValue::Double(1.0)),
    (21, FixtureValue::Double(0.0)),
    (31, FixtureValue::Double(0.0)),
];

#[test]
fn every_dialect_has_ascii_binary_field_evidence_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.text_symbol_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.text_symbol_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            signatures(DxfRawDocumentView::from_ascii(&ascii), &ascii_directory)?,
            signatures(DxfRawDocumentView::from_binary(&binary), &binary_directory)?
        );
        let expected_mtext = if version == DxfAcadVersion::Ac1009 {
            MTEXT_VALUES.iter().filter(|(code, _)| *code < 256).count()
        } else {
            MTEXT_VALUES.len()
        };
        assert_eq!(
            ascii_directory.records()[1].value_range().len(),
            expected_mtext as u64
        );
    }
    Ok(())
}

#[test]
fn invalid_duplicates_domains_and_exact_matching_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nTEXT\n40\n1\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\ntext\n40\n1\n0\nTEXT \n40\n1\n\
0\nTEXT\n40\n.\n40\n2\n71\n32768\n1\na\n1\nb\n90\n7\n\
0\nMTEXT\n90\n2147483648\n420\n.\n1\nx\n\
0\nSHAPE\n2\nBOLT\n71\n7\n\
0\nTOLERANCE\n3\nISO\n40\n2\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_symbol_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 4);
    let text = values(&directory, 0)?;
    assert_eq!(text.len(), 5);
    assert_invalid_double(text[0]);
    assert_invalid_i16(text[2]);
    assert_eq!(text[3].role(), DxfTextSymbolValueRole::Content);
    assert_eq!(text[4].role(), DxfTextSymbolValueRole::Content);

    let mtext = values(&directory, 1)?;
    assert_invalid_i32(mtext[0]);
    assert_invalid_i32(mtext[1]);
    assert_eq!(mtext[2].data(), DxfTextSymbolValueData::Text);
    assert_eq!(values(&directory, 2)?.len(), 1);
    assert_eq!(values(&directory, 3)?.len(), 1);
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    Ok(())
}

#[test]
fn cancellation_source_identity_lookups_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.text_symbol_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.text_symbol_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    for record in directory.records().iter().copied() {
        assert_eq!(
            directory.record_for_raw_ordinal(record.record().ordinal()),
            Some(record)
        );
        for value in directory
            .values_for_raw_record(record.record().ordinal())
            .ok_or(io::Error::other("values"))?
        {
            assert_eq!(
                directory.value_for_group(value.group().occurrence()),
                Some(*value)
            );
        }
    }
    assert_copy::<DxfTextSymbolValue>();
    assert_copy::<DxfTextSymbolRecordEntry>();
    assert_send_sync::<DxfTextSymbolDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfTextSymbolDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        directory
            .records()
            .iter()
            .map(|record| record.kind())
            .collect::<Vec<_>>(),
        [
            DxfTextSymbolKind::Text,
            DxfTextSymbolKind::MText,
            DxfTextSymbolKind::Shape,
            DxfTextSymbolKind::Tolerance
        ]
    );
    assert!(directory.raw_record_count() >= 4);
    for (index, record) in directory.records().iter().copied().enumerate() {
        assert_eq!(
            values(directory, index)?.len() as u64,
            record.value_range().len()
        );
    }
    Ok(())
}

fn values(
    directory: &DxfTextSymbolDirectory,
    index: usize,
) -> Result<&[DxfTextSymbolValue], Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .ok_or(io::Error::other("record"))?;
    directory
        .values_for_raw_record(record.record().ordinal())
        .ok_or_else(|| io::Error::other("values").into())
}

fn signatures(
    document: DxfRawDocumentView<'_>,
    directory: &DxfTextSymbolDirectory,
) -> Result<Vec<RecordSignature>, Box<dyn Error>> {
    directory
        .records()
        .iter()
        .copied()
        .map(|record| {
            let values = directory
                .values_for_raw_record(record.record().ordinal())
                .ok_or(io::Error::other("values"))?
                .iter()
                .copied()
                .map(|value| Ok((value.role(), signature(document, value)?)))
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
            Ok((record.kind(), values))
        })
        .collect()
}

fn signature(
    document: DxfRawDocumentView<'_>,
    value: DxfTextSymbolValue,
) -> Result<ValueSignature, Box<dyn Error>> {
    Ok(match value.data() {
        DxfTextSymbolValueData::Text => {
            let span = value.group().value_payload_span();
            let mut bytes = vec![0; usize::try_from(span.len())?];
            document.read_span(span, &mut bytes)?;
            ValueSignature::Text(bytes)
        }
        DxfTextSymbolValueData::Double(value) => {
            ValueSignature::Double(value.map_err(|_| io::Error::other("double"))?.to_bits())
        }
        DxfTextSymbolValueData::Int16(value) => {
            ValueSignature::Int16(value.map_err(|_| io::Error::other("i16"))?)
        }
        DxfTextSymbolValueData::Int32(value) => {
            ValueSignature::Int32(value.map_err(|_| io::Error::other("i32"))?)
        }
        _ => return Err(io::Error::other("unknown value data").into()),
    })
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_ascii(&mut bytes, 0, FixtureValue::Text(b"SECTION"));
    push_ascii(&mut bytes, 2, FixtureValue::Text(b"HEADER"));
    push_ascii(&mut bytes, 9, FixtureValue::Text(b"$ACADVER"));
    push_ascii(&mut bytes, 1, FixtureValue::Text(version.code().as_bytes()));
    push_ascii(&mut bytes, 0, FixtureValue::Text(b"ENDSEC"));
    push_ascii(&mut bytes, 0, FixtureValue::Text(b"SECTION"));
    push_ascii(&mut bytes, 2, FixtureValue::Text(b"BLOCKS"));
    append_ascii_record(&mut bytes, b"TEXT", TEXT_VALUES, version);
    push_ascii(&mut bytes, 0, FixtureValue::Text(b"ENDSEC"));
    push_ascii(&mut bytes, 0, FixtureValue::Text(b"SECTION"));
    push_ascii(&mut bytes, 2, FixtureValue::Text(b"ENTITIES"));
    append_ascii_record(&mut bytes, b"MTEXT", MTEXT_VALUES, version);
    append_ascii_record(&mut bytes, b"SHAPE", SHAPE_VALUES, version);
    append_ascii_record(&mut bytes, b"TOLERANCE", TOLERANCE_VALUES, version);
    push_ascii(&mut bytes, 0, FixtureValue::Text(b"ENDSEC"));
    push_ascii(&mut bytes, 0, FixtureValue::Text(b"EOF"));
    bytes
}

fn append_ascii_record(
    bytes: &mut Vec<u8>,
    marker: &'static [u8],
    values: &[(i16, FixtureValue)],
    version: DxfAcadVersion,
) {
    push_ascii(bytes, 0, FixtureValue::Text(marker));
    for (code, value) in values.iter().copied() {
        if version != DxfAcadVersion::Ac1009 || code < 256 {
            push_ascii(bytes, code, value);
        }
    }
}

fn push_ascii(bytes: &mut Vec<u8>, code: i16, value: FixtureValue) {
    bytes.extend_from_slice(code.to_string().as_bytes());
    bytes.push(b'\n');
    match value {
        FixtureValue::Text(value) => bytes.extend_from_slice(value),
        FixtureValue::Double(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        FixtureValue::Int16(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        FixtureValue::Int32(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
    }
    bytes.push(b'\n');
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary(&mut bytes, version, 0, FixtureValue::Text(b"SECTION"))?;
    push_binary(&mut bytes, version, 2, FixtureValue::Text(b"HEADER"))?;
    push_binary(&mut bytes, version, 9, FixtureValue::Text(b"$ACADVER"))?;
    push_binary(
        &mut bytes,
        version,
        1,
        FixtureValue::Text(version.code().as_bytes()),
    )?;
    push_binary(&mut bytes, version, 0, FixtureValue::Text(b"ENDSEC"))?;
    push_binary(&mut bytes, version, 0, FixtureValue::Text(b"SECTION"))?;
    push_binary(&mut bytes, version, 2, FixtureValue::Text(b"BLOCKS"))?;
    append_binary_record(&mut bytes, version, b"TEXT", TEXT_VALUES)?;
    push_binary(&mut bytes, version, 0, FixtureValue::Text(b"ENDSEC"))?;
    push_binary(&mut bytes, version, 0, FixtureValue::Text(b"SECTION"))?;
    push_binary(&mut bytes, version, 2, FixtureValue::Text(b"ENTITIES"))?;
    append_binary_record(&mut bytes, version, b"MTEXT", MTEXT_VALUES)?;
    append_binary_record(&mut bytes, version, b"SHAPE", SHAPE_VALUES)?;
    append_binary_record(&mut bytes, version, b"TOLERANCE", TOLERANCE_VALUES)?;
    push_binary(&mut bytes, version, 0, FixtureValue::Text(b"ENDSEC"))?;
    push_binary(&mut bytes, version, 0, FixtureValue::Text(b"EOF"))?;
    Ok(bytes)
}

fn append_binary_record(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    marker: &'static [u8],
    values: &[(i16, FixtureValue)],
) -> io::Result<()> {
    push_binary(bytes, version, 0, FixtureValue::Text(marker))?;
    for (code, value) in values.iter().copied() {
        if version != DxfAcadVersion::Ac1009 || code < 256 {
            push_binary(bytes, version, code, value)?;
        }
    }
    Ok(())
}

fn push_binary(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: FixtureValue,
) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    match value {
        FixtureValue::Text(value) => {
            bytes.extend_from_slice(value);
            bytes.push(0);
        }
        FixtureValue::Double(value) => bytes.extend_from_slice(&value.to_bits().to_le_bytes()),
        FixtureValue::Int16(value) => bytes.extend_from_slice(&value.to_le_bytes()),
        FixtureValue::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
    }
    Ok(())
}

fn assert_invalid_double(value: DxfTextSymbolValue) {
    assert!(matches!(
        value.data(),
        DxfTextSymbolValueData::Double(Err(DxfTextSymbolNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        )))
    ));
}

fn assert_invalid_i16(value: DxfTextSymbolValue) {
    assert!(matches!(
        value.data(),
        DxfTextSymbolValueData::Int16(Err(DxfTextSymbolNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        )))
    ));
}

fn assert_invalid_i32(value: DxfTextSymbolValue) {
    assert!(matches!(
        value.data(),
        DxfTextSymbolValueData::Int32(Err(DxfTextSymbolNumericIssue::InvalidAsciiNumber(_)))
    ));
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMTextColumnMode, DxfMTextColumnRelationDirectory,
    DxfMTextColumnSemanticDirectory, DxfMTextColumnSourceEntry, DxfMTextColumnType,
    DxfMTextFlatColumnDirectory, DxfMTextFlatColumnEntry, DxfMTextFlatColumnRole, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValue, DxfTextSymbolValueData,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    role: DxfMTextFlatColumnRole,
    bits: u64,
}

#[test]
fn every_dialect_has_ascii_binary_direct_column_evidence_parity() -> Result<(), Box<dyn Error>> {
    let expected = [
        double(DxfMTextFlatColumnRole::RotationOrColumnHeight, 0.5),
        integer(DxfMTextFlatColumnRole::ColumnType, 2),
        integer(DxfMTextFlatColumnRole::ColumnCount, 0),
        integer(DxfMTextFlatColumnRole::ColumnFlowReversed, 0),
        integer(DxfMTextFlatColumnRole::ColumnAutoHeight, 1),
        double(DxfMTextFlatColumnRole::ColumnWidth, 20.0),
        double(DxfMTextFlatColumnRole::ColumnGutter, 1.0),
        double(DxfMTextFlatColumnRole::RotationOrColumnHeight, 30.0),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_document(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        assert_eq!(
            signatures(&ascii.mtext_flat_column_directory(&cancellation)?)?,
            expected
        );
        assert_unified(
            &ascii.mtext_column_semantic_directory(&cancellation)?,
            &ascii.mtext_column_relation_directory(&cancellation)?,
        )?;

        let binary_bytes = binary_document(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        assert_eq!(
            signatures(&binary.mtext_flat_column_directory(&cancellation)?)?,
            expected
        );
        assert_unified(
            &binary.mtext_column_semantic_directory(&cancellation)?,
            &binary.mtext_column_relation_directory(&cancellation)?,
        )?;
    }
    Ok(())
}

#[test]
fn rotation_alone_is_not_a_column_and_embedded_values_do_not_leak_back()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n50\n0.5\n\
0\nMTEXT\n75\n2\n50\n30\n101\nEmbedded Object\n71\n1\n50\n99\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.mtext_flat_column_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 1);
    let entry = directory.entries()[0];
    assert_eq!(entry.marker().group_code().value(), 75);
    assert_eq!(
        directory
            .values_for_entry(entry)
            .ok_or_else(invalid_test_data)?
            .iter()
            .map(|value| value.role())
            .collect::<Vec<_>>(),
        [
            DxfMTextFlatColumnRole::ColumnType,
            DxfMTextFlatColumnRole::RotationOrColumnHeight,
        ]
    );
    Ok(())
}

#[test]
fn invalid_numeric_cancellation_lookup_identity_and_traits_remain_typed()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfMTextFlatColumnEntry>();
    assert_send_sync::<DxfMTextFlatColumnDirectory>();

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1021\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n75\nBAD\n50\nBAD\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.mtext_flat_column_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.mtext_flat_column_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.source_id(), document.source_id());
    let entry = directory.entries()[0];
    assert_eq!(
        directory.entry_for_record_ordinal(entry.record().ordinal()),
        Some(entry)
    );
    assert_eq!(directory.entry_for_record_ordinal(u64::MAX), None);
    assert_eq!(entry.value_count(), 2);
    for value in directory.values() {
        assert!(matches!(
            value.data(),
            DxfTextSymbolValueData::Int16(Err(_)) | DxfTextSymbolValueData::Double(Err(_))
        ));
    }
    Ok(())
}

fn signatures(directory: &DxfMTextFlatColumnDirectory) -> Result<Vec<Signature>, Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 1);
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            let bits = match value.data() {
                DxfTextSymbolValueData::Int16(Ok(number)) => number as i64 as u64,
                DxfTextSymbolValueData::Double(Ok(number)) => number.to_bits(),
                _ => return Err(invalid_test_data().into()),
            };
            Ok(Signature {
                role: value.role(),
                bits,
            })
        })
        .collect()
}

fn assert_unified(
    scalars: &DxfMTextColumnSemanticDirectory,
    relations: &DxfMTextColumnRelationDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(scalars.semantics().len(), 1);
    let scalar = scalars.semantics()[0];
    assert!(matches!(scalar.entry(), DxfMTextColumnSourceEntry::Flat(_)));
    assert_eq!(
        scalar.column_type().value(),
        Some(&DxfMTextColumnType::Dynamic)
    );
    assert_eq!(scalar.column_count().value(), Some(&0));
    assert_eq!(scalar.auto_height().value(), Some(&true));
    assert_eq!(scalar.individual_height_count(), 0);
    assert!(matches!(
        scalar.shared_height(),
        DxfSemanticValue::Absent { .. }
    ));
    assert_eq!(
        scalars
            .individual_heights(scalar)
            .ok_or_else(invalid_test_data)?,
        []
    );
    assert_eq!(
        relations.semantics()[0].mode().value(),
        Some(&DxfMTextColumnMode::DynamicAutomatic)
    );
    Ok(())
}

fn integer(role: DxfMTextFlatColumnRole, value: i16) -> Signature {
    Signature {
        role,
        bits: value as i64 as u64,
    }
}

fn double(role: DxfMTextFlatColumnRole, value: f64) -> Signature {
    Signature {
        role,
        bits: value.to_bits(),
    }
}

fn ascii_document(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nMTEXT\n50\n0.5\n75\n2\n76\n0\n\
78\n0\n79\n1\n48\n20\n49\n1\n50\n30\n0\nENDSEC\n0\nEOF\n"
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
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 50, 0.5)?;
    for (code, value) in [(75, 2), (76, 0), (78, 0), (79, 1)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 48, 20.0)?;
    push_double(&mut bytes, version, 49, 1.0)?;
    push_double(&mut bytes, version, 50, 30.0)?;
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
    if version == DxfAcadVersion::Ac1009 && (0..=255).contains(&code) {
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

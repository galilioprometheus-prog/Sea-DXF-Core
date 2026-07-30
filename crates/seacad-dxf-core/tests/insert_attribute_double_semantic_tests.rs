use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertAttributeDoubleSemanticDirectory, DxfInsertAttributeDoubleSemanticIssue,
    DxfInsertAttributeDoubleSemantics, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_ascii_binary_double_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_double_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_double_semantic_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            semantic_bits(&ascii_directory)?,
            semantic_bits(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn documented_defaults_optional_alignment_and_required_failures_are_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_double_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = directory
        .semantics_for_entry(directory.records()[0])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        states([
            semantics.thickness(),
            semantics.rotation_degrees(),
            semantics.relative_x_scale(),
            semantics.oblique_degrees(),
        ]),
        [
            DxfSemanticValueState::Defaulted,
            DxfSemanticValueState::Defaulted,
            DxfSemanticValueState::Defaulted,
            DxfSemanticValueState::Defaulted,
        ]
    );
    assert_double_bits(
        [
            semantics.thickness().value().copied(),
            semantics.rotation_degrees().value().copied(),
            semantics.relative_x_scale().value().copied(),
            semantics.oblique_degrees().value().copied(),
        ],
        [0.0, 0.0, 1.0, 0.0],
    )?;
    assert_eq!(
        states(semantics.text_start().each_ref()),
        [DxfSemanticValueState::Invalid; 3]
    );
    assert_eq!(
        semantics.text_height().state(),
        DxfSemanticValueState::Invalid
    );
    assert_eq!(
        states(semantics.alignment_point().each_ref()),
        [DxfSemanticValueState::Absent; 3]
    );
    assert_eq!(
        states(semantics.extrusion().each_ref()),
        [DxfSemanticValueState::Defaulted; 3]
    );
    assert_eq!(semantics.text_start_value(), None);
    assert_eq!(semantics.text_height_value(), None);
    assert_eq!(semantics.alignment_point_value(), None);
    assert_double_bits(
        semantics
            .extrusion_value()
            .ok_or_else(invalid_test_data)?
            .map(Some),
        [0.0, 0.0, 1.0],
    )?;
    Ok(())
}

#[test]
fn invalid_duplicates_and_partial_optional_tuple_fail_typed_with_provenance()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n10\nbad\n20\n2\n20\n3\n30\n4\n40\n5\n50\nbad\n11\n6\n31\nbad\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_double_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = directory
        .semantics_for_raw_record(directory.records()[0].record().ordinal())?
        .ok_or_else(invalid_test_data)?;
    assert!(matches!(
        semantics.text_start()[0].invalid_issue(),
        Some(DxfInsertAttributeDoubleSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { .. }
        ))
    ));
    assert!(matches!(
        semantics.text_start()[1].invalid_issue(),
        Some(DxfInsertAttributeDoubleSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    ));
    assert!(semantics.text_start()[0].raw_provenance().is_some());
    assert!(semantics.text_start()[1].raw_provenance().is_some());
    assert_eq!(
        semantics.rotation_degrees().state(),
        DxfSemanticValueState::Invalid
    );
    assert_ne!(
        semantics.rotation_degrees().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        states(semantics.alignment_point().each_ref()),
        [
            DxfSemanticValueState::Explicit,
            DxfSemanticValueState::Absent,
            DxfSemanticValueState::Invalid,
        ]
    );
    assert_eq!(semantics.alignment_point_value(), None);
    Ok(())
}

#[test]
fn cancellation_lookups_sequence_index_source_identity_and_public_traits_hold()
-> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeDoubleSemantics>();
    assert_send_sync::<DxfInsertAttributeDoubleSemanticDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_double_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.insert_attribute_double_semantic_directory(&DxfCancellationToken::default())?;
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
    assert_eq!(
        directory
            .semantics_for_insert_sequence_attribute(insert, 0)?
            .map(DxfInsertAttributeDoubleSemantics::record),
        Some(first)
    );
    assert!(
        directory
            .semantics_for_insert_sequence_attribute(insert, u64::MAX)?
            .is_none()
    );
    Ok(())
}

fn assert_directory(directory: &DxfInsertAttributeDoubleSemanticDirectory) -> Result<(), DxfError> {
    assert_eq!(directory.records().len(), 2);
    let full = directory
        .semantics_for_entry(directory.records()[0])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        states(full.text_start().each_ref()),
        [DxfSemanticValueState::Explicit; 3]
    );
    assert_eq!(full.text_height().state(), DxfSemanticValueState::Explicit);
    assert_eq!(
        states(full.alignment_point().each_ref()),
        [DxfSemanticValueState::Explicit; 3]
    );
    assert_eq!(
        states(full.extrusion().each_ref()),
        [DxfSemanticValueState::Explicit; 3]
    );
    assert_double_bits(
        full.text_start_value()
            .ok_or_else(invalid_test_data)?
            .map(Some),
        [2.0, 3.0, 4.0],
    )?;
    assert_double_bits(
        full.alignment_point_value()
            .ok_or_else(invalid_test_data)?
            .map(Some),
        [9.0, 10.0, 11.0],
    )?;
    assert_double_bits(
        full.extrusion_value()
            .ok_or_else(invalid_test_data)?
            .map(Some),
        [12.0, 13.0, 14.0],
    )?;
    let empty = directory
        .semantics_for_entry(directory.records()[1])?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        states(empty.text_start().each_ref()),
        [DxfSemanticValueState::Invalid; 3]
    );
    assert_eq!(
        states(empty.alignment_point().each_ref()),
        [DxfSemanticValueState::Absent; 3]
    );
    Ok(())
}

fn semantic_bits(
    directory: &DxfInsertAttributeDoubleSemanticDirectory,
) -> Result<Vec<Vec<Option<u64>>>, DxfError> {
    directory
        .records()
        .iter()
        .map(|record| {
            let semantics = directory
                .semantics_for_entry(*record)?
                .ok_or_else(invalid_test_data)?;
            Ok([
                [semantics.thickness().value().copied()].as_slice(),
                semantics
                    .text_start()
                    .each_ref()
                    .map(|value| value.value().copied())
                    .as_slice(),
                [semantics.text_height().value().copied()].as_slice(),
                [semantics.rotation_degrees().value().copied()].as_slice(),
                [semantics.relative_x_scale().value().copied()].as_slice(),
                [semantics.oblique_degrees().value().copied()].as_slice(),
                semantics
                    .alignment_point()
                    .each_ref()
                    .map(|value| value.value().copied())
                    .as_slice(),
                semantics
                    .extrusion()
                    .each_ref()
                    .map(|value| value.value().copied())
                    .as_slice(),
            ]
            .into_iter()
            .flatten()
            .map(|value| value.map(DxfDouble::to_bits))
            .collect())
        })
        .collect()
}

fn states<const N: usize>(
    values: [&seacad_dxf_core::DxfInsertAttributeSemanticDouble; N],
) -> [DxfSemanticValueState; N] {
    values.map(|value| value.state())
}

fn assert_double_bits<const N: usize>(
    actual: [Option<DxfDouble>; N],
    expected: [f64; N],
) -> Result<(), DxfError> {
    for (actual, expected) in actual.into_iter().zip(expected) {
        let actual = actual.ok_or_else(invalid_test_data)?;
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n39\n1\n10\n2\n20\n3\n30\n4\n40\n5\n50\n6\n41\n7\n51\n8\n11\n9\n21\n10\n31\n11\n210\n12\n220\n13\n230\n14\n0\nATTRIB\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [
        (39, 1.0),
        (10, 2.0),
        (20, 3.0),
        (30, 4.0),
        (40, 5.0),
        (50, 6.0),
        (41, 7.0),
        (51, 8.0),
        (11, 9.0),
        (21, 10.0),
        (31, 11.0),
        (210, 12.0),
        (220, 13.0),
        (230, 14.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
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

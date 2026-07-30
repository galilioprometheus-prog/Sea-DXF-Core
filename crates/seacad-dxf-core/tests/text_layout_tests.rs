use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, DxfTextGenerationFlags, DxfTextHorizontalJustification,
    DxfTextLayoutDirectory, DxfTextLayoutIssue, DxfTextLayoutSemantics,
    DxfTextVerticalJustification, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    flags: Option<u16>,
    horizontal: Option<i16>,
    vertical: Option<i16>,
    uses_second: Option<bool>,
}

#[test]
fn every_dialect_has_ascii_binary_text_layout_parity() -> Result<(), Box<dyn Error>> {
    let rows = [(0, 0, 0), (2, 5, 0), (4, 0, 3), (6, 3, 2)];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.text_layout_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.text_layout_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
        assert_eq!(
            signatures(&ascii_directory)?,
            [
                Signature {
                    flags: Some(0),
                    horizontal: Some(0),
                    vertical: Some(0),
                    uses_second: Some(false),
                },
                Signature {
                    flags: Some(2),
                    horizontal: Some(5),
                    vertical: Some(0),
                    uses_second: Some(true),
                },
                Signature {
                    flags: Some(4),
                    horizontal: Some(0),
                    vertical: Some(3),
                    uses_second: Some(true),
                },
                Signature {
                    flags: Some(6),
                    horizontal: Some(3),
                    vertical: Some(2),
                    uses_second: Some(true),
                },
            ]
        );
    }
    Ok(())
}

#[test]
fn documented_codes_and_exact_generation_bits_are_classified() -> Result<(), Box<dyn Error>> {
    let rows = [
        (0, 0, 0),
        (2, 1, 0),
        (4, 2, 0),
        (6, 3, 0),
        (-32768, 4, 0),
        (0, 5, 0),
        (0, 0, 1),
        (0, 0, 2),
        (0, 0, 3),
    ];
    let bytes = ascii_fixture("AC1032", &rows);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_layout_directory(&DxfCancellationToken::default())?;

    let expected_horizontal = [
        DxfTextHorizontalJustification::Left,
        DxfTextHorizontalJustification::Center,
        DxfTextHorizontalJustification::Right,
        DxfTextHorizontalJustification::Aligned,
        DxfTextHorizontalJustification::Middle,
        DxfTextHorizontalJustification::Fit,
    ];
    for (index, (record, expected)) in directory
        .records()
        .iter()
        .take(expected_horizontal.len())
        .zip(expected_horizontal)
        .enumerate()
    {
        let semantics = semantics(&directory, *record)?;
        assert_eq!(semantics.horizontal().value(), Some(&expected));
        assert_eq!(expected.code(), rows[index].1);
    }

    let expected_vertical = [
        DxfTextVerticalJustification::Bottom,
        DxfTextVerticalJustification::Middle,
        DxfTextVerticalJustification::Top,
    ];
    for (record, expected) in directory.records()[6..].iter().zip(expected_vertical) {
        let semantics = semantics(&directory, *record)?;
        assert_eq!(semantics.vertical().value(), Some(&expected));
        assert!(semantics.requires_second_alignment_point().unwrap_or(false));
    }

    let backward = semantics(&directory, directory.records()[1])?
        .generation_flags()
        .value()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert!(backward.is_backward());
    assert!(!backward.is_upside_down());
    assert_eq!(backward.unknown_bits(), 0);

    let combined = semantics(&directory, directory.records()[3])?
        .generation_flags()
        .value()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert!(combined.is_backward());
    assert!(combined.is_upside_down());
    assert_eq!(combined.source_value(), 6);

    let unknown = semantics(&directory, directory.records()[4])?
        .generation_flags()
        .value()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert_eq!(unknown.bits(), 0x8000);
    assert_eq!(unknown.unknown_bits(), 0x8000);
    assert_eq!(unknown.source_value(), i16::MIN);
    Ok(())
}

#[test]
fn unsupported_invalid_and_duplicate_codes_fail_typed_with_provenance() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n71\n-32768\n72\n6\n73\n-1\n\
0\nTEXT\n71\nbad\n72\nbad\n73\nbad\n\
0\nTEXT\n71\n2\n71\n4\n72\n1\n72\n2\n73\n1\n73\n2\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_layout_directory(&DxfCancellationToken::default())?;

    let unsupported = semantics(&directory, directory.records()[0])?;
    assert_eq!(
        unsupported.horizontal().invalid_issue(),
        Some(&DxfTextLayoutIssue::UnsupportedHorizontalCode { code: 6 })
    );
    assert_eq!(
        unsupported.vertical().invalid_issue(),
        Some(&DxfTextLayoutIssue::UnsupportedVerticalCode { code: -1 })
    );
    assert!(unsupported.horizontal().raw_provenance().is_some());
    assert!(unsupported.vertical().raw_provenance().is_some());
    assert_eq!(unsupported.requires_second_alignment_point(), None);
    assert_eq!(
        unsupported
            .generation_flags()
            .value()
            .map(|flags| flags.unknown_bits()),
        Some(0x8000)
    );

    let invalid = semantics(&directory, directory.records()[1])?;
    for issue in [
        invalid.generation_flags().invalid_issue(),
        invalid.horizontal().invalid_issue(),
        invalid.vertical().invalid_issue(),
    ] {
        assert!(matches!(issue, Some(DxfTextLayoutIssue::Scalar(_))));
    }

    let multiple = semantics(&directory, directory.records()[2])?;
    for value in [
        multiple.generation_flags().raw_provenance(),
        multiple.horizontal().raw_provenance(),
        multiple.vertical().raw_provenance(),
    ] {
        assert!(value.is_some());
    }
    assert!(matches!(
        multiple.horizontal().invalid_issue(),
        Some(DxfTextLayoutIssue::Scalar(
            seacad_dxf_core::DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2
            }
        ))
    ));
    Ok(())
}

#[test]
fn defaults_cancellation_scope_lookups_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfTextGenerationFlags>();
    assert_copy::<DxfTextLayoutSemantics>();
    assert_send_sync::<DxfTextLayoutDirectory>();

    let bytes = ascii_fixture("AC1032", &[]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.text_layout_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nTEXT\n0\nSHAPE\n2\nBOX\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_layout_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.scalar_directory().source_id()
    );

    let text = semantics(&directory, directory.records()[0])?;
    assert_eq!(
        text.generation_flags().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(text.horizontal().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(text.vertical().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(text.requires_second_alignment_point(), Some(false));
    assert_eq!(text.uses_first_alignment_point(), Some(true));

    assert_eq!(
        directory.semantics_for_record(directory.records()[1])?,
        None
    );
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    Ok(())
}

fn signatures(directory: &DxfTextLayoutDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(semantics)) => Some(Ok(Signature {
                flags: semantics
                    .generation_flags()
                    .value()
                    .map(|value| value.bits()),
                horizontal: semantics.horizontal().value().map(|value| value.code()),
                vertical: semantics.vertical().value().map(|value| value.code()),
                uses_second: semantics.requires_second_alignment_point(),
            })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfTextLayoutDirectory,
    record: seacad_dxf_core::DxfTextSymbolRecordEntry,
) -> Result<DxfTextLayoutSemantics, Box<dyn Error>> {
    directory
        .semantics_for_record(record)?
        .ok_or_else(|| invalid_test_data().into())
}

fn ascii_fixture(version: &str, rows: &[(i16, i16, i16)]) -> Vec<u8> {
    let mut entities = String::new();
    for (flags, horizontal, vertical) in rows {
        entities.push_str(&format!(
            "0\nTEXT\n71\n{flags}\n72\n{horizontal}\n73\n{vertical}\n"
        ));
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, rows: &[(i16, i16, i16)]) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    for (flags, horizontal, vertical) in rows {
        push_string(&mut bytes, version, 0, b"TEXT")?;
        push_i16(&mut bytes, version, 71, *flags)?;
        push_i16(&mut bytes, version, 72, *horizontal)?;
        push_i16(&mut bytes, version, 73, *vertical)?;
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

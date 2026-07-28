use std::{error::Error, io, num::NonZeroU64};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfError, DxfHandle, DxfHandleParseIssue, DxfHeaderHandleDirectory,
    DxfHeaderHandleEntry, DxfHeaderHandleIssue, DxfHeaderHandleValue, DxfMemorySource,
    DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
struct ExpectedHandleField {
    ordinal: u64,
    id: &'static str,
    name: &'static str,
    group_code: i16,
    parsed: u64,
    raw: &'static [u8],
}

const HANDLE_FIELDS: [ExpectedHandleField; 5] = [
    ExpectedHandleField {
        ordinal: 10,
        id: "handseed",
        name: "$HANDSEED",
        group_code: 5,
        parsed: 10,
        raw: b"00000a",
    },
    ExpectedHandleField {
        ordinal: 210,
        id: "cepsnid",
        name: "$CEPSNID",
        group_code: 390,
        parsed: 0,
        raw: b"0",
    },
    ExpectedHandleField {
        ordinal: 211,
        id: "dragvs",
        name: "$DRAGVS",
        group_code: 349,
        parsed: 0x00ab_cdef,
        raw: b"ABCdef",
    },
    ExpectedHandleField {
        ordinal: 212,
        id: "interfereobjvs",
        name: "$INTERFEREOBJVS",
        group_code: 345,
        parsed: 11,
        raw: b"000B",
    },
    ExpectedHandleField {
        ordinal: 213,
        id: "interferevpvs",
        name: "$INTERFEREVPVS",
        group_code: 346,
        parsed: 255,
        raw: b"Ff",
    },
];

#[test]
fn ascii_and_binary_preserve_handle_values_raw_spelling_and_schema_order()
-> Result<(), Box<dyn Error>> {
    let ascii_bytes = ascii_fixture(standard_handle_body());
    let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
    let ascii = open_ascii(&ascii_source)?;
    let ascii_directory = ascii.header_handle_directory(&DxfCancellationToken::default())?;
    assert_directory_shape(
        &ascii_directory,
        ascii.source_id(),
        DxfRawDocumentView::from(&ascii),
    )?;
    assert_eq!(
        explicit(&ascii_directory, "handseed")?.handle(),
        ascii
            .header_view()?
            .handseed()
            .value()
            .copied()
            .ok_or(io::Error::other("missing ASCII HANDSEED"))?
    );

    let binary_bytes = binary_fixture()?;
    let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
    let binary = open_binary(&binary_source)?;
    let binary_directory = binary.header_handle_directory(&DxfCancellationToken::default())?;
    assert_directory_shape(
        &binary_directory,
        binary.source_id(),
        DxfRawDocumentView::from(&binary),
    )?;
    assert_eq!(
        explicit(&binary_directory, "handseed")?.handle(),
        binary
            .header_view()?
            .handseed()
            .value()
            .copied()
            .ok_or(io::Error::other("missing Binary HANDSEED"))?
    );
    Ok(())
}

#[test]
fn structural_and_lexical_failures_remain_typed_and_source_anchored() -> Result<(), Box<dyn Error>>
{
    let cases = [
        (
            "9\n$CEPSNID\n5\nA\n",
            DxfHeaderHandleIssue::InvalidGroupCode(code(5)?),
        ),
        (
            "9\n$CEPSNID\n9\n$DRAGVS\n349\nA\n",
            DxfHeaderHandleIssue::MissingValue,
        ),
        (
            "9\n$CEPSNID\n390\nA\n390\nB\n",
            DxfHeaderHandleIssue::MultipleValueGroups {
                group_count: NonZeroU64::new(2).ok_or(io::Error::other("count"))?,
            },
        ),
        (
            "9\n$CEPSNID\n390\nA\n9\n$CEPSNID\n390\nB\n",
            DxfHeaderHandleIssue::MultipleVariables {
                occurrence_count: NonZeroU64::new(2).ok_or(io::Error::other("count"))?,
            },
        ),
        (
            "9\n$CEPSNID\n390\n12G4\n",
            DxfHeaderHandleIssue::InvalidHandle(DxfHandleParseIssue::InvalidDigit { offset: 2 }),
        ),
        (
            "9\n$CEPSNID\n390\n00000000000000000\n",
            DxfHeaderHandleIssue::InvalidHandle(DxfHandleParseIssue::TooLong),
        ),
    ];
    for (body, expected) in cases {
        let bytes = ascii_fixture(body);
        let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        let directory = document.header_handle_directory(&DxfCancellationToken::default())?;
        let value = directory
            .entry("cepsnid")
            .ok_or(io::Error::other("missing CEPSNID entry"))?
            .value();
        assert_eq!(value.state(), DxfSemanticValueState::Invalid);
        assert_eq!(value.invalid_issue(), Some(&expected));
        assert!(value.raw_provenance().is_some());
    }
    Ok(())
}

#[test]
fn absence_cancellation_and_source_identity_fail_closed() -> Result<(), Box<dyn Error>> {
    let first_bytes = ascii_fixture("9\n$DRAGVS\n349\n00aB\n");
    let first_source = DxfMemorySource::new(&first_bytes, DxfResourceProfile::Safe)?;
    let first = open_ascii(&first_source)?;
    let first_directory = first.header_handle_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        first_directory
            .entry("cepsnid")
            .ok_or(io::Error::other("missing CEPSNID entry"))?
            .value()
            .state(),
        DxfSemanticValueState::Absent
    );

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        first.header_handle_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let second_bytes = ascii_fixture("9\n$DRAGVS\n349\nB\n");
    let second_source = DxfMemorySource::new(&second_bytes, DxfResourceProfile::Safe)?;
    let second = open_ascii(&second_source)?;
    let value = explicit(&first_directory, "dragvs")?;
    let mut destination = [0_u8; 4];
    assert!(matches!(
        value.read_raw_spelling(DxfRawDocumentView::from(&second), &mut destination),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

#[test]
fn public_metadata_is_copy_send_sync_and_debug_redacted() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfHeaderHandleEntry>();
    assert_copy::<DxfHeaderHandleValue>();
    assert_copy::<DxfHeaderHandleIssue>();
    assert_send_sync::<DxfHeaderHandleDirectory>();
    let bytes = ascii_fixture("9\n$DRAGVS\n349\nABCDEF\n");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.header_handle_directory(&DxfCancellationToken::default())?;
    let debug = format!("{directory:?}");
    assert!(debug.contains("handle_field_count"));
    assert!(!debug.contains("ABCDEF"));
    Ok(())
}

fn assert_directory_shape(
    directory: &DxfHeaderHandleDirectory,
    source_id: seacad_dxf_core::DxfSourceId,
    document: DxfRawDocumentView<'_>,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.source_id(), source_id);
    assert_eq!(directory.schema_version(), "dxf.v1");
    assert_eq!(directory.entries().len(), HANDLE_FIELDS.len());
    for (entry, expected) in directory.entries().iter().zip(HANDLE_FIELDS) {
        assert_eq!(entry.schema_ordinal(), expected.ordinal);
        assert_eq!(entry.schema_field_id(), expected.id);
        assert_eq!(entry.dxf_name(), expected.name);
        assert_eq!(entry.group_code(), expected.group_code);
        assert_eq!(entry.value().state(), DxfSemanticValueState::Explicit);
        assert_eq!(
            entry.value().field_provenance().document_source_id(),
            source_id
        );
        assert_eq!(
            entry.value().field_provenance().schema_namespace(),
            "header"
        );
        assert_eq!(
            entry.value().field_provenance().schema_field_id(),
            expected.id
        );
        assert_eq!(directory.entry(expected.id), Some(entry));
        assert_eq!(
            directory.entry_at_schema_ordinal(expected.ordinal),
            Some(entry)
        );

        let value = explicit(directory, expected.id)?;
        assert_eq!(value.handle(), DxfHandle::from_u64(expected.parsed));
        let provenance = entry
            .value()
            .raw_provenance()
            .ok_or(io::Error::other("missing raw provenance"))?;
        assert_eq!(value.source_id(), source_id);
        assert_eq!(value.group_occurrence(), provenance.group_occurrence());
        assert_eq!(value.value_span(), provenance.value_span());
        let mut destination = vec![0_u8; expected.raw.len()];
        value.read_raw_spelling(document, &mut destination)?;
        assert_eq!(destination, expected.raw);
    }
    assert!(directory.entry_at_schema_ordinal(0).is_none());
    assert!(directory.entry_at_schema_ordinal(209).is_none());
    Ok(())
}

fn explicit(
    directory: &DxfHeaderHandleDirectory,
    id: &str,
) -> Result<DxfHeaderHandleValue, io::Error> {
    directory
        .entry(id)
        .and_then(|entry| entry.value().value())
        .copied()
        .ok_or(io::Error::other("missing explicit handle"))
}

fn standard_handle_body() -> &'static str {
    "9\n$HANDSEED\n5\n00000a\n9\n$CEPSNID\n390\n0\n9\n$DRAGVS\n349\nABCdef\n9\n$INTERFEREOBJVS\n345\n000B\n9\n$INTERFEREVPVS\n346\nFf\n"
}

fn ascii_fixture(body: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n{body}0\nENDSEC\n0\nEOF\n").into_bytes()
}

fn binary_fixture() -> Result<Vec<u8>, io::Error> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (group_code, value) in [
        (0_i16, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, b"AC1032"),
    ] {
        push_binary_string(&mut bytes, group_code, value);
    }
    for (name, group_code, value) in [
        (b"$HANDSEED".as_slice(), 5_i16, b"00000a".as_slice()),
        (b"$CEPSNID", 390, b"0"),
        (b"$DRAGVS", 349, b"ABCdef"),
        (b"$INTERFEREOBJVS", 345, b"000B"),
        (b"$INTERFEREVPVS", 346, b"Ff"),
    ] {
        push_binary_string(&mut bytes, 9, name);
        push_binary_string(&mut bytes, group_code, value);
    }
    for (group_code, value) in [(0_i16, b"ENDSEC".as_slice()), (0, b"EOF")] {
        push_binary_string(&mut bytes, group_code, value);
    }
    assert_eq!(version.code(), "AC1032");
    Ok(bytes)
}

fn push_binary_string(bytes: &mut Vec<u8>, group_code: i16, value: &[u8]) {
    bytes.extend_from_slice(&group_code.to_le_bytes());
    bytes.extend_from_slice(value);
    bytes.push(0);
}

fn open_ascii<'a>(source: &'a DxfMemorySource<'_>) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a DxfMemorySource<'_>) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn code(value: i16) -> Result<seacad_dxf_core::DxfGroupCode, io::Error> {
    seacad_dxf_core::DxfGroupCode::new(value).ok_or(io::Error::other("group code"))
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

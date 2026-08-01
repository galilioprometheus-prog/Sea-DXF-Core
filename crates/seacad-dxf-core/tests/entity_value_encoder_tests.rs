use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_ENTITY_BINARY_CHUNK_MAX_BYTES, DxfAcadVersion, DxfAcadVersionState,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble,
    DxfEncodedEntityGroup, DxfEntityEditValue, DxfEntityField, DxfEntityFieldDescriptor,
    DxfEntityGroupEncodeIssue, DxfEntityGroupEncoder, DxfError, DxfHandle, DxfMemorySource,
    DxfRawDocumentFormat, DxfReadOptions, DxfResource, DxfResourceProfile, NoopDxfReadObserver,
    dxf_entity_common_fields,
};

#[test]
fn every_dialect_emits_strict_ascii_and_binary_documents() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = encoded_fixture(DxfRawDocumentFormat::Ascii, version)?;
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source)?;
        assert_eq!(
            document.acad_version_report().state(),
            DxfAcadVersionState::Supported(version)
        );
        assert_eq!(
            document
                .entity_directory(&DxfCancellationToken::default())?
                .entities()
                .len(),
            1
        );

        let binary = encoded_fixture(DxfRawDocumentFormat::Binary, version)?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let document = open_binary(&source)?;
        assert_eq!(
            document.acad_version_report().state(),
            DxfAcadVersionState::Supported(version)
        );
        assert_eq!(
            document
                .entity_directory(&DxfCancellationToken::default())?
                .entities()
                .len(),
            1
        );
    }

    let issue = encode_issue(
        DxfRawDocumentFormat::Binary,
        DxfAcadVersion::Ac1009,
        DxfEntityField::TRUE_COLOR,
        DxfEntityEditValue::Int32(0x12_34_56),
    )?;
    assert_eq!(
        issue,
        DxfEntityGroupEncodeIssue::GroupCodeUnavailableInDialect {
            group_code: 420,
            version: DxfAcadVersion::Ac1009,
        }
    );
    Ok(())
}

#[test]
fn canonical_bytes_cover_every_supported_value_wire_type() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"Layer"),
            b"8\nLayer\n".as_slice(),
            [8, 0, b'L', b'a', b'y', b'e', b'r', 0].as_slice(),
        ),
        (
            DxfEntityField::COLOR,
            DxfEntityEditValue::Int16(i16::MIN),
            b"62\n-32768\n",
            [62, 0, 0, 128].as_slice(),
        ),
        (
            DxfEntityField::PROXY_GRAPHICS_SIZE,
            DxfEntityEditValue::Int32(i32::MIN),
            b"92\n-2147483648\n",
            [92, 0, 0, 0, 0, 128].as_slice(),
        ),
        (
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(-0.0)),
            b"48\n-0\n",
            [48, 0, 0, 0, 0, 0, 0, 0, 0, 128].as_slice(),
        ),
        (
            DxfEntityField::HANDLE,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(u64::MAX)),
            b"5\nFFFFFFFFFFFFFFFF\n",
            b"\x05\0FFFFFFFFFFFFFFFF\0",
        ),
        (
            DxfEntityField::PROXY_GRAPHICS_DATA,
            DxfEntityEditValue::BinaryChunk(&[0, 0xab, 0xff]),
            b"310\n00ABFF\n",
            [54, 1, 3, 0, 0xab, 0xff].as_slice(),
        ),
    ];
    for (field, value, expected_ascii, expected_binary) in cases {
        let ascii = encode_group(
            DxfRawDocumentFormat::Ascii,
            DxfAcadVersion::Ac1032,
            field,
            value,
        )?;
        assert_eq!(ascii.bytes(), expected_ascii);
        let binary = encode_group(
            DxfRawDocumentFormat::Binary,
            DxfAcadVersion::Ac1032,
            field,
            value,
        )?;
        assert_eq!(binary.bytes(), expected_binary);
        assert_eq!(binary.descriptor(), descriptor(field)?);
        assert_eq!(binary.format(), DxfRawDocumentFormat::Binary);
        assert_eq!(binary.version(), DxfAcadVersion::Ac1032);
    }
    Ok(())
}

#[test]
fn invalid_values_and_resource_limits_fail_closed() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        encode_issue(
            DxfRawDocumentFormat::Ascii,
            DxfAcadVersion::Ac1032,
            DxfEntityField::COLOR,
            DxfEntityEditValue::Int32(1),
        )?,
        DxfEntityGroupEncodeIssue::WireTypeMismatch {
            expected: seacad_dxf_core::DxfEntityFieldWireType::Int16,
            observed: seacad_dxf_core::DxfEntityEditValueKind::Int32,
        }
    );
    let nan = DxfDouble::from_f64(f64::NAN);
    assert!(matches!(
        encode_issue(
            DxfRawDocumentFormat::Binary,
            DxfAcadVersion::Ac1032,
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(nan),
        )?,
        DxfEntityGroupEncodeIssue::NonFiniteDouble(value) if value.to_f64().is_nan()
    ));
    for (text, offset, byte) in [
        (b"A\0B".as_slice(), 1, 0),
        (b"\r", 0, b'\r'),
        (b"\n", 0, b'\n'),
    ] {
        assert_eq!(
            encode_issue(
                DxfRawDocumentFormat::Ascii,
                DxfAcadVersion::Ac1032,
                DxfEntityField::LAYER,
                DxfEntityEditValue::ExactRawText(text),
            )?,
            DxfEntityGroupEncodeIssue::ForbiddenTextByte { offset, byte }
        );
    }
    let oversized_chunk = [0_u8; DXF_ENTITY_BINARY_CHUNK_MAX_BYTES as usize + 1];
    assert_eq!(
        encode_issue(
            DxfRawDocumentFormat::Binary,
            DxfAcadVersion::Ac1032,
            DxfEntityField::PROXY_GRAPHICS_DATA,
            DxfEntityEditValue::BinaryChunk(&oversized_chunk),
        )?,
        DxfEntityGroupEncodeIssue::BinaryChunkTooLong {
            limit: DXF_ENTITY_BINARY_CHUNK_MAX_BYTES,
            observed: DXF_ENTITY_BINARY_CHUNK_MAX_BYTES + 1,
        }
    );

    let limit = DxfResourceProfile::Safe.limits().max_value_bytes();
    let large = vec![b'A'; usize::try_from(limit + 1)?];
    let result = DxfEntityGroupEncoder::new(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfResourceProfile::Safe,
    )
    .encode(
        descriptor(DxfEntityField::LAYER)?,
        DxfEntityEditValue::ExactRawText(&large),
        &DxfCancellationToken::default(),
    );
    assert!(matches!(
        result,
        Err(DxfError::ResourceLimitExceeded {
            resource: DxfResource::ValueBytes,
            limit: observed_limit,
            observed,
        }) if observed_limit == limit && observed == limit + 1
    ));
    Ok(())
}

#[test]
fn cancellation_traits_and_debug_output_are_bounded() -> Result<(), Box<dyn Error>> {
    assert_copy_send_sync::<DxfEntityGroupEncoder>();
    assert_copy_send_sync::<DxfEntityGroupEncodeIssue>();
    assert_send_sync::<DxfEncodedEntityGroup>();

    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    let result = DxfEntityGroupEncoder::new(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfResourceProfile::Safe,
    )
    .encode(
        descriptor(DxfEntityField::LAYER)?,
        DxfEntityEditValue::ExactRawText(b"SECRET_LAYER"),
        &cancellation,
    );
    assert!(matches!(result, Err(DxfError::Cancelled)));

    let encoded = encode_group(
        DxfRawDocumentFormat::Ascii,
        DxfAcadVersion::Ac1032,
        DxfEntityField::LAYER,
        DxfEntityEditValue::ExactRawText(b"SECRET_LAYER"),
    )?;
    let debug = format!("{encoded:?}");
    assert!(!debug.contains("SECRET_LAYER"));
    assert!(debug.contains("byte_count"));
    Ok(())
}

fn encoded_fixture(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = match format {
        DxfRawDocumentFormat::Ascii => Vec::new(),
        DxfRawDocumentFormat::Binary => DXF_BINARY_SENTINEL.to_vec(),
        _ => return Err(io::Error::other("unsupported raw format").into()),
    };
    for (code, text) in [
        (0, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version.code()),
        (0, "ENDSEC"),
        (0, "SECTION"),
        (2, "ENTITIES"),
        (0, "LINE"),
    ] {
        push_text_group(&mut bytes, format, version, code, text)?;
    }
    for (field, value) in [
        (
            DxfEntityField::HANDLE,
            DxfEntityEditValue::Handle(DxfHandle::from_u64(0x10)),
        ),
        (
            DxfEntityField::LAYER,
            DxfEntityEditValue::ExactRawText(b"Layer"),
        ),
        (DxfEntityField::COLOR, DxfEntityEditValue::Int16(7)),
        (
            DxfEntityField::LINETYPE_SCALE,
            DxfEntityEditValue::Double(DxfDouble::from_f64(1.25)),
        ),
        (
            DxfEntityField::PROXY_GRAPHICS_SIZE,
            DxfEntityEditValue::Int32(3),
        ),
    ] {
        bytes.extend_from_slice(encode_group(format, version, field, value)?.bytes());
    }
    if version != DxfAcadVersion::Ac1009 {
        bytes.extend_from_slice(
            encode_group(
                format,
                version,
                DxfEntityField::TRUE_COLOR,
                DxfEntityEditValue::Int32(0x12_34_56),
            )?
            .bytes(),
        );
        bytes.extend_from_slice(
            encode_group(
                format,
                version,
                DxfEntityField::PROXY_GRAPHICS_DATA,
                DxfEntityEditValue::BinaryChunk(&[1, 2, 3]),
            )?
            .bytes(),
        );
    }
    for (code, text) in [(0, "ENDSEC"), (0, "EOF")] {
        push_text_group(&mut bytes, format, version, code, text)?;
    }
    Ok(bytes)
}

fn encode_group(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> Result<DxfEncodedEntityGroup, Box<dyn Error>> {
    DxfEntityGroupEncoder::new(format, version, DxfResourceProfile::Safe)
        .encode(descriptor(field)?, value, &DxfCancellationToken::default())?
        .map_err(|issue| io::Error::other(format!("encode issue: {issue:?}")).into())
}

fn encode_issue(
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    field: DxfEntityField,
    value: DxfEntityEditValue<'_>,
) -> Result<DxfEntityGroupEncodeIssue, Box<dyn Error>> {
    match DxfEntityGroupEncoder::new(format, version, DxfResourceProfile::Safe).encode(
        descriptor(field)?,
        value,
        &DxfCancellationToken::default(),
    )? {
        Ok(_) => Err(io::Error::other("value unexpectedly encoded").into()),
        Err(issue) => Ok(issue),
    }
}

fn descriptor(field: DxfEntityField) -> Result<DxfEntityFieldDescriptor, io::Error> {
    dxf_entity_common_fields()
        .iter()
        .copied()
        .find(|item| item.field() == field)
        .ok_or_else(|| io::Error::other("missing common field descriptor"))
}

fn push_text_group(
    bytes: &mut Vec<u8>,
    format: DxfRawDocumentFormat,
    version: DxfAcadVersion,
    code: i16,
    value: &str,
) -> Result<(), io::Error> {
    match format {
        DxfRawDocumentFormat::Ascii => {
            bytes.extend_from_slice(format!("{code}\n{value}\n").as_bytes())
        }
        DxfRawDocumentFormat::Binary => {
            if version == DxfAcadVersion::Ac1009 {
                bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
            } else {
                bytes.extend_from_slice(&code.to_le_bytes());
            }
            bytes.extend_from_slice(value.as_bytes());
            bytes.push(0);
        }
        _ => return Err(io::Error::other("unsupported raw format")),
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

fn assert_copy_send_sync<T: Copy + Send + Sync>() {}
fn assert_send_sync<T: Send + Sync>() {}

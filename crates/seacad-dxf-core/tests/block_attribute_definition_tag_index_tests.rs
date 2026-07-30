use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockAttributeDefinitionTagIndexBlock, DxfBlockAttributeDefinitionTagIndexDirectory,
    DxfBlockAttributeDefinitionTagIndexLookup, DxfBlockAttributeDefinitionTagIndexMatch,
    DxfByteSource, DxfCancellationToken, DxfError, DxfInsertAttributeTextSemanticDirectory,
    DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct IndexSignature {
    block_counts: Vec<(u64, u64, u64)>,
    b_a: Vec<u64>,
    b_b: Vec<u64>,
    c_a: Vec<u64>,
}

#[test]
fn every_supported_dialect_has_ascii_binary_block_local_tag_index_parity()
-> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii
            .block_attribute_definition_tag_index_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary
            .block_attribute_definition_tag_index_directory(&DxfCancellationToken::default())?;

        let ascii_signature = signature(DxfRawDocumentView::from(&ascii), &ascii_directory)?;
        let binary_signature = signature(DxfRawDocumentView::from(&binary), &binary_directory)?;
        assert_eq!(ascii_signature, binary_signature);
        assert_eq!(
            ascii_signature.block_counts,
            [(4, 3, 1), (1, 1, 0), (0, 0, 0)]
        );
        assert_eq!(ascii_signature.b_a, [0, 1]);
        assert_eq!(ascii_signature.b_b, [2]);
        assert_eq!(ascii_signature.c_a, [0]);
    }
    Ok(())
}

#[test]
fn same_document_attrib_tag_span_drives_exact_lookup_without_payload_buffer()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let index =
        document.block_attribute_definition_tag_index_directory(&DxfCancellationToken::default())?;
    let attributes =
        document.insert_attribute_text_semantic_directory(&DxfCancellationToken::default())?;
    let block = index.blocks()[0].block().block_record().ordinal();
    let attribute = attribute_tag(&attributes, 0)?;
    let matches = index.matches_for_block_exact_source_span(
        view,
        block,
        attribute.value_span(),
        &DxfCancellationToken::default(),
    )?;
    assert_eq!(definition_ordinals(matches), [0, 1]);

    let lowercase = attribute_tag(&attributes, 1)?;
    assert!(
        index
            .matches_for_block_exact_source_span(
                view,
                block,
                lowercase.value_span(),
                &DxfCancellationToken::default(),
            )?
            .is_empty()
    );
    Ok(())
}

#[test]
fn duplicate_missing_unusable_and_empty_block_evidence_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let index =
        document.block_attribute_definition_tag_index_directory(&DxfCancellationToken::default())?;
    assert_eq!(index.blocks().len(), 3);
    assert_eq!(index.matches().len(), 4);
    let b = index.blocks()[0];
    assert_eq!(b.definition_count(), 4);
    assert_eq!(b.indexed_tag_count(), 3);
    assert_eq!(b.unusable_tag_count(), 1);
    assert!(matches!(
        index.lookup(
            view,
            b.block().block_record().ordinal(),
            b"A",
            &DxfCancellationToken::default()
        )?,
        DxfBlockAttributeDefinitionTagIndexLookup::Ambiguous(matches)
            if matches.len() == 2
    ));
    assert!(matches!(
        index.lookup(
            view,
            b.block().block_record().ordinal(),
            b"B",
            &DxfCancellationToken::default()
        )?,
        DxfBlockAttributeDefinitionTagIndexLookup::Unique(_)
    ));
    assert!(matches!(
        index.lookup(
            view,
            b.block().block_record().ordinal(),
            b"MISSING",
            &DxfCancellationToken::default()
        )?,
        DxfBlockAttributeDefinitionTagIndexLookup::Missing
    ));
    let empty = index.blocks()[2];
    assert_eq!(empty.definition_count(), 0);
    assert_eq!(empty.indexed_tag_count(), 0);
    assert_eq!(empty.unusable_tag_count(), 0);
    Ok(())
}

#[test]
fn cancellation_source_identity_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfBlockAttributeDefinitionTagIndexBlock>();
    assert_copy::<DxfBlockAttributeDefinitionTagIndexMatch>();
    assert_send_sync::<DxfBlockAttributeDefinitionTagIndexDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.block_attribute_definition_tag_index_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let index =
        document.block_attribute_definition_tag_index_directory(&DxfCancellationToken::default())?;
    assert_eq!(index.source_id(), index.semantic_directory().source_id());
    let lookup_cancellation = DxfCancellationToken::default();
    lookup_cancellation.cancel();
    assert!(matches!(
        index.matches_for_block_exact_tag(
            DxfRawDocumentView::from(&document),
            index.blocks()[0].block().block_record().ordinal(),
            b"A",
            &lookup_cancellation,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(index.block_for_raw_ordinal(u64::MAX).is_none());
    assert!(
        index
            .matches_for_block_exact_tag(
                DxfRawDocumentView::from(&document),
                u64::MAX,
                b"A",
                &DxfCancellationToken::default(),
            )?
            .is_empty()
    );

    let other_bytes = ascii_fixture("AC1027");
    let other_source = DxfMemorySource::new(&other_bytes, DxfResourceProfile::Safe)?;
    let other = open_ascii(&other_source)?;
    assert!(matches!(
        index.matches_for_block_exact_tag(
            DxfRawDocumentView::from(&other),
            index.blocks()[0].block().block_record().ordinal(),
            b"A",
            &DxfCancellationToken::default()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn signature(
    document: DxfRawDocumentView<'_>,
    directory: &DxfBlockAttributeDefinitionTagIndexDirectory,
) -> Result<IndexSignature, DxfError> {
    let block_counts = directory
        .blocks()
        .iter()
        .map(|block| {
            (
                block.definition_count(),
                block.indexed_tag_count(),
                block.unusable_tag_count(),
            )
        })
        .collect();
    let b = directory.blocks()[0].block().block_record().ordinal();
    let c = directory.blocks()[1].block().block_record().ordinal();
    Ok(IndexSignature {
        block_counts,
        b_a: definition_ordinals(directory.matches_for_block_exact_tag(
            document,
            b,
            b"A",
            &DxfCancellationToken::default(),
        )?),
        b_b: definition_ordinals(directory.matches_for_block_exact_tag(
            document,
            b,
            b"B",
            &DxfCancellationToken::default(),
        )?),
        c_a: definition_ordinals(directory.matches_for_block_exact_tag(
            document,
            c,
            b"A",
            &DxfCancellationToken::default(),
        )?),
    })
}

fn definition_ordinals(matches: &[DxfBlockAttributeDefinitionTagIndexMatch]) -> Vec<u64> {
    matches
        .iter()
        .map(|indexed| indexed.record().definition().attribute_definition_ordinal())
        .collect()
}

fn attribute_tag(
    directory: &DxfInsertAttributeTextSemanticDirectory,
    ordinal: usize,
) -> Result<seacad_dxf_core::DxfInsertAttributeTextValue, DxfError> {
    directory
        .semantics_for_entry(directory.records()[ordinal])?
        .ok_or_else(invalid_test_data)?
        .attribute_tag()
        .value()
        .copied()
        .ok_or_else(invalid_test_data)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nB\n0\nATTDEF\n2\nA\n0\nATTDEF\n2\nA\n0\nATTDEF\n2\nB\n0\nATTDEF\n0\nENDBLK\n0\nBLOCK\n2\nC\n0\nATTDEF\n2\nA\n0\nENDBLK\n0\nBLOCK\n2\nZ\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n66\n1\n0\nATTRIB\n1\nvalue\n2\nA\n70\n0\n0\nATTRIB\n1\nvalue\n2\na\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_block(
        &mut bytes,
        version,
        b"B",
        &[Some(b"A"), Some(b"A"), Some(b"B"), None],
    )?;
    push_block(&mut bytes, version, b"C", &[Some(b"A")])?;
    push_block(&mut bytes, version, b"Z", &[])?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"B")?;
    push_i16(&mut bytes, version, 66, 1)?;
    push_attribute(&mut bytes, version, b"A")?;
    push_attribute(&mut bytes, version, b"a")?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_block(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
    tags: &[Option<&[u8]>],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"BLOCK")?;
    push_string(bytes, version, 2, name)?;
    for tag in tags {
        push_string(bytes, version, 0, b"ATTDEF")?;
        if let Some(tag) = tag {
            push_string(bytes, version, 2, tag)?;
        }
    }
    push_string(bytes, version, 0, b"ENDBLK")
}

fn push_attribute(bytes: &mut Vec<u8>, version: DxfAcadVersion, tag: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"ATTRIB")?;
    push_string(bytes, version, 1, b"value")?;
    push_string(bytes, version, 2, tag)?;
    push_i16(bytes, version, 70, 0)
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

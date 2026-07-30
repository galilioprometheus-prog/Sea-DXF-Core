use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockDefinitionState, DxfBlockNameConsistencyState, DxfBlockNameIndexDirectory,
    DxfBlockNameIndexLookup, DxfBlockNameIndexMatch, DxfByteSource, DxfCancellationToken, DxfError,
    DxfMemorySource, DxfRawDocumentView, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_ascii_binary_exact_lookup_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_index = ascii.block_name_index_directory(&DxfCancellationToken::default())?;
        assert_index(DxfRawDocumentView::from(&ascii), &ascii_index)?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_index = binary.block_name_index_directory(&DxfCancellationToken::default())?;
        assert_index(DxfRawDocumentView::from(&binary), &binary_index)?;
    }
    Ok(())
}

#[test]
fn empty_case_utf8_and_long_names_remain_byte_exact() -> Result<(), Box<dyn Error>> {
    let long = vec![b'x'; 4 * 1024 + 1];
    let mut bytes = ascii_prefix("AC1032");
    for (primary, secondary) in [
        (b"".as_slice(), b"".as_slice()),
        (b"Case".as_slice(), b"Case".as_slice()),
        (b"case".as_slice(), b"case".as_slice()),
        ("Biển".as_bytes(), "Biển".as_bytes()),
        (long.as_slice(), long.as_slice()),
        (b"Conflict".as_slice(), b"Other".as_slice()),
    ] {
        push_ascii_block(&mut bytes, &[(2, primary), (3, secondary)], true);
    }
    push_ascii_block(
        &mut bytes,
        &[(2, b"Duplicate"), (2, b"Duplicate"), (3, b"Duplicate")],
        true,
    );
    ascii_suffix(&mut bytes);

    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let index = document.block_name_index_directory(&DxfCancellationToken::default())?;
    assert_eq!(index.matches().len(), 5);
    for exact_name in [
        b"".as_slice(),
        b"Case".as_slice(),
        b"case".as_slice(),
        "Biển".as_bytes(),
        long.as_slice(),
    ] {
        assert!(matches!(
            index.lookup(view, exact_name, &DxfCancellationToken::default())?,
            DxfBlockNameIndexLookup::Unique(_)
        ));
    }
    for absent in [
        b"CASE".as_slice(),
        b"Conflict".as_slice(),
        b"Other".as_slice(),
        b"Duplicate".as_slice(),
    ] {
        assert_eq!(
            index.lookup(view, absent, &DxfCancellationToken::default())?,
            DxfBlockNameIndexLookup::Missing
        );
    }
    assert_eq!(
        index
            .consistency_directory()
            .entries()
            .iter()
            .map(|entry| entry.state())
            .collect::<Vec<_>>(),
        [
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::Matched,
            DxfBlockNameConsistencyState::Conflicting,
            DxfBlockNameConsistencyState::NotComparable,
        ]
    );
    Ok(())
}

#[test]
fn duplicate_names_are_contiguous_and_preserve_source_order() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nSame\n3\nSame\n0\nENDBLK\n0\nBLOCK\n2\nOther\n3\nOther\n0\nENDBLK\n0\nBLOCK\n2\nSame\n3\nSame\n0\nENDBLK\n0\nBLOCK\n2\nSame\n3\nSame\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let index = document.block_name_index_directory(&DxfCancellationToken::default())?;

    let matches = index.matches_for_exact_name(view, b"Same", &DxfCancellationToken::default())?;
    assert_eq!(matches.len(), 3);
    assert!(matches.windows(2).all(|pair| {
        pair[0].record().definition().block_record().ordinal()
            < pair[1].record().definition().block_record().ordinal()
    }));
    match index.lookup(view, b"Same", &DxfCancellationToken::default())? {
        DxfBlockNameIndexLookup::Ambiguous(ambiguous) => assert_eq!(ambiguous, matches),
        other => return Err(io::Error::other(format!("unexpected lookup: {other:?}")).into()),
    }
    assert_eq!(
        index
            .matches_for_exact_name(view, b"Missing", &DxfCancellationToken::default())?
            .len(),
        0
    );
    Ok(())
}

#[test]
fn exact_index_keeps_every_block_definition_state() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n3\nA\n0\nLINE\n0\nBLOCK\n2\nB\n3\nB\n0\nLINE\n0\nENDBLK\n0\nBLOCK\n2\nC\n3\nC\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let view = DxfRawDocumentView::from(&document);
    let index = document.block_name_index_directory(&DxfCancellationToken::default())?;
    let expected = [
        (b"A".as_slice(), DxfBlockDefinitionState::Interrupted),
        (b"B".as_slice(), DxfBlockDefinitionState::Closed),
        (b"C".as_slice(), DxfBlockDefinitionState::Unclosed),
    ];
    for (name, state) in expected {
        match index.lookup(view, name, &DxfCancellationToken::default())? {
            DxfBlockNameIndexLookup::Unique(indexed) => {
                assert_eq!(indexed.record().definition().state(), state);
            }
            other => return Err(io::Error::other(format!("unexpected lookup: {other:?}")).into()),
        }
    }
    Ok(())
}

#[test]
fn cancellation_source_identity_and_public_traits_are_enforced() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfBlockNameIndexMatch>();
    assert_send_sync::<DxfBlockNameIndexDirectory>();
    assert!(std::mem::size_of::<DxfBlockNameIndexMatch>() <= 256);

    let first_bytes = ascii_fixture("AC1032");
    let first_source = DxfMemorySource::new(&first_bytes, DxfResourceProfile::Safe)?;
    let first = open_ascii(&first_source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        first.block_name_index_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let index = first.block_name_index_directory(&DxfCancellationToken::default())?;
    assert_eq!(index.source_id(), index.consistency_directory().source_id());
    assert!(matches!(
        index.lookup(DxfRawDocumentView::from(&first), b"A", &cancellation),
        Err(DxfError::Cancelled)
    ));

    let second_bytes = ascii_fixture("AC1027");
    let second_source = DxfMemorySource::new(&second_bytes, DxfResourceProfile::Safe)?;
    let second = open_ascii(&second_source)?;
    assert!(matches!(
        index.lookup(
            DxfRawDocumentView::from(&second),
            b"A",
            &DxfCancellationToken::default()
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    Ok(())
}

fn assert_index(
    view: DxfRawDocumentView<'_>,
    index: &DxfBlockNameIndexDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(index.source_id(), view.source_id());
    assert_eq!(index.matches().len(), 3);
    match index.lookup(view, b"A", &DxfCancellationToken::default())? {
        DxfBlockNameIndexLookup::Ambiguous(matches) => {
            assert_eq!(matches.len(), 2);
            assert!(
                matches[0].record().definition().block_record().ordinal()
                    < matches[1].record().definition().block_record().ordinal()
            );
        }
        other => return Err(io::Error::other(format!("unexpected lookup: {other:?}")).into()),
    }
    assert!(matches!(
        index.lookup(view, b"B", &DxfCancellationToken::default())?,
        DxfBlockNameIndexLookup::Unique(_)
    ));
    for missing in [b"C".as_slice(), b"D".as_slice(), b"E".as_slice()] {
        assert_eq!(
            index.lookup(view, missing, &DxfCancellationToken::default())?,
            DxfBlockNameIndexLookup::Missing
        );
    }
    Ok(())
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n3\nA\n0\nENDBLK\n0\nBLOCK\n2\nB\n3\nB\n0\nENDBLK\n0\nBLOCK\n2\nA\n3\nA\n0\nENDBLK\n0\nBLOCK\n2\nC\n3\nD\n0\nENDBLK\n0\nBLOCK\n2\nE\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn ascii_prefix(version: &str) -> Vec<u8> {
    format!("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n")
        .into_bytes()
}

fn push_ascii_block(bytes: &mut Vec<u8>, groups: &[(i16, &[u8])], closed: bool) {
    push_ascii_group(bytes, 0, b"BLOCK");
    for (code, value) in groups {
        push_ascii_group(bytes, *code, value);
    }
    if closed {
        push_ascii_group(bytes, 0, b"ENDBLK");
    }
}

fn push_ascii_group(bytes: &mut Vec<u8>, code: i16, value: &[u8]) {
    bytes.extend_from_slice(code.to_string().as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(value);
    bytes.push(b'\n');
}

fn ascii_suffix(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(b"0\nENDSEC\n0\nEOF\n");
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_binary_string(&mut bytes, version, 0, b"SECTION")?;
    push_binary_string(&mut bytes, version, 2, b"HEADER")?;
    push_binary_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_binary_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"SECTION")?;
    push_binary_string(&mut bytes, version, 2, b"BLOCKS")?;
    for names in [
        (Some(b"A".as_slice()), Some(b"A".as_slice())),
        (Some(b"B".as_slice()), Some(b"B".as_slice())),
        (Some(b"A".as_slice()), Some(b"A".as_slice())),
        (Some(b"C".as_slice()), Some(b"D".as_slice())),
        (Some(b"E".as_slice()), None),
    ] {
        push_binary_string(&mut bytes, version, 0, b"BLOCK")?;
        if let Some(primary) = names.0 {
            push_binary_string(&mut bytes, version, 2, primary)?;
        }
        if let Some(secondary) = names.1 {
            push_binary_string(&mut bytes, version, 3, secondary)?;
        }
        push_binary_string(&mut bytes, version, 0, b"ENDBLK")?;
    }
    push_binary_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_binary_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_binary_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfBlockDefinitionDirectory, DxfBlockDefinitionEntry, DxfBlockDefinitionState,
    DxfBlockMemberRecordRange, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfRawDocumentView, DxfRawRecord, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type DefinitionEvidence = (DxfBlockDefinitionState, u64, Option<Vec<u8>>);

#[test]
fn every_supported_dialect_has_ascii_binary_block_definition_parity() -> Result<(), Box<dyn Error>>
{
    let expected = vec![
        (DxfBlockDefinitionState::Closed, 4, Some(b"ENDBLK".to_vec())),
        (DxfBlockDefinitionState::Closed, 0, Some(b"ENDBLK".to_vec())),
    ];

    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.block_definition_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.block_definition_directory(&DxfCancellationToken::default())?;

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
fn exact_markers_members_and_all_definition_boundaries_remain_explicit()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nBLOCK\n0\nLINE\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nENDBLK\n0\nblock\n0\nBLOCK \n0\nBLOCK\n2\nA\n0\nLINE\n0\nCIRCLE\n0\nENDBLK\n0\nBLOCK\n2\nB\n0\nPOINT\n0\nBLOCK\n2\nC\n0\nARC\n0\nENDBLK\n0\nBLOCK\n2\nD\n0\nLINE\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nBLOCK\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_definition_directory(&DxfCancellationToken::default())?;
    let view = DxfRawDocumentView::from(&document);

    assert_eq!(directory.definitions().len(), 4);
    assert_eq!(directory.member_records().len(), 5);
    let expected = [
        (
            DxfBlockDefinitionState::Closed,
            2,
            Some(b"ENDBLK".as_slice()),
        ),
        (
            DxfBlockDefinitionState::Interrupted,
            1,
            Some(b"BLOCK".as_slice()),
        ),
        (
            DxfBlockDefinitionState::Closed,
            1,
            Some(b"ENDBLK".as_slice()),
        ),
        (DxfBlockDefinitionState::Unclosed, 1, None),
    ];
    for (entry, (state, member_count, boundary)) in directory.definitions().iter().zip(expected) {
        assert_eq!(entry.state(), state);
        assert_eq!(entry.member_range().len(), member_count);
        assert_eq!(
            entry
                .boundary_record()
                .map(|record| record_marker(view, record))
                .transpose()?
                .as_deref(),
            boundary
        );
    }

    let interrupted = directory.definitions()[1];
    let nested = directory.definitions()[2];
    assert_eq!(interrupted.boundary_record(), Some(nested.block_record()));
    assert_eq!(
        directory.members_for_block_raw_ordinal(nested.block_record().ordinal()),
        Some(&directory.member_records()[3..4])
    );
    assert_eq!(
        member_markers(
            view,
            directory
                .members_for_block_raw_ordinal(directory.definitions()[0].block_record().ordinal())
                .ok_or_else(invalid_test_data)?
        )?,
        [b"LINE".as_slice(), b"CIRCLE".as_slice()]
    );
    Ok(())
}

#[test]
fn incomplete_blocks_sections_do_not_publish_partial_definitions() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n0\nLINE\n0\nSECTION\n2\nENTITIES\n0\nLINE\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.block_definition_directory(&DxfCancellationToken::default())?;

    assert!(directory.definitions().is_empty());
    assert!(directory.member_records().is_empty());
    Ok(())
}

#[test]
fn cancellation_lookups_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.block_definition_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.block_definition_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.raw_record_directory().source_id()
    );
    assert_eq!(directory.definition_for_block_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.members_for_block_raw_ordinal(u64::MAX), None);
    assert_copy::<DxfBlockDefinitionState>();
    assert_copy::<DxfBlockMemberRecordRange>();
    assert_copy::<DxfBlockDefinitionEntry>();
    assert_send_sync::<DxfBlockDefinitionDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfBlockDefinitionDirectory) {
    assert_eq!(directory.definitions().len(), 2);
    assert_eq!(directory.member_records().len(), 4);
    for entry in directory.definitions() {
        assert_eq!(entry.state(), DxfBlockDefinitionState::Closed);
        assert_eq!(
            directory.definition_for_block_raw_ordinal(entry.block_record().ordinal()),
            Some(*entry)
        );
        let members = directory
            .members_for_block_raw_ordinal(entry.block_record().ordinal())
            .unwrap_or(&[]);
        assert_eq!(members.len() as u64, entry.member_range().len());
        for member in members {
            assert_eq!(
                member.structure_section_ordinal(),
                entry.block_record().structure_section_ordinal()
            );
            assert!(member.ordinal() > entry.block_record().ordinal());
        }
    }
}

fn evidence(
    view: DxfRawDocumentView<'_>,
    directory: &DxfBlockDefinitionDirectory,
) -> Result<Vec<DefinitionEvidence>, DxfError> {
    directory
        .definitions()
        .iter()
        .copied()
        .map(|entry| {
            Ok((
                entry.state(),
                entry.member_range().len(),
                entry
                    .boundary_record()
                    .map(|record| record_marker(view, record))
                    .transpose()?,
            ))
        })
        .collect()
}

fn member_markers(
    view: DxfRawDocumentView<'_>,
    records: &[DxfRawRecord],
) -> Result<Vec<Vec<u8>>, DxfError> {
    records
        .iter()
        .copied()
        .map(|record| record_marker(view, record))
        .collect()
}

fn record_marker(view: DxfRawDocumentView<'_>, record: DxfRawRecord) -> Result<Vec<u8>, DxfError> {
    let marker = view
        .group(record.marker_occurrence())
        .ok_or_else(invalid_test_data)?;
    let mut value = vec![
        0_u8;
        usize::try_from(marker.value_payload_span().len())
            .map_err(|_| invalid_test_data())?
    ];
    view.read_span(marker.value_payload_span(), &mut value)?;
    Ok(value)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nA\n0\nLINE\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nENDBLK\n0\nBLOCK\n2\nB\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nBLOCK\n0\nENDBLK\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 2, b"A")?;
    for marker in [b"LINE".as_slice(), b"POLYLINE", b"VERTEX", b"SEQEND"] {
        push_string(&mut bytes, version, 0, marker)?;
    }
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 2, b"B")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"SECTION")?;
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    push_string(&mut bytes, version, 0, b"BLOCK")?;
    push_string(&mut bytes, version, 0, b"ENDBLK")?;
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

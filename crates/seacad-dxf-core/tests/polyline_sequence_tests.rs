use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineSequenceDirectory,
    DxfPolylineSequenceEntry, DxfPolylineSequenceState, DxfPolylineVertexRecordRange,
    DxfRawDocumentView, DxfRawRecord, DxfRawRecordSectionKind, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

type SequenceEvidence = (
    DxfRawRecordSectionKind,
    DxfPolylineSequenceState,
    u64,
    Option<Vec<u8>>,
);

#[test]
fn every_supported_dialect_has_ascii_binary_polyline_sequence_parity() -> Result<(), Box<dyn Error>>
{
    let expected = vec![
        (
            DxfRawRecordSectionKind::Blocks,
            DxfPolylineSequenceState::Closed,
            2,
            Some(b"SEQEND".to_vec()),
        ),
        (
            DxfRawRecordSectionKind::Entities,
            DxfPolylineSequenceState::Closed,
            1,
            Some(b"SEQEND".to_vec()),
        ),
    ];

    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.polyline_sequence_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_sequence_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
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
fn exact_markers_and_all_sequence_boundaries_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nTABLES\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nVERTEX\n0\nSEQEND\n0\npolyline\n0\nPOLYLINE \n0\nPOLYLINE\n0\nVERTEX\n0\nLINE\n0\nPOLYLINE\n0\nVERTEX\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nPOLYLINE\n0\nSEQEND\n0\nPOLYLINE\n0\nVERTEX\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_sequence_directory(&DxfCancellationToken::default())?;
    let view = DxfRawDocumentView::from(&document);

    assert_eq!(directory.sequences().len(), 6);
    assert_eq!(directory.vertex_records().len(), 5);
    let expected = [
        (
            DxfPolylineSequenceState::Interrupted,
            1,
            Some(b"LINE".as_slice()),
        ),
        (
            DxfPolylineSequenceState::Interrupted,
            1,
            Some(b"POLYLINE".as_slice()),
        ),
        (
            DxfPolylineSequenceState::Closed,
            1,
            Some(b"SEQEND".as_slice()),
        ),
        (
            DxfPolylineSequenceState::Closed,
            0,
            Some(b"SEQEND".as_slice()),
        ),
        (DxfPolylineSequenceState::Unclosed, 1, None),
        (
            DxfPolylineSequenceState::Closed,
            1,
            Some(b"SEQEND".as_slice()),
        ),
    ];
    for (entry, (state, vertex_count, boundary)) in directory.sequences().iter().zip(expected) {
        assert_eq!(entry.state(), state);
        assert_eq!(entry.vertex_range().len(), vertex_count);
        assert_eq!(
            entry
                .boundary_record()
                .map(|record| record_marker(view, record))
                .transpose()?
                .as_deref(),
            boundary
        );
    }

    let second = directory.sequences()[1];
    let third = directory.sequences()[2];
    assert_eq!(second.boundary_record(), Some(third.polyline_record()));
    assert_eq!(
        directory.vertices_for_polyline_raw_ordinal(third.polyline_record().ordinal()),
        Some(&directory.vertex_records()[2..3])
    );
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
        document.polyline_sequence_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_sequence_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.raw_record_directory().source_id()
    );
    assert_eq!(directory.sequence_for_polyline_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.vertices_for_polyline_raw_ordinal(u64::MAX), None);
    assert_copy::<DxfPolylineSequenceState>();
    assert_copy::<DxfPolylineVertexRecordRange>();
    assert_copy::<DxfPolylineSequenceEntry>();
    assert_send_sync::<DxfPolylineSequenceDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfPolylineSequenceDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.sequences().len(), 2);
    assert_eq!(directory.vertex_records().len(), 3);
    for entry in directory.sequences() {
        assert_eq!(entry.state(), DxfPolylineSequenceState::Closed);
        let vertices = directory
            .vertices_for_polyline_raw_ordinal(entry.polyline_record().ordinal())
            .ok_or(io::Error::other("polyline vertices"))?;
        assert_eq!(vertices.len() as u64, entry.vertex_range().len());
        assert_eq!(
            directory.sequence_for_polyline_raw_ordinal(entry.polyline_record().ordinal()),
            Some(*entry)
        );
        for vertex in vertices {
            assert_eq!(
                vertex.structure_section_ordinal(),
                entry.polyline_record().structure_section_ordinal()
            );
            assert!(vertex.ordinal() > entry.polyline_record().ordinal());
        }
    }
    Ok(())
}

fn evidence(
    view: DxfRawDocumentView<'_>,
    directory: &DxfPolylineSequenceDirectory,
) -> Result<Vec<SequenceEvidence>, DxfError> {
    directory
        .sequences()
        .iter()
        .copied()
        .map(|entry| {
            Ok((
                entry.polyline_record().section_kind(),
                entry.state(),
                entry.vertex_range().len(),
                entry
                    .boundary_record()
                    .map(|record| record_marker(view, record))
                    .transpose()?,
            ))
        })
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
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nPOLYLINE\n66\n1\n0\nVERTEX\n70\n0\n0\nVERTEX\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n66\n0\n0\nVERTEX\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    for (section, flag, vertex_count) in [(b"BLOCKS".as_slice(), 1_i16, 2), (b"ENTITIES", 0, 1)] {
        push_string(&mut bytes, version, 0, b"SECTION")?;
        push_string(&mut bytes, version, 2, section)?;
        push_string(&mut bytes, version, 0, b"POLYLINE")?;
        push_i16(&mut bytes, version, 66, flag)?;
        for _ in 0..vertex_count {
            push_string(&mut bytes, version, 0, b"VERTEX")?;
            push_i16(&mut bytes, version, 70, 0)?;
        }
        push_string(&mut bytes, version, 0, b"SEQEND")?;
        push_string(&mut bytes, version, 0, b"ENDSEC")?;
    }
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfPolylineFamily,
    DxfPolylineFamilySemanticDirectory, DxfPolylineFamilySemantics, DxfPolylineFamilyState,
    DxfPolylineVertexFamily, DxfPolylineVertexFamilyComparison, DxfPolylineVertexFamilySemantics,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type FamilyState = DxfPolylineFamilyState<DxfPolylineFamily>;
type VertexState = DxfPolylineFamilyState<DxfPolylineVertexFamily>;
type VertexEvidence = (FamilyState, VertexState, DxfPolylineVertexFamilyComparison);
type Evidence = (Vec<FamilyState>, Vec<VertexEvidence>);

#[test]
fn every_supported_dialect_has_ascii_binary_family_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_family_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_family_semantic_directory(&DxfCancellationToken::default())?;

        let ascii_evidence = evidence(&ascii_directory)?;
        let binary_evidence = evidence(&binary_directory)?;
        assert_eq!(ascii_evidence, binary_evidence);
        assert_eq!(
            ascii_evidence.0,
            vec![
                FamilyState::Classified(DxfPolylineFamily::TwoDimensional),
                FamilyState::Classified(DxfPolylineFamily::ThreeDimensional),
                FamilyState::Classified(DxfPolylineFamily::PolygonMesh),
                FamilyState::Classified(DxfPolylineFamily::PolyfaceMesh),
            ]
        );
        assert!(ascii_evidence.1.iter().all(|(_, _, comparison)| matches!(
            comparison,
            DxfPolylineVertexFamilyComparison::Matched { .. }
        )));
    }
    Ok(())
}

#[test]
fn unavailable_conflicting_and_mismatched_flags_remain_distinct() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n8\n70\n16\n0\nVERTEX\n70\n32\n0\nSEQEND\n0\nPOLYLINE\n70\n24\n0\nVERTEX\n70\n96\n0\nSEQEND\n0\nPOLYLINE\n70\n8\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_family_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 4);
    assert_eq!(directory.vertices().len(), 4);

    let first = vertex(&directory, 0)?;
    assert_eq!(first.polyline_family(), FamilyState::Unavailable);
    assert_eq!(
        first.vertex_family(),
        VertexState::Classified(DxfPolylineVertexFamily::ThreeDimensional)
    );
    assert_eq!(
        first.comparison(),
        DxfPolylineVertexFamilyComparison::NotComparable
    );

    let second = vertex(&directory, 1)?;
    assert_eq!(
        second.polyline_family(),
        FamilyState::Conflicting { family_bits: 24 }
    );
    assert_eq!(
        second.vertex_family(),
        VertexState::Conflicting { family_bits: 96 }
    );
    assert_eq!(
        second.comparison(),
        DxfPolylineVertexFamilyComparison::NotComparable
    );

    let third = vertex(&directory, 2)?;
    assert_eq!(
        third.comparison(),
        DxfPolylineVertexFamilyComparison::Mismatched {
            polyline: DxfPolylineFamily::ThreeDimensional,
            vertex: DxfPolylineVertexFamily::PolygonMesh,
        }
    );

    let fourth = vertex(&directory, 3)?;
    assert_eq!(
        fourth.polyline_family(),
        FamilyState::Classified(DxfPolylineFamily::TwoDimensional)
    );
    assert_eq!(fourth.vertex_family(), VertexState::Unavailable);
    assert_eq!(
        fourth.comparison(),
        DxfPolylineVertexFamilyComparison::NotComparable
    );
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_family_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_family_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.record_semantic_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.vertex_integer_semantic_directory().source_id()
    );
    assert!(
        directory
            .polyline_semantics_for_raw_ordinal(u64::MAX)?
            .is_none()
    );
    assert!(
        directory
            .vertex_semantics_for_raw_ordinal(u64::MAX)?
            .is_none()
    );
    assert_copy::<DxfPolylineFamilySemantics>();
    assert_copy::<DxfPolylineVertexFamilySemantics>();
    assert_send_sync::<DxfPolylineFamilySemanticDirectory>();
    Ok(())
}

fn evidence(directory: &DxfPolylineFamilySemanticDirectory) -> Result<Evidence, DxfError> {
    let records = directory
        .records()
        .iter()
        .map(|record| {
            directory
                .polyline_semantics_for_raw_ordinal(record.sequence().polyline_record().ordinal())
                .and_then(|value| value.ok_or_else(invalid_test_data))
                .map(DxfPolylineFamilySemantics::family)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let vertices = directory
        .vertices()
        .iter()
        .map(|vertex| {
            directory
                .vertex_semantics_for_raw_ordinal(vertex.vertex_record().ordinal())
                .and_then(|value| value.ok_or_else(invalid_test_data))
                .map(|value| {
                    (
                        value.polyline_family(),
                        value.vertex_family(),
                        value.comparison(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((records, vertices))
}

fn vertex(
    directory: &DxfPolylineFamilySemanticDirectory,
    index: usize,
) -> Result<DxfPolylineVertexFamilySemantics, Box<dyn Error>> {
    let vertex = directory
        .vertices()
        .get(index)
        .ok_or(io::Error::other("vertex"))?;
    directory
        .vertex_semantics_for_raw_ordinal(vertex.vertex_record().ordinal())?
        .ok_or_else(|| io::Error::other("vertex family").into())
}

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n0\n0\nVERTEX\n70\n0\n0\nSEQEND\n0\nPOLYLINE\n70\n8\n0\nVERTEX\n70\n32\n0\nSEQEND\n0\nPOLYLINE\n70\n16\n0\nVERTEX\n70\n64\n0\nSEQEND\n0\nPOLYLINE\n70\n64\n0\nVERTEX\n70\n192\n0\nVERTEX\n70\n128\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 2, b"ENTITIES")?;
    for (parent, vertex_flags) in [
        (0, &[0][..]),
        (8, &[32][..]),
        (16, &[64][..]),
        (64, &[192, 128][..]),
    ] {
        push_string(&mut bytes, version, 0, b"POLYLINE")?;
        push_i16(&mut bytes, version, 70, parent)?;
        for flags in vertex_flags {
            push_string(&mut bytes, version, 0, b"VERTEX")?;
            push_i16(&mut bytes, version, 70, *flags)?;
        }
        push_string(&mut bytes, version, 0, b"SEQEND")?;
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

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

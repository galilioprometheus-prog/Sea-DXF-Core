use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError, DxfMemorySource,
    DxfPolylineVertexIntegerSemanticDirectory, DxfPolylineVertexIntegerSemanticIssue,
    DxfPolylineVertexIntegerSemantics, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct IntegerEvidence {
    values: [Option<i64>; 6],
    flag_bits: [Option<bool>; 7],
}

#[test]
fn every_supported_dialect_has_ascii_binary_integer_semantic_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_vertex_integer_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_vertex_integer_semantic_directory(&DxfCancellationToken::default())?;

        let ascii_semantics = only_semantics(&ascii_directory)?;
        let binary_semantics = only_semantics(&binary_directory)?;
        assert_valid_semantics(&ascii_directory, ascii_semantics);
        assert_valid_semantics(&binary_directory, binary_semantics);
        assert_eq!(evidence(&ascii_semantics), evidence(&binary_semantics));
    }
    Ok(())
}

#[test]
fn absence_duplicates_invalidity_and_signed_indices_remain_explicit() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n0\n0\nVERTEX\n70\n.\n71\n1\n71\n2\n72\n.\n73\n-3\n91\n.\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_vertex_integer_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.vertices().len(), 2);

    let first = semantics(&directory, 0)?;
    assert_eq!(first.flags().state(), DxfSemanticValueState::Absent);
    assert_eq!(first.flags_value(), None);
    assert_eq!(first.is_polyface_mesh_vertex(), None);
    for index in first.polyface_vertex_indices() {
        assert_eq!(index.state(), DxfSemanticValueState::Absent);
        assert_eq!(index.raw_provenance(), None);
    }
    assert_eq!(first.identifier().state(), DxfSemanticValueState::Absent);

    let second = semantics(&directory, 1)?;
    assert_invalid_i16(second.flags());
    assert_eq!(second.is_extra_curve_fit_vertex(), None);
    assert_eq!(
        second.polyface_vertex_indices()[0].invalid_issue(),
        Some(&DxfPolylineVertexIntegerSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(
        second.polyface_vertex_indices()[0]
            .raw_provenance()
            .is_some()
    );
    assert_invalid_i16(&second.polyface_vertex_indices()[1]);
    assert_eq!(second.polyface_vertex_indices()[2].value(), Some(&-3));
    assert_eq!(
        second.polyface_vertex_indices()[3].state(),
        DxfSemanticValueState::Absent
    );
    assert_eq!(
        second.identifier().invalid_issue(),
        Some(&DxfPolylineVertexIntegerSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert!(second.identifier().raw_provenance().is_some());
    Ok(())
}

#[test]
fn sequence_states_cancellation_lookup_and_public_traits_remain_bounded()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n70\n1\n0\nSEQEND\n0\nPOLYLINE\n0\nVERTEX\n70\n2\n0\nPOLYLINE\n0\nVERTEX\n70\n8\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_vertex_integer_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.polyline_vertex_integer_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.vertices().len(), 3);
    for vertex in directory.vertices().iter().copied() {
        assert!(directory.semantics_for_entry(vertex)?.is_some());
        assert!(
            directory
                .semantics_for_vertex_raw_ordinal(vertex.vertex_record().ordinal())?
                .is_some()
        );
        assert!(
            directory
                .semantics_for_polyline_sequence_vertex(
                    vertex.polyline_record().ordinal(),
                    vertex.sequence_vertex_ordinal(),
                )?
                .is_some()
        );
    }
    assert!(
        directory
            .semantics_for_vertex_raw_ordinal(u64::MAX)?
            .is_none()
    );
    assert!(
        directory
            .semantics_for_polyline_sequence_vertex(u64::MAX, u64::MAX)?
            .is_none()
    );
    assert_copy::<DxfPolylineVertexIntegerSemantics>();
    assert_send_sync::<DxfPolylineVertexIntegerSemanticDirectory>();
    Ok(())
}

fn assert_valid_semantics(
    directory: &DxfPolylineVertexIntegerSemanticDirectory,
    semantics: DxfPolylineVertexIntegerSemantics,
) {
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(semantics.flags_value(), Some(251));
    assert_eq!(
        [
            semantics.is_extra_curve_fit_vertex(),
            semantics.has_curve_fit_tangent(),
            semantics.is_spline_fit_vertex(),
            semantics.is_spline_frame_control_point(),
            semantics.is_3d_polyline_vertex(),
            semantics.is_3d_polygon_mesh_vertex(),
            semantics.is_polyface_mesh_vertex(),
        ],
        [Some(true); 7]
    );
    assert_eq!(
        semantics
            .polyface_vertex_indices()
            .map(|value| value.value().copied()),
        [Some(1), Some(-2), Some(0), Some(i16::MIN)]
    );
    assert_eq!(semantics.identifier_value(), Some(i32::MAX));
    for value in std::iter::once(semantics.flags()).chain(semantics.polyface_vertex_indices()) {
        assert_eq!(value.state(), DxfSemanticValueState::Explicit);
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
        assert!(value.raw_provenance().is_some());
    }
    assert_eq!(
        semantics.identifier().state(),
        DxfSemanticValueState::Explicit
    );
    assert!(semantics.identifier().raw_provenance().is_some());
}

fn evidence(semantics: &DxfPolylineVertexIntegerSemantics) -> IntegerEvidence {
    IntegerEvidence {
        values: [
            semantics.flags_value().map(i64::from),
            semantics.polyface_vertex_indices()[0]
                .value()
                .copied()
                .map(i64::from),
            semantics.polyface_vertex_indices()[1]
                .value()
                .copied()
                .map(i64::from),
            semantics.polyface_vertex_indices()[2]
                .value()
                .copied()
                .map(i64::from),
            semantics.polyface_vertex_indices()[3]
                .value()
                .copied()
                .map(i64::from),
            semantics.identifier_value().map(i64::from),
        ],
        flag_bits: [
            semantics.is_extra_curve_fit_vertex(),
            semantics.has_curve_fit_tangent(),
            semantics.is_spline_fit_vertex(),
            semantics.is_spline_frame_control_point(),
            semantics.is_3d_polyline_vertex(),
            semantics.is_3d_polygon_mesh_vertex(),
            semantics.is_polyface_mesh_vertex(),
        ],
    }
}

fn only_semantics(
    directory: &DxfPolylineVertexIntegerSemanticDirectory,
) -> Result<DxfPolylineVertexIntegerSemantics, Box<dyn Error>> {
    let [vertex] = directory.vertices() else {
        return Err(io::Error::other("one vertex").into());
    };
    directory
        .semantics_for_entry(*vertex)?
        .ok_or_else(|| io::Error::other("vertex semantics").into())
}

fn semantics(
    directory: &DxfPolylineVertexIntegerSemanticDirectory,
    index: usize,
) -> Result<DxfPolylineVertexIntegerSemantics, Box<dyn Error>> {
    let vertex = directory
        .vertices()
        .get(index)
        .copied()
        .ok_or(io::Error::other("vertex"))?;
    directory
        .semantics_for_entry(vertex)?
        .ok_or_else(|| io::Error::other("vertex semantics").into())
}

fn assert_invalid_i16(value: &seacad_dxf_core::DxfPolylineVertexSemanticI16) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfPolylineVertexIntegerSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert!(value.raw_provenance().is_some());
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n0\n20\n0\n30\n0\n70\n251\n71\n1\n72\n-2\n73\n0\n74\n-32768\n91\n2147483647\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"POLYLINE")?;
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    for (code, value) in [(10, 0.0), (20, 0.0), (30, 0.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(70, 251), (71, 1), (72, -2), (73, 0), (74, i16::MIN)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, i32::MAX)?;
    push_string(&mut bytes, version, 0, b"SEQEND")?;
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

fn push_i32(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i32) -> io::Result<()> {
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

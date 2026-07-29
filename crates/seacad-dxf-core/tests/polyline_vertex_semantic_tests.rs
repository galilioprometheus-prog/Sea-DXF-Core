use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfMemorySource, DxfPolylineSequenceState, DxfPolylineVertexSemanticDirectory,
    DxfPolylineVertexSemanticIssue, DxfPolylineVertexSemantics, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

type DoubleState = (DxfSemanticValueState, Option<u64>);
type VertexEvidence = ([DoubleState; 3], [DoubleState; 2], DoubleState, DoubleState);

#[test]
fn every_supported_dialect_has_ascii_binary_vertex_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;

        assert_valid_semantics(&ascii_directory)?;
        assert_valid_semantics(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn missing_invalid_and_multiple_values_fail_closed_while_defaults_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n.\n30\n3\n40\n1\n40\n2\n41\n1e-9999\n50\n.\n0\nVERTEX\n10\n1\n20\n2\n30\n3\n20\n4\n42\n0\n42\n1\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;

    let first_entry = directory.vertices()[0];
    let first = directory
        .semantics_for_entry(first_entry)?
        .ok_or(io::Error::other("first semantics"))?;
    assert_eq!(first.position_value(), None);
    assert_eq!(
        first.position()[0].invalid_issue(),
        Some(&DxfPolylineVertexSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert!(first.position()[0].raw_provenance().is_some());
    assert_eq!(
        first.position()[1].invalid_issue(),
        Some(&DxfPolylineVertexSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(first.position()[1].raw_provenance(), None);
    assert_eq!(
        first.start_width().invalid_issue(),
        Some(&DxfPolylineVertexSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        first.end_width().invalid_issue(),
        Some(&DxfPolylineVertexSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(first.bulge().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(
        first.bulge_value().map(DxfDouble::to_bits),
        Some(0.0_f64.to_bits())
    );
    assert_eq!(
        first.curve_fit_tangent_direction().invalid_issue(),
        Some(&DxfPolylineVertexSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );

    let second_entry = directory.vertices()[1];
    let second = directory
        .semantics_for_vertex_raw_ordinal(second_entry.vertex_record().ordinal())?
        .ok_or(io::Error::other("second semantics"))?;
    assert_eq!(
        second.position()[1].invalid_issue(),
        Some(&DxfPolylineVertexSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        second.bulge().invalid_issue(),
        Some(&DxfPolylineVertexSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        second.width_values().map(bits2),
        Some(bits2_values(0.0, 0.0))
    );
    assert_eq!(
        second.curve_fit_tangent_direction().state(),
        DxfSemanticValueState::Absent
    );
    Ok(())
}

#[test]
fn semantics_remain_available_for_every_sequence_state_and_section() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n1\n20\n2\n30\n3\n0\nLINE\n0\nPOLYLINE\n0\nVERTEX\n10\n4\n20\n5\n30\n6\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nPOLYLINE\n0\nVERTEX\n10\n7\n20\n8\n30\n9\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;
    let expected_states = [
        DxfPolylineSequenceState::Interrupted,
        DxfPolylineSequenceState::Unclosed,
        DxfPolylineSequenceState::Closed,
    ];
    let sequences = directory
        .card_directory()
        .evidence_directory()
        .sequence_directory()
        .sequences();
    assert_eq!(sequences.len(), 3);
    for (sequence, state) in sequences.iter().zip(expected_states) {
        assert_eq!(sequence.state(), state);
        let semantics = directory
            .semantics_for_polyline_sequence_vertex(sequence.polyline_record().ordinal(), 0)?
            .ok_or(io::Error::other("sequence vertex semantics"))?;
        assert!(semantics.position_value().is_some());
        assert_eq!(semantics.bulge().state(), DxfSemanticValueState::Defaulted);
    }
    Ok(())
}

#[test]
fn cancellation_lookup_provenance_and_public_traits_remain_explicit() -> Result<(), Box<dyn Error>>
{
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_vertex_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.semantics_for_vertex_raw_ordinal(u64::MAX)?, None);
    assert_eq!(
        directory.semantics_for_polyline_sequence_vertex(u64::MAX, 0)?,
        None
    );
    let vertex = directory.vertices()[0];
    assert_eq!(
        directory.semantics_for_entry(vertex)?,
        directory.semantics_for_vertex_raw_ordinal(vertex.vertex_record().ordinal())?
    );
    let semantics = directory
        .semantics_for_entry(vertex)?
        .ok_or(io::Error::other("vertex semantics"))?;
    for value in semantics.position().iter().chain([
        semantics.start_width(),
        semantics.end_width(),
        semantics.bulge(),
        semantics.curve_fit_tangent_direction(),
    ]) {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
        assert_eq!(
            value.field_provenance().schema_namespace(),
            "polyline.vertex"
        );
    }
    assert_eq!(
        semantics
            .curve_fit_tangent_direction()
            .field_provenance()
            .schema_field_id(),
        "curve_fit_tangent_direction"
    );
    assert_copy::<DxfPolylineVertexSemantics>();
    assert_send_sync::<DxfPolylineVertexSemanticDirectory>();
    Ok(())
}

fn assert_valid_semantics(
    directory: &DxfPolylineVertexSemanticDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.vertices().len(), 2);
    let first = directory
        .semantics_for_entry(directory.vertices()[0])?
        .ok_or(io::Error::other("first semantics"))?;
    assert_eq!(
        first.position_value().map(bits3),
        Some(bits3_values(-0.0, 2.0, 3.0))
    );
    assert_eq!(
        first.width_values().map(bits2),
        Some(bits2_values(0.25, 0.5))
    );
    assert_eq!(
        first.bulge_value().map(DxfDouble::to_bits),
        Some((-0.0_f64).to_bits())
    );
    assert_eq!(
        first
            .curve_fit_tangent_direction_value()
            .map(DxfDouble::to_bits),
        Some(45.0_f64.to_bits())
    );
    assert!(
        first
            .position()
            .iter()
            .chain([
                first.start_width(),
                first.end_width(),
                first.bulge(),
                first.curve_fit_tangent_direction(),
            ])
            .all(|value| value.state() == DxfSemanticValueState::Explicit)
    );

    let second = directory
        .semantics_for_entry(directory.vertices()[1])?
        .ok_or(io::Error::other("second semantics"))?;
    assert_eq!(
        second.position_value().map(bits3),
        Some(bits3_values(4.0, 5.0, 6.0))
    );
    assert_eq!(
        second.width_values().map(bits2),
        Some(bits2_values(0.0, 0.0))
    );
    assert_eq!(
        second.start_width().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(second.end_width().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(second.bulge().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(
        second.curve_fit_tangent_direction().state(),
        DxfSemanticValueState::Absent
    );
    Ok(())
}

fn evidence(
    directory: &DxfPolylineVertexSemanticDirectory,
) -> Result<Vec<VertexEvidence>, io::Error> {
    directory
        .vertices()
        .iter()
        .copied()
        .map(|vertex| {
            let value = directory
                .semantics_for_entry(vertex)
                .map_err(|_| io::Error::other("semantic projection"))?
                .ok_or(io::Error::other("vertex semantics"))?;
            Ok((
                value.position().map(double_state),
                [
                    double_state(*value.start_width()),
                    double_state(*value.end_width()),
                ],
                double_state(*value.bulge()),
                double_state(*value.curve_fit_tangent_direction()),
            ))
        })
        .collect()
}

fn double_state(value: seacad_dxf_core::DxfPolylineVertexSemanticDouble) -> DoubleState {
    (
        value.state(),
        value.value().copied().map(DxfDouble::to_bits),
    )
}

fn bits2(values: [DxfDouble; 2]) -> [u64; 2] {
    values.map(DxfDouble::to_bits)
}

fn bits2_values(x: f64, y: f64) -> [u64; 2] {
    [x.to_bits(), y.to_bits()]
}

fn bits3(values: [DxfDouble; 3]) -> [u64; 3] {
    values.map(DxfDouble::to_bits)
}

fn bits3_values(x: f64, y: f64, z: f64) -> [u64; 3] {
    [x.to_bits(), y.to_bits(), z.to_bits()]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n-0\n20\n2\n30\n3\n40\n0.25\n41\n0.5\n42\n-0\n50\n45\n0\nVERTEX\n10\n4\n20\n5\n30\n6\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [
        (10, -0.0),
        (20, 2.0),
        (30, 3.0),
        (40, 0.25),
        (41, 0.5),
        (42, -0.0),
        (50, 45.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    for (code, value) in [(10, 4.0), (20, 5.0), (30, 6.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
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

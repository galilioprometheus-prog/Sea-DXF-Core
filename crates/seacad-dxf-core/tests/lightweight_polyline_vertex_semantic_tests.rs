use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfLightweightPolylineVertexSemanticDirectory, DxfLightweightPolylineVertexSemanticIssue,
    DxfLightweightPolylineVertexSemantics, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

type DoubleState = (DxfSemanticValueState, Option<u64>);
type VertexEvidence = (
    [DoubleState; 2],
    [DoubleState; 2],
    DoubleState,
    (DxfSemanticValueState, Option<i32>),
);

#[test]
fn every_supported_dialect_has_ascii_binary_vertex_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii
            .lightweight_polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary
            .lightweight_polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;

        assert_valid_semantics(&ascii_directory)?;
        assert_valid_semantics(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
    }
    Ok(())
}

#[test]
fn missing_invalid_and_multiple_values_fail_closed_while_defaults_remain_distinct()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n10\n.\n40\n1\n40\n2\n41\n1e-9999\n91\n2147483648\n10\n1\n20\n2\n20\n3\n42\n0\n42\n1\n91\n7\n91\n8\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.lightweight_polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;

    let first = directory
        .semantics_for_vertex(0)?
        .ok_or(io::Error::other("first semantics"))?;
    assert_eq!(first.ocs_position_value(), None);
    assert_eq!(
        first.ocs_position()[0].invalid_issue(),
        Some(
            &DxfLightweightPolylineVertexSemanticIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
            )
        )
    );
    assert!(first.ocs_position()[0].raw_provenance().is_some());
    assert_eq!(
        first.ocs_position()[1].invalid_issue(),
        Some(&DxfLightweightPolylineVertexSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(first.ocs_position()[1].raw_provenance(), None);
    assert_eq!(
        first.local_start_width().invalid_issue(),
        Some(&DxfLightweightPolylineVertexSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(first.local_start_width().raw_provenance().is_some());
    assert_eq!(
        first.local_end_width().invalid_issue(),
        Some(
            &DxfLightweightPolylineVertexSemanticIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::OutOfRange
            )
        )
    );
    assert_eq!(first.bulge().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(
        first.bulge_value().map(DxfDouble::to_bits),
        Some(0.0_f64.to_bits())
    );
    assert_eq!(first.bulge().raw_provenance(), None);
    assert_eq!(
        first.identifier().invalid_issue(),
        Some(
            &DxfLightweightPolylineVertexSemanticIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::OutOfRange
            )
        )
    );

    let second = directory
        .semantics_for_vertex(1)?
        .ok_or(io::Error::other("second semantics"))?;
    assert_eq!(
        second.ocs_position()[1].invalid_issue(),
        Some(&DxfLightweightPolylineVertexSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        second.bulge().invalid_issue(),
        Some(&DxfLightweightPolylineVertexSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        second.identifier().invalid_issue(),
        Some(&DxfLightweightPolylineVertexSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        second.local_width_values().map(bits2),
        Some(bits2_values(0.0, 0.0))
    );
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
        document.lightweight_polyline_vertex_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.lightweight_polyline_vertex_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.semantics_for_vertex(u64::MAX)?, None);
    assert_eq!(
        directory.semantics_for_raw_record_vertex(u64::MAX, 0)?,
        None
    );
    let vertex = directory.vertices()[0];
    assert_eq!(
        directory.semantics_for_entry(vertex)?,
        directory.semantics_for_vertex(0)?
    );
    let semantics = directory
        .semantics_for_vertex(0)?
        .ok_or(io::Error::other("vertex semantics"))?;
    for value in semantics.ocs_position().iter().chain([
        semantics.local_start_width(),
        semantics.local_end_width(),
        semantics.bulge(),
    ]) {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
        assert_eq!(
            value.field_provenance().schema_namespace(),
            "lightweight_polyline.vertex"
        );
    }
    assert_eq!(
        semantics.identifier().field_provenance().schema_field_id(),
        "identifier"
    );
    assert_copy::<DxfLightweightPolylineVertexSemantics>();
    assert_send_sync::<DxfLightweightPolylineVertexSemanticDirectory>();
    Ok(())
}

fn assert_valid_semantics(
    directory: &DxfLightweightPolylineVertexSemanticDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.vertices().len(), 2);
    let first = directory
        .semantics_for_vertex(0)?
        .ok_or(io::Error::other("first semantics"))?;
    assert_eq!(
        first.ocs_position_value().map(bits2),
        Some(bits2_values(-0.0, 2.0))
    );
    assert_eq!(
        first.local_width_values().map(bits2),
        Some(bits2_values(0.25, 0.5))
    );
    assert_eq!(
        first.bulge_value().map(DxfDouble::to_bits),
        Some((-0.0_f64).to_bits())
    );
    assert_eq!(first.identifier_value(), Some(7));
    assert!(
        first
            .ocs_position()
            .iter()
            .chain([
                first.local_start_width(),
                first.local_end_width(),
                first.bulge()
            ])
            .all(|value| value.state() == DxfSemanticValueState::Explicit)
    );
    assert_eq!(first.identifier().state(), DxfSemanticValueState::Explicit);

    let second = directory
        .semantics_for_vertex(1)?
        .ok_or(io::Error::other("second semantics"))?;
    assert_eq!(
        second.ocs_position_value().map(bits2),
        Some(bits2_values(3.0, 4.0))
    );
    assert_eq!(
        second.local_width_values().map(bits2),
        Some(bits2_values(0.0, 0.0))
    );
    assert_eq!(
        second.bulge_value().map(DxfDouble::to_bits),
        Some(0.0_f64.to_bits())
    );
    assert_eq!(
        second.local_start_width().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        second.local_end_width().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(second.bulge().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(second.identifier().state(), DxfSemanticValueState::Absent);
    assert_eq!(second.identifier_value(), None);
    Ok(())
}

fn evidence(
    directory: &DxfLightweightPolylineVertexSemanticDirectory,
) -> Result<Vec<VertexEvidence>, io::Error> {
    (0..directory.vertices().len())
        .map(|ordinal| {
            let value = directory
                .semantics_for_vertex(ordinal as u64)
                .map_err(|_| io::Error::other("semantic projection"))?
                .ok_or(io::Error::other("vertex semantics"))?;
            Ok((
                value.ocs_position().map(double_state),
                [
                    double_state(*value.local_start_width()),
                    double_state(*value.local_end_width()),
                ],
                double_state(*value.bulge()),
                (value.identifier().state(), value.identifier_value()),
            ))
        })
        .collect()
}

fn double_state(value: seacad_dxf_core::DxfLightweightPolylineVertexSemanticDouble) -> DoubleState {
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

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n43\n2.5\n10\n-0\n20\n2\n40\n0.25\n41\n0.5\n42\n-0\n91\n7\n10\n3\n20\n4\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"LWPOLYLINE")?;
    push_i32(&mut bytes, version, 90, 2)?;
    for (code, value) in [
        (43, 2.5),
        (10, -0.0),
        (20, 2.0),
        (40, 0.25),
        (41, 0.5),
        (42, -0.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 7)?;
    for (code, value) in [(10, 3.0), (20, 4.0)] {
        push_double(&mut bytes, version, code, value)?;
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken,
    DxfCircularGeometrySemanticDirectory, DxfCircularGeometrySemanticIssue,
    DxfCircularGeometrySemanticValue, DxfCircularGeometrySemantics, DxfDouble, DxfError,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

type TripleEvidence = [(DxfSemanticValueState, Option<u64>); 3];
type SemanticEvidence = (
    TripleEvidence,
    (DxfSemanticValueState, Option<u64>),
    Option<(DxfSemanticValueState, Option<u64>)>,
    Option<(DxfSemanticValueState, Option<u64>)>,
    TripleEvidence,
);

#[test]
fn every_dialect_has_ascii_binary_semantic_and_default_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.circular_geometry_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.circular_geometry_semantic_directory(&DxfCancellationToken::default())?;

        assert_semantics(&ascii_directory)?;
        assert_semantics(&binary_directory)?;
        assert_eq!(
            semantic_evidence(&ascii_directory)?,
            semantic_evidence(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn missing_invalid_and_multiple_required_values_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nARC\n10\n1\n10\n2\n30\n.\n40\n1e-9999\n51\n10\n51\n20\n210\n.\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.circular_geometry_semantic_directory(&DxfCancellationToken::default())?;
    let record = directory.record(0).ok_or(io::Error::other("arc record"))?;
    let arc = directory
        .semantics_for_record(record)
        .map_err(|_| io::Error::other("arc projection"))?
        .ok_or(io::Error::other("arc semantics"))?;

    assert_eq!(arc.center_value(), None);
    assert_eq!(
        arc.center()[0].invalid_issue(),
        Some(&DxfCircularGeometrySemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(arc.center()[0].raw_provenance().is_some());
    assert_eq!(
        arc.center()[1].invalid_issue(),
        Some(&DxfCircularGeometrySemanticIssue::MissingRequiredValue)
    );
    assert_eq!(arc.center()[1].raw_provenance(), None);
    assert_eq!(
        arc.center()[2].invalid_issue(),
        Some(&DxfCircularGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert!(arc.center()[2].raw_provenance().is_some());

    assert_eq!(arc.radius_value(), None);
    assert_eq!(
        arc.radius().invalid_issue(),
        Some(&DxfCircularGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(arc.start_angle_degrees_value(), None);
    assert_eq!(
        arc.start_angle_degrees()
            .and_then(DxfCircularGeometrySemanticValue::invalid_issue),
        Some(&DxfCircularGeometrySemanticIssue::MissingRequiredValue)
    );
    assert_eq!(arc.end_angle_degrees_value(), None);
    assert_eq!(
        arc.end_angle_degrees()
            .and_then(DxfCircularGeometrySemanticValue::invalid_issue),
        Some(&DxfCircularGeometrySemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );

    assert_eq!(arc.extrusion_value(), None);
    assert_eq!(
        arc.extrusion()[0].invalid_issue(),
        Some(&DxfCircularGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(arc.extrusion()[1].state(), DxfSemanticValueState::Defaulted);
    assert_eq!(arc.extrusion()[2].state(), DxfSemanticValueState::Defaulted);
    assert_eq!(directory.card_directory().members().len(), 7);
    Ok(())
}

#[test]
fn lazy_lookup_cancellation_and_public_traits_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.circular_geometry_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.circular_geometry_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record(u64::MAX), None);
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    for record in directory.records().iter().copied() {
        assert_eq!(
            directory.semantics_for_record(record)?,
            directory.semantics_for_raw_record(record.record().ordinal())?
        );
    }
    assert_copy::<DxfCircularGeometrySemantics>();
    assert_send_sync::<DxfCircularGeometrySemanticDirectory>();
    Ok(())
}

fn assert_semantics(
    directory: &DxfCircularGeometrySemanticDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );

    let circle = directory
        .semantics_for_record(directory.records()[0])?
        .ok_or(io::Error::other("circle semantics"))?;
    assert_eq!(
        circle.center_value().map(bits),
        Some([(-0.0_f64).to_bits(), 2.0_f64.to_bits(), 3.0_f64.to_bits()])
    );
    assert_eq!(
        circle.radius_value().map(DxfDouble::to_bits),
        Some((-4.0_f64).to_bits())
    );
    assert_eq!(circle.start_angle_degrees(), None);
    assert_eq!(circle.end_angle_degrees(), None);
    assert_eq!(
        circle.extrusion_value().map(bits),
        Some([0.0_f64.to_bits(), 0.0_f64.to_bits(), 1.0_f64.to_bits()])
    );
    assert!(
        circle
            .extrusion()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Defaulted)
    );

    let arc = directory
        .semantics_for_record(directory.records()[1])?
        .ok_or(io::Error::other("arc semantics"))?;
    assert_eq!(arc.center_value().map(bits), Some(bits3(1.0, 2.0, 3.0)));
    assert_eq!(
        arc.radius_value().map(DxfDouble::to_bits),
        Some(4.0_f64.to_bits())
    );
    assert_eq!(
        arc.start_angle_degrees_value().map(DxfDouble::to_bits),
        Some((-0.0_f64).to_bits())
    );
    assert_eq!(
        arc.end_angle_degrees_value().map(DxfDouble::to_bits),
        Some(450.0_f64.to_bits())
    );
    assert_eq!(arc.extrusion_value().map(bits), Some(bits3(2.0, 0.0, 3.0)));
    assert_eq!(arc.extrusion()[0].state(), DxfSemanticValueState::Explicit);
    assert_eq!(arc.extrusion()[1].state(), DxfSemanticValueState::Defaulted);
    assert_eq!(arc.extrusion()[2].state(), DxfSemanticValueState::Explicit);

    assert_eq!(
        circle.center()[0].field_provenance().schema_namespace(),
        "circular_geometry.circle"
    );
    assert_eq!(
        arc.end_angle_degrees()
            .ok_or(io::Error::other("end angle"))?
            .field_provenance()
            .schema_field_id(),
        "end_angle_degrees"
    );
    for value in circle.center().iter().chain(circle.extrusion()) {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
    }
    Ok(())
}

fn semantic_evidence(
    directory: &DxfCircularGeometrySemanticDirectory,
) -> Result<Vec<SemanticEvidence>, io::Error> {
    directory
        .records()
        .iter()
        .copied()
        .map(|record| {
            let semantics = directory
                .semantics_for_record(record)
                .map_err(|_| io::Error::other("semantic projection"))?
                .ok_or(io::Error::other("circular semantics"))?;
            Ok((
                triple_evidence(semantics.center()),
                value_evidence(semantics.radius()),
                semantics.start_angle_degrees().map(value_evidence),
                semantics.end_angle_degrees().map(value_evidence),
                triple_evidence(semantics.extrusion()),
            ))
        })
        .collect()
}

fn triple_evidence(values: &[DxfCircularGeometrySemanticValue; 3]) -> TripleEvidence {
    values.map(|value| value_evidence(&value))
}

fn value_evidence(
    value: &DxfCircularGeometrySemanticValue,
) -> (DxfSemanticValueState, Option<u64>) {
    (
        value.state(),
        value.value().copied().map(DxfDouble::to_bits),
    )
}

fn bits(values: [DxfDouble; 3]) -> [u64; 3] {
    values.map(DxfDouble::to_bits)
}

fn bits3(x: f64, y: f64, z: f64) -> [u64; 3] {
    [x.to_bits(), y.to_bits(), z.to_bits()]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n10\n-0\n20\n2\n30\n3\n40\n-4\n0\nARC\n10\n1\n20\n2\n30\n3\n40\n4\n50\n-0\n51\n450\n210\n2\n230\n3\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"CIRCLE")?;
    for (code, value) in [(10, -0.0), (20, 2.0), (30, 3.0), (40, -4.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"ARC")?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (40, 4.0),
        (50, -0.0),
        (51, 450.0),
        (210, 2.0),
        (230, 3.0),
    ] {
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

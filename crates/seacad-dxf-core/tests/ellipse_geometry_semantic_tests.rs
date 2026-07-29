use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble,
    DxfEllipseGeometrySemanticDirectory, DxfEllipseGeometrySemanticIssue,
    DxfEllipseGeometrySemanticValue, DxfEllipseGeometrySemantics, DxfError, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

type ValueEvidence = (DxfSemanticValueState, Option<u64>);
type TripleEvidence = [ValueEvidence; 3];
type SemanticEvidence = (
    TripleEvidence,
    TripleEvidence,
    ValueEvidence,
    ValueEvidence,
    ValueEvidence,
    TripleEvidence,
);

#[test]
fn every_dialect_has_ascii_binary_semantic_and_default_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.ellipse_geometry_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.ellipse_geometry_semantic_directory(&DxfCancellationToken::default())?;

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
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nELLIPSE\n10\n1\n10\n2\n30\n.\n11\n4\n21\n5\n31\n6\n40\n1e-9999\n42\n1\n42\n2\n210\n.\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.ellipse_geometry_semantic_directory(&DxfCancellationToken::default())?;
    let record = directory
        .record(0)
        .ok_or(io::Error::other("ellipse record"))?;
    let ellipse = directory
        .semantics_for_record(record)
        .map_err(|_| io::Error::other("ellipse projection"))?
        .ok_or(io::Error::other("ellipse semantics"))?;

    assert_eq!(ellipse.center_value(), None);
    assert_eq!(
        ellipse.center()[0].invalid_issue(),
        Some(&DxfEllipseGeometrySemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(ellipse.center()[0].raw_provenance().is_some());
    assert_eq!(
        ellipse.center()[1].invalid_issue(),
        Some(&DxfEllipseGeometrySemanticIssue::MissingRequiredValue)
    );
    assert_eq!(ellipse.center()[1].raw_provenance(), None);
    assert_eq!(
        ellipse.center()[2].invalid_issue(),
        Some(&DxfEllipseGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert!(ellipse.center()[2].raw_provenance().is_some());

    assert_eq!(
        ellipse
            .major_axis_endpoint_relative_to_center_value()
            .map(bits),
        Some(bits3(4.0, 5.0, 6.0))
    );
    assert_eq!(ellipse.minor_to_major_axis_ratio_value(), None);
    assert_eq!(
        ellipse.minor_to_major_axis_ratio().invalid_issue(),
        Some(&DxfEllipseGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(ellipse.start_parameter_value(), None);
    assert_eq!(
        ellipse.start_parameter().invalid_issue(),
        Some(&DxfEllipseGeometrySemanticIssue::MissingRequiredValue)
    );
    assert_eq!(ellipse.end_parameter_value(), None);
    assert_eq!(
        ellipse.end_parameter().invalid_issue(),
        Some(&DxfEllipseGeometrySemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );

    assert_eq!(ellipse.extrusion_value(), None);
    assert_eq!(
        ellipse.extrusion()[0].invalid_issue(),
        Some(&DxfEllipseGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        ellipse.extrusion()[1].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        ellipse.extrusion()[2].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(directory.card_directory().members().len(), 10);
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
        document.ellipse_geometry_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.ellipse_geometry_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record(u64::MAX), None);
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    for record in directory.records().iter().copied() {
        assert_eq!(
            directory.semantics_for_record(record)?,
            directory.semantics_for_raw_record(record.record().ordinal())?
        );
    }
    assert_copy::<DxfEllipseGeometrySemantics>();
    assert_send_sync::<DxfEllipseGeometrySemanticDirectory>();
    Ok(())
}

fn assert_semantics(directory: &DxfEllipseGeometrySemanticDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );

    let ellipse = directory
        .semantics_for_record(directory.records()[0])?
        .ok_or(io::Error::other("ellipse semantics"))?;
    assert_eq!(
        ellipse.center_value().map(bits),
        Some([(-0.0_f64).to_bits(), 2.0_f64.to_bits(), 3.0_f64.to_bits()])
    );
    assert_eq!(
        ellipse
            .major_axis_endpoint_relative_to_center_value()
            .map(bits),
        Some(bits3(4.0, 5.0, 6.0))
    );
    assert_eq!(
        ellipse
            .minor_to_major_axis_ratio_value()
            .map(DxfDouble::to_bits),
        Some((-0.5_f64).to_bits())
    );
    assert_eq!(
        ellipse.start_parameter_value().map(DxfDouble::to_bits),
        Some((-0.0_f64).to_bits())
    );
    assert_eq!(
        ellipse.end_parameter_value().map(DxfDouble::to_bits),
        Some(7.0_f64.to_bits())
    );
    assert_eq!(
        ellipse.extrusion_value().map(bits),
        Some(bits3(2.0, 0.0, 3.0))
    );
    assert_eq!(
        ellipse.extrusion()[0].state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        ellipse.extrusion()[1].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        ellipse.extrusion()[2].state(),
        DxfSemanticValueState::Explicit
    );

    assert_eq!(
        ellipse.center()[0].field_provenance().schema_namespace(),
        "ellipse_geometry.ellipse"
    );
    assert_eq!(
        ellipse.major_axis_endpoint_relative_to_center()[2]
            .field_provenance()
            .schema_field_id(),
        "major_axis_endpoint_relative_z"
    );
    for value in ellipse
        .center()
        .iter()
        .chain(ellipse.major_axis_endpoint_relative_to_center())
        .chain(ellipse.extrusion())
    {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
    }
    Ok(())
}

fn semantic_evidence(
    directory: &DxfEllipseGeometrySemanticDirectory,
) -> Result<Vec<SemanticEvidence>, io::Error> {
    directory
        .records()
        .iter()
        .copied()
        .map(|record| {
            let semantics = directory
                .semantics_for_record(record)
                .map_err(|_| io::Error::other("semantic projection"))?
                .ok_or(io::Error::other("ellipse semantics"))?;
            Ok((
                triple_evidence(semantics.center()),
                triple_evidence(semantics.major_axis_endpoint_relative_to_center()),
                value_evidence(semantics.minor_to_major_axis_ratio()),
                value_evidence(semantics.start_parameter()),
                value_evidence(semantics.end_parameter()),
                triple_evidence(semantics.extrusion()),
            ))
        })
        .collect()
}

fn triple_evidence(values: &[DxfEllipseGeometrySemanticValue; 3]) -> TripleEvidence {
    values.map(|value| value_evidence(&value))
}

fn value_evidence(value: &DxfEllipseGeometrySemanticValue) -> ValueEvidence {
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
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nELLIPSE\n10\n-0\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n40\n-0.5\n41\n-0\n42\n7\n210\n2\n230\n3\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"ELLIPSE")?;
    for (code, value) in [
        (10, -0.0),
        (20, 2.0),
        (30, 3.0),
        (11, 4.0),
        (21, 5.0),
        (31, 6.0),
        (40, -0.5),
        (41, -0.0),
        (42, 7.0),
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

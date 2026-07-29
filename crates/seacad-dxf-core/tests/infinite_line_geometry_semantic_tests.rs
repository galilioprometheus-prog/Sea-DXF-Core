use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfInfiniteLineGeometryKind, DxfInfiniteLineGeometrySemanticDirectory,
    DxfInfiniteLineGeometrySemanticIssue, DxfInfiniteLineGeometrySemanticValue,
    DxfInfiniteLineGeometrySemantics, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

type ValueEvidence = (DxfSemanticValueState, Option<u64>);
type TripleEvidence = [ValueEvidence; 3];
type SemanticEvidence = (DxfInfiniteLineGeometryKind, TripleEvidence, TripleEvidence);

#[test]
fn every_dialect_has_ascii_binary_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.infinite_line_geometry_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.infinite_line_geometry_semantic_directory(&DxfCancellationToken::default())?;

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
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nRAY\n10\n1\n10\n2\n30\n.\n11\n4\n21\n1e-9999\n31\n5\n31\n6\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.infinite_line_geometry_semantic_directory(&DxfCancellationToken::default())?;
    let record = directory.record(0).ok_or(io::Error::other("ray record"))?;
    let ray = directory
        .semantics_for_record(record)
        .map_err(|_| io::Error::other("ray projection"))?
        .ok_or(io::Error::other("ray semantics"))?;

    assert_eq!(ray.start_or_first_point_value(), None);
    assert_eq!(
        ray.start_or_first_point()[0].invalid_issue(),
        Some(&DxfInfiniteLineGeometrySemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(ray.start_or_first_point()[0].raw_provenance().is_some());
    assert_eq!(
        ray.start_or_first_point()[1].invalid_issue(),
        Some(&DxfInfiniteLineGeometrySemanticIssue::MissingRequiredValue)
    );
    assert_eq!(ray.start_or_first_point()[1].raw_provenance(), None);
    assert_eq!(
        ray.start_or_first_point()[2].invalid_issue(),
        Some(&DxfInfiniteLineGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert!(ray.start_or_first_point()[2].raw_provenance().is_some());

    assert_eq!(ray.unit_direction_value(), None);
    assert_eq!(
        ray.unit_direction()[0]
            .value()
            .copied()
            .map(DxfDouble::to_bits),
        Some(4.0_f64.to_bits())
    );
    assert_eq!(
        ray.unit_direction()[1].invalid_issue(),
        Some(&DxfInfiniteLineGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        ray.unit_direction()[2].invalid_issue(),
        Some(&DxfInfiniteLineGeometrySemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(ray.unit_direction()[2].raw_provenance().is_some());
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
        document.infinite_line_geometry_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.infinite_line_geometry_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record(u64::MAX), None);
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    for record in directory.records().iter().copied() {
        assert_eq!(
            directory.semantics_for_record(record)?,
            directory.semantics_for_raw_record(record.record().ordinal())?
        );
    }
    assert_copy::<DxfInfiniteLineGeometrySemantics>();
    assert_send_sync::<DxfInfiniteLineGeometrySemanticDirectory>();
    Ok(())
}

fn assert_semantics(
    directory: &DxfInfiniteLineGeometrySemanticDirectory,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 2);
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );

    let ray = directory
        .semantics_for_record(directory.records()[0])?
        .ok_or(io::Error::other("ray semantics"))?;
    assert_eq!(ray.kind(), DxfInfiniteLineGeometryKind::Ray);
    assert_eq!(
        ray.start_or_first_point_value().map(bits),
        Some([(-0.0_f64).to_bits(), 2.0_f64.to_bits(), 3.0_f64.to_bits()])
    );
    assert_eq!(
        ray.unit_direction_value().map(bits),
        Some(bits3(2.0, -0.0, 0.0))
    );

    let xline = directory
        .semantics_for_record(directory.records()[1])?
        .ok_or(io::Error::other("xline semantics"))?;
    assert_eq!(xline.kind(), DxfInfiniteLineGeometryKind::Xline);
    assert_eq!(
        xline.start_or_first_point_value().map(bits),
        Some(bits3(4.0, 5.0, 6.0))
    );
    assert_eq!(
        xline.unit_direction_value().map(bits),
        Some(bits3(0.0, 0.0, -3.0))
    );

    assert!(
        ray.start_or_first_point()
            .iter()
            .chain(ray.unit_direction())
            .chain(xline.start_or_first_point())
            .chain(xline.unit_direction())
            .all(|value| value.state() == DxfSemanticValueState::Explicit)
    );
    assert_eq!(
        ray.start_or_first_point()[0]
            .field_provenance()
            .schema_namespace(),
        "infinite_line_geometry.ray"
    );
    assert_eq!(
        xline.unit_direction()[2]
            .field_provenance()
            .schema_namespace(),
        "infinite_line_geometry.xline"
    );
    assert_eq!(
        xline.unit_direction()[2]
            .field_provenance()
            .schema_field_id(),
        "unit_direction_z"
    );
    for value in ray
        .start_or_first_point()
        .iter()
        .chain(ray.unit_direction())
    {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
    }
    Ok(())
}

fn semantic_evidence(
    directory: &DxfInfiniteLineGeometrySemanticDirectory,
) -> Result<Vec<SemanticEvidence>, io::Error> {
    directory
        .records()
        .iter()
        .copied()
        .map(|record| {
            let semantics = directory
                .semantics_for_record(record)
                .map_err(|_| io::Error::other("semantic projection"))?
                .ok_or(io::Error::other("infinite-line semantics"))?;
            Ok((
                semantics.kind(),
                triple_evidence(semantics.start_or_first_point()),
                triple_evidence(semantics.unit_direction()),
            ))
        })
        .collect()
}

fn triple_evidence(values: &[DxfInfiniteLineGeometrySemanticValue; 3]) -> TripleEvidence {
    values.map(|value| {
        (
            value.state(),
            value.value().copied().map(DxfDouble::to_bits),
        )
    })
}

fn bits(values: [DxfDouble; 3]) -> [u64; 3] {
    values.map(DxfDouble::to_bits)
}

fn bits3(x: f64, y: f64, z: f64) -> [u64; 3] {
    [x.to_bits(), y.to_bits(), z.to_bits()]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nRAY\n10\n-0\n20\n2\n30\n3\n11\n2\n21\n-0\n31\n0\n0\nXLINE\n10\n4\n20\n5\n30\n6\n11\n0\n21\n0\n31\n-3\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"RAY")?;
    for (code, value) in [
        (10, -0.0),
        (20, 2.0),
        (30, 3.0),
        (11, 2.0),
        (21, -0.0),
        (31, 0.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"XLINE")?;
    for (code, value) in [
        (10, 4.0),
        (20, 5.0),
        (30, 6.0),
        (11, 0.0),
        (21, 0.0),
        (31, -3.0),
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

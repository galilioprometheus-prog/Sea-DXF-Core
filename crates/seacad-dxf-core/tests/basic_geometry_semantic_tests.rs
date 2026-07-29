use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBasicGeometryKind, DxfBasicGeometrySemanticDirectory, DxfBasicGeometrySemanticEntry,
    DxfBasicGeometrySemanticIssue, DxfBasicGeometrySemanticKind, DxfBinaryRawDocument,
    DxfByteSource, DxfCancellationToken, DxfDouble, DxfError, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, DxfSemanticValueState, NoopDxfReadObserver,
};

type TripleEvidence = [(DxfSemanticValueState, Option<u64>); 3];

#[test]
fn every_dialect_has_ascii_binary_semantic_and_default_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.basic_geometry_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.basic_geometry_semantic_directory(&DxfCancellationToken::default())?;

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
fn missing_invalid_and_multiple_required_components_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOINT\n10\n1\n10\n2\n30\n.\n210\n1e-9999\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.basic_geometry_semantic_directory(&DxfCancellationToken::default())?;
    let point_entry = directory.entry(0).ok_or(io::Error::other("point entry"))?;
    let point = directory
        .point_for_entry(point_entry)
        .map_err(|_| io::Error::other("point projection"))?
        .ok_or(io::Error::other("point semantics"))?;

    assert_eq!(point.location_value(), None);
    assert_eq!(
        point.location()[0].invalid_issue(),
        Some(&DxfBasicGeometrySemanticIssue::MultipleComponents {
            occurrence_count: 2
        })
    );
    assert!(point.location()[0].raw_provenance().is_some());
    assert_eq!(
        point.location()[1].invalid_issue(),
        Some(&DxfBasicGeometrySemanticIssue::MissingRequiredComponent)
    );
    assert_eq!(point.location()[1].raw_provenance(), None);
    assert_eq!(
        point.location()[2].invalid_issue(),
        Some(&DxfBasicGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert!(point.location()[2].raw_provenance().is_some());

    assert_eq!(point.extrusion_value(), None);
    assert_eq!(
        point.extrusion()[0].invalid_issue(),
        Some(&DxfBasicGeometrySemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        point.extrusion()[1].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        point.extrusion()[2].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(directory.card_directory().members().len(), 4);
    Ok(())
}

#[test]
fn cancellation_bounds_and_public_traits_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.basic_geometry_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.basic_geometry_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_raw_record(u64::MAX), None);
    assert_copy::<DxfBasicGeometrySemanticEntry>();
    assert_send_sync::<DxfBasicGeometrySemanticDirectory>();
    Ok(())
}

fn assert_semantics(directory: &DxfBasicGeometrySemanticDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 2);
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );

    let point_entry = directory.entries()[0];
    assert_eq!(point_entry.kind(), DxfBasicGeometrySemanticKind::Point);
    let point = directory
        .point_for_entry(point_entry)
        .map_err(|_| io::Error::other("point projection"))?
        .ok_or(io::Error::other("point entry"))?;
    assert_eq!(directory.line_for_entry(point_entry)?, None);
    assert_eq!(
        point.location_value().map(bits),
        Some([(-0.0_f64).to_bits(), 2.0_f64.to_bits(), 3.0_f64.to_bits()])
    );
    assert_eq!(
        point.extrusion_value().map(bits),
        Some([0.0_f64.to_bits(), 0.0_f64.to_bits(), 1.0_f64.to_bits()])
    );
    assert!(
        point
            .location()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Explicit)
    );
    assert!(
        point
            .extrusion()
            .iter()
            .all(|value| value.state() == DxfSemanticValueState::Defaulted)
    );

    let line_entry = directory.entries()[1];
    assert_eq!(line_entry.kind(), DxfBasicGeometrySemanticKind::Line);
    let line = directory
        .line_for_entry(line_entry)
        .map_err(|_| io::Error::other("line projection"))?
        .ok_or(io::Error::other("line entry"))?;
    assert_eq!(directory.point_for_entry(line_entry)?, None);
    assert_eq!(line.start_value().map(bits), Some(bits3(1.0, 2.0, 3.0)));
    assert_eq!(line.endpoint_value().map(bits), Some(bits3(4.0, 5.0, 6.0)));
    assert_eq!(line.extrusion_value().map(bits), Some(bits3(2.0, 0.0, 3.0)));
    assert_eq!(line.extrusion()[0].state(), DxfSemanticValueState::Explicit);
    assert_eq!(
        line.extrusion()[1].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(line.extrusion()[2].state(), DxfSemanticValueState::Explicit);

    for entry in directory.entries().iter().copied() {
        assert_eq!(
            directory.entry_for_raw_record(entry.record().record().ordinal()),
            Some(entry)
        );
        match entry.kind() {
            DxfBasicGeometrySemanticKind::Point => assert_eq!(
                directory.point_for_raw_record(entry.record().record().ordinal())?,
                directory.point_for_entry(entry)?
            ),
            DxfBasicGeometrySemanticKind::Line => assert_eq!(
                directory.line_for_raw_record(entry.record().record().ordinal())?,
                directory.line_for_entry(entry)?
            ),
            _ => return Err(io::Error::other("unsupported semantic kind").into()),
        }
    }
    for value in point.location().iter().chain(point.extrusion()) {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
    }
    assert_eq!(
        point.location()[0].field_provenance().schema_namespace(),
        "basic_geometry.point"
    );
    assert_eq!(
        line.endpoint()[2].field_provenance().schema_field_id(),
        "endpoint_z"
    );
    Ok(())
}

fn semantic_evidence(
    directory: &DxfBasicGeometrySemanticDirectory,
) -> Result<Vec<(DxfBasicGeometryKind, Vec<TripleEvidence>)>, io::Error> {
    directory
        .entries()
        .iter()
        .copied()
        .map(|entry| {
            if let Some(point) = directory
                .point_for_entry(entry)
                .map_err(|_| io::Error::other("point projection"))?
            {
                Ok((
                    DxfBasicGeometryKind::Point,
                    vec![
                        triple_evidence(point.location()),
                        triple_evidence(point.extrusion()),
                    ],
                ))
            } else if let Some(line) = directory
                .line_for_entry(entry)
                .map_err(|_| io::Error::other("line projection"))?
            {
                Ok((
                    DxfBasicGeometryKind::Line,
                    vec![
                        triple_evidence(line.start()),
                        triple_evidence(line.endpoint()),
                        triple_evidence(line.extrusion()),
                    ],
                ))
            } else {
                Err(io::Error::other("unsupported semantic entry"))
            }
        })
        .collect()
}

fn triple_evidence(values: &[seacad_dxf_core::DxfBasicGeometrySemanticValue; 3]) -> TripleEvidence {
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
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOINT\n10\n-0\n20\n2\n30\n3\n0\nLINE\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n210\n2\n230\n3\n0\nENDSEC\n0\nEOF\n"
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
    push_string(&mut bytes, version, 0, b"POINT")?;
    for (code, value) in [(10, -0.0), (20, 2.0), (30, 3.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"LINE")?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (11, 4.0),
        (21, 5.0),
        (31, 6.0),
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

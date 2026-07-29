use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfMemorySource, DxfPolylineRecordSemanticDirectory, DxfPolylineRecordSemanticDouble,
    DxfPolylineRecordSemanticIssue, DxfPolylineRecordSemantics, DxfPolylineRecordValueCardState,
    DxfPolylineRecordValueRole, DxfReadOptions, DxfResourceProfile, DxfSemanticValueState,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct SemanticEvidence {
    doubles: [Option<u64>; 9],
    integers: [Option<i16>; 6],
    flag_bits: [Option<bool>; 8],
}

#[test]
fn every_supported_dialect_has_ascii_binary_record_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_record_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_record_semantic_directory(&DxfCancellationToken::default())?;

        let ascii_semantics = only_semantics(&ascii_directory)?;
        let binary_semantics = only_semantics(&binary_directory)?;
        assert_valid_semantics(&ascii_directory, ascii_semantics)?;
        assert_valid_semantics(&binary_directory, binary_semantics)?;
        assert_eq!(evidence(&ascii_semantics), evidence(&binary_semantics));
    }
    Ok(())
}

#[test]
fn defaults_failures_and_ignored_entities_follow_remain_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n66\n9\n0\nSEQEND\n0\nPOLYLINE\n10\n.\n20\n0\n30\n1\n30\n2\n39\n3\n40\n.\n41\n4\n41\n5\n66\n.\n70\n.\n71\n1\n71\n2\n72\n-1\n73\n.\n75\n7\n210\n.\n230\n2\n0\nVERTEX\n10\n99\n70\n255\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_record_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 2);

    let first = semantics(&directory, 0)?;
    for value in first.dummy_point() {
        assert_eq!(
            value.invalid_issue(),
            Some(&DxfPolylineRecordSemanticIssue::MissingRequiredValue)
        );
        assert_eq!(value.raw_provenance(), None);
    }
    for value in [
        first.thickness(),
        first.default_start_width(),
        first.default_end_width(),
        &first.extrusion()[0],
        &first.extrusion()[1],
        &first.extrusion()[2],
    ] {
        assert_eq!(value.state(), DxfSemanticValueState::Defaulted);
        assert_eq!(value.raw_provenance(), None);
    }
    for value in [
        first.flags(),
        first.mesh_m_vertex_count(),
        first.mesh_n_vertex_count(),
        first.smooth_surface_m_density(),
        first.smooth_surface_n_density(),
        first.smooth_surface_type(),
    ] {
        assert_eq!(value.state(), DxfSemanticValueState::Defaulted);
        assert_eq!(value.value(), Some(&0));
    }
    assert_eq!(first.flags_value(), Some(0));
    assert_eq!(
        first.extrusion_value().map(bits3),
        Some(bits3([0.0, 0.0, 1.0].map(DxfDouble::from_f64)))
    );
    let entities_follow = directory
        .card_directory()
        .card_for_role(
            first.record().sequence().polyline_record().ordinal(),
            DxfPolylineRecordValueRole::EntitiesFollow,
        )
        .ok_or(io::Error::other("entities-follow card"))?;
    assert_eq!(
        entities_follow.state(),
        DxfPolylineRecordValueCardState::Unique
    );

    let second = semantics(&directory, 1)?;
    assert_invalid_ascii(&second.dummy_point()[0]);
    assert_eq!(
        second.dummy_point()[1].state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        second.dummy_point()[2].invalid_issue(),
        Some(&DxfPolylineRecordSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_invalid_ascii(second.default_start_width());
    assert_eq!(
        second.default_end_width().invalid_issue(),
        Some(&DxfPolylineRecordSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_invalid_integer(second.flags());
    assert_eq!(second.flags_value(), None);
    assert_eq!(second.is_polyface_mesh(), None);
    assert_eq!(
        second.mesh_m_vertex_count().invalid_issue(),
        Some(&DxfPolylineRecordSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(second.mesh_n_vertex_count().value(), Some(&-1));
    assert_invalid_integer(second.smooth_surface_m_density());
    assert_eq!(
        second.smooth_surface_n_density().state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(second.smooth_surface_type().value(), Some(&7));
    assert_invalid_ascii(&second.extrusion()[0]);
    assert_eq!(
        second.extrusion()[1].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        second.extrusion()[2].state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(second.extrusion_value(), None);
    assert!(second.dummy_point()[2].raw_provenance().is_some());
    Ok(())
}

#[test]
fn sequence_states_cancellation_lookup_and_public_traits_remain_bounded()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n0\n20\n0\n30\n0\n0\nSEQEND\n0\nPOLYLINE\n10\n0\n20\n0\n30\n1\n0\nVERTEX\n0\nPOLYLINE\n10\n0\n20\n0\n30\n2\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.polyline_record_semantic_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_record_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 3);
    for record in directory.records().iter().copied() {
        let semantics = directory
            .semantics_for_entry(record)?
            .ok_or(io::Error::other("record semantics"))?;
        assert!(semantics.dummy_point_value().is_some());
        assert!(
            directory
                .semantics_for_polyline_raw_ordinal(record.sequence().polyline_record().ordinal())?
                .is_some()
        );
    }
    assert!(
        directory
            .semantics_for_polyline_raw_ordinal(u64::MAX)?
            .is_none()
    );
    assert_send_sync::<DxfPolylineRecordSemanticDirectory>();
    assert_copy::<DxfPolylineRecordSemantics>();
    Ok(())
}

fn assert_valid_semantics(
    directory: &DxfPolylineRecordSemanticDirectory,
    semantics: DxfPolylineRecordSemantics,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(
        semantics.dummy_point_value().map(bits3),
        Some([(-0.0_f64).to_bits(), 0.0_f64.to_bits(), 2.5_f64.to_bits()])
    );
    assert_eq!(semantics.flags_value(), Some(255));
    assert_eq!(
        [
            semantics.is_closed_or_mesh_closed_m(),
            semantics.has_curve_fit_vertices(),
            semantics.has_spline_fit_vertices(),
            semantics.is_3d_polyline(),
            semantics.is_polygon_mesh(),
            semantics.is_mesh_closed_n(),
            semantics.is_polyface_mesh(),
            semantics.has_continuous_linetype_pattern(),
        ],
        [Some(true); 8]
    );
    assert_eq!(semantics.mesh_m_vertex_count().value(), Some(&2));
    assert_eq!(semantics.mesh_n_vertex_count().value(), Some(&3));
    assert_eq!(semantics.smooth_surface_m_density().value(), Some(&4));
    assert_eq!(semantics.smooth_surface_n_density().value(), Some(&5));
    assert_eq!(semantics.smooth_surface_type().value(), Some(&8));
    assert_eq!(
        semantics.extrusion_value().map(bits3),
        Some([0.0_f64.to_bits(), (-0.0_f64).to_bits(), 1.0_f64.to_bits()])
    );
    for value in semantics
        .dummy_point()
        .iter()
        .chain([
            semantics.thickness(),
            semantics.default_start_width(),
            semantics.default_end_width(),
        ])
        .chain(semantics.extrusion())
    {
        assert_eq!(value.state(), DxfSemanticValueState::Explicit);
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
        assert!(value.raw_provenance().is_some());
    }
    Ok(())
}

fn evidence(semantics: &DxfPolylineRecordSemantics) -> SemanticEvidence {
    SemanticEvidence {
        doubles: [
            bits(&semantics.dummy_point()[0]),
            bits(&semantics.dummy_point()[1]),
            bits(&semantics.dummy_point()[2]),
            bits(semantics.thickness()),
            bits(semantics.default_start_width()),
            bits(semantics.default_end_width()),
            bits(&semantics.extrusion()[0]),
            bits(&semantics.extrusion()[1]),
            bits(&semantics.extrusion()[2]),
        ],
        integers: [
            semantics.flags_value(),
            semantics.mesh_m_vertex_count().value().copied(),
            semantics.mesh_n_vertex_count().value().copied(),
            semantics.smooth_surface_m_density().value().copied(),
            semantics.smooth_surface_n_density().value().copied(),
            semantics.smooth_surface_type().value().copied(),
        ],
        flag_bits: [
            semantics.is_closed_or_mesh_closed_m(),
            semantics.has_curve_fit_vertices(),
            semantics.has_spline_fit_vertices(),
            semantics.is_3d_polyline(),
            semantics.is_polygon_mesh(),
            semantics.is_mesh_closed_n(),
            semantics.is_polyface_mesh(),
            semantics.has_continuous_linetype_pattern(),
        ],
    }
}

fn only_semantics(
    directory: &DxfPolylineRecordSemanticDirectory,
) -> Result<DxfPolylineRecordSemantics, Box<dyn Error>> {
    let [record] = directory.records() else {
        return Err(io::Error::other("one record").into());
    };
    directory
        .semantics_for_entry(*record)?
        .ok_or_else(|| io::Error::other("record semantics").into())
}

fn semantics(
    directory: &DxfPolylineRecordSemanticDirectory,
    index: usize,
) -> Result<DxfPolylineRecordSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or(io::Error::other("record"))?;
    directory
        .semantics_for_entry(record)?
        .ok_or_else(|| io::Error::other("record semantics").into())
}

fn assert_invalid_ascii(value: &DxfPolylineRecordSemanticDouble) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfPolylineRecordSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert!(value.raw_provenance().is_some());
}

fn assert_invalid_integer(value: &seacad_dxf_core::DxfPolylineRecordSemanticInteger) {
    assert_eq!(
        value.invalid_issue(),
        Some(&DxfPolylineRecordSemanticIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert!(value.raw_provenance().is_some());
}

fn bits(value: &DxfPolylineRecordSemanticDouble) -> Option<u64> {
    value.value().copied().map(DxfDouble::to_bits)
}

fn bits3(values: [DxfDouble; 3]) -> [u64; 3] {
    values.map(DxfDouble::to_bits)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n-0\n20\n0\n30\n2.5\n39\n0.5\n40\n0.25\n41\n0.75\n66\n1\n70\n255\n71\n2\n72\n3\n73\n4\n74\n5\n75\n8\n210\n0\n220\n-0\n230\n1\n0\nVERTEX\n10\n99\n70\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    for (code, value) in [
        (10, -0.0),
        (20, 0.0),
        (30, 2.5),
        (39, 0.5),
        (40, 0.25),
        (41, 0.75),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [
        (66, 1),
        (70, 255),
        (71, 2),
        (72, 3),
        (73, 4),
        (74, 5),
        (75, 8),
    ] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(210, 0.0), (220, -0.0), (230, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    push_double(&mut bytes, version, 10, 99.0)?;
    push_i16(&mut bytes, version, 70, 0)?;
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

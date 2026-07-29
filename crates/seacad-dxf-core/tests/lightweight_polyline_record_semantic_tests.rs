use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfLightweightPolylineRecordSemanticDirectory, DxfLightweightPolylineRecordSemanticIssue,
    DxfLightweightPolylineRecordSemantics, DxfLightweightPolylineVertexCountComparison,
    DxfLightweightPolylineWidthEvidenceState, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSemanticValueState, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct SemanticEvidence {
    declared: Option<i32>,
    observed: u32,
    comparison: DxfLightweightPolylineVertexCountComparison,
    flags: Option<i16>,
    closed: Option<bool>,
    plinegen: Option<bool>,
    doubles: [Option<u64>; 6],
    width: DxfLightweightPolylineWidthEvidenceState,
}

#[test]
fn every_supported_dialect_has_ascii_binary_record_semantic_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii
            .lightweight_polyline_record_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary
            .lightweight_polyline_record_semantic_directory(&DxfCancellationToken::default())?;

        let ascii_semantics = only_semantics(&ascii_directory)?;
        let binary_semantics = only_semantics(&binary_directory)?;
        assert_valid_semantics(&ascii_directory, ascii_semantics)?;
        assert_valid_semantics(&binary_directory, binary_semantics)?;
        assert_eq!(evidence(&ascii_semantics), evidence(&binary_semantics));
    }
    Ok(())
}

#[test]
fn defaults_failures_count_mismatch_and_width_shapes_remain_explicit() -> Result<(), Box<dyn Error>>
{
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n10\n1\n20\n2\n0\nLWPOLYLINE\n90\n2\n90\n2\n70\n.\n38\n.\n39\n1\n39\n2\n43\n5\n210\n.\n230\n2\n10\n3\n20\n4\n0\nLWPOLYLINE\n90\n-1\n70\n0\n10\n5\n20\n6\n40\n1\n41\n2\n10\n7\n20\n8\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.lightweight_polyline_record_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 3);

    let first = semantics(&directory, 0)?;
    assert_eq!(first.observed_vertex_count(), 1);
    assert_eq!(
        first.vertex_count().invalid_issue(),
        Some(&DxfLightweightPolylineRecordSemanticIssue::MissingRequiredValue)
    );
    assert_eq!(
        first.vertex_count_comparison(),
        DxfLightweightPolylineVertexCountComparison::NotComparable
    );
    assert_eq!(first.flags().state(), DxfSemanticValueState::Defaulted);
    assert_eq!(first.flags_value(), Some(0));
    assert_eq!(first.is_closed(), Some(false));
    assert_eq!(first.has_plinegen(), Some(false));
    for value in [
        first.ocs_elevation(),
        first.thickness(),
        first.constant_width(),
        &first.extrusion()[0],
        &first.extrusion()[1],
        &first.extrusion()[2],
    ] {
        assert_eq!(value.state(), DxfSemanticValueState::Defaulted);
        assert_eq!(value.raw_provenance(), None);
    }
    assert_eq!(
        first.width_evidence(),
        DxfLightweightPolylineWidthEvidenceState::NoExplicitWidth
    );

    let second = semantics(&directory, 1)?;
    assert_eq!(
        second.vertex_count().invalid_issue(),
        Some(&DxfLightweightPolylineRecordSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        second.flags().invalid_issue(),
        Some(
            &DxfLightweightPolylineRecordSemanticIssue::InvalidAsciiNumber(
                DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
            )
        )
    );
    assert_eq!(second.flags_value(), None);
    assert_eq!(second.is_closed(), None);
    assert!(second.vertex_count().raw_provenance().is_some());
    assert!(second.flags().raw_provenance().is_some());
    assert!(second.ocs_elevation().invalid_issue().is_some());
    assert_eq!(
        second.thickness().invalid_issue(),
        Some(&DxfLightweightPolylineRecordSemanticIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert_eq!(
        second.extrusion()[0].state(),
        DxfSemanticValueState::Invalid
    );
    assert_eq!(
        second.extrusion()[1].state(),
        DxfSemanticValueState::Defaulted
    );
    assert_eq!(
        second.extrusion()[2].state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(second.extrusion_value(), None);
    assert_eq!(
        second.width_evidence(),
        DxfLightweightPolylineWidthEvidenceState::ConstantOnly {
            constant_occurrence_count: 1
        }
    );

    let third = semantics(&directory, 2)?;
    assert_eq!(third.vertex_count_value(), Some(-1));
    assert_eq!(third.observed_vertex_count(), 2);
    assert_eq!(
        third.vertex_count_comparison(),
        DxfLightweightPolylineVertexCountComparison::Mismatched {
            declared: -1,
            observed: 2
        }
    );
    assert_eq!(
        third.width_evidence(),
        DxfLightweightPolylineWidthEvidenceState::VariableOnly {
            variable_occurrence_count: 2
        }
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
        document.lightweight_polyline_record_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory =
        document.lightweight_polyline_record_semantic_directory(&DxfCancellationToken::default())?;
    assert!(directory.semantics_for_raw_record(u64::MAX)?.is_none());
    assert_send_sync::<DxfLightweightPolylineRecordSemanticDirectory>();
    assert_copy::<DxfLightweightPolylineRecordSemantics>();
    Ok(())
}

fn only_semantics(
    directory: &DxfLightweightPolylineRecordSemanticDirectory,
) -> Result<DxfLightweightPolylineRecordSemantics, Box<dyn Error>> {
    let [record] = directory.records() else {
        return Err(io::Error::other("one record").into());
    };
    directory
        .semantics_for_entry(*record)?
        .ok_or_else(|| io::Error::other("record semantics").into())
}

fn semantics(
    directory: &DxfLightweightPolylineRecordSemanticDirectory,
    index: usize,
) -> Result<DxfLightweightPolylineRecordSemantics, Box<dyn Error>> {
    let record = directory
        .records()
        .get(index)
        .copied()
        .ok_or(io::Error::other("record"))?;
    directory
        .semantics_for_entry(record)?
        .ok_or_else(|| io::Error::other("record semantics").into())
}

fn assert_valid_semantics(
    directory: &DxfLightweightPolylineRecordSemanticDirectory,
    semantics: DxfLightweightPolylineRecordSemantics,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        directory.source_id(),
        directory.card_directory().source_id()
    );
    assert_eq!(semantics.vertex_count_value(), Some(2));
    assert_eq!(semantics.observed_vertex_count(), 2);
    assert_eq!(
        semantics.vertex_count_comparison(),
        DxfLightweightPolylineVertexCountComparison::Matched { count: 2 }
    );
    assert_eq!(semantics.flags_value(), Some(129));
    assert_eq!(semantics.is_closed(), Some(true));
    assert_eq!(semantics.has_plinegen(), Some(true));
    assert_eq!(
        semantics.ocs_elevation().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        semantics
            .ocs_elevation()
            .value()
            .copied()
            .map(DxfDouble::to_bits),
        Some((-0.0_f64).to_bits())
    );
    assert_eq!(
        semantics.thickness().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        semantics.constant_width().state(),
        DxfSemanticValueState::Explicit
    );
    assert_eq!(
        semantics.extrusion()[1]
            .value()
            .copied()
            .map(DxfDouble::to_bits),
        Some((-0.0_f64).to_bits())
    );
    assert_eq!(
        semantics.width_evidence(),
        DxfLightweightPolylineWidthEvidenceState::ConstantAndVariable {
            constant_occurrence_count: 1,
            variable_occurrence_count: 1
        }
    );
    for value in [semantics.vertex_count(), semantics.flags()] {
        assert_eq!(
            value.field_provenance().document_source_id(),
            directory.source_id()
        );
        assert!(value.raw_provenance().is_some());
    }
    Ok(())
}

fn evidence(semantics: &DxfLightweightPolylineRecordSemantics) -> SemanticEvidence {
    SemanticEvidence {
        declared: semantics.vertex_count_value(),
        observed: semantics.observed_vertex_count(),
        comparison: semantics.vertex_count_comparison(),
        flags: semantics.flags_value(),
        closed: semantics.is_closed(),
        plinegen: semantics.has_plinegen(),
        doubles: [
            bits(semantics.ocs_elevation()),
            bits(semantics.thickness()),
            bits(semantics.constant_width()),
            bits(&semantics.extrusion()[0]),
            bits(&semantics.extrusion()[1]),
            bits(&semantics.extrusion()[2]),
        ],
        width: semantics.width_evidence(),
    }
}

fn bits(value: &seacad_dxf_core::DxfLightweightPolylineRecordSemanticDouble) -> Option<u64> {
    value.value().copied().map(DxfDouble::to_bits)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n70\n129\n38\n-0\n39\n0.5\n43\n1.5\n10\n1\n20\n2\n40\n0.25\n10\n3\n20\n4\n210\n0\n220\n-0\n230\n1\n0\nENDSEC\n0\nEOF\n"
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
    push_i16(&mut bytes, version, 70, 129)?;
    for (code, value) in [
        (38, -0.0),
        (39, 0.5),
        (43, 1.5),
        (10, 1.0),
        (20, 2.0),
        (40, 0.25),
        (10, 3.0),
        (20, 4.0),
        (210, 0.0),
        (220, -0.0),
        (230, 1.0),
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

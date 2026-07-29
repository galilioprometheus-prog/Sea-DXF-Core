use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfMemorySource, DxfPolylineSegmentCoordinateSystem,
    DxfPolylineSegmentSemanticDirectory, DxfPolylineSegmentSemantics, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct Evidence {
    system: DxfPolylineSegmentCoordinateSystem,
    start: Option<[u64; 3]>,
    end: Option<[u64; 3]>,
    local_widths: Option<[u64; 2]>,
    parent_widths: Option<[u64; 2]>,
    bulge: Option<u64>,
    tangent: Option<u64>,
}

#[test]
fn every_supported_dialect_has_ascii_binary_segment_semantic_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_segment_semantic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_segment_semantic_directory(&DxfCancellationToken::default())?;

        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
        let values = evidence(&ascii_directory)?;
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].system, DxfPolylineSegmentCoordinateSystem::Object);
        assert_eq!(values[1].system, DxfPolylineSegmentCoordinateSystem::World);
        assert_eq!(values[0].start, Some(bits3([1.0, 2.0, 3.0])));
        assert_eq!(values[0].end, Some(bits3([4.0, 5.0, 6.0])));
        assert_eq!(values[0].local_widths, Some(bits2([0.25, 0.5])));
        assert_eq!(values[0].parent_widths, Some(bits2([1.0, 2.0])));
        assert_eq!(values[0].bulge, Some(0.5_f64.to_bits()));
        assert_eq!(values[0].tangent, Some((-0.0_f64).to_bits()));
    }
    Ok(())
}

#[test]
fn endpoint_and_width_failures_remain_independent_without_effective_precedence()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n0\n40\n.\n41\n2\n0\nVERTEX\n70\n0\n10\n.\n20\n2\n30\n3\n40\n.\n41\n0.5\n42\n.\n50\n.\n0\nVERTEX\n70\n0\n10\n4\n20\n5\n30\n6\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.polyline_segment_semantic_directory(&DxfCancellationToken::default())?;
    let semantics = only_semantics(&directory)?;
    assert_eq!(semantics.start_position(), None);
    assert_eq!(
        semantics.end_position().map(bits3d),
        Some(bits3([4.0, 5.0, 6.0]))
    );
    assert_eq!(semantics.start_vertex_local_widths(), None);
    assert_eq!(semantics.parent_default_widths(), None);
    assert_eq!(semantics.bulge(), None);
    assert_eq!(semantics.curve_fit_tangent_direction(), None);
    assert!(
        semantics.start_vertex().position()[0]
            .raw_provenance()
            .is_some()
    );
    assert!(
        semantics
            .parent()
            .default_start_width()
            .raw_provenance()
            .is_some()
    );
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_traits_remain_bounded()
-> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_segment_semantic_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.polyline_segment_semantic_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.segment_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.vertex_semantic_directory().source_id()
    );
    assert!(directory.semantics_for_segment(u64::MAX)?.is_none());
    assert_send_sync::<DxfPolylineSegmentSemanticDirectory>();
    assert_copy::<DxfPolylineSegmentSemantics>();
    Ok(())
}

fn evidence(directory: &DxfPolylineSegmentSemanticDirectory) -> Result<Vec<Evidence>, DxfError> {
    directory
        .segment_directory()
        .segments()
        .iter()
        .map(|segment| {
            let value = directory
                .semantics_for_segment(segment.ordinal())?
                .ok_or_else(invalid_test_data)?;
            Ok(Evidence {
                system: value.coordinate_system(),
                start: value.start_position().map(bits3d),
                end: value.end_position().map(bits3d),
                local_widths: value.start_vertex_local_widths().map(bits2d),
                parent_widths: value.parent_default_widths().map(bits2d),
                bulge: value.bulge().map(DxfDouble::to_bits),
                tangent: value.curve_fit_tangent_direction().map(DxfDouble::to_bits),
            })
        })
        .collect()
}

fn only_semantics(
    directory: &DxfPolylineSegmentSemanticDirectory,
) -> Result<DxfPolylineSegmentSemantics, Box<dyn Error>> {
    directory
        .semantics_for_segment(0)?
        .ok_or_else(|| io::Error::other("segment semantics").into())
}

fn bits2(values: [f64; 2]) -> [u64; 2] {
    values.map(f64::to_bits)
}

fn bits3(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
}

fn bits2d(values: [DxfDouble; 2]) -> [u64; 2] {
    values.map(DxfDouble::to_bits)
}

fn bits3d(values: [DxfDouble; 3]) -> [u64; 3] {
    values.map(DxfDouble::to_bits)
}

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n0\n40\n1\n41\n2\n0\nVERTEX\n70\n0\n10\n1\n20\n2\n30\n3\n40\n0.25\n41\n0.5\n42\n0.5\n50\n-0\n0\nVERTEX\n70\n0\n10\n4\n20\n5\n30\n6\n0\nSEQEND\n0\nPOLYLINE\n70\n8\n40\n3\n41\n4\n0\nVERTEX\n70\n32\n10\n7\n20\n8\n30\n9\n0\nVERTEX\n70\n32\n10\n10\n20\n11\n30\n12\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
    for (parent_flags, parent_widths, vertices) in [
        (0, [1.0, 2.0], [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
        (8, [3.0, 4.0], [[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]]),
    ] {
        push_string(&mut bytes, version, 0, b"POLYLINE")?;
        push_i16(&mut bytes, version, 70, parent_flags)?;
        push_double(&mut bytes, version, 40, parent_widths[0])?;
        push_double(&mut bytes, version, 41, parent_widths[1])?;
        for (index, position) in vertices.into_iter().enumerate() {
            push_string(&mut bytes, version, 0, b"VERTEX")?;
            push_i16(
                &mut bytes,
                version,
                70,
                if parent_flags == 8 { 32 } else { 0 },
            )?;
            for (code, value) in [(10, position[0]), (20, position[1]), (30, position[2])] {
                push_double(&mut bytes, version, code, value)?;
            }
            if index == 0 && parent_flags == 0 {
                for (code, value) in [(40, 0.25), (41, 0.5), (42, 0.5), (50, -0.0)] {
                    push_double(&mut bytes, version, code, value)?;
                }
            }
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

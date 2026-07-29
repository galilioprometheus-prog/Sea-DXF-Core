use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfMemorySource, DxfPolylineSequenceState, DxfPolylineVertexNumber,
    DxfPolylineVertexNumericIssue, DxfPolylineVertexValue, DxfPolylineVertexValueDirectory,
    DxfPolylineVertexValueEntry, DxfPolylineVertexValueRange, DxfPolylineVertexValueRole,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum NumberEvidence {
    Double(u64),
    Int16(i16),
    Int32(i32),
}

type ValueEvidence = (DxfPolylineVertexValueRole, NumberEvidence);

#[test]
fn every_supported_dialect_has_ascii_binary_vertex_value_parity() -> Result<(), Box<dyn Error>> {
    let expected = expected_evidence();
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_vertex_value_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_vertex_value_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, expected);
        assert_eq!(evidence(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn duplicates_ascii_failures_and_record_locality_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n99\n70\n1\n0\nVERTEX\n10\n.\n10\n1e-9999\n20\n2\n70\n32768\n71\n-32768\n91\n2147483648\n50\n.\n75\n9\n0\nSEQEND\n10\n100\n0\nPOLYLINE\n0\nVERTEX\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_vertex_value_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.vertices().len(), 2);
    assert_eq!(directory.values().len(), 7);
    let first = directory.vertices()[0];
    let values = directory
        .values_for_vertex_raw_ordinal(first.vertex_record().ordinal())
        .ok_or(io::Error::other("first vertex values"))?;
    assert_eq!(values.len(), 7);
    assert_eq!(
        values[0].value(),
        Err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        values[1].value(),
        Err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[2].value(),
        Ok(DxfPolylineVertexNumber::Double(DxfDouble::from_f64(2.0)))
    );
    assert_eq!(
        values[3].value(),
        Err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[4].value(),
        Ok(DxfPolylineVertexNumber::Int16(i16::MIN))
    );
    assert_eq!(
        values[5].value(),
        Err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[6].value(),
        Err(DxfPolylineVertexNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );

    let second = directory.vertices()[1];
    assert!(second.value_range().is_empty());
    assert_eq!(
        directory.values_for_vertex_raw_ordinal(second.vertex_record().ordinal()),
        Some([].as_slice())
    );
    Ok(())
}

#[test]
fn vertex_values_remain_available_for_every_sequence_state_and_section()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n1\n0\nLINE\n0\nPOLYLINE\n0\nVERTEX\n10\n2\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nPOLYLINE\n0\nVERTEX\n10\n3\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_vertex_value_directory(&DxfCancellationToken::default())?;

    let expected_states = [
        DxfPolylineSequenceState::Interrupted,
        DxfPolylineSequenceState::Unclosed,
        DxfPolylineSequenceState::Closed,
    ];
    let expected_x = [1.0_f64, 2.0, 3.0];
    assert_eq!(directory.sequence_directory().sequences().len(), 3);
    assert_eq!(directory.vertices().len(), 3);
    for ((sequence, state), x) in directory
        .sequence_directory()
        .sequences()
        .iter()
        .zip(expected_states)
        .zip(expected_x)
    {
        assert_eq!(sequence.state(), state);
        let vertices = directory
            .vertices_for_polyline_raw_ordinal(sequence.polyline_record().ordinal())
            .ok_or(io::Error::other("sequence vertices"))?;
        let [vertex] = vertices else {
            return Err(io::Error::other("one sequence vertex").into());
        };
        let values = directory
            .values_for_vertex_raw_ordinal(vertex.vertex_record().ordinal())
            .ok_or(io::Error::other("sequence vertex values"))?;
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].role(), DxfPolylineVertexValueRole::LocationX);
        assert_eq!(
            values[0].value(),
            Ok(DxfPolylineVertexNumber::Double(DxfDouble::from_f64(x)))
        );
    }
    Ok(())
}

#[test]
fn cancellation_lookups_and_public_traits_remain_bounded() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.polyline_vertex_value_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_vertex_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.sequence_directory().source_id()
    );
    assert_eq!(directory.vertex_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.vertices_for_polyline_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_vertex_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_copy::<DxfPolylineVertexValue>();
    assert_copy::<DxfPolylineVertexValueEntry>();
    assert_copy::<DxfPolylineVertexValueRange>();
    assert_send_sync::<DxfPolylineVertexValueDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfPolylineVertexValueDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.sequence_directory().sequences().len(), 1);
    assert_eq!(directory.vertices().len(), 2);
    assert_eq!(directory.values().len(), 13);
    let sequence = directory.sequence_directory().sequences()[0];
    assert_eq!(sequence.state(), DxfPolylineSequenceState::Closed);
    let vertices = directory
        .vertices_for_polyline_raw_ordinal(sequence.polyline_record().ordinal())
        .ok_or(io::Error::other("polyline vertices"))?;
    assert_eq!(vertices.len(), 2);
    assert_eq!(vertices[0].sequence_vertex_ordinal(), 0);
    assert_eq!(vertices[1].sequence_vertex_ordinal(), 1);
    assert_eq!(vertices[0].value_range().len(), 13);
    assert!(vertices[1].value_range().is_empty());
    assert_eq!(vertices[0].polyline_record(), sequence.polyline_record());
    assert_eq!(vertices[1].polyline_record(), sequence.polyline_record());
    for value in directory.values() {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(*value)
        );
    }
    Ok(())
}

fn evidence(directory: &DxfPolylineVertexValueDirectory) -> Result<Vec<ValueEvidence>, io::Error> {
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            let number = match value
                .value()
                .map_err(|_| io::Error::other("vertex number"))?
            {
                DxfPolylineVertexNumber::Double(number) => NumberEvidence::Double(number.to_bits()),
                DxfPolylineVertexNumber::Int16(number) => NumberEvidence::Int16(number),
                DxfPolylineVertexNumber::Int32(number) => NumberEvidence::Int32(number),
                _ => return Err(io::Error::other("unknown vertex number")),
            };
            Ok((value.role(), number))
        })
        .collect()
}

fn expected_evidence() -> Vec<ValueEvidence> {
    use DxfPolylineVertexValueRole::{
        Bulge, CurveFitTangentDirection, EndWidth, Flags, Identifier, LocationX, LocationY,
        LocationZ, PolyfaceVertexIndex1, PolyfaceVertexIndex2, PolyfaceVertexIndex3,
        PolyfaceVertexIndex4, StartWidth,
    };
    vec![
        (LocationX, NumberEvidence::Double((-0.0_f64).to_bits())),
        (LocationY, NumberEvidence::Double(2.0_f64.to_bits())),
        (LocationZ, NumberEvidence::Double(3.0_f64.to_bits())),
        (StartWidth, NumberEvidence::Double(0.25_f64.to_bits())),
        (EndWidth, NumberEvidence::Double(0.75_f64.to_bits())),
        (Bulge, NumberEvidence::Double((-1.0_f64).to_bits())),
        (
            CurveFitTangentDirection,
            NumberEvidence::Double(45.0_f64.to_bits()),
        ),
        (Flags, NumberEvidence::Int16(192)),
        (PolyfaceVertexIndex1, NumberEvidence::Int16(1)),
        (PolyfaceVertexIndex2, NumberEvidence::Int16(-2)),
        (PolyfaceVertexIndex3, NumberEvidence::Int16(3)),
        (PolyfaceVertexIndex4, NumberEvidence::Int16(-4)),
        (Identifier, NumberEvidence::Int32(i32::MAX)),
    ]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n0\nVERTEX\n10\n-0\n20\n2\n30\n3\n40\n0.25\n41\n0.75\n42\n-1\n50\n45\n70\n192\n71\n1\n72\n-2\n73\n3\n74\n-4\n91\n2147483647\n0\nVERTEX\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
        (41, 0.75),
        (42, -1.0),
        (50, 45.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(70, 192), (71, 1), (72, -2), (73, 3), (74, -4)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, i32::MAX)?;
    push_string(&mut bytes, version, 0, b"VERTEX")?;
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

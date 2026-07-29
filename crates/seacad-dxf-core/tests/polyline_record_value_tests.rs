use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfMemorySource, DxfPolylineRecordNumber, DxfPolylineRecordNumericIssue,
    DxfPolylineRecordValue, DxfPolylineRecordValueDirectory, DxfPolylineRecordValueEntry,
    DxfPolylineRecordValueRange, DxfPolylineRecordValueRole, DxfPolylineSequenceState,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum NumberEvidence {
    Double(u64),
    Int16(i16),
}

type ValueEvidence = (DxfPolylineRecordValueRole, NumberEvidence);

#[test]
fn every_supported_dialect_has_ascii_binary_polyline_record_value_parity()
-> Result<(), Box<dyn Error>> {
    let expected = expected_evidence();
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.polyline_record_value_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.polyline_record_value_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(evidence(&ascii_directory)?, expected);
        assert_eq!(evidence(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn duplicates_ascii_failures_and_vertex_values_remain_separate() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n.\n10\n1e-9999\n20\n2\n66\n32768\n70\n-32768\n71\n.\n0\nVERTEX\n10\n99\n70\n1\n0\nSEQEND\n0\nPOLYLINE\n0\nVERTEX\n10\n100\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_record_value_directory(&DxfCancellationToken::default())?;

    assert_eq!(directory.records().len(), 2);
    assert_eq!(directory.values().len(), 6);
    let first = directory.records()[0];
    let values = directory
        .values_for_polyline_raw_ordinal(first.sequence().polyline_record().ordinal())
        .ok_or(io::Error::other("first polyline values"))?;
    assert_eq!(values.len(), 6);
    assert_eq!(
        values[0].value(),
        Err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 }
        ))
    );
    assert_eq!(
        values[1].value(),
        Err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[2].value(),
        Ok(DxfPolylineRecordNumber::Double(DxfDouble::from_f64(2.0)))
    );
    assert_eq!(
        values[3].value(),
        Err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::OutOfRange
        ))
    );
    assert_eq!(
        values[4].value(),
        Ok(DxfPolylineRecordNumber::Int16(i16::MIN))
    );
    assert_eq!(
        values[5].value(),
        Err(DxfPolylineRecordNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );

    let second = directory.records()[1];
    assert!(second.value_range().is_empty());
    assert_eq!(
        directory.values_for_polyline_raw_ordinal(second.sequence().polyline_record().ordinal()),
        Some([].as_slice())
    );
    Ok(())
}

#[test]
fn values_remain_available_for_every_sequence_state_and_later_sections()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n70\n8\n0\nVERTEX\n0\nLINE\n0\nPOLYLINE\n70\n16\n0\nVERTEX\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nPOLYLINE\n70\n64\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.polyline_record_value_directory(&DxfCancellationToken::default())?;

    let expected_states = [
        DxfPolylineSequenceState::Interrupted,
        DxfPolylineSequenceState::Unclosed,
        DxfPolylineSequenceState::Closed,
    ];
    let expected_flags = [8_i16, 16, 64];
    assert_eq!(directory.records().len(), expected_states.len());
    for ((entry, state), flags) in directory
        .records()
        .iter()
        .zip(expected_states)
        .zip(expected_flags)
    {
        assert_eq!(entry.sequence().state(), state);
        let values = directory
            .values_for_polyline_raw_ordinal(entry.sequence().polyline_record().ordinal())
            .ok_or(io::Error::other("sequence-state values"))?;
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].role(), DxfPolylineRecordValueRole::Flags);
        assert_eq!(values[0].value(), Ok(DxfPolylineRecordNumber::Int16(flags)));
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
        document.polyline_record_value_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let directory = document.polyline_record_value_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.sequence_directory().source_id()
    );
    assert_eq!(directory.record_for_polyline_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_polyline_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_copy::<DxfPolylineRecordValue>();
    assert_copy::<DxfPolylineRecordValueEntry>();
    assert_copy::<DxfPolylineRecordValueRange>();
    assert_send_sync::<DxfPolylineRecordValueDirectory>();
    Ok(())
}

fn assert_directory(directory: &DxfPolylineRecordValueDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    assert_eq!(directory.values().len(), 16);
    assert_eq!(directory.sequence_directory().sequences().len(), 1);
    let entry = directory.records()[0];
    assert_eq!(entry.sequence().state(), DxfPolylineSequenceState::Closed);
    assert_eq!(entry.value_range().len(), 16);
    assert_eq!(
        directory
            .sequence_directory()
            .vertices_for_polyline_raw_ordinal(entry.sequence().polyline_record().ordinal())
            .map(<[seacad_dxf_core::DxfRawRecord]>::len),
        Some(1)
    );
    let values = directory
        .values_for_polyline_raw_ordinal(entry.sequence().polyline_record().ordinal())
        .ok_or(io::Error::other("polyline values"))?;
    for value in values {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(*value)
        );
    }
    Ok(())
}

fn evidence(directory: &DxfPolylineRecordValueDirectory) -> Result<Vec<ValueEvidence>, io::Error> {
    directory
        .values()
        .iter()
        .copied()
        .map(|value| {
            let number = match value
                .value()
                .map_err(|_| io::Error::other("polyline number"))?
            {
                DxfPolylineRecordNumber::Double(number) => NumberEvidence::Double(number.to_bits()),
                DxfPolylineRecordNumber::Int16(number) => NumberEvidence::Int16(number),
                _ => return Err(io::Error::other("unknown polyline number")),
            };
            Ok((value.role(), number))
        })
        .collect()
}

fn expected_evidence() -> Vec<ValueEvidence> {
    use DxfPolylineRecordValueRole::{
        DefaultEndWidth, DefaultStartWidth, DummyX, DummyY, Elevation, EntitiesFollow, ExtrusionX,
        ExtrusionY, ExtrusionZ, Flags, MeshMVertexCount, MeshNVertexCount, SmoothSurfaceMDensity,
        SmoothSurfaceNDensity, SmoothSurfaceType, Thickness,
    };
    vec![
        (DummyX, NumberEvidence::Double((-0.0_f64).to_bits())),
        (DummyY, NumberEvidence::Double(0.0_f64.to_bits())),
        (Elevation, NumberEvidence::Double(2.5_f64.to_bits())),
        (Thickness, NumberEvidence::Double(0.5_f64.to_bits())),
        (
            DefaultStartWidth,
            NumberEvidence::Double(0.25_f64.to_bits()),
        ),
        (DefaultEndWidth, NumberEvidence::Double(0.75_f64.to_bits())),
        (EntitiesFollow, NumberEvidence::Int16(1)),
        (Flags, NumberEvidence::Int16(129)),
        (MeshMVertexCount, NumberEvidence::Int16(2)),
        (MeshNVertexCount, NumberEvidence::Int16(3)),
        (SmoothSurfaceMDensity, NumberEvidence::Int16(4)),
        (SmoothSurfaceNDensity, NumberEvidence::Int16(5)),
        (SmoothSurfaceType, NumberEvidence::Int16(6)),
        (ExtrusionX, NumberEvidence::Double(0.0_f64.to_bits())),
        (ExtrusionY, NumberEvidence::Double((-0.0_f64).to_bits())),
        (ExtrusionZ, NumberEvidence::Double(1.0_f64.to_bits())),
    ]
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nPOLYLINE\n10\n-0\n20\n0\n30\n2.5\n39\n0.5\n40\n0.25\n41\n0.75\n66\n1\n70\n129\n71\n2\n72\n3\n73\n4\n74\n5\n75\n6\n210\n0\n220\n-0\n230\n1\n0\nVERTEX\n10\n99\n70\n1\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
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
        (70, 129),
        (71, 2),
        (72, 3),
        (73, 4),
        (74, 5),
        (75, 6),
    ] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(210, 0.0), (220, -0.0), (230, 1.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_string(&mut bytes, version, 0, b"VERTEX")?;
    push_double(&mut bytes, version, 10, 99.0)?;
    push_i16(&mut bytes, version, 70, 1)?;
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

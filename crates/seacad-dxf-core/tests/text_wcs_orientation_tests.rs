use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfTextExtrusionComponent, DxfTextLayoutIssue, DxfTextSymbolScalarIssue,
    DxfTextWcsOrientationDirectory, DxfTextWcsOrientationIssue, DxfTextWcsOrientationSemantics,
    NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum Row {
    Default,
    Rotate90,
    BackwardUnknown,
    UpsideDown,
    BothNormalY,
}

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    x_axis: [u64; 3],
    y_axis: [u64; 3],
    normal: [u64; 3],
    flags: u16,
}

#[test]
fn every_dialect_has_ascii_binary_rotation_and_mirroring_parity() -> Result<(), Box<dyn Error>> {
    let rows = [
        Row::Default,
        Row::Rotate90,
        Row::BackwardUnknown,
        Row::UpsideDown,
        Row::BothNormalY,
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        let ascii_directory = ascii.text_wcs_orientation_directory(&cancellation)?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.text_wcs_orientation_directory(&cancellation)?;

        let actual = signatures(&ascii_directory)?;
        assert_eq!(actual, signatures(&binary_directory)?);
        assert_eq!(
            actual[0],
            signature([1., 0., 0.], [0., 1., 0.], [0., 0., 1.], 0)
        );
        let quarter_turn = 90.0_f64.to_radians();
        assert_eq!(
            actual[1],
            signature(
                [quarter_turn.cos(), quarter_turn.sin(), 0.],
                [-quarter_turn.sin(), quarter_turn.cos(), 0.],
                [0., 0., 1.],
                0,
            )
        );
        assert_eq!(
            actual[2],
            signature([-1., 0., 0.], [0., 1., 0.], [0., 0., 1.], 10)
        );
        assert_eq!(
            actual[3],
            signature([1., 0., 0.], [0., -1., 0.], [0., 0., 1.], 4)
        );
        for value in actual {
            assert_orthonormal(&value);
        }
    }
    Ok(())
}

#[test]
fn invalid_duplicate_and_zero_inputs_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n50\nbad\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n50\n1\n50\n2\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n71\nbad\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n71\n2\n71\n4\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n210\nbad\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n210\n0\n220\n0\n230\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_wcs_orientation_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        issue(&directory, 0)?,
        DxfTextWcsOrientationIssue::RotationInvalid(DxfTextSymbolScalarIssue::InvalidAsciiNumber(
            _
        ))
    ));
    assert_eq!(
        issue(&directory, 1)?,
        DxfTextWcsOrientationIssue::RotationInvalid(DxfTextSymbolScalarIssue::MultipleValues {
            occurrence_count: 2
        })
    );
    assert!(matches!(
        issue(&directory, 2)?,
        DxfTextWcsOrientationIssue::GenerationFlagsInvalid(DxfTextLayoutIssue::Scalar(
            DxfTextSymbolScalarIssue::InvalidAsciiNumber(_)
        ))
    ));
    assert_eq!(
        issue(&directory, 3)?,
        DxfTextWcsOrientationIssue::GenerationFlagsInvalid(DxfTextLayoutIssue::Scalar(
            DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2
            }
        ))
    );
    assert!(matches!(
        issue(&directory, 4)?,
        DxfTextWcsOrientationIssue::ExtrusionComponentInvalid {
            component: DxfTextExtrusionComponent::X,
            issue: DxfTextSymbolScalarIssue::InvalidAsciiNumber(_),
        }
    ));
    assert_eq!(
        issue(&directory, 5)?,
        DxfTextWcsOrientationIssue::ZeroLengthExtrusion
    );
    for index in 0..6 {
        assert!(
            semantics(&directory, index)?
                .orientation()
                .raw_provenance()
                .is_some()
        );
    }
    Ok(())
}

#[test]
fn binary_nonfinite_rotation_and_extrusion_fail_typed() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_preamble(version)?;
    push_text(&mut bytes, version, f64::NAN, 0, [0., 0., 1.])?;
    push_text(&mut bytes, version, 0., 0, [f64::INFINITY, 0., 1.])?;
    binary_epilogue(&mut bytes, version)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.text_wcs_orientation_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        issue(&directory, 0)?,
        DxfTextWcsOrientationIssue::NonFiniteRotation
    );
    assert_eq!(
        issue(&directory, 1)?,
        DxfTextWcsOrientationIssue::NonFiniteExtrusion
    );
    Ok(())
}

#[test]
fn cancellation_scope_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfTextWcsOrientationSemantics>();
    assert_send_sync::<DxfTextWcsOrientationDirectory>();

    let bytes = ascii_fixture("AC1032", &[Row::Default]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.text_wcs_orientation_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nTEXT\n10\n1\n20\n2\n30\n3\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.text_wcs_orientation_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.placement_directory().source_id()
    );
    assert_eq!(directory.records().len(), 2);
    let text = directory.records()[0];
    let shape = directory.records()[1];
    assert!(directory.semantics_for_record(text)?.is_some());
    assert_eq!(directory.semantics_for_record(shape)?, None);
    assert!(
        directory
            .semantics_for_raw_record(text.record().ordinal())?
            .is_some()
    );
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    Ok(())
}

fn signatures(directory: &DxfTextWcsOrientationDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(value)) => Some(
                value
                    .orientation()
                    .value()
                    .copied()
                    .ok_or_else(invalid_test_data)
                    .map(|orientation| Signature {
                        x_axis: orientation.x_axis().map(canonical_dxf_bits),
                        y_axis: orientation.y_axis().map(canonical_dxf_bits),
                        normal: orientation.normal().map(canonical_dxf_bits),
                        flags: orientation.generation_flags().bits(),
                    }),
            ),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfTextWcsOrientationDirectory,
    index: usize,
) -> Result<DxfTextWcsOrientationSemantics, DxfError> {
    directory
        .semantics_for_record(directory.records()[index])?
        .ok_or_else(invalid_test_data)
}

fn issue(
    directory: &DxfTextWcsOrientationDirectory,
    index: usize,
) -> Result<DxfTextWcsOrientationIssue, DxfError> {
    semantics(directory, index)?
        .orientation()
        .invalid_issue()
        .copied()
        .ok_or_else(invalid_test_data)
}

fn signature(x: [f64; 3], y: [f64; 3], normal: [f64; 3], flags: u16) -> Signature {
    Signature {
        x_axis: x.map(canonical_bits),
        y_axis: y.map(canonical_bits),
        normal: normal.map(canonical_bits),
        flags,
    }
}

fn assert_orthonormal(value: &Signature) {
    let x = value.x_axis.map(f64::from_bits);
    let y = value.y_axis.map(f64::from_bits);
    let normal = value.normal.map(f64::from_bits);
    for vector in [x, y, normal] {
        assert!((dot(vector, vector) - 1.0).abs() < 1.0e-12);
    }
    assert!(dot(x, y).abs() < 1.0e-12);
    assert!(dot(x, normal).abs() < 1.0e-12);
    assert!(dot(y, normal).abs() < 1.0e-12);
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn canonical_dxf_bits(value: seacad_dxf_core::DxfDouble) -> u64 {
    canonical_bits(value.to_f64())
}

fn canonical_bits(value: f64) -> u64 {
    (if value == 0.0 { 0.0 } else { value }).to_bits()
}

fn ascii_fixture(version: &str, rows: &[Row]) -> Vec<u8> {
    let mut entities = String::new();
    for row in rows {
        entities.push_str("0\nTEXT\n10\n1\n20\n2\n30\n3\n");
        match row {
            Row::Default => {}
            Row::Rotate90 => entities.push_str("50\n90\n"),
            Row::BackwardUnknown => entities.push_str("71\n10\n"),
            Row::UpsideDown => entities.push_str("71\n4\n"),
            Row::BothNormalY => {
                entities.push_str("50\n90\n71\n6\n210\n0\n220\n1\n230\n0\n");
            }
        }
    }
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, rows: &[Row]) -> io::Result<Vec<u8>> {
    let mut bytes = binary_preamble(version)?;
    for row in rows {
        let (rotation, flags, extrusion) = match row {
            Row::Default => (0., 0, [0., 0., 1.]),
            Row::Rotate90 => (90., 0, [0., 0., 1.]),
            Row::BackwardUnknown => (0., 10, [0., 0., 1.]),
            Row::UpsideDown => (0., 4, [0., 0., 1.]),
            Row::BothNormalY => (90., 6, [0., 1., 0.]),
        };
        push_text(&mut bytes, version, rotation, flags, extrusion)?;
    }
    binary_epilogue(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_preamble(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    Ok(bytes)
}

fn push_text(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    rotation: f64,
    flags: i16,
    extrusion: [f64; 3],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"TEXT")?;
    push_point(bytes, version, [10, 20, 30], [1., 2., 3.])?;
    push_double(bytes, version, 50, rotation)?;
    push_i16(bytes, version, 71, flags)?;
    push_point(bytes, version, [210, 220, 230], extrusion)
}

fn binary_epilogue(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_point(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    codes: [i16; 3],
    values: [f64; 3],
) -> io::Result<()> {
    for (code, value) in codes.into_iter().zip(values) {
        push_double(bytes, version, code, value)?;
    }
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

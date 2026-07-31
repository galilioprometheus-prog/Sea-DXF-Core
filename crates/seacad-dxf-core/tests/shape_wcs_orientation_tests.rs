use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfShapeInsertionComponent, DxfShapeWcsOrientationDirectory, DxfShapeWcsOrientationIssue,
    DxfShapeWcsOrientationSemantics, DxfTextSymbolScalarIssue, NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum Row {
    Default,
    Rotate90,
    Rotate90NormalY,
    RotateNegative30NormalNegativeZ,
}

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    x_axis: [u64; 3],
    y_axis: [u64; 3],
    normal: [u64; 3],
}

#[test]
fn every_dialect_has_ascii_binary_orientation_parity() -> Result<(), Box<dyn Error>> {
    let rows = [
        Row::Default,
        Row::Rotate90,
        Row::Rotate90NormalY,
        Row::RotateNegative30NormalNegativeZ,
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        let ascii_directory = ascii.shape_wcs_orientation_directory(&cancellation)?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.shape_wcs_orientation_directory(&cancellation)?;

        let ascii_signatures = signatures(&ascii_directory)?;
        assert_eq!(ascii_signatures, signatures(&binary_directory)?);
        assert_eq!(
            ascii_signatures[0],
            signature([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0])
        );
        for signature in ascii_signatures {
            assert_orthonormal(&signature);
        }
    }
    Ok(())
}

#[test]
fn invalid_duplicate_and_zero_inputs_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n50\nbad\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n50\n1\n50\n2\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n210\nbad\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n210\n0\n220\n0\n230\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.shape_wcs_orientation_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        issue(&directory, 0)?,
        DxfShapeWcsOrientationIssue::RotationInvalid(DxfTextSymbolScalarIssue::InvalidAsciiNumber(
            _
        ))
    ));
    assert_eq!(
        issue(&directory, 1)?,
        DxfShapeWcsOrientationIssue::RotationInvalid(DxfTextSymbolScalarIssue::MultipleValues {
            occurrence_count: 2,
        })
    );
    assert!(matches!(
        issue(&directory, 2)?,
        DxfShapeWcsOrientationIssue::ExtrusionComponentInvalid {
            component: DxfShapeInsertionComponent::X,
            issue: DxfTextSymbolScalarIssue::InvalidAsciiNumber(_),
        }
    ));
    assert_eq!(
        issue(&directory, 3)?,
        DxfShapeWcsOrientationIssue::ZeroLengthExtrusion
    );
    for index in 0..4 {
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
    push_shape(&mut bytes, version, f64::NAN, [0.0, 0.0, 1.0])?;
    push_shape(&mut bytes, version, 0.0, [f64::INFINITY, 0.0, 1.0])?;
    binary_epilogue(&mut bytes, version)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.shape_wcs_orientation_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        issue(&directory, 0)?,
        DxfShapeWcsOrientationIssue::NonFiniteRotation
    );
    assert_eq!(
        issue(&directory, 1)?,
        DxfShapeWcsOrientationIssue::NonFiniteExtrusion
    );
    Ok(())
}

#[test]
fn cancellation_scope_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfShapeWcsOrientationSemantics>();
    assert_send_sync::<DxfShapeWcsOrientationDirectory>();

    let bytes = ascii_fixture("AC1032", &[Row::Default]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.shape_wcs_orientation_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSHAPE\n10\n1\n20\n2\n30\n3\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.shape_wcs_orientation_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.insertion_directory().source_id()
    );
    assert_eq!(directory.records().len(), 2);
    let shape = directory.records()[0];
    let text = directory.records()[1];
    assert!(directory.semantics_for_record(shape)?.is_some());
    assert_eq!(directory.semantics_for_record(text)?, None);
    assert!(
        directory
            .semantics_for_raw_record(shape.record().ordinal())?
            .is_some()
    );
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    Ok(())
}

fn signatures(directory: &DxfShapeWcsOrientationDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(semantics)) => Some(
                semantics
                    .orientation()
                    .value()
                    .copied()
                    .ok_or_else(invalid_test_data)
                    .map(|orientation| Signature {
                        x_axis: orientation.x_axis().map(canonical_dxf_bits),
                        y_axis: orientation.y_axis().map(canonical_dxf_bits),
                        normal: orientation.normal().map(canonical_dxf_bits),
                    }),
            ),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfShapeWcsOrientationDirectory,
    index: usize,
) -> Result<DxfShapeWcsOrientationSemantics, DxfError> {
    directory
        .semantics_for_record(directory.records()[index])?
        .ok_or_else(invalid_test_data)
}

fn issue(
    directory: &DxfShapeWcsOrientationDirectory,
    index: usize,
) -> Result<DxfShapeWcsOrientationIssue, DxfError> {
    semantics(directory, index)?
        .orientation()
        .invalid_issue()
        .copied()
        .ok_or_else(invalid_test_data)
}

fn signature(x_axis: [f64; 3], y_axis: [f64; 3], normal: [f64; 3]) -> Signature {
    Signature {
        x_axis: x_axis.map(canonical_bits),
        y_axis: y_axis.map(canonical_bits),
        normal: normal.map(canonical_bits),
    }
}

fn assert_orthonormal(signature: &Signature) {
    let x = signature.x_axis.map(f64::from_bits);
    let y = signature.y_axis.map(f64::from_bits);
    let normal = signature.normal.map(f64::from_bits);
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
        entities.push_str("0\nSHAPE\n10\n1\n20\n2\n30\n3\n");
        match row {
            Row::Default => {}
            Row::Rotate90 => entities.push_str("50\n90\n"),
            Row::Rotate90NormalY => {
                entities.push_str("50\n90\n210\n0\n220\n1\n230\n0\n");
            }
            Row::RotateNegative30NormalNegativeZ => {
                entities.push_str("50\n-30\n210\n0\n220\n0\n230\n-2\n");
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
        let (rotation, extrusion) = match row {
            Row::Default => (0.0, [0.0, 0.0, 1.0]),
            Row::Rotate90 => (90.0, [0.0, 0.0, 1.0]),
            Row::Rotate90NormalY => (90.0, [0.0, 1.0, 0.0]),
            Row::RotateNegative30NormalNegativeZ => (-30.0, [0.0, 0.0, -2.0]),
        };
        push_shape(&mut bytes, version, rotation, extrusion)?;
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

fn push_shape(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    rotation: f64,
    extrusion: [f64; 3],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"SHAPE")?;
    push_point(bytes, version, [10, 20, 30], [1.0, 2.0, 3.0])?;
    push_double(bytes, version, 50, rotation)?;
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

use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfShapeInsertionComponent, DxfShapeWcsInsertionDirectory, DxfShapeWcsInsertionIssue,
    DxfShapeWcsInsertionSemantics, DxfTextSymbolScalarIssue, NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum Row {
    Default,
    NormalY,
    NormalNegativeZ,
}

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    point: [u64; 3],
    normal: [u64; 3],
}

#[test]
fn every_dialect_preserves_wcs_point_and_normal_parity() -> Result<(), Box<dyn Error>> {
    let rows = [Row::Default, Row::NormalY, Row::NormalNegativeZ];
    let expected = [
        projected([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]),
        projected([1.0, 2.0, 3.0], [0.0, 1.0, 0.0]),
        projected([1.0, 2.0, 3.0], [0.0, 0.0, -1.0]),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        let ascii_directory = ascii.shape_wcs_insertion_directory(&cancellation)?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.shape_wcs_insertion_directory(&cancellation)?;

        assert_eq!(signatures(&ascii_directory)?, expected);
        assert_eq!(signatures(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn source_failures_and_zero_extrusion_remain_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nSHAPE\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n210\nbad\n\
0\nSHAPE\n10\n1\n20\n2\n30\n3\n210\n0\n220\n0\n230\n0\n\
0\nSHAPE\n10\n1\n10\n4\n20\n2\n30\n3\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.shape_wcs_insertion_directory(&DxfCancellationToken::default())?;

    assert_eq!(
        issue(&directory, 0)?,
        DxfShapeWcsInsertionIssue::InsertionComponentInvalid {
            component: DxfShapeInsertionComponent::X,
            issue: DxfTextSymbolScalarIssue::MissingRequiredValue,
        }
    );
    assert!(matches!(
        issue(&directory, 1)?,
        DxfShapeWcsInsertionIssue::ExtrusionComponentInvalid {
            component: DxfShapeInsertionComponent::X,
            issue: DxfTextSymbolScalarIssue::InvalidAsciiNumber(_),
        }
    ));
    assert_eq!(
        issue(&directory, 2)?,
        DxfShapeWcsInsertionIssue::ZeroLengthExtrusion
    );
    assert_eq!(
        issue(&directory, 3)?,
        DxfShapeWcsInsertionIssue::InsertionComponentInvalid {
            component: DxfShapeInsertionComponent::X,
            issue: DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2,
            },
        }
    );
    for index in [1, 2, 3] {
        assert!(
            semantics(&directory, index)?
                .wcs_insertion()
                .raw_provenance()
                .is_some()
        );
    }
    Ok(())
}

#[test]
fn binary_nonfinite_inputs_and_large_wcs_point_are_exact() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_preamble(version)?;
    push_shape(&mut bytes, version, [f64::NAN, 2.0, 3.0], [0.0, 0.0, 1.0])?;
    push_shape(
        &mut bytes,
        version,
        [1.0, 2.0, 3.0],
        [f64::INFINITY, 0.0, 1.0],
    )?;
    push_shape(
        &mut bytes,
        version,
        [f64::MAX, f64::MAX, f64::MAX],
        [1.0, 1.0, 1.0],
    )?;
    binary_epilogue(&mut bytes, version)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.shape_wcs_insertion_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        issue(&directory, 0)?,
        DxfShapeWcsInsertionIssue::NonFiniteInsertion
    );
    assert_eq!(
        issue(&directory, 1)?,
        DxfShapeWcsInsertionIssue::NonFiniteExtrusion
    );
    let large = semantics(&directory, 2)?
        .wcs_insertion()
        .value()
        .copied()
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        large.point().map(|value| value.to_f64().to_bits()),
        [f64::MAX.to_bits(); 3]
    );
    Ok(())
}

#[test]
fn cancellation_scope_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfShapeWcsInsertionSemantics>();
    assert_send_sync::<DxfShapeWcsInsertionDirectory>();

    let bytes = ascii_fixture("AC1032", &[Row::Default]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.shape_wcs_insertion_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSHAPE\n10\n1\n20\n2\n30\n3\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.shape_wcs_insertion_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.scalar_directory().source_id()
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

fn signatures(directory: &DxfShapeWcsInsertionDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(semantics)) => Some(
                semantics
                    .wcs_insertion()
                    .value()
                    .copied()
                    .ok_or_else(invalid_test_data)
                    .map(|insertion| Signature {
                        point: insertion.point().map(canonical_dxf_bits),
                        normal: insertion.normal().map(canonical_dxf_bits),
                    }),
            ),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfShapeWcsInsertionDirectory,
    index: usize,
) -> Result<DxfShapeWcsInsertionSemantics, DxfError> {
    directory
        .semantics_for_record(directory.records()[index])?
        .ok_or_else(invalid_test_data)
}

fn issue(
    directory: &DxfShapeWcsInsertionDirectory,
    index: usize,
) -> Result<DxfShapeWcsInsertionIssue, DxfError> {
    semantics(directory, index)?
        .wcs_insertion()
        .invalid_issue()
        .copied()
        .ok_or_else(invalid_test_data)
}

fn projected(point: [f64; 3], normal: [f64; 3]) -> Signature {
    Signature {
        point: point.map(canonical_bits),
        normal: normal.map(canonical_bits),
    }
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
            Row::NormalY => entities.push_str("210\n0\n220\n1\n230\n0\n"),
            Row::NormalNegativeZ => entities.push_str("210\n0\n220\n0\n230\n-2\n"),
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
        let extrusion = match row {
            Row::Default => [0.0, 0.0, 1.0],
            Row::NormalY => [0.0, 1.0, 0.0],
            Row::NormalNegativeZ => [0.0, 0.0, -2.0],
        };
        push_shape(&mut bytes, version, [1.0, 2.0, 3.0], extrusion)?;
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
    point: [f64; 3],
    extrusion: [f64; 3],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"SHAPE")?;
    push_point(bytes, version, [10, 20, 30], point)?;
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
        push_code(bytes, version, code)?;
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
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

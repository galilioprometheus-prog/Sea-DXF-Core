use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfTextSymbolScalarIssue, DxfToleranceWcsComponent, DxfToleranceWcsPlacementDirectory,
    DxfToleranceWcsPlacementIssue, DxfToleranceWcsPlacementSemantics, DxfToleranceWcsVector,
    NoopDxfReadObserver,
};

#[derive(Clone, Copy)]
enum Row {
    Default,
    NormalY,
    NormalNegativeZ,
}

#[derive(Debug, Eq, PartialEq)]
struct Signature {
    insertion: [u64; 3],
    x_axis: [u64; 3],
    normal: [u64; 3],
}

#[test]
fn every_dialect_has_ascii_binary_exact_wcs_parity() -> Result<(), Box<dyn Error>> {
    let rows = [Row::Default, Row::NormalY, Row::NormalNegativeZ];
    let expected = [
        signature([1., 2., 3.], [2., 0., 0.], [0., 0., 1.]),
        signature([-0., 5., 6.], [0., 0., 5.], [0., 1., 0.]),
        signature([7., 8., 9.], [f64::MAX, 1., 0.], [0., 0., -1.]),
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code(), &rows);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let cancellation = DxfCancellationToken::default();
        let ascii_directory = ascii.tolerance_wcs_placement_directory(&cancellation)?;

        let binary_bytes = binary_fixture(version, &rows)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.tolerance_wcs_placement_directory(&cancellation)?;

        assert_eq!(signatures(&ascii_directory)?, expected);
        assert_eq!(signatures(&binary_directory)?, expected);
    }
    Ok(())
}

#[test]
fn missing_invalid_duplicate_and_zero_vectors_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nTOLERANCE\n20\n2\n30\n3\n11\n1\n21\n0\n31\n0\n\
0\nTOLERANCE\n10\nbad\n20\n2\n30\n3\n11\n1\n21\n0\n31\n0\n\
0\nTOLERANCE\n10\n1\n20\n2\n30\n3\n11\n1\n11\n2\n21\n0\n31\n0\n\
0\nTOLERANCE\n10\n1\n20\n2\n30\n3\n11\n0\n21\n0\n31\n0\n\
0\nTOLERANCE\n10\n1\n20\n2\n30\n3\n11\n1\n21\n0\n31\n0\n210\nbad\n\
0\nTOLERANCE\n10\n1\n20\n2\n30\n3\n11\n1\n21\n0\n31\n0\n210\n0\n220\n0\n230\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.tolerance_wcs_placement_directory(&DxfCancellationToken::default())?;

    assert_eq!(
        issue(&directory, 0)?,
        DxfToleranceWcsPlacementIssue::ComponentInvalid {
            vector: DxfToleranceWcsVector::Insertion,
            component: DxfToleranceWcsComponent::X,
            issue: DxfTextSymbolScalarIssue::MissingRequiredValue,
        }
    );
    assert!(matches!(
        issue(&directory, 1)?,
        DxfToleranceWcsPlacementIssue::ComponentInvalid {
            vector: DxfToleranceWcsVector::Insertion,
            component: DxfToleranceWcsComponent::X,
            issue: DxfTextSymbolScalarIssue::InvalidAsciiNumber(_),
        }
    ));
    assert_eq!(
        issue(&directory, 2)?,
        DxfToleranceWcsPlacementIssue::ComponentInvalid {
            vector: DxfToleranceWcsVector::XAxis,
            component: DxfToleranceWcsComponent::X,
            issue: DxfTextSymbolScalarIssue::MultipleValues {
                occurrence_count: 2,
            },
        }
    );
    assert_eq!(
        issue(&directory, 3)?,
        DxfToleranceWcsPlacementIssue::ZeroLengthXAxis
    );
    assert!(matches!(
        issue(&directory, 4)?,
        DxfToleranceWcsPlacementIssue::ComponentInvalid {
            vector: DxfToleranceWcsVector::Extrusion,
            component: DxfToleranceWcsComponent::X,
            issue: DxfTextSymbolScalarIssue::InvalidAsciiNumber(_),
        }
    ));
    assert_eq!(
        issue(&directory, 5)?,
        DxfToleranceWcsPlacementIssue::ZeroLengthExtrusion
    );
    for index in 1..6 {
        assert!(
            semantics(&directory, index)?
                .placement()
                .raw_provenance()
                .is_some()
        );
    }
    Ok(())
}

#[test]
fn binary_nonfinite_inputs_fail_typed() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_preamble(version)?;
    push_tolerance(
        &mut bytes,
        version,
        [f64::NAN, 2., 3.],
        [1., 0., 0.],
        Some([0., 0., 1.]),
    )?;
    push_tolerance(
        &mut bytes,
        version,
        [1., 2., 3.],
        [f64::INFINITY, 0., 0.],
        Some([0., 0., 1.]),
    )?;
    push_tolerance(
        &mut bytes,
        version,
        [1., 2., 3.],
        [1., 0., 0.],
        Some([f64::INFINITY, 0., 1.]),
    )?;
    binary_epilogue(&mut bytes, version)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.tolerance_wcs_placement_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        issue(&directory, 0)?,
        DxfToleranceWcsPlacementIssue::NonFiniteInsertion
    );
    assert_eq!(
        issue(&directory, 1)?,
        DxfToleranceWcsPlacementIssue::NonFiniteXAxis
    );
    assert_eq!(
        issue(&directory, 2)?,
        DxfToleranceWcsPlacementIssue::NonFiniteExtrusion
    );
    Ok(())
}

#[test]
fn cancellation_scope_identity_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfToleranceWcsPlacementSemantics>();
    assert_send_sync::<DxfToleranceWcsPlacementDirectory>();

    let bytes = ascii_fixture("AC1032", &[Row::Default]);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.tolerance_wcs_placement_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));

    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nTOLERANCE\n10\n1\n20\n2\n30\n3\n11\n1\n21\n0\n31\n0\n\
0\nTEXT\n10\n1\n20\n2\n30\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.tolerance_wcs_placement_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.scalar_directory().source_id()
    );
    assert_eq!(directory.records().len(), 2);
    let tolerance = directory.records()[0];
    let text = directory.records()[1];
    assert!(directory.semantics_for_record(tolerance)?.is_some());
    assert_eq!(directory.semantics_for_record(text)?, None);
    assert!(
        directory
            .semantics_for_raw_record(tolerance.record().ordinal())?
            .is_some()
    );
    assert_eq!(directory.semantics_for_raw_record(u64::MAX)?, None);
    Ok(())
}

fn signatures(directory: &DxfToleranceWcsPlacementDirectory) -> Result<Vec<Signature>, DxfError> {
    directory
        .records()
        .iter()
        .copied()
        .filter_map(|record| match directory.semantics_for_record(record) {
            Ok(Some(value)) => Some(
                value
                    .placement()
                    .value()
                    .copied()
                    .ok_or_else(invalid_test_data)
                    .map(|placement| Signature {
                        insertion: placement.insertion().map(dxf_bits),
                        x_axis: placement.x_axis().map(dxf_bits),
                        normal: placement.normal().map(dxf_bits),
                    }),
            ),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn semantics(
    directory: &DxfToleranceWcsPlacementDirectory,
    index: usize,
) -> Result<DxfToleranceWcsPlacementSemantics, DxfError> {
    directory
        .semantics_for_record(directory.records()[index])?
        .ok_or_else(invalid_test_data)
}

fn issue(
    directory: &DxfToleranceWcsPlacementDirectory,
    index: usize,
) -> Result<DxfToleranceWcsPlacementIssue, DxfError> {
    semantics(directory, index)?
        .placement()
        .invalid_issue()
        .copied()
        .ok_or_else(invalid_test_data)
}

fn signature(insertion: [f64; 3], x_axis: [f64; 3], normal: [f64; 3]) -> Signature {
    Signature {
        insertion: insertion.map(f64::to_bits),
        x_axis: x_axis.map(f64::to_bits),
        normal: normal.map(canonical_bits),
    }
}

fn dxf_bits(value: seacad_dxf_core::DxfDouble) -> u64 {
    value.to_bits()
}

fn canonical_bits(value: f64) -> u64 {
    (if value == 0.0 { 0.0 } else { value }).to_bits()
}

fn ascii_fixture(version: &str, rows: &[Row]) -> Vec<u8> {
    let mut entities = String::new();
    for row in rows {
        entities.push_str("0\nTOLERANCE\n");
        match row {
            Row::Default => {
                entities.push_str("10\n1\n20\n2\n30\n3\n11\n2\n21\n0\n31\n0\n");
            }
            Row::NormalY => {
                entities.push_str(
                    "10\n-0\n20\n5\n30\n6\n11\n0\n21\n0\n31\n5\n\
210\n0\n220\n1\n230\n0\n",
                );
            }
            Row::NormalNegativeZ => {
                entities.push_str(&format!(
                    "10\n7\n20\n8\n30\n9\n11\n{}\n21\n1\n31\n0\n\
210\n0\n220\n0\n230\n-2\n",
                    f64::MAX
                ));
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
        let (insertion, x_axis, extrusion) = match row {
            Row::Default => ([1., 2., 3.], [2., 0., 0.], None),
            Row::NormalY => ([-0., 5., 6.], [0., 0., 5.], Some([0., 1., 0.])),
            Row::NormalNegativeZ => ([7., 8., 9.], [f64::MAX, 1., 0.], Some([0., 0., -2.])),
        };
        push_tolerance(&mut bytes, version, insertion, x_axis, extrusion)?;
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

fn push_tolerance(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    insertion: [f64; 3],
    x_axis: [f64; 3],
    extrusion: Option<[f64; 3]>,
) -> io::Result<()> {
    push_string(bytes, version, 0, b"TOLERANCE")?;
    push_vector(bytes, version, [10, 20, 30], insertion)?;
    push_vector(bytes, version, [11, 21, 31], x_axis)?;
    if let Some(extrusion) = extrusion {
        push_vector(bytes, version, [210, 220, 230], extrusion)?;
    }
    Ok(())
}

fn binary_epilogue(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_vector(
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

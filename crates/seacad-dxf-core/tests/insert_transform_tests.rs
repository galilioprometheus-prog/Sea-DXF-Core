use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfInsertAffineTransform,
    DxfInsertTargetEligibilityState, DxfInsertTransformApplicationIssue,
    DxfInsertTransformDirectory, DxfInsertTransformEntry, DxfInsertTransformInput,
    DxfInsertTransformIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_supported_dialect_has_ascii_binary_transform_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.insert_transform_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_transform_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            transform_bits(&ascii_directory)?,
            transform_bits(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn unavailable_base_insert_values_zero_extrusion_and_resolution_fail_typed()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nGood\n3\nGood\n10\n0\n20\n0\n30\n0\n0\nENDBLK\n0\nBLOCK\n2\nNoBase\n3\nNoBase\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nNoBase\n10\n0\n20\n0\n30\n0\n0\nINSERT\n2\nGood\n0\nINSERT\n2\nGood\n10\n0\n20\n0\n30\n0\n210\n0\n220\n0\n230\n0\n0\nINSERT\n2\nMissing\n10\n0\n20\n0\n30\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_transform_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.transform())
            .collect::<Vec<_>>(),
        [
            Err(DxfInsertTransformIssue::BlockBasePointUnavailable),
            Err(DxfInsertTransformIssue::InsertSemanticsUnavailable),
            Err(DxfInsertTransformIssue::ZeroLengthExtrusion),
            Err(DxfInsertTransformIssue::TargetNotEligible(
                DxfInsertTargetEligibilityState::NotUniquelyResolved,
            )),
        ]
    );
    Ok(())
}

#[test]
fn matrix_application_rejects_nonfinite_input_and_overflow() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nHuge\n3\nHuge\n10\n0\n20\n0\n30\n0\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nHuge\n10\n0\n20\n0\n30\n0\n41\n1e308\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_transform_directory(&DxfCancellationToken::default())?;
    let transform = directory.entries()[0]
        .transform()
        .map_err(|_| invalid_test_data())?;
    assert_eq!(
        transform.transform_point([
            DxfDouble::from_f64(f64::NAN),
            DxfDouble::from_f64(0.0),
            DxfDouble::from_f64(0.0),
        ]),
        Err(DxfInsertTransformApplicationIssue::NonFiniteInput)
    );
    assert_eq!(
        transform.transform_point([
            DxfDouble::from_f64(2.0),
            DxfDouble::from_f64(0.0),
            DxfDouble::from_f64(0.0),
        ]),
        Err(DxfInsertTransformApplicationIssue::NonFiniteResult)
    );
    Ok(())
}

#[test]
fn nonfinite_transform_inputs_and_derived_overflow_fail_typed() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_block(&mut bytes, version, b"T", [0.0, 0.0, 0.0])?;
    push_block(&mut bytes, version, b"B", [f64::NAN, 0.0, 0.0])?;
    push_block(&mut bytes, version, b"O", [2.0, 0.0, 0.0])?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_insert(&mut bytes, version, b"T", [f64::NAN, 0.0, 0.0], None)?;
    push_insert(
        &mut bytes,
        version,
        b"T",
        [0.0, 0.0, 0.0],
        Some(([f64::NAN, 1.0, 1.0], 0.0, [0.0, 0.0, 1.0])),
    )?;
    push_insert(
        &mut bytes,
        version,
        b"T",
        [0.0, 0.0, 0.0],
        Some(([1.0, 1.0, 1.0], f64::NAN, [0.0, 0.0, 1.0])),
    )?;
    push_insert(
        &mut bytes,
        version,
        b"T",
        [0.0, 0.0, 0.0],
        Some(([1.0, 1.0, 1.0], 0.0, [f64::NAN, 0.0, 1.0])),
    )?;
    push_insert(&mut bytes, version, b"B", [0.0, 0.0, 0.0], None)?;
    push_insert(
        &mut bytes,
        version,
        b"O",
        [0.0, 0.0, 0.0],
        Some(([f64::MAX, 1.0, 1.0], 0.0, [0.0, 0.0, 1.0])),
    )?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;

    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.insert_transform_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.transform())
            .collect::<Vec<_>>(),
        [
            Err(DxfInsertTransformIssue::NonFiniteInput(
                DxfInsertTransformInput::InsertionPoint,
            )),
            Err(DxfInsertTransformIssue::NonFiniteInput(
                DxfInsertTransformInput::ScaleFactors,
            )),
            Err(DxfInsertTransformIssue::NonFiniteInput(
                DxfInsertTransformInput::RotationDegrees,
            )),
            Err(DxfInsertTransformIssue::NonFiniteInput(
                DxfInsertTransformInput::Extrusion,
            )),
            Err(DxfInsertTransformIssue::NonFiniteInput(
                DxfInsertTransformInput::BlockBasePoint,
            )),
            Err(DxfInsertTransformIssue::NonFiniteDerivedTransform),
        ]
    );
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAffineTransform>();
    assert_copy::<DxfInsertTransformEntry>();
    assert_send_sync::<DxfInsertTransformDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_transform_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.insert_transform_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.eligibility_directory().source_id()
    );
    assert_eq!(directory.entry_for_insert_raw_ordinal(u64::MAX), None);
    for entry in directory.entries() {
        let ordinal = entry.eligibility().resolution().insert().record().ordinal();
        assert_eq!(
            directory.entry_for_insert_raw_ordinal(ordinal),
            Some(*entry)
        );
    }
    Ok(())
}

fn assert_directory(directory: &DxfInsertTransformDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 2);
    let transformed = directory.entries()[0]
        .transform()
        .map_err(|_| invalid_test_data())?;
    let expected_rows = [
        [0.0, 3.0, 0.0, -16.0],
        [0.0, 0.0, 4.0, 18.0],
        [2.0, 0.0, 0.0, 18.0],
    ];
    for (row, expected) in transformed.rows().into_iter().zip(expected_rows) {
        for (actual, expected) in row.into_iter().zip(expected) {
            assert_close(actual.to_f64(), expected);
        }
    }
    let insertion_wcs = transformed
        .transform_point([
            DxfDouble::from_f64(1.0),
            DxfDouble::from_f64(2.0),
            DxfDouble::from_f64(3.0),
        ])
        .map_err(|_| invalid_test_data())?;
    for (actual, expected) in insertion_wcs.into_iter().zip([-10.0, 30.0, 20.0]) {
        assert_close(actual.to_f64(), expected);
    }
    for (actual, expected) in transformed.normal().into_iter().zip([0.0, 1.0, 0.0]) {
        assert_close(actual.to_f64(), expected);
    }

    let identity = directory.entries()[1]
        .transform()
        .map_err(|_| invalid_test_data())?;
    assert_eq!(
        identity.rows().map(|row| row.map(DxfDouble::to_bits)),
        [
            [
                1.0_f64.to_bits(),
                0.0_f64.to_bits(),
                0.0_f64.to_bits(),
                0.0_f64.to_bits()
            ],
            [
                0.0_f64.to_bits(),
                1.0_f64.to_bits(),
                0.0_f64.to_bits(),
                0.0_f64.to_bits()
            ],
            [
                0.0_f64.to_bits(),
                0.0_f64.to_bits(),
                1.0_f64.to_bits(),
                0.0_f64.to_bits()
            ],
        ]
    );
    Ok(())
}

fn transform_bits(directory: &DxfInsertTransformDirectory) -> Result<Vec<[[u64; 4]; 3]>, DxfError> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            entry
                .transform()
                .map(|transform| transform.rows().map(|row| row.map(DxfDouble::to_bits)))
                .map_err(|_| invalid_test_data())
        })
        .collect()
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1.0e-12);
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nT\n3\nT\n10\n1\n20\n2\n30\n3\n0\nENDBLK\n0\nBLOCK\n2\nI\n3\nI\n10\n0\n20\n0\n30\n0\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nT\n10\n10\n20\n20\n30\n30\n41\n2\n42\n3\n43\n4\n50\n90\n210\n0\n220\n1\n230\n0\n0\nINSERT\n2\nI\n10\n0\n20\n0\n30\n0\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_block(&mut bytes, version, b"T", [1.0, 2.0, 3.0])?;
    push_block(&mut bytes, version, b"I", [0.0, 0.0, 0.0])?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_insert(
        &mut bytes,
        version,
        b"T",
        [10.0, 20.0, 30.0],
        Some(([2.0, 3.0, 4.0], 90.0, [0.0, 1.0, 0.0])),
    )?;
    push_insert(&mut bytes, version, b"I", [0.0, 0.0, 0.0], None)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_block(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
    base: [f64; 3],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"BLOCK")?;
    push_string(bytes, version, 2, name)?;
    push_string(bytes, version, 3, name)?;
    for (code, value) in [(10, base[0]), (20, base[1]), (30, base[2])] {
        push_double(bytes, version, code, value)?;
    }
    push_string(bytes, version, 0, b"ENDBLK")
}

fn push_insert(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
    insertion: [f64; 3],
    transform: Option<([f64; 3], f64, [f64; 3])>,
) -> io::Result<()> {
    push_string(bytes, version, 0, b"INSERT")?;
    push_string(bytes, version, 2, name)?;
    for (code, value) in [(10, insertion[0]), (20, insertion[1]), (30, insertion[2])] {
        push_double(bytes, version, code, value)?;
    }
    if let Some((scale, rotation, extrusion)) = transform {
        for (code, value) in [
            (41, scale[0]),
            (42, scale[1]),
            (43, scale[2]),
            (50, rotation),
        ] {
            push_double(bytes, version, code, value)?;
        }
        for (code, value) in [
            (210, extrusion[0]),
            (220, extrusion[1]),
            (230, extrusion[2]),
        ] {
            push_double(bytes, version, code, value)?;
        }
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

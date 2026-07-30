use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfInsertArrayApplicationIssue,
    DxfInsertArrayDirectory, DxfInsertArrayEntry, DxfInsertArrayInstance, DxfInsertArrayIssue,
    DxfInsertArrayLayout, DxfInsertTargetEligibilityState, DxfInsertTransformIssue,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

type LayoutBits = ([u64; 3], [u64; 3], u16, u16);

#[test]
fn every_supported_dialect_has_ascii_binary_array_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.insert_array_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.insert_array_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            layout_bits(&ascii_directory)?,
            layout_bits(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn invalid_counts_spacing_semantics_and_target_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nT\n3\nT\n10\n0\n20\n0\n30\n0\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nT\n10\n0\n20\n0\n30\n0\n70\n0\n0\nINSERT\n2\nT\n10\n0\n20\n0\n30\n0\n71\n-2\n0\nINSERT\n2\nT\n10\n0\n20\n0\n30\n0\n70\nx\n0\nINSERT\n2\nMissing\n10\n0\n20\n0\n30\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.insert_array_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory
            .entries()
            .iter()
            .map(|entry| entry.layout())
            .collect::<Vec<_>>(),
        [
            Err(DxfInsertArrayIssue::NonPositiveColumnCount { value: 0 }),
            Err(DxfInsertArrayIssue::NonPositiveRowCount { value: -2 }),
            Err(DxfInsertArrayIssue::ArraySemanticsUnavailable),
            Err(DxfInsertArrayIssue::TransformUnavailable(
                DxfInsertTransformIssue::TargetNotEligible(
                    DxfInsertTargetEligibilityState::NotUniquelyResolved,
                ),
            )),
        ]
    );
    Ok(())
}

#[test]
fn binary_nonfinite_spacing_fails_typed() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_prefix(version)?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_block(&mut bytes, version, b"T")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_array_insert(
        &mut bytes,
        version,
        b"T",
        [0.0, 0.0, 0.0],
        2,
        1,
        [f64::NAN, 0.0],
        0.0,
        [0.0, 0.0, 1.0],
    )?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.insert_array_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.entries()[0].layout(),
        Err(DxfInsertArrayIssue::NonFiniteSpacing)
    );
    Ok(())
}

#[test]
fn maximum_counts_remain_constant_space_and_placement_overflow_is_typed()
-> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_prefix(version)?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_block(&mut bytes, version, b"T")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_array_insert(
        &mut bytes,
        version,
        b"T",
        [0.0, 0.0, 0.0],
        i16::MAX,
        i16::MAX,
        [f64::MAX, f64::MAX],
        0.0,
        [0.0, 0.0, 1.0],
    )?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.insert_array_directory(&DxfCancellationToken::default())?;
    let layout = directory.entries()[0]
        .layout()
        .map_err(|_| invalid_test_data())?;
    assert_eq!(layout.instance_count(), 1_073_676_289);
    assert!(
        layout
            .instance(0, 0)
            .map_err(|_| invalid_test_data())?
            .is_some()
    );
    assert_eq!(
        layout.instance(2, 0),
        Err(DxfInsertArrayApplicationIssue::NonFinitePlacement)
    );
    assert_eq!(layout.instance(u16::MAX, 0), Ok(None));
    Ok(())
}

#[test]
fn cancellation_lookup_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertArrayLayout>();
    assert_copy::<DxfInsertArrayInstance>();
    assert_copy::<DxfInsertArrayEntry>();
    assert_send_sync::<DxfInsertArrayDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_array_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.insert_array_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.transform_directory().source_id()
    );
    assert_eq!(directory.entry_for_insert_raw_ordinal(u64::MAX), None);
    for entry in directory.entries() {
        let ordinal = entry
            .transform_entry()
            .eligibility()
            .resolution()
            .insert()
            .record()
            .ordinal();
        assert_eq!(
            directory.entry_for_insert_raw_ordinal(ordinal),
            Some(*entry)
        );
    }
    Ok(())
}

fn assert_directory(directory: &DxfInsertArrayDirectory) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.entries().len(), 2);
    let layout = directory.entries()[0]
        .layout()
        .map_err(|_| invalid_test_data())?;
    assert_eq!((layout.column_count(), layout.row_count()), (3, 2));
    assert_eq!(layout.instance_count(), 6);
    assert_vector(layout.column_step_wcs(), [0.0, 0.0, 5.0]);
    assert_vector(layout.row_step_wcs(), [7.0, 0.0, 0.0]);
    let instance = layout
        .instance(2, 1)
        .map_err(|_| invalid_test_data())?
        .ok_or_else(invalid_test_data)?;
    assert_eq!((instance.column_index(), instance.row_index()), (2, 1));
    assert_translation(instance, [-3.0, 30.0, 30.0]);
    assert_eq!(layout.instance(3, 0), Ok(None));
    assert_eq!(layout.instance(0, 2), Ok(None));

    let defaulted = directory.entries()[1]
        .layout()
        .map_err(|_| invalid_test_data())?;
    assert_eq!((defaulted.column_count(), defaulted.row_count()), (1, 1));
    assert_eq!(defaulted.instance_count(), 1);
    assert_vector(defaulted.column_step_wcs(), [0.0, 0.0, 0.0]);
    assert_vector(defaulted.row_step_wcs(), [0.0, 0.0, 0.0]);
    assert_eq!(
        defaulted
            .instance(0, 0)
            .map_err(|_| invalid_test_data())?
            .map(DxfInsertArrayInstance::transform),
        Some(defaulted.base_transform())
    );
    Ok(())
}

fn assert_translation(instance: DxfInsertArrayInstance, expected: [f64; 3]) {
    for (row, expected) in instance.transform().rows().into_iter().zip(expected) {
        assert_close(row[3].to_f64(), expected);
    }
}

fn assert_vector(actual: [DxfDouble; 3], expected: [f64; 3]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert_close(actual.to_f64(), expected);
    }
}

fn layout_bits(directory: &DxfInsertArrayDirectory) -> Result<Vec<LayoutBits>, DxfError> {
    directory
        .entries()
        .iter()
        .map(|entry| {
            entry
                .layout()
                .map(|layout| {
                    (
                        layout.column_step_wcs().map(DxfDouble::to_bits),
                        layout.row_step_wcs().map(DxfDouble::to_bits),
                        layout.column_count(),
                        layout.row_count(),
                    )
                })
                .map_err(|_| invalid_test_data())
        })
        .collect()
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1.0e-12);
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nT\n3\nT\n10\n0\n20\n0\n30\n0\n0\nENDBLK\n0\nBLOCK\n2\nI\n3\nI\n10\n0\n20\n0\n30\n0\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nT\n10\n10\n20\n20\n30\n30\n41\n2\n42\n3\n43\n4\n50\n90\n70\n3\n71\n2\n44\n5\n45\n7\n210\n0\n220\n1\n230\n0\n0\nINSERT\n2\nI\n10\n0\n20\n0\n30\n0\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = binary_prefix(version)?;
    push_section(&mut bytes, version, b"BLOCKS")?;
    push_block(&mut bytes, version, b"T")?;
    push_block(&mut bytes, version, b"I")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_section(&mut bytes, version, b"ENTITIES")?;
    push_array_insert(
        &mut bytes,
        version,
        b"T",
        [10.0, 20.0, 30.0],
        3,
        2,
        [5.0, 7.0],
        90.0,
        [0.0, 1.0, 0.0],
    )?;
    push_simple_insert(&mut bytes, version, b"I")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn binary_prefix(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    push_section(&mut bytes, version, b"HEADER")?;
    push_string(&mut bytes, version, 9, b"$ACADVER")?;
    push_string(&mut bytes, version, 1, version.code().as_bytes())?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    Ok(bytes)
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
}

fn push_block(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"BLOCK")?;
    push_string(bytes, version, 2, name)?;
    push_string(bytes, version, 3, name)?;
    for code in [10, 20, 30] {
        push_double(bytes, version, code, 0.0)?;
    }
    push_string(bytes, version, 0, b"ENDBLK")
}

#[allow(clippy::too_many_arguments)]
fn push_array_insert(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    name: &[u8],
    insertion: [f64; 3],
    columns: i16,
    rows: i16,
    spacing: [f64; 2],
    rotation: f64,
    extrusion: [f64; 3],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"INSERT")?;
    push_string(bytes, version, 2, name)?;
    for (code, value) in [(10, insertion[0]), (20, insertion[1]), (30, insertion[2])] {
        push_double(bytes, version, code, value)?;
    }
    for (code, value) in [(41, 2.0), (42, 3.0), (43, 4.0), (50, rotation)] {
        push_double(bytes, version, code, value)?;
    }
    push_i16(bytes, version, 70, columns)?;
    push_i16(bytes, version, 71, rows)?;
    push_double(bytes, version, 44, spacing[0])?;
    push_double(bytes, version, 45, spacing[1])?;
    for (code, value) in [
        (210, extrusion[0]),
        (220, extrusion[1]),
        (230, extrusion[2]),
    ] {
        push_double(bytes, version, code, value)?;
    }
    Ok(())
}

fn push_simple_insert(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"INSERT")?;
    push_string(bytes, version, 2, name)?;
    for code in [10, 20, 30] {
        push_double(bytes, version, code, 0.0)?;
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

fn invalid_test_data() -> DxfError {
    DxfError::from_io(
        seacad_dxf_core::DxfIoOperation::Read,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

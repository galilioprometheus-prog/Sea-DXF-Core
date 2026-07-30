use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfInsertAttributePlacementAnchorState,
    DxfInsertAttributeWcsAnchorDirectory, DxfInsertAttributeWcsAnchorEntry,
    DxfInsertAttributeWcsAnchorIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
struct ProjectionSignature {
    point: [u64; 3],
    normal: [u64; 3],
}

#[test]
fn every_supported_dialect_has_ascii_binary_ocs_to_wcs_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_wcs_anchor_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_wcs_anchor_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
        assert_eq!(
            signatures(&ascii_directory)?,
            [
                projected([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]),
                projected([-1.0, 3.0, 2.0], [0.0, 1.0, 0.0]),
                projected([-1.0, 2.0, -3.0], [0.0, 0.0, -1.0]),
            ]
        );
    }
    Ok(())
}

#[test]
fn unavailable_placement_extrusion_and_zero_normal_fail_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n0\n0\nATTRIB\n70\n0\n10\n1\n20\n2\n30\n3\n210\nbad\n220\n0\n230\n1\n0\nATTRIB\n70\n0\n10\n1\n20\n2\n30\n3\n210\n0\n220\n0\n230\n0\n0\nATTRIB\n70\n0\n72\n6\n10\n1\n20\n2\n30\n3\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_wcs_anchor_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        issue(&directory, 0)?,
        DxfInsertAttributeWcsAnchorIssue::PlacementUnavailable(
            DxfInsertAttributePlacementAnchorState::TextStartUnavailable
        )
    );
    assert_eq!(
        issue(&directory, 1)?,
        DxfInsertAttributeWcsAnchorIssue::ExtrusionUnavailable
    );
    assert_eq!(
        issue(&directory, 2)?,
        DxfInsertAttributeWcsAnchorIssue::ZeroLengthExtrusion
    );
    assert_eq!(
        issue(&directory, 3)?,
        DxfInsertAttributeWcsAnchorIssue::PlacementUnavailable(
            DxfInsertAttributePlacementAnchorState::JustificationUnavailable
        )
    );
    Ok(())
}

#[test]
fn binary_non_finite_inputs_and_derived_overflow_fail_typed() -> Result<(), Box<dyn Error>> {
    let version = DxfAcadVersion::Ac1032;
    let mut bytes = binary_preamble(version)?;
    push_attribute(&mut bytes, version, [f64::NAN, 2.0, 3.0], [0.0, 0.0, 1.0])?;
    push_attribute(
        &mut bytes,
        version,
        [1.0, 2.0, 3.0],
        [f64::INFINITY, 0.0, 1.0],
    )?;
    push_attribute(
        &mut bytes,
        version,
        [f64::MAX, f64::MAX, f64::MAX],
        [1.0, 1.0, 1.0],
    )?;
    binary_epilogue(&mut bytes, version)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory =
        document.insert_attribute_wcs_anchor_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        issue(&directory, 0)?,
        DxfInsertAttributeWcsAnchorIssue::NonFinitePlacement
    );
    assert_eq!(
        issue(&directory, 1)?,
        DxfInsertAttributeWcsAnchorIssue::NonFiniteExtrusion
    );
    assert_eq!(
        issue(&directory, 2)?,
        DxfInsertAttributeWcsAnchorIssue::NonFiniteDerivedPoint
    );
    Ok(())
}

#[test]
fn cancellation_lookups_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributeWcsAnchorEntry>();
    assert_send_sync::<DxfInsertAttributeWcsAnchorDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_wcs_anchor_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.insert_attribute_wcs_anchor_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.placement_directory().source_id()
    );
    assert!(directory.entry_for_raw_record(u64::MAX)?.is_none());
    let first = directory.records()[0];
    let insert = first.sequence().insert().record().ordinal();
    assert_eq!(
        directory
            .entry_for_insert_sequence_attribute(insert, 0)?
            .map(DxfInsertAttributeWcsAnchorEntry::record),
        Some(first)
    );
    assert!(
        directory
            .entry_for_insert_sequence_attribute(insert, u64::MAX)?
            .is_none()
    );
    Ok(())
}

fn signatures(
    directory: &DxfInsertAttributeWcsAnchorDirectory,
) -> Result<Vec<ProjectionSignature>, DxfError> {
    directory
        .records()
        .iter()
        .map(|record| {
            let projection = directory
                .entry_for_attribute(*record)?
                .ok_or_else(invalid_test_data)?
                .projection()
                .map_err(|_| invalid_test_data())?;
            Ok(ProjectionSignature {
                point: projection.point().map(DxfDouble::to_bits),
                normal: projection.normal().map(DxfDouble::to_bits),
            })
        })
        .collect()
}

fn issue(
    directory: &DxfInsertAttributeWcsAnchorDirectory,
    ordinal: usize,
) -> Result<DxfInsertAttributeWcsAnchorIssue, DxfError> {
    directory
        .entry_for_attribute(directory.records()[ordinal])?
        .ok_or_else(invalid_test_data)?
        .projection()
        .err()
        .ok_or_else(invalid_test_data)
}

fn projected(point: [f64; 3], normal: [f64; 3]) -> ProjectionSignature {
    ProjectionSignature {
        point: point.map(canonical_bits),
        normal: normal.map(canonical_bits),
    }
}

fn canonical_bits(value: f64) -> u64 {
    (if value == 0.0 { 0.0 } else { value }).to_bits()
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n0\n10\n1\n20\n2\n30\n3\n0\nATTRIB\n70\n0\n10\n1\n20\n2\n30\n3\n210\n0\n220\n1\n230\n0\n0\nATTRIB\n70\n0\n10\n1\n20\n2\n30\n3\n210\n0\n220\n0\n230\n-2\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = binary_preamble(version)?;
    push_attribute(&mut bytes, version, [1.0, 2.0, 3.0], [0.0, 0.0, 1.0])?;
    push_attribute(&mut bytes, version, [1.0, 2.0, 3.0], [0.0, 1.0, 0.0])?;
    push_attribute(&mut bytes, version, [1.0, 2.0, 3.0], [0.0, 0.0, -2.0])?;
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
    push_string(&mut bytes, version, 0, b"INSERT")?;
    push_string(&mut bytes, version, 2, b"B")?;
    for code in [10, 20, 30] {
        push_double(&mut bytes, version, code, 0.0)?;
    }
    push_i16(&mut bytes, version, 66, 1)?;
    Ok(bytes)
}

fn push_attribute(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    point: [f64; 3],
    extrusion: [f64; 3],
) -> io::Result<()> {
    push_string(bytes, version, 0, b"ATTRIB")?;
    push_i16(bytes, version, 70, 0)?;
    for (code, value) in [10, 20, 30].into_iter().zip(point) {
        push_double(bytes, version, code, value)?;
    }
    for (code, value) in [210, 220, 230].into_iter().zip(extrusion) {
        push_double(bytes, version, code, value)?;
    }
    Ok(())
}

fn binary_epilogue(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"SEQEND")?;
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
}

fn push_section(bytes: &mut Vec<u8>, version: DxfAcadVersion, name: &[u8]) -> io::Result<()> {
    push_string(bytes, version, 0, b"SECTION")?;
    push_string(bytes, version, 2, name)
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

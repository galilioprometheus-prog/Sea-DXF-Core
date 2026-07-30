use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfInsertAttributePlacementAnchor,
    DxfInsertAttributePlacementAnchorDirectory, DxfInsertAttributePlacementAnchorState,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[derive(Debug, Eq, PartialEq)]
enum AnchorSignature {
    JustificationUnavailable,
    TextStartUnavailable,
    AlignmentPointUnavailable,
    TextStart([u64; 3]),
    AlignmentPoint([u64; 3]),
}

#[test]
fn every_supported_dialect_has_ascii_binary_anchor_selection_parity() -> Result<(), Box<dyn Error>>
{
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.insert_attribute_placement_anchor_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.insert_attribute_placement_anchor_directory(&DxfCancellationToken::default())?;

        assert_eq!(
            signatures(&ascii_directory)?,
            signatures(&binary_directory)?
        );
        assert_eq!(
            signatures(&ascii_directory)?,
            [
                AnchorSignature::TextStart(bits([1.0, 2.0, 3.0])),
                AnchorSignature::AlignmentPoint(bits([4.0, 5.0, 6.0])),
                AnchorSignature::TextStartUnavailable,
                AnchorSignature::AlignmentPointUnavailable,
            ]
        );
    }
    Ok(())
}

#[test]
fn only_the_applicable_tuple_controls_anchor_availability() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n0\n72\n0\n74\n0\n10\n1\n20\n2\n30\n3\n11\nbad\n21\nbad\n31\nbad\n0\nATTRIB\n70\n0\n72\n1\n74\n0\n10\nbad\n20\nbad\n30\nbad\n11\n4\n21\n5\n31\n6\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_placement_anchor_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        signature(
            directory
                .anchor_for_entry(directory.records()[0])?
                .ok_or_else(invalid_test_data)?
        )?,
        AnchorSignature::TextStart(bits([1.0, 2.0, 3.0]))
    );
    assert_eq!(
        signature(
            directory
                .anchor_for_entry(directory.records()[1])?
                .ok_or_else(invalid_test_data)?
        )?,
        AnchorSignature::AlignmentPoint(bits([4.0, 5.0, 6.0]))
    );
    Ok(())
}

#[test]
fn unavailable_justification_precedes_coordinate_selection() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n0\n72\n6\n74\n0\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory =
        document.insert_attribute_placement_anchor_directory(&DxfCancellationToken::default())?;
    let anchor = directory
        .anchor_for_raw_record(directory.records()[0].record().ordinal())?
        .ok_or_else(invalid_test_data)?;
    assert_eq!(
        anchor.state(),
        DxfInsertAttributePlacementAnchorState::JustificationUnavailable
    );
    assert_eq!(anchor.state().point(), None);
    Ok(())
}

#[test]
fn cancellation_lookups_source_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    assert_copy::<DxfInsertAttributePlacementAnchor>();
    assert_send_sync::<DxfInsertAttributePlacementAnchorDirectory>();

    let bytes = ascii_fixture("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.insert_attribute_placement_anchor_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory =
        document.insert_attribute_placement_anchor_directory(&DxfCancellationToken::default())?;
    assert_eq!(
        directory.source_id(),
        directory.justification_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.double_directory().source_id()
    );
    assert!(directory.anchor_for_raw_record(u64::MAX)?.is_none());
    let first = directory.records()[0];
    let insert = first.sequence().insert().record().ordinal();
    assert_eq!(
        directory
            .anchor_for_insert_sequence_attribute(insert, 0)?
            .map(DxfInsertAttributePlacementAnchor::record),
        Some(first)
    );
    assert!(
        directory
            .anchor_for_insert_sequence_attribute(insert, u64::MAX)?
            .is_none()
    );
    Ok(())
}

fn signatures(
    directory: &DxfInsertAttributePlacementAnchorDirectory,
) -> Result<Vec<AnchorSignature>, DxfError> {
    directory
        .records()
        .iter()
        .map(|record| {
            let anchor = directory
                .anchor_for_entry(*record)?
                .ok_or_else(invalid_test_data)?;
            signature(anchor)
        })
        .collect()
}

fn signature(anchor: DxfInsertAttributePlacementAnchor) -> Result<AnchorSignature, DxfError> {
    Ok(match anchor.state() {
        DxfInsertAttributePlacementAnchorState::JustificationUnavailable => {
            AnchorSignature::JustificationUnavailable
        }
        DxfInsertAttributePlacementAnchorState::TextStartUnavailable => {
            AnchorSignature::TextStartUnavailable
        }
        DxfInsertAttributePlacementAnchorState::AlignmentPointUnavailable => {
            AnchorSignature::AlignmentPointUnavailable
        }
        DxfInsertAttributePlacementAnchorState::TextStart { point } => {
            AnchorSignature::TextStart(point.map(DxfDouble::to_bits))
        }
        DxfInsertAttributePlacementAnchorState::AlignmentPoint { point } => {
            AnchorSignature::AlignmentPoint(point.map(DxfDouble::to_bits))
        }
        _ => return Err(invalid_test_data()),
    })
}

fn bits(values: [f64; 3]) -> [u64; 3] {
    values.map(f64::to_bits)
}

fn ascii_fixture(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nB\n10\n0\n20\n0\n30\n0\n66\n1\n0\nATTRIB\n70\n0\n72\n0\n74\n0\n10\n1\n20\n2\n30\n3\n11\nbad\n21\nbad\n31\nbad\n0\nATTRIB\n70\n0\n72\n1\n74\n0\n10\nbad\n20\nbad\n30\nbad\n11\n4\n21\n5\n31\n6\n0\nATTRIB\n70\n0\n72\n0\n74\n0\n0\nATTRIB\n70\n0\n72\n1\n74\n0\n0\nSEQEND\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
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

    push_attribute_header(&mut bytes, version, 0)?;
    for (code, value) in [(10, 1.0), (20, 2.0), (30, 3.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_attribute_header(&mut bytes, version, 1)?;
    for (code, value) in [(11, 4.0), (21, 5.0), (31, 6.0)] {
        push_double(&mut bytes, version, code, value)?;
    }
    push_attribute_header(&mut bytes, version, 0)?;
    push_attribute_header(&mut bytes, version, 1)?;

    push_string(&mut bytes, version, 0, b"SEQEND")?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_attribute_header(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    horizontal: i16,
) -> io::Result<()> {
    push_string(bytes, version, 0, b"ATTRIB")?;
    push_i16(bytes, version, 70, 0)?;
    push_i16(bytes, version, 72, horizontal)?;
    push_i16(bytes, version, 74, 0)
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

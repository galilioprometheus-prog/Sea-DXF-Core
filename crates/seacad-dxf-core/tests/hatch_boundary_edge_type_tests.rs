use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiNumericIssue, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfError,
    DxfHatchBoundaryEdgeCountRelation, DxfHatchBoundaryEdgePathState, DxfHatchBoundaryEdgeType,
    DxfHatchBoundaryEdgeTypeDirectory, DxfHatchBoundaryEdgeTypeEntry,
    DxfHatchBoundaryEdgeTypeIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_parity_for_all_four_edge_types() -> Result<(), Box<dyn Error>> {
    let expected = [
        DxfHatchBoundaryEdgeType::Line,
        DxfHatchBoundaryEdgeType::CircularArc,
        DxfHatchBoundaryEdgeType::EllipticArc,
        DxfHatchBoundaryEdgeType::Spline,
    ];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii = ascii_fixture(
            version,
            "91\n1\n92\n0\n93\n4\n72\n1\n10\n1\n72\n2\n10\n2\n72\n3\n10\n3\n72\n4\n10\n4\n97\n0\n",
        );
        let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
        let ascii_directory = open_ascii(&source)?
            .hatch_boundary_edge_type_directory(&DxfCancellationToken::default())?;
        let binary = binary_fixture(version, 4, &[1, 2, 3, 4])?;
        let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
        let binary_directory = open_binary(&source)?
            .hatch_boundary_edge_type_directory(&DxfCancellationToken::default())?;
        for directory in [&ascii_directory, &binary_directory] {
            assert_eq!(directory.entries().len(), 4);
            assert_eq!(
                directory.entries().len(),
                directory.edge_directory().edges().len()
            );
            assert_eq!(directory.entries_for_path(0), Some(directory.entries()));
            assert_eq!(
                directory
                    .entries()
                    .iter()
                    .map(|entry| entry.edge_type())
                    .collect::<Result<Vec<_>, _>>(),
                Ok(expected.to_vec())
            );
            for entry in directory.entries() {
                assert_eq!(entry.edge().marker().group().group_code().value(), 72);
                assert!(
                    !directory
                        .payload_fields_for_entry(entry.ordinal())
                        .ok_or(io::Error::other("edge payload"))?
                        .is_empty()
                );
            }
        }
    }
    Ok(())
}

#[test]
fn malformed_and_out_of_domain_values_fail_typed_without_losing_edges() -> Result<(), Box<dyn Error>>
{
    let ascii = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n4\n72\n.\n10\n1\n72\n0\n10\n2\n72\n5\n10\n3\n72\n-1\n10\n4\n",
    );
    let source = DxfMemorySource::new(&ascii, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_boundary_edge_type_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 4);
    assert!(matches!(
        directory.entries()[0].edge_type(),
        Err(DxfHatchBoundaryEdgeTypeIssue::InvalidAsciiNumber {
            issue: DxfAsciiNumericIssue::InvalidSyntax { .. },
            ..
        })
    ));
    for (entry, expected) in directory.entries()[1..].iter().zip([0_i16, 5, -1]) {
        assert!(matches!(
            entry.edge_type(),
            Err(DxfHatchBoundaryEdgeTypeIssue::ValueOutOfDomain { value, .. })
                if value == expected
        ));
    }

    let binary = binary_fixture(DxfAcadVersion::Ac1032, 3, &[0, 5, -1])?;
    let source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let directory = open_binary(&source)?
        .hatch_boundary_edge_type_directory(&DxfCancellationToken::default())?;
    for (entry, expected) in directory.entries().iter().zip([0_i16, 5, -1]) {
        assert!(matches!(
            entry.edge_type(),
            Err(DxfHatchBoundaryEdgeTypeIssue::ValueOutOfDomain { value, .. })
                if value == expected
        ));
    }
    Ok(())
}

#[test]
fn count_mismatch_empty_and_non_edge_paths_keep_independent_states() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n3\n\
         92\n0\n93\n1\n72\n1\n10\n1\n72\n2\n10\n2\n\
         92\n0\n93\n0\n\
         92\n2\n72\n0\n73\n0\n93\n0\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let directory =
        open_ascii(&source)?.hatch_boundary_edge_type_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    let DxfHatchBoundaryEdgePathState::Grouped(path) =
        directory.edge_directory().paths()[0].state()
    else {
        return Err(io::Error::other("grouped mismatched path").into());
    };
    assert_eq!(
        path.count_relation(),
        DxfHatchBoundaryEdgeCountRelation::Mismatched {
            declared: 1,
            observed: 2
        }
    );
    assert_eq!(
        directory
            .entries_for_path(0)
            .ok_or(io::Error::other("typed path"))?
            .len(),
        2
    );
    assert_eq!(directory.entries_for_path(1), Some(&[][..]));
    assert!(directory.entries_for_path(2).is_none());
    assert_eq!(
        directory.edge_directory().paths()[2].state(),
        DxfHatchBoundaryEdgePathState::NotEdges
    );
    Ok(())
}

#[test]
fn cancellation_bounds_identity_traits_and_redaction_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(
        DxfAcadVersion::Ac1032,
        "91\n1\n92\n0\n93\n1\n72\n1\n10\n12345.625\n",
    );
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancellation = DxfCancellationToken::default();
    cancellation.cancel();
    assert!(matches!(
        document.hatch_boundary_edge_type_directory(&cancellation),
        Err(DxfError::Cancelled)
    ));
    let directory = document.hatch_boundary_edge_type_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), Some(directory.entries()[0]));
    assert_eq!(directory.entry(u64::MAX), None);
    assert!(directory.payload_fields_for_entry(u64::MAX).is_none());
    assert!(directory.entries_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.edge_directory().source_id()
    );
    assert!(
        size_of::<DxfHatchBoundaryEdgeTypeEntry>() <= 192,
        "typed edge entry is {} bytes",
        size_of::<DxfHatchBoundaryEdgeTypeEntry>()
    );
    assert!(!format!("{:?}", directory.entries()[0]).contains("12345.625"));
    copy::<DxfHatchBoundaryEdgeTypeEntry>();
    send_sync::<DxfHatchBoundaryEdgeTypeDirectory>();
    Ok(())
}

fn ascii_fixture(version: DxfAcadVersion, payload: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n{}75\n0\n0\nENDSEC\n0\nEOF\n",
        version.code(), payload
    )
    .into_bytes()
}

fn binary_fixture(
    version: DxfAcadVersion,
    declared: i32,
    edge_types: &[i16],
) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, declared)?;
    for edge_type in edge_types {
        push_i16(&mut bytes, version, 72, *edge_type)?;
        push_double(&mut bytes, version, 10, f64::from(*edge_type))?;
    }
    push_i16(&mut bytes, version, 75, 0)?;
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
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

fn copy<T: Copy>() {}
fn send_sync<T: Send + Sync>() {}

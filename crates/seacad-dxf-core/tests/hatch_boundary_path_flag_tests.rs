use std::{error::Error, io};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundaryPathFlagDirectory,
    DxfHatchBoundaryPathFlagEntry, DxfHatchBoundaryPathFlagIssue, DxfHatchBoundaryPathKind,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_decodes_all_documented_flag_bits() -> Result<(), Box<dyn Error>> {
    let expected = [0, 1, 2, 4, 8, 16, 31];
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version, &expected.map(|value| value.to_string()));
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.hatch_boundary_path_flag_directory(&DxfCancellationToken::default())?;
        let binary_bytes = binary_fixture(version, &expected)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.hatch_boundary_path_flag_directory(&DxfCancellationToken::default())?;
        assert_flags(&ascii_directory, &expected)?;
        assert_flags(&binary_directory, &expected)?;
    }
    Ok(())
}

#[test]
fn invalid_negative_and_unsupported_bits_fail_closed() -> Result<(), Box<dyn Error>> {
    let values = [
        ".".to_owned(),
        "-1".to_owned(),
        "32".to_owned(),
        "34".to_owned(),
    ];
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032, &values);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_boundary_path_flag_directory(&DxfCancellationToken::default())?;
    assert!(matches!(
        directory.entries()[0].state(),
        Err(DxfHatchBoundaryPathFlagIssue::InvalidAsciiNumber { .. })
    ));
    assert!(matches!(
        directory.entries()[1].state(),
        Err(DxfHatchBoundaryPathFlagIssue::Negative { value: -1, .. })
    ));
    assert!(matches!(
        directory.entries()[2].state(),
        Err(DxfHatchBoundaryPathFlagIssue::UnsupportedBits {
            value: 32,
            unsupported_bits: 32,
            ..
        })
    ));
    assert!(matches!(
        directory.entries()[3].state(),
        Err(DxfHatchBoundaryPathFlagIssue::UnsupportedBits {
            value: 34,
            unsupported_bits: 32,
            ..
        })
    ));
    Ok(())
}

#[test]
fn duplicate_subclasses_and_non_path_records_remain_isolated() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n1\n92\n2\n75\n0\n100\nAcDbHatch\n91\n1\n92\n0\n75\n0\n0\nMESH\n100\nAcDbSubDMesh\n91\n1\n92\n2\n75\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_boundary_path_flag_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    assert_eq!(
        available(directory.entries()[0])?.kind(),
        DxfHatchBoundaryPathKind::Polyline
    );
    assert_eq!(
        available(directory.entries()[1])?.kind(),
        DxfHatchBoundaryPathKind::Edges
    );
    assert_eq!(directory.entries_for_subclass(0).len(), 1);
    assert_eq!(directory.entries_for_subclass(1).len(), 1);
    let raw = directory.entries()[0].raw_record_ordinal();
    assert_eq!(directory.entries_for_raw_record(raw).len(), 2);
    Ok(())
}

#[test]
fn cancellation_bounds_identity_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let values = ["2".to_owned()];
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032, &values);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_boundary_path_flag_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));
    let directory = document.hatch_boundary_path_flag_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(0), directory.entry_for_path(0));
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_path(u64::MAX), None);
    assert!(directory.entries_for_subclass(u64::MAX).is_empty());
    assert!(directory.entries_for_raw_record(u64::MAX).is_empty());
    assert_eq!(
        directory.source_id(),
        directory.path_directory().source_id()
    );
    assert_copy::<DxfHatchBoundaryPathFlagEntry>();
    assert_send_sync::<DxfHatchBoundaryPathFlagDirectory>();
    assert!(size_of::<DxfHatchBoundaryPathFlagEntry>() <= 256);
    Ok(())
}

fn assert_flags(
    directory: &DxfHatchBoundaryPathFlagDirectory,
    expected: &[i32],
) -> Result<(), io::Error> {
    assert_eq!(directory.entries().len(), expected.len());
    for (entry, raw) in directory
        .entries()
        .iter()
        .copied()
        .zip(expected.iter().copied())
    {
        let value = available(entry)?;
        assert_eq!(value.flags().raw(), raw as u32);
        assert_eq!(
            value.kind(),
            if raw & 2 != 0 {
                DxfHatchBoundaryPathKind::Polyline
            } else {
                DxfHatchBoundaryPathKind::Edges
            }
        );
    }
    let all = available(
        *directory
            .entries()
            .last()
            .ok_or_else(|| io::Error::other("last flag"))?,
    )?
    .flags();
    assert!(
        all.is_external()
            && all.is_polyline()
            && all.is_derived()
            && all.is_textbox()
            && all.is_outermost()
    );
    Ok(())
}

fn available(
    entry: DxfHatchBoundaryPathFlagEntry,
) -> Result<seacad_dxf_core::DxfHatchBoundaryPathFlagValue, io::Error> {
    entry
        .state()
        .map_err(|_| io::Error::other("available path flags"))
}

fn ascii_fixture(version: DxfAcadVersion, values: &[String]) -> Vec<u8> {
    let mut text = format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n91\n{}\n",
        version.code(),
        values.len()
    );
    for value in values {
        text.push_str(&format!("92\n{value}\n"));
    }
    text.push_str("75\n0\n0\nENDSEC\n0\nEOF\n");
    text.into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, values: &[i32]) -> io::Result<Vec<u8>> {
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
    push_i32(
        &mut bytes,
        version,
        91,
        i32::try_from(values.len()).map_err(|_| io::Error::other("count"))?,
    )?;
    for value in values {
        push_i32(&mut bytes, version, 92, *value)?;
    }
    push_i16(&mut bytes, version, 75, 0)?;
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
fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("code"))?);
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

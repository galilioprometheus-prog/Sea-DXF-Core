use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_HELIX_ROLES, DxfAcadVersion, DxfAsciiNumericIssue,
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble,
    DxfError, DxfHelixDirectory, DxfHelixNumber, DxfHelixNumericIssue, DxfHelixRecordEntry,
    DxfHelixValue, DxfHelixValueRole, DxfMemorySource, DxfRawRecordSectionKind, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

type Evidence = (DxfHelixValueRole, DxfHelixNumber);

#[test]
fn every_dialect_has_ascii_binary_helix_evidence_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.helix_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.helix_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory, version)?;
        assert_directory(&binary_directory, version)?;
        assert_eq!(evidence(&ascii_directory)?, evidence(&binary_directory)?);
        assert_eq!(evidence(&ascii_directory)?, expected_evidence(version));
    }
    Ok(())
}

#[test]
fn duplicates_invalid_values_and_scope_decoys_remain_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n40\n90\n100\nAcDbSpline\n40\n91\n\
100\nAcDbHelix\n90\n.\n40\n1\n102\n{APP\n40\n98\n102\n}\n40\n2\n290\n2\n\
100\nAcDbHelixX\n41\n92\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.helix_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 1);
    let record = directory.records()[0];
    let values = directory
        .values_for_raw_record(record.entity().record().ordinal())
        .ok_or(io::Error::other("helix values"))?;
    assert_eq!(values.len(), 4);
    assert_eq!(values[0].role(), DxfHelixValueRole::MajorVersion);
    assert_eq!(
        values[0].value(),
        Err(DxfHelixNumericIssue::InvalidAsciiNumber(
            DxfAsciiNumericIssue::InvalidSyntax { token_offset: 0 }
        ))
    );
    assert_eq!(
        values[1].value(),
        Ok(DxfHelixNumber::Double(DxfDouble::from_f64(1.0)))
    );
    assert_eq!(
        values[2].value(),
        Ok(DxfHelixNumber::Double(DxfDouble::from_f64(2.0)))
    );
    assert_eq!(values[3].value(), Ok(DxfHelixNumber::Int16(2)));
    assert!(values.iter().all(|value| !matches!(
        value.value(),
        Ok(DxfHelixNumber::Double(number)) if number.to_f64() >= 90.0
    )));
    assert_eq!(
        directory.value_for_group(values[2].group().occurrence()),
        Some(values[2])
    );
    Ok(())
}

#[test]
fn matching_is_exact_subclass_aware_and_entity_section_limited() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nHELIX\n100\nAcDbHelix\n90\n1\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nHELIX\n100\nAcDbHelix\n91\n2\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nhelix\n100\nAcDbHelix\n40\n1\n\
0\nHELIX \n100\nAcDbHelix\n40\n2\n0\nHELIX\n100\nacdbhelix\n40\n3\n\
0\nHELIX\n100\nAcDbHelix\n40\n4\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.helix_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.records().len(), 3);
    assert_eq!(directory.values().len(), 2);
    assert_eq!(
        directory.records()[0].entity().record().section_kind(),
        DxfRawRecordSectionKind::Blocks
    );
    assert!(directory.records()[1].value_range().is_empty());
    assert_eq!(
        directory.values()[0].role(),
        DxfHelixValueRole::MaintenanceVersion
    );
    assert_eq!(directory.values()[1].role(), DxfHelixValueRole::Radius);
    Ok(())
}

#[test]
fn cancellation_lookup_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.helix_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.helix_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.helix_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.record_for_raw_ordinal(u64::MAX), None);
    assert_eq!(directory.values_for_raw_record(u64::MAX), None);
    assert_eq!(directory.value_for_group(u64::MAX), None);
    assert_eq!(
        directory.source_id(),
        directory.entity_directory().source_id()
    );
    assert_eq!(
        directory.source_id(),
        directory.application_group_directory().source_id()
    );
    assert_copy::<DxfHelixValue>();
    assert_copy::<DxfHelixRecordEntry>();
    assert_send_sync::<DxfHelixDirectory>();
    assert!(size_of::<DxfHelixValue>() <= 96);
    assert!(size_of::<DxfHelixRecordEntry>() <= 224);
    Ok(())
}

fn assert_directory(
    directory: &DxfHelixDirectory,
    version: DxfAcadVersion,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(directory.records().len(), 1);
    let record = directory.records()[0];
    let values = directory
        .values_for_raw_record(record.entity().record().ordinal())
        .ok_or(io::Error::other("helix values"))?;
    let expected_len = if version == DxfAcadVersion::Ac1009 {
        DXF_HELIX_ROLES.len() - 2
    } else {
        DXF_HELIX_ROLES.len()
    };
    assert_eq!(values.len(), expected_len);
    assert_eq!(values.len() as u64, record.value_range().len());
    for value in values.iter().copied() {
        assert_eq!(
            directory.value_for_group(value.group().occurrence()),
            Some(value)
        );
    }
    Ok(())
}

fn evidence(directory: &DxfHelixDirectory) -> Result<Vec<Evidence>, io::Error> {
    let record = directory
        .records()
        .first()
        .copied()
        .ok_or(io::Error::other("helix record"))?;
    directory
        .values_for_raw_record(record.entity().record().ordinal())
        .ok_or(io::Error::other("helix values"))?
        .iter()
        .map(|value| {
            Ok((
                value.role(),
                value
                    .value()
                    .map_err(|_| io::Error::other("helix number"))?,
            ))
        })
        .collect()
}

fn expected_evidence(version: DxfAcadVersion) -> Vec<Evidence> {
    use DxfHelixValueRole::*;
    let mut expected = vec![
        (MajorVersion, DxfHelixNumber::Int32(1)),
        (MaintenanceVersion, DxfHelixNumber::Int32(0)),
    ];
    for (role, value) in [
        (AxisBaseX, 1.0),
        (AxisBaseY, 2.0),
        (AxisBaseZ, 3.0),
        (StartPointX, 4.0),
        (StartPointY, 5.0),
        (StartPointZ, 6.0),
        (AxisVectorX, 0.0),
        (AxisVectorY, 0.0),
        (AxisVectorZ, 1.0),
        (Radius, 4.0),
        (Turns, 2.5),
        (TurnHeight, -3.0),
    ] {
        expected.push((role, DxfHelixNumber::Double(DxfDouble::from_f64(value))));
    }
    if version != DxfAcadVersion::Ac1009 {
        expected.push((Handedness, DxfHelixNumber::Int16(1)));
        expected.push((ConstraintType, DxfHelixNumber::Int16(2)));
    }
    expected
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    let modern_tail = if version == DxfAcadVersion::Ac1009 {
        ""
    } else {
        "290\n1\n280\n2\n"
    };
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHELIX\n100\nAcDbSpline\n40\n99\n\
100\nAcDbHelix\n90\n1\n91\n0\n10\n1\n20\n2\n30\n3\n11\n4\n21\n5\n31\n6\n\
12\n0\n22\n0\n32\n1\n40\n4\n41\n2.5\n42\n-3\n{modern_tail}0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"HELIX"),
        (100, b"AcDbSpline"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 40, 99.0)?;
    push_string(&mut bytes, version, 100, b"AcDbHelix")?;
    push_i32(&mut bytes, version, 90, 1)?;
    push_i32(&mut bytes, version, 91, 0)?;
    for (code, value) in [
        (10, 1.0),
        (20, 2.0),
        (30, 3.0),
        (11, 4.0),
        (21, 5.0),
        (31, 6.0),
        (12, 0.0),
        (22, 0.0),
        (32, 1.0),
        (40, 4.0),
        (41, 2.5),
        (42, -3.0),
    ] {
        push_double(&mut bytes, version, code, value)?;
    }
    if version != DxfAcadVersion::Ac1009 {
        push_code(&mut bytes, version, 290)?;
        bytes.push(1);
        push_i16(&mut bytes, version, 280, 2)?;
    }
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

struct CancellingSource<'a> {
    bytes: &'a [u8],
    token: DxfCancellationToken,
    armed: AtomicBool,
}

impl<'a> CancellingSource<'a> {
    fn new(bytes: &'a [u8], token: DxfCancellationToken) -> Self {
        Self {
            bytes,
            token,
            armed: AtomicBool::new(false),
        }
    }

    fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }
}

impl DxfByteSource for CancellingSource<'_> {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = self.bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        if self.armed.load(Ordering::Acquire) {
            self.token.cancel();
        }
        Ok(count)
    }
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}

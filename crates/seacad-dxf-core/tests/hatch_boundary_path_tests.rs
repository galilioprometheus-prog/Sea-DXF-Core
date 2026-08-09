use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundaryPartitionIssue, DxfHatchBoundaryPathCountIssue,
    DxfHatchBoundaryPathCountRelation, DxfHatchBoundaryPathDirectory, DxfHatchBoundaryPathEntry,
    DxfHatchBoundaryPathTopology, DxfHatchBoundaryPathTopologyEntry,
    DxfHatchBoundaryPathTopologyIssue, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    NoopDxfReadObserver,
};

#[test]
fn every_dialect_groups_ascii_binary_paths_and_matches_count() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.hatch_boundary_path_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.hatch_boundary_path_directory(&DxfCancellationToken::default())?;

        assert_directory(&ascii_directory)?;
        assert_directory(&binary_directory)?;
        assert_eq!(
            payload_codes(&ascii_directory)?,
            payload_codes(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn mismatch_invalid_negative_and_partition_failures_remain_typed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n\
100\nAcDbHatch\n91\n3\n92\n0\n40\n1\n92\n0\n40\n2\n75\n0\n\
100\nAcDbHatch\n91\n.\n92\n0\n75\n0\n\
100\nAcDbHatch\n91\n-1\n75\n0\n\
100\nAcDbHatch\n91\n1\n92\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_boundary_path_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 4);

    assert_eq!(
        available_topology(directory.entries()[0])?.count_relation(),
        DxfHatchBoundaryPathCountRelation::Mismatched {
            declared: 3,
            observed: 2
        }
    );
    assert!(matches!(
        available_topology(directory.entries()[1])?.count_relation(),
        DxfHatchBoundaryPathCountRelation::DeclaredUnavailable(
            DxfHatchBoundaryPathCountIssue::InvalidAsciiNumber { .. }
        )
    ));
    assert!(matches!(
        available_topology(directory.entries()[2])?.count_relation(),
        DxfHatchBoundaryPathCountRelation::DeclaredUnavailable(
            DxfHatchBoundaryPathCountIssue::Negative { value: -1, .. }
        )
    ));
    assert_eq!(
        directory.entries()[3].topology(),
        Err(DxfHatchBoundaryPathTopologyIssue::PartitionUnavailable(
            DxfHatchBoundaryPartitionIssue::HatchStyleAbsent
        ))
    );
    assert!(
        directory
            .paths_for_subclass(directory.entries()[3].subclass_ordinal())
            .is_none()
    );
    assert!(
        directory
            .orphan_fields_for_subclass(directory.entries()[3].subclass_ordinal())
            .is_none()
    );
    Ok(())
}

#[test]
fn orphan_fields_duplicate_subclasses_and_empty_paths_stay_exact() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n\
100\nAcDbHatch\n91\n1\n40\n7\n92\n2\n10\n1\n20\n2\n75\n0\n\
100\nAcDbHatch\n91\n0\n40\n8\n75\n1\n\
0\nMESH\n100\nAcDbSubDMesh\n91\n1\n92\n0\n75\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_boundary_path_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    assert_eq!(directory.paths().len(), 1);

    let first = available_topology(directory.entries()[0])?;
    assert_eq!(first.path_count(), 1);
    assert_eq!(first.orphan_field_count(), 1);
    assert_eq!(
        codes(
            directory
                .orphan_fields_for_subclass(directory.entries()[0].subclass_ordinal())
                .ok_or_else(|| io::Error::other("first orphan fields"))?
        ),
        vec![40]
    );
    assert_eq!(
        first.count_relation(),
        DxfHatchBoundaryPathCountRelation::Matched { count: 1 }
    );
    assert_eq!(
        payload_codes_for_path(&directory, directory.paths()[0])?,
        vec![10, 20]
    );

    let second = available_topology(directory.entries()[1])?;
    assert_eq!(second.path_count(), 0);
    assert_eq!(second.orphan_field_count(), 1);
    assert_eq!(
        second.count_relation(),
        DxfHatchBoundaryPathCountRelation::Matched { count: 0 }
    );
    assert!(
        directory
            .paths_for_subclass(directory.entries()[1].subclass_ordinal())
            .ok_or_else(|| io::Error::other("second path slice"))?
            .is_empty()
    );
    let raw = directory.entries()[0].raw_record_ordinal();
    assert_eq!(directory.entries_for_raw_record(raw).len(), 2);
    Ok(())
}

#[test]
fn cancellation_lookup_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_boundary_path_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.hatch_boundary_path_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.hatch_boundary_path_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one topology entry").into());
    };
    assert_eq!(directory.entry(entry.ordinal()), Some(*entry));
    assert_eq!(
        directory.entry_for_subclass(entry.subclass_ordinal()),
        Some(*entry)
    );
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_subclass(u64::MAX), None);
    assert!(directory.entries_for_raw_record(u64::MAX).is_empty());
    assert_eq!(directory.path(u64::MAX), None);
    assert!(directory.payload_fields_for_path(u64::MAX).is_none());
    assert_eq!(
        directory.source_id(),
        directory.partition_directory().source_id()
    );
    assert_copy::<DxfHatchBoundaryPathEntry>();
    assert_copy::<DxfHatchBoundaryPathTopologyEntry>();
    assert_send_sync::<DxfHatchBoundaryPathDirectory>();
    assert!(size_of::<DxfHatchBoundaryPathEntry>() <= 80);
    assert!(size_of::<DxfHatchBoundaryPathTopologyEntry>() <= 96);
    Ok(())
}

fn assert_directory(directory: &DxfHatchBoundaryPathDirectory) -> Result<(), io::Error> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one topology entry"));
    };
    let topology = entry
        .topology()
        .map_err(|_| io::Error::other("available topology"))?;
    assert_eq!(topology.path_count(), 2);
    assert_eq!(topology.orphan_field_count(), 0);
    assert_eq!(
        topology.count_relation(),
        DxfHatchBoundaryPathCountRelation::Matched { count: 2 }
    );
    let paths = directory
        .paths_for_subclass(entry.subclass_ordinal())
        .ok_or_else(|| io::Error::other("path slice"))?;
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].subclass_path_ordinal(), 0);
    assert_eq!(paths[1].subclass_path_ordinal(), 1);
    assert!(
        paths
            .iter()
            .all(|path| path.marker().group().group_code().value() == 92)
    );
    Ok(())
}

fn available_topology(
    entry: DxfHatchBoundaryPathTopologyEntry,
) -> Result<DxfHatchBoundaryPathTopology, io::Error> {
    entry
        .topology()
        .map_err(|_| io::Error::other("available topology"))
}

fn payload_codes(directory: &DxfHatchBoundaryPathDirectory) -> Result<Vec<Vec<i16>>, io::Error> {
    directory
        .paths()
        .iter()
        .copied()
        .map(|path| payload_codes_for_path(directory, path))
        .collect()
}

fn payload_codes_for_path(
    directory: &DxfHatchBoundaryPathDirectory,
    path: DxfHatchBoundaryPathEntry,
) -> Result<Vec<i16>, io::Error> {
    Ok(codes(
        directory
            .payload_fields_for_path(path.ordinal())
            .ok_or_else(|| io::Error::other("path payload"))?,
    ))
}

fn codes(fields: &[seacad_dxf_core::DxfFillMeshField]) -> Vec<i16> {
    fields
        .iter()
        .map(|field| field.group().group_code().value())
        .collect()
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n10\n0\n20\n0\n30\n0\n\
91\n2\n92\n2\n72\n0\n73\n1\n93\n1\n10\n1\n20\n2\n\
92\n0\n93\n1\n72\n1\n10\n3\n20\n4\n11\n5\n21\n6\n75\n0\n\
0\nENDSEC\n0\nEOF\n",
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
        (0, b"HATCH"),
        (100, b"AcDbHatch"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    push_double(&mut bytes, version, 10, 0.0)?;
    push_double(&mut bytes, version, 20, 0.0)?;
    push_double(&mut bytes, version, 30, 0.0)?;
    push_i32(&mut bytes, version, 91, 2)?;
    push_i32(&mut bytes, version, 92, 2)?;
    push_i16(&mut bytes, version, 72, 0)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i32(&mut bytes, version, 93, 1)?;
    push_double(&mut bytes, version, 10, 1.0)?;
    push_double(&mut bytes, version, 20, 2.0)?;
    push_i32(&mut bytes, version, 92, 0)?;
    push_i32(&mut bytes, version, 93, 1)?;
    push_i16(&mut bytes, version, 72, 1)?;
    push_double(&mut bytes, version, 10, 3.0)?;
    push_double(&mut bytes, version, 20, 4.0)?;
    push_double(&mut bytes, version, 11, 5.0)?;
    push_double(&mut bytes, version, 21, 6.0)?;
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

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
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

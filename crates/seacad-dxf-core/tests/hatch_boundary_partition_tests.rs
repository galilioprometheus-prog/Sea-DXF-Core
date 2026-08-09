use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfError, DxfHatchBoundaryPartition, DxfHatchBoundaryPartitionDirectory,
    DxfHatchBoundaryPartitionEntry, DxfHatchBoundaryPartitionIssue, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_partitions_ascii_binary_boundary_envelope() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_partition_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory =
            ascii.hatch_boundary_partition_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_partition_fixture(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory =
            binary.hatch_boundary_partition_directory(&DxfCancellationToken::default())?;

        assert_partition(&ascii_directory)?;
        assert_partition(&binary_directory)?;
        assert_eq!(
            partition_codes(&ascii_directory)?,
            partition_codes(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn missing_duplicate_and_reversed_anchors_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n\
100\nAcDbHatch\n75\n0\n\
100\nAcDbHatch\n91\n1\n91\n2\n75\n0\n\
100\nAcDbHatch\n91\n1\n92\n0\n\
100\nAcDbHatch\n91\n1\n75\n0\n75\n1\n\
100\nAcDbHatch\n75\n0\n91\n1\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_boundary_partition_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 5);
    assert_eq!(
        directory.entries()[0].partition(),
        Err(DxfHatchBoundaryPartitionIssue::BoundaryPathCountAbsent)
    );
    assert_eq!(
        directory.entries()[1].partition(),
        Err(DxfHatchBoundaryPartitionIssue::BoundaryPathCountMultiple { count: 2 })
    );
    assert_eq!(
        directory.entries()[2].partition(),
        Err(DxfHatchBoundaryPartitionIssue::HatchStyleAbsent)
    );
    assert_eq!(
        directory.entries()[3].partition(),
        Err(DxfHatchBoundaryPartitionIssue::HatchStyleMultiple { count: 2 })
    );
    assert_eq!(
        directory.entries()[4].partition(),
        Err(DxfHatchBoundaryPartitionIssue::AnchorOrderInvalid)
    );
    for entry in directory.entries() {
        assert!(
            directory
                .header_fields_for_subclass(entry.subclass_ordinal())
                .is_none()
        );
        assert!(
            directory
                .boundary_fields_for_subclass(entry.subclass_ordinal())
                .is_none()
        );
        assert!(
            directory
                .trailing_fields_for_subclass(entry.subclass_ordinal())
                .is_none()
        );
    }
    Ok(())
}

#[test]
fn duplicate_subclasses_remain_independent_and_raw_payload_stays_reachable()
-> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n\
100\nAcDbHatch\n30\n3\n91\n1\n92\n2\n10\n1\n20\n2\n40\n7\n75\n0\n98\n1\n10\n8\n20\n9\n\
100\nAcDbHatch\n30\n4\n91\n0\n75\n1\n\
0\nMESH\n100\nAcDbSubDMesh\n91\n1\n75\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_boundary_partition_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    let raw = directory.entries()[0]
        .subclass()
        .entity()
        .record()
        .ordinal();
    assert_eq!(directory.entries_for_raw_record(raw).len(), 2);
    assert!(
        directory
            .entries_for_raw_record(
                directory.entries()[1]
                    .subclass()
                    .entity()
                    .record()
                    .ordinal()
            )
            .len()
            == 2
    );
    assert_eq!(
        codes(
            directory
                .boundary_fields_for_subclass(0)
                .ok_or_else(|| { io::Error::other("first boundary fields") })?
        ),
        vec![92, 10, 20, 40]
    );
    assert!(
        directory
            .boundary_fields_for_subclass(1)
            .ok_or_else(|| io::Error::other("second boundary fields"))?
            .is_empty()
    );
    for entry in directory.entries() {
        assert_eq!(directory.entry(entry.ordinal()), Some(*entry));
        assert_eq!(
            directory.entry_for_subclass(entry.subclass_ordinal()),
            Some(*entry)
        );
    }
    assert_eq!(
        directory.source_id(),
        directory.evidence_directory().source_id()
    );
    Ok(())
}

#[test]
fn cancellation_bounds_and_public_traits_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_partition_fixture(DxfAcadVersion::Ac1032);
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_boundary_partition_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.hatch_boundary_partition_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.hatch_boundary_partition_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_subclass(u64::MAX), None);
    assert!(directory.entries_for_raw_record(u64::MAX).is_empty());
    assert!(directory.header_fields_for_subclass(u64::MAX).is_none());
    assert!(directory.boundary_fields_for_subclass(u64::MAX).is_none());
    assert!(directory.trailing_fields_for_subclass(u64::MAX).is_none());
    assert_copy::<DxfHatchBoundaryPartition>();
    assert_copy::<DxfHatchBoundaryPartitionEntry>();
    assert_send_sync::<DxfHatchBoundaryPartitionDirectory>();
    assert!(
        size_of::<DxfHatchBoundaryPartitionEntry>() <= 304,
        "entry size {}",
        size_of::<DxfHatchBoundaryPartitionEntry>()
    );
    Ok(())
}

fn assert_partition(directory: &DxfHatchBoundaryPartitionDirectory) -> Result<(), io::Error> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one HATCH partition"));
    };
    let partition = entry
        .partition()
        .map_err(|_| io::Error::other("available boundary partition"))?;
    assert_eq!(
        partition.boundary_path_count().group().group_code().value(),
        91
    );
    assert_eq!(partition.hatch_style().group().group_code().value(), 75);
    assert_eq!(partition.header_range().len(), 4);
    assert_eq!(partition.boundary_range().len(), 8);
    assert_eq!(partition.trailing_range().len(), 6);
    Ok(())
}

fn partition_codes(
    directory: &DxfHatchBoundaryPartitionDirectory,
) -> Result<[Vec<i16>; 3], io::Error> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one HATCH partition"));
    };
    let ordinal = entry.subclass_ordinal();
    Ok([
        codes(
            directory
                .header_fields_for_subclass(ordinal)
                .ok_or_else(|| io::Error::other("header fields"))?,
        ),
        codes(
            directory
                .boundary_fields_for_subclass(ordinal)
                .ok_or_else(|| io::Error::other("boundary fields"))?,
        ),
        codes(
            directory
                .trailing_fields_for_subclass(ordinal)
                .ok_or_else(|| io::Error::other("trailing fields"))?,
        ),
    ])
}

fn codes(fields: &[seacad_dxf_core::DxfFillMeshField]) -> Vec<i16> {
    fields
        .iter()
        .map(|field| field.group().group_code().value())
        .collect()
}

fn ascii_partition_fixture(version: DxfAcadVersion) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n\
10\n0\n20\n0\n30\n5\n91\n1\n92\n2\n72\n0\n73\n1\n93\n1\n\
10\n1\n20\n2\n42\n0\n97\n0\n75\n0\n76\n1\n78\n0\n98\n1\n10\n9\n20\n8\n\
0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_partition_fixture(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
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
    push_double(&mut bytes, version, 30, 5.0)?;
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, 2)?;
    push_i16(&mut bytes, version, 72, 0)?;
    push_i16(&mut bytes, version, 73, 1)?;
    push_i32(&mut bytes, version, 93, 1)?;
    push_double(&mut bytes, version, 10, 1.0)?;
    push_double(&mut bytes, version, 20, 2.0)?;
    push_double(&mut bytes, version, 42, 0.0)?;
    push_i32(&mut bytes, version, 97, 0)?;
    push_i16(&mut bytes, version, 75, 0)?;
    push_i16(&mut bytes, version, 76, 1)?;
    push_i16(&mut bytes, version, 78, 0)?;
    push_i32(&mut bytes, version, 98, 1)?;
    push_double(&mut bytes, version, 10, 9.0)?;
    push_double(&mut bytes, version, 20, 8.0)?;
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

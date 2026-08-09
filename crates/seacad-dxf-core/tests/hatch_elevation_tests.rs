use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfHatchBoundaryPartitionIssue, DxfHatchElevation,
    DxfHatchElevationComponentIssue, DxfHatchElevationDirectory, DxfHatchElevationEntry,
    DxfHatchElevationEntryState, DxfHatchElevationIssue, DxfMemorySource, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_exact_elevation_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version);
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.hatch_elevation_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, 5.25_f64.to_bits(), f64::NAN.to_bits())?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.hatch_elevation_directory(&DxfCancellationToken::default())?;

        let ascii_elevation = only_elevation(&ascii_directory)?;
        let binary_elevation = only_elevation(&binary_directory)?;
        assert_eq!(ascii_elevation.values(), binary_elevation.values());
        assert_eq!(
            ascii_elevation.values(),
            [
                DxfDouble::from_f64(0.0),
                DxfDouble::from_f64(-0.0),
                DxfDouble::from_f64(5.25),
            ]
        );
        assert_eq!(
            [
                ascii_elevation.x().group().group_code().value(),
                ascii_elevation.y().group().group_code().value(),
                ascii_elevation.z().group().group_code().value(),
            ],
            [10, 20, 30]
        );
    }
    Ok(())
}

#[test]
fn boundary_seed_decoys_and_duplicate_subclasses_remain_isolated() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n\
100\nAcDbHatch\n10\n0\n20\n0\n30\n3\n91\n1\n92\n2\n10\n11\n20\n12\n30\n.\n75\n0\n98\n1\n10\n21\n20\n22\n\
100\nAcDbHatch\n10\n-0\n20\n0\n30\n4\n91\n0\n75\n1\n\
0\nMESH\n100\nAcDbSubDMesh\n10\n8\n20\n9\n30\n10\n91\n0\n75\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_elevation_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    assert_eq!(
        available(directory.entries()[0])?.values(),
        doubles([0.0, 0.0, 3.0])
    );
    assert_eq!(
        available(directory.entries()[1])?.values(),
        doubles([-0.0, 0.0, 4.0])
    );
    let raw = directory.entries()[0].raw_record_ordinal();
    assert_eq!(directory.entries_for_raw_record(raw).len(), 2);
    assert_eq!(
        directory.entries()[0].subclass_ordinal() + 1,
        directory.entries()[1].subclass_ordinal()
    );
    Ok(())
}

#[test]
fn unavailable_components_domains_and_partition_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n\
100\nAcDbHatch\n20\n0\n30\n1\n91\n0\n75\n0\n\
100\nAcDbHatch\n10\n0\n20\n0\n20\n-0\n30\n1\n91\n0\n75\n0\n\
100\nAcDbHatch\n10\n0\n20\n0\n30\n.\n91\n0\n75\n0\n\
100\nAcDbHatch\n10\n1\n20\n-2\n30\n3\n91\n0\n75\n0\n\
100\nAcDbHatch\n10\n0\n20\n0\n30\n1\n75\n0\n\
0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_elevation_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 5);

    let DxfHatchElevationEntryState::Components(first) = directory.entries()[0].state() else {
        return Err(io::Error::other("component state").into());
    };
    assert_eq!(first.x(), Err(DxfHatchElevationComponentIssue::Absent));
    assert_unavailable(directory.entries()[0], [true, false, false])?;

    let DxfHatchElevationEntryState::Components(second) = directory.entries()[1].state() else {
        return Err(io::Error::other("component state").into());
    };
    assert_eq!(
        second.y(),
        Err(DxfHatchElevationComponentIssue::Multiple {
            occurrence_count: 2
        })
    );
    assert_unavailable(directory.entries()[1], [false, true, false])?;

    let DxfHatchElevationEntryState::Components(third) = directory.entries()[2].state() else {
        return Err(io::Error::other("component state").into());
    };
    assert!(matches!(
        third.z(),
        Err(DxfHatchElevationComponentIssue::InvalidAsciiNumber { .. })
    ));
    assert_unavailable(directory.entries()[2], [false, false, true])?;

    let DxfHatchElevationEntryState::Components(fourth) = directory.entries()[3].state() else {
        return Err(io::Error::other("component state").into());
    };
    assert!(matches!(
        fourth.x(),
        Err(DxfHatchElevationComponentIssue::PlanarComponentNonZero { .. })
    ));
    assert!(matches!(
        fourth.y(),
        Err(DxfHatchElevationComponentIssue::PlanarComponentNonZero { .. })
    ));
    assert_unavailable(directory.entries()[3], [true, true, false])?;

    assert_eq!(
        directory.entries()[4].state(),
        DxfHatchElevationEntryState::PartitionUnavailable(
            DxfHatchBoundaryPartitionIssue::BoundaryPathCountAbsent
        )
    );
    assert_eq!(
        directory.entries()[4].elevation(),
        Err(DxfHatchElevationIssue::PartitionUnavailable(
            DxfHatchBoundaryPartitionIssue::BoundaryPathCountAbsent
        ))
    );

    let version = DxfAcadVersion::Ac1032;
    let binary = binary_fixture(version, f64::NAN.to_bits(), 0.0_f64.to_bits())?;
    let binary_source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let binary_document = open_binary(&binary_source)?;
    let binary_directory =
        binary_document.hatch_elevation_directory(&DxfCancellationToken::default())?;
    let DxfHatchElevationEntryState::Components(binary_components) =
        binary_directory.entries()[0].state()
    else {
        return Err(io::Error::other("binary component state").into());
    };
    assert!(matches!(
        binary_components.z(),
        Err(DxfHatchElevationComponentIssue::NonFiniteDouble { .. })
    ));
    assert_unavailable(binary_directory.entries()[0], [false, false, true])?;
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
        document.hatch_elevation_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));

    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.hatch_elevation_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));

    let directory = document.hatch_elevation_directory(&DxfCancellationToken::default())?;
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one elevation entry").into());
    };
    assert_eq!(directory.entry(entry.ordinal()), Some(*entry));
    assert_eq!(
        directory.entry_for_subclass(entry.subclass_ordinal()),
        Some(*entry)
    );
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_subclass(u64::MAX), None);
    assert!(directory.entries_for_raw_record(u64::MAX).is_empty());
    assert_eq!(
        directory.source_id(),
        directory.partition_directory().source_id()
    );
    assert_copy::<DxfHatchElevation>();
    assert_copy::<DxfHatchElevationEntry>();
    assert_send_sync::<DxfHatchElevationDirectory>();
    assert!(size_of::<DxfHatchElevationEntry>() <= 256);
    Ok(())
}

fn only_elevation(directory: &DxfHatchElevationDirectory) -> Result<DxfHatchElevation, io::Error> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one elevation entry"));
    };
    entry
        .elevation()
        .map_err(|_| io::Error::other("available elevation"))
}

fn available(entry: DxfHatchElevationEntry) -> Result<DxfHatchElevation, io::Error> {
    entry
        .elevation()
        .map_err(|_| io::Error::other("available elevation"))
}

fn assert_unavailable(entry: DxfHatchElevationEntry, expected: [bool; 3]) -> Result<(), io::Error> {
    let Err(DxfHatchElevationIssue::ComponentsUnavailable(mask)) = entry.elevation() else {
        return Err(io::Error::other("unavailable elevation components"));
    };
    assert_eq!([mask.x(), mask.y(), mask.z()], expected);
    assert_eq!(
        mask.count(),
        expected.into_iter().filter(|value| *value).count() as u32
    );
    Ok(())
}

fn doubles(values: [f64; 3]) -> [DxfDouble; 3] {
    values.map(DxfDouble::from_f64)
}

fn ascii_fixture(version: DxfAcadVersion) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n\
10\n0\n20\n-0\n30\n5.25\n91\n1\n92\n2\n10\n7\n20\n8\n30\n.\n75\n0\n\
98\n1\n10\n9\n20\n10\n0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(
    version: DxfAcadVersion,
    header_z_bits: u64,
    boundary_z_bits: u64,
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
    push_double(&mut bytes, version, 10, 0.0)?;
    push_double(&mut bytes, version, 20, -0.0)?;
    push_double_bits(&mut bytes, version, 30, header_z_bits)?;
    push_i32(&mut bytes, version, 91, 1)?;
    push_i32(&mut bytes, version, 92, 2)?;
    push_double(&mut bytes, version, 10, 7.0)?;
    push_double(&mut bytes, version, 20, 8.0)?;
    push_double_bits(&mut bytes, version, 30, boundary_z_bits)?;
    push_i16(&mut bytes, version, 75, 0)?;
    push_i32(&mut bytes, version, 98, 1)?;
    push_double(&mut bytes, version, 10, 9.0)?;
    push_double(&mut bytes, version, 20, 10.0)?;
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
    push_double_bits(bytes, version, code, value.to_bits())
}

fn push_double_bits(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    bits: u64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&bits.to_le_bytes());
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

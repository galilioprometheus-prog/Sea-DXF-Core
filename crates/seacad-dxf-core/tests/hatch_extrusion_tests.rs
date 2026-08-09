use std::{
    error::Error,
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfHatchExtrusion, DxfHatchExtrusionDirectory,
    DxfHatchExtrusionEntry, DxfHatchExtrusionInputKind, DxfHatchExtrusionIssue, DxfMemorySource,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_ascii_binary_explicit_extrusion_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = ascii_fixture(version, "210\n2\n220\n-3\n230\n4\n");
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.hatch_extrusion_directory(&DxfCancellationToken::default())?;

        let binary_bytes = binary_fixture(version, Some([2.0, -3.0, 4.0]))?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.hatch_extrusion_directory(&DxfCancellationToken::default())?;

        let ascii_extrusion = only_extrusion(&ascii_directory)?;
        let binary_extrusion = only_extrusion(&binary_directory)?;
        assert_eq!(ascii_extrusion, binary_extrusion);
        assert_eq!(ascii_extrusion.values(), doubles([2.0, -3.0, 4.0]));
        assert_eq!(
            [
                ascii_extrusion.x().input_kind(),
                ascii_extrusion.y().input_kind(),
                ascii_extrusion.z().input_kind(),
            ],
            [DxfHatchExtrusionInputKind::Explicit; 3]
        );
    }
    Ok(())
}

#[test]
fn absent_and_partial_components_use_reviewed_defaults() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n\
0\nHATCH\n100\nAcDbHatch\n210\n2\n\
0\nHATCH\n100\nAcDbHatch\n220\n-3\n230\n4\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_extrusion_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 3);
    let first = available(directory.entries()[0])?;
    assert_eq!(first.values(), doubles([0.0, 0.0, 1.0]));
    assert_eq!(
        [
            first.x().input_kind(),
            first.y().input_kind(),
            first.z().input_kind(),
        ],
        [DxfHatchExtrusionInputKind::Defaulted; 3]
    );
    let second = available(directory.entries()[1])?;
    assert_eq!(second.values(), doubles([2.0, 0.0, 1.0]));
    assert_eq!(
        second.x().input_kind(),
        DxfHatchExtrusionInputKind::Explicit
    );
    assert_eq!(
        second.y().input_kind(),
        DxfHatchExtrusionInputKind::Defaulted
    );
    assert_eq!(
        second.z().input_kind(),
        DxfHatchExtrusionInputKind::Defaulted
    );
    let third = available(directory.entries()[2])?;
    assert_eq!(third.values(), doubles([0.0, -3.0, 4.0]));
    assert_eq!(
        third.x().input_kind(),
        DxfHatchExtrusionInputKind::Defaulted
    );
    assert_eq!(third.y().input_kind(), DxfHatchExtrusionInputKind::Explicit);
    assert_eq!(third.z().input_kind(), DxfHatchExtrusionInputKind::Explicit);
    Ok(())
}

#[test]
fn unavailable_components_and_exact_zero_vectors_fail_closed() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nHATCH\n100\nAcDbHatch\n210\n1\n210\n2\n\
0\nHATCH\n100\nAcDbHatch\n220\n.\n\
0\nHATCH\n100\nAcDbHatch\n210\n0\n220\n-0\n230\n0\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.hatch_extrusion_directory(&DxfCancellationToken::default())?;
    let Err(DxfHatchExtrusionIssue::ComponentsUnavailable(first)) =
        directory.entries()[0].extrusion()
    else {
        return Err(io::Error::other("duplicate x unavailable").into());
    };
    assert!(first.x());
    assert!(!first.y());
    assert!(!first.z());
    assert_eq!(first.count(), 1);
    let Err(DxfHatchExtrusionIssue::ComponentsUnavailable(second)) =
        directory.entries()[1].extrusion()
    else {
        return Err(io::Error::other("invalid y unavailable").into());
    };
    assert!(!second.x());
    assert!(second.y());
    assert!(!second.z());
    assert_eq!(
        directory.entries()[2].extrusion(),
        Err(DxfHatchExtrusionIssue::ZeroVector)
    );

    let version = DxfAcadVersion::Ac1032;
    let mut binary = binary_prefix(version)?;
    push_double_bits(&mut binary, version, 210, f64::NAN.to_bits())?;
    binary_suffix(&mut binary, version)?;
    let binary_source = DxfMemorySource::new(&binary, DxfResourceProfile::Safe)?;
    let binary_document = open_binary(&binary_source)?;
    let binary_directory =
        binary_document.hatch_extrusion_directory(&DxfCancellationToken::default())?;
    let Err(DxfHatchExtrusionIssue::ComponentsUnavailable(nonfinite)) =
        binary_directory.entries()[0].extrusion()
    else {
        return Err(io::Error::other("nonfinite x unavailable").into());
    };
    assert!(nonfinite.x());
    assert_eq!(nonfinite.count(), 1);
    Ok(())
}

#[test]
fn cancellation_duplicate_subclass_lookup_and_public_bounds_hold() -> Result<(), Box<dyn Error>> {
    let bytes = ascii_fixture(DxfAcadVersion::Ac1032, "210\n1\n100\nAcDbHatch\n220\n2\n");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        document.hatch_extrusion_directory(&cancelled),
        Err(DxfError::Cancelled)
    ));
    let mid_token = DxfCancellationToken::default();
    let cancelling_source = CancellingSource::new(&bytes, mid_token.clone());
    let cancelling_document = open_ascii(&cancelling_source)?;
    cancelling_source.arm();
    assert!(matches!(
        cancelling_document.hatch_extrusion_directory(&mid_token),
        Err(DxfError::Cancelled)
    ));
    let directory = document.hatch_extrusion_directory(&DxfCancellationToken::default())?;
    assert_eq!(directory.entries().len(), 2);
    let raw = directory.entries()[0]
        .scalar_entry()
        .subclass()
        .entity()
        .record()
        .ordinal();
    assert_eq!(directory.entries_for_raw_record(raw).len(), 2);
    for entry in directory.entries() {
        assert_eq!(
            directory.entry_for_subclass(entry.scalar_entry().subclass_ordinal()),
            Some(*entry)
        );
    }
    assert_eq!(directory.entry(u64::MAX), None);
    assert_eq!(directory.entry_for_subclass(u64::MAX), None);
    assert!(directory.entries_for_raw_record(u64::MAX).is_empty());
    assert_eq!(
        directory.source_id(),
        directory.semantic_directory().source_id()
    );
    assert_copy::<DxfHatchExtrusion>();
    assert_copy::<DxfHatchExtrusionEntry>();
    assert_send_sync::<DxfHatchExtrusionDirectory>();
    assert!(size_of::<DxfHatchExtrusionEntry>() <= 288);
    Ok(())
}

fn only_extrusion(directory: &DxfHatchExtrusionDirectory) -> Result<DxfHatchExtrusion, io::Error> {
    let [entry] = directory.entries() else {
        return Err(io::Error::other("one extrusion entry"));
    };
    entry
        .extrusion()
        .map_err(|_| io::Error::other("available extrusion"))
}

fn available(entry: DxfHatchExtrusionEntry) -> Result<DxfHatchExtrusion, io::Error> {
    entry
        .extrusion()
        .map_err(|_| io::Error::other("available extrusion"))
}

fn doubles(values: [f64; 3]) -> [DxfDouble; 3] {
    values.map(DxfDouble::from_f64)
}

fn ascii_fixture(version: DxfAcadVersion, fields: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
0\nHATCH\n100\nAcDbHatch\n{fields}0\nENDSEC\n0\nEOF\n",
        version.code()
    )
    .into_bytes()
}

fn binary_fixture(version: DxfAcadVersion, values: Option<[f64; 3]>) -> io::Result<Vec<u8>> {
    let mut bytes = binary_prefix(version)?;
    if let Some([x, y, z]) = values {
        for (code, value) in [(210, x), (220, y), (230, z)] {
            push_double(&mut bytes, version, code, value)?;
        }
    }
    binary_suffix(&mut bytes, version)?;
    Ok(bytes)
}

fn binary_prefix(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
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
    Ok(bytes)
}

fn binary_suffix(bytes: &mut Vec<u8>, version: DxfAcadVersion) -> io::Result<()> {
    push_string(bytes, version, 0, b"ENDSEC")?;
    push_string(bytes, version, 0, b"EOF")
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

use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    sync::{
        RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use seacad_dxf_core::{
    DxfAcadVersion, DxfAsciiRawDocument, DxfByteSource, DxfCancellationToken,
    DxfCanonicalAsciiEnvelopeAction, DxfCanonicalAsciiWriteReceipt, DxfError, DxfIoOperation,
    DxfMemorySource, DxfReadControl, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[test]
fn every_supported_version_has_one_exact_canonical_ascii_framing() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    for version in DxfAcadVersion::SUPPORTED {
        let (input, expected, group_count) = mixed_framing_fixture(version.code());
        let source = DxfMemorySource::new(&input, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source, DxfReadOptions::strict())?;
        let output = directory.path().join(format!("{}.dxf", version.code()));
        let mut progress = Vec::new();
        let mut observer = |event: seacad_dxf_core::DxfReadProgress| {
            progress.push((event.processed_bytes(), event.total_bytes()));
            DxfReadControl::Continue
        };

        let receipt = document.write_canonical_ascii_to_new_file(
            &output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut observer,
        )?;
        assert_eq!(fs::read(&output)?, expected);
        assert_receipt(
            receipt,
            document.source_id(),
            expected.len() as u64,
            group_count,
            DxfCanonicalAsciiEnvelopeAction::PreservedStrict,
        );
        assert_eq!(
            progress.first().copied(),
            Some((0, input.len() as u64 + expected.len() as u64))
        );
        assert_eq!(
            progress.last().copied(),
            Some((
                input.len() as u64 + expected.len() as u64,
                input.len() as u64 + expected.len() as u64
            ))
        );
        assert!(progress.windows(2).all(|pair| pair[0].0 < pair[1].0));

        let output_bytes = fs::read(output)?;
        let output_source = DxfMemorySource::new(&output_bytes, DxfResourceProfile::Safe)?;
        let reparsed = open_ascii(&output_source, DxfReadOptions::strict())?;
        assert_eq!(reparsed.source_id(), receipt.output_id());
    }
    Ok(())
}

#[test]
fn every_accepted_eof_recovery_becomes_one_strict_terminal_group() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let cases = [
        RecoveryCase {
            name: "padded-eof",
            input: b"\xef\xbb\xbf0\n EOF \n",
            expected: b"0\nEOF\n",
            action: DxfCanonicalAsciiEnvelopeAction::CanonicalizedRecovered,
            groups: 1,
        },
        RecoveryCase {
            name: "missing-eof",
            input: b"0\rSECTION\r2\rHEADER\r0\rENDSEC\r",
            expected: b"0\nSECTION\n2\nHEADER\n0\nENDSEC\n0\nEOF\n",
            action: DxfCanonicalAsciiEnvelopeAction::AppendedMissingEof,
            groups: 4,
        },
        RecoveryCase {
            name: "trailing-data",
            input: b"0\nEOF\n999\nafter\n\x1a",
            expected: b"0\nEOF\n",
            action: DxfCanonicalAsciiEnvelopeAction::CanonicalizedRecovered,
            groups: 1,
        },
    ];
    for case in cases {
        let source = DxfMemorySource::new(case.input, DxfResourceProfile::Safe)?;
        let document = open_ascii(&source, DxfReadOptions::compatible())?;
        let output = directory.path().join(format!("{}.dxf", case.name));
        let mut noop = NoopDxfReadObserver;
        let receipt = document.write_canonical_ascii_to_new_file(
            &output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut noop,
        )?;
        assert_eq!(fs::read(&output)?, case.expected);
        assert_receipt(
            receipt,
            document.source_id(),
            case.expected.len() as u64,
            case.groups,
            case.action,
        );
        let output_bytes = fs::read(output)?;
        let output_source = DxfMemorySource::new(&output_bytes, DxfResourceProfile::Safe)?;
        open_ascii(&output_source, DxfReadOptions::strict())?;
    }
    Ok(())
}

struct RecoveryCase {
    name: &'static str,
    input: &'static [u8],
    expected: &'static [u8],
    action: DxfCanonicalAsciiEnvelopeAction,
    groups: u64,
}

#[test]
fn cancellation_and_live_source_mismatch_remove_incomplete_outputs() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let mut large = b"999\n".to_vec();
    large.extend(std::iter::repeat_n(b'x', 140_000));
    large.extend_from_slice(b"\n0\nEOF\n");
    let source = DxfMemorySource::new(&large, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source, DxfReadOptions::strict())?;
    let cancelled_output = directory.path().join("cancelled.dxf");
    let mut observer = |progress: seacad_dxf_core::DxfReadProgress| {
        if progress.processed_bytes() >= 64 * 1024 {
            DxfReadControl::Cancel
        } else {
            DxfReadControl::Continue
        }
    };
    assert!(matches!(
        document.write_canonical_ascii_to_new_file(
            &cancelled_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut observer,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!cancelled_output.exists());

    let original = b"  0\nSECTION\n  2\nHEADER\n  0\nENDSEC\n  0\nEOF\n";
    let mutable = MutableSource::new(original);
    let document = open_ascii(&mutable, DxfReadOptions::strict())?;
    mutable.replace_same_len(b"  0\nSECTIOO\n  2\nHEADER\n  0\nENDSEC\n  0\nEOF\n")?;
    let changed_output = directory.path().join("changed-source.dxf");
    let mut noop = NoopDxfReadObserver;
    assert!(matches!(
        document.write_canonical_ascii_to_new_file(
            &changed_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut noop,
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(!changed_output.exists());
    Ok(())
}

#[test]
fn existing_destination_and_public_receipt_bounds_remain_fail_closed() -> Result<(), Box<dyn Error>>
{
    assert_copy::<DxfCanonicalAsciiEnvelopeAction>();
    assert_copy::<DxfCanonicalAsciiWriteReceipt>();

    let directory = TestDirectory::new()?;
    let output = directory.path().join("existing.dxf");
    fs::write(&output, b"KEEP")?;
    let source = DxfMemorySource::new(b"0\nEOF\n", DxfResourceProfile::Safe)?;
    let document = open_ascii(&source, DxfReadOptions::strict())?;
    let mut noop = NoopDxfReadObserver;
    assert!(matches!(
        document.write_canonical_ascii_to_new_file(
            &output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut noop,
        ),
        Err(DxfError::Io {
            operation: DxfIoOperation::Create,
            kind: io::ErrorKind::AlreadyExists,
            ..
        })
    ));
    assert_eq!(fs::read(output)?, b"KEEP");
    Ok(())
}

fn mixed_framing_fixture(version: &str) -> (Vec<u8>, Vec<u8>, u64) {
    let groups = [
        (0_i16, "SECTION"),
        (2, "HEADER"),
        (9, "$ACADVER"),
        (1, version),
        (9, "$HANDSEED"),
        (5, "10"),
        (0, "ENDSEC"),
        (999, "  keep spaces  "),
        (0, "EOF"),
    ];
    let endings = ["\r\n", "\n", "\r"];
    let mut input = Vec::new();
    let mut expected = Vec::new();
    for (index, (code, value)) in groups.iter().copied().enumerate() {
        let ending = endings[index % endings.len()];
        input.extend_from_slice(format!("  {code}  {ending}{value}{ending}").as_bytes());
        expected.extend_from_slice(format!("{code}\n{value}\n").as_bytes());
    }
    (input, expected, groups.len() as u64)
}

fn assert_receipt(
    receipt: DxfCanonicalAsciiWriteReceipt,
    source_id: seacad_dxf_core::DxfSourceId,
    bytes: u64,
    groups: u64,
    action: DxfCanonicalAsciiEnvelopeAction,
) {
    assert_eq!(receipt.source_id(), source_id);
    assert_eq!(receipt.bytes_written(), bytes);
    assert_eq!(receipt.groups_written(), groups);
    assert_eq!(receipt.envelope_action(), action);
}

fn open_ascii<'a>(
    source: &'a dyn DxfByteSource,
    options: DxfReadOptions,
) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        options,
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

struct MutableSource {
    bytes: RwLock<Vec<u8>>,
}

impl MutableSource {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: RwLock::new(bytes.to_vec()),
        }
    }

    fn replace_same_len(&self, bytes: &[u8]) -> Result<(), io::Error> {
        let mut current = self
            .bytes
            .write()
            .map_err(|_| io::Error::other("mutable source lock poisoned"))?;
        if current.len() != bytes.len() {
            return Err(io::Error::other("replacement length mismatch"));
        }
        current.copy_from_slice(bytes);
        Ok(())
    }
}

impl DxfByteSource for MutableSource {
    fn len(&self) -> u64 {
        self.bytes.read().map_or(0, |bytes| bytes.len() as u64)
    }

    fn read_at(&self, offset: u64, destination: &mut [u8]) -> Result<usize, DxfError> {
        let bytes = self.bytes.read().map_err(|_| {
            DxfError::from_io(
                DxfIoOperation::Read,
                &io::Error::other("mutable source lock poisoned"),
            )
        })?;
        let start = usize::try_from(offset).map_err(|_| DxfError::OffsetOverflow {
            offset,
            requested: destination.len() as u64,
        })?;
        let Some(available) = bytes.get(start..) else {
            return Ok(0);
        };
        let count = available.len().min(destination.len());
        destination[..count].copy_from_slice(&available[..count]);
        Ok(count)
    }
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Result<Self, io::Error> {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "seacad-canonical-ascii-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn assert_copy<T: Copy>() {}

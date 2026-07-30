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
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfBinaryGroupCodeEncoding, DxfBinaryRawDocument,
    DxfByteSource, DxfCancellationToken, DxfCanonicalBinaryEnvelopeAction,
    DxfCanonicalBinaryWriteReceipt, DxfError, DxfIoOperation, DxfMemorySource, DxfReadControl,
    DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[test]
fn every_supported_version_has_one_exact_canonical_binary_framing() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    for version in DxfAcadVersion::SUPPORTED {
        let input = binary_fixture(version, true)?;
        let source = DxfMemorySource::new(&input, DxfResourceProfile::Safe)?;
        let document = open_binary(&source, DxfReadOptions::strict())?;
        let output = directory.path().join(format!("{}.dxf", version.code()));
        let mut progress = Vec::new();
        let mut observer = |event: seacad_dxf_core::DxfReadProgress| {
            progress.push((event.processed_bytes(), event.total_bytes()));
            DxfReadControl::Continue
        };

        let receipt = document.write_canonical_binary_to_new_file(
            &output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut observer,
        )?;
        assert_eq!(fs::read(&output)?, input);
        assert_receipt(
            receipt,
            document.source_id(),
            input.len() as u64,
            12,
            version.binary_group_code_encoding(),
            DxfCanonicalBinaryEnvelopeAction::PreservedStrict,
        );
        assert_eq!(
            progress.first().copied(),
            Some((0, (input.len() * 2) as u64))
        );
        assert_eq!(
            progress.last().copied(),
            Some(((input.len() * 2) as u64, (input.len() * 2) as u64))
        );
        assert!(progress.windows(2).all(|pair| pair[0].0 < pair[1].0));

        let output_bytes = fs::read(output)?;
        let output_source = DxfMemorySource::new(&output_bytes, DxfResourceProfile::Safe)?;
        let reparsed = open_binary(&output_source, DxfReadOptions::strict())?;
        assert_eq!(reparsed.source_id(), receipt.output_id());
        assert_eq!(
            reparsed.group_code_encoding(),
            version.binary_group_code_encoding()
        );
    }
    Ok(())
}

#[test]
fn every_binary_value_family_retains_exact_wire_bytes() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let version = DxfAcadVersion::Ac1032;
    let mut input = binary_fixture_prefix(version)?;
    push_code(&mut input, version, 0)?;
    input.extend_from_slice(b"LINE\0");
    push_code(&mut input, version, 1)?;
    input.extend_from_slice(b"\x80raw-text\0");
    push_code(&mut input, version, 10)?;
    input.extend_from_slice(&(-0.0_f64).to_bits().to_le_bytes());
    push_code(&mut input, version, 70)?;
    input.extend_from_slice(&i16::MIN.to_le_bytes());
    push_code(&mut input, version, 90)?;
    input.extend_from_slice(&i32::MIN.to_le_bytes());
    push_code(&mut input, version, 160)?;
    input.extend_from_slice(&i64::MIN.to_le_bytes());
    push_code(&mut input, version, 290)?;
    input.push(0x7f);
    push_code(&mut input, version, 310)?;
    input.extend_from_slice(b"\x03\x00\xff\x80");
    push_string(&mut input, version, 1000, b"xdata")?;
    binary_fixture_suffix(&mut input, version, true)?;

    let source = DxfMemorySource::new(&input, DxfResourceProfile::Safe)?;
    let document = open_binary(&source, DxfReadOptions::strict())?;
    let output = directory.path().join("all-values.dxf");
    let mut noop = NoopDxfReadObserver;
    let receipt = document.write_canonical_binary_to_new_file(
        &output,
        DxfResourceProfile::Safe,
        &DxfCancellationToken::default(),
        &mut noop,
    )?;
    assert_eq!(fs::read(output)?, input);
    assert_eq!(receipt.groups_written(), 18);
    Ok(())
}

#[test]
fn accepted_binary_eof_recoveries_become_one_strict_terminal_group() -> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let version = DxfAcadVersion::Ac1032;
    let missing = binary_fixture(version, false)?;
    let mut missing_expected = missing.clone();
    push_string(&mut missing_expected, version, 0, b"EOF")?;
    let mut trailing = binary_fixture(version, true)?;
    trailing.extend_from_slice(b"\x1aopaque");
    let strict = binary_fixture(version, true)?;
    let cases = [
        RecoveryCase {
            name: "missing-eof",
            input: missing,
            expected: missing_expected,
            action: DxfCanonicalBinaryEnvelopeAction::AppendedMissingEof,
            groups: 12,
        },
        RecoveryCase {
            name: "trailing-data",
            input: trailing,
            expected: strict,
            action: DxfCanonicalBinaryEnvelopeAction::CanonicalizedRecovered,
            groups: 12,
        },
    ];

    for case in cases {
        let source = DxfMemorySource::new(&case.input, DxfResourceProfile::Safe)?;
        let document = open_binary(&source, DxfReadOptions::compatible())?;
        let output = directory.path().join(format!("{}.dxf", case.name));
        let mut noop = NoopDxfReadObserver;
        let receipt = document.write_canonical_binary_to_new_file(
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
            version.binary_group_code_encoding(),
            case.action,
        );
        let output_bytes = fs::read(output)?;
        let output_source = DxfMemorySource::new(&output_bytes, DxfResourceProfile::Safe)?;
        open_binary(&output_source, DxfReadOptions::strict())?;
    }
    Ok(())
}

struct RecoveryCase {
    name: &'static str,
    input: Vec<u8>,
    expected: Vec<u8>,
    action: DxfCanonicalBinaryEnvelopeAction,
    groups: u64,
}

#[test]
fn cancellation_live_source_and_existing_destination_remain_fail_closed()
-> Result<(), Box<dyn Error>> {
    let directory = TestDirectory::new()?;
    let version = DxfAcadVersion::Ac1032;
    let mut large = binary_fixture_prefix(version)?;
    push_code(&mut large, version, 999)?;
    large.extend(std::iter::repeat_n(b'x', 140_000));
    large.push(0);
    binary_fixture_suffix(&mut large, version, true)?;
    let source = DxfMemorySource::new(&large, DxfResourceProfile::Safe)?;
    let document = open_binary(&source, DxfReadOptions::strict())?;
    let cancelled_output = directory.path().join("cancelled.dxf");
    let mut observer = |progress: seacad_dxf_core::DxfReadProgress| {
        if progress.processed_bytes() >= 64 * 1024 {
            DxfReadControl::Cancel
        } else {
            DxfReadControl::Continue
        }
    };
    assert!(matches!(
        document.write_canonical_binary_to_new_file(
            &cancelled_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut observer,
        ),
        Err(DxfError::Cancelled)
    ));
    assert!(!cancelled_output.exists());

    let original = binary_fixture(version, true)?;
    let mutable = MutableSource::new(&original);
    let document = open_binary(&mutable, DxfReadOptions::strict())?;
    let mut changed = original;
    let sentinel_end = DXF_BINARY_SENTINEL.len();
    changed[sentinel_end + 2] ^= 1;
    mutable.replace_same_len(&changed)?;
    let changed_output = directory.path().join("changed-source.dxf");
    let mut noop = NoopDxfReadObserver;
    assert!(matches!(
        document.write_canonical_binary_to_new_file(
            &changed_output,
            DxfResourceProfile::Safe,
            &DxfCancellationToken::default(),
            &mut noop,
        ),
        Err(DxfError::SourceIdentityMismatch { .. })
    ));
    assert!(!changed_output.exists());

    assert_copy::<DxfCanonicalBinaryEnvelopeAction>();
    assert_copy::<DxfCanonicalBinaryWriteReceipt>();
    assert_send_sync::<DxfCanonicalBinaryWriteReceipt>();
    let existing_output = directory.path().join("existing.dxf");
    fs::write(&existing_output, b"KEEP")?;
    let existing_fixture = binary_fixture(version, true)?;
    let source = DxfMemorySource::new(&existing_fixture, DxfResourceProfile::Safe)?;
    let document = open_binary(&source, DxfReadOptions::strict())?;
    assert!(matches!(
        document.write_canonical_binary_to_new_file(
            &existing_output,
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
    assert_eq!(fs::read(existing_output)?, b"KEEP");
    Ok(())
}

fn binary_fixture(version: DxfAcadVersion, include_eof: bool) -> Result<Vec<u8>, io::Error> {
    let mut bytes = binary_fixture_prefix(version)?;
    push_string(&mut bytes, version, 0, b"POINT")?;
    push_string(&mut bytes, version, 1, b"exact")?;
    push_string(&mut bytes, version, 1000, b"xdata")?;
    binary_fixture_suffix(&mut bytes, version, include_eof)?;
    Ok(bytes)
}

fn binary_fixture_prefix(version: DxfAcadVersion) -> Result<Vec<u8>, io::Error> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER".as_slice()),
        (9, b"$ACADVER".as_slice()),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC".as_slice()),
        (0, b"SECTION".as_slice()),
        (2, b"ENTITIES".as_slice()),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    Ok(bytes)
}

fn binary_fixture_suffix(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    include_eof: bool,
) -> Result<(), io::Error> {
    push_string(bytes, version, 0, b"ENDSEC")?;
    if include_eof {
        push_string(bytes, version, 0, b"EOF")?;
    }
    Ok(())
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> Result<(), io::Error> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> Result<(), io::Error> {
    match version.binary_group_code_encoding() {
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape if (0..=254).contains(&code) => {
            bytes.push(
                u8::try_from(code).map_err(|_| io::Error::other("invalid one-byte group code"))?,
            );
        }
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape
            if (1000..=1071).contains(&code) =>
        {
            bytes.push(u8::MAX);
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        DxfBinaryGroupCodeEncoding::TwoByteLittleEndian => {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        DxfBinaryGroupCodeEncoding::OneByteWithExtendedDataEscape => {
            return Err(io::Error::other("invalid pre-R13 test group code"));
        }
        _ => return Err(io::Error::other("unsupported test group-code encoding")),
    }
    Ok(())
}

fn assert_receipt(
    receipt: DxfCanonicalBinaryWriteReceipt,
    source_id: seacad_dxf_core::DxfSourceId,
    bytes: u64,
    groups: u64,
    encoding: DxfBinaryGroupCodeEncoding,
    action: DxfCanonicalBinaryEnvelopeAction,
) {
    assert_eq!(receipt.source_id(), source_id);
    assert_eq!(receipt.bytes_written(), bytes);
    assert_eq!(receipt.groups_written(), groups);
    assert_eq!(receipt.group_code_encoding(), encoding);
    assert_eq!(receipt.envelope_action(), action);
}

fn open_binary<'a>(
    source: &'a dyn DxfByteSource,
    options: DxfReadOptions,
) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
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
            "seacad-canonical-binary-{}-{id}",
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
fn assert_send_sync<T: Send + Sync>() {}

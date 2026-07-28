//! Redacted, bounded receipts for an offline DXF corpus.

#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, OsString},
    fmt, fs,
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

use seacad_dxf_core::{
    DxfAsciiDocumentConformance, DxfAsciiRawDocument, DxfBinaryDocumentConformance,
    DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfFileSource, DxfPhysicalFormat,
    DxfReadMode, DxfReadOptions, DxfResourceProfile, NoopDxfReadObserver,
    probe_dxf_physical_format,
};
use serde::{Deserialize, Serialize};

const MANIFEST_CONTRACT: &str = "seacad-offline-corpus-manifest/v1";
const RECEIPT_CONTRACT: &str = "seacad-offline-corpus-receipt/v1";
const MAX_MANIFEST_BYTES: u64 = 16 * 1024;
const MAX_DEPTH: u64 = 32;
const MAX_ENTRIES: u64 = 10_000;
const MAX_FILES: u64 = 1_000;
const MAX_TOTAL_BYTES: u64 = 10 * 1024 * 1024 * 1024;

fn main() -> ExitCode {
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    ExitCode::from(run(env::args_os().collect(), &mut stdout, &mut stderr))
}

fn run(args: Vec<OsString>, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.len() == 2 && matches!(args[1].to_str(), Some("-h" | "--help")) {
        return write_help(stdout);
    }
    if args.len() != 3 {
        let _ignored = writeln!(
            stderr,
            "CORPUS-E0001: usage: seacad-corpus-receipt <MANIFEST> <CORPUS_ROOT>"
        );
        return 2;
    }

    let manifest_path = Path::new(&args[1]);
    let corpus_root = Path::new(&args[2]);
    let receipt = match create_receipt(manifest_path, corpus_root) {
        Ok(receipt) => receipt,
        Err(error) => {
            let _ignored = writeln!(stderr, "{}: {}", error.code, error.message);
            return 1;
        }
    };
    if serde_json::to_writer_pretty(&mut *stdout, &receipt).is_err() || writeln!(stdout).is_err() {
        let _ignored = writeln!(stderr, "CORPUS-E0002: unable to write the redacted receipt");
        return 1;
    }
    if receipt.status == "verified" { 0 } else { 1 }
}

fn write_help(writer: &mut dyn Write) -> u8 {
    let result = writeln!(
        writer,
        "Usage: seacad-corpus-receipt <MANIFEST> <CORPUS_ROOT>\n\
         Verifies bounded offline .dxf files and emits aggregate JSON only."
    );
    u8::from(result.is_err())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CorpusLimits {
    max_depth: u64,
    max_entries: u64,
    max_files: u64,
    max_total_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusManifest {
    contract: String,
    schema_version: u32,
    corpus_id: String,
    privacy: String,
    resource_profile: String,
    extensions: Vec<String>,
    limits: CorpusLimits,
}

impl CorpusManifest {
    fn validate(&self) -> Result<(), HarnessError> {
        if self.contract != MANIFEST_CONTRACT
            || self.schema_version != 1
            || self.corpus_id != "seacad-offline-dxf-core"
            || self.privacy != "aggregate-only"
            || self.resource_profile != "safe"
            || self.extensions.as_slice() != ["dxf"]
        {
            return Err(HarnessError::new(
                "CORPUS-E0005",
                "manifest contract or fixed policy fields are invalid",
            ));
        }
        if self.limits.max_depth == 0
            || self.limits.max_entries == 0
            || self.limits.max_files == 0
            || self.limits.max_total_bytes == 0
            || self.limits.max_files > self.limits.max_entries
            || self.limits.max_depth > MAX_DEPTH
            || self.limits.max_entries > MAX_ENTRIES
            || self.limits.max_files > MAX_FILES
            || self.limits.max_total_bytes > MAX_TOTAL_BYTES
        {
            return Err(HarnessError::new(
                "CORPUS-E0006",
                "manifest limits are invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct CorpusReceipt {
    contract: &'static str,
    schema_version: u32,
    status: &'static str,
    manifest_contract: &'static str,
    corpus_id: String,
    platform: PlatformReceipt,
    privacy: PrivacyReceipt,
    limits: CorpusLimits,
    observed: ObservedReceipt,
    failure_codes: BTreeMap<String, u64>,
}

#[derive(Debug, Serialize)]
struct PlatformReceipt {
    operating_system: &'static str,
    architecture: &'static str,
}

#[derive(Debug, Serialize)]
struct PrivacyReceipt {
    paths_included: bool,
    filenames_included: bool,
    source_ids_included: bool,
    per_file_hashes_included: bool,
}

#[derive(Debug, Default, Serialize)]
struct ObservedReceipt {
    entries: u64,
    directories: u64,
    candidate_files: u64,
    ignored_entries: u64,
    total_bytes: u64,
    ascii_files: u64,
    binary_files: u64,
    verified_files: u64,
    invalid_files: u64,
}

fn create_receipt(manifest_path: &Path, corpus_root: &Path) -> Result<CorpusReceipt, HarnessError> {
    let manifest = read_manifest(manifest_path)?;
    let root_metadata = fs::symlink_metadata(corpus_root)
        .map_err(|_| HarnessError::new("CORPUS-E0010", "corpus root metadata is unavailable"))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(HarnessError::new(
            "CORPUS-E0011",
            "corpus root must be a real directory, not a symlink",
        ));
    }
    let canonical_root = fs::canonicalize(corpus_root)
        .map_err(|_| HarnessError::new("CORPUS-E0012", "corpus root is inaccessible"))?;
    let (observed, failure_codes) = scan_corpus(&canonical_root, &manifest.limits)?;
    let status = if observed.invalid_files == 0 && observed.candidate_files > 0 {
        "verified"
    } else {
        "failed"
    };
    Ok(CorpusReceipt {
        contract: RECEIPT_CONTRACT,
        schema_version: 1,
        status,
        manifest_contract: MANIFEST_CONTRACT,
        corpus_id: manifest.corpus_id,
        platform: PlatformReceipt {
            operating_system: env::consts::OS,
            architecture: env::consts::ARCH,
        },
        privacy: PrivacyReceipt {
            paths_included: false,
            filenames_included: false,
            source_ids_included: false,
            per_file_hashes_included: false,
        },
        limits: manifest.limits,
        observed,
        failure_codes,
    })
}

fn read_manifest(path: &Path) -> Result<CorpusManifest, HarnessError> {
    let metadata = fs::metadata(path)
        .map_err(|_| HarnessError::new("CORPUS-E0003", "manifest metadata is unavailable"))?;
    if !metadata.is_file() || metadata.len() > MAX_MANIFEST_BYTES {
        return Err(HarnessError::new(
            "CORPUS-E0003",
            "manifest must be a bounded regular file",
        ));
    }
    let bytes =
        fs::read(path).map_err(|_| HarnessError::new("CORPUS-E0004", "manifest cannot be read"))?;
    let manifest: CorpusManifest = serde_json::from_slice(&bytes)
        .map_err(|_| HarnessError::new("CORPUS-E0005", "manifest JSON is invalid"))?;
    manifest.validate()?;
    Ok(manifest)
}

fn scan_corpus(
    canonical_root: &Path,
    limits: &CorpusLimits,
) -> Result<(ObservedReceipt, BTreeMap<String, u64>), HarnessError> {
    let mut observed = ObservedReceipt::default();
    let mut failure_codes = BTreeMap::new();
    let mut pending = vec![(canonical_root.to_path_buf(), 0_u64)];

    while let Some((directory, directory_depth)) = pending.pop() {
        let read_dir = fs::read_dir(&directory)
            .map_err(|_| HarnessError::new("CORPUS-E0013", "a corpus directory is unreadable"))?;
        let mut entries = Vec::new();
        for entry in read_dir {
            observed.entries = observed.entries.checked_add(1).ok_or_else(|| {
                HarnessError::new("CORPUS-E0020", "corpus entry count overflowed")
            })?;
            if observed.entries > limits.max_entries {
                return Err(HarnessError::new(
                    "CORPUS-E0021",
                    "corpus entry limit exceeded",
                ));
            }
            entries.push(
                entry.map_err(|_| {
                    HarnessError::new("CORPUS-E0014", "a corpus entry is unreadable")
                })?,
            );
        }
        entries.sort_by_key(fs::DirEntry::file_name);

        for entry in entries {
            let entry_depth = directory_depth
                .checked_add(1)
                .ok_or_else(|| HarnessError::new("CORPUS-E0022", "corpus depth overflowed"))?;
            if entry_depth > limits.max_depth {
                return Err(HarnessError::new(
                    "CORPUS-E0023",
                    "corpus depth limit exceeded",
                ));
            }
            let file_type = entry.file_type().map_err(|_| {
                HarnessError::new("CORPUS-E0015", "corpus entry type is unavailable")
            })?;
            if file_type.is_symlink() {
                return Err(HarnessError::new(
                    "CORPUS-E0016",
                    "symlinks are prohibited in the corpus",
                ));
            }
            let path = entry.path();
            let canonical_path = fs::canonicalize(&path)
                .map_err(|_| HarnessError::new("CORPUS-E0017", "a corpus entry is inaccessible"))?;
            if !canonical_path.starts_with(canonical_root) {
                return Err(HarnessError::new(
                    "CORPUS-E0018",
                    "a corpus entry escapes the root",
                ));
            }
            if file_type.is_dir() {
                observed.directories = observed.directories.checked_add(1).ok_or_else(|| {
                    HarnessError::new("CORPUS-E0024", "corpus directory count overflowed")
                })?;
                pending.push((canonical_path, entry_depth));
            } else if file_type.is_file() && is_dxf_path(&path) {
                account_candidate(&canonical_path, limits, &mut observed, &mut failure_codes)?;
            } else {
                observed.ignored_entries =
                    observed.ignored_entries.checked_add(1).ok_or_else(|| {
                        HarnessError::new("CORPUS-E0025", "ignored entry count overflowed")
                    })?;
            }
        }
    }
    Ok((observed, failure_codes))
}

fn is_dxf_path(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dxf"))
}

fn account_candidate(
    path: &Path,
    limits: &CorpusLimits,
    observed: &mut ObservedReceipt,
    failure_codes: &mut BTreeMap<String, u64>,
) -> Result<(), HarnessError> {
    observed.candidate_files = observed
        .candidate_files
        .checked_add(1)
        .ok_or_else(|| HarnessError::new("CORPUS-E0026", "candidate file count overflowed"))?;
    if observed.candidate_files > limits.max_files {
        return Err(HarnessError::new(
            "CORPUS-E0027",
            "corpus file limit exceeded",
        ));
    }
    let bytes = fs::metadata(path)
        .map_err(|_| HarnessError::new("CORPUS-E0019", "file metadata is unavailable"))?
        .len();
    observed.total_bytes = observed
        .total_bytes
        .checked_add(bytes)
        .ok_or_else(|| HarnessError::new("CORPUS-E0028", "corpus byte count overflowed"))?;
    if observed.total_bytes > limits.max_total_bytes {
        return Err(HarnessError::new(
            "CORPUS-E0029",
            "corpus byte limit exceeded",
        ));
    }

    match verify_dxf(path) {
        FileOutcome::Verified(DxfPhysicalFormat::AsciiCandidate) => {
            observed.ascii_files += 1;
            observed.verified_files += 1;
        }
        FileOutcome::Verified(DxfPhysicalFormat::Binary) => {
            observed.binary_files += 1;
            observed.verified_files += 1;
        }
        FileOutcome::Verified(_) => {
            record_failure(observed, failure_codes, "CORPUS-E0102");
        }
        FileOutcome::Invalid(code) => record_failure(observed, failure_codes, &code),
    }
    Ok(())
}

fn record_failure(
    observed: &mut ObservedReceipt,
    failure_codes: &mut BTreeMap<String, u64>,
    code: &str,
) {
    observed.invalid_files += 1;
    let count = failure_codes.entry(code.to_owned()).or_default();
    *count += 1;
}

enum FileOutcome {
    Verified(DxfPhysicalFormat),
    Invalid(String),
}

fn verify_dxf(path: &Path) -> FileOutcome {
    let profile = DxfResourceProfile::Safe;
    let source = match DxfFileSource::open(path, profile) {
        Ok(source) => source,
        Err(error) => return invalid_core(error),
    };
    let physical = match probe_dxf_physical_format(&source, profile) {
        Ok(physical) => physical,
        Err(error) => return invalid_core(error),
    };
    let cancellation = DxfCancellationToken::default();
    let mut observer = NoopDxfReadObserver;
    let options = DxfReadOptions::new(DxfReadMode::Strict, profile);
    match physical {
        DxfPhysicalFormat::AsciiCandidate => {
            match DxfAsciiRawDocument::open(&source, options, &cancellation, &mut observer) {
                Ok(document) if document.conformance() == DxfAsciiDocumentConformance::Strict => {
                    FileOutcome::Verified(physical)
                }
                Ok(_) => FileOutcome::Invalid("CORPUS-E0103".to_owned()),
                Err(error) => invalid_core(error),
            }
        }
        DxfPhysicalFormat::Binary => {
            match DxfBinaryRawDocument::open(&source, options, &cancellation, &mut observer) {
                Ok(document) if document.conformance() == DxfBinaryDocumentConformance::Strict => {
                    FileOutcome::Verified(physical)
                }
                Ok(_) => FileOutcome::Invalid("CORPUS-E0103".to_owned()),
                Err(error) => invalid_core(error),
            }
        }
        DxfPhysicalFormat::Unknown => FileOutcome::Invalid("CORPUS-E0101".to_owned()),
        _ => FileOutcome::Invalid("CORPUS-E0102".to_owned()),
    }
}

fn invalid_core(error: DxfError) -> FileOutcome {
    FileOutcome::Invalid(error.code().as_str().to_owned())
}

#[derive(Debug)]
struct HarnessError {
    code: &'static str,
    message: &'static str,
}

impl HarnessError {
    const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}

impl fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for HarnessError {}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs, io,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use serde_json::Value;

    use super::{create_receipt, run};

    static NEXT_TEMP_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);
    const STRICT_ASCII: &[u8] = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";

    #[test]
    fn receipt_is_aggregate_only_and_counts_both_physical_formats() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let root = directory.create_directory("private-corpus-name")?;
        let nested = directory.create_directory("private-corpus-name/nested-secret")?;
        fs::write(root.join("customer-alpha.dxf"), STRICT_ASCII)?;
        fs::write(nested.join("customer-beta.DXF"), binary_fixture())?;
        fs::write(nested.join("ignored-private.txt"), b"not selected")?;
        let manifest = directory.write_manifest(16, 20, 10, 1024 * 1024)?;

        let receipt = create_receipt(&manifest, &root)?;
        assert_eq!(receipt.status, "verified");
        assert_eq!(receipt.observed.candidate_files, 2);
        assert_eq!(receipt.observed.verified_files, 2);
        assert_eq!(receipt.observed.ascii_files, 1);
        assert_eq!(receipt.observed.binary_files, 1);
        assert_eq!(receipt.observed.invalid_files, 0);
        assert_eq!(receipt.observed.ignored_entries, 1);
        assert!(receipt.failure_codes.is_empty());

        let json = serde_json::to_string(&receipt)?;
        assert_redacted(&json, &directory.path);
        for secret in [
            "private-corpus-name",
            "nested-secret",
            "customer-alpha",
            "customer-beta",
            "ignored-private",
        ] {
            assert!(!json.contains(secret));
        }
        assert!(json.contains("\"paths_included\":false"));
        assert!(json.contains("\"per_file_hashes_included\":false"));
        Ok(())
    }

    #[test]
    fn invalid_files_produce_a_redacted_failed_receipt() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let root = directory.create_directory("hidden-root")?;
        fs::write(root.join("hidden-invalid.dxf"), b"0\n")?;
        let manifest = directory.write_manifest(4, 10, 2, 1024)?;

        let receipt = create_receipt(&manifest, &root)?;
        assert_eq!(receipt.status, "failed");
        assert_eq!(receipt.observed.invalid_files, 1);
        assert_eq!(receipt.failure_codes.values().sum::<u64>(), 1);
        let json = serde_json::to_string(&receipt)?;
        assert_redacted(&json, &directory.path);
        assert!(!json.contains("hidden"));
        Ok(())
    }

    #[test]
    fn empty_corpus_is_not_a_passing_receipt() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let root = directory.create_directory("empty-private-root")?;
        let manifest = directory.write_manifest(4, 10, 2, 1024)?;
        let receipt = create_receipt(&manifest, &root)?;
        assert_eq!(receipt.status, "failed");
        assert_eq!(receipt.observed.candidate_files, 0);
        Ok(())
    }

    #[test]
    fn manifest_and_scan_limits_fail_closed() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let root = directory.create_directory("bounded-root")?;
        fs::write(root.join("one.dxf"), STRICT_ASCII)?;
        fs::write(root.join("two.dxf"), STRICT_ASCII)?;

        let manifest = directory.write_manifest(4, 1, 1, 1024)?;
        let error = create_receipt(&manifest, &root)
            .err()
            .ok_or("expected entry limit error")?;
        assert_eq!(error.code, "CORPUS-E0021");

        let manifest = directory.write_manifest(4, 10, 1, 1024)?;
        let error = create_receipt(&manifest, &root)
            .err()
            .ok_or("expected file limit error")?;
        assert_eq!(error.code, "CORPUS-E0027");

        let manifest = directory.write_manifest(4, 10, 2, 1)?;
        let error = create_receipt(&manifest, &root)
            .err()
            .ok_or("expected byte limit error")?;
        assert_eq!(error.code, "CORPUS-E0029");

        let nested_root = directory.create_directory("depth-root/level-one")?;
        fs::write(nested_root.join("deep.dxf"), STRICT_ASCII)?;
        let depth_root = directory.path.join("depth-root");
        let manifest = directory.write_manifest(1, 10, 2, 1024)?;
        let error = create_receipt(&manifest, &depth_root)
            .err()
            .ok_or("expected depth limit error")?;
        assert_eq!(error.code, "CORPUS-E0023");

        let invalid_manifest = directory.path.join("invalid-manifest.json");
        fs::write(
            &invalid_manifest,
            br#"{"contract":"wrong","schema_version":1}"#,
        )?;
        let error = create_receipt(&invalid_manifest, &root)
            .err()
            .ok_or("expected manifest error")?;
        assert_eq!(error.code, "CORPUS-E0005");
        Ok(())
    }

    #[test]
    fn committed_manifest_matches_the_v1_contract() -> Result<(), Box<dyn Error>> {
        let manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/offline-manifest.json");
        let manifest = super::read_manifest(&manifest_path)?;
        assert_eq!(manifest.contract, "seacad-offline-corpus-manifest/v1");
        assert_eq!(manifest.resource_profile, "safe");
        assert_eq!(manifest.extensions, ["dxf"]);
        assert_eq!(manifest.limits.max_files, 1000);
        assert_eq!(manifest.limits.max_total_bytes, 10 * 1024 * 1024 * 1024);
        Ok(())
    }

    #[test]
    fn command_output_never_echoes_argument_paths() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let root = directory.create_directory("command-secret-root")?;
        fs::write(root.join("command-secret-file.dxf"), STRICT_ASCII)?;
        let manifest = directory.write_manifest(4, 10, 2, 1024)?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = run(
            vec![
                "seacad-corpus-receipt".into(),
                manifest.as_os_str().to_owned(),
                root.as_os_str().to_owned(),
            ],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit, 0);
        assert!(stderr.is_empty());
        let output = String::from_utf8(stdout)?;
        assert_redacted(&output, &directory.path);
        assert!(!output.contains("command-secret"));
        let value: Value = serde_json::from_str(&output)?;
        assert_eq!(value["status"], "verified");
        Ok(())
    }

    fn assert_redacted(output: &str, root: &Path) {
        assert!(!output.contains(&root.to_string_lossy().to_string()));
        assert!(!output.contains('\\'));
    }

    fn binary_fixture() -> Vec<u8> {
        let mut bytes = seacad_dxf_core::DXF_BINARY_SENTINEL.to_vec();
        binary_pair(&mut bytes, 0, b"SECTION\0");
        binary_pair(&mut bytes, 2, b"HEADER\0");
        binary_pair(&mut bytes, 9, b"$ACADVER\0");
        binary_pair(&mut bytes, 1, b"AC1032\0");
        binary_pair(&mut bytes, 0, b"ENDSEC\0");
        binary_pair(&mut bytes, 0, b"EOF\0");
        bytes
    }

    fn binary_pair(bytes: &mut Vec<u8>, code: i16, value: &[u8]) {
        bytes.extend_from_slice(&code.to_le_bytes());
        bytes.extend_from_slice(value);
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> io::Result<Self> {
            for _ in 0..100 {
                let id = NEXT_TEMP_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir()
                    .join(format!("seacad-corpus-receipt-{}-{id}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => return Ok(Self { path }),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error),
                }
            }
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "unable to reserve temporary receipt test directory",
            ))
        }

        fn create_directory(&self, relative: &str) -> io::Result<PathBuf> {
            let path = self.path.join(relative);
            fs::create_dir_all(&path)?;
            Ok(path)
        }

        fn write_manifest(
            &self,
            max_depth: u64,
            max_entries: u64,
            max_files: u64,
            max_total_bytes: u64,
        ) -> io::Result<PathBuf> {
            let path = self.path.join("manifest.json");
            let json = format!(
                concat!(
                    "{{\n",
                    "  \"contract\": \"seacad-offline-corpus-manifest/v1\",\n",
                    "  \"schema_version\": 1,\n",
                    "  \"corpus_id\": \"seacad-offline-dxf-core\",\n",
                    "  \"privacy\": \"aggregate-only\",\n",
                    "  \"resource_profile\": \"safe\",\n",
                    "  \"extensions\": [\"dxf\"],\n",
                    "  \"limits\": {{\n",
                    "    \"max_depth\": {},\n",
                    "    \"max_entries\": {},\n",
                    "    \"max_files\": {},\n",
                    "    \"max_total_bytes\": {}\n",
                    "  }}\n",
                    "}}\n"
                ),
                max_depth, max_entries, max_files, max_total_bytes
            );
            fs::write(&path, json)?;
            Ok(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ignored = fs::remove_dir_all(&self.path);
        }
    }
}

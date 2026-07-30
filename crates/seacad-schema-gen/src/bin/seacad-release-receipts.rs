//! Fail-closed verification and aggregation of the six native artifacts.

#![forbid(unsafe_code)]

use std::{
    collections::BTreeSet,
    env,
    ffi::{OsStr, OsString},
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const INPUT_CONTRACT: &str = "seacad-native-artifact-receipt/v1";
const OUTPUT_CONTRACT: &str = "seacad-six-native-artifact-receipt/v1";
const INPUT_RECEIPT: &str = "RELEASE_RECEIPT.json";
const OUTPUT_RECEIPT: &str = "SIX_NATIVE_RECEIPT.json";
const COPY_BUFFER_BYTES: usize = 64 * 1024;
const MAX_RECEIPT_BYTES: u64 = 1024 * 1024;
const MAX_PACKAGE_ENTRIES: usize = 8_192;
const MAX_PACKAGE_DEPTH: usize = 20;
const MAX_PAYLOAD_FILES: usize = 4_096;
const MAX_PAYLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const SUPPORTED_TARGETS: [&str; 6] = [
    "aarch64-apple-darwin",
    "aarch64-pc-windows-msvc",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

#[derive(Debug)]
struct ReceiptError {
    code: &'static str,
    detail: String,
}

impl ReceiptError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for ReceiptError {}

struct Arguments {
    root: PathBuf,
    output: PathBuf,
    commit_sha: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactReceipt {
    contract: String,
    schema_version: u32,
    package: String,
    version: String,
    target: String,
    commit_sha: String,
    rust_version: String,
    files: Vec<FileReceipt>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileReceipt {
    path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MatrixReceipt {
    contract: String,
    schema_version: u32,
    version: String,
    commit_sha: String,
    rust_version: String,
    payload_files: u64,
    payload_bytes: u64,
    artifacts: Vec<VerifiedArtifact>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VerifiedArtifact {
    target: String,
    package: String,
    receipt_sha256: String,
    payload_files: u64,
    payload_bytes: u64,
}

fn main() -> ExitCode {
    match parse_arguments(env::args_os().collect()).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_arguments(arguments: Vec<OsString>) -> Result<Arguments, ReceiptError> {
    let mut values = arguments.into_iter();
    let _program = values.next();
    let mut root = None;
    let mut output = None;
    let mut commit_sha = None;
    while let Some(flag) = values.next() {
        let value = values.next().ok_or_else(usage_error)?;
        if flag == OsStr::new("--root") && root.is_none() {
            root = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--output") && output.is_none() {
            output = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--commit") && commit_sha.is_none() {
            commit_sha = Some(
                value
                    .into_string()
                    .map_err(|_| ReceiptError::new("RECEIPT_ARGUMENT_UTF8", "commit"))?,
            );
        } else {
            return Err(usage_error());
        }
    }
    let arguments = Arguments {
        root: root.ok_or_else(usage_error)?,
        output: output.ok_or_else(usage_error)?,
        commit_sha: commit_sha.ok_or_else(usage_error)?,
    };
    if !lower_hex(&arguments.commit_sha, 40) {
        return Err(ReceiptError::new(
            "RECEIPT_COMMIT",
            "commit must be 40 lowercase hexadecimal characters",
        ));
    }
    Ok(arguments)
}

fn usage_error() -> ReceiptError {
    ReceiptError::new(
        "RECEIPT_USAGE",
        "expected --root DIRECTORY --output FILE --commit SHA",
    )
}

fn run(arguments: Arguments) -> Result<(), ReceiptError> {
    verify_matrix(&arguments)
}

fn verify_matrix(arguments: &Arguments) -> Result<(), ReceiptError> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    verify_matrix_with_source(arguments, &source_root)
}

fn verify_matrix_with_source(
    arguments: &Arguments,
    source_root: &Path,
) -> Result<(), ReceiptError> {
    let canonical_root = canonical_real_directory(&arguments.root, "RECEIPT_ROOT")?;
    let canonical_source = canonical_real_directory(source_root, "RECEIPT_SOURCE_ROOT")?;
    validate_output(&canonical_root, &arguments.output)?;
    validate_root_entries(&canonical_root)?;

    let mut artifacts = Vec::new();
    artifacts
        .try_reserve(SUPPORTED_TARGETS.len())
        .map_err(|error| ReceiptError::new("RECEIPT_MEMORY", error.to_string()))?;
    let mut payload_files = 0_u64;
    let mut payload_bytes = 0_u64;
    for target in SUPPORTED_TARGETS {
        let package = package_name(target);
        let artifact = verify_artifact(
            &canonical_root.join(&package),
            target,
            &package,
            &arguments.commit_sha,
            &canonical_source,
        )?;
        payload_files = payload_files
            .checked_add(artifact.payload_files)
            .ok_or_else(|| ReceiptError::new("RECEIPT_TOTAL", "file count overflow"))?;
        payload_bytes = payload_bytes
            .checked_add(artifact.payload_bytes)
            .ok_or_else(|| ReceiptError::new("RECEIPT_TOTAL", "byte count overflow"))?;
        artifacts.push(artifact);
    }

    let receipt = MatrixReceipt {
        contract: OUTPUT_CONTRACT.to_owned(),
        schema_version: 1,
        version: env!("CARGO_PKG_VERSION").to_owned(),
        commit_sha: arguments.commit_sha.clone(),
        rust_version: "1.97.1".to_owned(),
        payload_files,
        payload_bytes,
        artifacts,
    };
    let mut encoded = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| ReceiptError::new("RECEIPT_OUTPUT_JSON", error.to_string()))?;
    encoded.push(b'\n');
    write_new(&arguments.output, &encoded)
}

fn canonical_real_directory(path: &Path, code: &'static str) -> Result<PathBuf, ReceiptError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| ReceiptError::new(code, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ReceiptError::new(code, "expected a non-symlink directory"));
    }
    fs::canonicalize(path).map_err(|error| ReceiptError::new(code, error.to_string()))
}

fn validate_output(root: &Path, output: &Path) -> Result<(), ReceiptError> {
    if output.file_name() != Some(OsStr::new(OUTPUT_RECEIPT)) {
        return Err(ReceiptError::new(
            "RECEIPT_OUTPUT_PATH",
            "output filename must be SIX_NATIVE_RECEIPT.json",
        ));
    }
    let parent = output
        .parent()
        .ok_or_else(|| ReceiptError::new("RECEIPT_OUTPUT_PATH", "output has no parent"))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|error| ReceiptError::new("RECEIPT_OUTPUT_PARENT", error.to_string()))?;
    if canonical_parent != root {
        return Err(ReceiptError::new(
            "RECEIPT_OUTPUT_PATH",
            "output must be directly inside the artifact root",
        ));
    }
    match fs::symlink_metadata(output) {
        Ok(_) => {
            return Err(ReceiptError::new(
                "RECEIPT_OUTPUT_EXISTS",
                "output already exists",
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(ReceiptError::new("RECEIPT_OUTPUT_STATE", error.to_string()));
        }
    }
    Ok(())
}

fn validate_root_entries(root: &Path) -> Result<(), ReceiptError> {
    let expected: BTreeSet<String> = SUPPORTED_TARGETS
        .iter()
        .map(|target| package_name(target))
        .collect();
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(root)
        .map_err(|error| ReceiptError::new("RECEIPT_ROOT_READ", error.to_string()))?
    {
        let entry =
            entry.map_err(|error| ReceiptError::new("RECEIPT_ROOT_ENTRY", error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| ReceiptError::new("RECEIPT_ROOT_TYPE", error.to_string()))?;
        if file_type.is_symlink() || !file_type.is_dir() {
            return Err(ReceiptError::new(
                "RECEIPT_ROOT_CONTENT",
                "artifact root must contain only the six package directories",
            ));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| ReceiptError::new("RECEIPT_ROOT_UTF8", "package name is not UTF-8"))?;
        if !actual.insert(name) {
            return Err(ReceiptError::new(
                "RECEIPT_ROOT_DUPLICATE",
                "duplicate package directory",
            ));
        }
    }
    if actual == expected {
        Ok(())
    } else {
        Err(ReceiptError::new(
            "RECEIPT_TARGET_SET",
            "artifact root does not contain the exact six-target set",
        ))
    }
}

fn verify_artifact(
    root: &Path,
    target: &str,
    package: &str,
    commit_sha: &str,
    source_root: &Path,
) -> Result<VerifiedArtifact, ReceiptError> {
    canonical_real_directory(root, "RECEIPT_PACKAGE_ROOT")?;
    let receipt_path = root.join(INPUT_RECEIPT);
    let receipt_bytes = read_bounded(&receipt_path, MAX_RECEIPT_BYTES)?;
    let receipt: ArtifactReceipt = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| ReceiptError::new("RECEIPT_INPUT_JSON", error.to_string()))?;
    validate_receipt_header(&receipt, target, package, commit_sha)?;
    validate_payload_rows(&receipt.files)?;

    let actual_files = walk_payload_files(root)?;
    let expected_files: BTreeSet<String> =
        receipt.files.iter().map(|file| file.path.clone()).collect();
    if actual_files != expected_files {
        return Err(ReceiptError::new(
            "RECEIPT_PAYLOAD_SET",
            "payload files do not exactly match the receipt",
        ));
    }
    require_payload_paths(&expected_files, target)?;
    validate_source_payload(source_root, target, &receipt.files, &expected_files)?;

    let mut payload_bytes = 0_u64;
    for file in &receipt.files {
        let (bytes, sha256) = hash_regular_file(&root.join(&file.path))?;
        if bytes != file.bytes || sha256 != file.sha256 {
            return Err(ReceiptError::new(
                "RECEIPT_PAYLOAD_HASH",
                format!("payload mismatch: {}", file.path),
            ));
        }
        payload_bytes = payload_bytes
            .checked_add(bytes)
            .ok_or_else(|| ReceiptError::new("RECEIPT_PAYLOAD_LIMIT", "byte count overflow"))?;
        if payload_bytes > MAX_PAYLOAD_BYTES {
            return Err(ReceiptError::new(
                "RECEIPT_PAYLOAD_LIMIT",
                "package exceeds the reviewed payload-byte limit",
            ));
        }
    }
    let (_, receipt_sha256) = hash_regular_file(&receipt_path)?;
    Ok(VerifiedArtifact {
        target: target.to_owned(),
        package: package.to_owned(),
        receipt_sha256,
        payload_files: u64::try_from(receipt.files.len())
            .map_err(|error| ReceiptError::new("RECEIPT_TOTAL", error.to_string()))?,
        payload_bytes,
    })
}

fn validate_source_payload(
    source_root: &Path,
    target: &str,
    files: &[FileReceipt],
    received_paths: &BTreeSet<String>,
) -> Result<(), ReceiptError> {
    let executable = if target.contains("windows") {
        "bin/seacad.exe"
    } else {
        "bin/seacad"
    };
    let mut expected_paths = BTreeSet::from([
        executable.to_owned(),
        "README.md".to_owned(),
        "sbom.cdx.json".to_owned(),
    ]);
    let legal_root =
        canonical_real_directory(&source_root.join("release/legal"), "RECEIPT_SOURCE_LEGAL")?;
    for path in walk_payload_files(&legal_root)? {
        expected_paths.insert(format!("legal/{path}"));
    }
    if &expected_paths != received_paths {
        return Err(ReceiptError::new(
            "RECEIPT_SOURCE_SET",
            "artifact payload does not match the committed release-evidence set",
        ));
    }

    for file in files {
        let source = if file.path == executable {
            continue;
        } else if file.path == "README.md" {
            source_root.join("README.md")
        } else if file.path == "sbom.cdx.json" {
            source_root.join("release/sbom.cdx.json")
        } else if let Some(relative) = file.path.strip_prefix("legal/") {
            legal_root.join(relative)
        } else {
            return Err(ReceiptError::new(
                "RECEIPT_SOURCE_PATH",
                format!("unexpected payload source path: {}", file.path),
            ));
        };
        let (source_bytes, source_sha256) = hash_regular_file(&source)?;
        if source_bytes != file.bytes || source_sha256 != file.sha256 {
            return Err(ReceiptError::new(
                "RECEIPT_SOURCE_HASH",
                format!("payload differs from committed evidence: {}", file.path),
            ));
        }
    }
    Ok(())
}

fn validate_receipt_header(
    receipt: &ArtifactReceipt,
    target: &str,
    package: &str,
    commit_sha: &str,
) -> Result<(), ReceiptError> {
    if receipt.contract != INPUT_CONTRACT
        || receipt.schema_version != 1
        || receipt.package != package
        || receipt.version != env!("CARGO_PKG_VERSION")
        || receipt.target != target
        || receipt.commit_sha != commit_sha
        || receipt.rust_version != "1.97.1"
    {
        Err(ReceiptError::new(
            "RECEIPT_HEADER",
            "artifact receipt header does not match the requested matrix",
        ))
    } else {
        Ok(())
    }
}

fn validate_payload_rows(files: &[FileReceipt]) -> Result<(), ReceiptError> {
    if files.is_empty() || files.len() > MAX_PAYLOAD_FILES {
        return Err(ReceiptError::new(
            "RECEIPT_PAYLOAD_LIMIT",
            "payload file count is outside the reviewed bounds",
        ));
    }
    let mut previous = None;
    for file in files {
        if !portable_relative_path(&file.path)
            || file.path == INPUT_RECEIPT
            || !lower_hex(&file.sha256, 64)
        {
            return Err(ReceiptError::new(
                "RECEIPT_PAYLOAD_ROW",
                "payload row has an invalid path or digest",
            ));
        }
        if previous.is_some_and(|value: &str| value >= file.path.as_str()) {
            return Err(ReceiptError::new(
                "RECEIPT_PAYLOAD_ORDER",
                "payload rows must be strictly sorted",
            ));
        }
        previous = Some(file.path.as_str());
    }
    Ok(())
}

fn walk_payload_files(root: &Path) -> Result<BTreeSet<String>, ReceiptError> {
    let mut pending = vec![(root.to_path_buf(), 0_usize)];
    let mut files = BTreeSet::new();
    let mut entries_seen = 0_usize;
    while let Some((directory, depth)) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .map_err(|error| ReceiptError::new("RECEIPT_DIRECTORY", error.to_string()))?
        {
            entries_seen = entries_seen
                .checked_add(1)
                .ok_or_else(|| ReceiptError::new("RECEIPT_ENTRY_LIMIT", "entry count overflow"))?;
            if entries_seen > MAX_PACKAGE_ENTRIES {
                return Err(ReceiptError::new(
                    "RECEIPT_ENTRY_LIMIT",
                    "package exceeds the reviewed entry limit",
                ));
            }
            let entry =
                entry.map_err(|error| ReceiptError::new("RECEIPT_ENTRY", error.to_string()))?;
            let file_type = entry
                .file_type()
                .map_err(|error| ReceiptError::new("RECEIPT_ENTRY_TYPE", error.to_string()))?;
            if file_type.is_symlink() {
                return Err(ReceiptError::new(
                    "RECEIPT_SYMLINK",
                    "package contains a symlink",
                ));
            }
            if file_type.is_dir() {
                let child_depth = depth
                    .checked_add(1)
                    .ok_or_else(|| ReceiptError::new("RECEIPT_DEPTH", "depth overflow"))?;
                if child_depth > MAX_PACKAGE_DEPTH {
                    return Err(ReceiptError::new(
                        "RECEIPT_DEPTH",
                        "package exceeds the reviewed depth limit",
                    ));
                }
                pending.push((entry.path(), child_depth));
            } else if file_type.is_file() {
                let relative = relative_path(root, &entry.path())?;
                if relative != INPUT_RECEIPT && !files.insert(relative) {
                    return Err(ReceiptError::new(
                        "RECEIPT_DUPLICATE_PATH",
                        "duplicate payload path",
                    ));
                }
            } else {
                return Err(ReceiptError::new(
                    "RECEIPT_ENTRY_TYPE",
                    "package contains a non-regular entry",
                ));
            }
        }
    }
    Ok(files)
}

fn require_payload_paths(files: &BTreeSet<String>, target: &str) -> Result<(), ReceiptError> {
    let executable = if target.contains("windows") {
        "bin/seacad.exe"
    } else {
        "bin/seacad"
    };
    for required in [
        executable,
        "README.md",
        "sbom.cdx.json",
        "legal/manifest.json",
    ] {
        if !files.contains(required) {
            return Err(ReceiptError::new(
                "RECEIPT_REQUIRED_PAYLOAD",
                format!("missing required payload: {required}"),
            ));
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, ReceiptError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| ReceiptError::new("RECEIPT_RELATIVE_PATH", error.to_string()))?;
    let mut output = String::new();
    for component in relative.components() {
        let value = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| ReceiptError::new("RECEIPT_PATH_UTF8", "path component is not UTF-8"))?;
        if !output.is_empty() {
            output.push('/');
        }
        output.push_str(value);
    }
    if portable_relative_path(&output) {
        Ok(output)
    } else {
        Err(ReceiptError::new("RECEIPT_RELATIVE_PATH", output))
    }
}

fn portable_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && path.split('/').all(|component| {
            !component.is_empty()
                && component != "."
                && component != ".."
                && component
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, ReceiptError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| ReceiptError::new("RECEIPT_INPUT", error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > limit {
        return Err(ReceiptError::new(
            "RECEIPT_INPUT_TYPE",
            "receipt must be a bounded non-symlink regular file",
        ));
    }
    let input = File::open(path)
        .map_err(|error| ReceiptError::new("RECEIPT_INPUT_OPEN", error.to_string()))?;
    let capacity = usize::try_from(metadata.len())
        .map_err(|error| ReceiptError::new("RECEIPT_INPUT_SIZE", error.to_string()))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve(capacity)
        .map_err(|error| ReceiptError::new("RECEIPT_MEMORY", error.to_string()))?;
    input
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| ReceiptError::new("RECEIPT_INPUT_READ", error.to_string()))?;
    if u64::try_from(bytes.len())
        .map_err(|error| ReceiptError::new("RECEIPT_INPUT_SIZE", error.to_string()))?
        > limit
    {
        Err(ReceiptError::new(
            "RECEIPT_INPUT_SIZE",
            "receipt grew beyond the reviewed limit",
        ))
    } else {
        Ok(bytes)
    }
}

fn hash_regular_file(path: &Path) -> Result<(u64, String), ReceiptError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| ReceiptError::new("RECEIPT_FILE", error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ReceiptError::new(
            "RECEIPT_FILE_TYPE",
            "payload must be a non-symlink regular file",
        ));
    }
    let mut input = File::open(path)
        .map_err(|error| ReceiptError::new("RECEIPT_FILE_OPEN", error.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    let mut bytes = 0_u64;
    loop {
        let read = input
            .read(&mut buffer)
            .map_err(|error| ReceiptError::new("RECEIPT_FILE_READ", error.to_string()))?;
        if read == 0 {
            break;
        }
        let chunk = buffer
            .get(..read)
            .ok_or_else(|| ReceiptError::new("RECEIPT_BUFFER", "invalid read length"))?;
        hasher.update(chunk);
        bytes = bytes
            .checked_add(
                u64::try_from(read)
                    .map_err(|error| ReceiptError::new("RECEIPT_FILE_SIZE", error.to_string()))?,
            )
            .ok_or_else(|| ReceiptError::new("RECEIPT_FILE_SIZE", "byte count overflow"))?;
        if bytes > MAX_PAYLOAD_BYTES {
            return Err(ReceiptError::new(
                "RECEIPT_PAYLOAD_LIMIT",
                "file exceeds the reviewed payload-byte limit",
            ));
        }
    }
    Ok((bytes, finalize_hash(hasher)))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), ReceiptError> {
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| ReceiptError::new("RECEIPT_OUTPUT_CREATE", error.to_string()))?;
    output
        .write_all(bytes)
        .map_err(|error| ReceiptError::new("RECEIPT_OUTPUT_WRITE", error.to_string()))?;
    output
        .flush()
        .map_err(|error| ReceiptError::new("RECEIPT_OUTPUT_FLUSH", error.to_string()))
}

fn package_name(target: &str) -> String {
    format!("seacad-dxf-core-{}-{target}", env!("CARGO_PKG_VERSION"))
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn finalize_hash(hasher: Sha256) -> String {
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs, io,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use sha2::{Digest, Sha256};

    use super::{
        Arguments, ArtifactReceipt, FileReceipt, MatrixReceipt, SUPPORTED_TARGETS, finalize_hash,
        package_name, verify_matrix_with_source,
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    const TEST_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn exact_six_target_set_produces_one_matrix_receipt() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let source = directory.path().join("source");
        let matrix = directory.path().join("matrix");
        build_source(&source)?;
        build_matrix(&matrix, &source)?;
        let output = matrix.join("SIX_NATIVE_RECEIPT.json");
        verify_matrix_with_source(
            &Arguments {
                root: matrix,
                output: output.clone(),
                commit_sha: TEST_COMMIT.to_owned(),
            },
            &source,
        )?;
        let receipt: MatrixReceipt = serde_json::from_slice(&fs::read(output)?)?;
        assert_eq!(receipt.contract, "seacad-six-native-artifact-receipt/v1");
        assert_eq!(receipt.commit_sha, TEST_COMMIT);
        assert_eq!(receipt.artifacts.len(), 6);
        assert_eq!(receipt.payload_files, 24);
        assert!(
            receipt
                .artifacts
                .windows(2)
                .all(|pair| pair[0].target < pair[1].target)
        );
        Ok(())
    }

    #[test]
    fn tampering_and_incomplete_target_sets_fail_closed() -> Result<(), Box<dyn Error>> {
        let tampered = TestDirectory::new()?;
        let tampered_source = tampered.path().join("source");
        let tampered_matrix = tampered.path().join("matrix");
        build_source(&tampered_source)?;
        build_matrix(&tampered_matrix, &tampered_source)?;
        let package = package_name(SUPPORTED_TARGETS[0]);
        fs::write(tampered_matrix.join(package).join("README.md"), b"tampered")?;
        let error = verify_matrix_with_source(
            &Arguments {
                root: tampered_matrix.clone(),
                output: tampered_matrix.join("SIX_NATIVE_RECEIPT.json"),
                commit_sha: TEST_COMMIT.to_owned(),
            },
            &tampered_source,
        )
        .err()
        .ok_or("tampered payload passed")?;
        assert_eq!(error.code, "RECEIPT_PAYLOAD_HASH");

        let incomplete = TestDirectory::new()?;
        let incomplete_source = incomplete.path().join("source");
        let incomplete_matrix = incomplete.path().join("matrix");
        build_source(&incomplete_source)?;
        build_matrix(&incomplete_matrix, &incomplete_source)?;
        fs::remove_dir_all(incomplete_matrix.join(package_name(SUPPORTED_TARGETS[0])))?;
        let error = verify_matrix_with_source(
            &Arguments {
                root: incomplete_matrix.clone(),
                output: incomplete_matrix.join("SIX_NATIVE_RECEIPT.json"),
                commit_sha: TEST_COMMIT.to_owned(),
            },
            &incomplete_source,
        )
        .err()
        .ok_or("incomplete target set passed")?;
        assert_eq!(error.code, "RECEIPT_TARGET_SET");
        Ok(())
    }

    fn build_source(root: &Path) -> Result<(), Box<dyn Error>> {
        fs::create_dir(root)?;
        fs::create_dir(root.join("release"))?;
        fs::create_dir(root.join("release/legal"))?;
        fs::write(root.join("README.md"), b"readme")?;
        fs::write(root.join("release/sbom.cdx.json"), b"{}")?;
        fs::write(root.join("release/legal/manifest.json"), b"{}")?;
        Ok(())
    }

    fn build_matrix(root: &Path, source: &Path) -> Result<(), Box<dyn Error>> {
        fs::create_dir(root)?;
        for target in SUPPORTED_TARGETS {
            let package = package_name(target);
            let directory = root.join(&package);
            fs::create_dir(&directory)?;
            fs::create_dir(directory.join("bin"))?;
            fs::create_dir(directory.join("legal"))?;
            let executable = if target.contains("windows") {
                "bin/seacad.exe"
            } else {
                "bin/seacad"
            };
            let payloads = [
                (executable, b"binary".to_vec()),
                ("README.md", fs::read(source.join("README.md"))?),
                (
                    "sbom.cdx.json",
                    fs::read(source.join("release/sbom.cdx.json"))?,
                ),
                (
                    "legal/manifest.json",
                    fs::read(source.join("release/legal/manifest.json"))?,
                ),
            ];
            let mut files = Vec::new();
            for (path, bytes) in payloads {
                fs::write(directory.join(path), &bytes)?;
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                files.push(FileReceipt {
                    path: path.to_owned(),
                    bytes: u64::try_from(bytes.len())?,
                    sha256: finalize_hash(hasher),
                });
            }
            files.sort_by(|left, right| left.path.cmp(&right.path));
            let receipt = ArtifactReceipt {
                contract: "seacad-native-artifact-receipt/v1".to_owned(),
                schema_version: 1,
                package,
                version: env!("CARGO_PKG_VERSION").to_owned(),
                target: target.to_owned(),
                commit_sha: TEST_COMMIT.to_owned(),
                rust_version: "1.97.1".to_owned(),
                files,
            };
            let mut encoded = serde_json::to_vec_pretty(&receipt)?;
            encoded.push(b'\n');
            fs::write(directory.join("RELEASE_RECEIPT.json"), encoded)?;
        }
        Ok(())
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> io::Result<Self> {
            for _ in 0..100 {
                let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "seacad-release-receipts-{}-{id}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Ok(Self { path }),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error),
                }
            }
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "unable to reserve receipt test directory",
            ))
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
}

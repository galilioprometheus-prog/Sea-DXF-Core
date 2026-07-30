//! Fail-closed native artifact directory assembly with per-file receipts.

#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
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

const CONTRACT: &str = "seacad-native-artifact-receipt/v1";
const RECEIPT_NAME: &str = "RELEASE_RECEIPT.json";
const COPY_BUFFER_BYTES: usize = 64 * 1024;
const MAX_LEGAL_ENTRIES: usize = 4_096;
const MAX_LEGAL_DEPTH: usize = 16;
const SUPPORTED_TARGETS: [&str; 6] = [
    "aarch64-apple-darwin",
    "aarch64-pc-windows-msvc",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

#[derive(Debug)]
struct PackageError {
    code: &'static str,
    detail: String,
}

impl PackageError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for PackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for PackageError {}

struct Arguments {
    target: String,
    binary: PathBuf,
    output: PathBuf,
    commit_sha: String,
}

#[derive(Debug, Deserialize, Serialize)]
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
struct FileReceipt {
    path: String,
    bytes: u64,
    sha256: String,
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

fn parse_arguments(arguments: Vec<OsString>) -> Result<Arguments, PackageError> {
    let mut values = arguments.into_iter();
    let _program = values.next();
    let mut target = None;
    let mut binary = None;
    let mut output = None;
    let mut commit_sha = None;
    while let Some(flag) = values.next() {
        let value = values.next().ok_or_else(usage_error)?;
        if flag == OsStr::new("--target") && target.is_none() {
            target = Some(required_utf8(value, "target")?);
        } else if flag == OsStr::new("--binary") && binary.is_none() {
            binary = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--output") && output.is_none() {
            output = Some(PathBuf::from(value));
        } else if flag == OsStr::new("--commit") && commit_sha.is_none() {
            commit_sha = Some(required_utf8(value, "commit")?);
        } else {
            return Err(usage_error());
        }
    }
    let arguments = Arguments {
        target: target.ok_or_else(usage_error)?,
        binary: binary.ok_or_else(usage_error)?,
        output: output.ok_or_else(usage_error)?,
        commit_sha: commit_sha.ok_or_else(usage_error)?,
    };
    validate_arguments(&arguments)?;
    Ok(arguments)
}

fn required_utf8(value: OsString, field: &'static str) -> Result<String, PackageError> {
    value
        .into_string()
        .map_err(|_| PackageError::new("PACKAGE_ARGUMENT_UTF8", field))
}

fn usage_error() -> PackageError {
    PackageError::new(
        "PACKAGE_USAGE",
        "expected --target TARGET --binary PATH --output PATH --commit SHA",
    )
}

fn validate_arguments(arguments: &Arguments) -> Result<(), PackageError> {
    if SUPPORTED_TARGETS
        .binary_search(&arguments.target.as_str())
        .is_err()
    {
        return Err(PackageError::new(
            "PACKAGE_TARGET",
            "target is not in the reviewed six-native matrix",
        ));
    }
    if arguments.commit_sha.len() != 40
        || !arguments
            .commit_sha
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PackageError::new(
            "PACKAGE_COMMIT",
            "commit must be 40 lowercase hexadecimal characters",
        ));
    }
    Ok(())
}

fn run(arguments: Arguments) -> Result<(), PackageError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    package(&root, &arguments)
}

fn package(root: &Path, arguments: &Arguments) -> Result<(), PackageError> {
    validate_source_file(&arguments.binary, true)?;
    validate_output_location(&root.join("release/legal"), &arguments.output)?;
    fs::create_dir(&arguments.output)
        .map_err(|error| PackageError::new("PACKAGE_OUTPUT_CREATE", error.to_string()))?;
    let result = assemble_package(root, arguments);
    if let Err(error) = result {
        return match fs::remove_dir_all(&arguments.output) {
            Ok(()) => Err(error),
            Err(cleanup) => Err(PackageError::new(
                "PACKAGE_OUTPUT_CLEANUP",
                cleanup.to_string(),
            )),
        };
    }
    Ok(())
}

fn validate_output_location(legal_root: &Path, output: &Path) -> Result<(), PackageError> {
    let parent = output
        .parent()
        .ok_or_else(|| PackageError::new("PACKAGE_OUTPUT_PATH", "output has no parent"))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let name = output
        .file_name()
        .ok_or_else(|| PackageError::new("PACKAGE_OUTPUT_PATH", "output has no final component"))?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|error| PackageError::new("PACKAGE_OUTPUT_PARENT", error.to_string()))?;
    let canonical_legal = fs::canonicalize(legal_root)
        .map_err(|error| PackageError::new("PACKAGE_LEGAL_ROOT", error.to_string()))?;
    if canonical_parent.join(name).starts_with(canonical_legal) {
        Err(PackageError::new(
            "PACKAGE_OUTPUT_OVERLAP",
            "output must not be inside the legal source tree",
        ))
    } else {
        Ok(())
    }
}

fn assemble_package(root: &Path, arguments: &Arguments) -> Result<(), PackageError> {
    let mut sources = BTreeMap::new();
    let executable = if arguments.target.contains("windows") {
        "bin/seacad.exe"
    } else {
        "bin/seacad"
    };
    insert_source(&mut sources, executable, arguments.binary.clone())?;
    insert_source(&mut sources, "README.md", root.join("README.md"))?;
    insert_source(
        &mut sources,
        "sbom.cdx.json",
        root.join("release/sbom.cdx.json"),
    )?;
    for (relative, source) in legal_sources(&root.join("release/legal"))? {
        insert_source(&mut sources, &format!("legal/{relative}"), source)?;
    }

    let mut files = Vec::new();
    files
        .try_reserve(sources.len())
        .map_err(|error| PackageError::new("PACKAGE_MEMORY", error.to_string()))?;
    for (relative, source) in sources {
        validate_source_file(&source, false)?;
        let destination = arguments.output.join(&relative);
        files.push(copy_and_hash(&source, &destination, relative)?);
    }
    let receipt = ArtifactReceipt {
        contract: CONTRACT.to_owned(),
        schema_version: 1,
        package: format!(
            "seacad-dxf-core-{}-{}",
            env!("CARGO_PKG_VERSION"),
            arguments.target
        ),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        target: arguments.target.clone(),
        commit_sha: arguments.commit_sha.clone(),
        rust_version: "1.97.1".to_owned(),
        files,
    };
    let mut encoded = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| PackageError::new("PACKAGE_RECEIPT_JSON", error.to_string()))?;
    encoded.push(b'\n');
    write_new_file(&arguments.output.join(RECEIPT_NAME), &encoded)
}

fn insert_source(
    sources: &mut BTreeMap<String, PathBuf>,
    relative: &str,
    source: PathBuf,
) -> Result<(), PackageError> {
    if !portable_relative_path(relative) {
        return Err(PackageError::new("PACKAGE_RELATIVE_PATH", relative));
    }
    if sources.insert(relative.to_owned(), source).is_some() {
        Err(PackageError::new("PACKAGE_DUPLICATE_PATH", relative))
    } else {
        Ok(())
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

fn legal_sources(root: &Path) -> Result<Vec<(String, PathBuf)>, PackageError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| PackageError::new("PACKAGE_LEGAL_ROOT", error.to_string()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(PackageError::new(
            "PACKAGE_LEGAL_ROOT_TYPE",
            "legal root must be a real directory",
        ));
    }
    let mut pending = vec![(root.to_path_buf(), 0_usize)];
    let mut files = Vec::new();
    let mut entries_seen = 0_usize;
    while let Some((directory, depth)) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| PackageError::new("PACKAGE_LEGAL_DIRECTORY", error.to_string()))?;
        for entry in entries {
            entries_seen = entries_seen
                .checked_add(1)
                .ok_or_else(|| PackageError::new("PACKAGE_LEGAL_LIMIT", "entry count overflow"))?;
            if entries_seen > MAX_LEGAL_ENTRIES {
                return Err(PackageError::new(
                    "PACKAGE_LEGAL_LIMIT",
                    "legal bundle exceeds the reviewed entry limit",
                ));
            }
            let entry = entry
                .map_err(|error| PackageError::new("PACKAGE_LEGAL_ENTRY", error.to_string()))?;
            let file_type = entry
                .file_type()
                .map_err(|error| PackageError::new("PACKAGE_LEGAL_TYPE", error.to_string()))?;
            if file_type.is_symlink() {
                return Err(PackageError::new(
                    "PACKAGE_LEGAL_SYMLINK",
                    "legal bundle contains a symlink",
                ));
            }
            if file_type.is_dir() {
                let child_depth = depth
                    .checked_add(1)
                    .ok_or_else(|| PackageError::new("PACKAGE_LEGAL_LIMIT", "depth overflow"))?;
                if child_depth > MAX_LEGAL_DEPTH {
                    return Err(PackageError::new(
                        "PACKAGE_LEGAL_LIMIT",
                        "legal bundle exceeds the reviewed depth limit",
                    ));
                }
                pending.push((entry.path(), child_depth));
            } else if file_type.is_file() {
                let path = entry.path();
                let relative = path
                    .strip_prefix(root)
                    .map_err(|error| PackageError::new("PACKAGE_LEGAL_PATH", error.to_string()))?;
                let relative = relative_path(relative)?;
                files.push((relative, path));
            } else {
                return Err(PackageError::new(
                    "PACKAGE_LEGAL_ENTRY_TYPE",
                    "legal bundle contains a non-regular entry",
                ));
            }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

fn relative_path(path: &Path) -> Result<String, PackageError> {
    let mut output = String::new();
    for component in path.components() {
        let value = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| PackageError::new("PACKAGE_PATH_UTF8", "path component is not UTF-8"))?;
        if !output.is_empty() {
            output.push('/');
        }
        output.push_str(value);
    }
    if portable_relative_path(&output) {
        Ok(output)
    } else {
        Err(PackageError::new("PACKAGE_RELATIVE_PATH", output))
    }
}

fn validate_source_file(path: &Path, require_nonempty: bool) -> Result<(), PackageError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| PackageError::new("PACKAGE_SOURCE", error.to_string()))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || (require_nonempty && metadata.len() == 0)
    {
        return Err(PackageError::new(
            "PACKAGE_SOURCE_TYPE",
            "source must be a non-symlink regular file",
        ));
    }
    Ok(())
}

fn copy_and_hash(
    source: &Path,
    destination: &Path,
    relative: String,
) -> Result<FileReceipt, PackageError> {
    let parent = destination
        .parent()
        .ok_or_else(|| PackageError::new("PACKAGE_DESTINATION", relative.clone()))?;
    fs::create_dir_all(parent)
        .map_err(|error| PackageError::new("PACKAGE_DESTINATION_DIR", error.to_string()))?;
    let mut input = File::open(source)
        .map_err(|error| PackageError::new("PACKAGE_SOURCE_OPEN", error.to_string()))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| PackageError::new("PACKAGE_DESTINATION_CREATE", error.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    let mut bytes = 0_u64;
    loop {
        let read = input
            .read(&mut buffer)
            .map_err(|error| PackageError::new("PACKAGE_SOURCE_READ", error.to_string()))?;
        if read == 0 {
            break;
        }
        let chunk = buffer
            .get(..read)
            .ok_or_else(|| PackageError::new("PACKAGE_BUFFER", relative.clone()))?;
        output
            .write_all(chunk)
            .map_err(|error| PackageError::new("PACKAGE_DESTINATION_WRITE", error.to_string()))?;
        hasher.update(chunk);
        bytes = bytes
            .checked_add(
                u64::try_from(read)
                    .map_err(|error| PackageError::new("PACKAGE_SIZE", error.to_string()))?,
            )
            .ok_or_else(|| PackageError::new("PACKAGE_SIZE", "file size overflow"))?;
    }
    output
        .flush()
        .map_err(|error| PackageError::new("PACKAGE_DESTINATION_FLUSH", error.to_string()))?;
    Ok(FileReceipt {
        path: relative,
        bytes,
        sha256: finalize_hash(hasher),
    })
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), PackageError> {
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| PackageError::new("PACKAGE_RECEIPT_CREATE", error.to_string()))?;
    output
        .write_all(bytes)
        .map_err(|error| PackageError::new("PACKAGE_RECEIPT_WRITE", error.to_string()))?;
    output
        .flush()
        .map_err(|error| PackageError::new("PACKAGE_RECEIPT_FLUSH", error.to_string()))
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
        Arguments, ArtifactReceipt, SUPPORTED_TARGETS, finalize_hash, package, parse_arguments,
        validate_output_location,
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn every_native_target_has_a_complete_hash_receipt() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let binary = directory.path().join("input-binary");
        fs::write(&binary, b"native-binary")?;
        for target in SUPPORTED_TARGETS {
            let output = directory.path().join(target);
            package(
                &root,
                &Arguments {
                    target: target.to_owned(),
                    binary: binary.clone(),
                    output: output.clone(),
                    commit_sha: COMMIT.to_owned(),
                },
            )?;
            let receipt: ArtifactReceipt =
                serde_json::from_slice(&fs::read(output.join("RELEASE_RECEIPT.json"))?)?;
            assert_eq!(receipt.contract, "seacad-native-artifact-receipt/v1");
            assert_eq!(receipt.target, target);
            assert_eq!(receipt.commit_sha, COMMIT);
            assert_eq!(receipt.files.len(), 61);
            assert!(
                receipt
                    .files
                    .windows(2)
                    .all(|pair| pair[0].path < pair[1].path)
            );
            for file in receipt.files {
                let bytes = fs::read(output.join(&file.path))?;
                assert_eq!(file.bytes, bytes.len() as u64);
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                assert_eq!(file.sha256, finalize_hash(hasher));
            }
            let executable = if target.contains("windows") {
                "bin/seacad.exe"
            } else {
                "bin/seacad"
            };
            assert_eq!(fs::read(output.join(executable))?, b"native-binary");
        }
        Ok(())
    }

    #[test]
    fn invalid_arguments_and_existing_output_fail_closed() -> Result<(), Box<dyn Error>> {
        let invalid = parse_arguments(vec![
            "packager".into(),
            "--target".into(),
            "unsupported".into(),
            "--binary".into(),
            "binary".into(),
            "--output".into(),
            "output".into(),
            "--commit".into(),
            COMMIT.into(),
        ])
        .err()
        .ok_or("unsupported target passed")?;
        assert_eq!(invalid.code, "PACKAGE_TARGET");

        let directory = TestDirectory::new()?;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let overlap = validate_output_location(
            &root.join("release/legal"),
            &root.join("release/legal/packager-test-output-must-not-exist"),
        )
        .err()
        .ok_or("overlapping output passed")?;
        assert_eq!(overlap.code, "PACKAGE_OUTPUT_OVERLAP");
        let binary = directory.path().join("binary");
        fs::write(&binary, b"binary")?;
        let output = directory.path().join("existing");
        fs::create_dir(&output)?;
        fs::write(output.join("marker"), b"preserve")?;
        let error = package(
            &root,
            &Arguments {
                target: "x86_64-pc-windows-msvc".to_owned(),
                binary,
                output: output.clone(),
                commit_sha: COMMIT.to_owned(),
            },
        )
        .err()
        .ok_or("existing output passed")?;
        assert_eq!(error.code, "PACKAGE_OUTPUT_CREATE");
        assert_eq!(fs::read(output.join("marker"))?, b"preserve");
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
                    "seacad-release-packager-{}-{id}",
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
                "unable to reserve packager test directory",
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

//! Deterministic CycloneDX inventory from the complete locked Cargo graph.

#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsStr,
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const OUTPUT_PATH: &str = "release/sbom.cdx.json";
const LEGAL_ROOT: &str = "release/legal";
const LEGAL_MANIFEST_PATH: &str = "release/legal/manifest.json";
const NOTICES_PATH: &str = "THIRD_PARTY_NOTICES.md";
const LOCK_PATH: &str = "Cargo.lock";
const WORKSPACE_BOM_REF: &str = "urn:seacad:workspace";
const REVIEWED_PLATFORMS: [&str; 6] = [
    "aarch64-apple-darwin",
    "aarch64-pc-windows-msvc",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

#[derive(Clone, Copy)]
enum Mode {
    Check,
    Write,
}

#[derive(Debug)]
struct EvidenceError {
    code: &'static str,
    detail: String,
}

impl EvidenceError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for EvidenceError {}

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_members: Vec<String>,
    resolve: CargoResolve,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    id: String,
    license: Option<String>,
    license_file: Option<String>,
    source: Option<String>,
    manifest_path: PathBuf,
}

#[derive(Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoNode>,
}

#[derive(Deserialize)]
struct CargoNode {
    id: String,
    dependencies: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CycloneDxBom {
    bom_format: &'static str,
    spec_version: &'static str,
    version: u32,
    metadata: BomMetadata,
    components: Vec<BomComponent>,
    dependencies: Vec<BomDependency>,
}

#[derive(Serialize)]
struct BomMetadata {
    component: BomComponent,
    properties: Vec<BomProperty>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BomComponent {
    #[serde(rename = "type")]
    component_type: &'static str,
    #[serde(rename = "bom-ref")]
    bom_ref: String,
    name: String,
    version: String,
    licenses: Vec<BomLicenseChoice>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hashes: Vec<BomHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    purl: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    properties: Vec<BomProperty>,
}

#[derive(Serialize)]
struct BomLicenseChoice {
    expression: String,
}

#[derive(Serialize)]
struct BomHash {
    alg: &'static str,
    content: String,
}

#[derive(Serialize)]
struct BomProperty {
    name: String,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BomDependency {
    #[serde(rename = "ref")]
    bom_ref: String,
    depends_on: Vec<String>,
}

#[derive(Eq, PartialEq)]
struct LegalBundle {
    files: BTreeMap<String, Vec<u8>>,
    manifest: Vec<u8>,
}

#[derive(Serialize)]
struct LegalManifest {
    contract: &'static str,
    cargo_lock_sha256: String,
    root_files: Vec<LegalFileReceipt>,
    packages: Vec<LegalPackageReceipt>,
}

#[derive(Serialize)]
struct LegalPackageReceipt {
    name: String,
    version: String,
    license_expression: String,
    crate_sha256: String,
    files: Vec<LegalFileReceipt>,
}

#[derive(Serialize)]
struct LegalFileReceipt {
    path: String,
    bytes: u64,
    sha256: String,
}

fn main() -> ExitCode {
    match parse_mode().and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_mode() -> Result<Mode, EvidenceError> {
    let mut arguments = env::args_os().skip(1);
    let mode = match arguments.next().as_deref() {
        Some(value) if value == OsStr::new("--check") => Mode::Check,
        Some(value) if value == OsStr::new("--write") => Mode::Write,
        _ => return Err(usage_error()),
    };
    if arguments.next().is_some() {
        return Err(usage_error());
    }
    Ok(mode)
}

fn usage_error() -> EvidenceError {
    EvidenceError::new(
        "RELEASE_USAGE",
        "expected exactly one of --check or --write",
    )
}

fn run(mode: Mode) -> Result<(), EvidenceError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let metadata = cargo_metadata(&root)?;
    validate_notices(&root, &metadata)?;
    let checksums = locked_checksums(&root.join(LOCK_PATH))?;
    let lock_hash = hash_file(&root.join(LOCK_PATH))?;
    let output = render_bom(&metadata, &checksums, lock_hash.clone())?;
    let legal = build_legal_bundle(&root, &metadata, &checksums, lock_hash)?;
    let target = root.join(OUTPUT_PATH);
    match mode {
        Mode::Check => {
            check_output(&target, &output)?;
            check_legal_bundle(&root, &legal)
        }
        Mode::Write => {
            write_output(&target, &output)?;
            write_legal_bundle(&root, &legal)
        }
    }
}

fn cargo_metadata(root: &Path) -> Result<CargoMetadata, EvidenceError> {
    let mut packages = BTreeMap::new();
    let mut workspace_members: Option<Vec<String>> = None;
    let mut dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for platform in REVIEWED_PLATFORMS {
        let metadata = cargo_metadata_for_platform(root, platform)?;
        let current_workspace: BTreeSet<&str> = metadata
            .workspace_members
            .iter()
            .map(String::as_str)
            .collect();
        if let Some(baseline) = &workspace_members {
            let baseline_workspace: BTreeSet<&str> = baseline.iter().map(String::as_str).collect();
            if current_workspace != baseline_workspace {
                return Err(EvidenceError::new(
                    "RELEASE_METADATA_WORKSPACE_SET",
                    platform,
                ));
            }
        } else {
            workspace_members = Some(metadata.workspace_members);
        }
        for package in metadata.packages {
            packages.entry(package.id.clone()).or_insert(package);
        }
        for node in metadata.resolve.nodes {
            dependencies
                .entry(node.id)
                .or_default()
                .extend(node.dependencies);
        }
    }
    Ok(CargoMetadata {
        packages: packages.into_values().collect(),
        workspace_members: workspace_members
            .ok_or_else(|| EvidenceError::new("RELEASE_METADATA_EMPTY", "no platforms"))?,
        resolve: CargoResolve {
            nodes: dependencies
                .into_iter()
                .map(|(id, dependencies)| CargoNode {
                    id,
                    dependencies: dependencies.into_iter().collect(),
                })
                .collect(),
        },
    })
}

fn cargo_metadata_for_platform(
    root: &Path,
    platform: &str,
) -> Result<CargoMetadata, EvidenceError> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--filter-platform",
            platform,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| EvidenceError::new("RELEASE_METADATA_IO", error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(EvidenceError::new(
            "RELEASE_METADATA",
            format!("{platform}: {}", stderr.trim()),
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| EvidenceError::new("RELEASE_METADATA_JSON", error.to_string()))
}

fn validate_notices(root: &Path, metadata: &CargoMetadata) -> Result<(), EvidenceError> {
    let notices = fs::read_to_string(root.join(NOTICES_PATH))
        .map_err(|error| EvidenceError::new("RELEASE_NOTICES_IO", error.to_string()))?;
    let workspace: BTreeSet<&str> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
    for package in &metadata.packages {
        if workspace.contains(package.id.as_str()) {
            continue;
        }
        let row = format!("| {} | {} |", package.name, package.version);
        if !notices.contains(&row) {
            return Err(EvidenceError::new(
                "RELEASE_NOTICE_MISSING",
                format!("{} {}", package.name, package.version),
            ));
        }
        if package.license.as_deref().is_none_or(str::is_empty) {
            return Err(EvidenceError::new(
                "RELEASE_LICENSE_MISSING",
                format!("{} {}", package.name, package.version),
            ));
        }
    }
    Ok(())
}

fn render_bom(
    metadata: &CargoMetadata,
    checksums: &BTreeMap<(String, String), String>,
    lock_hash: String,
) -> Result<Vec<u8>, EvidenceError> {
    let workspace: BTreeSet<&str> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
    let mut references = BTreeMap::new();
    for package in &metadata.packages {
        let workspace_package = workspace.contains(package.id.as_str());
        let reference = package_reference(package, workspace_package);
        if references.insert(package.id.as_str(), reference).is_some() {
            return Err(EvidenceError::new(
                "RELEASE_DUPLICATE_ID",
                package.id.clone(),
            ));
        }
    }

    let mut components = Vec::new();
    components
        .try_reserve(metadata.packages.len())
        .map_err(|error| EvidenceError::new("RELEASE_MEMORY", error.to_string()))?;
    for package in &metadata.packages {
        let workspace_package = workspace.contains(package.id.as_str());
        let license = if workspace_package {
            if package.license_file.as_deref().is_none_or(str::is_empty) {
                return Err(EvidenceError::new(
                    "RELEASE_WORKSPACE_LICENSE_MISSING",
                    package.name.clone(),
                ));
            }
            "LicenseRef-SeaCad-Proprietary".to_owned()
        } else {
            package.license.clone().ok_or_else(|| {
                EvidenceError::new("RELEASE_LICENSE_MISSING", package.name.clone())
            })?
        };
        let source_kind =
            if workspace_package {
                "workspace"
            } else if package.source.as_deref().is_some_and(|source| {
                source == "registry+https://github.com/rust-lang/crates.io-index"
            }) {
                "crates.io"
            } else {
                return Err(EvidenceError::new(
                    "RELEASE_SOURCE",
                    format!("{} has an unapproved source", package.name),
                ));
            };
        let hashes = if workspace_package {
            Vec::new()
        } else {
            let checksum = checksums
                .get(&(package.name.clone(), package.version.clone()))
                .ok_or_else(|| {
                    EvidenceError::new(
                        "RELEASE_CHECKSUM_MISSING",
                        format!("{} {}", package.name, package.version),
                    )
                })?;
            vec![BomHash {
                alg: "SHA-256",
                content: checksum.clone(),
            }]
        };
        components.push(BomComponent {
            component_type: if package.name == "seacad-dxf-core" {
                "library"
            } else if workspace_package {
                "application"
            } else {
                "library"
            },
            bom_ref: references
                .get(package.id.as_str())
                .cloned()
                .ok_or_else(|| EvidenceError::new("RELEASE_REFERENCE", package.id.clone()))?,
            name: package.name.clone(),
            version: package.version.clone(),
            licenses: vec![BomLicenseChoice {
                expression: license,
            }],
            hashes,
            purl: (!workspace_package)
                .then(|| format!("pkg:cargo/{}@{}", package.name, package.version)),
            properties: vec![BomProperty {
                name: "seacad:source".to_owned(),
                value: source_kind.to_owned(),
            }],
        });
    }
    components.sort_by(|left, right| left.bom_ref.cmp(&right.bom_ref));

    let mut dependencies = Vec::new();
    dependencies
        .try_reserve(metadata.resolve.nodes.len() + 1)
        .map_err(|error| EvidenceError::new("RELEASE_MEMORY", error.to_string()))?;
    let mut workspace_dependencies: Vec<String> = metadata
        .workspace_members
        .iter()
        .map(|id| {
            references
                .get(id.as_str())
                .cloned()
                .ok_or_else(|| EvidenceError::new("RELEASE_WORKSPACE_REFERENCE", id.clone()))
        })
        .collect::<Result<_, _>>()?;
    workspace_dependencies.sort();
    dependencies.push(BomDependency {
        bom_ref: WORKSPACE_BOM_REF.to_owned(),
        depends_on: workspace_dependencies,
    });
    for node in &metadata.resolve.nodes {
        let mut depends_on: Vec<String> =
            node.dependencies
                .iter()
                .map(|id| {
                    references.get(id.as_str()).cloned().ok_or_else(|| {
                        EvidenceError::new("RELEASE_DEPENDENCY_REFERENCE", id.clone())
                    })
                })
                .collect::<Result<_, _>>()?;
        depends_on.sort();
        depends_on.dedup();
        dependencies.push(BomDependency {
            bom_ref: references
                .get(node.id.as_str())
                .cloned()
                .ok_or_else(|| EvidenceError::new("RELEASE_NODE_REFERENCE", node.id.clone()))?,
            depends_on,
        });
    }
    dependencies.sort_by(|left, right| left.bom_ref.cmp(&right.bom_ref));

    let bom = CycloneDxBom {
        bom_format: "CycloneDX",
        spec_version: "1.6",
        version: 1,
        metadata: BomMetadata {
            component: BomComponent {
                component_type: "application",
                bom_ref: WORKSPACE_BOM_REF.to_owned(),
                name: "SeaCad".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                licenses: vec![BomLicenseChoice {
                    expression: "LicenseRef-SeaCad-Proprietary".to_owned(),
                }],
                hashes: Vec::new(),
                purl: None,
                properties: Vec::new(),
            },
            properties: vec![
                BomProperty {
                    name: "seacad:cargo-lock-sha256".to_owned(),
                    value: lock_hash,
                },
                BomProperty {
                    name: "seacad:rust-version".to_owned(),
                    value: "1.97.1".to_owned(),
                },
            ],
        },
        components,
        dependencies,
    };
    let mut output = serde_json::to_vec_pretty(&bom)
        .map_err(|error| EvidenceError::new("RELEASE_SERIALIZE", error.to_string()))?;
    output.push(b'\n');
    Ok(output)
}

fn build_legal_bundle(
    root: &Path,
    metadata: &CargoMetadata,
    checksums: &BTreeMap<(String, String), String>,
    lock_hash: String,
) -> Result<LegalBundle, EvidenceError> {
    let workspace: BTreeSet<&str> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
    let mut files = BTreeMap::new();
    let mut root_files = Vec::new();
    for relative in ["LICENSE", "NOTICE", NOTICES_PATH] {
        let bytes = fs::read(root.join(relative))
            .map_err(|error| EvidenceError::new("RELEASE_LEGAL_ROOT_IO", error.to_string()))?;
        let bytes = canonical_project_text(bytes)?;
        insert_legal_file(&mut files, relative.to_owned(), bytes)?;
        let stored = files
            .get(relative)
            .ok_or_else(|| EvidenceError::new("RELEASE_LEGAL_ROOT", relative))?;
        root_files.push(legal_file_receipt(relative, stored)?);
    }

    let mut packages: Vec<&CargoPackage> = metadata
        .packages
        .iter()
        .filter(|package| !workspace.contains(package.id.as_str()))
        .collect();
    packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
    });
    let mut package_receipts = Vec::new();
    package_receipts
        .try_reserve(packages.len())
        .map_err(|error| EvidenceError::new("RELEASE_MEMORY", error.to_string()))?;
    for package in packages {
        if !valid_path_token(&package.name) || !valid_path_token(&package.version) {
            return Err(EvidenceError::new(
                "RELEASE_LEGAL_PACKAGE_PATH",
                format!("{} {}", package.name, package.version),
            ));
        }
        let crate_root = package.manifest_path.parent().ok_or_else(|| {
            EvidenceError::new(
                "RELEASE_LEGAL_MANIFEST_PATH",
                package.manifest_path.to_string_lossy(),
            )
        })?;
        let mut source_files = license_source_files(crate_root)?;
        if source_files.is_empty() {
            return Err(EvidenceError::new(
                "RELEASE_LEGAL_FILES_MISSING",
                format!("{} {}", package.name, package.version),
            ));
        }
        source_files.sort_by(|left, right| left.0.cmp(&right.0));
        let mut file_receipts = Vec::new();
        file_receipts
            .try_reserve(source_files.len())
            .map_err(|error| EvidenceError::new("RELEASE_MEMORY", error.to_string()))?;
        for (filename, source_path) in source_files {
            let destination = format!("packages/{}-{}/{}", package.name, package.version, filename);
            let bytes = fs::read(&source_path).map_err(|error| {
                EvidenceError::new(
                    "RELEASE_LEGAL_SOURCE_IO",
                    format!("{}: {error}", source_path.to_string_lossy()),
                )
            })?;
            insert_legal_file(&mut files, destination.clone(), bytes)?;
            let stored = files
                .get(&destination)
                .ok_or_else(|| EvidenceError::new("RELEASE_LEGAL_FILE", &destination))?;
            file_receipts.push(legal_file_receipt(&destination, stored)?);
        }
        let crate_sha256 = checksums
            .get(&(package.name.clone(), package.version.clone()))
            .cloned()
            .ok_or_else(|| {
                EvidenceError::new(
                    "RELEASE_CHECKSUM_MISSING",
                    format!("{} {}", package.name, package.version),
                )
            })?;
        package_receipts.push(LegalPackageReceipt {
            name: package.name.clone(),
            version: package.version.clone(),
            license_expression: package.license.clone().ok_or_else(|| {
                EvidenceError::new("RELEASE_LICENSE_MISSING", package.name.clone())
            })?,
            crate_sha256,
            files: file_receipts,
        });
    }

    let manifest = LegalManifest {
        contract: "seacad-release-legal-bundle/v1",
        cargo_lock_sha256: lock_hash,
        root_files,
        packages: package_receipts,
    };
    let mut manifest = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| EvidenceError::new("RELEASE_LEGAL_SERIALIZE", error.to_string()))?;
    manifest.push(b'\n');
    Ok(LegalBundle { files, manifest })
}

fn canonical_project_text(bytes: Vec<u8>) -> Result<Vec<u8>, EvidenceError> {
    canonical_text(bytes, "RELEASE_LEGAL_LINE_ENDING")
}

fn canonical_text(
    bytes: Vec<u8>,
    line_ending_code: &'static str,
) -> Result<Vec<u8>, EvidenceError> {
    let mut output = Vec::new();
    output
        .try_reserve(bytes.len())
        .map_err(|error| EvidenceError::new("RELEASE_MEMORY", error.to_string()))?;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\r' {
            if bytes.get(index + 1) != Some(&b'\n') {
                return Err(EvidenceError::new(
                    line_ending_code,
                    "canonical text contains a lone carriage return",
                ));
            }
            index += 1;
        }
        output.push(bytes[index]);
        index += 1;
    }
    Ok(output)
}

fn license_source_files(crate_root: &Path) -> Result<Vec<(String, PathBuf)>, EvidenceError> {
    let entries = fs::read_dir(crate_root).map_err(|error| {
        EvidenceError::new(
            "RELEASE_LEGAL_DIRECTORY",
            format!("{}: {error}", crate_root.to_string_lossy()),
        )
    })?;
    let mut files = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| EvidenceError::new("RELEASE_LEGAL_ENTRY", error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| EvidenceError::new("RELEASE_LEGAL_TYPE", error.to_string()))?;
        if !file_type.is_file() {
            continue;
        }
        let filename = entry.file_name().into_string().map_err(|_| {
            EvidenceError::new("RELEASE_LEGAL_FILENAME", crate_root.to_string_lossy())
        })?;
        if is_license_filename(&filename) {
            files.push((filename, entry.path()));
        }
    }
    Ok(files)
}

fn is_license_filename(filename: &str) -> bool {
    ["LICENSE", "LICENCE", "COPYING", "UNLICENSE", "NOTICE"]
        .iter()
        .any(|prefix| filename.starts_with(prefix))
}

fn valid_path_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+'))
}

fn insert_legal_file(
    files: &mut BTreeMap<String, Vec<u8>>,
    path: String,
    bytes: Vec<u8>,
) -> Result<(), EvidenceError> {
    if files.insert(path.clone(), bytes).is_some() {
        Err(EvidenceError::new("RELEASE_LEGAL_DUPLICATE", path))
    } else {
        Ok(())
    }
}

fn legal_file_receipt(path: &str, bytes: &[u8]) -> Result<LegalFileReceipt, EvidenceError> {
    Ok(LegalFileReceipt {
        path: path.to_owned(),
        bytes: u64::try_from(bytes.len())
            .map_err(|error| EvidenceError::new("RELEASE_LEGAL_SIZE", error.to_string()))?,
        sha256: hash_bytes(bytes),
    })
}

fn package_reference(package: &CargoPackage, workspace: bool) -> String {
    if workspace {
        format!(
            "pkg:cargo/{}@{}?workspace=true",
            package.name, package.version
        )
    } else {
        format!("pkg:cargo/{}@{}", package.name, package.version)
    }
}

fn locked_checksums(path: &Path) -> Result<BTreeMap<(String, String), String>, EvidenceError> {
    let lock = fs::read_to_string(path)
        .map_err(|error| EvidenceError::new("RELEASE_LOCK_IO", error.to_string()))?;
    let mut checksums = BTreeMap::new();
    let mut name = None;
    let mut version = None;
    let mut checksum = None;
    for line in lock.lines().chain(["[[package]]"]) {
        if line == "[[package]]" {
            if let (Some(name), Some(version), Some(checksum)) =
                (name.take(), version.take(), checksum.take())
                && checksums.insert((name, version), checksum).is_some()
            {
                return Err(EvidenceError::new(
                    "RELEASE_CHECKSUM_DUPLICATE",
                    "duplicate locked package identity",
                ));
            }
            name = None;
            version = None;
            checksum = None;
        } else if let Some(value) = quoted_value(line, "name = ") {
            name = Some(value.to_owned());
        } else if let Some(value) = quoted_value(line, "version = ") {
            version = Some(value.to_owned());
        } else if let Some(value) = quoted_value(line, "checksum = ") {
            checksum = Some(value.to_owned());
        }
    }
    Ok(checksums)
}

fn quoted_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    line.strip_prefix(prefix)?
        .strip_prefix('"')?
        .strip_suffix('"')
}

fn hash_file(path: &Path) -> Result<String, EvidenceError> {
    let bytes =
        fs::read(path).map_err(|error| EvidenceError::new("RELEASE_LOCK_IO", error.to_string()))?;
    Ok(hash_bytes(&canonical_text(
        bytes,
        "RELEASE_LOCK_LINE_ENDING",
    )?))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn check_legal_bundle(root: &Path, legal: &LegalBundle) -> Result<(), EvidenceError> {
    let legal_root = root.join(LEGAL_ROOT);
    let mut expected: BTreeSet<String> = legal.files.keys().cloned().collect();
    expected.insert("manifest.json".to_owned());
    let observed = legal_bundle_paths(&legal_root)?;
    if observed != expected {
        let missing: Vec<&String> = expected.difference(&observed).collect();
        let unexpected: Vec<&String> = observed.difference(&expected).collect();
        return Err(EvidenceError::new(
            "RELEASE_LEGAL_FILE_SET",
            format!("missing={missing:?}, unexpected={unexpected:?}"),
        ));
    }
    for (relative, bytes) in &legal.files {
        check_legal_file(&legal_root.join(relative), bytes)?;
    }
    check_legal_file(&root.join(LEGAL_MANIFEST_PATH), &legal.manifest)
}

fn legal_bundle_paths(root: &Path) -> Result<BTreeSet<String>, EvidenceError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| EvidenceError::new("RELEASE_LEGAL_ROOT_IO", error.to_string()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(EvidenceError::new(
            "RELEASE_LEGAL_ROOT_TYPE",
            display_path(root),
        ));
    }
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| EvidenceError::new("RELEASE_LEGAL_DIRECTORY", error.to_string()))?;
        for entry in entries {
            let entry = entry
                .map_err(|error| EvidenceError::new("RELEASE_LEGAL_ENTRY", error.to_string()))?;
            let file_type = entry
                .file_type()
                .map_err(|error| EvidenceError::new("RELEASE_LEGAL_TYPE", error.to_string()))?;
            if file_type.is_symlink() {
                return Err(EvidenceError::new(
                    "RELEASE_LEGAL_SYMLINK",
                    display_path(&entry.path()),
                ));
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                let entry_path = entry.path();
                let relative = entry_path.strip_prefix(root).map_err(|error| {
                    EvidenceError::new("RELEASE_LEGAL_RELATIVE", error.to_string())
                })?;
                let relative = display_path(relative);
                if !files.insert(relative.clone()) {
                    return Err(EvidenceError::new("RELEASE_LEGAL_DUPLICATE", relative));
                }
            } else {
                return Err(EvidenceError::new(
                    "RELEASE_LEGAL_ENTRY_TYPE",
                    display_path(&entry.path()),
                ));
            }
        }
    }
    Ok(files)
}

fn check_legal_file(path: &Path, expected: &[u8]) -> Result<(), EvidenceError> {
    let observed = fs::read(path)
        .map_err(|error| EvidenceError::new("RELEASE_LEGAL_FILE_IO", error.to_string()))?;
    if observed == expected {
        Ok(())
    } else {
        Err(EvidenceError::new(
            "RELEASE_LEGAL_STALE",
            display_path(path),
        ))
    }
}

fn write_legal_bundle(root: &Path, legal: &LegalBundle) -> Result<(), EvidenceError> {
    let legal_root = root.join(LEGAL_ROOT);
    for (relative, bytes) in &legal.files {
        write_output(&legal_root.join(relative), bytes)?;
    }
    write_output(&root.join(LEGAL_MANIFEST_PATH), &legal.manifest)
}

fn check_output(path: &Path, expected: &[u8]) -> Result<(), EvidenceError> {
    let observed =
        fs::read(path).map_err(|error| EvidenceError::new("RELEASE_SBOM_IO", error.to_string()))?;
    if observed == expected {
        Ok(())
    } else {
        let difference = sbom_difference(&observed, expected);
        Err(EvidenceError::new(
            "RELEASE_SBOM_STALE",
            format!(
                "run seacad-release-evidence --write: {}; committed_sha256={}; \
                 generated_sha256={}; {difference}",
                display_path(path),
                hash_bytes(&observed),
                hash_bytes(expected),
            ),
        ))
    }
}

fn sbom_difference(observed: &[u8], expected: &[u8]) -> String {
    let observed_edges = dependency_edges(observed);
    let expected_edges = dependency_edges(expected);
    match (observed_edges, expected_edges) {
        (Ok(observed), Ok(expected)) => {
            let missing: Vec<&String> = expected.difference(&observed).take(8).collect();
            let unexpected: Vec<&String> = observed.difference(&expected).take(8).collect();
            format!(
                "missing_edges={} {missing:?}; unexpected_edges={} {unexpected:?}",
                expected.difference(&observed).count(),
                observed.difference(&expected).count(),
            )
        }
        (Err(error), _) => format!("committed_json_error={error}"),
        (_, Err(error)) => format!("generated_json_error={error}"),
    }
}

fn dependency_edges(bytes: &[u8]) -> Result<BTreeSet<String>, serde_json::Error> {
    let document: serde_json::Value = serde_json::from_slice(bytes)?;
    let mut edges = BTreeSet::new();
    if let Some(dependencies) = document
        .get("dependencies")
        .and_then(|value| value.as_array())
    {
        for dependency in dependencies {
            let Some(reference) = dependency.get("ref").and_then(|value| value.as_str()) else {
                continue;
            };
            let Some(depends_on) = dependency
                .get("dependsOn")
                .and_then(|value| value.as_array())
            else {
                continue;
            };
            for target in depends_on {
                if let Some(target) = target.as_str() {
                    edges.insert(format!("{reference} -> {target}"));
                }
            }
        }
    }
    Ok(edges)
}

fn write_output(path: &Path, output: &[u8]) -> Result<(), EvidenceError> {
    let parent = path
        .parent()
        .ok_or_else(|| EvidenceError::new("RELEASE_OUTPUT_PATH", display_path(path)))?;
    fs::create_dir_all(parent)
        .map_err(|error| EvidenceError::new("RELEASE_OUTPUT_DIR", error.to_string()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|error| EvidenceError::new("RELEASE_OUTPUT_CREATE", error.to_string()))?;
    file.write_all(output)
        .map_err(|error| EvidenceError::new("RELEASE_OUTPUT_WRITE", error.to_string()))?;
    file.flush()
        .map_err(|error| EvidenceError::new("RELEASE_OUTPUT_FLUSH", error.to_string()))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{
        LegalBundle, build_legal_bundle, canonical_text, cargo_metadata, check_legal_bundle,
        hash_bytes, hash_file, locked_checksums, render_bom, sbom_difference, validate_notices,
        write_legal_bundle,
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn complete_locked_graph_has_notices_and_deterministic_bom() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let metadata = cargo_metadata(&root)?;
        validate_notices(&root, &metadata)?;
        let checksums = locked_checksums(&root.join("Cargo.lock"))?;
        let lock_hash = hash_file(&root.join("Cargo.lock"))?;
        let first = render_bom(&metadata, &checksums, lock_hash.clone())?;
        let second = render_bom(&metadata, &checksums, lock_hash)?;
        assert_eq!(first, second);
        let value: serde_json::Value = serde_json::from_slice(&first)?;
        assert_eq!(value["bomFormat"], "CycloneDX");
        assert_eq!(value["specVersion"], "1.6");
        assert_eq!(
            value["components"].as_array().map(Vec::len),
            Some(metadata.packages.len())
        );
        Ok(())
    }

    #[test]
    fn legal_bundle_covers_every_registry_package_deterministically() -> Result<(), Box<dyn Error>>
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let metadata = cargo_metadata(&root)?;
        let checksums = locked_checksums(&root.join("Cargo.lock"))?;
        let lock_hash = hash_file(&root.join("Cargo.lock"))?;
        let first = build_legal_bundle(&root, &metadata, &checksums, lock_hash.clone())?;
        let second = build_legal_bundle(&root, &metadata, &checksums, lock_hash)?;
        assert_eq!(first.files, second.files);
        assert_eq!(first.manifest, second.manifest);
        let manifest: serde_json::Value = serde_json::from_slice(&first.manifest)?;
        assert_eq!(manifest["contract"], "seacad-release-legal-bundle/v1");
        assert_eq!(manifest["packages"].as_array().map(Vec::len), Some(26));
        assert!(first.files.len() > 26);
        Ok(())
    }

    #[test]
    fn legal_bundle_check_rejects_stale_and_unexpected_files() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let legal = LegalBundle {
            files: [("LICENSE".to_owned(), b"license".to_vec())]
                .into_iter()
                .collect(),
            manifest: b"{}\n".to_vec(),
        };
        write_legal_bundle(directory.path(), &legal)?;
        check_legal_bundle(directory.path(), &legal)?;

        let extra = directory.path().join("release/legal/extra");
        fs::write(&extra, b"extra")?;
        let error = check_legal_bundle(directory.path(), &legal)
            .err()
            .ok_or("unexpected legal file passed")?;
        assert_eq!(error.code, "RELEASE_LEGAL_FILE_SET");
        fs::remove_file(extra)?;

        fs::write(directory.path().join("release/legal/LICENSE"), b"changed")?;
        let error = check_legal_bundle(directory.path(), &legal)
            .err()
            .ok_or("stale legal file passed")?;
        assert_eq!(error.code, "RELEASE_LEGAL_STALE");
        Ok(())
    }

    #[test]
    fn stale_sbom_difference_reports_dependency_edges() {
        let committed = br#"{"dependencies":[{"ref":"pkg:a","dependsOn":["pkg:b","pkg:c"]}]}"#;
        let generated = br#"{"dependencies":[{"ref":"pkg:a","dependsOn":["pkg:b","pkg:d"]}]}"#;
        let difference = sbom_difference(committed, generated);
        assert!(difference.contains("missing_edges=1 [\"pkg:a -> pkg:d\"]"));
        assert!(difference.contains("unexpected_edges=1 [\"pkg:a -> pkg:c\"]"));
    }

    #[test]
    fn cargo_lock_identity_normalizes_crlf_and_rejects_lone_cr() -> Result<(), Box<dyn Error>> {
        let lf = canonical_text(b"version = 4\nname = \"sample\"\n".to_vec(), "LOCK_ENDING")?;
        let crlf = canonical_text(
            b"version = 4\r\nname = \"sample\"\r\n".to_vec(),
            "LOCK_ENDING",
        )?;
        assert_eq!(lf, crlf);
        assert_eq!(hash_bytes(&lf), hash_bytes(&crlf));

        let error = canonical_text(b"version = 4\rname = \"sample\"\n".to_vec(), "LOCK_ENDING")
            .err()
            .ok_or("lone carriage return passed")?;
        assert_eq!(error.code, "LOCK_ENDING");
        Ok(())
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Result<Self, std::io::Error> {
            let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "seacad-release-evidence-{}-{id}",
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
}

//! Deterministic CycloneDX inventory from the complete locked Cargo graph.

#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsStr,
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::{Command, ExitCode},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const OUTPUT_PATH: &str = "release/sbom.cdx.json";
const NOTICES_PATH: &str = "THIRD_PARTY_NOTICES.md";
const LOCK_PATH: &str = "Cargo.lock";
const WORKSPACE_BOM_REF: &str = "urn:seacad:workspace";

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
    let output = render_bom(&metadata, &checksums, lock_hash)?;
    let target = root.join(OUTPUT_PATH);
    match mode {
        Mode::Check => check_output(&target, &output),
        Mode::Write => write_output(&target, &output),
    }
}

fn cargo_metadata(root: &Path) -> Result<CargoMetadata, EvidenceError> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(root)
        .output()
        .map_err(|error| EvidenceError::new("RELEASE_METADATA_IO", error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(EvidenceError::new(
            "RELEASE_METADATA",
            stderr.trim().to_owned(),
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
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(output)
}

fn check_output(path: &Path, expected: &[u8]) -> Result<(), EvidenceError> {
    let observed =
        fs::read(path).map_err(|error| EvidenceError::new("RELEASE_SBOM_IO", error.to_string()))?;
    if observed == expected {
        Ok(())
    } else {
        Err(EvidenceError::new(
            "RELEASE_SBOM_STALE",
            format!(
                "run seacad-release-evidence --write: {}",
                display_path(path)
            ),
        ))
    }
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
    use std::error::Error;

    use super::{cargo_metadata, hash_file, locked_checksums, render_bom, validate_notices};

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
}

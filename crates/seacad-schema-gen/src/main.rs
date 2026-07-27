//! Deterministic internal generator for reviewed SeaCad DXF schema facts.

#![forbid(unsafe_code)]

use std::{
    collections::BTreeSet,
    env,
    ffi::OsStr,
    fmt::{self, Write as _},
    fs::{self, OpenOptions},
    io::Write as _,
    path::Path,
    process::ExitCode,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MANIFEST_PATH: &str = "schema/dxf/v1/manifest.json";
const OUTPUT_PATH: &str = "crates/seacad-dxf-core/src/generated/header_schema.rs";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaManifest {
    schema_version: String,
    families: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaFamily {
    schema_version: String,
    namespace: String,
    source: SchemaSource,
    fields: Vec<SchemaField>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaSource {
    id: String,
    topic_id: String,
    normalized_facts_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaField {
    id: String,
    dxf_name: String,
    group_codes: Vec<i16>,
    storage: StorageKind,
    cardinality: Cardinality,
    applicability: Applicability,
    default_policy: DefaultPolicy,
    source_id: String,
    evidence: String,
    review_state: ReviewState,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum StorageKind {
    ExactText,
    Handle,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Cardinality {
    Optional,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Applicability {
    NotYetReviewed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum DefaultPolicy {
    NotYetReviewed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewState {
    ShapeOnly,
}

#[derive(Clone, Copy)]
enum Mode {
    Check,
    Write,
}

#[derive(Debug)]
struct SchemaError {
    code: &'static str,
    path: String,
    entry: String,
    detail: String,
}

impl SchemaError {
    fn new(
        code: &'static str,
        path: impl Into<String>,
        entry: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            entry: entry.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} [{} {}]: {}",
            self.code, self.path, self.entry, self.detail
        )
    }
}

impl std::error::Error for SchemaError {}

fn main() -> ExitCode {
    match parse_mode().and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_mode() -> Result<Mode, SchemaError> {
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

fn usage_error() -> SchemaError {
    SchemaError::new(
        "SCHEMA_USAGE",
        "<arguments>",
        "root",
        "expected exactly one of --check or --write",
    )
}

fn run(mode: Mode) -> Result<(), SchemaError> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (manifest, families) = load_schema(&root)?;
    validate_schema(&manifest, &families)?;
    let receipt = normalized_receipt(&manifest, &families)?;
    let output = render_registry(&manifest, &families, &receipt)?;
    let target = root.join(OUTPUT_PATH);
    match mode {
        Mode::Check => check_output(&target, &output),
        Mode::Write => write_output(&target, &output),
    }
}

fn load_schema(root: &Path) -> Result<(SchemaManifest, Vec<SchemaFamily>), SchemaError> {
    let manifest: SchemaManifest = read_json(root, MANIFEST_PATH, "root")?;
    let mut families = Vec::new();
    families
        .try_reserve(manifest.families.len())
        .map_err(|error| {
            SchemaError::new(
                "SCHEMA_MEMORY",
                MANIFEST_PATH,
                "families",
                error.to_string(),
            )
        })?;
    for (index, relative) in manifest.families.iter().enumerate() {
        if !valid_family_filename(relative) {
            return Err(SchemaError::new(
                "SCHEMA_FAMILY_PATH",
                MANIFEST_PATH,
                format!("families[{index}]"),
                "family path must be one lowercase .json filename",
            ));
        }
        let path = format!("schema/dxf/v1/{relative}");
        families.push(read_json(root, &path, "root")?);
    }
    Ok((manifest, families))
}

fn read_json<T: for<'de> Deserialize<'de>>(
    root: &Path,
    relative: &str,
    entry: &str,
) -> Result<T, SchemaError> {
    let bytes = fs::read(root.join(relative))
        .map_err(|error| SchemaError::new("SCHEMA_IO", relative, entry, error.to_string()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| SchemaError::new("SCHEMA_JSON", relative, entry, error.to_string()))
}

fn validate_schema(
    manifest: &SchemaManifest,
    families: &[SchemaFamily],
) -> Result<(), SchemaError> {
    if manifest.schema_version != "dxf.v1" {
        return Err(SchemaError::new(
            "SCHEMA_VERSION",
            MANIFEST_PATH,
            "schema_version",
            "expected dxf.v1",
        ));
    }
    if families.is_empty() || families.len() != manifest.families.len() {
        return Err(SchemaError::new(
            "SCHEMA_FAMILY_COUNT",
            MANIFEST_PATH,
            "families",
            "manifest must load one or more families exactly",
        ));
    }
    let mut namespaces = BTreeSet::new();
    for (relative, family) in manifest.families.iter().zip(families) {
        let path = format!("schema/dxf/v1/{relative}");
        if family.schema_version != manifest.schema_version {
            return Err(SchemaError::new(
                "SCHEMA_VERSION",
                &path,
                "schema_version",
                "family version does not match manifest",
            ));
        }
        if !valid_id(&family.namespace) || !namespaces.insert(family.namespace.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_NAMESPACE",
                &path,
                "namespace",
                "namespace must be unique lowercase ASCII",
            ));
        }
        validate_source(&family.source, &path)?;
        validate_fields(family, &path)?;
    }
    Ok(())
}

fn validate_source(source: &SchemaSource, path: &str) -> Result<(), SchemaError> {
    if !valid_id(&source.id)
        || !source.topic_id.starts_with("GUID-")
        || !source
            .topic_id
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
        || !valid_sha256(&source.normalized_facts_sha256)
    {
        return Err(SchemaError::new(
            "SCHEMA_SOURCE",
            path,
            "source",
            "source id, topic id, or normalized SHA-256 is invalid",
        ));
    }
    Ok(())
}

fn validate_fields(family: &SchemaFamily, path: &str) -> Result<(), SchemaError> {
    if family.fields.is_empty() {
        return Err(SchemaError::new(
            "SCHEMA_FIELD_COUNT",
            path,
            "fields",
            "schema family must contain at least one field",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut previous_id: Option<&str> = None;
    for (index, field) in family.fields.iter().enumerate() {
        let entry = format!("fields[{index}]");
        if !valid_id(&field.id) || !ids.insert(field.id.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_FIELD_ID",
                path,
                format!("{entry}.id"),
                "field id must be unique lowercase ASCII",
            ));
        }
        if previous_id.is_some_and(|previous| previous >= field.id.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_FIELD_ORDER",
                path,
                format!("{entry}.id"),
                "field ids must be strictly increasing",
            ));
        }
        previous_id = Some(&field.id);
        if !valid_dxf_name(&field.dxf_name) || !names.insert(field.dxf_name.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_DXF_NAME",
                path,
                format!("{entry}.dxf_name"),
                "DXF name must be unique canonical uppercase ASCII",
            ));
        }
        if field.source_id != family.source.id
            || field.evidence != format!("row:{}", field.dxf_name)
        {
            return Err(SchemaError::new(
                "SCHEMA_EVIDENCE",
                path,
                format!("{entry}.source_id"),
                "field must cite its family's source and exact normalized row",
            ));
        }
        validate_wire_shape(field, path, &entry)?;
    }
    Ok(())
}

fn validate_wire_shape(field: &SchemaField, path: &str, entry: &str) -> Result<(), SchemaError> {
    let valid = matches!(
        (field.storage, field.group_codes.as_slice()),
        (StorageKind::ExactText, [1 | 3]) | (StorageKind::Handle, [5])
    );
    if !valid {
        return Err(SchemaError::new(
            "SCHEMA_WIRE_SHAPE",
            path,
            format!("{entry}.group_codes"),
            "bootstrap storage kind does not match the reviewed M5 wire family",
        ));
    }
    Ok(())
}

fn normalized_receipt(
    manifest: &SchemaManifest,
    families: &[SchemaFamily],
) -> Result<String, SchemaError> {
    let mut hasher = Sha256::new();
    update_normalized_hash(&mut hasher, manifest, "manifest")?;
    for (index, family) in families.iter().enumerate() {
        update_normalized_hash(&mut hasher, family, &format!("families[{index}]"))?;
    }
    let mut receipt = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut receipt, "{byte:02x}").map_err(|error| {
            SchemaError::new("SCHEMA_RENDER", OUTPUT_PATH, "receipt", error.to_string())
        })?;
    }
    Ok(receipt)
}

fn update_normalized_hash<T: Serialize>(
    hasher: &mut Sha256,
    value: &T,
    entry: &str,
) -> Result<(), SchemaError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        SchemaError::new("SCHEMA_JSON", MANIFEST_PATH, entry, error.to_string())
    })?;
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn render_registry(
    manifest: &SchemaManifest,
    families: &[SchemaFamily],
    receipt: &str,
) -> Result<String, SchemaError> {
    let mut output = String::new();
    writeln!(
        output,
        "// @generated by seacad-schema-gen {}.",
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(output, "// Normalized input SHA-256: {receipt}")?;
    writeln!(
        output,
        "// Shape metadata only; this file makes no semantic support claim.\n"
    )?;
    output.push_str("#![allow(dead_code)]\n\n");
    output.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    output.push_str("pub(crate) enum DxfSchemaStorageKind {\n");
    output.push_str("    ExactText,\n    Handle,\n}\n\n");
    output.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    output.push_str("pub(crate) struct DxfHeaderSchemaField {\n");
    output.push_str("    pub id: &'static str,\n    pub dxf_name: &'static str,\n");
    output.push_str("    pub group_codes: &'static [i16],\n");
    output.push_str("    pub storage: DxfSchemaStorageKind,\n");
    output.push_str("    pub source_id: &'static str,\n    pub source_topic_id: &'static str,\n");
    output.push_str("    pub evidence: &'static str,\n    pub shape_only: bool,\n}\n\n");
    writeln!(
        output,
        "pub(crate) const SCHEMA_VERSION: &str = {:?};",
        manifest.schema_version
    )?;
    output.push_str("pub(crate) const NORMALIZED_INPUT_SHA256: &str =\n");
    writeln!(output, "    {receipt:?};\n")?;
    output.push_str("pub(crate) const HEADER_FIELDS: &[DxfHeaderSchemaField] = &[\n");
    for family in families {
        for field in &family.fields {
            output.push_str("    DxfHeaderSchemaField {\n");
            writeln!(output, "        id: {:?},", field.id)?;
            writeln!(output, "        dxf_name: {:?},", field.dxf_name)?;
            writeln!(output, "        group_codes: &{:?},", field.group_codes)?;
            writeln!(
                output,
                "        storage: DxfSchemaStorageKind::{},",
                storage_variant(field.storage)
            )?;
            writeln!(output, "        source_id: {:?},", field.source_id)?;
            writeln!(
                output,
                "        source_topic_id: {:?},",
                family.source.topic_id
            )?;
            writeln!(output, "        evidence: {:?},", field.evidence)?;
            output.push_str("        shape_only: true,\n    },\n");
        }
    }
    output.push_str("];\n");
    Ok(output)
}

fn storage_variant(storage: StorageKind) -> &'static str {
    match storage {
        StorageKind::ExactText => "ExactText",
        StorageKind::Handle => "Handle",
    }
}

fn check_output(target: &Path, expected: &str) -> Result<(), SchemaError> {
    let actual = fs::read_to_string(target)
        .map_err(|error| SchemaError::new("SCHEMA_IO", OUTPUT_PATH, "output", error.to_string()))?;
    if actual != expected {
        return Err(SchemaError::new(
            "SCHEMA_OUTPUT_DIFF",
            OUTPUT_PATH,
            "output",
            "committed generated Rust differs; run --write and review the diff",
        ));
    }
    Ok(())
}

fn write_output(target: &Path, output: &str) -> Result<(), SchemaError> {
    if fs::read_to_string(target).is_ok_and(|current| current == output) {
        return Ok(());
    }
    let temporary = target.with_extension("rs.tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            SchemaError::new("SCHEMA_IO", OUTPUT_PATH, "temporary", error.to_string())
        })?;
    let write_result = file
        .write_all(output.as_bytes())
        .and_then(|()| file.sync_all());
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(SchemaError::new(
            "SCHEMA_IO",
            OUTPUT_PATH,
            "temporary",
            error.to_string(),
        ));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        let _ = fs::remove_file(&temporary);
        return Err(SchemaError::new(
            "SCHEMA_IO",
            OUTPUT_PATH,
            "rename",
            error.to_string(),
        ));
    }
    Ok(())
}

fn valid_family_filename(value: &str) -> bool {
    value.ends_with(".json")
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.')
        })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.')
        })
}

fn valid_dxf_name(value: &str) -> bool {
    value.starts_with('$')
        && value.len() > 1
        && value.as_bytes()[1..]
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl From<fmt::Error> for SchemaError {
    fn from(error: fmt::Error) -> Self {
        Self::new("SCHEMA_RENDER", OUTPUT_PATH, "output", error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{
        MANIFEST_PATH, SchemaManifest, load_schema, normalized_receipt, render_registry,
        validate_schema,
    };

    #[test]
    fn approved_bootstrap_is_valid_and_deterministic() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, families) = load_schema(&root)?;
        validate_schema(&manifest, &families)?;
        let first = normalized_receipt(&manifest, &families)?;
        let second = normalized_receipt(&manifest, &families)?;
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert_eq!(
            render_registry(&manifest, &families, &first)?,
            render_registry(&manifest, &families, &second)?
        );
        Ok(())
    }

    #[test]
    fn unknown_manifest_key_is_rejected() {
        let json = r#"{"schema_version":"dxf.v1","families":[],"typo":true}"#;
        let result = serde_json::from_str::<SchemaManifest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_field_id_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, mut families) = load_schema(&root)?;
        let duplicate = families[0].fields[0].clone();
        families[0].fields.insert(1, duplicate);
        let error = validate_schema(&manifest, &families)
            .err()
            .ok_or("duplicate schema field unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_FIELD_ID");
        assert_eq!(MANIFEST_PATH, "schema/dxf/v1/manifest.json");
        Ok(())
    }

    #[test]
    fn invalid_wire_shape_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, mut families) = load_schema(&root)?;
        families[0].fields[0].group_codes = vec![70];
        let error = validate_schema(&manifest, &families)
            .err()
            .ok_or("invalid wire shape unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");
        assert_eq!(error.path, "schema/dxf/v1/header.bootstrap.json");
        Ok(())
    }

    #[test]
    fn mismatched_source_reference_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, mut families) = load_schema(&root)?;
        families[0].fields[0].source_id = "autodesk.other".to_string();
        let error = validate_schema(&manifest, &families)
            .err()
            .ok_or("mismatched source reference unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_EVIDENCE");
        Ok(())
    }
}

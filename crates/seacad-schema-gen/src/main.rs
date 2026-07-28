//! Deterministic internal generator for reviewed SeaCad DXF schema facts.

#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
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
    sources: String,
    families: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceRegistry {
    schema_version: String,
    sources: Vec<SchemaSource>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaSource {
    id: String,
    topic_id: String,
    normalized_facts_sha256: String,
    evidence_kind: EvidenceKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaFamily {
    schema_version: String,
    namespace: String,
    fields: Vec<SchemaField>,
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
    Boolean,
    Double,
    Double2,
    Double3,
    ElapsedDays,
    ExactText,
    Handle,
    Int16,
    JulianDate,
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
    Semantic,
    ShapeOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceKind {
    Row,
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
    let (manifest, sources, families) = load_schema(&root)?;
    validate_schema(&manifest, &sources, &families)?;
    let receipt = normalized_receipt(&manifest, &sources, &families)?;
    let output = render_registry(&manifest, &sources, &families, &receipt)?;
    let target = root.join(OUTPUT_PATH);
    match mode {
        Mode::Check => check_output(&target, &output),
        Mode::Write => write_output(&target, &output),
    }
}

fn load_schema(
    root: &Path,
) -> Result<(SchemaManifest, SourceRegistry, Vec<SchemaFamily>), SchemaError> {
    let manifest: SchemaManifest = read_json(root, MANIFEST_PATH, "root")?;
    if !valid_family_filename(&manifest.sources) {
        return Err(SchemaError::new(
            "SCHEMA_SOURCE_PATH",
            MANIFEST_PATH,
            "sources",
            "source registry path must be one lowercase .json filename",
        ));
    }
    let sources_path = format!("schema/dxf/v1/{}", manifest.sources);
    let sources = read_json(root, &sources_path, "root")?;
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
    Ok((manifest, sources, families))
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
    registry: &SourceRegistry,
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
    let sources_path = format!("schema/dxf/v1/{}", manifest.sources);
    if registry.schema_version != manifest.schema_version {
        return Err(SchemaError::new(
            "SCHEMA_VERSION",
            &sources_path,
            "schema_version",
            "source registry version does not match manifest",
        ));
    }
    let sources = validate_sources(registry, &sources_path)?;
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
        validate_fields(family, &path, &sources)?;
    }
    Ok(())
}

fn validate_sources<'a>(
    registry: &'a SourceRegistry,
    path: &str,
) -> Result<BTreeMap<&'a str, &'a SchemaSource>, SchemaError> {
    if registry.sources.is_empty() {
        return Err(SchemaError::new(
            "SCHEMA_SOURCE_COUNT",
            path,
            "sources",
            "source registry must contain at least one source",
        ));
    }
    let mut sources = BTreeMap::new();
    let mut source_receipts = BTreeSet::new();
    let mut previous_id: Option<&str> = None;
    for (index, source) in registry.sources.iter().enumerate() {
        let entry = format!("sources[{index}]");
        if !valid_id(&source.id)
            || previous_id.is_some_and(|previous| previous >= source.id.as_str())
            || sources.insert(source.id.as_str(), source).is_some()
        {
            return Err(SchemaError::new(
                "SCHEMA_SOURCE_ID",
                path,
                format!("{entry}.id"),
                "source ids must be unique lowercase ASCII in increasing order",
            ));
        }
        previous_id = Some(&source.id);
        if !valid_topic_id(&source.topic_id) || !valid_sha256(&source.normalized_facts_sha256) {
            return Err(SchemaError::new(
                "SCHEMA_SOURCE",
                path,
                entry,
                "source topic or normalized SHA-256 is invalid",
            ));
        }
        if !source_receipts.insert((
            source.topic_id.as_str(),
            source.normalized_facts_sha256.as_str(),
        )) {
            return Err(SchemaError::new(
                "SCHEMA_SOURCE_RECEIPT",
                path,
                entry,
                "source topic and normalized SHA-256 pair must be unique",
            ));
        }
    }
    Ok(sources)
}

fn validate_fields(
    family: &SchemaFamily,
    path: &str,
    sources: &BTreeMap<&str, &SchemaSource>,
) -> Result<(), SchemaError> {
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
    // Manifest order defines append-only schema ordinals; ids need not sort.
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
        if !valid_dxf_name(&field.dxf_name) || !names.insert(field.dxf_name.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_DXF_NAME",
                path,
                format!("{entry}.dxf_name"),
                "DXF name must be unique canonical uppercase ASCII",
            ));
        }
        let source = sources.get(field.source_id.as_str()).ok_or_else(|| {
            SchemaError::new(
                "SCHEMA_SOURCE_REF",
                path,
                format!("{entry}.source_id"),
                "field references an unknown source id",
            )
        })?;
        if !evidence_matches(source.evidence_kind, field) {
            return Err(SchemaError::new(
                "SCHEMA_EVIDENCE",
                path,
                format!("{entry}.evidence"),
                "field evidence does not match its source evidence kind",
            ));
        }
        validate_wire_shape(field, path, &entry)?;
    }
    Ok(())
}

fn validate_wire_shape(field: &SchemaField, path: &str, entry: &str) -> Result<(), SchemaError> {
    let valid = match (field.storage, field.group_codes.as_slice()) {
        (StorageKind::Boolean, [group_code]) => (290..=299).contains(group_code),
        (StorageKind::Double, [group_code]) => (10..=59).contains(group_code),
        (StorageKind::Double2, [x, y]) => (10..=18).contains(x) && *y == *x + 10,
        (StorageKind::Double3, [x, y, z]) => {
            (10..=18).contains(x) && *y == *x + 10 && *z == *x + 20
        }
        (StorageKind::ElapsedDays | StorageKind::JulianDate, [40]) => true,
        (StorageKind::ExactText, [1 | 2 | 3 | 6 | 7 | 8]) => true,
        (StorageKind::Handle, [5 | 345 | 346 | 349 | 390]) => true,
        (StorageKind::Int16, [group_code]) => {
            (60..=79).contains(group_code)
                || (270..=289).contains(group_code)
                || (370..=389).contains(group_code)
        }
        _ => false,
    };
    if !valid {
        return Err(SchemaError::new(
            "SCHEMA_WIRE_SHAPE",
            path,
            format!("{entry}.group_codes"),
            "storage kind does not match the reviewed M5 wire family",
        ));
    }
    Ok(())
}

fn normalized_receipt(
    manifest: &SchemaManifest,
    sources: &SourceRegistry,
    families: &[SchemaFamily],
) -> Result<String, SchemaError> {
    let mut hasher = Sha256::new();
    update_normalized_hash(&mut hasher, manifest, "manifest")?;
    update_normalized_hash(&mut hasher, sources, "sources")?;
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
    registry: &SourceRegistry,
    families: &[SchemaFamily],
    receipt: &str,
) -> Result<String, SchemaError> {
    let sources: BTreeMap<_, _> = registry
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect();
    let mut output = String::new();
    writeln!(
        output,
        "// @generated by seacad-schema-gen {}.",
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(output, "// Normalized input SHA-256: {receipt}")?;
    writeln!(
        output,
        "// Reviewed metadata; shape_only=false marks separately audited semantics.\n"
    )?;
    output.push_str("#![allow(dead_code)]\n\n");
    output.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    output.push_str("pub(crate) enum DxfSchemaStorageKind {\n");
    output.push_str(
        "    Boolean,\n    Double,\n    Double2,\n    Double3,\n    ElapsedDays,\n    ExactText,\n    Handle,\n    Int16,\n    JulianDate,\n}\n\n",
    );
    output.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    output.push_str("pub(crate) struct DxfHeaderSchemaField {\n");
    output.push_str("    pub id: &'static str,\n    pub dxf_name: &'static str,\n");
    output.push_str("    pub group_codes: &'static [i16],\n");
    output.push_str("    pub storage: DxfSchemaStorageKind,\n");
    output.push_str("    pub source_id: &'static str,\n    pub source_topic_id: &'static str,\n");
    output.push_str("    pub source_facts_sha256: &'static str,\n");
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
            let source = sources.get(field.source_id.as_str()).ok_or_else(|| {
                SchemaError::new(
                    "SCHEMA_SOURCE_REF",
                    OUTPUT_PATH,
                    &field.id,
                    "validated field source disappeared before rendering",
                )
            })?;
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
            writeln!(output, "        source_topic_id: {:?},", source.topic_id)?;
            writeln!(
                output,
                "        source_facts_sha256: {:?},",
                source.normalized_facts_sha256
            )?;
            writeln!(output, "        evidence: {:?},", field.evidence)?;
            writeln!(
                output,
                "        shape_only: {},",
                matches!(field.review_state, ReviewState::ShapeOnly)
            )?;
            output.push_str("    },\n");
        }
    }
    output.push_str("];\n");
    Ok(output)
}

fn storage_variant(storage: StorageKind) -> &'static str {
    match storage {
        StorageKind::Boolean => "Boolean",
        StorageKind::Double => "Double",
        StorageKind::Double2 => "Double2",
        StorageKind::Double3 => "Double3",
        StorageKind::ElapsedDays => "ElapsedDays",
        StorageKind::ExactText => "ExactText",
        StorageKind::Handle => "Handle",
        StorageKind::Int16 => "Int16",
        StorageKind::JulianDate => "JulianDate",
    }
}

fn evidence_matches(kind: EvidenceKind, field: &SchemaField) -> bool {
    match kind {
        EvidenceKind::Row => row_evidence_matches(field),
    }
}

fn row_evidence_matches(field: &SchemaField) -> bool {
    let Some(anchor) = field.evidence.strip_prefix("row:") else {
        return false;
    };
    if anchor == field.dxf_name {
        return true;
    }
    let Some((first_name, last_suffix)) = anchor.rsplit_once(" - ") else {
        return false;
    };
    let digit_start = first_name
        .bytes()
        .rposition(|byte| !byte.is_ascii_digit())
        .map_or(0, |index| index + 1);
    if digit_start == 0 || digit_start == first_name.len() {
        return false;
    }
    let (prefix, first_suffix) = first_name.split_at(digit_start);
    let Some(field_suffix) = field.dxf_name.strip_prefix(prefix) else {
        return false;
    };
    let (Ok(first), Ok(last), Ok(current)) = (
        first_suffix.parse::<u32>(),
        last_suffix.parse::<u32>(),
        field_suffix.parse::<u32>(),
    ) else {
        return false;
    };
    first <= current && current <= last
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
    value.len() > ".json".len()
        && value.ends_with(".json")
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.')
        })
}

fn valid_id(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && !value.ends_with('.')
        && !value.contains("..")
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.')
        })
}

fn valid_topic_id(value: &str) -> bool {
    let Some(guid) = value.strip_prefix("GUID-") else {
        return false;
    };
    guid.len() == 36
        && guid.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte)
            }
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
        EvidenceKind, MANIFEST_PATH, SchemaManifest, StorageKind, evidence_matches, load_schema,
        normalized_receipt, render_registry, validate_schema, validate_wire_shape,
    };

    #[test]
    fn approved_bootstrap_is_valid_and_deterministic() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, families) = load_schema(&root)?;
        validate_schema(&manifest, &sources, &families)?;
        assert_eq!(families[0].fields.len(), 214);
        let previous_ids = [
            "acadmaintver",
            "acadver",
            "angbase",
            "angdir",
            "attmode",
            "aunits",
            "auprec",
            "dwgcodepage",
            "extmax",
            "extmin",
            "handseed",
            "insbase",
            "limmax",
            "limmin",
            "pextmax",
            "pextmin",
            "pinsbase",
            "plimmax",
            "plimmin",
            "pucsorg",
            "pucsxdir",
            "pucsydir",
            "ucsorg",
            "ucsxdir",
            "ucsydir",
            "pucsorgback",
            "pucsorgbottom",
            "pucsorgfront",
            "pucsorgleft",
            "pucsorgright",
            "pucsorgtop",
            "ucsorgback",
            "ucsorgbottom",
            "ucsorgfront",
            "ucsorgleft",
            "ucsorgright",
            "ucsorgtop",
            "cecolor",
            "celtscale",
            "chamfera",
            "chamferb",
            "chamferc",
            "chamferd",
            "cmljust",
            "cmlscale",
            "elevation",
            "filletrad",
            "fillmode",
            "ltscale",
            "limcheck",
            "lunits",
            "luprec",
            "maxactvp",
            "measurement",
            "mirrtext",
            "orthomode",
            "pdmode",
            "pdsize",
            "pelevation",
            "plimcheck",
            "plinewid",
            "plinegen",
            "proxygraphics",
            "psltscale",
            "psvpscale",
            "pucsorthoview",
            "qtextmode",
            "regenmode",
            "shadedge",
            "shadedif",
            "shadowplanelocation",
            "sketchinc",
            "skpoly",
            "splinesegs",
            "splinetype",
            "surftab1",
            "surftab2",
            "surftype",
            "surfu",
            "surfv",
            "textsize",
            "thickness",
            "tilemode",
            "tracewid",
            "treedepth",
            "tdcreate",
            "tducreate",
            "tdupdate",
            "tduupdate",
            "tdindwg",
            "tdusrtimer",
            "endcaps",
            "extnames",
            "halogap",
            "hidetext",
            "indexctl",
            "intersectiondisplay",
            "joinstyle",
            "lwdisplay",
            "obsltype",
            "pstylemode",
            "celweight",
            "cepsntype",
            "cshadow",
            "dispsilh",
            "insunits",
            "interferecolor",
            "intersectioncolor",
            "obscolor",
            "sortents",
            "ucsorthoview",
            "unitmode",
            "usrtimer",
            "visretain",
            "worldview",
            "xclipframe",
            "xedit",
            "dimaltf",
            "dimaltrnd",
            "dimasz",
            "dimcen",
            "dimdle",
            "dimdli",
            "dimexe",
            "dimexo",
            "dimfac",
            "dimgap",
            "dimlfac",
            "dimrnd",
            "dimscale",
            "dimtfac",
            "dimtm",
            "dimtp",
            "dimtsz",
            "dimtvp",
            "dimtxt",
            "dimadec",
            "dimaltd",
            "dimalttd",
            "dimalttz",
            "dimaltu",
            "dimaltz",
            "dimaunit",
            "dimazin",
            "dimdec",
            "dimdsep",
            "dimlunit",
            "dimtdec",
            "dimtzin",
            "dimzin",
            "dimalt",
            "dimaso",
            "dimlim",
            "dimsah",
            "dimsd1",
            "dimsd2",
            "dimse1",
            "dimse2",
            "dimsho",
            "dimsoxd",
            "dimtih",
            "dimtix",
            "dimtofl",
            "dimtoh",
            "dimtol",
            "dimupt",
        ];
        for (field, expected_id) in families[0].fields.iter().zip(previous_ids) {
            assert_eq!(field.id, expected_id);
        }
        assert_eq!(families[0].fields[166].id, "dimassoc");
        assert_eq!(families[0].fields[186].id, "userr5");
        assert_eq!(families[0].fields[187].id, "celtype");
        assert_eq!(families[0].fields[197].id, "dimtxsty");
        let m6_6b_ids = [
            "fingerprintguid",
            "hyperlinkbase",
            "menu",
            "projectname",
            "pucsbase",
            "pucsname",
            "pucsorthoref",
            "textstyle",
            "ucsbase",
            "ucsname",
            "ucsorthoref",
            "versionguid",
        ];
        for (field, expected_id) in families[0].fields[198..].iter().zip(m6_6b_ids) {
            assert_eq!(field.id, expected_id);
        }
        let m6_6c_ids = ["cepsnid", "dragvs", "interfereobjvs", "interferevpvs"];
        for (field, expected_id) in families[0].fields[210..].iter().zip(m6_6c_ids) {
            assert_eq!(field.id, expected_id);
        }
        let first = normalized_receipt(&manifest, &sources, &families)?;
        let second = normalized_receipt(&manifest, &sources, &families)?;
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert_eq!(
            render_registry(&manifest, &sources, &families, &first)?,
            render_registry(&manifest, &sources, &families, &second)?
        );
        Ok(())
    }

    #[test]
    fn unknown_manifest_key_is_rejected() {
        let json =
            r#"{"schema_version":"dxf.v1","sources":"sources.json","families":[],"typo":true}"#;
        let result = serde_json::from_str::<SchemaManifest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_field_id_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, mut families) = load_schema(&root)?;
        let duplicate = families[0].fields[0].clone();
        families[0].fields.insert(1, duplicate);
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("duplicate schema field unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_FIELD_ID");
        assert_eq!(MANIFEST_PATH, "schema/dxf/v1/manifest.json");
        Ok(())
    }

    #[test]
    fn invalid_wire_shape_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, mut families) = load_schema(&root)?;
        families[0].fields[0].group_codes = vec![80];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid wire shape unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");
        assert_eq!(error.path, "schema/dxf/v1/header.bootstrap.json");
        Ok(())
    }

    #[test]
    fn exact_text_wire_codes_are_explicitly_bounded() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, families) = load_schema(&root)?;
        for code in [1, 2, 3, 6, 7, 8] {
            let field = families[0]
                .fields
                .iter()
                .find(|field| {
                    matches!(field.storage, StorageKind::ExactText) && field.group_codes == [code]
                })
                .ok_or("missing reviewed exact-text wire code")?;
            validate_wire_shape(field, "test", "field")?;
        }
        validate_schema(&manifest, &sources, &families)?;

        for code in [0, 4, 5, 9] {
            let mut field = families[0]
                .fields
                .iter()
                .find(|field| matches!(field.storage, StorageKind::ExactText))
                .cloned()
                .ok_or("missing exact-text field")?;
            field.group_codes = vec![code];
            let error = validate_wire_shape(&field, "test", "field")
                .err()
                .ok_or("invalid exact-text wire code unexpectedly passed")?;
            assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");
        }
        Ok(())
    }

    #[test]
    fn handle_wire_codes_are_explicitly_bounded() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, families) = load_schema(&root)?;
        for code in [5, 345, 346, 349, 390] {
            let field = families[0]
                .fields
                .iter()
                .find(|field| {
                    matches!(field.storage, StorageKind::Handle) && field.group_codes == [code]
                })
                .ok_or("missing reviewed handle wire code")?;
            validate_wire_shape(field, "test", "field")?;
        }
        validate_schema(&manifest, &sources, &families)?;

        for code in [4, 6, 344, 347, 348, 350, 389, 391] {
            let mut field = families[0]
                .fields
                .iter()
                .find(|field| matches!(field.storage, StorageKind::Handle))
                .cloned()
                .ok_or("missing handle field")?;
            field.group_codes = vec![code];
            let error = validate_wire_shape(&field, "test", "field")
                .err()
                .ok_or("invalid handle wire code unexpectedly passed")?;
            assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");
        }
        Ok(())
    }

    #[test]
    fn numeric_wire_family_boundaries_fail_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, mut families) = load_schema(&root)?;
        let double = families[0]
            .fields
            .iter()
            .position(|field| field.id == "angbase")
            .ok_or("missing double field")?;
        families[0].fields[double].group_codes = vec![60];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid double wire family unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");

        let (manifest, sources, mut families) = load_schema(&root)?;
        let int16 = families[0]
            .fields
            .iter()
            .position(|field| field.id == "angdir")
            .ok_or("missing int16 field")?;
        families[0].fields[int16].group_codes = vec![59];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid int16 wire family unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");

        let (manifest, sources, mut families) = load_schema(&root)?;
        let double3 = families[0]
            .fields
            .iter()
            .position(|field| field.id == "extmax")
            .ok_or("missing double3 field")?;
        families[0].fields[double3].group_codes = vec![10, 20, 31];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid double3 wire shape unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");

        let (manifest, sources, mut families) = load_schema(&root)?;
        let double2 = families[0]
            .fields
            .iter()
            .position(|field| field.id == "limmax")
            .ok_or("missing double2 field")?;
        families[0].fields[double2].group_codes = vec![10, 21];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid double2 wire shape unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");

        let (manifest, sources, mut families) = load_schema(&root)?;
        let boolean = families[0]
            .fields
            .iter()
            .position(|field| field.id == "extnames")
            .ok_or("missing boolean field")?;
        families[0].fields[boolean].group_codes = vec![289];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid boolean wire family unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");

        let (manifest, sources, mut families) = load_schema(&root)?;
        let extended_int16 = families[0]
            .fields
            .iter()
            .position(|field| field.id == "endcaps")
            .ok_or("missing extended int16 field")?;
        families[0].fields[extended_int16].group_codes = vec![290];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid extended int16 wire family unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");

        let (manifest, sources, mut families) = load_schema(&root)?;
        let lineweight = families[0]
            .fields
            .iter()
            .position(|field| field.id == "celweight")
            .ok_or("missing lineweight int16 field")?;
        families[0].fields[lineweight].group_codes = vec![369];
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid lineweight int16 wire family unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_WIRE_SHAPE");
        Ok(())
    }

    #[test]
    fn mismatched_source_reference_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, mut families) = load_schema(&root)?;
        families[0].fields[0].source_id = "autodesk.other".to_string();
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("mismatched source reference unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_SOURCE_REF");
        Ok(())
    }

    #[test]
    fn duplicate_source_id_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, mut sources, families) = load_schema(&root)?;
        sources.sources.push(sources.sources[0].clone());
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("duplicate source id unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_SOURCE_ID");
        assert_eq!(error.path, "schema/dxf/v1/sources.json");
        Ok(())
    }

    #[test]
    fn invalid_source_receipt_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, mut sources, families) = load_schema(&root)?;
        sources.sources[0].normalized_facts_sha256 = "invalid".to_string();
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("invalid source receipt unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_SOURCE");
        Ok(())
    }

    #[test]
    fn duplicate_topic_receipt_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, mut sources, families) = load_schema(&root)?;
        let mut duplicate = sources.sources[0].clone();
        duplicate.id = "autodesk.header.copy".to_string();
        sources.sources.push(duplicate);
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("duplicate source receipt unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_SOURCE_RECEIPT");
        Ok(())
    }

    #[test]
    fn ranged_row_evidence_expands_only_declared_numeric_suffix() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (_, _, families) = load_schema(&root)?;
        let mut field = families[0].fields[0].clone();
        field.evidence = "row:$USERI1 - 5".to_string();
        field.dxf_name = "$USERI3".to_string();
        assert!(evidence_matches(EvidenceKind::Row, &field));
        field.dxf_name = "$USERI6".to_string();
        assert!(!evidence_matches(EvidenceKind::Row, &field));
        field.dxf_name = "$USERJ3".to_string();
        assert!(!evidence_matches(EvidenceKind::Row, &field));
        Ok(())
    }
    #[test]
    fn wrong_evidence_anchor_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, mut families) = load_schema(&root)?;
        families[0].fields[0].evidence = "row:$OTHER".to_string();
        let error = validate_schema(&manifest, &sources, &families)
            .err()
            .ok_or("wrong evidence anchor unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_EVIDENCE");
        Ok(())
    }
}

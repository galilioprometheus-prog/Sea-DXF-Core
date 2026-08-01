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
const HEADER_OUTPUT_PATH: &str = "crates/seacad-dxf-core/src/generated/header_schema.rs";
const ENTITY_OUTPUT_PATH: &str = "crates/seacad-dxf-core/src/generated/entity_schema.rs";
const EXPECTED_ENTITY_TOPIC_COUNT: usize = 45;
const EXPECTED_ENTITY_ALIAS_COUNT: usize = 14;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaManifest {
    schema_version: String,
    sources: String,
    entity_topics: String,
    entity_aliases: String,
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
struct EntityTopicRegistry {
    schema_version: String,
    namespace: String,
    source_id: String,
    topics: Vec<EntityTopic>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EntityTopic {
    id: String,
    dxf_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EntityAliasRegistry {
    schema_version: String,
    namespace: String,
    aliases: Vec<EntityAlias>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EntityAlias {
    id: String,
    dxf_name: String,
    topic_id: String,
    source_id: String,
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
    AliasList,
    OracleInventory,
    Row,
    TopicList,
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
    let entity_topics = load_entity_topics(&root, &manifest)?;
    let entity_source = validate_entity_topics(&manifest, &sources, &entity_topics)?;
    let entity_aliases = load_entity_aliases(&root, &manifest)?;
    validate_entity_aliases(&manifest, &sources, &entity_topics, &entity_aliases)?;
    let header_receipt = normalized_receipt(&manifest, &sources, &families)?;
    let entity_receipt = normalized_entity_receipt(&entity_topics, entity_source)?;
    let alias_receipt = normalized_alias_receipt(&entity_aliases, &sources)?;
    let header_output = render_registry(&manifest, &sources, &families, &header_receipt)?;
    let entity_output = render_entity_registry(
        &entity_topics,
        entity_source,
        &entity_receipt,
        &entity_aliases,
        &sources,
        &alias_receipt,
    )?;
    let header_target = root.join(HEADER_OUTPUT_PATH);
    let entity_target = root.join(ENTITY_OUTPUT_PATH);
    match mode {
        Mode::Check => {
            check_output(&header_target, &header_output, HEADER_OUTPUT_PATH)?;
            check_output(&entity_target, &entity_output, ENTITY_OUTPUT_PATH)
        }
        Mode::Write => {
            write_output(&header_target, &header_output, HEADER_OUTPUT_PATH)?;
            write_output(&entity_target, &entity_output, ENTITY_OUTPUT_PATH)
        }
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

fn load_entity_topics(
    root: &Path,
    manifest: &SchemaManifest,
) -> Result<EntityTopicRegistry, SchemaError> {
    if !valid_family_filename(&manifest.entity_topics) {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_PATH",
            MANIFEST_PATH,
            "entity_topics",
            "entity topic path must be one lowercase .json filename",
        ));
    }
    let path = format!("schema/dxf/v1/{}", manifest.entity_topics);
    read_json(root, &path, "root")
}

fn load_entity_aliases(
    root: &Path,
    manifest: &SchemaManifest,
) -> Result<EntityAliasRegistry, SchemaError> {
    if !valid_family_filename(&manifest.entity_aliases) {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_PATH",
            MANIFEST_PATH,
            "entity_aliases",
            "entity alias path must be one lowercase .json filename",
        ));
    }
    let path = format!("schema/dxf/v1/{}", manifest.entity_aliases);
    read_json(root, &path, "root")
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

fn validate_entity_topics<'a>(
    manifest: &SchemaManifest,
    registry: &'a SourceRegistry,
    entity_topics: &EntityTopicRegistry,
) -> Result<&'a SchemaSource, SchemaError> {
    let path = format!("schema/dxf/v1/{}", manifest.entity_topics);
    if entity_topics.schema_version != manifest.schema_version {
        return Err(SchemaError::new(
            "SCHEMA_VERSION",
            &path,
            "schema_version",
            "entity topic version does not match manifest",
        ));
    }
    if entity_topics.namespace != "entity" {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_NAMESPACE",
            &path,
            "namespace",
            "entity topic namespace must be exactly entity",
        ));
    }
    if entity_topics.topics.len() != EXPECTED_ENTITY_TOPIC_COUNT {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_COUNT",
            &path,
            "topics",
            format!("expected exactly {EXPECTED_ENTITY_TOPIC_COUNT} reviewed topics"),
        ));
    }

    let source = registry
        .sources
        .iter()
        .find(|source| source.id == entity_topics.source_id)
        .ok_or_else(|| {
            SchemaError::new(
                "SCHEMA_SOURCE_REF",
                &path,
                "source_id",
                "entity inventory references an unknown source id",
            )
        })?;
    if !matches!(source.evidence_kind, EvidenceKind::TopicList) {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_SOURCE",
            &path,
            "source_id",
            "entity inventory source must use topic_list evidence",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (index, topic) in entity_topics.topics.iter().enumerate() {
        let entry = format!("topics[{index}]");
        if !valid_entity_id(&topic.id) || !ids.insert(topic.id.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_ENTITY_ID",
                &path,
                format!("{entry}.id"),
                "entity topic id must be unique lowercase ASCII",
            ));
        }
        if !valid_entity_dxf_name(&topic.dxf_name) || !names.insert(topic.dxf_name.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_ENTITY_NAME",
                &path,
                format!("{entry}.dxf_name"),
                "entity DXF name must be unique uppercase ASCII",
            ));
        }
    }
    let normalized_facts = normalized_entity_facts_sha256(entity_topics)?;
    if source.normalized_facts_sha256 != normalized_facts {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_SOURCE_RECEIPT",
            &path,
            "source_id",
            format!(
                "recorded source facts differ from normalized topic facts; observed {normalized_facts}"
            ),
        ));
    }
    Ok(source)
}

fn normalized_entity_facts_sha256(
    entity_topics: &EntityTopicRegistry,
) -> Result<String, SchemaError> {
    let bytes = serde_json::to_vec(&entity_topics.topics).map_err(|error| {
        SchemaError::new(
            "SCHEMA_JSON",
            ENTITY_OUTPUT_PATH,
            "entity_topics",
            error.to_string(),
        )
    })?;
    let mut receipt = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut receipt, "{byte:02x}").map_err(|error| {
            SchemaError::new(
                "SCHEMA_RENDER",
                ENTITY_OUTPUT_PATH,
                "entity_facts_receipt",
                error.to_string(),
            )
        })?;
    }
    Ok(receipt)
}

fn validate_entity_aliases(
    manifest: &SchemaManifest,
    registry: &SourceRegistry,
    entity_topics: &EntityTopicRegistry,
    entity_aliases: &EntityAliasRegistry,
) -> Result<(), SchemaError> {
    let path = format!("schema/dxf/v1/{}", manifest.entity_aliases);
    if entity_aliases.schema_version != manifest.schema_version {
        return Err(SchemaError::new(
            "SCHEMA_VERSION",
            &path,
            "schema_version",
            "entity alias version does not match manifest",
        ));
    }
    if entity_aliases.namespace != "entity_alias" {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_ALIAS_NAMESPACE",
            &path,
            "namespace",
            "entity alias namespace must be exactly entity_alias",
        ));
    }
    if entity_aliases.aliases.len() != EXPECTED_ENTITY_ALIAS_COUNT {
        return Err(SchemaError::new(
            "SCHEMA_ENTITY_ALIAS_COUNT",
            &path,
            "aliases",
            format!("expected exactly {EXPECTED_ENTITY_ALIAS_COUNT} reviewed aliases"),
        ));
    }

    let topics: BTreeMap<_, _> = entity_topics
        .topics
        .iter()
        .map(|topic| (topic.id.as_str(), topic))
        .collect();
    let canonical_names: BTreeSet<_> = entity_topics
        .topics
        .iter()
        .map(|topic| topic.dxf_name.as_str())
        .collect();
    let sources: BTreeMap<_, _> = registry
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect();
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut referenced_sources = BTreeSet::new();
    for (index, alias) in entity_aliases.aliases.iter().enumerate() {
        let entry = format!("aliases[{index}]");
        if !valid_entity_id(&alias.id) || !ids.insert(alias.id.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_ENTITY_ALIAS_ID",
                &path,
                format!("{entry}.id"),
                "entity alias id must be unique lowercase ASCII",
            ));
        }
        if !valid_entity_dxf_name(&alias.dxf_name)
            || canonical_names.contains(alias.dxf_name.as_str())
            || !names.insert(alias.dxf_name.as_str())
        {
            return Err(SchemaError::new(
                "SCHEMA_ENTITY_ALIAS_NAME",
                &path,
                format!("{entry}.dxf_name"),
                "entity alias must be unique uppercase ASCII and not canonical",
            ));
        }
        if !topics.contains_key(alias.topic_id.as_str()) {
            return Err(SchemaError::new(
                "SCHEMA_ENTITY_ALIAS_TOPIC",
                &path,
                format!("{entry}.topic_id"),
                "entity alias references an unknown canonical topic id",
            ));
        }
        let source = sources.get(alias.source_id.as_str()).ok_or_else(|| {
            SchemaError::new(
                "SCHEMA_SOURCE_REF",
                &path,
                format!("{entry}.source_id"),
                "entity alias references an unknown source id",
            )
        })?;
        if !matches!(
            source.evidence_kind,
            EvidenceKind::AliasList | EvidenceKind::OracleInventory
        ) {
            return Err(SchemaError::new(
                "SCHEMA_ENTITY_ALIAS_SOURCE",
                &path,
                format!("{entry}.source_id"),
                "entity alias source must use alias_list or oracle_inventory evidence",
            ));
        }
        referenced_sources.insert(alias.source_id.as_str());
    }

    for source_id in referenced_sources {
        let source = sources.get(source_id).ok_or_else(|| {
            SchemaError::new(
                "SCHEMA_SOURCE_REF",
                &path,
                source_id,
                "validated alias source disappeared",
            )
        })?;
        let observed = normalized_alias_source_facts_sha256(entity_aliases, source_id)?;
        if source.normalized_facts_sha256 != observed {
            return Err(SchemaError::new(
                "SCHEMA_ENTITY_ALIAS_SOURCE_RECEIPT",
                &path,
                source_id,
                format!(
                    "recorded source facts differ from normalized alias facts; observed {observed}"
                ),
            ));
        }
    }
    Ok(())
}

fn normalized_alias_source_facts_sha256(
    entity_aliases: &EntityAliasRegistry,
    source_id: &str,
) -> Result<String, SchemaError> {
    let facts: Vec<_> = entity_aliases
        .aliases
        .iter()
        .filter(|alias| alias.source_id == source_id)
        .collect();
    let bytes = serde_json::to_vec(&facts).map_err(|error| {
        SchemaError::new(
            "SCHEMA_JSON",
            ENTITY_OUTPUT_PATH,
            "entity_aliases",
            error.to_string(),
        )
    })?;
    sha256_hex(&bytes, "entity_alias_facts")
}

fn sha256_hex(bytes: &[u8], entry: &str) -> Result<String, SchemaError> {
    digest_hex(Sha256::digest(bytes).as_slice(), entry)
}

fn digest_hex(bytes: &[u8], entry: &str) -> Result<String, SchemaError> {
    let mut receipt = String::with_capacity(64);
    for byte in bytes {
        write!(&mut receipt, "{byte:02x}").map_err(|error| {
            SchemaError::new(
                "SCHEMA_RENDER",
                ENTITY_OUTPUT_PATH,
                entry,
                error.to_string(),
            )
        })?;
    }
    Ok(receipt)
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
        if !valid_source_reference(source) || !valid_sha256(&source.normalized_facts_sha256) {
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
            SchemaError::new(
                "SCHEMA_RENDER",
                HEADER_OUTPUT_PATH,
                "receipt",
                error.to_string(),
            )
        })?;
    }
    Ok(receipt)
}

fn normalized_entity_receipt(
    entity_topics: &EntityTopicRegistry,
    source: &SchemaSource,
) -> Result<String, SchemaError> {
    let mut hasher = Sha256::new();
    update_normalized_hash(&mut hasher, entity_topics, "entity_topics")?;
    update_normalized_hash(&mut hasher, source, "entity_source")?;
    let mut receipt = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(&mut receipt, "{byte:02x}").map_err(|error| {
            SchemaError::new(
                "SCHEMA_RENDER",
                ENTITY_OUTPUT_PATH,
                "receipt",
                error.to_string(),
            )
        })?;
    }
    Ok(receipt)
}

fn normalized_alias_receipt(
    entity_aliases: &EntityAliasRegistry,
    registry: &SourceRegistry,
) -> Result<String, SchemaError> {
    let mut hasher = Sha256::new();
    update_normalized_hash(&mut hasher, entity_aliases, "entity_aliases")?;
    for source in &registry.sources {
        if matches!(
            source.evidence_kind,
            EvidenceKind::AliasList | EvidenceKind::OracleInventory
        ) {
            update_normalized_hash(&mut hasher, source, "entity_alias_source")?;
        }
    }
    digest_hex(hasher.finalize().as_slice(), "entity_alias_receipt")
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
                    HEADER_OUTPUT_PATH,
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

fn render_entity_registry(
    entity_topics: &EntityTopicRegistry,
    source: &SchemaSource,
    receipt: &str,
    entity_aliases: &EntityAliasRegistry,
    registry: &SourceRegistry,
    alias_receipt: &str,
) -> Result<String, SchemaError> {
    let mut output = String::new();
    writeln!(
        output,
        "// @generated by seacad-schema-gen {}.",
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(output, "// Normalized entity input SHA-256: {receipt}")?;
    output.push_str("// Reviewed names only; classification does not imply semantic support.\n\n");
    output.push_str("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]\n");
    output.push_str("pub struct DxfEntityTopic {\n    ordinal: u8,\n}\n\n");
    output.push_str("impl DxfEntityTopic {\n");
    for (ordinal, topic) in entity_topics.topics.iter().enumerate() {
        writeln!(
            output,
            "    pub const {}: Self = Self {{ ordinal: {ordinal} }};",
            topic.id.to_ascii_uppercase()
        )?;
    }
    output.push_str("\n    #[must_use]\n    pub const fn ordinal(self) -> u8 {\n        self.ordinal\n    }\n\n");
    output.push_str(
        "    #[must_use]\n    pub fn from_ordinal(ordinal: u8) -> Option<Self> {\n        DXF_ENTITY_TOPICS\n            .get(usize::from(ordinal))\n            .map(|descriptor| descriptor.topic())\n    }\n\n",
    );
    output.push_str(
        "    #[must_use]\n    pub fn from_exact_name(name: &[u8]) -> Option<Self> {\n        match name {\n",
    );
    for topic in &entity_topics.topics {
        writeln!(
            output,
            "            b\"{}\" => Some(Self::{}),",
            topic.dxf_name,
            topic.id.to_ascii_uppercase()
        )?;
    }
    output.push_str("            _ => None,\n        }\n    }\n\n");
    output.push_str(
        "    #[must_use]\n    pub fn descriptor(self) -> Option<&'static DxfEntityTopicDescriptor> {\n        DXF_ENTITY_TOPICS.get(usize::from(self.ordinal))\n    }\n}\n\n",
    );
    output.push_str("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]\n");
    output.push_str(
        "pub struct DxfEntityTopicDescriptor {\n    topic: DxfEntityTopic,\n    id: &'static str,\n    dxf_name: &'static str,\n    source_id: &'static str,\n    source_topic_id: &'static str,\n    source_facts_sha256: &'static str,\n}\n\n",
    );
    output.push_str("impl DxfEntityTopicDescriptor {\n");
    output.push_str("    #[must_use]\n    pub const fn topic(self) -> DxfEntityTopic {\n        self.topic\n    }\n\n");
    output.push_str(
        "    #[must_use]\n    pub const fn id(self) -> &'static str {\n        self.id\n    }\n\n",
    );
    output.push_str("    #[must_use]\n    pub const fn dxf_name(self) -> &'static str {\n        self.dxf_name\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn source_id(self) -> &'static str {\n        self.source_id\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn source_topic_id(self) -> &'static str {\n        self.source_topic_id\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn source_facts_sha256(self) -> &'static str {\n        self.source_facts_sha256\n    }\n}\n\n");
    output.push_str("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]\n");
    output.push_str("pub struct DxfEntityAlias {\n    ordinal: u8,\n}\n\n");
    output.push_str("impl DxfEntityAlias {\n");
    for (ordinal, alias) in entity_aliases.aliases.iter().enumerate() {
        writeln!(
            output,
            "    pub const {}: Self = Self {{ ordinal: {ordinal} }};",
            alias.id.to_ascii_uppercase()
        )?;
    }
    output.push_str("\n    #[must_use]\n    pub const fn ordinal(self) -> u8 {\n        self.ordinal\n    }\n\n");
    output.push_str(
        "    #[must_use]\n    pub fn from_ordinal(ordinal: u8) -> Option<Self> {\n        DXF_ENTITY_ALIASES\n            .get(usize::from(ordinal))\n            .map(|descriptor| descriptor.alias())\n    }\n\n",
    );
    output.push_str(
        "    #[must_use]\n    pub fn from_exact_name(name: &[u8]) -> Option<Self> {\n        match name {\n",
    );
    for alias in &entity_aliases.aliases {
        writeln!(
            output,
            "            b\"{}\" => Some(Self::{}),",
            alias.dxf_name,
            alias.id.to_ascii_uppercase()
        )?;
    }
    output.push_str("            _ => None,\n        }\n    }\n\n");
    output.push_str(
        "    #[must_use]\n    pub fn descriptor(self) -> Option<&'static DxfEntityAliasDescriptor> {\n        DXF_ENTITY_ALIASES.get(usize::from(self.ordinal))\n    }\n}\n\n",
    );
    output.push_str("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]\n");
    output.push_str(
        "pub enum DxfEntityAliasEvidence {\n    Normative,\n    BehavioralOracle,\n}\n\n",
    );
    output.push_str("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]\n");
    output.push_str(
        "pub struct DxfEntityAliasDescriptor {\n    alias: DxfEntityAlias,\n    id: &'static str,\n    dxf_name: &'static str,\n    topic: DxfEntityTopic,\n    evidence: DxfEntityAliasEvidence,\n    source_id: &'static str,\n    source_reference: &'static str,\n    source_facts_sha256: &'static str,\n}\n\n",
    );
    output.push_str("impl DxfEntityAliasDescriptor {\n");
    output.push_str("    #[must_use]\n    pub const fn alias(self) -> DxfEntityAlias {\n        self.alias\n    }\n\n");
    output.push_str(
        "    #[must_use]\n    pub const fn id(self) -> &'static str {\n        self.id\n    }\n\n",
    );
    output.push_str("    #[must_use]\n    pub const fn dxf_name(self) -> &'static str {\n        self.dxf_name\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn topic(self) -> DxfEntityTopic {\n        self.topic\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn evidence(self) -> DxfEntityAliasEvidence {\n        self.evidence\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn source_id(self) -> &'static str {\n        self.source_id\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn source_reference(self) -> &'static str {\n        self.source_reference\n    }\n\n");
    output.push_str("    #[must_use]\n    pub const fn source_facts_sha256(self) -> &'static str {\n        self.source_facts_sha256\n    }\n}\n\n");
    output.push_str("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]\n");
    output.push_str("pub enum DxfEntityNameClassification {\n    Canonical(DxfEntityTopic),\n    Alias(DxfEntityAlias),\n    Unknown,\n}\n\n");
    output.push_str("impl DxfEntityNameClassification {\n");
    output.push_str(
        "    #[must_use]\n    pub fn topic(self) -> Option<DxfEntityTopic> {\n        match self {\n            Self::Canonical(topic) => Some(topic),\n            Self::Alias(alias) => alias.descriptor().map(|descriptor| descriptor.topic()),\n            Self::Unknown => None,\n        }\n    }\n\n",
    );
    output.push_str(
        "    #[must_use]\n    pub fn exact_name(self) -> Option<&'static str> {\n        match self {\n            Self::Canonical(topic) => topic.descriptor().map(|descriptor| descriptor.dxf_name()),\n            Self::Alias(alias) => alias.descriptor().map(|descriptor| descriptor.dxf_name()),\n            Self::Unknown => None,\n        }\n    }\n}\n\n",
    );
    output.push_str(
        "#[must_use]\npub fn classify_exact_dxf_entity_name(name: &[u8]) -> DxfEntityNameClassification {\n    if let Some(topic) = DxfEntityTopic::from_exact_name(name) {\n        DxfEntityNameClassification::Canonical(topic)\n    } else if let Some(alias) = DxfEntityAlias::from_exact_name(name) {\n        DxfEntityNameClassification::Alias(alias)\n    } else {\n        DxfEntityNameClassification::Unknown\n    }\n}\n\n",
    );
    output.push_str("pub const DXF_ENTITY_TOPIC_SCHEMA_SHA256: &str =\n");
    writeln!(output, "    {receipt:?};\n")?;
    output.push_str("pub const DXF_ENTITY_ALIAS_SCHEMA_SHA256: &str =\n");
    writeln!(output, "    {alias_receipt:?};\n")?;
    output.push_str("pub static DXF_ENTITY_TOPICS: &[DxfEntityTopicDescriptor] = &[\n");
    for topic in &entity_topics.topics {
        output.push_str("    DxfEntityTopicDescriptor {\n");
        writeln!(
            output,
            "        topic: DxfEntityTopic::{},",
            topic.id.to_ascii_uppercase()
        )?;
        writeln!(output, "        id: {:?},", topic.id)?;
        writeln!(output, "        dxf_name: {:?},", topic.dxf_name)?;
        writeln!(output, "        source_id: {:?},", source.id)?;
        writeln!(output, "        source_topic_id: {:?},", source.topic_id)?;
        writeln!(
            output,
            "        source_facts_sha256: {:?},",
            source.normalized_facts_sha256
        )?;
        output.push_str("    },\n");
    }
    output.push_str(
        "];\n\n#[must_use]\npub const fn dxf_entity_topics() -> &'static [DxfEntityTopicDescriptor] {\n    DXF_ENTITY_TOPICS\n}\n\n",
    );
    let sources: BTreeMap<_, _> = registry
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect();
    let topics: BTreeMap<_, _> = entity_topics
        .topics
        .iter()
        .map(|topic| (topic.id.as_str(), topic))
        .collect();
    output.push_str("pub static DXF_ENTITY_ALIASES: &[DxfEntityAliasDescriptor] = &[\n");
    for alias in &entity_aliases.aliases {
        let alias_source = sources.get(alias.source_id.as_str()).ok_or_else(|| {
            SchemaError::new(
                "SCHEMA_SOURCE_REF",
                ENTITY_OUTPUT_PATH,
                &alias.id,
                "validated alias source disappeared before rendering",
            )
        })?;
        let topic = topics.get(alias.topic_id.as_str()).ok_or_else(|| {
            SchemaError::new(
                "SCHEMA_ENTITY_ALIAS_TOPIC",
                ENTITY_OUTPUT_PATH,
                &alias.id,
                "validated alias topic disappeared before rendering",
            )
        })?;
        let evidence = match alias_source.evidence_kind {
            EvidenceKind::AliasList => "Normative",
            EvidenceKind::OracleInventory => "BehavioralOracle",
            EvidenceKind::Row | EvidenceKind::TopicList => {
                return Err(SchemaError::new(
                    "SCHEMA_ENTITY_ALIAS_SOURCE",
                    ENTITY_OUTPUT_PATH,
                    &alias.id,
                    "validated alias source kind changed before rendering",
                ));
            }
        };
        output.push_str("    DxfEntityAliasDescriptor {\n");
        writeln!(
            output,
            "        alias: DxfEntityAlias::{},",
            alias.id.to_ascii_uppercase()
        )?;
        writeln!(output, "        id: {:?},", alias.id)?;
        writeln!(output, "        dxf_name: {:?},", alias.dxf_name)?;
        writeln!(
            output,
            "        topic: DxfEntityTopic::{},",
            topic.id.to_ascii_uppercase()
        )?;
        writeln!(
            output,
            "        evidence: DxfEntityAliasEvidence::{evidence},"
        )?;
        writeln!(output, "        source_id: {:?},", alias_source.id)?;
        writeln!(
            output,
            "        source_reference: {:?},",
            alias_source.topic_id
        )?;
        writeln!(
            output,
            "        source_facts_sha256: {:?},",
            alias_source.normalized_facts_sha256
        )?;
        output.push_str("    },\n");
    }
    output.push_str(
        "];\n\n#[must_use]\npub const fn dxf_entity_aliases() -> &'static [DxfEntityAliasDescriptor] {\n    DXF_ENTITY_ALIASES\n}\n",
    );
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
        EvidenceKind::AliasList | EvidenceKind::OracleInventory => false,
        EvidenceKind::Row => row_evidence_matches(field),
        EvidenceKind::TopicList => false,
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

fn check_output(target: &Path, expected: &str, output_path: &str) -> Result<(), SchemaError> {
    let actual = fs::read_to_string(target)
        .map_err(|error| SchemaError::new("SCHEMA_IO", output_path, "output", error.to_string()))?;
    if actual != expected {
        return Err(SchemaError::new(
            "SCHEMA_OUTPUT_DIFF",
            output_path,
            "output",
            "committed generated Rust differs; run --write and review the diff",
        ));
    }
    Ok(())
}

fn write_output(target: &Path, output: &str, output_path: &str) -> Result<(), SchemaError> {
    if fs::read_to_string(target).is_ok_and(|current| current == output) {
        return Ok(());
    }
    let temporary = target.with_extension("rs.tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            SchemaError::new("SCHEMA_IO", output_path, "temporary", error.to_string())
        })?;
    let write_result = file
        .write_all(output.as_bytes())
        .and_then(|()| file.sync_all());
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(SchemaError::new(
            "SCHEMA_IO",
            output_path,
            "temporary",
            error.to_string(),
        ));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        let _ = fs::remove_file(&temporary);
        return Err(SchemaError::new(
            "SCHEMA_IO",
            output_path,
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

fn valid_entity_id(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
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

fn valid_source_reference(source: &SchemaSource) -> bool {
    match source.evidence_kind {
        EvidenceKind::OracleInventory => {
            source
                .topic_id
                .strip_prefix("SHA256-")
                .is_some_and(|receipt| {
                    receipt.len() == 64
                        && receipt
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
                })
        }
        EvidenceKind::AliasList | EvidenceKind::Row | EvidenceKind::TopicList => {
            valid_topic_id(&source.topic_id)
        }
    }
}

fn valid_dxf_name(value: &str) -> bool {
    value.starts_with('$')
        && value.len() > 1
        && value.as_bytes()[1..]
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn valid_entity_dxf_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl From<fmt::Error> for SchemaError {
    fn from(error: fmt::Error) -> Self {
        Self::new(
            "SCHEMA_RENDER",
            HEADER_OUTPUT_PATH,
            "output",
            error.to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{
        EXPECTED_ENTITY_ALIAS_COUNT, EvidenceKind, MANIFEST_PATH, SchemaManifest, StorageKind,
        evidence_matches, load_entity_aliases, load_entity_topics, load_schema,
        normalized_alias_receipt, normalized_entity_receipt, normalized_receipt,
        render_entity_registry, render_registry, validate_entity_aliases, validate_entity_topics,
        validate_schema, validate_wire_shape,
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
    fn reviewed_entity_inventory_is_complete_and_deterministic() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, _) = load_schema(&root)?;
        let entity_topics = load_entity_topics(&root, &manifest)?;
        let source = validate_entity_topics(&manifest, &sources, &entity_topics)?;
        let entity_aliases = load_entity_aliases(&root, &manifest)?;
        validate_entity_aliases(&manifest, &sources, &entity_topics, &entity_aliases)?;
        assert_eq!(entity_topics.topics.len(), 45);
        assert_eq!(entity_topics.topics[0].dxf_name, "3DFACE");
        assert_eq!(entity_topics.topics[44].dxf_name, "XLINE");
        let first = normalized_entity_receipt(&entity_topics, source)?;
        let second = normalized_entity_receipt(&entity_topics, source)?;
        let alias_receipt = normalized_alias_receipt(&entity_aliases, &sources)?;
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert_eq!(
            render_entity_registry(
                &entity_topics,
                source,
                &first,
                &entity_aliases,
                &sources,
                &alias_receipt,
            )?,
            render_entity_registry(
                &entity_topics,
                source,
                &second,
                &entity_aliases,
                &sources,
                &alias_receipt,
            )?
        );
        Ok(())
    }

    #[test]
    fn reviewed_entity_aliases_are_complete_and_fail_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, _) = load_schema(&root)?;
        let topics = load_entity_topics(&root, &manifest)?;
        let aliases = load_entity_aliases(&root, &manifest)?;
        validate_entity_aliases(&manifest, &sources, &topics, &aliases)?;
        assert_eq!(aliases.aliases.len(), EXPECTED_ENTITY_ALIAS_COUNT);
        assert_eq!(aliases.aliases[0].dxf_name, "MPOLYGON");
        assert_eq!(aliases.aliases[13].dxf_name, "SWEPTSURFACE");

        let mut duplicate = load_entity_aliases(&root, &manifest)?;
        duplicate.aliases[1].dxf_name = duplicate.aliases[0].dxf_name.clone();
        let error = validate_entity_aliases(&manifest, &sources, &topics, &duplicate)
            .err()
            .ok_or("duplicate entity alias unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_ENTITY_ALIAS_NAME");

        let mut bad_topic = load_entity_aliases(&root, &manifest)?;
        bad_topic.aliases[0].topic_id = "future_topic".to_string();
        let error = validate_entity_aliases(&manifest, &sources, &topics, &bad_topic)
            .err()
            .ok_or("unknown alias topic unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_ENTITY_ALIAS_TOPIC");

        let mut stale = load_entity_aliases(&root, &manifest)?;
        stale.aliases[0].dxf_name = "MPOLYGON2".to_string();
        let error = validate_entity_aliases(&manifest, &sources, &topics, &stale)
            .err()
            .ok_or("stale alias source receipt unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_ENTITY_ALIAS_SOURCE_RECEIPT");
        Ok(())
    }

    #[test]
    fn duplicate_entity_topic_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, _) = load_schema(&root)?;
        let mut entity_topics = load_entity_topics(&root, &manifest)?;
        entity_topics.topics[1].id = entity_topics.topics[0].id.clone();
        let error = validate_entity_topics(&manifest, &sources, &entity_topics)
            .err()
            .ok_or("duplicate entity topic unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_ENTITY_ID");

        let mut entity_topics = load_entity_topics(&root, &manifest)?;
        entity_topics.topics[1].dxf_name = entity_topics.topics[0].dxf_name.clone();
        let error = validate_entity_topics(&manifest, &sources, &entity_topics)
            .err()
            .ok_or("duplicate entity name unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_ENTITY_NAME");
        Ok(())
    }

    #[test]
    fn stale_entity_source_receipt_fails_closed() -> Result<(), Box<dyn Error>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (manifest, sources, _) = load_schema(&root)?;
        let mut entity_topics = load_entity_topics(&root, &manifest)?;
        entity_topics.topics[0].dxf_name = "3DFACE2".to_string();
        let error = validate_entity_topics(&manifest, &sources, &entity_topics)
            .err()
            .ok_or("stale entity source receipt unexpectedly passed")?;
        assert_eq!(error.code, "SCHEMA_ENTITY_SOURCE_RECEIPT");
        Ok(())
    }

    #[test]
    fn unknown_manifest_key_is_rejected() {
        let json = r#"{"schema_version":"dxf.v1","sources":"sources.json","entity_topics":"entity_topics.json","entity_aliases":"entity_aliases.json","families":[],"typo":true}"#;
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
        duplicate.id = "zz.oracle.copy".to_string();
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
